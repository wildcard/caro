// Shared client for hosted OpenAI-compatible chat-completions APIs.
//
// One implementation, one `Provider` profile per service (base URL, API-key
// env var, default model, extra headers). Adding a hosted provider is a new
// `Provider` arm, not a copy of the request/response structs.
//
// - OpenRouter: unified API for 100+ LLMs. Docs: https://openrouter.ai/docs
// - xAI Grok:   https://api.x.ai/v1 (the default endpoint Grok Build uses)

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

const DEFAULT_MAX_TOKENS: u32 = 512;
const DEFAULT_TEMPERATURE: f32 = 0.1;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

static CMD_EXTRACT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{\s*"cmd"\s*:\s*"(.+)"\s*\}"#).expect("Invalid regex pattern"));

/// A hosted OpenAI-compatible service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// OpenRouter (`OPENROUTER_API_KEY`).
    OpenRouter,
    /// xAI Grok (`XAI_API_KEY`).
    Grok,
}

impl Provider {
    /// Human-readable name used in messages and `backend_used`.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::OpenRouter => "OpenRouter",
            Self::Grok => "Grok",
        }
    }

    /// Base URL, including the `/v1`-style API prefix.
    pub fn default_endpoint(self) -> &'static str {
        match self {
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::Grok => "https://api.x.ai/v1",
        }
    }

    /// Model used when neither `--model-name` nor `CARO_MODEL` is set.
    pub fn default_model(self) -> &'static str {
        match self {
            Self::OpenRouter => "qwen/qwen3-coder",
            // Grok Build's own default model (xai-org/grok-build, 2026-09).
            Self::Grok => "grok-4.5",
        }
    }

    /// Environment variable holding the API key.
    pub fn api_key_env(self) -> &'static str {
        match self {
            Self::OpenRouter => "OPENROUTER_API_KEY",
            Self::Grok => "XAI_API_KEY",
        }
    }

    /// Environment variable that overrides the base URL.
    pub fn endpoint_env(self) -> &'static str {
        match self {
            Self::OpenRouter => "OPENROUTER_BASE_URL",
            Self::Grok => "XAI_API_BASE_URL",
        }
    }

    pub fn backend_type(self) -> BackendType {
        match self {
            Self::OpenRouter => BackendType::OpenRouter,
            Self::Grok => BackendType::Grok,
        }
    }

    /// Whether to request per-token logprobs. Only asked where the service is
    /// known to accept the field on every model; xAI reasoning models are not
    /// guaranteed to, so Grok reports confidence as unknown rather than risk
    /// a 400 on every request.
    fn request_logprobs(self) -> bool {
        matches!(self, Self::OpenRouter)
    }
}

#[derive(Clone)]
pub struct OpenAiCompatConfig {
    pub provider: Provider,
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl std::fmt::Debug for OpenAiCompatConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiCompatConfig")
            .field("provider", &self.provider)
            .field("api_key", &"[REDACTED]")
            .field("model", &self.model)
            .field("endpoint", &self.endpoint)
            .field("max_tokens", &self.max_tokens)
            .field("temperature", &self.temperature)
            .finish()
    }
}

impl OpenAiCompatConfig {
    /// Defaults for `provider` with an empty API key.
    pub fn for_provider(provider: Provider) -> Self {
        Self {
            provider,
            api_key: String::new(),
            model: provider.default_model().to_string(),
            endpoint: provider.default_endpoint().to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
        }
    }

    /// Resolve key and endpoint from the environment, with optional
    /// overrides (config-file endpoint, `--model-name`/`CARO_MODEL` model).
    /// Precedence for the endpoint: `endpoint` argument, then the provider's
    /// env var, then the default.
    pub fn from_env(provider: Provider, endpoint: Option<&str>, model: Option<&str>) -> Self {
        let mut cfg = Self::for_provider(provider);
        cfg.api_key = std::env::var(provider.api_key_env()).unwrap_or_default();
        if let Some(e) = endpoint
            .map(str::to_owned)
            .or_else(|| std::env::var(provider.endpoint_env()).ok())
        {
            cfg.endpoint = e.trim_end_matches('/').to_string();
        }
        if let Some(m) = model {
            cfg.model = m.to_string();
        }
        cfg
    }
}

