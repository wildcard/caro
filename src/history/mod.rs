//! History module for tracking request/response memory
//!
//! Provides persistent storage for all user requests, inferred commands,
//! execution results, and performance metrics. Implements size-based rotation
//! to stay under the configured limit (default 100MB).

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

mod manifest;
mod models;
mod summarizer;

pub use manifest::HistoryManifestManager;
pub use models::{
    ExecutionOutcome, HistoryConfig, HistoryManifest, HistoryStats, PromptVersion, RequestRecord,
};
pub use summarizer::OutputSummarizer;

/// History-related errors
#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("Failed to create history directory: {0}")]
    DirectoryCreation(String),

    #[error("Failed to read history: {0}")]
    ReadError(String),

    #[error("Failed to write history: {0}")]
    WriteError(String),

    #[error("Manifest error: {0}")]
    ManifestError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),
}

/// Manages request history storage with size-based rotation
pub struct HistoryManager {
    history_dir: PathBuf,
    manifest: Arc<RwLock<HistoryManifestManager>>,
    config: HistoryConfig,
    summarizer: OutputSummarizer,
}

impl HistoryManager {
    /// Create a new HistoryManager with default XDG data directory
    pub fn new() -> Result<Self, HistoryError> {
        let history_dir = dirs::data_dir()
            .ok_or_else(|| {
                HistoryError::DirectoryCreation("Could not determine data directory".to_string())
            })?
            .join("cmdai")
            .join("history");

        Self::with_directory(history_dir, HistoryConfig::default())
    }

    /// Create a HistoryManager with custom directory and configuration
    pub fn with_directory(history_dir: PathBuf, config: HistoryConfig) -> Result<Self, HistoryError> {
        // Create history directory if it doesn't exist
        if !history_dir.exists() {
            std::fs::create_dir_all(&history_dir)?;
        }

        if !history_dir.is_dir() {
            return Err(HistoryError::DirectoryCreation(format!(
                "History path is not a directory: {}",
                history_dir.display()
            )));
        }

        let manifest = HistoryManifestManager::new(history_dir.clone(), config.clone())?;

        Ok(Self {
            history_dir,
            manifest: Arc::new(RwLock::new(manifest)),
            config,
            summarizer: OutputSummarizer::new(),
        })
    }

    /// Record a new request with all associated metadata
    pub async fn record_request(&self, record: RequestRecord) -> Result<String, HistoryError> {
        let record_id = record.id.clone();

        // Summarize output if present and too large
        let mut record = record;
        if let Some(ref output) = record.execution_output {
            if output.len() > self.config.max_output_size_bytes {
                record.execution_output_summarized = Some(self.summarizer.summarize(output));
                // Keep a truncated version of the original
                record.execution_output = Some(
                    output
                        .chars()
                        .take(self.config.max_output_size_bytes / 2)
                        .collect::<String>()
                        + "\n... [truncated, see execution_output_summarized] ...",
                );
            }
        }

        // Calculate record size
        let record_json = serde_json::to_string(&record)
            .map_err(|e| HistoryError::SerializationError(e.to_string()))?;
        let record_size = record_json.len() as u64;

        // Write record to individual file
        let record_path = self.history_dir.join(format!("{}.json", record.id));
        tokio::fs::write(&record_path, &record_json).await?;

        // Get records to remove (if any) while holding the lock briefly
        let records_to_remove: Vec<String> = {
            let mut manifest = self
                .manifest
                .write()
                .map_err(|e| HistoryError::ManifestError(format!("Lock error: {}", e)))?;

            manifest.add_record(&record.id, record_size, record.timestamp)?;

            // Check if we need to rotate (size limit exceeded)
            if manifest.total_size() > self.config.max_history_size_bytes {
                manifest.get_oldest_records_for_cleanup(
                    manifest.total_size() - self.config.max_history_size_bytes,
                )
            } else {
                Vec::new()
            }
        }; // Lock released here

        // Remove old records outside the lock
        for old_record_id in &records_to_remove {
            let old_path = self.history_dir.join(format!("{}.json", old_record_id));
            if old_path.exists() {
                tokio::fs::remove_file(&old_path).await.ok();
            }
            // Remove from manifest (get lock again briefly)
            let mut manifest = self
                .manifest
                .write()
                .map_err(|e| HistoryError::ManifestError(format!("Lock error: {}", e)))?;
            manifest.remove_record(old_record_id)?;
        }

        Ok(record_id)
    }

