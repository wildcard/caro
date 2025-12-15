// Model loading and distribution strategy for embedded models

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::backends::embedded::ModelVariant;

/// Default model filename for Q4_K_M quantization (recommended)
const DEFAULT_MODEL_Q4: &str = "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf";

/// Alternative model filename for Q8_0 quantization (higher quality)
#[allow(dead_code)]
const DEFAULT_MODEL_Q8: &str = "qwen2.5-coder-1.5b-instruct-q8_0.gguf";

/// Hugging Face model repository for GGUF files
const HF_MODEL_REPO: &str = "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF";

/// Model loader for managing embedded model distribution and caching
#[derive(Clone)]
pub struct ModelLoader {
    cache_dir: PathBuf,
}

impl ModelLoader {
    /// Create a new model loader with default cache directory
    pub fn new() -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        Ok(Self { cache_dir })
    }

    /// Create a model loader with a custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get the default cache directory for models
    /// Returns: ~/.cache/cmdai/models/ on Unix, %LOCALAPPDATA%\cmdai\models\ on Windows
    pub fn default_cache_dir() -> Result<PathBuf> {
        let cache_base = directories::BaseDirs::new()
            .context("Failed to determine user cache directory")?
            .cache_dir()
            .to_path_buf();

        let cmdai_cache = cache_base.join("cmdai").join("models");

        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&cmdai_cache).context("Failed to create model cache directory")?;

        Ok(cmdai_cache)
    }

    /// Auto-detect the best available model variant for the current platform
    pub fn detect_platform() -> ModelVariant {
        ModelVariant::detect()
    }

    /// Get the embedded model path (bundled with binary or from cache)
    ///
    /// Priority:
    /// 1. Check bundled model path (models/qwen2.5-coder-1.5b/*.gguf)
    /// 2. Check cache directory (~/.cache/cmdai/models/*.gguf)
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
            "Model not found, will download to: {}",
            cached_path.display()
        );
        Ok(cached_path)
    }

    /// Download model from Hugging Face Hub if missing
    ///
    /// This will download the Q4_K_M quantized model (~1.1GB) from Hugging Face
    /// and save it to the cache directory.
    pub async fn download_model_if_missing(&self, variant: ModelVariant) -> Result<PathBuf> {
        let model_path = self.get_embedded_model_path()?;

        if model_path.exists() {
            debug!("Model already exists at: {}", model_path.display());
            return Ok(model_path);
        }

        info!("Downloading model from Hugging Face Hub...");
        self.download_model(&model_path, variant).await?;

        Ok(model_path)
    }

    /// Download the model from Hugging Face Hub
    async fn download_model(&self, dest_path: &Path, _variant: ModelVariant) -> Result<()> {
        use hf_hub::api::tokio::Api;

        let api = Api::new().context("Failed to initialize Hugging Face API")?;
        let repo = api.model(HF_MODEL_REPO.to_string());

        info!("Downloading {} from Hugging Face Hub...", DEFAULT_MODEL_Q4);
        info!("This may take a few minutes (~1.1GB)...");

        // Download the model file
        let downloaded = repo
            .get(DEFAULT_MODEL_Q4)
            .await
            .context("Failed to download model from Hugging Face Hub")?;

        // Copy to cache directory
        std::fs::copy(&downloaded, dest_path).context("Failed to copy model to cache directory")?;

        info!("Model downloaded successfully to: {}", dest_path.display());

        Ok(())
    }

    /// Get the path to the bundled model (if embedded in binary)
    fn bundled_model_path(&self) -> PathBuf {
        // For future: models embedded in binary distribution
        // For now, this will always return a non-existent path
        PathBuf::from("models")
            .join("qwen2.5-coder-1.5b")
            .join(DEFAULT_MODEL_Q4)
    }

    /// Get the path to the cached model
    fn cached_model_path(&self) -> PathBuf {
        self.cache_dir.join(DEFAULT_MODEL_Q4)
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
        assert!(cache_dir.to_string_lossy().contains("cmdai"));
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
    }

    #[test]
    fn test_custom_cache_dir() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ModelLoader::with_cache_dir(temp_dir.path().to_path_buf());
        assert_eq!(loader.cache_dir, temp_dir.path());
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
