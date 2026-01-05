// Model loading and distribution strategy for embedded models

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::backends::embedded::ModelVariant;
use crate::model_catalog::{ModelCatalog, ModelInfo};

/// Errors that can occur during model download
#[derive(Debug, thiserror::Error)]
pub enum ModelDownloadError {
    #[error("Network error: {message}\n\nPossible solutions:\n  1. Check your internet connection\n  2. Try again later (Hugging Face may be rate-limiting)\n  3. Use Ollama backend: export CARO_BACKEND=ollama\n  4. See: https://caro.sh/docs/troubleshooting#download-issues")]
    Network { message: String },

    #[error("SSL/TLS error: {message}\n\nPossible solutions:\n  1. Check your system's SSL certificates\n  2. Ensure your clock is set correctly\n  3. Try: export SSL_CERT_FILE=/path/to/ca-bundle.crt")]
    SslError { message: String },

    #[error("Disk space error: {message}\n\nThe model requires approximately {required_mb} MB of disk space.\nFree up space in your cache directory: {cache_dir}")]
    DiskSpace {
        message: String,
        required_mb: u64,
        cache_dir: String,
    },

    #[error("Rate limited by Hugging Face Hub\n\nPossible solutions:\n  1. Wait a few minutes and try again\n  2. Set HF_TOKEN environment variable for authenticated access\n  3. Use a smaller model: export CARO_MODEL=smollm-135m-q4")]
    RateLimited,

    #[error("Model not found: {model_id}\n\nThe model may have been moved or deleted from Hugging Face Hub.\nCheck available models at: https://huggingface.co/{repo}")]
    ModelNotFound { model_id: String, repo: String },

    #[error("Download failed after {attempts} attempts: {message}\n\nLast error: {last_error}\n\nPossible solutions:\n  1. Check your internet connection\n  2. Use Ollama backend: export CARO_BACKEND=ollama\n  3. Pre-download model: caro --download-model\n  4. Use smaller model for testing: export CARO_MODEL=smollm-135m-q4")]
    RetryExhausted {
        attempts: u32,
        message: String,
        last_error: String,
    },

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Model loader for managing embedded model distribution and caching
#[derive(Clone)]
pub struct ModelLoader {
    cache_dir: PathBuf,
    selected_model: &'static ModelInfo,
}

impl ModelLoader {
    /// Create a new model loader with default cache directory and default model
    /// Checks CARO_MODEL environment variable for model selection
    pub fn new() -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;

        // Check for CARO_MODEL environment variable
        let selected_model = if let Ok(model_id) = std::env::var("CARO_MODEL") {
            debug!("Using model from CARO_MODEL env var: {}", model_id);
            ModelCatalog::by_id(&model_id)
                .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?
        } else {
            ModelCatalog::default_model()
        };

        Ok(Self {
            cache_dir,
            selected_model,
        })
    }

    /// Create a new model loader with a specific model
    pub fn with_model(model_id: &str) -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        let model = ModelCatalog::by_id(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;
        Ok(Self {
            cache_dir,
            selected_model: model,
        })
    }

