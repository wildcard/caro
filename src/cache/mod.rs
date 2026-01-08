//! Cache module for managing Hugging Face model downloads and local storage
//!
//! Provides LRU cache management, integrity validation, and offline support.

use crate::models::CachedModel;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

mod manifest;
pub use manifest::ManifestManager;

mod http_client;
pub use http_client::{HfHubClient, HttpClientError};

mod download;
pub use download::download_file;

mod checksum;
pub use checksum::StreamingHasher;

mod progress;
pub use progress::DownloadProgress;

/// Cache-related errors
///
/// Provides comprehensive error handling for cache operations with user-friendly messages
/// and proper error chaining for debugging.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// Download operation failed
    #[error("Download failed: {0}. Please check your internet connection and try again.")]
    DownloadFailed(String),

    /// Network error during HTTP request
    #[error("Network error: {0}. This may be due to connectivity issues or server unavailability.")]
    NetworkError(String),

    /// File checksum mismatch indicating corruption
    #[error("File integrity check failed for {model_id}.\nExpected checksum: {expected}\nActual checksum: {actual}\nThe downloaded file may be corrupted. Please try downloading again.")]
    ChecksumMismatch {
        model_id: String,
        expected: String,
        actual: String,
    },

    /// Server does not support resume (HTTP Range requests)
    #[error("Resume not supported: The server does not support resuming downloads.\nYou'll need to download the complete file from the beginning.")]
    ResumeNotSupported,

    /// Authentication required but token not provided
    #[error("Authentication required: Please set the HF_TOKEN environment variable.\nYou can obtain a token from https://huggingface.co/settings/tokens")]
    AuthenticationRequired,

    /// Requested model not found in cache
    #[error("Model '{0}' not found in cache. Use the download command to fetch it first.")]
    ModelNotFound(String),

    /// I/O operation failed
    #[error("File system error: {0}")]
    IoError(#[from] std::io::Error),

    /// Cache manifest operation failed
    #[error("Cache manifest error: {0}. The cache may be in an inconsistent state.")]
    ManifestError(String),

    /// Cache directory issue
    #[error("Cache directory error: {0}. Please ensure the cache directory is writable.")]
    DirectoryError(String),
}

impl From<reqwest::Error> for CacheError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_connect() || err.is_timeout() {
            CacheError::NetworkError(format!(
                "Failed to connect to server: {}. Check your internet connection.",
                err
            ))
        } else if err.is_status() {
            let status = err.status().map(|s| s.as_u16()).unwrap_or(0);
            match status {
                401 | 403 => CacheError::AuthenticationRequired,
                404 => CacheError::DownloadFailed("Resource not found on server".to_string()),
                416 => CacheError::ResumeNotSupported,
                500..=599 => CacheError::NetworkError(format!(
                    "Server error ({}). The server may be experiencing issues. Please try again later.",
                    status
                )),
                _ => CacheError::NetworkError(format!("HTTP error: {}", err)),
            }
        } else {
            CacheError::DownloadFailed(err.to_string())
        }
    }
}

/// Statistics about the cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_dir: PathBuf,
    pub total_models: usize,
    pub total_size_bytes: u64,
    pub models: Vec<String>,
}

/// Integrity validation report
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub valid_models: Vec<String>,
    pub corrupted_models: Vec<String>,
    pub missing_models: Vec<String>,
}

/// Manages cached Hugging Face models
pub struct CacheManager {
    cache_dir: PathBuf,
    manifest: Arc<RwLock<ManifestManager>>,
}