impl Default for OpenAiCompatConfig {
    fn default() -> Self {
        Self::for_provider(Provider::OpenRouter)
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
    /// Ask for per-token log-probs so confidence is measured, not constant
    /// (#1464). Omitted for providers that may reject it.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    logprobs: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    #[allow(dead_code)]
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
    #[serde(default)]
    logprobs: Option<ChoiceLogprobs>,
}

/// OpenAI-compatible per-choice log-probabilities (`"logprobs": {"content": [...]}`).
#[derive(Debug, Deserialize)]
struct ChoiceLogprobs {
    #[serde(default)]
    content: Option<Vec<TokenLogprob>>,
}

#[derive(Debug, Deserialize)]
struct TokenLogprob {
    /// Token text; absent in incomplete logprob metadata, which then yields
    /// no measured confidence rather than an all-token fallback.
    #[serde(default)]
    token: Option<String>,
    logprob: f64,
}

/// Per-token `(text, logprob)` pairs for one choice, `None` when the server
/// omitted `logprobs` (#1464). Scored against the parsed command with
/// [`crate::backends::command_token_confidence`].
fn token_logprobs(logprobs: Option<&ChoiceLogprobs>) -> Option<Vec<(String, f64)>> {
    let tokens = logprobs?.content.as_ref()?;
    tokens
        .iter()
        .map(|t| t.token.clone().map(|text| (text, t.logprob)))
        .collect()
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
    #[allow(dead_code)]
    role: String,
}

#[derive(Debug, Deserialize)]
struct ChatUsage {
    #[allow(dead_code)]
    prompt_tokens: u32,
    #[allow(dead_code)]
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

pub struct OpenAiCompatBackend {
    config: OpenAiCompatConfig,
    client: Client,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
    forward_context: bool,
}

impl OpenAiCompatBackend {
    pub fn new(config: OpenAiCompatConfig) -> Result<Self, GeneratorError> {
        if config.api_key.is_empty() {
            return Err(GeneratorError::ConfigError {
                message: format!(
                    "{} API key is required (set {})",
                    config.provider.display_name(),
                    config.provider.api_key_env()
                ),
            });
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| GeneratorError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            config,
            client,
            embedded_fallback: None,
            forward_context: false,
        })
    }

    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }

