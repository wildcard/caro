// Hybrid privacy gateway backend.
//
// The hybrid backend composes a *local* model (the embedded backend, which is
// also the fallback) with a *remote* enhancer (Mesh-LLM or AI-Horde). Its
// purpose is to let users benefit from a more capable distributed model WITHOUT
// leaking personally identifying or environment-revealing information to that
// network.
//
// Pipeline (default, `allow_public = false`):
//   1. A `ContextSanitizer` session redacts PII from the request `input` and
//      `context` into reversible placeholders (`<PATH_1>`, `<USER_1>`, ...).
//   2. The sanitized request is sent to the remote enhancer.
//   3. Placeholders in the returned command are restored to real values
//      locally — the network never saw them.
//   4. If the remote fails, we fall back to the local model on the ORIGINAL
//      (un-sanitized) request, which is fine because it stays on-device.
//
// When the user explicitly opts into public inference (`allow_public = true`),
// sanitization is skipped and the request is sent verbatim — the "standard
// remote warning" path that Claude/OpenRouter already use. This is the user's
// deliberate choice for, e.g., a trusted private mesh where redaction is
// unnecessary.
//
// Note: deterministic common-command handling (the static matcher) already runs
// upstream in `AgentLoop` before any backend is consulted, so the hybrid
// backend does not duplicate it here.

pub mod sanitizer;

pub use sanitizer::{ContextSanitizer, SanitizeSession};

use async_trait::async_trait;
use std::sync::Arc;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand};

/// Privacy-preserving backend pairing a local model with a remote enhancer.
pub struct HybridBackend {
    /// On-device model used as the fallback and for un-sanitized work.
    local: Arc<dyn CommandGenerator>,
    /// Remote enhancer (Mesh-LLM / AI-Horde) that only ever sees sanitized input.
    remote: Arc<dyn CommandGenerator>,
    sanitizer: ContextSanitizer,
    /// When true, skip sanitization and send prompts verbatim (opt-in).
    allow_public: bool,
}

impl HybridBackend {
    pub fn new(
        local: Arc<dyn CommandGenerator>,
        remote: Arc<dyn CommandGenerator>,
        sanitizer: ContextSanitizer,
        allow_public: bool,
    ) -> Self {
        Self {
            local,
            remote,
            sanitizer,
            allow_public,
        }
    }

    /// Whether this gateway sanitizes before sending (true) or sends verbatim.
    pub fn sanitizes(&self) -> bool {
        !self.allow_public
    }

    /// Build a sanitized clone of `request`, returning the request plus the
    /// session needed to restore the result.
    fn sanitize_request<'a>(
        &'a self,
        request: &CommandRequest,
    ) -> (CommandRequest, SanitizeSession<'a>) {
        let mut session = self.sanitizer.session();
        let input = session.sanitize(&request.input);
        let context = request.context.as_ref().map(|c| session.sanitize(c));

        let sanitized = CommandRequest {
            input,
            shell: request.shell,
            safety_level: request.safety_level,
            context,
            backend_preference: request.backend_preference.clone(),
        };
        (sanitized, session)
    }

    /// Run the remote enhancer with sanitization, restoring PII in the result.
    async fn generate_sanitized(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let (sanitized_req, session) = self.sanitize_request(request);
        let mut result = self.remote.generate_command(&sanitized_req).await?;

        // Restore real values that the remote echoed back as placeholders.
        result.command = session.restore(&result.command);
        result.alternatives = result
            .alternatives
            .into_iter()
            .map(|a| session.restore(&a))
            .collect();

        let redactions = session.redaction_count();
        result.backend_used = format!(
            "Hybrid[{} via local sanitizer, {} redaction(s)]",
            result.backend_used, redactions
        );
        Ok(result)
    }

    async fn fallback_local(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        tracing::info!("Hybrid: remote enhancer failed, using local model");
        let mut result = self.local.generate_command(request).await?;
        result.backend_used = format!("Hybrid[local: {}]", result.backend_used);
        Ok(result)
    }
}

