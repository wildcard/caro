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
/// in [`BackendType`] but are **not yet CLI-wired** (`mlx`, `static`) are
/// intentionally excluded so no surface advertises a name that `--backend`
/// rejects. `claude` and `openrouter` were added here once `create_backend`
/// was wired to route to them (the follow-up promised by #1115). Wiring the
/// remainder (and unifying the remaining help-text rosters) stays tracked by
/// #1115's follow-on cluster.
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
    (
        "claude",
        "Anthropic Claude API (requires: ANTHROPIC_API_KEY)",
    ),
    (
        "openrouter",
        "OpenRouter unified API, 100+ models (requires: OPENROUTER_API_KEY)",
    ),
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

    /// Whether [`classify_risk`](CommandGenerator::classify_risk) is
    /// implemented. Lets callers (the eval harness, #1465) tell "cannot
    /// judge" from "judged and failed", which both come back as `None`.
    fn supports_risk_judge(&self) -> bool {
        false
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

    /// The backend decided (a `Noul` gate, see `crate::decision`) that the
    /// request needs a clarifying question before a command can be generated.
    /// Not a failure: the CLI renders `question` and exits cleanly.
    #[error("Clarification needed: {}", question.as_deref().unwrap_or("please rephrase the request"))]
    NeedsClarification {
        question: Option<String>,
        /// Probability the model assigned to "needs clarification".
        p: f64,
    },
}

impl GeneratorError {
    /// Build a `NeedsClarification` from a typed decision.
    pub fn from_clarification(c: &crate::decision::Clarification) -> Self {
        Self::NeedsClarification {
            question: c.question.clone(),
            p: c.needed.p_yes,
        }
    }
}

// Types are already public, no re-export needed

// Re-export static matcher
pub use static_matcher::StaticMatcher;

/// Turn per-token log-probabilities into a confidence in `0.0..=1.0`
/// (ADR-017, #1464): the geometric-mean token probability, `exp(mean(logprob))`.
///
/// Non-finite entries are ignored; an empty (or all non-finite) slice yields
/// `None` so the caller reports [`crate::models::ConfidenceSource::Unknown`]
/// rather than a made-up number.
pub fn mean_logprob_confidence(logprobs: &[f64]) -> Option<f64> {
    let (sum, n) = logprobs
        .iter()
        .filter(|lp| lp.is_finite())
        .fold((0.0_f64, 0usize), |(s, n), lp| (s + lp, n + 1));
    (n > 0).then(|| (sum / n as f64).exp().clamp(0.0, 1.0))
}

/// Like [`mean_logprob_confidence`], restricted to the tokens that spell the
/// generated `command` inside the full completion (#1464). The JSON wrapper
/// tokens (`{"cmd": "`) are near-certain and would otherwise inflate the
/// score. Falls back to all tokens when the command cannot be located in the
/// concatenated token text (tokenisers that split mid-escape, for example).
pub fn command_token_confidence(tokens: &[(String, f64)], command: &str) -> Option<f64> {
    let text: String = tokens.iter().map(|(t, _)| t.as_str()).collect();
    let selected: Vec<f64> = match text.find(command).filter(|_| !command.is_empty()) {
        Some(start) => {
            let end = start + command.len();
            let mut offset = 0usize;
            tokens
                .iter()
                .filter_map(|(t, lp)| {
                    let (tok_start, tok_end) = (offset, offset + t.len());
                    offset = tok_end;
                    (tok_start < end && tok_end > start).then_some(*lp)
                })
                .collect()
        }
        None => tokens.iter().map(|(_, lp)| *lp).collect(),
    };
    mean_logprob_confidence(&selected)
}

#[cfg(test)]
mod confidence_tests {
    use super::{command_token_confidence, mean_logprob_confidence};

    #[test]
    fn command_token_confidence_ignores_json_wrapper() {
        // Wrapper tokens are certain (logprob 0); the command tokens are 0.5 each.
        let half = 0.5_f64.ln();
        let tokens = vec![
            ("{\"cmd\": \"".to_string(), 0.0),
            ("ls".to_string(), half),
            (" -la".to_string(), half),
            ("\"}".to_string(), 0.0),
        ];
        let c = command_token_confidence(&tokens, "ls -la").unwrap();
        assert!((c - 0.5).abs() < 1e-9, "{c}");
        // Not found in the token text → fall back to every token.
        let c = command_token_confidence(&tokens, "pwd").unwrap();
        assert!(c > 0.5 && c < 1.0);
        assert_eq!(command_token_confidence(&[], "ls"), None);
    }

    #[test]
    fn mean_logprob_confidence_hand_values() {
        let half = 0.5_f64.ln();
        let c = mean_logprob_confidence(&[half, half]).unwrap();
        assert!((c - 0.5).abs() < 1e-9);
        // 0.9 and 0.1 average to the geometric mean 0.3.
        let c = mean_logprob_confidence(&[0.9_f64.ln(), 0.1_f64.ln()]).unwrap();
        assert!((c - 0.3).abs() < 1e-9);
        assert_eq!(mean_logprob_confidence(&[]), None);
        assert_eq!(
            mean_logprob_confidence(&[f64::NAN, f64::NEG_INFINITY]),
            None
        );
        // Positive log-probs (malformed) still clamp to 1.0 rather than exceed it.
        assert_eq!(mean_logprob_confidence(&[0.5]), Some(1.0));
    }
}
