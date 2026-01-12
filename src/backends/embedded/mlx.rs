// MLX GPU-accelerated inference for Apple Silicon via llama.cpp
// Only compiles on macOS aarch64

#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::backends::embedded::common::{EmbeddedConfig, InferenceBackend, ModelVariant};
use crate::backends::GeneratorError;

#[cfg(feature = "embedded-mlx")]
use llama_cpp::{
    standard_sampler::{SamplerStage, StandardSampler},
    LlamaModel, LlamaParams, SessionParams,
};

/// MLX-backed inference state using llama.cpp with Metal acceleration
struct MlxModelState {
    #[cfg(feature = "embedded-mlx")]
    model: LlamaModel,
    #[cfg(not(feature = "embedded-mlx"))]
    _loaded: bool,
}

/// MLX backend for Apple Silicon GPU acceleration
#[allow(dead_code)]
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

#[cfg(feature = "embedded-mlx")]
fn extract_json_command(text: &str) -> Result<String, GeneratorError> {
    // Try to find JSON in the response
    if let Some(start) = text.find('{') {
        if let Some(end) = text[start..].find('}') {
            let json_str = &text[start..start + end + 1];

            // Parse and validate JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                    return Ok(format!(r#"{{"cmd": "{}"}}"#, cmd));
                }
            }
        }
    }

    // Fallback: return safe default if no valid JSON found
    Ok(r#"{"cmd": "echo 'Unable to generate command'"}"#.to_string())
}

#[cfg(feature = "embedded-mlx")]
fn build_prompt(system_prompt: &str) -> String {
    // The system_prompt from embedded_backend already contains:
    // - Detailed system instructions with examples
    // - The user's request embedded as "Request: {input}"
    // We format it as a ChatML conversation using the passed prompt as the system message
    format!(
        r#"<|im_start|>system
{}
<|im_end|>
<|im_start|>assistant
"#,
        system_prompt
    )
}

