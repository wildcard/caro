//! Cache module for managing Hugging Face model downloads and local storage
//!
//! Provides LRU cache management, integrity validation, and offline support.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

mod manifest;
pub use manifest::ManifestManager;

/// Cache-related errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Failed to download model '{model}': {reason}\n\nSuggestion: {suggestion}")]
    DownloadFailed {
        model: String,
        reason: String,
        suggestion: String,
    },

    #[error("Network timeout while downloading '{model}' after {timeout_secs}s\n\nSuggestion: Check your internet connection and try again.\nIf behind a corporate proxy, set the HTTPS_PROXY environment variable:\n  export HTTPS_PROXY=http://proxy.example.com:8080")]
    NetworkTimeout { model: String, timeout_secs: u64 },

    #[error("Connection refused for model '{model}' at {url}\n\nSuggestion: Check that the model repository is accessible.\nVerify the URL is correct: {url}\nIf behind a firewall, ensure outbound HTTPS is allowed.")]
    ConnectionRefused { model: String, url: String },

    #[error("DNS resolution failed for model '{model}'\n\nSuggestion: Check your DNS settings and internet connection.\nTry using a different DNS server (e.g., 8.8.8.8).")]
    DnsResolutionFailed { model: String },

    #[error("Model '{model}' not found in repository\n\nSuggestion: Verify the model ID is correct.\nBrowse available models at: https://huggingface.co/models?filter=text-generation\nExample valid model IDs:\n  • Qwen/Qwen2.5-Coder-1.5B\n  • codellama/CodeLlama-7b-hf")]
    ModelNotFoundInRepo { model: String },

    #[error("Checksum mismatch for model '{model_id}'\nExpected: {expected}\nActual:   {actual}\n\nSuggestion: The cached model may be corrupted.\nRemove the corrupted cache:\n  cmdai cache remove {model_id}\nThen retry to re-download the model.")]
    ChecksumMismatch {
        model_id: String,
        expected: String,
        actual: String,
    },

    #[error("Model '{0}' not found in cache\n\nSuggestion: Download the model first:\n  cmdai cache download {0}\nOr use the model directly (will auto-download):\n  cmdai \"your command\" --model {0}")]
    ModelNotFound(String),

    #[error("Permission denied: {path}\n\nThe cache directory requires {operation} permissions.\nSuggestion: Fix permissions with:\n  chmod 700 {path}")]
    PermissionDenied {
        path: PathBuf,
        operation: String,
    },

    #[error("Insufficient disk space for model '{model}'\nRequired: {required_mb} MB\nAvailable: {available_mb} MB\n\nSuggestion: Free up disk space or increase cache size limit in config:\n  cache_max_size_gb = {suggested_size_gb}")]
    InsufficientDiskSpace {
        model: String,
        required_mb: u64,
        available_mb: u64,
        suggested_size_gb: u64,
    },

    #[error("I/O error: {message}\nPath: {path}\n\nSuggestion: {suggestion}")]
    IoError {
        message: String,
        path: PathBuf,
        suggestion: String,
    },

    #[error("Manifest error: {0}")]
    ManifestError(String),

    #[error("Cache directory error: {0}")]
    DirectoryError(String),
}

impl CacheError {
    /// Create a network timeout error with helpful suggestions
    pub fn network_timeout(model: &str, timeout_secs: u64) -> Self {
        Self::NetworkTimeout {
            model: model.to_string(),
            timeout_secs,
        }
    }

    /// Create a connection refused error with helpful suggestions
    pub fn connection_refused(model: &str, url: &str) -> Self {
        Self::ConnectionRefused {
            model: model.to_string(),
            url: url.to_string(),
        }
    }

    /// Create a DNS resolution error with helpful suggestions
    pub fn dns_failed(model: &str) -> Self {
        Self::DnsResolutionFailed {
            model: model.to_string(),
        }
    }

    /// Create a model not found in repository error with examples
    pub fn model_not_in_repo(model: &str) -> Self {
        Self::ModelNotFoundInRepo {
            model: model.to_string(),
        }
    }

    /// Create a permission denied error with exact fix commands
    pub fn permission_denied(path: PathBuf, operation: &str) -> Self {
        Self::PermissionDenied {
            path,
            operation: operation.to_string(),
        }
    }

    /// Create an insufficient disk space error with suggestions
    pub fn insufficient_space(
        model: &str,
        required_mb: u64,
        available_mb: u64,
    ) -> Self {
        let suggested_size_gb = ((required_mb / 1024) + 5).max(10);
        Self::InsufficientDiskSpace {
            model: model.to_string(),
            required_mb,
            available_mb,
            suggested_size_gb,
        }
    }

