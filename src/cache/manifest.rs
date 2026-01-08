//! Manifest management for tracking cached models

use crate::models::{CacheManifest, CachedModel};
use chrono::Utc;
use fd_lock::RwLock;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use super::CacheError;

/// Manages the cache manifest file
pub struct ManifestManager {
    manifest_path: PathBuf,
    manifest: CacheManifest,
}

impl ManifestManager {
    /// Create a new ManifestManager
    pub fn new(cache_dir: PathBuf) -> Result<Self, CacheError> {
        let manifest_path = cache_dir.join("manifest.json");

        // Load existing manifest or create new one
        let manifest = if manifest_path.exists() {
            Self::load_manifest(&manifest_path)?
        } else {
            CacheManifest {
                version: "1.0".to_string(),
                models: std::collections::HashMap::new(),
                total_size_bytes: 0,
                max_cache_size_bytes: 10 * 1024 * 1024 * 1024, // 10GB default
                last_updated: Utc::now(),
            }
        };

        let manifest_exists = manifest_path.exists();

        let mut manager = Self {
            manifest_path,
            manifest,
        };

        // Save initial manifest if it didn't exist
        if !manifest_exists {
            manager.save()?;
        }

        Ok(manager)
    }

    /// Load manifest from disk
    fn load_manifest(path: &PathBuf) -> Result<CacheManifest, CacheError> {
        let contents = std::fs::read_to_string(path)?;

        // Handle empty file (can happen in concurrent scenarios)
        if contents.trim().is_empty() {
            return Ok(CacheManifest {
                version: "1.0".to_string(),
                models: std::collections::HashMap::new(),
                total_size_bytes: 0,
                max_cache_size_bytes: 10 * 1024 * 1024 * 1024, // 10GB default
                last_updated: Utc::now(),
            });
        }

        serde_json::from_str(&contents)
            .map_err(|e| CacheError::ManifestError(format!("Failed to parse manifest: {}", e)))
    }

    /// Save manifest to disk with file locking
    ///
    /// This method provides atomic updates by:
    /// 1. Acquiring an exclusive write lock on the manifest file
    /// 2. Writing the updated manifest
    /// 3. Flushing to disk
    /// 4. Releasing the lock (automatic on drop)
    pub fn save(&mut self) -> Result<(), CacheError> {
        self.manifest.last_updated = Utc::now();

        let contents = serde_json::to_string_pretty(&self.manifest).map_err(|e| {
            CacheError::ManifestError(format!("Failed to serialize manifest: {}", e))
        })?;

        // Open file with write access
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.manifest_path)?;

        // Acquire exclusive write lock (blocking)
        // This will wait if another process holds the lock
        let mut lock = RwLock::new(file);
        let mut guard = lock.write().map_err(|e| {
            CacheError::ManifestError(format!("Failed to acquire manifest lock: {}", e))
        })?;

        // Write manifest with lock held
        guard.write_all(contents.as_bytes())?;
        guard.flush()?;

