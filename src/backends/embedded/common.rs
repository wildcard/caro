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

/// Default number of parse retries before giving up
fn default_max_parse_retries() -> u32 {
    2
}

/// Configuration for embedded model inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedConfig {
    pub temperature: f32,
    pub max_tokens: usize,
    pub top_p: f32,
    pub stop_tokens: Vec<String>,
    /// Maximum number of retries when LLM output fails to parse as valid JSON.
    /// Set to 0 to disable retries (single attempt only). Clamped to 0..=5.
    #[serde(default = "default_max_parse_retries")]
    pub max_parse_retries: u32,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        Self {
            // Lower temperature (0.1) for more deterministic command generation
            // Previous value (0.7) was too high, causing variability in output
            temperature: 0.1,
            max_tokens: 100,
            top_p: 0.9,
            stop_tokens: vec!["\n\n".to_string(), "```".to_string()],
            max_parse_retries: default_max_parse_retries(),
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

    /// Builder: Set max parse retries (0..=5). Set to 0 to disable retries.
    pub fn with_max_parse_retries(mut self, retries: u32) -> Self {
        self.max_parse_retries = retries.min(5);
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
        assert_eq!(config.temperature, 0.1); // Updated for deterministic command generation
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

    #[test]
    fn test_parse_retries_config_default() {
        let config = EmbeddedConfig::default();
        assert_eq!(config.max_parse_retries, 2);
    }

    #[test]
    fn test_parse_retries_config_clamp_max() {
        let config = EmbeddedConfig::default().with_max_parse_retries(100);
        assert_eq!(config.max_parse_retries, 5);
    }

    #[test]
    fn test_parse_retries_config_zero() {
        let config = EmbeddedConfig::default().with_max_parse_retries(0);
        assert_eq!(config.max_parse_retries, 0);
    }

    #[test]
    fn test_parse_retries_serde_default() {
        // When deserializing without max_parse_retries field, should use default
        let json = r#"{"temperature": 0.1, "max_tokens": 100, "top_p": 0.9, "stop_tokens": []}"#;
        let config: EmbeddedConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_parse_retries, 2);
    }
}
