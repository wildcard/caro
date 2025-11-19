// Backends module - LLM backend trait and implementations
// These are placeholder stubs - tests should fail until proper implementation

pub mod embedded;
#[cfg(feature = "remote-backends")]
pub mod remote;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::models::{BackendType, CommandRequest, GeneratedCommand};

/// Core trait that all command generation backends must implement
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    /// Generate a shell command from natural language input
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError>;

    /// Check if this backend is currently available for use
    async fn is_available(&self) -> bool;

    /// Get information about this backend's capabilities and performance
    fn backend_info(&self) -> BackendInfo;

    /// Perform any necessary cleanup when shutting down
    async fn shutdown(&self) -> Result<(), GeneratorError>;
}

/// Backend capability and performance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    pub backend_type: BackendType,
    pub model_name: String,
    pub supports_streaming: bool,
    pub max_tokens: u32,
    pub typical_latency_ms: u64,
    pub memory_usage_mb: u64,
    pub version: String,
}

/// Errors that can occur during command generation
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum GeneratorError {
    #[error("Backend '{backend}' is not available: {reason}\n\nSuggestion: {suggestion}")]
    BackendUnavailable {
        backend: String,
        reason: String,
        suggestion: String,
    },

    #[error("Request timeout after {timeout_secs}s\n\nSuggestion: The backend may be overloaded or unresponsive.\nTry:\n  • Using a different backend: --backend {suggested_backend}\n  • Increasing timeout: --timeout {suggested_timeout_secs}\n  • Simplifying your request")]
    Timeout {
        timeout_secs: u64,
        suggested_backend: String,
        suggested_timeout_secs: u64,
    },

    #[error("Invalid request: {message}\n\nSuggestion: {suggestion}")]
    InvalidRequest {
        message: String,
        suggestion: String,
    },

    #[error("Model generation failed: {details}\n\nSuggestion: {suggestion}")]
    GenerationFailed {
        details: String,
        suggestion: String,
    },

    #[error("Response parsing failed: {reason}\nReceived: {content}\n\nSuggestion: This may be a bug in the backend or model.\nTry:\n  • Using a different model: --model {suggested_model}\n  • Reporting this issue with the full error details")]
    ParseError {
        reason: String,
        content: String,
        suggested_model: String,
    },

    #[error("Configuration error: {message}\n\nSuggestion: {suggestion}")]
    ConfigError {
        message: String,
        suggestion: String,
    },

    #[error("Network error: {message}\n\nSuggestion: {suggestion}")]
    NetworkError {
        message: String,
        suggestion: String,
    },

    #[error("Internal error: {message}\n\nThis is likely a bug. Please report it with these details:\n  Error: {message}\n  Version: cmdai {version}")]
    Internal { message: String, version: String },
}

impl GeneratorError {
    /// Create a backend unavailable error with helpful suggestions
    pub fn backend_unavailable(backend: &str, reason: &str) -> Self {
        let suggestion = match backend {
            "ollama" => {
                "Ensure Ollama is running:\n  ollama serve\n\nOr use a different backend:\n  --backend embedded"
            }
            "vllm" => {
                "Ensure vLLM server is running.\nOr use a different backend:\n  --backend embedded"
            }
            "mlx" => {
                "MLX backend requires Apple Silicon.\nUse embedded backend instead:\n  --backend embedded"
            }
            "embedded" => {
                "The embedded model may not be cached.\nDownload it first:\n  cmdai cache download qwen2.5-coder-1.5b"
            }
            _ => "Try using a different backend:\n  --backend embedded",
        };

        Self::BackendUnavailable {
            backend: backend.to_string(),
            reason: reason.to_string(),
            suggestion: suggestion.to_string(),
        }
    }

    /// Create a timeout error with suggestions
    pub fn timeout(timeout_secs: u64) -> Self {
        let suggested_backend = if timeout_secs < 30 {
            "embedded".to_string()
        } else {
            "vllm".to_string()
        };

        Self::Timeout {
            timeout_secs,
            suggested_backend,
            suggested_timeout_secs: timeout_secs * 2,
        }
    }

    /// Create an invalid request error with suggestions
    pub fn invalid_request(message: &str) -> Self {
        let suggestion = if message.contains("empty") {
            "Provide a description of the command you want to generate.".to_string()
        } else if message.contains("too long") {
            "Shorten your request to focus on the essential requirements.".to_string()
        } else {
            "Check that your request is valid and try again.".to_string()
        };

        Self::InvalidRequest {
            message: message.to_string(),
            suggestion,
        }
    }

    /// Create a generation failed error with suggestions
    pub fn generation_failed(details: &str) -> Self {
        let suggestion = if details.contains("CUDA") || details.contains("GPU") {
            "The model may require GPU resources that are unavailable.\nTry:\n  • Using the embedded backend: --backend embedded\n  • Using a smaller model"
                .to_string()
        } else if details.contains("memory") || details.contains("OOM") {
            "The model ran out of memory.\nTry:\n  • Using a smaller model\n  • Closing other applications\n  • Using a remote backend with more resources"
                .to_string()
        } else {
            "The model failed to generate a response.\nTry:\n  • Simplifying your request\n  • Using a different backend or model"
                .to_string()
        };

        Self::GenerationFailed {
            details: details.to_string(),
            suggestion,
        }
    }