    /// Get a specific record by ID
    pub async fn get_record(&self, record_id: &str) -> Result<RequestRecord, HistoryError> {
        let record_path = self.history_dir.join(format!("{}.json", record_id));

        if !record_path.exists() {
            return Err(HistoryError::RecordNotFound(record_id.to_string()));
        }

        let contents = tokio::fs::read_to_string(&record_path).await?;
        let record: RequestRecord = serde_json::from_str(&contents)
            .map_err(|e| HistoryError::SerializationError(e.to_string()))?;

        // Update last accessed time in manifest
        {
            let mut manifest = self
                .manifest
                .write()
                .map_err(|e| HistoryError::ManifestError(format!("Lock error: {}", e)))?;
            manifest.update_last_accessed(record_id)?;
        }

        Ok(record)
    }

    /// List recent records (most recent first)
    pub fn list_records(&self, limit: usize) -> Result<Vec<String>, HistoryError> {
        let manifest = self
            .manifest
            .read()
            .map_err(|e| HistoryError::ManifestError(format!("Lock error: {}", e)))?;

        Ok(manifest.get_recent_records(limit))
    }

    /// Search records by user input pattern
    pub async fn search_by_input(&self, pattern: &str) -> Result<Vec<RequestRecord>, HistoryError> {
        let record_ids = self.list_records(1000)?;
        let mut matching = Vec::new();

        for record_id in record_ids {
            if let Ok(record) = self.get_record(&record_id).await {
                if record.user_input.to_lowercase().contains(&pattern.to_lowercase()) {
                    matching.push(record);
                }
            }
        }

        Ok(matching)
    }

    /// Get history statistics
    pub fn stats(&self) -> Result<HistoryStats, HistoryError> {
        let manifest = self
            .manifest
            .read()
            .map_err(|e| HistoryError::ManifestError(format!("Lock error: {}", e)))?;

        Ok(manifest.stats())
    }

    /// Clear all history
    pub async fn clear_history(&self) -> Result<(), HistoryError> {
        let record_ids = self.list_records(usize::MAX)?;

        // Delete all record files
        for record_id in &record_ids {
            let record_path = self.history_dir.join(format!("{}.json", record_id));
            if record_path.exists() {
                tokio::fs::remove_file(&record_path).await.ok();
            }
        }

        // Clear manifest
        let mut manifest = self
            .manifest
            .write()
            .map_err(|e| HistoryError::ManifestError(format!("Lock error: {}", e)))?;
        manifest.clear()?;

        Ok(())
    }

    /// Get the current prompt version being used
    pub fn get_current_prompt_version(&self) -> Result<PromptVersion, HistoryError> {
        let manifest = self
            .manifest
            .read()
            .map_err(|e| HistoryError::ManifestError(format!("Lock error: {}", e)))?;

        Ok(manifest.get_current_prompt_version())
    }

    /// Register a new prompt version
    pub fn register_prompt_version(&self, version: PromptVersion) -> Result<(), HistoryError> {
        let mut manifest = self
            .manifest
            .write()
            .map_err(|e| HistoryError::ManifestError(format!("Lock error: {}", e)))?;

        manifest.register_prompt_version(version)?;
        Ok(())
    }

    /// Get history directory path
    pub fn history_dir(&self) -> &PathBuf {
        &self.history_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_history_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HistoryManager::with_directory(
            temp_dir.path().to_path_buf(),
            HistoryConfig::default(),
        );
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_record_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HistoryManager::with_directory(
            temp_dir.path().to_path_buf(),
            HistoryConfig::default(),
        )
        .unwrap();

        let record = RequestRecord::new(
            "list all files".to_string(),
            "ls -la".to_string(),
            "0.1.0".to_string(),
            "1.0.0".to_string(),
        );
        let record_id = record.id.clone();

        manager.record_request(record).await.unwrap();

        let retrieved = manager.get_record(&record_id).await.unwrap();
        assert_eq!(retrieved.user_input, "list all files");
        assert_eq!(retrieved.inferred_command, "ls -la");
    }

    #[tokio::test]
    async fn test_size_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let config = HistoryConfig {
            max_history_size_bytes: 1000, // Very small limit for testing
            ..Default::default()
        };
        let manager =
            HistoryManager::with_directory(temp_dir.path().to_path_buf(), config).unwrap();

        // Add multiple records to trigger rotation
        for i in 0..10 {
            let record = RequestRecord::new(
                format!("test input {}", i),
                format!("echo {}", i),
                "0.1.0".to_string(),
                "1.0.0".to_string(),
            );
            manager.record_request(record).await.unwrap();
        }

        // Should have rotated old records
        let stats = manager.stats().unwrap();
        assert!(stats.total_size_bytes <= 1000 || stats.total_records < 10);
    }
}
