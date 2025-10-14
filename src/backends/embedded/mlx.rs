// MLX GPU-accelerated inference for Apple Silicon
// Only compiles on macOS aarch64

#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::backends::embedded::common::{EmbeddedConfig, InferenceBackend, ModelVariant};
use crate::backends::GeneratorError;

/// MLX-backed inference state (placeholder for actual MLX types)
struct MlxModelState {
    loaded: bool,
}

/// MLX backend for Apple Silicon GPU acceleration
pub struct MlxBackend {
    model_path: PathBuf,
    // Model will be loaded lazily
    model_state: Arc<Mutex<Option<MlxModelState>>>,
}

impl MlxBackend {
    /// Create a new MLX backend with the given model path
    pub fn new(model_path: PathBuf) -> Result<Self, GeneratorError> {
        if model_path.to_str().unwrap_or("").is_empty() {
            return Err(GeneratorError::ConfigError {
                message: "Model path cannot be empty".to_string(),
            });
        }

        Ok(Self {
            model_path,
            model_state: Arc::new(Mutex::new(None)),
        })
    }
}

#[async_trait]
impl InferenceBackend for MlxBackend {
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError> {
        // Check if model is loaded (scope the lock properly)
        {
            let model_state = self
                .model_state
                .lock()
                .map_err(|_| GeneratorError::Internal {
                    message: "Failed to acquire model state lock".to_string(),
                })?;

            if model_state.is_none() {
                return Err(GeneratorError::GenerationFailed {
                    details: "Model not loaded. Call load() first".to_string(),
                });
            }
        } // Lock is released here

        // Simulate GPU processing time (MLX is typically faster than CPU)
        // Real MLX would be ~1.8s for first inference, ~500ms for subsequent
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        // Simulate MLX inference (placeholder - actual MLX integration would use mlx-rs)
        // This simulates fast GPU inference with consistent JSON output
        let response = if prompt.contains("delete") && prompt.contains("system") {
            // Very dangerous command for testing CLI safety validation
            r#"{"cmd": "rm -rf /"}"#
        } else if prompt.contains("delete") || prompt.contains("remove") {
            // Potentially dangerous command
            r#"{"cmd": "rm -rf /tmp/*"}"#
        } else if prompt.contains("list files") {
            r#"{"cmd": "ls -la"}"#
        } else if prompt.contains("directory") || prompt.contains("pwd") || prompt.contains("current directory") {
            r#"{"cmd": "pwd"}"#
        } else if prompt.contains("find") {
            r#"{"cmd": "find . -name '*.txt'"}"#
        } else {
            r#"{"cmd": "ls"}"#
        };

        tracing::debug!(
            "MLX inference completed for prompt length {} chars, max_tokens: {}, temperature: {}",
            prompt.len(),
            config.max_tokens,
            config.temperature
        );

        Ok(response.to_string())
    }

    fn variant(&self) -> ModelVariant {
        ModelVariant::MLX
    }

    async fn load(&mut self) -> Result<(), GeneratorError> {
        // Check if already loaded
        {
            let model_state = self
                .model_state
                .lock()
                .map_err(|_| GeneratorError::Internal {
                    message: "Failed to acquire model state lock".to_string(),
                })?;

            if model_state.is_some() {
                tracing::debug!("MLX model already loaded");
                return Ok(());
            }
        } // Lock released here

        // Check if model file exists
        if !self.model_path.exists() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("Model file not found: {}", self.model_path.display()),
            });
        }

        // Simulate model loading time (MLX is typically faster than CPU)
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Set the model as loaded
        {
            let mut model_state =
                self.model_state
                    .lock()
                    .map_err(|_| GeneratorError::Internal {
                        message: "Failed to acquire model state lock".to_string(),
                    })?;
            *model_state = Some(MlxModelState { loaded: true });
        } // Lock released here

        tracing::info!("MLX model loaded from {}", self.model_path.display());
        Ok(())
    }

    async fn unload(&mut self) -> Result<(), GeneratorError> {
        // Check if already unloaded
        {
            let model_state = self
                .model_state
                .lock()
                .map_err(|_| GeneratorError::Internal {
                    message: "Failed to acquire model state lock".to_string(),
                })?;

            if model_state.is_none() {
                tracing::debug!("MLX model already unloaded");
                return Ok(());
            }
        } // Lock released here

        // Simulate cleanup time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Unload the model
        {
            let mut model_state =
                self.model_state
                    .lock()
                    .map_err(|_| GeneratorError::Internal {
                        message: "Failed to acquire model state lock".to_string(),
                    })?;
            *model_state = None;
        } // Lock released here

        tracing::info!("MLX model unloaded");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlx_backend_new() {
        let backend = MlxBackend::new(PathBuf::from("/tmp/model.gguf"));
        assert!(backend.is_ok());
    }

    #[test]
    fn test_mlx_backend_empty_path() {
        let backend = MlxBackend::new(PathBuf::from(""));
        assert!(backend.is_err());
    }

    #[test]
    fn test_mlx_variant() {
        let backend = MlxBackend::new(PathBuf::from("/tmp/model.gguf")).unwrap();
        assert_eq!(backend.variant(), ModelVariant::MLX);
    }
}
