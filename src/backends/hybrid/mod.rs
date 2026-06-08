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
        let (mut sanitized_req, session) = self.sanitize_request(request);

        // Prepend a briefing that tells the remote model the redaction was done
        // by Caro's local model, describes each placeholder, and instructs it to
        // reproduce placeholders verbatim. Injected into `input` so it reaches
        // every backend's prompt (which all include the request input).
        if let Some(briefing) = session.redaction_briefing() {
            sanitized_req.input = format!("{}\n\n{}", briefing, sanitized_req.input);
        }

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

        // Make the local model an aware participant in the redaction pipeline by
        // attaching the privacy-layer contract note to its context. Only when
        // sanitizing (the note describes active redaction, which the opt-in
        // public path disables).
        let local_request = if self.sanitizes() {
            let note = ContextSanitizer::local_awareness_note();
            let mut req = request.clone();
            req.context = Some(match req.context.take() {
                Some(c) => format!("{}\n{}", c, note),
                None => note.to_string(),
            });
            req
        } else {
            request.clone()
        };

        let mut result = self.local.generate_command(&local_request).await?;
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
        seen_context: std::sync::Mutex<Option<String>>,
    }

    impl StubBackend {
        fn new(name: &str, reply: &str, fail: bool) -> Self {
            Self {
                name: name.to_string(),
                reply_command: reply.to_string(),
                fail,
                seen: std::sync::Mutex::new(None),
                seen_context: std::sync::Mutex::new(None),
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
            *self.seen_context.lock().unwrap() = request.context.clone();
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
        let remote = Arc::new(StubBackend::new(
            "remote",
            "rm <REDACTED_FILEPATH_1>",
            false,
        ));
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

        // The remote saw a sanitized prompt with NO PII...
        let seen = remote.seen.lock().unwrap().clone().unwrap();
        assert!(!seen.contains("alice"));
        assert!(!seen.contains("secret.txt"));
        assert!(seen.contains("<REDACTED_FILEPATH_1>"));
        // ...plus a briefing attributing the redaction to Caro's local model
        // and describing the placeholder.
        assert!(seen.contains("Caro's local model"));
        assert!(seen.contains("filesystem path"));

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
        let hybrid = HybridBackend::new(local.clone(), remote, ContextSanitizer::new(), false);

        let result = hybrid.generate_command(&req("greet me")).await.unwrap();
        assert_eq!(result.command, "echo hi");
        assert!(result.backend_used.contains("local"));

        // The local model was made aware of the redaction pipeline via context.
        let ctx = local.seen_context.lock().unwrap().clone().unwrap();
        assert!(ctx.contains("caro-privacy-layer"));
    }

    #[tokio::test]
    async fn test_public_optin_local_fallback_has_no_awareness_note() {
        // In allow_public mode there is no active redaction, so the local
        // fallback should NOT be told values are being rewritten.
        let remote = Arc::new(StubBackend::new("remote", "x", true)); // fails
        let local = Arc::new(StubBackend::new("local", "echo hi", false));
        let hybrid = HybridBackend::new(local.clone(), remote, ContextSanitizer::new(), true);

        hybrid.generate_command(&req("greet me")).await.unwrap();
        assert!(local.seen_context.lock().unwrap().is_none());
    }

    /// End-to-end on-the-wire proof: drive a REAL MeshBackend (which performs an
    /// actual reqwest POST) through the hybrid gateway against a mock server,
    /// then inspect the exact bytes that left the machine. The briefing and the
    /// redacted token must be present; the real PII must not.
    #[cfg(feature = "remote-backends")]
    #[tokio::test]
    async fn test_redacted_briefing_appears_on_the_wire_not_pii() {
        use crate::backends::remote::MeshBackend;
        use reqwest::Url;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [ {
                    "message": {
                        "role": "assistant",
                        "content": "{\"cmd\": \"cat <REDACTED_FILEPATH_1>\"}"
                    }
                } ]
            })))
            .mount(&server)
            .await;

        let mesh = Arc::new(
            MeshBackend::new(Url::parse(&server.uri()).unwrap(), "mesh".to_string()).unwrap(),
        );
        let local = Arc::new(StubBackend::new("local", "noop", false));
        let hybrid = HybridBackend::new(local, mesh, ContextSanitizer::new(), false);

        let result = hybrid
            .generate_command(&req("show me /Users/alice/secret.txt"))
            .await
            .unwrap();

        // Inspect the actual request body that crossed the network boundary.
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let wire_body = String::from_utf8(requests[0].body.clone()).unwrap();

        // The redacted token and the briefing attribution are on the wire...
        assert!(wire_body.contains("<REDACTED_FILEPATH_1>"));
        assert!(wire_body.contains("Caro's local model"));
        assert!(wire_body.contains("filesystem path"));
        // ...but the real PII is NOT.
        assert!(
            !wire_body.contains("alice"),
            "PII leaked on the wire: {wire_body}"
        );
        assert!(!wire_body.contains("secret.txt"), "PII leaked on the wire");

        // And the command handed back to the user has the real path restored.
        assert_eq!(result.command, "cat /Users/alice/secret.txt");
    }
}