        // Lock is automatically released when guard is dropped
        Ok(())
    }

    /// Atomically update the manifest by applying a modification function
    ///
    /// This ensures thread-safe manifest updates by:
    /// 1. Acquiring an exclusive write lock on the manifest file
    /// 2. Reloading the manifest from disk (to get latest state)
    /// 3. Applying the modification
    /// 4. Writing the updated manifest (still holding the lock)
    /// 5. Releasing the lock (automatic on drop)
    pub fn atomic_update<F>(&mut self, update_fn: F) -> Result<(), CacheError>
    where
        F: FnOnce(&mut CacheManifest) -> Result<(), CacheError>,
    {
        // Open file for read/write to acquire lock
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.manifest_path)?;

        // Acquire exclusive write lock FIRST (before reading)
        // This ensures the read-modify-write cycle is atomic
        let mut lock = RwLock::new(file);
        let mut _guard = lock.write().map_err(|e| {
            CacheError::ManifestError(format!("Failed to acquire manifest lock: {}", e))
        })?;

        // Now that we have the lock, reload manifest from the locked file
        use std::io::Read;
        use std::ops::DerefMut;

        let mut contents = String::new();
        let file_handle = _guard.deref_mut();

        // Read current content (if any)
        file_handle.read_to_string(&mut contents)?;

        // Parse manifest if file has content
        if !contents.is_empty() {
            self.manifest = serde_json::from_str(&contents)
                .map_err(|e| CacheError::ManifestError(format!("Failed to parse manifest: {}", e)))?;
        }

        // Apply update function
        update_fn(&mut self.manifest)?;

        // Update timestamp
        self.manifest.last_updated = Utc::now();

        // Serialize
        let contents = serde_json::to_string_pretty(&self.manifest).map_err(|e| {
            CacheError::ManifestError(format!("Failed to serialize manifest: {}", e))
        })?;

        // Write to file (lock is still held)
        // We need to write to the file directly since we already have the lock
        use std::io::Seek;

        // Get mutable reference to the file through the guard
        let file_handle = _guard.deref_mut();
        file_handle.set_len(0)?; // Truncate
        file_handle.seek(std::io::SeekFrom::Start(0))?; // Rewind
        file_handle.write_all(contents.as_bytes())?;
        file_handle.flush()?;

        // Lock is automatically released when _guard is dropped
        Ok(())
    }

    /// Check if a model is in the manifest
    pub fn has_model(&self, model_id: &str) -> bool {
        self.manifest.models.contains_key(model_id)
    }

    /// Get a model from the manifest
    pub fn get_model(&self, model_id: &str) -> Option<CachedModel> {
        self.manifest.models.get(model_id).cloned()
    }

    /// Add a model to the manifest
    pub fn add_model(
        &mut self,
        model_id: String,
        cached_model: CachedModel,
    ) -> Result<(), CacheError> {
        // Update total size
        self.manifest.total_size_bytes += cached_model.size_bytes;

        // Add model
        self.manifest.models.insert(model_id, cached_model);

        // Check if we need LRU cleanup
        if self.manifest.total_size_bytes > self.manifest.max_cache_size_bytes {
            self.manifest.cleanup_lru();
            self.recalculate_total_size();
        }

        self.save()?;

        Ok(())
    }

    /// Remove a model from the manifest
    pub fn remove_model(&mut self, model_id: &str) -> Result<(), CacheError> {
        if let Some(cached_model) = self.manifest.models.remove(model_id) {
            self.manifest.total_size_bytes = self
                .manifest
                .total_size_bytes
                .saturating_sub(cached_model.size_bytes);
            self.save()?;
        }

        Ok(())
    }

    /// Clear all models from the manifest
    pub fn clear(&mut self) -> Result<(), CacheError> {
        self.manifest.models.clear();
        self.manifest.total_size_bytes = 0;
        self.save()?;

        Ok(())
    }

    /// Update the last accessed time for a model
    pub fn update_last_accessed(&mut self, model_id: &str) -> Result<(), CacheError> {
        if let Some(cached_model) = self.manifest.models.get_mut(model_id) {
            cached_model.last_accessed = Utc::now();
            self.save()?;
        }

        Ok(())
    }

    /// List all model IDs
    pub fn list_models(&self) -> Vec<String> {
        self.manifest.models.keys().cloned().collect()
    }

    /// Get total cache size
    pub fn total_size(&self) -> u64 {
        self.manifest.total_size_bytes
    }

    /// Recalculate total size from all models
    fn recalculate_total_size(&mut self) {
        self.manifest.total_size_bytes = self
            .manifest
            .models
            .values()
            .map(|model| model.size_bytes)
            .sum();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_manifest_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ManifestManager::new(temp_dir.path().to_path_buf());
        assert!(manifest.is_ok());
    }

    #[test]
    fn test_manifest_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        {
            let _manifest = ManifestManager::new(temp_dir.path().to_path_buf()).unwrap();
            assert!(manifest_path.exists());
        }

        // Manifest file should persist after drop
        assert!(manifest_path.exists());
    }

    #[test]
    fn test_has_model() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ManifestManager::new(temp_dir.path().to_path_buf()).unwrap();

        assert!(!manifest.has_model("test-model"));
    }

    #[test]
    fn test_list_models_empty() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ManifestManager::new(temp_dir.path().to_path_buf()).unwrap();

        let models = manifest.list_models();
        assert_eq!(models.len(), 0);
    }

    #[test]
    fn test_atomic_update_simple() {
        let temp_dir = TempDir::new().unwrap();
        let mut manifest = ManifestManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Use atomic_update to modify manifest
        let result = manifest.atomic_update(|manifest_data| {
            manifest_data.total_size_bytes = 12345;
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(manifest.manifest.total_size_bytes, 12345);
    }

    #[test]
    fn test_atomic_update_with_error() {
        let temp_dir = TempDir::new().unwrap();
        let mut manifest = ManifestManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Test error handling in atomic_update
        let result = manifest.atomic_update(|_manifest_data| {
            Err(CacheError::ManifestError("Test error".to_string()))
        });

        assert!(result.is_err());
        if let Err(CacheError::ManifestError(msg)) = result {
            assert_eq!(msg, "Test error");
        } else {
            panic!("Expected ManifestError");
        }
    }

    #[test]
    fn test_save_with_file_locking() {
        let temp_dir = TempDir::new().unwrap();
        let mut manifest = ManifestManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Modify and save
        manifest.manifest.total_size_bytes = 54321;
        let result = manifest.save();

        assert!(result.is_ok());

        // Reload and verify
        let manifest2 = ManifestManager::new(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(manifest2.manifest.total_size_bytes, 54321);
    }

    #[test]
    fn test_concurrent_updates_with_atomic_update() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();

        // Create initial manifest
        {
            let _manifest = ManifestManager::new(cache_dir.clone()).unwrap();
        }

        // Spawn multiple threads that each perform atomic updates
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let cache_dir = cache_dir.clone();
                thread::spawn(move || {
                    let mut manifest = ManifestManager::new(cache_dir).unwrap();

                    // Each thread adds a unique model
                    manifest.atomic_update(|manifest_data| {
                        let cached_model = CachedModel {
                            model_id: format!("model-{}", i),
                            path: PathBuf::from(format!("/fake/path/model-{}.bin", i)),
                            size_bytes: 1000,
                            checksum: format!("checksum-{}", i),
                            downloaded_at: Utc::now(),
                            last_accessed: Utc::now(),
                            version: None,
                        };

                        manifest_data.models.insert(format!("model-{}", i), cached_model);
                        manifest_data.total_size_bytes += 1000;
                        Ok(())
                    }).unwrap();
                })
            })
            .collect();

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all updates were applied
        let final_manifest = ManifestManager::new(cache_dir).unwrap();
        assert_eq!(final_manifest.manifest.models.len(), 5);
        assert_eq!(final_manifest.manifest.total_size_bytes, 5000);

        // Verify all models exist
        for i in 0..5 {
            assert!(final_manifest.has_model(&format!("model-{}", i)));
        }
    }

    #[test]
    fn test_atomic_update_reloads_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();

        // Create manifest and make initial modification
        {
            let mut manifest = ManifestManager::new(cache_dir.clone()).unwrap();
            manifest.manifest.total_size_bytes = 100;
            manifest.save().unwrap();
        }

        // Create second manifest instance that will have stale data
        let mut manifest2 = ManifestManager::new(cache_dir.clone()).unwrap();
        manifest2.manifest.total_size_bytes = 999; // Local modification (not saved)

        // Atomic update should reload from disk (getting 100) and then apply update
        manifest2.atomic_update(|manifest_data| {
            // Should see the reloaded value (100), not the local value (999)
            assert_eq!(manifest_data.total_size_bytes, 100);
            manifest_data.total_size_bytes += 50;
            Ok(())
        }).unwrap();

        // Verify final value
        assert_eq!(manifest2.manifest.total_size_bytes, 150);
    }
}
