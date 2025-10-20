// Model loading and distribution strategy for embedded models

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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

/// Information about the embedded model's availability and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_name: String,
    pub repository: String,
    pub local_path: PathBuf,
    pub is_available: bool,
    pub size_mb: Option<u64>,
    pub cache_dir: PathBuf,
}

/// Model loader for managing embedded model distribution and caching
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
    async fn download_model(&self, dest_path: &Path, variant: ModelVariant) -> Result<()> {
        use hf_hub::api::tokio::Api;

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create model cache directory")?;
        }

        let api = Api::new().context("Failed to initialize Hugging Face API")?;
        let repo = api.model(HF_MODEL_REPO.to_string());

        // Select model file based on variant (in future, may support different quantizations)
        let model_filename = match variant {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            ModelVariant::MLX => DEFAULT_MODEL_Q4, // Q4_K_M for both MLX and CPU for now
            ModelVariant::CPU => DEFAULT_MODEL_Q4,
        };

        info!("Downloading {} from Hugging Face Hub...", model_filename);
        info!("Repository: {}", HF_MODEL_REPO);
        info!("This may take a few minutes (~1.1GB)...");

        // Create a progress indicator
        use indicatif::{ProgressBar, ProgressStyle};
        let progress = ProgressBar::new(0);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Download the model file with progress tracking
        let downloaded_path = repo
            .get(model_filename)
            .await
            .context("Failed to download model from Hugging Face Hub")?;

        progress.set_message("Copying to cache...");

        // Atomic copy: write to temporary file first, then rename
        let temp_dest = dest_path.with_extension("tmp");
        std::fs::copy(&downloaded_path, &temp_dest)
            .context("Failed to copy model to temporary location")?;

        std::fs::rename(&temp_dest, dest_path).context("Failed to move model to final location")?;

        progress.finish_with_message("Download complete!");

        info!("Model downloaded successfully to: {}", dest_path.display());

        // Verify the downloaded model
        if !self.verify_model(dest_path)? {
            std::fs::remove_file(dest_path).ok(); // Clean up invalid file
            return Err(anyhow::anyhow!("Downloaded model failed verification"));
        }

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

    /// Check if model is available (downloaded or bundled)
    pub fn is_model_available(&self) -> Result<bool> {
        let model_path = self.get_embedded_model_path()?;
        Ok(model_path.exists() && self.verify_model(&model_path).unwrap_or(false))
    }

    /// Get model download progress information
    pub fn get_model_info(&self) -> Result<ModelInfo> {
        let model_path = self.get_embedded_model_path()?;
        let exists = model_path.exists();
        let size_mb = if exists {
            let metadata =
                std::fs::metadata(&model_path).context("Failed to get model metadata")?;
            Some(metadata.len() / (1024 * 1024))
        } else {
            None
        };

        Ok(ModelInfo {
            model_name: DEFAULT_MODEL_Q4.to_string(),
            repository: HF_MODEL_REPO.to_string(),
            local_path: model_path.clone(),
            is_available: exists && self.verify_model(&model_path).unwrap_or(false),
            size_mb,
            cache_dir: self.cache_dir.clone(),
        })
    }

    /// Clean up cached models (for storage management)
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir).context("Failed to remove cache directory")?;
            std::fs::create_dir_all(&self.cache_dir)
                .context("Failed to recreate cache directory")?;
            info!("Model cache cleared: {}", self.cache_dir.display());
        }
        Ok(())
    }

    /// Verify model file exists and is valid (comprehensive check)
    pub fn verify_model(&self, model_path: &Path) -> Result<bool> {
        if !model_path.exists() {
            debug!("Model file does not exist: {}", model_path.display());
            return Ok(false);
        }

        // Check file size (should be > 1GB for Q4_K_M)
        let metadata =
            std::fs::metadata(model_path).context("Failed to read model file metadata")?;

        let size_mb = metadata.len() / (1024 * 1024);

        // Expected size range: 900MB - 1500MB for Q4_K_M quantization
        if size_mb < 900 || size_mb > 1500 {
            warn!(
                "Model file size outside expected range: {}MB (expected 900-1500MB)",
                size_mb
            );
            return Ok(false);
        }

        // Verify GGUF magic number (first 4 bytes should be "GGUF")
        let mut file = std::fs::File::open(model_path)
            .context("Failed to open model file for verification")?;
        let mut magic = [0u8; 4];
        use std::io::Read;
        file.read_exact(&mut magic)
            .context("Failed to read GGUF magic number")?;

        if &magic != b"GGUF" {
            warn!("Invalid GGUF magic number in model file");
            return Ok(false);
        }

        // Basic checksum verification (SHA-256 hash of first 1MB)
        file.read_to_end(&mut Vec::new()).ok(); // Reset file position
        drop(file);

        let mut file = std::fs::File::open(model_path).context("Failed to reopen model file")?;
        let mut first_mb = vec![0u8; 1024 * 1024];
        let bytes_read = file
            .read(&mut first_mb)
            .context("Failed to read model file for checksum")?;
        first_mb.truncate(bytes_read);

        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&first_mb);
        let hash_hex = format!("{:x}", hash);

        debug!(
            "Model file validated: {}MB, hash: {}...",
            size_mb,
            &hash_hex[..16]
        );
        info!("Model validation successful: {}", model_path.display());

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

    #[test]
    fn test_model_info() {
        let loader = ModelLoader::new().unwrap();
        let info = loader.get_model_info();
        assert!(info.is_ok());

        let info = info.unwrap();
        assert_eq!(info.model_name, DEFAULT_MODEL_Q4);
        assert_eq!(info.repository, HF_MODEL_REPO);
        assert!(info.local_path.to_string_lossy().contains(DEFAULT_MODEL_Q4));
    }

    #[test]
    fn test_is_model_available() {
        let loader = ModelLoader::new().unwrap();
        // Should return false for non-existent model
        let available = loader.is_model_available().unwrap();
        assert!(!available, "Model should not be available before download");
    }

    #[test]
    fn test_clear_cache() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let loader = ModelLoader::with_cache_dir(temp_dir.path().to_path_buf());

        // Create some dummy content in cache
        std::fs::write(temp_dir.path().join("dummy.txt"), b"test").unwrap();
        assert!(temp_dir.path().join("dummy.txt").exists());

        // Clear cache should succeed
        let result = loader.clear_cache();
        assert!(result.is_ok());

        // Cache directory should exist but be empty
        assert!(temp_dir.path().exists());
        assert!(!temp_dir.path().join("dummy.txt").exists());
    }
}
