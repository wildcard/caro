// Backends module - LLM backend trait and implementations
// These are placeholder stubs - tests should fail until proper implementation

pub mod embedded;
pub mod hybrid;
#[cfg(feature = "remote-backends")]
pub mod remote;
pub mod static_matcher;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::models::{
    BackendType, CommandRequest, GeneratedCommand, RiskJudgeContext, RiskJudgment,
};

/// The set of backend names the CLI's `--backend <name>` flag actually routes
/// to, each paired with a one-line note.
///
/// This is the **single source of truth** shared by
/// [`crate::cli::CommandLineInterface::validate_backend_name`] (the acceptor +
/// its error text) and `print_backend_info` in `main.rs` (the `--backend-info`
/// table). Iterating the same slice in both places is what keeps the two
/// user-facing rosters from drifting — the divergence tracked by
/// [#1115](https://github.com/wildcard/caro/issues/1115), where `--backend-info`
/// advertised `static`/`claude` while `--backend static`/`--backend claude`
/// hard-errored "Unknown backend".
///
/// It lists only backends the CLI can route to today. Enum variants that exist
/// in [`BackendType`] but are **not yet CLI-wired** (`claude`, `openrouter`,
/// `mlx`, `static`) are intentionally excluded so no surface advertises a name
/// that `--backend` rejects. Wiring those (and unifying the remaining help-text
/// rosters) is the larger follow-up on #1115.
pub const CLI_SERVABLE_BACKENDS: &[(&str, &str)] = &[
    (
        "embedded",
        "local LLM (MLX/CPU); downloads model on first use, no setup",
    ),
    ("ollama", "remote Ollama HTTP API (requires: ollama serve)"),
    ("exo", "Exo distributed cluster (requires: exo cluster)"),
    ("vllm", "remote vLLM HTTP API (requires: vllm server)"),
    (
        "mesh",
        "Mesh-LLM pooled mesh (requires: mesh node on :9337)",
    ),
    (
        "ai-horde",
        "AI-Horde volunteer cluster (free, public, no setup)",
    ),
    ("hybrid", "local sanitizer + remote enhancer (PII-safe)"),
];

/// Core trait that all command generation backends must implement
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    /// Generate a shell command from natural language input
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError>;

    /// Context-aware risk verdict used by `--approval smart`.
    ///
    /// The default is a no-op (`None`): backends that cannot reliably judge
    /// risk opt out, and the caller falls back to the static decision
    /// (fail-safe). Implementing this lets a backend relax benign flagged
    /// commands or escalate static-`Safe` commands it finds dangerous —
    /// always bounded by the hard floor in
    /// [`blend_smart_decision`](crate::safety::blend_smart_decision).
    async fn classify_risk(&self, _command: &str, _ctx: &RiskJudgeContext) -> Option<RiskJudgment> {
        None
    }

    /// Act as a "frontier advisor": review and improve a low-confidence draft.
    ///
    /// The default is a no-op (`None`): a backend opts out of advising, so a
    /// local worker is never used as its own advisor. A stronger/hosted backend
    /// implements this to return an improved command for the same request,
    /// informed by the local draft.
    ///
    /// This mirrors [`CommandGenerator::classify_risk`]'s opt-in, fail-safe
    /// shape. The agent loop calls it only on a low-confidence draft (sparse —
    /// the Fireworks "frontier advisor" pattern), re-validates the result
    /// through the safety validator, and keeps the local result if this returns
    /// `None` (advisor unavailable, opted out, or errored).
    async fn advise(
        &self,
        _draft: &GeneratedCommand,
        _request: &CommandRequest,
    ) -> Option<GeneratedCommand> {
        None
    }

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