    /// Forward `request.context` to the provider. Only for callers that
    /// sanitize the context first: the `hybrid` remote enhancer.
    pub fn with_context_forwarding(mut self) -> Self {
        self.forward_context = true;
        self
    }

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
7. If the request is unclear, output ONLY: {{"needs_clarification": true, "p": <0.0-1.0>, "question": "<one short question>"}}
"#,
            request.shell
        )
    }

    /// The user turn. By default `request.context` is NOT forwarded:
    /// `AgentLoop` always fills it with the execution context (cwd, user,
    /// directory and knowledge data), and a hosted API must not receive that
    /// raw. Same policy as `claude.rs`. The `hybrid` backend sanitizes the
    /// context first and opts in via [`Self::with_context_forwarding`].
    fn user_input(&self, request: &CommandRequest) -> String {
        match request.context.as_deref() {
            Some(ctx) if self.forward_context && !ctx.trim().is_empty() => format!(
                "<user_request>{}</user_request>\n<context>{}</context>",
                request.input, ctx
            ),
            _ => format!("<user_request>{}</user_request>", request.input),
        }
    }

    fn parse_command_response(&self, response: &str) -> Result<String, GeneratorError> {
        // Typed clarification gate (#1462): never return a runnable command
        // whose only purpose is to ask the user something.
        if let Some(c) = crate::decision::clarification_from_raw(response) {
            if c.should_ask() {
                return Err(GeneratorError::from_clarification(&c));
            }
        }
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                if !cmd.is_empty() {
                    return Ok(cmd.trim().to_string());
                }
            }
        }

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

        if let Some(caps) = CMD_EXTRACT_REGEX.captures(response) {
            if let Some(cmd_match) = caps.get(1) {
                let cmd = cmd_match.as_str().trim();
                if !cmd.is_empty() {
                    return Ok(cmd.to_string());
                }
            }
        }

        let truncated: String = response.chars().take(200).collect();
        Err(GeneratorError::ParseError {
            content: format!("{}...", truncated),
        })
    }

    async fn call_api(
        &self,
        system_prompt: &str,
        user_input: &str,
    ) -> Result<(String, Option<Vec<(String, f64)>>), GeneratorError> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_input.to_string(),
                },
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: false,
            logprobs: self.config.provider.request_logprobs(),
        };

        let name = self.config.provider.display_name();
        let url = format!("{}/chat/completions", self.config.endpoint);

        let mut http = self.client.post(&url).header(
            header::AUTHORIZATION,
            format!("Bearer {}", self.config.api_key),
        );
        if self.config.provider == Provider::OpenRouter {
            // OpenRouter attribution headers (optional, used for rankings).
            http = http
                .header("HTTP-Referer", "https://github.com/wildcard/caro")
                .header("X-Title", "caro");
        }
        let response = http.json(&request).send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                GeneratorError::BackendUnavailable {
                    reason: format!("{} unavailable: {}", name, e),
                }
            } else {
                GeneratorError::GenerationFailed {
                    details: format!("HTTP request failed: {}", e),
                }
            }
        })?;

        let status = response.status();
        let auth_failed = || GeneratorError::BackendUnavailable {
            reason: format!(
                "{} authentication failed - check {}",
                name,
                self.config.provider.api_key_env()
            ),
        };
        if status == 401 || status == 403 {
            return Err(auth_failed());
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            // xAI answers a bad key with 400 "Incorrect API key provided"
            // rather than 401; treat exactly that as auth so it is not
            // silently swallowed by the embedded fallback.
            if self.config.provider == Provider::Grok
                && status == reqwest::StatusCode::BAD_REQUEST
                && body.to_ascii_lowercase().contains("incorrect api key")
            {
                return Err(auth_failed());
            }
            let snippet: String = body.chars().take(200).collect();
            return Err(GeneratorError::GenerationFailed {
                details: format!("{} API error: {} {}", name, status, snippet),
            });
        }

        let chat_response: ChatResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse {} response: {}", name, e),
                })?;

        if let Some(choice) = chat_response.choices.first() {
            let tokens = token_logprobs(choice.logprobs.as_ref());
            Ok((choice.message.content.clone(), tokens))
        } else {
            Err(GeneratorError::ParseError {
                content: format!("{} response contained no choices", name),
            })
        }
    }

    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        match self
            .call_api(
                &self.create_system_prompt(request),
                &self.user_input(request),
            )
            .await
        {
            Ok((response, tokens)) => match self.parse_command_response(&response) {
                Ok(command) => {
                    let measured = tokens
                        .as_deref()
                        .and_then(|t| crate::backends::command_token_confidence(t, &command));
                    let (confidence_score, confidence_source) = match measured {
                        Some(c) => (c, crate::models::ConfidenceSource::Measured),
                        None => (0.0, crate::models::ConfidenceSource::Unknown),
                    };
                    return Ok(GeneratedCommand {
                        command,
                        explanation: format!(
                            "Generated using {}",
                            self.config.provider.display_name()
                        ),
                        safety_level: RiskLevel::Moderate,
                        estimated_impact: "Remote inference operation".to_string(),
                        alternatives: vec![],
                        backend_used: format!(
                            "{} ({})",
                            self.config.provider.display_name(),
                            self.config.model
                        ),
                        generation_time_ms: 0,
                        confidence_score,
                        confidence_source,
                    });
                }
                // A typed clarification decision is not a parse failure:
                // surface the question instead of falling back to a
                // command from another backend (#1462).
                Err(err @ GeneratorError::NeedsClarification { .. }) => return Err(err),
                Err(parse_error) => {
                    tracing::warn!(
                        "Failed to parse {} response: {}",
                        self.config.provider.display_name(),
                        parse_error
                    );
                }
            },
            Err(api_error) => {
                tracing::warn!(
                    "{} backend failed: {}",
                    self.config.provider.display_name(),
                    api_error
                );
                if let GeneratorError::BackendUnavailable { ref reason } = api_error {
                    if reason.to_lowercase().contains("authentication failed") {
                        return Err(api_error);
                    }
                }
            }
        }

        if let Some(fallback) = &self.embedded_fallback {
            tracing::info!("Falling back to embedded backend");
            let mut fallback_result = fallback.generate_command(request).await?;
            fallback_result.backend_used = format!(
                "Embedded ({} fallback from {})",
                self.config.provider.display_name(),
                self.config.model
            );
            return Ok(fallback_result);
        }

        Err(GeneratorError::BackendUnavailable {
            reason: format!(
                "{} unavailable and no fallback configured",
                self.config.provider.display_name()
            ),
        })
    }
}