    /// Create a parse error with suggestions
    pub fn parse_error(reason: &str, content: &str) -> Self {
        let truncated_content = if content.len() > 200 {
            format!("{}... (truncated)", &content[..200])
        } else {
            content.to_string()
        };

        Self::ParseError {
            reason: reason.to_string(),
            content: truncated_content,
            suggested_model: "qwen2.5-coder-1.5b".to_string(),
        }
    }

    /// Create a config error with suggestions
    pub fn config_error(message: &str) -> Self {
        let suggestion = if message.contains("not found") {
            "Initialize configuration:\n  cmdai config init".to_string()
        } else if message.contains("invalid") {
            "Fix the configuration file or reset to defaults:\n  cmdai config reset".to_string()
        } else {
            "Check your configuration file for errors.".to_string()
        };

        Self::ConfigError {
            message: message.to_string(),
            suggestion,
        }
    }

    /// Create a network error with suggestions
    pub fn network_error(message: &str) -> Self {
        let suggestion = if message.contains("timeout") {
            "Check your internet connection and try again.\nIf behind a proxy, set HTTPS_PROXY:\n  export HTTPS_PROXY=http://proxy.example.com:8080"
                .to_string()
        } else if message.contains("refused") {
            "Ensure the backend server is running and accessible.".to_string()
        } else if message.contains("DNS") || message.contains("resolve") {
            "Check your DNS settings and internet connection.".to_string()
        } else {
            "Check your network connection and try again.".to_string()
        };

        Self::NetworkError {
            message: message.to_string(),
            suggestion,
        }
    }

    /// Create an internal error with version info
    pub fn internal(message: &str) -> Self {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Self::Internal {
            message: message.to_string(),
            version,
        }
    }
}

// Types are already public, no re-export needed

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages_have_suggestions() {
        // Test backend unavailable errors
        let error = GeneratorError::backend_unavailable("ollama", "service not running");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("ollama serve"));

        let error = GeneratorError::backend_unavailable("embedded", "model not cached");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("cmdai cache download"));

        // Test timeout error
        let error = GeneratorError::timeout(30);
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("timeout"));

        // Test invalid request error
        let error = GeneratorError::invalid_request("Input is empty");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));

        // Test generation failed error with GPU mention
        let error = GeneratorError::generation_failed("CUDA out of memory");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("embedded"));

        // Test parse error
        let error = GeneratorError::parse_error("Invalid JSON", "{malformed}");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("model"));

        // Test config error
        let error = GeneratorError::config_error("Config file not found");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("cmdai config init"));

        // Test network error
        let error = GeneratorError::network_error("Connection timeout");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Suggestion"));
        assert!(error_msg.contains("HTTPS_PROXY"));

        // Test internal error
        let error = GeneratorError::internal("Unexpected panic");
        let error_msg = error.to_string();
        assert!(error_msg.contains("bug"));
        assert!(error_msg.contains("Version"));
    }

    #[test]
    fn test_parse_error_truncates_long_content() {
        let long_content = "x".repeat(500);
        let error = GeneratorError::parse_error("Invalid format", &long_content);
        let error_msg = error.to_string();

        // Should be truncated
        assert!(error_msg.contains("truncated"));
        assert!(error_msg.len() < long_content.len());
    }

    #[test]
    fn test_backend_specific_suggestions() {
        // MLX backend suggestion
        let error = GeneratorError::backend_unavailable("mlx", "not supported on this platform");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Apple Silicon"));
        assert!(error_msg.contains("embedded"));

        // vLLM backend suggestion
        let error = GeneratorError::backend_unavailable("vllm", "server not responding");
        let error_msg = error.to_string();
        assert!(error_msg.contains("vLLM server"));
    }

    #[test]
    fn test_timeout_adjusts_suggestions() {
        // Short timeout
        let error = GeneratorError::timeout(10);
        let error_msg = error.to_string();
        assert!(error_msg.contains("embedded"));

        // Long timeout
        let error = GeneratorError::timeout(60);
        let error_msg = error.to_string();
        assert!(error_msg.contains("vllm") || error_msg.contains("timeout"));
    }

    #[test]
    fn test_generation_failed_context_awareness() {
        // Memory error
        let error = GeneratorError::generation_failed("Out of memory (OOM)");
        let error_msg = error.to_string();
        assert!(error_msg.contains("memory"));
        assert!(error_msg.contains("smaller model"));

        // GPU error
        let error = GeneratorError::generation_failed("CUDA error: device not found");
        let error_msg = error.to_string();
        assert!(error_msg.contains("GPU"));

        // Generic error
        let error = GeneratorError::generation_failed("Unknown failure");
        let error_msg = error.to_string();
        assert!(error_msg.contains("Simplifying your request"));
    }
}
