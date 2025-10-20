// Common types and traits for embedded model backends

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::backends::GeneratorError;

/// Model variant selection for embedded inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelVariant {
    /// MLX GPU backend for Apple Silicon (macOS aarch64 only)
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    MLX,
    /// Candle CPU backend for cross-platform fallback
    CPU,
}

impl ModelVariant {
    /// Auto-detect the best available model variant for the current platform
    pub fn detect() -> Self {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            Self::MLX
        }
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            Self::CPU
        }
    }
}

impl std::fmt::Display for ModelVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            ModelVariant::MLX => write!(f, "MLX"),
            ModelVariant::CPU => write!(f, "CPU"),
        }
    }
}

/// Configuration for embedded model inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedConfig {
    pub temperature: f32,
    pub max_tokens: usize,
    pub top_p: f32,
    pub stop_tokens: Vec<String>,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 100,
            top_p: 0.9,
            stop_tokens: vec!["\n\n".to_string(), "```".to_string()],
        }
    }
}

impl EmbeddedConfig {
    /// Builder: Set temperature (0.0-2.0)
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 2.0);
        self
    }

    /// Builder: Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Builder: Set top_p
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set stop tokens
    pub fn with_stop_tokens(mut self, stop_tokens: Vec<String>) -> Self {
        self.stop_tokens = stop_tokens;
        self
    }
}

/// Internal trait for platform-specific inference backends (MLX, Candle)
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Run inference with the given prompt and config
    async fn infer(&self, prompt: &str, config: &EmbeddedConfig) -> Result<String, GeneratorError>;

    /// Get the model variant this backend implements
    fn variant(&self) -> ModelVariant;

    /// Load the model into memory (lazy loading support)
    async fn load(&mut self) -> Result<(), GeneratorError>;

    /// Unload the model and release resources
    async fn unload(&mut self) -> Result<(), GeneratorError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_variant_detect() {
        let variant = ModelVariant::detect();
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        assert_eq!(variant, ModelVariant::MLX);
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        assert_eq!(variant, ModelVariant::CPU);
    }

    #[test]
    fn test_embedded_config_default() {
        let config = EmbeddedConfig::default();
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 100);
        assert_eq!(config.top_p, 0.9);
        assert_eq!(config.stop_tokens.len(), 2);
    }

    #[test]
    fn test_embedded_config_builder() {
        let config = EmbeddedConfig::default()
            .with_temperature(0.5)
            .with_max_tokens(200)
            .with_top_p(0.95);

        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 200);
        assert_eq!(config.top_p, 0.95);
    }

    #[test]
    fn test_temperature_clamping() {
        let config = EmbeddedConfig::default().with_temperature(5.0);
        assert_eq!(config.temperature, 2.0); // Clamped to max

        let config = EmbeddedConfig::default().with_temperature(-1.0);
        assert_eq!(config.temperature, 0.0); // Clamped to min
    }
}
