// EmbeddedModelBackend implementation for offline command generation

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;

use crate::backends::embedded::{CpuBackend, EmbeddedConfig, InferenceBackend, ModelVariant};
use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel};
use crate::ModelLoader;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::backends::embedded::MlxBackend;

/// Primary command generator using embedded Qwen model with platform-specific inference
pub struct EmbeddedModelBackend {
    model_variant: ModelVariant,
    model_path: PathBuf,
    backend: Arc<Mutex<Box<dyn InferenceBackend>>>,
    config: EmbeddedConfig,
    model_loader: ModelLoader,
}

impl EmbeddedModelBackend {
    /// Create a new embedded model backend with auto-detected platform variant
    pub fn new() -> Result<Self, GeneratorError> {
        let variant = ModelVariant::detect();
        let model_loader = ModelLoader::new().map_err(|e| GeneratorError::ConfigError {
            message: format!("Failed to initialize model loader: {}", e),
        })?;
        let model_path = model_loader.get_embedded_model_path().map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Failed to get model path: {}", e),
            }
        })?;

        Self::with_variant_and_path(variant, model_path)
    }

    /// Create a new embedded model backend with specific variant and model path
    pub fn with_variant_and_path(
        variant: ModelVariant,
        model_path: PathBuf,
    ) -> Result<Self, GeneratorError> {
        // Create the appropriate backend based on variant
        let backend: Box<dyn InferenceBackend> = match variant {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            ModelVariant::MLX => {
                Box::new(MlxBackend::new(model_path.clone()).map_err(|e| {
                    GeneratorError::ConfigError {
                        message: format!("Failed to create MLX backend: {}", e),
                    }
                })?)
            }
            ModelVariant::CPU => {
                Box::new(CpuBackend::new(model_path.clone()).map_err(|e| {
                    GeneratorError::ConfigError {
                        message: format!("Failed to create CPU backend: {}", e),
                    }
                })?)
            }
        };

        let model_loader = ModelLoader::new().map_err(|e| GeneratorError::ConfigError {
            message: format!("Failed to initialize model loader: {}", e),
        })?;

        Ok(Self {
            model_variant: variant,
            model_path,
            backend: Arc::new(Mutex::new(backend)),
            config: EmbeddedConfig::default(),
            model_loader,
        })
    }

    /// Update the embedded configuration
    pub fn with_config(mut self, config: EmbeddedConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the model variant this backend uses
    pub fn variant(&self) -> ModelVariant {
        self.model_variant
    }

    /// Get the model path
    pub fn model_path(&self) -> &PathBuf {
        &self.model_path
    }

    /// Explicitly load the model (usually not needed as loading is lazy)
    pub async fn load_model(&mut self) -> Result<(), GeneratorError> {
        // Ensure model is downloaded
        self.model_loader
            .download_model_if_missing(self.model_variant)
            .await
            .map_err(|e| GeneratorError::BackendUnavailable {
                reason: format!("Failed to download model: {}", e),
            })?;

        // Load the model in the backend
        let mut backend = self.backend.lock().await;
        backend.load().await.map_err(|e| GeneratorError::GenerationFailed {
            details: format!("Failed to load model: {}", e),
        })
    }

    /// Explicitly unload the model to free memory
    pub async fn unload_model(&mut self) -> Result<(), GeneratorError> {
        let mut backend = self.backend.lock().await;
        backend.unload().await.map_err(|e| GeneratorError::Internal {
            message: format!("Failed to unload model: {}", e),
        })
    }

    /// Generate system prompt for shell command generation
    fn create_system_prompt(&self, request: &CommandRequest) -> String {
        format!(
            r#"You are a helpful assistant that converts natural language to safe POSIX shell commands.

CRITICAL: You MUST respond with ONLY valid JSON in this exact format:
{{"cmd": "your_shell_command_here"}}

Rules:
1. Generate ONLY the shell command, no explanation
2. Use POSIX-compliant utilities (ls, find, grep, awk, sed, sort, etc.)
3. Quote file paths with spaces using double quotes
4. Target shell: {}
5. NEVER generate destructive commands (rm -rf /, mkfs, dd, etc.)
6. Keep commands simple and safe
7. If the request is unclear, generate "echo 'Please clarify your request'"

Request: {}
"#,
            request.shell, request.input
        )
    }

    /// Parse JSON response from model inference
    fn parse_command_response(&self, response: &str) -> Result<String, GeneratorError> {
        // Try structured JSON parsing first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                if !cmd.is_empty() {
                    return Ok(cmd.trim().to_string());
                }
            }
        }

        // Fallback: Try to extract JSON from response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_part = &response[start..=end];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_part) {
                    if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                        if !cmd.is_empty() {
                            return Ok(cmd.trim().to_string());
                        }
                    }
                }
            }
        }

        // Final fallback: Look for command-like patterns
        for line in response.lines() {
            let line = line.trim();
            if line.starts_with("cmd") && line.contains(':') {
                if let Some(cmd_part) = line.split(':').nth(1) {
                    let cmd = cmd_part.trim().trim_matches('"').trim_matches('\'');
                    if !cmd.is_empty() && !cmd.contains('{') && !cmd.contains('}') {
                        return Ok(cmd.to_string());
                    }
                }
            }
        }

        Err(GeneratorError::ParseError {
            content: response.to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for EmbeddedModelBackend {
    /// Generate a shell command from natural language input
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = std::time::Instant::now();

        // Ensure model is downloaded if needed
        self.model_loader
            .download_model_if_missing(self.model_variant)
            .await
            .map_err(|e| GeneratorError::BackendUnavailable {
                reason: format!("Failed to download model: {}", e),
            })?;

        // Create system prompt
        let system_prompt = self.create_system_prompt(request);

        // Acquire lock on backend and perform inference
        let mut backend = self.backend.lock().await;

        // Load model if not already loaded (lazy loading)
        backend.load().await.map_err(|e| GeneratorError::GenerationFailed {
            details: format!("Failed to load model: {}", e),
        })?;

        // Run inference
        let raw_response = backend
            .infer(&system_prompt, &self.config)
            .await
            .map_err(|e| GeneratorError::GenerationFailed {
                details: format!("Inference failed: {}", e),
            })?;

        // Parse the response
        let command = self.parse_command_response(&raw_response)?;

        let generation_time = start_time.elapsed().as_millis() as u64;

        Ok(GeneratedCommand {
            command,
            explanation: format!("Generated using {} backend", self.model_variant),
            safety_level: RiskLevel::Safe, // Default to safe - safety validation happens later
            estimated_impact: "Minimal system impact".to_string(),
            alternatives: vec![], // Embedded model generates single command
            backend_used: "embedded".to_string(),
            generation_time_ms: generation_time,
            confidence_score: 0.85, // Default confidence for embedded model
        })
    }

    /// Check if this backend is currently available for use
    async fn is_available(&self) -> bool {
        // Embedded model is always available (offline operation)
        true
    }

    /// Get information about this backend's capabilities and performance
    fn backend_info(&self) -> BackendInfo {
        let (typical_latency, memory_usage) = match self.model_variant {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            ModelVariant::MLX => (1800, 1200), // MLX: ~1.8s, ~1.2GB
            ModelVariant::CPU => (4000, 1500), // CPU: ~4s, ~1.5GB
        };

        BackendInfo {
            backend_type: BackendType::Embedded,
            model_name: "qwen2.5-coder-1.5b-instruct-q4_k_m".to_string(),
            supports_streaming: false,
            max_tokens: self.config.max_tokens as u32,
            typical_latency_ms: typical_latency,
            memory_usage_mb: memory_usage,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Perform any necessary cleanup when shutting down
    async fn shutdown(&self) -> Result<(), GeneratorError> {
        let mut backend = self.backend.lock().await;
        backend.unload().await.map_err(|e| GeneratorError::Internal {
            message: format!("Failed to unload model: {}", e),
        })?;

        tracing::debug!("Embedded model backend shutdown complete");
        Ok(())
    }
}

impl Default for EmbeddedModelBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create default embedded model backend")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShellType;

    #[test]
    fn test_embedded_backend_creation() {
        let backend = EmbeddedModelBackend::new();
        assert!(backend.is_ok(), "Should create embedded backend successfully");

        if let Ok(backend) = backend {
            // Verify variant matches platform
            let expected_variant = ModelVariant::detect();
            assert_eq!(backend.variant(), expected_variant);
        }
    }

    #[test]
    fn test_system_prompt_generation() {
        let backend = EmbeddedModelBackend::new().unwrap();
        let request = CommandRequest::new("list files", ShellType::Bash);

        let prompt = backend.create_system_prompt(&request);

        assert!(prompt.contains("list files"));
        assert!(prompt.contains("bash"));
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("cmd"));
    }

    #[test]
    fn test_json_response_parsing() {
        let backend = EmbeddedModelBackend::new().unwrap();

        // Test valid JSON
        let response = r#"{"cmd": "ls -la"}"#;
        let result = backend.parse_command_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ls -la");

        // Test JSON with extra content
        let response = r#"Here's the command: {"cmd": "find . -name '*.txt'"} - that should work!"#;
        let result = backend.parse_command_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "find . -name '*.txt'");

        // Test malformed response
        let response = "This is not JSON at all";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_available_always_true() {
        let backend = EmbeddedModelBackend::new().unwrap();
        assert!(backend.is_available().await, "Embedded backend must always be available");
    }

    #[test]
    fn test_backend_info() {
        let backend = EmbeddedModelBackend::new().unwrap();
        let info = backend.backend_info();

        assert_eq!(info.backend_type, BackendType::Embedded);
        assert_eq!(info.model_name, "qwen2.5-coder-1.5b-instruct-q4_k_m");
        assert!(!info.supports_streaming);
        assert!(info.max_tokens > 0);
        assert!(info.typical_latency_ms > 0);
        assert!(info.memory_usage_mb > 0);
    }
}