#[async_trait]
impl CommandGenerator for OpenAiCompatBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = std::time::Instant::now();
        let mut result = self.generate_with_fallback(request).await?;
        result.generation_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Act as a frontier advisor: produce this backend's own best command for
    /// the request. The agent loop re-validates the result before using it.
    async fn advise(
        &self,
        _draft: &GeneratedCommand,
        request: &CommandRequest,
    ) -> Option<GeneratedCommand> {
        self.generate_command(request).await.ok()
    }

    /// A key is configured. Hosted APIs have no `/health` endpoint and a
    /// network probe on every invocation would add a round trip; a bad key
    /// surfaces as `BackendUnavailable` on the first request instead.
    async fn is_available(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: self.config.provider.backend_type(),
            model_name: self.config.model.clone(),
            supports_streaming: false,
            max_tokens: self.config.max_tokens,
            typical_latency_ms: 3000,
            memory_usage_mb: 0,
            version: "1.0".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_config_default() {
        let config = OpenAiCompatConfig::default();
        assert_eq!(config.model, "qwen/qwen3-coder");
        assert_eq!(config.endpoint, "https://openrouter.ai/api/v1");
        assert_eq!(config.max_tokens, 512);
    }

    #[test]
    fn test_openrouter_requires_api_key() {
        let config = OpenAiCompatConfig::default();
        assert!(OpenAiCompatBackend::new(config).is_err());
    }

    #[test]
    fn test_openrouter_creation() {
        let config = OpenAiCompatConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        assert!(OpenAiCompatBackend::new(config).is_ok());
    }

    #[test]
    fn test_parse_valid_json() {
        let config = OpenAiCompatConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let backend = OpenAiCompatBackend::new(config).unwrap();
        let response = r#"{"cmd": "grep -r 'pattern' ."}"#;
        assert_eq!(
            backend.parse_command_response(response).unwrap(),
            "grep -r 'pattern' ."
        );
    }

    #[test]
    fn test_parse_embedded_json() {
        let config = OpenAiCompatConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let backend = OpenAiCompatBackend::new(config).unwrap();
        let response = r#"Here: {"cmd": "sort file.txt"} done"#;
        assert_eq!(
            backend.parse_command_response(response).unwrap(),
            "sort file.txt"
        );
    }

    #[test]
    fn test_parse_invalid_response() {
        let config = OpenAiCompatConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let backend = OpenAiCompatBackend::new(config).unwrap();
        assert!(backend.parse_command_response("no command here").is_err());
    }

    #[test]
    fn test_backend_info() {
        let config = OpenAiCompatConfig {
            api_key: "test-key".to_string(),
            model: "qwen/qwen3-coder".to_string(),
            ..Default::default()
        };
        let backend = OpenAiCompatBackend::new(config).unwrap();
        let info = backend.backend_info();
        assert_eq!(info.backend_type, BackendType::OpenRouter);
        assert_eq!(info.model_name, "qwen/qwen3-coder");
    }
}

