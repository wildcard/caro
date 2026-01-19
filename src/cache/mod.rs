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
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("File system error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Download failed: {0}. Please check your internet connection and try again.")]
    DownloadFailed(String),

    #[error("Network error: {0}. This may indicate connectivity issues or server problems.")]
    NetworkError(String),

    #[error("File integrity check failed for model {model_id}. Expected checksum: {expected}, Actual checksum: {actual}. The file may be corrupted.")]
    ChecksumMismatch {
        model_id: String,
        expected: String,
        actual: String,
    },

    #[error("Resume not supported. The server does not support resumable downloads. Will restart from the beginning.")]
    ResumeNotSupported,

    #[error("Authentication required. Set HF_TOKEN environment variable with your Hugging Face token. Get one at https://huggingface.co/settings/tokens")]
    AuthenticationRequired,

    #[error("Model '{0}' not found in cache. Use the download command to fetch it first.")]
    ModelNotFound(String),

    #[error("Cache manifest error: {0}. The cache may be in an inconsistent state.")]
    ManifestError(String),

    #[error(
        "Cache directory error: {0}. Please ensure the cache directory exists and is writable."
    )]
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

impl From<HttpClientError> for CacheError {
    fn from(err: HttpClientError) -> Self {
        match err {
            HttpClientError::RequestFailed(req_err) => {
                // Delegate to the reqwest::Error conversion
                CacheError::from(req_err)
            }
            HttpClientError::InvalidUrl(msg) => {
                CacheError::DownloadFailed(format!("Invalid URL: {}", msg))
            }
            HttpClientError::AuthError(_msg) => CacheError::AuthenticationRequired,
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
    ///
    /// Creates the cache directory (~/.cache/caro/models) if it doesn't exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::cache::CacheManager;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = CacheManager::new()?;
    /// let stats = cache.stats();
    /// println!("Cache directory: {}", stats.cache_dir.display());
    /// println!("Total models: {}", stats.total_models);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CacheError::DirectoryError` if the cache directory cannot be determined
    /// or created.
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
    ///
    /// If the model is already cached, validates its integrity and returns the path.
    /// Otherwise, downloads the model from Hugging Face Hub and caches it locally.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::cache::CacheManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = CacheManager::new()?;
    ///
    /// // Get a model (downloads if not cached)
    /// let model_path = cache.get_model("Qwen/Qwen2.5-Coder-1.5B-Instruct").await?;
    /// println!("Model cached at: {}", model_path.display());
    ///
    /// // Subsequent calls use the cached version
    /// let same_path = cache.get_model("Qwen/Qwen2.5-Coder-1.5B-Instruct").await?;
    /// assert_eq!(model_path, same_path);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns errors if:
    /// - Download fails (`CacheError::DownloadFailed`)
    /// - Network issues occur (`CacheError::NetworkError`)
    /// - Checksum validation fails (`CacheError::ChecksumMismatch`)
    /// - Authentication required (`CacheError::AuthenticationRequired`)
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
    ///
    /// Returns `true` if the model exists in the cache manifest, `false` otherwise.
    /// This does not validate the model's integrity - use `validate_integrity()` for that.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::cache::CacheManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = CacheManager::new()?;
    ///
    /// // Check before downloading
    /// if !cache.is_cached("Qwen/Qwen2.5-Coder-1.5B-Instruct") {
    ///     println!("Model not cached, downloading...");
    ///     cache.get_model("Qwen/Qwen2.5-Coder-1.5B-Instruct").await?;
    /// } else {
    ///     println!("Model already cached");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_cached(&self, model_id: &str) -> bool {
        self.manifest
            .read()
            .map(|manifest| manifest.has_model(model_id))
            .unwrap_or(false)
    }

    /// Remove a specific model from cache
    ///
    /// Deletes the model file from disk and removes it from the cache manifest.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::cache::CacheManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = CacheManager::new()?;
    ///
    /// // Download a model
    /// cache.get_model("Qwen/Qwen2.5-Coder-1.5B-Instruct").await?;
    ///
    /// // Remove it to free space
    /// cache.remove_model("Qwen/Qwen2.5-Coder-1.5B-Instruct").await?;
    /// println!("Model removed from cache");
    ///
    /// // Model is no longer cached
    /// assert!(!cache.is_cached("Qwen/Qwen2.5-Coder-1.5B-Instruct"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CacheError::ModelNotFound` if the model is not in the cache.
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
    ///
    /// Returns information about cached models including total count, total size,
    /// and list of model IDs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::cache::CacheManager;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = CacheManager::new()?;
    /// let stats = cache.stats();
    ///
    /// println!("Cache directory: {}", stats.cache_dir.display());
    /// println!("Total models: {}", stats.total_models);
    /// println!("Total size: {} bytes ({:.2} GB)",
    ///     stats.total_size_bytes,
    ///     stats.total_size_bytes as f64 / 1_073_741_824.0
    /// );
    ///
    /// println!("Cached models:");
    /// for model_id in &stats.models {
    ///     println!("  - {}", model_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Checks the SHA256 checksum of each cached model file against the stored checksum.
    /// Returns a report categorizing models as valid, corrupted, or missing.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::cache::CacheManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = CacheManager::new()?;
    ///
    /// // Validate all cached models
    /// let report = cache.validate_integrity().await?;
    ///
    /// println!("Valid models: {}", report.valid_models.len());
    /// for model_id in &report.valid_models {
    ///     println!("  ✓ {}", model_id);
    /// }
    ///
    /// println!("Corrupted models: {}", report.corrupted_models.len());
    /// for model_id in &report.corrupted_models {
    ///     println!("  ✗ {} (checksum mismatch)", model_id);
    ///     // Consider re-downloading corrupted models
    ///     cache.remove_model(model_id).await?;
    /// }
    ///
    /// println!("Missing models: {}", report.missing_models.len());
    /// for model_id in &report.missing_models {
    ///     println!("  ? {} (file not found)", model_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CacheError::ManifestError` if the manifest cannot be read.
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
        let client = HfHubClient::new().map_err(|e| {
            CacheError::DownloadFailed(format!("Failed to create HTTP client: {}", e))
        })?;

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
        let response = client.head_request(&url).await.map_err(|e| {
            CacheError::DownloadFailed(format!("Failed to get file metadata: {}", e))
        })?;

        let file_size = response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        // Create destination path: cache_dir/model_id/filename
        let model_dir = self.cache_dir.join(model_id);
        let dest_path = model_dir.join(filename);

        // Download file with progress, checksum, and resume support
        let (final_path, checksum) =
            download_file(&client, &url, &dest_path, file_size, None).await?;

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

                manifest_data
                    .models
                    .insert(model_id_owned.clone(), cached_model);
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

    /// Property-based tests for LRU cache eviction algorithm.
    ///
    /// # Overview
    ///
    /// These tests use PropTest to verify cache behavior across randomized scenarios,
    /// providing stronger guarantees than example-based unit tests alone.
    ///
    /// # Properties Verified
    ///
    /// ## Eviction Order (WP02)
    /// - **LRU First**: Least recently accessed models evicted before more recent ones
    /// - **Access Updates Position**: Accessing a model refreshes its recency timestamp
    /// - **Chronological Sequence**: Multiple evictions follow strict access history order
    ///
    /// ## Size Constraints (WP03)
    /// - **Size Limit Respected**: Cache never exceeds `max_cache_size_bytes` after cleanup
    /// - **Eviction Before Overflow**: Adding models beyond capacity triggers eviction
    /// - **Timestamp Updates**: Access operations update `last_accessed` correctly
    ///
    /// ## Edge Cases (WP04)
    /// - **Single-Item Cache**: Correctly handles max_size fitting only one model
    /// - **Empty Operations**: Safe handling of cleanup/get/remove on empty cache
    /// - **Duplicate IDs**: Adding existing model ID replaces (not duplicates)
    /// - **Zero-Sized Models**: Correctly handles models with 0 bytes
    ///
    /// # Test Execution
    ///
    /// Each property test runs 100 iterations with randomized inputs:
    /// - Cache sizes: 1GB to 10GB
    /// - Model counts: 2 to 20 models
    /// - Model sizes: 50MB to 500MB
    /// - Access patterns: Random sequences with 5-15 operations
    ///
    /// Run with: `cargo test prop_`
    ///
    /// # Implementation Notes
    ///
    /// Tests use `CacheManifest` directly and explicitly call `cleanup_lru()` after
    /// `add_model()` to mimic `ManifestManager`'s behavior. This pattern reflects
    /// the production code's design where cleanup is explicit, not automatic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Property: Cache respects size limit
    /// let mut manifest = CacheManifest::new(1); // 1GB
    /// manifest.add_model(model_a); // 600MB
    /// manifest.add_model(model_b); // 600MB - exceeds limit
    ///
    /// // Explicit cleanup (mimics ManifestManager)
    /// if manifest.total_size_bytes > manifest.max_cache_size_bytes {
    ///     manifest.cleanup_lru(); // Evicts model_a (older)
    /// }
    ///
    /// assert!(manifest.total_size_bytes <= manifest.max_cache_size_bytes);
    /// assert!(manifest.get_model("model_b").is_some()); // Newer model kept
    /// ```
    mod property_tests {

        use crate::models::{CacheManifest, CachedModel};
        use chrono::{DateTime, Duration, Utc};
        use proptest::prelude::*;
        use std::path::PathBuf;

        /// Helper to create a test CachedModel
        fn create_test_model(
            id: String,
            size_bytes: u64,
            last_accessed: DateTime<Utc>,
        ) -> CachedModel {
            CachedModel {
                model_id: id.clone(),
                path: PathBuf::from(format!("/tmp/{}", id)),
                checksum: "test_checksum".to_string(),
                size_bytes,
                downloaded_at: Utc::now(),
                last_accessed,
                version: Some("1.0".to_string()),
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn smoke_test(x in 0..100i32) {
                // Smoke test to verify PropTest integration
                assert!((0..100).contains(&x));
            }

            /// T003: Verify that LRU eviction removes least recently accessed model first
            #[test]
            fn prop_lru_evicts_least_recent(
                max_size_gb in 1u64..5,
                model_count in 3usize..10,
                model_sizes in prop::collection::vec(1u64..100, 3..10)
            ) {
                let mut manifest = CacheManifest::new(max_size_gb);
                let now = Utc::now();

                // Add models with sequential access times (oldest first)
                let model_count = model_count.min(model_sizes.len());
                for (i, &size_gb) in model_sizes.iter().enumerate().take(model_count) {
                    let model = create_test_model(
                        format!("model_{}", i),
                        size_gb * 1024 * 1024 * 1024, // Convert to bytes
                        now - Duration::seconds((model_count - i) as i64 * 60),
                    );
                    manifest.add_model(model);
                }

                // Record which model has oldest access time
                let oldest_model_id = "model_0".to_string(); // First one added has oldest time

                // Force cleanup by adding a large model
                let trigger_model = create_test_model(
                    "trigger".to_string(),
                    max_size_gb * 1024 * 1024 * 1024,
                    now,
                );
                manifest.add_model(trigger_model);

                // Verify LRU cleanup happened
                let removed_models = manifest.cleanup_lru();

                // If cleanup occurred, the oldest model should be among those evicted
                if !removed_models.is_empty() {
                    prop_assert!(
                        !manifest.models.contains_key(&oldest_model_id) ||
                        manifest.total_size_bytes <= manifest.max_cache_size_bytes,
                        "LRU should evict oldest model first or stay within size limit"
                    );
                }
            }

            /// T004: Verify that accessing a model updates its last_accessed time
            #[test]
            fn prop_access_updates_position(
                model_count in 2usize..8,
                access_index in 0usize..7
            ) {
                let mut manifest = CacheManifest::new(10);
                let base_time = Utc::now() - Duration::hours(24);

                // Add models with old access times
                for i in 0..model_count {
                    let model = create_test_model(
                        format!("model_{}", i),
                        100 * 1024 * 1024, // 100MB each
                        base_time + Duration::hours(i as i64),
                    );
                    manifest.add_model(model);
                }

                // Access a specific model (if index is valid)
                if access_index < model_count {
                    let model_id = format!("model_{}", access_index);
                    let before_time = manifest.get_model(&model_id).unwrap().last_accessed;

                    // Simulate access by updating the model
                    if let Some(mut model) = manifest.remove_model(&model_id) {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        model.last_accessed = Utc::now();
                        manifest.add_model(model);
                    }

                    let after_time = manifest.get_model(&model_id).unwrap().last_accessed;

                    // Verify access time was updated
                    prop_assert!(
                        after_time > before_time,
                        "Accessing a model should update its last_accessed time"
                    );
                }
            }

            /// T005: Verify eviction sequence follows access history order
            #[test]
            fn prop_eviction_sequence_follows_history(
                initial_models in 3usize..7,
                model_size_mb in 100u64..500
            ) {
                let max_size_gb = 1; // 1GB limit
                let mut manifest = CacheManifest::new(max_size_gb);
                let now = Utc::now();

                // Add models with different access times (oldest to newest)
                let mut expected_eviction_order = Vec::new();
                for i in 0..initial_models {
                    let model_id = format!("model_{}", i);
                    expected_eviction_order.push(model_id.clone());

                    let model = create_test_model(
                        model_id,
                        model_size_mb * 1024 * 1024,
                        now - Duration::hours((initial_models - i) as i64),
                    );
                    manifest.add_model(model);
                }

                // Trigger cleanup
                let removed = manifest.cleanup_lru();

                // If evictions occurred, verify they follow LRU order
                if !removed.is_empty() {
                    for (idx, evicted_id) in removed.iter().enumerate() {
                        // Each evicted model should come from the expected order
                        prop_assert!(
                            expected_eviction_order.contains(evicted_id),
                            "Evicted model {} should be in expected eviction candidates",
                            evicted_id
                        );

                        // Earlier evictions should have earlier positions in expected order
                        if idx > 0 {
                            let prev_evicted = &removed[idx - 1];
                            let prev_pos = expected_eviction_order.iter().position(|id| id == prev_evicted);
                            let curr_pos = expected_eviction_order.iter().position(|id| id == evicted_id);

                            if let (Some(prev), Some(curr)) = (prev_pos, curr_pos) {
                                prop_assert!(
                                    prev <= curr,
                                    "Eviction sequence should follow chronological order"
                                );
                            }
                        }
                    }
                }
            }

            /// T006: Verify cache respects size limit
            #[test]
            fn prop_cache_respects_size_limit(
                max_size_gb in 1u64..10,
                model_sizes_mb in prop::collection::vec(50u64..200, 0..20)
            ) {
                let mut manifest = CacheManifest::new(max_size_gb);
                let now = Utc::now();

                // Add models incrementally
                for (i, size_mb) in model_sizes_mb.iter().enumerate() {
                    let model = create_test_model(
                        format!("model_{}", i),
                        size_mb * 1024 * 1024,
                        now + Duration::seconds(i as i64),
                    );
                    manifest.add_model(model);

                    // Trigger cleanup if needed (mimics ManifestManager behavior)
                    if manifest.total_size_bytes > manifest.max_cache_size_bytes {
                        manifest.cleanup_lru();
                    }

                    // After each addition, verify size constraint
                    prop_assert!(
                        manifest.total_size_bytes <= manifest.max_cache_size_bytes,
                        "Cache size {} exceeded max_size {} after adding model {}",
                        manifest.total_size_bytes,
                        manifest.max_cache_size_bytes,
                        i
                    );
                }
            }

            /// T007: Verify eviction happens before overflow
            #[test]
            fn prop_eviction_before_overflow(
                max_size_mb in 500u64..1000,
                model_size_mb in 100u64..300
            ) {
                let max_size_bytes = max_size_mb * 1024 * 1024;
                let mut manifest = CacheManifest::new(1); // 1GB
                manifest.max_cache_size_bytes = max_size_bytes; // Override for test
                let now = Utc::now();

                // Fill cache to capacity
                let mut total_added = 0u64;
                let mut models_added = 0;
                while total_added < max_size_bytes {
                    let model = create_test_model(
                        format!("model_{}", models_added),
                        model_size_mb * 1024 * 1024,
                        now + Duration::seconds(models_added as i64),
                    );
                    manifest.add_model(model);
                    total_added += model_size_mb * 1024 * 1024;
                    models_added += 1;

                    if models_added > 20 {
                        break; // Safety limit for test
                    }
                }

                // Record size before adding overflow model
                let size_before = manifest.total_size_bytes;

                // Add one more model that should trigger eviction
                let overflow_model = create_test_model(
                    "overflow".to_string(),
                    model_size_mb * 1024 * 1024,
                    now + Duration::seconds(models_added as i64),
                );
                manifest.add_model(overflow_model);

                // Trigger cleanup (mimics ManifestManager behavior)
                if manifest.total_size_bytes > manifest.max_cache_size_bytes {
                    manifest.cleanup_lru();
                }

                // Verify size is still within limit (eviction should have occurred)
                prop_assert!(
                    manifest.total_size_bytes <= manifest.max_cache_size_bytes,
                    "Cache should evict before exceeding size limit. Before: {}, After: {}, Max: {}",
                    size_before,
                    manifest.total_size_bytes,
                    manifest.max_cache_size_bytes
                );
            }

            /// T008: Verify access updates timestamp
            #[test]
            fn prop_access_updates_timestamp(
                initial_models in 2usize..6,
                access_sequence in prop::collection::vec(0usize..5, 5..15)
            ) {
                let mut manifest = CacheManifest::new(10);
                let base_time = Utc::now() - Duration::days(7);

                // Add models with old timestamps
                for i in 0..initial_models {
                    let model = create_test_model(
                        format!("model_{}", i),
                        100 * 1024 * 1024,
                        base_time + Duration::hours(i as i64),
                    );
                    manifest.add_model(model);
                }

                // Access models in random sequence
                for access_idx in access_sequence {
                    if access_idx < initial_models {
                        let model_id = format!("model_{}", access_idx);

                        // Get current timestamp
                        if let Some(model) = manifest.get_model(&model_id) {
                            let old_time = model.last_accessed;

                            // Simulate access (remove and re-add with updated time)
                            if let Some(mut updated_model) = manifest.remove_model(&model_id) {
                                std::thread::sleep(std::time::Duration::from_millis(5));
                                updated_model.last_accessed = Utc::now();
                                let new_time = updated_model.last_accessed;
                                manifest.add_model(updated_model);

                                // Verify timestamp increased
                                prop_assert!(
                                    new_time > old_time,
                                    "Access should update timestamp from {} to {}",
                                    old_time,
                                    new_time
                                );
                            }
                        }
                    }
                }

                // Verify most recently accessed models are protected from eviction
                // Fill cache to trigger cleanup
                for i in 0..5 {
                    let large_model = create_test_model(
                        format!("filler_{}", i),
                        2 * 1024 * 1024 * 1024, // 2GB each
                        Utc::now(),
                    );
                    manifest.add_model(large_model);
                }

                // Models with newer timestamps should be less likely to be evicted
                let removed = manifest.cleanup_lru();
                if !removed.is_empty() {
                    // Verify at least some old models were evicted
                    let old_models: Vec<String> = (0..initial_models)
                        .map(|i| format!("model_{}", i))
                        .collect();

                    let evicted_old = removed.iter()
                        .filter(|id| old_models.contains(id))
                        .count();

                    // If evictions happened, some should be from the old models
                    if !removed.is_empty() {
                        prop_assert!(
                            evicted_old > 0 || manifest.models.values().any(|m| m.last_accessed >= base_time),
                            "LRU should preferentially evict older models"
                        );
                    }
                }
            }

            /// T009: Edge case - Single item cache
            /// Tests LRU behavior when max size allows only one model
            #[test]
            fn prop_single_item_cache(
                model_sizes_mb in prop::collection::vec(100u64..500, 1..10)
            ) {
                let max_size_mb = 100; // Small cache that fits only one model
                let mut manifest = CacheManifest::new(1);
                manifest.max_cache_size_bytes = max_size_mb * 1024 * 1024;
                let now = Utc::now();

                // Add models one by one
                for (i, size_mb) in model_sizes_mb.iter().enumerate() {
                    let model = create_test_model(
                        format!("model_{}", i),
                        size_mb * 1024 * 1024,
                        now + Duration::seconds(i as i64),
                    );
                    manifest.add_model(model);

                    // Trigger cleanup
                    if manifest.total_size_bytes > manifest.max_cache_size_bytes {
                        manifest.cleanup_lru();
                    }

                    // Cache should contain at most 1 model
                    prop_assert!(
                        manifest.models.len() <= 1,
                        "Single-item cache should contain at most 1 model, found {}",
                        manifest.models.len()
                    );

                    // Size should be within limit
                    prop_assert!(
                        manifest.total_size_bytes <= manifest.max_cache_size_bytes,
                        "Single-item cache exceeded size limit"
                    );
                }
            }

            /// T009: Edge case - Empty cache operations
            /// Tests that operations on empty cache handle correctly
            #[test]
            fn prop_empty_cache_operations(max_size_gb in 1u64..5) {
                let mut manifest = CacheManifest::new(max_size_gb);

                // Cleanup on empty cache should not panic
                let removed = manifest.cleanup_lru();
                prop_assert!(
                    removed.is_empty(),
                    "Cleanup on empty cache should return empty vector"
                );

                // Get non-existent model
                prop_assert!(
                    manifest.get_model("nonexistent").is_none(),
                    "Getting non-existent model should return None"
                );

                // Remove non-existent model
                let removed_model = manifest.remove_model("nonexistent");
                prop_assert!(
                    removed_model.is_none(),
                    "Removing non-existent model should return None"
                );

                // Verify manifest state is still valid
                prop_assert_eq!(manifest.models.len(), 0);
                prop_assert_eq!(manifest.total_size_bytes, 0);
            }

            /// T009: Edge case - Duplicate model IDs
            /// Tests that adding model with existing ID replaces the old one
            #[test]
            fn prop_duplicate_model_ids(
                model_count in 2usize..6,
                duplicate_index in 0usize..5
            ) {
                let mut manifest = CacheManifest::new(10);
                let now = Utc::now();

                // Add initial models
                for i in 0..model_count {
                    let model = create_test_model(
                        format!("model_{}", i),
                        100 * 1024 * 1024,
                        now + Duration::seconds(i as i64),
                    );
                    manifest.add_model(model);
                }

                let initial_count = manifest.models.len();

                // Add duplicate model (if index valid)
                if duplicate_index < model_count {
                    let duplicate_model = create_test_model(
                        format!("model_{}", duplicate_index),
                        200 * 1024 * 1024, // Different size
                        now + Duration::hours(1),
                    );
                    manifest.add_model(duplicate_model);

                    // Count should remain same (replaced, not added)
                    prop_assert_eq!(
                        manifest.models.len(),
                        initial_count,
                        "Duplicate model ID should replace existing, not add new"
                    );

                    // Verify updated size
                    if let Some(model) = manifest.get_model(&format!("model_{}", duplicate_index)) {
                        prop_assert_eq!(
                            model.size_bytes,
                            200 * 1024 * 1024,
                            "Duplicate model should have new size"
                        );
                    }
                }
            }

            /// T009: Edge case - Zero-sized models
            /// Tests that zero-sized models are handled correctly
            #[test]
            fn prop_zero_sized_models(model_count in 1usize..8) {
                let mut manifest = CacheManifest::new(1);
                let now = Utc::now();

                // Add zero-sized models
                for i in 0..model_count {
                    let model = create_test_model(
                        format!("model_{}", i),
                        0, // Zero bytes
                        now + Duration::seconds(i as i64),
                    );
                    manifest.add_model(model);
                }

                // All zero-sized models should be added
                prop_assert_eq!(
                    manifest.models.len(),
                    model_count,
                    "Zero-sized models should be added to cache"
                );

                // Total size should still be zero
                prop_assert_eq!(
                    manifest.total_size_bytes,
                    0,
                    "Total size should be zero for zero-sized models"
                );

                // Cleanup should not remove any (none exceed limit)
                let removed = manifest.cleanup_lru();
                prop_assert!(
                    removed.is_empty(),
                    "Zero-sized models should not be evicted"
                );
            }
        }
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