#[async_trait]
impl CommandGenerator for HybridBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let attempt = if self.allow_public {
            // Opt-in: send verbatim, like any other remote backend.
            self.remote.generate_command(request).await
        } else {
            self.generate_sanitized(request).await
        };

        match attempt {
            Ok(result) => Ok(result),
            Err(err) => {
                tracing::warn!("Hybrid remote enhancer error: {}", err);
                self.fallback_local(request).await
            }
        }
    }

    async fn is_available(&self) -> bool {
        // The local model is always usable, so the gateway is always available;
        // remote availability only affects which path is taken.
        self.local.is_available().await || self.remote.is_available().await
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Hybrid,
            model_name: format!(
                "{} + {}",
                self.local.backend_info().model_name,
                self.remote.backend_info().model_name
            ),
            supports_streaming: false,
            max_tokens: 100,
            typical_latency_ms: self.remote.backend_info().typical_latency_ms,
            memory_usage_mb: self.local.backend_info().memory_usage_mb,
            version: "1.0".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        let _ = self.remote.shutdown().await;
        self.local.shutdown().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RiskLevel, SafetyLevel, ShellType};

    /// A stub backend that echoes a fixed command and records what it received.
    struct StubBackend {
        name: String,
        reply_command: String,
        fail: bool,
        seen: std::sync::Mutex<Option<String>>,
    }

    impl StubBackend {
        fn new(name: &str, reply: &str, fail: bool) -> Self {
            Self {
                name: name.to_string(),
                reply_command: reply.to_string(),
                fail,
                seen: std::sync::Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl CommandGenerator for StubBackend {
        async fn generate_command(
            &self,
            request: &CommandRequest,
        ) -> Result<GeneratedCommand, GeneratorError> {
            *self.seen.lock().unwrap() = Some(request.input.clone());
            if self.fail {
                return Err(GeneratorError::BackendUnavailable {
                    reason: "stub down".to_string(),
                });
            }
            Ok(GeneratedCommand {
                command: self.reply_command.clone(),
                explanation: "stub".to_string(),
                safety_level: RiskLevel::Safe,
                estimated_impact: "none".to_string(),
                alternatives: vec![],
                backend_used: self.name.clone(),
                generation_time_ms: 0,
                confidence_score: 0.9,
            })
        }
        async fn is_available(&self) -> bool {
            !self.fail
        }
        fn backend_info(&self) -> BackendInfo {
            BackendInfo {
                backend_type: BackendType::Mock,
                model_name: self.name.clone(),
                supports_streaming: false,
                max_tokens: 100,
                typical_latency_ms: 10,
                memory_usage_mb: 0,
                version: "0".to_string(),
            }
        }
        async fn shutdown(&self) -> Result<(), GeneratorError> {
            Ok(())
        }
    }

    fn req(input: &str) -> CommandRequest {
        CommandRequest {
            input: input.to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: None,
            backend_preference: None,
        }
    }

    #[tokio::test]
    async fn test_remote_never_sees_pii_but_result_is_restored() {
        // Remote echoes back a command referencing the placeholder it received.
        let remote = Arc::new(StubBackend::new("remote", "rm <PATH_1>", false));
        let local = Arc::new(StubBackend::new("local", "noop", false));
        let hybrid = HybridBackend::new(
            local,
            remote.clone(),
            ContextSanitizer::new(),
            false, // sanitize
        );

        let result = hybrid
            .generate_command(&req("delete /Users/alice/secret.txt"))
            .await
            .unwrap();

        // The remote saw a sanitized prompt with NO PII.
        let seen = remote.seen.lock().unwrap().clone().unwrap();
        assert!(!seen.contains("alice"));
        assert!(!seen.contains("secret.txt"));
        assert!(seen.contains("<PATH_1>"));

        // The final command has the real path restored locally.
        assert_eq!(result.command, "rm /Users/alice/secret.txt");
    }

    #[tokio::test]
    async fn test_allow_public_sends_verbatim() {
        let remote = Arc::new(StubBackend::new("remote", "ok", false));
        let local = Arc::new(StubBackend::new("local", "noop", false));
        let hybrid = HybridBackend::new(local, remote.clone(), ContextSanitizer::new(), true);

        hybrid
            .generate_command(&req("delete /Users/alice/secret.txt"))
            .await
            .unwrap();

        // Opt-in path: the remote saw the raw prompt, PII included.
        let seen = remote.seen.lock().unwrap().clone().unwrap();
        assert!(seen.contains("alice"));
        assert!(seen.contains("secret.txt"));
    }

    #[tokio::test]
    async fn test_falls_back_to_local_when_remote_fails() {
        let remote = Arc::new(StubBackend::new("remote", "x", true)); // fails
        let local = Arc::new(StubBackend::new("local", "echo hi", false));
        let hybrid = HybridBackend::new(local, remote, ContextSanitizer::new(), false);

        let result = hybrid.generate_command(&req("greet me")).await.unwrap();
        assert_eq!(result.command, "echo hi");
        assert!(result.backend_used.contains("local"));
    }
}
