// Candle CPU inference for cross-platform support

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::backends::embedded::common::{EmbeddedConfig, InferenceBackend, ModelVariant};
use crate::backends::GeneratorError;

use candle_core::{Device, Tensor};
use candle_core::quantized::gguf_file;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama::ModelWeights;
use tokenizers::Tokenizer;

/// Deterministic seed for reproducible generation (speed of light in m/s)
const DEFAULT_SEED: u64 = 299792458;

/// Candle-backed inference state with loaded model and tokenizer
struct CandleModelState {
    model: ModelWeights,
    tokenizer: Tokenizer,
    device: Device,
}

/// CPU backend using Candle for cross-platform inference
pub struct CpuBackend {
    model_path: PathBuf,
    // Model will be loaded lazily
    model_state: Arc<Mutex<Option<CandleModelState>>>,
}

/// Select best available device (Metal GPU on Apple Silicon, CPU otherwise)
fn select_device() -> Result<Device, GeneratorError> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "embedded-metal"))]
    {
        match Device::new_metal(0) {
            Ok(device) => {
                tracing::info!("✓ Metal GPU initialized (Apple Silicon)");
                Ok(device)
            }
            Err(e) => {
                tracing::warn!("Metal unavailable ({}), falling back to CPU", e);
                Ok(Device::Cpu)
            }
        }
    }

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64", feature = "embedded-metal")))]
    {
        tracing::info!("Using CPU device (non-Apple Silicon platform)");
        Ok(Device::Cpu)
    }
}

impl CpuBackend {
    /// Create a new CPU backend with the given model path
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
impl InferenceBackend for CpuBackend {
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError> {
        // Get mutable model state
        let mut model_state_guard = self
            .model_state
            .lock()
            .map_err(|_| GeneratorError::Internal {
                message: "Failed to acquire model state lock".to_string(),
            })?;

        let state = model_state_guard.as_mut()
            .ok_or_else(|| GeneratorError::GenerationFailed {
                details: "Model not loaded. Call load() first".to_string(),
            })?;

        // Format prompt for command generation
        let formatted_prompt = format!(
            "Generate a shell command for: {}\nRespond with JSON only: {{\"cmd\": \"your command here\"}}",
            prompt
        );

        // Tokenize input
        let tokens = state.tokenizer
            .encode(formatted_prompt.as_str(), true)
            .map_err(|e| GeneratorError::GenerationFailed {
                details: format!("Tokenization failed: {}", e),
            })?
            .get_ids()
            .to_vec();

        // Prepare input tensor
        let input = Tensor::new(&tokens[..], &state.device)
            .map_err(|e| GeneratorError::GenerationFailed {
                details: format!("Failed to create input tensor: {}", e),
            })?
            .unsqueeze(0)
            .map_err(|e| GeneratorError::GenerationFailed {
                details: format!("Failed to unsqueeze tensor: {}", e),
            })?;

        // Initialize logits processor for sampling
        let mut logits_processor = LogitsProcessor::new(
            DEFAULT_SEED,
            Some(config.temperature as f64),
            Some(config.top_p as f64),
        );

        // Generate tokens
        let mut generated_tokens = Vec::new();
        let mut current_input = input;

        for idx in 0..config.max_tokens {
            // Forward pass
            let logits = state.model.forward(&current_input, idx)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Model forward pass failed: {}", e),
                })?;

            // Sample next token
            let next_token = logits_processor
                .sample(&logits.squeeze(0).map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to squeeze logits: {}", e),
                })?)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Sampling failed: {}", e),
                })?;

            generated_tokens.push(next_token);

            // Check for EOS token (common EOS tokens: 151643, 151645, 2)
            if matches!(next_token, 151643 | 151645 | 2) {
                break;
            }

            // Check for stop tokens in generated text
            let partial_response = state.tokenizer
                .decode(&generated_tokens, true)
                .unwrap_or_default();

            if config.stop_tokens.iter().any(|stop| partial_response.contains(stop)) {
                break;
            }

            // Prepare next input (append new token)
            current_input = Tensor::new(&[next_token], &state.device)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to create next token tensor: {}", e),
                })?
                .unsqueeze(0)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to add batch dimension: {}", e),
                })?;
        }

        // Decode generated tokens
        let response = state.tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| GeneratorError::GenerationFailed {
                details: format!("Decoding failed: {}", e),
            })?;

        tracing::debug!(
            "Inference completed for prompt length {} chars, generated {} tokens, temperature: {}",
            prompt.len(),
            generated_tokens.len(),
            config.temperature
        );

        Ok(response)
    }

    fn variant(&self) -> ModelVariant {
        ModelVariant::CPU
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
                tracing::debug!("Model already loaded");
                return Ok(());
            }
        }

        // Initialize device (Metal on Apple Silicon, CPU elsewhere)
        let device = select_device()?;
        tracing::info!("Initializing Candle on device: {:?}", device);

        // Check model file exists
        if !self.model_path.exists() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("Model file not found: {}", self.model_path.display()),
            });
        }

        // Load GGUF model (blocking I/O, so run in blocking thread)
        let model_path = self.model_path.clone();
        let device_clone = device.clone();

        let (model, tokenizer) = tokio::task::spawn_blocking(move || {
            // Open model file
            let mut file = std::fs::File::open(&model_path)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to open model file: {}", e),
                })?;

            // Read GGUF content
            let content = gguf_file::Content::read(&mut file)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to read GGUF file: {}", e),
                })?;

            // Load model weights from GGUF
            let model = ModelWeights::from_gguf(content, &mut file, &device_clone)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to load model weights: {}", e),
                })?;

            // Load tokenizer
            let tokenizer_path = model_path.parent()
                .ok_or_else(|| GeneratorError::ConfigError {
                    message: "Model path has no parent directory".to_string(),
                })?
                .join("tokenizer.json");

            let tokenizer = Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to load tokenizer: {}", e),
                })?;

            Ok::<_, GeneratorError>((model, tokenizer))
        })
        .await
        .map_err(|e| GeneratorError::Internal {
            message: format!("Task join error: {}", e),
        })??;

        // Store loaded state
        {
            let mut model_state = self
                .model_state
                .lock()
                .map_err(|_| GeneratorError::Internal {
                    message: "Failed to acquire model state lock".to_string(),
                })?;
            *model_state = Some(CandleModelState {
                model,
                tokenizer,
                device,
            });
        }

        tracing::info!("Candle model loaded successfully from {}", self.model_path.display());
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
                tracing::debug!("CPU model already unloaded");
                return Ok(());
            }
        } // Lock released here

        // Simulate cleanup time
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

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

        tracing::info!("CPU model unloaded");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_backend_new() {
        let backend = CpuBackend::new(PathBuf::from("/tmp/model.gguf"));
        assert!(backend.is_ok());
    }

    #[test]
    fn test_cpu_backend_empty_path() {
        let backend = CpuBackend::new(PathBuf::from(""));
        assert!(backend.is_err());
    }

    #[test]
    fn test_cpu_variant() {
        let backend = CpuBackend::new(PathBuf::from("/tmp/model.gguf")).unwrap();
        assert_eq!(backend.variant(), ModelVariant::CPU);
    }
}
