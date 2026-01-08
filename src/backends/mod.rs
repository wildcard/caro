// Backends module - LLM backend trait and implementations
// These are placeholder stubs - tests should fail until proper implementation

pub mod embedded;
#[cfg(feature = "remote-backends")]
pub mod remote;
pub mod static_matcher;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    #[error("Backend is not available: {reason}")]
    BackendUnavailable { reason: String },

    #[error("Request timeout after {timeout:?}")]
    Timeout { timeout: Duration },

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("Model generation failed: {details}")]
    GenerationFailed { details: String },

    #[error("Response parsing failed: {content}")]
    ParseError { content: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Unsafe command detected: {reason}")]
    Unsafe {
        reason: String,
        risk_level: crate::models::RiskLevel,
        warnings: Vec<String>,
    },

    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },
}

// Types are already public, no re-export needed

// Re-export static matcher
pub use static_matcher::StaticMatcher;
