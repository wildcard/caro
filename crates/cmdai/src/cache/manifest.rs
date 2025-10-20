//! Manifest management for tracking cached models

use crate::models::{CacheManifest, CachedModel};
use chrono::Utc;
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
        serde_json::from_str(&contents)
            .map_err(|e| CacheError::ManifestError(format!("Failed to parse manifest: {}", e)))
    }

    /// Save manifest to disk
    pub fn save(&mut self) -> Result<(), CacheError> {
        self.manifest.last_updated = Utc::now();

        let contents = serde_json::to_string_pretty(&self.manifest).map_err(|e| {
            CacheError::ManifestError(format!("Failed to serialize manifest: {}", e))
        })?;

        std::fs::write(&self.manifest_path, contents)?;

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
}
