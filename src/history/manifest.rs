//! Manifest management for tracking history records

use super::models::{HistoryConfig, HistoryManifest, HistoryStats, PromptVersion, RecordMetadata};
use super::HistoryError;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Manages the history manifest file
pub struct HistoryManifestManager {
    manifest_path: PathBuf,
    manifest: HistoryManifest,
}

impl HistoryManifestManager {
    /// Create a new HistoryManifestManager
    pub fn new(history_dir: PathBuf, config: HistoryConfig) -> Result<Self, HistoryError> {
        let manifest_path = history_dir.join("manifest.json");

        // Load existing manifest or create new one
        let manifest = if manifest_path.exists() {
            Self::load_manifest(&manifest_path)?
        } else {
            HistoryManifest::new(config.max_history_size_bytes)
        };

        let manifest_exists = manifest_path.exists();

        let mut manager = Self {
            manifest_path,
            manifest,
        };

        // Update max size from config
        manager.manifest.max_size_bytes = config.max_history_size_bytes;

        // Save initial manifest if it didn't exist
        if !manifest_exists {
            manager.save()?;
        }

        Ok(manager)
    }

    /// Load manifest from disk
    fn load_manifest(path: &PathBuf) -> Result<HistoryManifest, HistoryError> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e| HistoryError::ManifestError(format!("Failed to parse manifest: {}", e)))
    }

    /// Save manifest to disk
    pub fn save(&mut self) -> Result<(), HistoryError> {
        self.manifest.last_updated = Utc::now();
        self.update_stats();

        let contents = serde_json::to_string_pretty(&self.manifest).map_err(|e| {
            HistoryError::ManifestError(format!("Failed to serialize manifest: {}", e))
        })?;

        std::fs::write(&self.manifest_path, contents)?;

        Ok(())
    }

    /// Add a record to the manifest
    pub fn add_record(
        &mut self,
        record_id: &str,
        size_bytes: u64,
        created_at: DateTime<Utc>,
    ) -> Result<(), HistoryError> {
        let metadata = RecordMetadata {
            size_bytes,
            created_at,
            last_accessed: Utc::now(),
        };

        self.manifest.records.insert(record_id.to_string(), metadata);
        self.manifest.total_size_bytes += size_bytes;
        self.manifest.total_records_created += 1;

        self.save()?;

        Ok(())
    }

    /// Remove a record from the manifest
    pub fn remove_record(&mut self, record_id: &str) -> Result<(), HistoryError> {
        if let Some(metadata) = self.manifest.records.remove(record_id) {
            self.manifest.total_size_bytes = self
                .manifest
                .total_size_bytes
                .saturating_sub(metadata.size_bytes);
            self.save()?;
        }

        Ok(())
    }

    /// Update last accessed time for a record
    pub fn update_last_accessed(&mut self, record_id: &str) -> Result<(), HistoryError> {
        if let Some(metadata) = self.manifest.records.get_mut(record_id) {
            metadata.last_accessed = Utc::now();
            self.save()?;
        }

        Ok(())
    }

    /// Get records sorted by creation time (most recent first)
    pub fn get_recent_records(&self, limit: usize) -> Vec<String> {
        let mut records: Vec<_> = self.manifest.records.iter().collect();
        records.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
        records
            .into_iter()
            .take(limit)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get oldest records that should be removed to free up space
    pub fn get_oldest_records_for_cleanup(&self, bytes_to_free: u64) -> Vec<String> {
        let mut records: Vec<_> = self.manifest.records.iter().collect();
        // Sort by last accessed (oldest first for LRU)
        records.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));

        let mut freed = 0u64;
        let mut to_remove = Vec::new();

        for (id, metadata) in records {
            if freed >= bytes_to_free {
                break;
            }
            freed += metadata.size_bytes;
            to_remove.push(id.clone());
        }

        to_remove
    }

    /// Clear all records from the manifest
    pub fn clear(&mut self) -> Result<(), HistoryError> {
        self.manifest.records.clear();
        self.manifest.total_size_bytes = 0;
        self.save()?;

        Ok(())
    }

    /// Get total size of all records
    pub fn total_size(&self) -> u64 {
        self.manifest.total_size_bytes
    }

    /// Get total number of records
    pub fn total_records(&self) -> usize {
        self.manifest.records.len()
    }

    /// Get current prompt version
    pub fn get_current_prompt_version(&self) -> PromptVersion {
        self.manifest
            .prompt_versions
            .iter()
            .find(|v| v.is_active)
            .cloned()
            .unwrap_or_else(|| PromptVersion::new("1.0.0".to_string(), "default".to_string()))
    }

    /// Register a new prompt version
    pub fn register_prompt_version(&mut self, version: PromptVersion) -> Result<(), HistoryError> {
        // Deactivate all existing versions
        for v in &mut self.manifest.prompt_versions {
            v.is_active = false;
        }

        // Add the new version as active
        let mut new_version = version;
        new_version.is_active = true;
        self.manifest.prompt_versions.push(new_version);

        self.save()?;

        Ok(())
    }

    /// Get history statistics
    pub fn stats(&self) -> HistoryStats {
        self.manifest.stats.clone()
    }

    /// Update statistics based on current records
    fn update_stats(&mut self) {
        let stats = &mut self.manifest.stats;

        stats.total_records = self.manifest.records.len();
        stats.total_size_bytes = self.manifest.total_size_bytes;

        // Find oldest and newest records
        let mut oldest: Option<DateTime<Utc>> = None;
        let mut newest: Option<DateTime<Utc>> = None;

        for metadata in self.manifest.records.values() {
            match oldest {
                None => oldest = Some(metadata.created_at),
                Some(o) if metadata.created_at < o => oldest = Some(metadata.created_at),
                _ => {}
            }
            match newest {
                None => newest = Some(metadata.created_at),
                Some(n) if metadata.created_at > n => newest = Some(metadata.created_at),
                _ => {}
            }
        }

        stats.oldest_record = oldest;
        stats.newest_record = newest;
    }

    /// Increment execution statistics (called when recording execution results)
    pub fn increment_stats(
        &mut self,
        success: bool,
        blocked: bool,
        not_executed: bool,
        inference_time_ms: u64,
    ) -> Result<(), HistoryError> {
        let stats = &mut self.manifest.stats;

        if blocked {
            stats.blocked_by_safety += 1;
        } else if not_executed {
            stats.not_executed += 1;
        } else if success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }

        // Update average inference time (exponential moving average)
        let total_executions = stats.successful_executions
            + stats.failed_executions
            + stats.not_executed
            + stats.blocked_by_safety;

        if total_executions > 0 {
            let alpha = 0.1; // Smoothing factor
            stats.avg_inference_time_ms =
                alpha * (inference_time_ms as f64) + (1.0 - alpha) * stats.avg_inference_time_ms;
        } else {
            stats.avg_inference_time_ms = inference_time_ms as f64;
        }

        self.save()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_manifest_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager =
            HistoryManifestManager::new(temp_dir.path().to_path_buf(), HistoryConfig::default());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_add_and_remove_record() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager =
            HistoryManifestManager::new(temp_dir.path().to_path_buf(), HistoryConfig::default())
                .unwrap();

        manager
            .add_record("test-id", 100, Utc::now())
            .unwrap();
        assert_eq!(manager.total_records(), 1);
        assert_eq!(manager.total_size(), 100);

        manager.remove_record("test-id").unwrap();
        assert_eq!(manager.total_records(), 0);
        assert_eq!(manager.total_size(), 0);
    }

    #[test]
    fn test_get_recent_records() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager =
            HistoryManifestManager::new(temp_dir.path().to_path_buf(), HistoryConfig::default())
                .unwrap();

        for i in 0..5 {
            manager
                .add_record(&format!("record-{}", i), 50, Utc::now())
                .unwrap();
        }

        let recent = manager.get_recent_records(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_cleanup_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager =
            HistoryManifestManager::new(temp_dir.path().to_path_buf(), HistoryConfig::default())
                .unwrap();

        // Add records with different sizes
        manager
            .add_record("small", 100, Utc::now())
            .unwrap();
        manager
            .add_record("medium", 500, Utc::now())
            .unwrap();
        manager
            .add_record("large", 1000, Utc::now())
            .unwrap();

        // Request cleanup of 200 bytes
        let to_remove = manager.get_oldest_records_for_cleanup(200);
        assert!(!to_remove.is_empty());
    }

    #[test]
    fn test_prompt_version_management() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager =
            HistoryManifestManager::new(temp_dir.path().to_path_buf(), HistoryConfig::default())
                .unwrap();

        let new_version = PromptVersion::new("2.0.0".to_string(), "updated".to_string())
            .with_description("Added new features".to_string());

        manager.register_prompt_version(new_version).unwrap();

        let current = manager.get_current_prompt_version();
        assert_eq!(current.version, "2.0.0");
        assert!(current.is_active);
    }
}