#[async_trait]
impl InferenceBackend for MlxBackend {
    #[cfg(feature = "embedded-mlx")]
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError> {
        // Build the full prompt with chat template
        let full_prompt = build_prompt(prompt);

        tracing::debug!(
            "Starting MLX inference (prompt: {} chars, max_tokens: {}, temperature: {})",
            full_prompt.len(),
            config.max_tokens,
            config.temperature
        );

        // Create session parameters
        let session_params = SessionParams {
            n_ctx: 2048,  // Context window
            n_batch: 512, // Batch size for prompt processing
            n_threads: 8, // Use multiple threads
            ..Default::default()
        };

        // Clone model for this inference (Arc internally, cheap)
        // Do this in a separate scope to release the lock before await
        let model = {
            let model_state_guard =
                self.model_state
                    .lock()
                    .map_err(|_| GeneratorError::Internal {
                        message: "Failed to acquire model state lock".to_string(),
                    })?;

            let model_state =
                model_state_guard
                    .as_ref()
                    .ok_or_else(|| GeneratorError::GenerationFailed {
                        details: "Model not loaded. Call load() first".to_string(),
                    })?;

            model_state.model.clone()
        }; // Lock released here

        let max_tokens = config.max_tokens;
        let temperature = config.temperature;

        // Run inference in blocking task (llama.cpp is blocking)
        let response_text = tokio::task::spawn_blocking(move || {
            // Create session
            let mut ctx = model.create_session(session_params).map_err(|e| {
                GeneratorError::GenerationFailed {
                    details: format!("Failed to create session: {}", e),
                }
            })?;

            // Advance context with the prompt
            ctx.advance_context(&full_prompt)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to advance context: {}", e),
                })?;

            // Create sampler with custom temperature
            let sampler = StandardSampler::new_softmax(
                vec![
                    SamplerStage::RepetitionPenalty {
                        repetition_penalty: 1.1,
                        frequency_penalty: 0.0,
                        presence_penalty: 0.0,
                        last_n: 64,
                    },
                    SamplerStage::TopK(40),
                    SamplerStage::TopP(0.95),
                    SamplerStage::MinP(0.05),
                    SamplerStage::Temperature(temperature),
                ],
                1, // min_keep
            );

            // Start completion
            let completions = ctx
                .start_completing_with(sampler, max_tokens)
                .map_err(|e| GeneratorError::GenerationFailed {
                    details: format!("Failed to start completion: {}", e),
                })?
                .into_strings();

            // Collect tokens
            let mut output = String::new();
            for completion_token in completions {
                output.push_str(&completion_token);
                // Stop if we have a complete JSON response
                if output.contains('}') && output.contains("cmd") {
                    break;
                }
            }

            Ok::<String, GeneratorError>(output)
        })
        .await
        .map_err(|e| GeneratorError::Internal {
            message: format!("Inference task failed: {}", e),
        })??;

        tracing::debug!("MLX inference completed, raw response: {}", response_text);

        // Extract JSON command from response
        extract_json_command(&response_text)
    }

    #[cfg(not(feature = "embedded-mlx"))]
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError> {
        // Stub implementation when feature is disabled
        let _ = (prompt, config);
        Err(GeneratorError::ConfigError {
            message: "MLX backend not enabled. Rebuild with --features embedded-mlx".to_string(),
        })
    }

    fn variant(&self) -> ModelVariant {
        ModelVariant::MLX
    }

    #[cfg(feature = "embedded-mlx")]
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
        }

        // Check if model file exists
        if !self.model_path.exists() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("Model file not found: {}", self.model_path.display()),
            });
        }

        tracing::info!("Loading MLX model from {}...", self.model_path.display());

        // Load model (blocking operation)
        let model_path = self.model_path.clone();
        let model = tokio::task::spawn_blocking(move || {
            // Create model parameters with Metal acceleration
            let params = LlamaParams {
                n_gpu_layers: 99, // Use all GPU layers (Metal acceleration)
                use_mmap: true,   // Use memory mapping for faster loading
                use_mlock: false, // Don't lock memory
                ..Default::default()
            };

            // Load the model
            LlamaModel::load_from_file(model_path, params).map_err(|e| {
                GeneratorError::GenerationFailed {
                    details: format!("Failed to load model: {}", e),
                }
            })
        })
        .await
        .map_err(|e| GeneratorError::Internal {
            message: format!("Model loading task failed: {}", e),
        })??;

        // Store the loaded model state
        {
            let mut model_state =
                self.model_state
                    .lock()
                    .map_err(|_| GeneratorError::Internal {
                        message: "Failed to acquire model state lock".to_string(),
                    })?;
            *model_state = Some(MlxModelState { model });
        }

        tracing::info!("MLX model loaded successfully with Metal acceleration");
        Ok(())
    }

    #[cfg(not(feature = "embedded-mlx"))]
    async fn load(&mut self) -> Result<(), GeneratorError> {
        Err(GeneratorError::ConfigError {
            message: "MLX backend not enabled. Rebuild with --features embedded-mlx".to_string(),
        })
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
        }

        // Unload the model
        {
            let mut model_state =
                self.model_state
                    .lock()
                    .map_err(|_| GeneratorError::Internal {
                        message: "Failed to acquire model state lock".to_string(),
                    })?;
            *model_state = None;
        }

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

    #[cfg(feature = "embedded-mlx")]
    #[test]
    fn test_extract_json_command() {
        let text = r#"Sure! Here's the command: {"cmd": "ls -la"}"#;
        let result = extract_json_command(text);
        assert!(result.is_ok());
        assert!(result.unwrap().contains(r#""cmd": "ls -la""#));
    }

    #[cfg(feature = "embedded-mlx")]
    #[test]
    fn test_build_prompt() {
        let system_prompt = "You are a shell command generator.\nRequest: list files";
        let prompt = build_prompt(system_prompt);
        assert!(prompt.contains("list files"));
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("<|im_start|>assistant"));
        assert!(prompt.contains("shell command generator"));
    }
}