impl CacheManager {
    /// Create a new CacheManager with default XDG cache directory
    pub fn new() -> Result<Self, CacheError> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| {
                CacheError::DirectoryError("Could not determine cache directory".to_string())
            })?
            .join("caro")
            .join("models");

        Self::with_cache_dir(cache_dir)
    }

    /// Create a CacheManager with a custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self, CacheError> {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }

        if !cache_dir.is_dir() {
            return Err(CacheError::DirectoryError(format!(
                "Cache path is not a directory: {}",
                cache_dir.display()
            )));
        }

        let manifest = ManifestManager::new(cache_dir.clone())?;

        Ok(Self {
            cache_dir,
            manifest: Arc::new(RwLock::new(manifest)),
        })
    }

    /// Get a model from cache or download if not present
    pub async fn get_model(&self, model_id: &str) -> Result<PathBuf, CacheError> {
        // Check if model is already cached
        if self.is_cached(model_id) {
            let cached_model = {
                let manifest = self
                    .manifest
                    .read()
                    .map_err(|e| CacheError::ManifestError(format!("Lock error: {}", e)))?;
                manifest
                    .get_model(model_id)
                    .ok_or_else(|| CacheError::ModelNotFound(model_id.to_string()))?
            };

            // Validate checksum
            let actual_checksum = Self::calculate_checksum(&cached_model.path).await?;
            if actual_checksum != cached_model.checksum {
                return Err(CacheError::ChecksumMismatch {
                    model_id: model_id.to_string(),
                    expected: cached_model.checksum.clone(),
                    actual: actual_checksum,
                });
            }

            // Update last accessed time
            {
                let mut manifest = self
                    .manifest
                    .write()
                    .map_err(|e| CacheError::ManifestError(format!("Lock error: {}", e)))?;
                manifest.update_last_accessed(model_id)?;
            }

            Ok(cached_model.path.clone())
        } else {
            // Download model (placeholder - will integrate with Hugging Face API)
            self.download_model(model_id).await
        }
    }

    /// Check if a model is cached
    pub fn is_cached(&self, model_id: &str) -> bool {
        self.manifest
            .read()
            .map(|manifest| manifest.has_model(model_id))
            .unwrap_or(false)
    }

    /// Remove a specific model from cache
    pub async fn remove_model(&self, model_id: &str) -> Result<(), CacheError> {
        let path_to_delete = {
            let manifest = self
                .manifest
                .read()
                .map_err(|e| CacheError::ManifestError(format!("Lock error: {}", e)))?;

            let cached_model = manifest
                .get_model(model_id)
                .ok_or_else(|| CacheError::ModelNotFound(model_id.to_string()))?;

            cached_model.path.clone()
        };

        // Delete the model file (lock released)
        if path_to_delete.exists() {
            tokio::fs::remove_file(&path_to_delete).await?;
        }

        // Remove from manifest
        let mut manifest = self
            .manifest
            .write()
            .map_err(|e| CacheError::ManifestError(format!("Lock error: {}", e)))?;
        manifest.remove_model(model_id)?;

        Ok(())
    }

    /// Clear all cached models
    pub async fn clear_cache(&self) -> Result<(), CacheError> {
        let paths_to_delete: Vec<PathBuf> = {
            let manifest = self
                .manifest
                .read()
                .map_err(|e| CacheError::ManifestError(format!("Lock error: {}", e)))?;

            // Get all model paths before clearing
            manifest
                .list_models()
                .into_iter()
                .filter_map(|model_id| manifest.get_model(&model_id))
                .map(|cached_model| cached_model.path.clone())
                .collect()
        };

        // Delete all model files (lock released)
        for path in &paths_to_delete {
            if path.exists() {
                tokio::fs::remove_file(path).await?;
            }
        }

        // Clear manifest
        let mut manifest = self
            .manifest
            .write()
            .map_err(|e| CacheError::ManifestError(format!("Lock error: {}", e)))?;
        manifest.clear()?;

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let (models, total_size) = self
            .manifest
            .read()
            .map(|manifest| {
                let models = manifest.list_models();
                let total_size = manifest.total_size();
                (models, total_size)
            })
            .unwrap_or_else(|_| (Vec::new(), 0));

        CacheStats {
            cache_dir: self.cache_dir.clone(),
            total_models: models.len(),
            total_size_bytes: total_size,
            models,
        }
    }

    /// Validate integrity of all cached models
    pub async fn validate_integrity(&self) -> Result<IntegrityReport, CacheError> {
        let models_to_check: Vec<(String, PathBuf, String)> = {
            let manifest = self
                .manifest
                .read()
                .map_err(|e| CacheError::ManifestError(format!("Lock error: {}", e)))?;

            manifest
                .list_models()
                .into_iter()
                .filter_map(|model_id| {
                    manifest.get_model(&model_id).map(|cached_model| {
                        (
                            model_id.clone(),
                            cached_model.path.clone(),
                            cached_model.checksum.clone(),
                        )
                    })
                })
                .collect()
        };

        let mut valid_models = Vec::new();
        let mut corrupted_models = Vec::new();
        let mut missing_models = Vec::new();

        for (model_id, path, expected_checksum) in models_to_check {
            if !path.exists() {
                missing_models.push(model_id);
            } else {
                match Self::calculate_checksum(&path).await {
                    Ok(actual_checksum) => {
                        if actual_checksum == expected_checksum {
                            valid_models.push(model_id);
                        } else {
                            corrupted_models.push(model_id);
                        }
                    }
                    Err(_) => {
                        corrupted_models.push(model_id);
                    }
                }
            }
        }

        Ok(IntegrityReport {
            valid_models,
            corrupted_models,
            missing_models,
        })
    }

    /// Download a model from Hugging Face Hub
    ///
    /// This method:
    /// 1. Creates an HTTP client
    /// 2. Gets file metadata from HF Hub
    /// 3. Downloads the file with progress tracking and resume support
    /// 4. Validates checksum
    /// 5. Adds model to manifest atomically
    /// 6. Returns the cached file path
    async fn download_model(&self, model_id: &str) -> Result<PathBuf, CacheError> {
        // Create HTTP client
        let client = HfHubClient::new()
            .map_err(|e| CacheError::DownloadFailed(format!("Failed to create HTTP client: {}", e)))?;

        // For now, assume the model filename is pytorch_model.bin
        // TODO: In a full implementation, we would:
        // 1. Query the model repo to get the list of files
        // 2. Download all necessary files (config.json, tokenizer, weights, etc.)
        // For this MVP, we'll just download a single file
        let filename = "pytorch_model.bin";

        // Get file URL
        let url = client
            .get_file_url(model_id, filename, None)
            .map_err(|e| CacheError::DownloadFailed(format!("Failed to get file URL: {}", e)))?;

        // Get file metadata (size) using HEAD request
        let response = client
            .head_request(&url)
            .await
            .map_err(|e| CacheError::DownloadFailed(format!("Failed to get file metadata: {}", e)))?;

        let file_size = response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        // Create destination path: cache_dir/model_id/filename
        let model_dir = self.cache_dir.join(model_id);
        let dest_path = model_dir.join(filename);

        // Download file with progress, checksum, and resume support
        let (final_path, checksum) = download_file(&client, &url, &dest_path, file_size, None).await?;

        // Add model to manifest atomically
        let model_id_owned = model_id.to_string();
        let manifest_result = {
            let mut manifest = self
                .manifest
                .write()
                .map_err(|e| CacheError::ManifestError(format!("Lock error: {}", e)))?;

            manifest.atomic_update(|manifest_data| {
                let cached_model = CachedModel {
                    model_id: model_id_owned.clone(),
                    path: final_path.clone(),
                    size_bytes: file_size.unwrap_or(0),
                    checksum: checksum.clone(),
                    downloaded_at: chrono::Utc::now(),
                    last_accessed: chrono::Utc::now(),
                    version: None, // TODO: Extract version from model_id or metadata
                };

                manifest_data.models.insert(model_id_owned.clone(), cached_model);
                manifest_data.total_size_bytes += file_size.unwrap_or(0);

                // Check if we need LRU cleanup
                if manifest_data.total_size_bytes > manifest_data.max_cache_size_bytes {
                    manifest_data.cleanup_lru();
                }

                Ok(())
            })
        };

        manifest_result?;

        Ok(final_path)
    }

    /// Calculate SHA256 checksum of a file
    async fn calculate_checksum(path: &PathBuf) -> Result<String, CacheError> {
        use sha2::{Digest, Sha256};

        let contents = tokio::fs::read(path).await?;
        let hash = Sha256::digest(&contents);
        Ok(format!("{:x}", hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf());
        assert!(cache_manager.is_ok());
    }

    #[tokio::test]
    async fn test_cache_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("new_cache");

        assert!(!cache_path.exists());

        let cache_manager = CacheManager::with_cache_dir(cache_path.clone());
        assert!(cache_manager.is_ok());
        assert!(cache_path.exists());
        assert!(cache_path.is_dir());
    }

    #[test]
    fn test_is_cached_returns_false_for_missing() {
        let temp_dir = TempDir::new().unwrap();
        let cache_manager = CacheManager::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        assert!(!cache_manager.is_cached("nonexistent-model"));
    }

    // Error handling tests for WP08
    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_io_error_conversion() {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
            let cache_err: CacheError = io_err.into();

            match cache_err {
                CacheError::IoError(_) => (),
                _ => panic!("Expected IoError variant"),
            }

            let err_msg = cache_err.to_string();
            assert!(err_msg.contains("File system error"));
        }

        #[test]
        fn test_download_failed_message() {
            let err = CacheError::DownloadFailed("Connection reset".to_string());
            let err_msg = err.to_string();

            assert!(err_msg.contains("Download failed"));
            assert!(err_msg.contains("check your internet connection"));
            assert!(err_msg.contains("Connection reset"));
        }

        #[test]
        fn test_network_error_message() {
            let err = CacheError::NetworkError("Timeout after 30s".to_string());
            let err_msg = err.to_string();

            assert!(err_msg.contains("Network error"));
            assert!(err_msg.contains("connectivity issues"));
            assert!(err_msg.contains("Timeout after 30s"));
        }

        #[test]
        fn test_checksum_mismatch_message() {
            let err = CacheError::ChecksumMismatch {
                model_id: "test-model".to_string(),
                expected: "abc123".to_string(),
                actual: "def456".to_string(),
            };
            let err_msg = err.to_string();

            assert!(err_msg.contains("File integrity check failed"));
            assert!(err_msg.contains("test-model"));
            assert!(err_msg.contains("Expected checksum: abc123"));
            assert!(err_msg.contains("Actual checksum: def456"));
            assert!(err_msg.contains("corrupted"));
        }

        #[test]
        fn test_resume_not_supported_message() {
            let err = CacheError::ResumeNotSupported;
            let err_msg = err.to_string();

            assert!(err_msg.contains("Resume not supported"));
            assert!(err_msg.contains("server does not support"));
            assert!(err_msg.contains("from the beginning"));
        }

        #[test]
        fn test_authentication_required_message() {
            let err = CacheError::AuthenticationRequired;
            let err_msg = err.to_string();

            assert!(err_msg.contains("Authentication required"));
            assert!(err_msg.contains("HF_TOKEN"));
            assert!(err_msg.contains("huggingface.co/settings/tokens"));
        }

        #[test]
        fn test_model_not_found_message() {
            let err = CacheError::ModelNotFound("my-model".to_string());
            let err_msg = err.to_string();

            assert!(err_msg.contains("Model 'my-model' not found"));
            assert!(err_msg.contains("download command"));
        }

        #[test]
        fn test_manifest_error_message() {
            let err = CacheError::ManifestError("Parse error".to_string());
            let err_msg = err.to_string();

            assert!(err_msg.contains("Cache manifest error"));
            assert!(err_msg.contains("Parse error"));
            assert!(err_msg.contains("inconsistent state"));
        }

        #[test]
        fn test_directory_error_message() {
            let err = CacheError::DirectoryError("Not writable".to_string());
            let err_msg = err.to_string();

            assert!(err_msg.contains("Cache directory error"));
            assert!(err_msg.contains("Not writable"));
            assert!(err_msg.contains("writable"));
        }
    }
}