#[cfg(test)]
mod confidence_tests {
    use super::*;

    #[test]
    fn logprobs_yield_measured_confidence() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"{\"cmd\":\"ls\"}"},
            "logprobs":{"content":[{"token":"a","logprob":-0.10536051565782628}]}}]}"#;
        let parsed: ChatResponse = serde_json::from_str(body).unwrap();
        let tokens = token_logprobs(parsed.choices[0].logprobs.as_ref()).unwrap();
        let c = crate::backends::mean_logprob_confidence(
            &tokens.iter().map(|(_, lp)| *lp).collect::<Vec<_>>(),
        )
        .unwrap();
        assert!((c - 0.9).abs() < 1e-9);
    }

    #[test]
    fn command_tokens_are_scored_not_the_wrapper() {
        let half = 0.5_f64.ln();
        let body = format!(
            r#"{{"choices":[{{"message":{{"role":"assistant","content":"{{\"cmd\":\"ls\"}}"}},
            "logprobs":{{"content":[{{"token":"{{\"cmd\":\"","logprob":0.0}},{{"token":"ls","logprob":{half}}},{{"token":"\"}}","logprob":0.0}}]}}}}]}}"#
        );
        let parsed: ChatResponse = serde_json::from_str(&body).unwrap();
        let tokens = token_logprobs(parsed.choices[0].logprobs.as_ref()).unwrap();
        let c = crate::backends::command_token_confidence(&tokens, "ls").unwrap();
        assert!((c - 0.5).abs() < 1e-9, "{c}");
    }

    #[test]
    fn missing_logprobs_yield_unknown() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"x"}}]}"#;
        let parsed: ChatResponse = serde_json::from_str(body).unwrap();
        assert_eq!(token_logprobs(parsed.choices[0].logprobs.as_ref()), None);
    }

    #[test]
    fn missing_token_text_yields_unknown() {
        // Incomplete logprob metadata (a token without text) must not become
        // measured confidence through the all-token fallback.
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"{\"cmd\":\"ls\"}"},
            "logprobs":{"content":[{"token":"{\"cmd","logprob":-0.1},{"logprob":-0.2}]}}]}"#;
        let parsed: ChatResponse = serde_json::from_str(body).unwrap();
        assert_eq!(token_logprobs(parsed.choices[0].logprobs.as_ref()), None);
    }
}

#[cfg(test)]
mod grok_wire_tests {
    use super::*;
    use crate::models::{SafetyLevel, ShellType};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn grok(server: &MockServer) -> OpenAiCompatBackend {
        OpenAiCompatBackend::new(OpenAiCompatConfig {
            api_key: "xai-test-key".to_string(),
            // xAI's base already carries the `/v1` prefix.
            endpoint: format!("{}/v1", server.uri()),
            ..OpenAiCompatConfig::for_provider(Provider::Grok)
        })
        .unwrap()
    }

    fn request() -> CommandRequest {
        CommandRequest {
            input: "list files by size".to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: None,
            backend_preference: None,
        }
    }

    #[test]
    fn grok_defaults() {
        let c = OpenAiCompatConfig::for_provider(Provider::Grok);
        assert_eq!(c.endpoint, "https://api.x.ai/v1");
        assert_eq!(c.model, "grok-4.5");
        assert_eq!(Provider::Grok.api_key_env(), "XAI_API_KEY");
        assert!(format!("{c:?}").contains("[REDACTED]"));
    }

