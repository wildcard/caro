// Model loading and distribution strategy for embedded models

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::backends::embedded::ModelVariant;
use crate::model_catalog::{ModelCatalog, ModelInfo};

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
    /// and save it to the cache directory with retry logic.
    pub async fn download_model_if_missing(&self, variant: ModelVariant) -> Result<PathBuf> {
        let model_path = self.get_embedded_model_path()?;

        if model_path.exists() {
            debug!("Model already exists at: {}", model_path.display());
            return Ok(model_path);
        }

        info!(
            "Downloading {} ({} MB) from Hugging Face Hub...",
            self.selected_model.name, self.selected_model.size_mb
        );
        self.download_model_with_retry(&model_path, variant).await?;

        Ok(model_path)
    }

    /// Download the model from Hugging Face Hub with retry logic
    async fn download_model_with_retry(
        &self,
        dest_path: &Path,
        variant: ModelVariant,
    ) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_DELAY_SECS: u64 = 2;

        let mut last_error = None;

        for attempt in 1..=MAX_RETRIES {
            if attempt > 1 {
                let delay = INITIAL_DELAY_SECS * 2u64.pow(attempt - 2);
                warn!(
                    "Download attempt {}/{} failed, retrying in {}s...",
                    attempt - 1,
                    MAX_RETRIES,
                    delay
                );
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }

            match self.download_model_attempt(dest_path, variant).await {
                Ok(_) => {
                    info!("Model downloaded successfully to: {}", dest_path.display());
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        // All retries exhausted - provide helpful error message
        Err(anyhow::anyhow!(
            "Failed to download model after {} attempts.\n\n\
            Troubleshooting:\n\
            1. Check internet connection:\n\
               ping huggingface.co\n\
            2. Check proxy settings:\n\
               echo $HTTP_PROXY\n\
               echo $HTTPS_PROXY\n\
            3. Try a smaller model:\n\
               export CARO_MODEL=smollm-135m-q4\n\
            4. Run diagnostics (if available):\n\
               caro doctor\n\n\
            Last error: {}",
            MAX_RETRIES,
            last_error.unwrap()
        ))
    }

    /// Single download attempt from Hugging Face Hub
    async fn download_model_attempt(&self, dest_path: &Path, _variant: ModelVariant) -> Result<()> {
        use hf_hub::api::tokio::Api;
        use indicatif::{ProgressBar, ProgressStyle};

        let api = Api::new().context("Failed to initialize Hugging Face API")?;
        let repo = api.model(self.selected_model.hf_repo.to_string());

        info!(
            "Downloading {} from {}...",
            self.selected_model.filename, self.selected_model.hf_repo
        );

        // Create progress bar
        let pb = ProgressBar::new(self.selected_model.size_mb as u64 * 1024 * 1024);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("Failed to create progress bar template")
                .progress_chars("#>-"),
        );

        // Download the model file
        let downloaded = repo
            .get(self.selected_model.filename)
            .await
            .context("Failed to download model from Hugging Face Hub")?;

        pb.finish_with_message("Download complete");

        // Copy to cache directory with progress
        let file_size = std::fs::metadata(&downloaded)?.len();
        pb.set_length(file_size);
        pb.set_position(0);
        pb.set_message("Copying to cache...");

        std::fs::copy(&downloaded, dest_path).context("Failed to copy model to cache directory")?;

        pb.finish_and_clear();

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
}