    /// Create a model loader with the smallest model (best for CI/CD)
    pub fn with_smallest_model() -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        Ok(Self {
            cache_dir,
            selected_model: ModelCatalog::smallest(),
        })
    }

    /// Create a model loader with a custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            selected_model: ModelCatalog::default_model(),
        }
    }

    /// Create a model loader with custom cache directory and model
    pub fn with_cache_dir_and_model(cache_dir: PathBuf, model: &'static ModelInfo) -> Self {
        Self {
            cache_dir,
            selected_model: model,
        }
    }

    /// Get the currently selected model info
    pub fn selected_model(&self) -> &'static ModelInfo {
        self.selected_model
    }

    /// List all available models
    pub fn list_models() -> &'static [&'static ModelInfo] {
        ModelCatalog::all_models()
    }

    /// List CI-suitable models (< 1GB)
    pub fn list_ci_models() -> Vec<&'static ModelInfo> {
        ModelCatalog::ci_models()
    }

    /// Get the default cache directory for models
    /// Returns: ~/.cache/caro/models/ on Unix, %LOCALAPPDATA%\caro\models\ on Windows
    pub fn default_cache_dir() -> Result<PathBuf> {
        let cache_base = directories::BaseDirs::new()
            .context("Failed to determine user cache directory")?
            .cache_dir()
            .to_path_buf();

        let caro_cache = cache_base.join("caro").join("models");

        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&caro_cache).context("Failed to create model cache directory")?;

        Ok(caro_cache)
    }

    /// Auto-detect the best available model variant for the current platform
    pub fn detect_platform() -> ModelVariant {
        ModelVariant::detect()
    }

    /// Get the embedded model path (bundled with binary or from cache)
    ///
    /// Priority:
    /// 1. Check bundled model path (models/*/filename.gguf)
    /// 2. Check cache directory (~/.cache/caro/models/filename.gguf)
    /// 3. If not found, returns path where it should be downloaded
    pub fn get_embedded_model_path(&self) -> Result<PathBuf> {
        // Check bundled models first (for future binary embedding)
        let bundled_path = self.bundled_model_path();
        if bundled_path.exists() {
            debug!("Using bundled model at: {}", bundled_path.display());
            return Ok(bundled_path);
        }

        // Check cache directory
        let cached_path = self.cached_model_path();
        if cached_path.exists() {
            debug!("Using cached model at: {}", cached_path.display());
            return Ok(cached_path);
        }

        // Return cache path where model should be downloaded
        info!(
            "Model {} not found, will download to: {}",
            self.selected_model.name,
            cached_path.display()
        );
        Ok(cached_path)
    }

    /// Download model from Hugging Face Hub if missing
    ///
    /// This will download the selected model from Hugging Face
    /// and save it to the cache directory. Includes retry logic with
    /// exponential backoff for transient network failures.
    pub async fn download_model_if_missing(&self, variant: ModelVariant) -> Result<PathBuf> {
        let model_path = self.get_embedded_model_path()?;

        if model_path.exists() {
            debug!("Model already exists at: {}", model_path.display());
            return Ok(model_path);
        }

        // Check available disk space before download
        self.check_disk_space()?;

        // Show download information with progress
        self.show_download_info();

        // Attempt download with retry logic
        match self.download_model_with_retry(&model_path, variant).await {
            Ok(()) => {
                info!("Model downloaded successfully to: {}", model_path.display());
                Ok(model_path)
            }
            Err(e) => {
                // Clean up partial download if exists
                if model_path.exists() {
                    let _ = std::fs::remove_file(&model_path);
                }
                Err(e.into())
            }
        }
    }

    /// Show download information to user
    fn show_download_info(&self) {
        use colored::Colorize;

        eprintln!(
            "\n{} Downloading {} ({} MB)",
            "→".cyan().bold(),
            self.selected_model.name.bold(),
            self.selected_model.size_mb
        );
        eprintln!(
            "  {} {}",
            "Source:".dimmed(),
            format!("huggingface.co/{}", self.selected_model.hf_repo).dimmed()
        );
        eprintln!(
            "  {} {}\n",
            "Cache:".dimmed(),
            self.cache_dir.display().to_string().dimmed()
        );
        eprintln!(
            "{}",
            "  This is a one-time download. Future runs will use the cached model.".dimmed()
        );
        eprintln!();
    }

    /// Check if there's sufficient disk space for the model
    fn check_disk_space(&self) -> Result<()> {
        // Try to get available space, but don't fail if we can't determine it
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;

            if let Ok(metadata) = std::fs::metadata(&self.cache_dir) {
                // Get filesystem stats using statvfs
                let path_cstr =
                    std::ffi::CString::new(self.cache_dir.to_string_lossy().as_bytes())
                        .map_err(|_| anyhow::anyhow!("Invalid path"))?;

                unsafe {
                    let mut stat: libc::statvfs = std::mem::zeroed();
                    if libc::statvfs(path_cstr.as_ptr(), &mut stat) == 0 {
                        let available_bytes = stat.f_bavail as u64 * stat.f_bsize as u64;
                        let available_mb = available_bytes / (1024 * 1024);
                        let required_mb = self.selected_model.size_mb + 100; // Extra buffer

                        if available_mb < required_mb {
                            return Err(ModelDownloadError::DiskSpace {
                                message: format!(
                                    "Only {} MB available, need {} MB",
                                    available_mb, required_mb
                                ),
                                required_mb,
                                cache_dir: self.cache_dir.display().to_string(),
                            }
                            .into());
                        }

                        debug!(
                            "Disk space check passed: {} MB available, {} MB required",
                            available_mb, required_mb
                        );
                    }
                }
                // Suppress unused variable warning
                let _ = metadata.dev();
            }
        }
        Ok(())
    }

    /// Download the model with retry logic and exponential backoff
    async fn download_model_with_retry(
        &self,
        dest_path: &Path,
        _variant: ModelVariant,
    ) -> Result<(), ModelDownloadError> {
        const MAX_ATTEMPTS: u32 = 3;
        const INITIAL_BACKOFF_MS: u64 = 2000;

        let mut last_error = String::new();

        for attempt in 1..=MAX_ATTEMPTS {
            match self.download_model_attempt(dest_path).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = e.to_string();

                    // Don't retry for non-transient errors
                    if Self::is_non_retryable_error(&e) {
                        return Err(e);
                    }

                    if attempt < MAX_ATTEMPTS {
                        let backoff_ms = INITIAL_BACKOFF_MS * (1 << (attempt - 1));
                        warn!(
                            "Download attempt {} failed: {}. Retrying in {}ms...",
                            attempt, e, backoff_ms
                        );
                        eprintln!(
                            "  Download attempt {} failed. Retrying in {}s...",
                            attempt,
                            backoff_ms / 1000
                        );
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(ModelDownloadError::RetryExhausted {
            attempts: MAX_ATTEMPTS,
            message: format!("Failed to download {}", self.selected_model.name),
            last_error,
        })
    }

    /// Check if an error is non-retryable (e.g., model not found, disk full)
    fn is_non_retryable_error(error: &ModelDownloadError) -> bool {
        matches!(
            error,
            ModelDownloadError::ModelNotFound { .. }
                | ModelDownloadError::DiskSpace { .. }
                | ModelDownloadError::SslError { .. }
        )
    }

    /// Single download attempt
    async fn download_model_attempt(&self, dest_path: &Path) -> Result<(), ModelDownloadError> {
        use hf_hub::api::tokio::Api;
        use indicatif::{ProgressBar, ProgressStyle};

        // Create progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message(format!(
            "Connecting to Hugging Face Hub for {}...",
            self.selected_model.filename
        ));
        pb.enable_steady_tick(Duration::from_millis(100));

        // Initialize HF API
        let api = Api::new().map_err(|e| {
            let error_msg = e.to_string().to_lowercase();
            if error_msg.contains("ssl") || error_msg.contains("certificate") {
                ModelDownloadError::SslError {
                    message: e.to_string(),
                }
            } else {
                ModelDownloadError::Network {
                    message: format!("Failed to initialize Hugging Face API: {}", e),
                }
            }
        })?;

        let repo = api.model(self.selected_model.hf_repo.to_string());

        pb.set_message(format!(
            "Downloading {} (~{} MB)...",
            self.selected_model.filename, self.selected_model.size_mb
        ));

        // Download the model file
        let downloaded = repo.get(self.selected_model.filename).await.map_err(|e| {
            let error_msg = e.to_string().to_lowercase();

            if error_msg.contains("rate") || error_msg.contains("429") {
                ModelDownloadError::RateLimited
            } else if error_msg.contains("not found") || error_msg.contains("404") {
                ModelDownloadError::ModelNotFound {
                    model_id: self.selected_model.id.to_string(),
                    repo: self.selected_model.hf_repo.to_string(),
                }
            } else if error_msg.contains("ssl")
                || error_msg.contains("certificate")
                || error_msg.contains("tls")
            {
                ModelDownloadError::SslError {
                    message: e.to_string(),
                }
            } else if error_msg.contains("timeout")
                || error_msg.contains("connection")
                || error_msg.contains("network")
            {
                ModelDownloadError::Network {
                    message: e.to_string(),
                }
            } else {
                ModelDownloadError::Network {
                    message: format!("Download failed: {}", e),
                }
            }
        })?;

        pb.set_message("Copying model to cache...");

        // Copy to cache directory
        std::fs::copy(&downloaded, dest_path).map_err(|e| {
            let error_msg = e.to_string().to_lowercase();
            if error_msg.contains("space") || error_msg.contains("quota") {
                ModelDownloadError::DiskSpace {
                    message: e.to_string(),
                    required_mb: self.selected_model.size_mb,
                    cache_dir: self.cache_dir.display().to_string(),
                }
            } else {
                ModelDownloadError::FileSystem(e)
            }
        })?;

        pb.finish_with_message(format!("✓ Downloaded {}", self.selected_model.name));

        Ok(())
    }

    /// Get the path to the bundled model (if embedded in binary)
    fn bundled_model_path(&self) -> PathBuf {
        // For future: models embedded in binary distribution
        // For now, this will always return a non-existent path
        PathBuf::from("models")
            .join(self.selected_model.id)
            .join(self.selected_model.filename)
    }

    /// Get the path to the cached model
    fn cached_model_path(&self) -> PathBuf {
        self.cache_dir.join(self.selected_model.filename)
    }

    /// Get the tokenizer path (bundled with source)
    pub fn get_tokenizer_path(&self) -> PathBuf {
        PathBuf::from("models")
            .join("qwen2.5-coder-1.5b")
            .join("tokenizer.json")
    }

    /// Get the model config path (bundled with source)
    pub fn get_config_path(&self) -> PathBuf {
        PathBuf::from("models")
            .join("qwen2.5-coder-1.5b")
            .join("config.json")
    }

    /// Verify model file exists and is valid (basic check)
    pub fn verify_model(&self, model_path: &Path) -> Result<bool> {
        if !model_path.exists() {
            return Ok(false);
        }

        // Basic validation: check file size (should be > 1GB for Q4_K_M)
        let metadata =
            std::fs::metadata(model_path).context("Failed to read model file metadata")?;

        let size_mb = metadata.len() / (1024 * 1024);

        if size_mb < 900 {
            warn!(
                "Model file seems too small: {}MB (expected ~1100MB)",
                size_mb
            );
            return Ok(false);
        }

        debug!("Model file validated: {}MB", size_mb);
        Ok(true)
    }
}