    #[test]
    fn grok_requires_xai_key() {
        let err = OpenAiCompatBackend::new(OpenAiCompatConfig::for_provider(Provider::Grok))
            .err()
            .unwrap();
        assert!(err.to_string().contains("XAI_API_KEY"), "{err}");
    }

    #[tokio::test]
    async fn grok_posts_chat_completions_with_bearer_and_model() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("authorization", "Bearer xai-test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{ "message": { "role": "assistant", "content": "{\"cmd\": \"ls -lS\"}" } }]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let backend = grok(&server);
        let generated = backend.generate_command(&request()).await.unwrap();
        assert_eq!(generated.command, "ls -lS");
        assert_eq!(generated.backend_used, "Grok (grok-4.5)");
        assert_eq!(backend.backend_info().backend_type, BackendType::Grok);

        let reqs = server.received_requests().await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).unwrap();
        assert_eq!(body["model"], "grok-4.5");
        // No logprobs field for xAI (not every model accepts it), and no
        // OpenRouter attribution headers.
        assert!(body.get("logprobs").is_none());
        assert!(reqs[0].headers.get("x-title").is_none());
    }

    #[tokio::test]
    async fn hosted_prompt_never_includes_local_context() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{ "message": { "role": "assistant", "content": "{\"cmd\": \"ls\"}" } }]
            })))
            .mount(&server)
            .await;

        let mut req = request();
        req.context = Some("CWD: /home/alice/secret-project user=alice".to_string());
        grok(&server).generate_command(&req).await.unwrap();

        let reqs = server.received_requests().await.unwrap();
        let body = String::from_utf8_lossy(&reqs[0].body);
        assert!(!body.contains("secret-project"), "{body}");
        assert!(!body.contains("alice"), "{body}");
    }

    #[tokio::test]
    async fn hybrid_remote_forwards_sanitized_context() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{ "message": { "role": "assistant", "content": "{\"cmd\": \"ls\"}" } }]
            })))
            .mount(&server)
            .await;

        let mut req = request();
        req.context = Some("CWD: <PATH_1> user=<USER_1>".to_string());
        grok(&server)
            .with_context_forwarding()
            .generate_command(&req)
            .await
            .unwrap();

        let reqs = server.received_requests().await.unwrap();
        let body = String::from_utf8_lossy(&reqs[0].body);
        assert!(body.contains("<PATH_1>"), "{body}");
    }

    #[tokio::test]
    async fn grok_auth_failure_is_backend_unavailable() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let err = grok(&server)
            .generate_command(&request())
            .await
            .unwrap_err();
        match err {
            GeneratorError::BackendUnavailable { reason } => {
                assert!(reason.contains("XAI_API_KEY"), "{reason}")
            }
            other => panic!("expected BackendUnavailable, got {other:?}"),
        }
    }

    /// Observed against api.x.ai (2026-10-03): a bad key is a 400 with
    /// `{"code":"invalid-argument","error":"Incorrect API key provided. ..."}`.
    #[tokio::test]
    async fn grok_400_incorrect_api_key_is_auth_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "code": "invalid-argument",
                "error": "Incorrect API key provided. You can obtain an API key from https://console.x.ai."
            })))
            .mount(&server)
            .await;

        let err = grok(&server)
            .generate_command(&request())
            .await
            .unwrap_err();
        assert!(
            matches!(err, GeneratorError::BackendUnavailable { ref reason } if reason.contains("authentication failed")),
            "{err:?}"
        );
    }

    #[test]
    fn from_env_endpoint_override_trims_trailing_slash() {
        let c = OpenAiCompatConfig::from_env(
            Provider::Grok,
            Some("https://proxy.example/v1/"),
            Some("grok-4.6"),
        );
        assert_eq!(c.endpoint, "https://proxy.example/v1");
        assert_eq!(c.model, "grok-4.6");
    }
}