    /// Wrap an IO error with context and suggestions
    pub fn from_io_error(err: std::io::Error, path: PathBuf) -> Self {
        let suggestion = match err.kind() {
            std::io::ErrorKind::PermissionDenied => {
                format!("Fix permissions with: chmod 700 {}", path.display())
            }
            std::io::ErrorKind::NotFound => {
                format!(
                    "The path does not exist. Create parent directories with:\n  mkdir -p {}",
                    path.parent().unwrap_or(&path).display()
                )
            }
            std::io::ErrorKind::AlreadyExists => {
                format!("The path already exists: {}", path.display())
            }
            _ => format!("Check that the path is accessible: {}", path.display()),
        };

        Self::IoError {
            message: err.to_string(),
            path,
            suggestion,
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
            .join("cmdai")
            .join("models");

        Self::with_cache_dir(cache_dir)
    }

    /// Create a CacheManager with a custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self, CacheError> {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)
                .map_err(|e| CacheError::from_io_error(e, cache_dir.clone()))?;
        }

        if !cache_dir.is_dir() {
            return Err(CacheError::DirectoryError(format!(
                "Cache path is not a directory: {}\n\nSuggestion: Remove the file and let cmdai create the directory:\n  rm {} && cmdai --version",
                cache_dir.display(),
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
            tokio::fs::remove_file(&path_to_delete)
                .await
                .map_err(|e| CacheError::from_io_error(e, path_to_delete.clone()))?;
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
                tokio::fs::remove_file(path)
                    .await
                    .map_err(|e| CacheError::from_io_error(e, path.clone()))?;
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

    /// Download a model from Hugging Face (placeholder implementation)
    async fn download_model(&self, model_id: &str) -> Result<PathBuf, CacheError> {
        // Placeholder: In real implementation, this would:
        // 1. Connect to Hugging Face API
        // 2. Download model files
        // 3. Calculate checksum
        // 4. Add to manifest
        // 5. Return path

        // For now, return a DownloadFailed error indicating network requirement
        Err(CacheError::DownloadFailed {
            model: model_id.to_string(),
            reason: "Download functionality not yet implemented".to_string(),
            suggestion: "This feature is coming soon! For now, manually download the model to the cache directory.\nAlternatively, use a remote backend like vLLM or Ollama:\n  cmdai \"your command\" --backend vllm".to_string(),
        })
    }

    /// Calculate SHA256 checksum of a file
    async fn calculate_checksum(path: &PathBuf) -> Result<String, CacheError> {
        use sha2::{Digest, Sha256};

        let contents = tokio::fs::read(path)
            .await
            .map_err(|e| CacheError::from_io_error(e, path.clone()))?;
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

    #[test]
    fn test_error_messages_have_suggestions() {
        // Test network timeout error
        let error = CacheError::network_timeout("qwen2.5-coder-1.5b", 30);
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("HTTPS_PROXY"));

        // Test connection refused error
        let error = CacheError::connection_refused("qwen2.5-coder-1.5b", "https://huggingface.co");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("firewall"));

        // Test DNS failed error
        let error = CacheError::dns_failed("qwen2.5-coder-1.5b");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("DNS"));

        // Test model not in repo error
        let error = CacheError::model_not_in_repo("invalid-model");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("huggingface.co"));

        // Test permission denied error
        let error = CacheError::permission_denied(PathBuf::from("/test/path"), "write");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("chmod 700"));

        // Test insufficient disk space error
        let error = CacheError::insufficient_space("qwen2.5-coder-1.5b", 5000, 1000);
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("cache_max_size_gb"));
    }

    #[test]
    fn test_io_error_context() {
        use std::io;

        // Test permission denied IO error
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let cache_err = CacheError::from_io_error(io_err, PathBuf::from("/test/path"));

        // The from_io_error function returns PermissionDenied variant for PermissionDenied errors
        let error_msg = format!("{}", cache_err);
        eprintln!("Error message: {}", error_msg);
        assert!(error_msg.contains("Permission denied") || error_msg.contains("permission"),
            "Expected 'Permission denied' in error message, got: {}", error_msg);
        assert!(error_msg.contains("/test/path"));
        assert!(error_msg.contains("chmod 700"));

        // Test not found IO error
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let cache_err = CacheError::from_io_error(io_err, PathBuf::from("/test/path"));
        let error_msg = cache_err.to_string();
        assert!(error_msg.contains("mkdir -p"));
    }

    #[test]
    fn test_checksum_mismatch_message() {
        let error = CacheError::ChecksumMismatch {
            model_id: "qwen2.5-coder-1.5b".to_string(),
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("cmdai cache remove"));
        assert!(error_msg.contains("corrupted"));
    }

    #[test]
    fn test_model_not_found_message() {
        let error = CacheError::ModelNotFound("qwen2.5-coder-1.5b".to_string());
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("cmdai cache download"));
    }
}