impl Default for ModelLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create model loader with default cache directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_cache_dir() {
        let cache_dir = ModelLoader::default_cache_dir().unwrap();
        assert!(cache_dir.to_string_lossy().contains("caro"));
        assert!(cache_dir.to_string_lossy().contains("models"));
    }

    #[test]
    fn test_detect_platform() {
        let variant = ModelLoader::detect_platform();
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        assert_eq!(variant, ModelVariant::MLX);
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        assert_eq!(variant, ModelVariant::CPU);
    }

    #[test]
    fn test_model_loader_new() {
        let loader = ModelLoader::new();
        assert!(loader.is_ok());
        let loader = loader.unwrap();
        // Default should be qwen-1.5b-q4 unless CARO_MODEL is set
        assert!(!loader.selected_model.id.is_empty());
    }

    #[test]
    fn test_model_loader_env_var() {
        // Test with environment variable
        std::env::set_var("CARO_MODEL", "smollm-135m-q4");
        let loader = ModelLoader::new();
        assert!(loader.is_ok());
        let loader = loader.unwrap();
        assert_eq!(loader.selected_model.id, "smollm-135m-q4");
        std::env::remove_var("CARO_MODEL");
    }

    #[test]
    fn test_with_smallest_model() {
        let loader = ModelLoader::with_smallest_model();
        assert!(loader.is_ok());
        let loader = loader.unwrap();
        assert_eq!(loader.selected_model.id, "smollm-135m-q4");
    }

    #[test]
    fn test_custom_cache_dir() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ModelLoader::with_cache_dir(temp_dir.path().to_path_buf());
        assert_eq!(loader.cache_dir, temp_dir.path());
    }

    #[test]
    fn test_list_models() {
        let models = ModelLoader::list_models();
        assert!(!models.is_empty());
        assert!(models.len() >= 5);
    }

    #[test]
    fn test_list_ci_models() {
        let models = ModelLoader::list_ci_models();
        assert!(!models.is_empty());
        for model in models {
            assert!(model.ci_suitable);
        }
    }

    #[test]
    fn test_tokenizer_path() {
        let loader = ModelLoader::new().unwrap();
        let path = loader.get_tokenizer_path();
        assert!(path.to_string_lossy().contains("tokenizer.json"));
    }

    #[test]
    fn test_config_path() {
        let loader = ModelLoader::new().unwrap();
        let path = loader.get_config_path();
        assert!(path.to_string_lossy().contains("config.json"));
    }

    #[test]
    fn test_verify_model_nonexistent() {
        let loader = ModelLoader::new().unwrap();
        let result = loader.verify_model(Path::new("/nonexistent/model.gguf"));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_model_download_error_display() {
        // Test Network error
        let err = ModelDownloadError::Network {
            message: "Connection refused".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("Network error"));
        assert!(display.contains("Connection refused"));
        assert!(display.contains("Possible solutions"));

        // Test RateLimited error
        let err = ModelDownloadError::RateLimited;
        let display = err.to_string();
        assert!(display.contains("Rate limited"));
        assert!(display.contains("HF_TOKEN"));

        // Test ModelNotFound error
        let err = ModelDownloadError::ModelNotFound {
            model_id: "test-model".to_string(),
            repo: "test-org/test-repo".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("Model not found"));
        assert!(display.contains("test-model"));

        // Test DiskSpace error
        let err = ModelDownloadError::DiskSpace {
            message: "No space left".to_string(),
            required_mb: 1000,
            cache_dir: "/home/user/.cache/caro/models".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("Disk space"));
        assert!(display.contains("1000 MB"));

        // Test RetryExhausted error
        let err = ModelDownloadError::RetryExhausted {
            attempts: 3,
            message: "Download failed".to_string(),
            last_error: "Connection timeout".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("3 attempts"));
        assert!(display.contains("Connection timeout"));
    }

    #[test]
    fn test_is_non_retryable_error() {
        // ModelNotFound should not be retried
        let err = ModelDownloadError::ModelNotFound {
            model_id: "test".to_string(),
            repo: "test".to_string(),
        };
        assert!(ModelLoader::is_non_retryable_error(&err));

        // DiskSpace should not be retried
        let err = ModelDownloadError::DiskSpace {
            message: "No space".to_string(),
            required_mb: 1000,
            cache_dir: "/tmp".to_string(),
        };
        assert!(ModelLoader::is_non_retryable_error(&err));

        // Network errors should be retried
        let err = ModelDownloadError::Network {
            message: "Connection refused".to_string(),
        };
        assert!(!ModelLoader::is_non_retryable_error(&err));

        // RateLimited can be retried (after waiting)
        let err = ModelDownloadError::RateLimited;
        assert!(!ModelLoader::is_non_retryable_error(&err));
    }
}
