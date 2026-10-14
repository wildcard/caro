// vLLM server backend implementation

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;

use crate::backends::remote::context_length_warning;
use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::decision::decide_with_retry;
use crate::models::{RiskJudgeContext, RiskJudgment};
use crate::prompts::{build_risk_judge_prompt, parse_risk_judgment, risk_judge_schema};

/// Output-token cap used unless configured otherwise.
const DEFAULT_MAX_TOKENS: u32 = 100;

/// Model used by the explicit `--backend vllm` route when neither
/// `--model-name` nor the config names one.
pub const DEFAULT_MODEL: &str = "codellama/CodeLlama-7b-hf";

/// Endpoint used when `[backends].vllm_url` is unset.
pub const DEFAULT_URL: &str = "http://localhost:8000";

/// Build the fail-closed backend for an explicit `--backend vllm`. The CLI
/// generation path and `caro test --backend vllm` both call this, so the two
/// routes honor the same endpoint, model and limits. Keyless: no credential is
/// read or sent. Nothing here constructs or attaches an embedded backend.
pub fn explicit_from_config(
    cfg: &crate::models::BackendsConfig,
    model_name: Option<&str>,
) -> Result<VllmBackend, GeneratorError> {
    let url_str = cfg.vllm_url.as_deref().unwrap_or(DEFAULT_URL);
    let url = Url::parse(url_str).map_err(|e| GeneratorError::ConfigError {
        message: format!("backends.vllm_url `{url_str}` is not a valid URL: {e}"),
    })?;
    let model = model_name.unwrap_or(DEFAULT_MODEL).to_string();
    let mut backend = VllmBackend::new(url, model)?
        .with_max_tokens(cfg.vllm_max_tokens.unwrap_or(DEFAULT_MAX_TOKENS))
        .with_logprobs(cfg.vllm_logprobs.unwrap_or(true))
        .fail_closed();
    if let Some(secs) = cfg.vllm_timeout_secs {
        backend = backend.with_timeout(Duration::from_secs(secs))?;
    }
    Ok(backend)
}

/// Regex pattern to extract command from malformed JSON with unescaped quotes
/// Handles cases like: {"cmd": "find . -type f -name "*.txt""}
static CMD_EXTRACT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{\s*"cmd"\s*:\s*"(.+)"\s*\}"#).expect("Invalid regex pattern"));
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

/// vLLM API request format (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct VllmRequest {
    model: String,
    messages: Vec<VllmMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
    /// Ask for per-token log-probs so confidence is measured, not constant (#1464).
    /// Omitted when false: some servers reject the field, and the explicit
    /// local route asks for no log-probs (`vllm_logprobs = false`).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    logprobs: bool,
    /// JSON Schema for guided decoding; set on decision requests only (#1465).
    #[serde(skip_serializing_if = "Option::is_none")]
    guided_json: Option<serde_json::Value>,
}

/// vLLM message format
#[derive(Debug, Serialize)]
struct VllmMessage {
    role: String,
    content: String,
}

/// vLLM API response format
#[derive(Debug, Deserialize)]
struct VllmResponse {
    choices: Vec<VllmChoice>,
    /// The model the server says it used. Absent on some servers.
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<VllmUsage>,
}

/// Exact shape of a command reply on the fail-closed route.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictCmdReply {
    cmd: String,
}

/// Exact shape of a clarification reply on the fail-closed route.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictAskReply {
    needs_clarification: bool,
    #[serde(default)]
    p: Option<f64>,
    question: String,
}

/// The JSON text of a reply that is a bare object, or exactly one fenced block
/// and nothing else. `None` for any other prose or fragment.
fn strip_whole_message_fence(response: &str) -> Option<&str> {
    let t = response.trim();
    let body = if let Some(rest) = t.strip_prefix("```") {
        let rest = rest
            .strip_prefix("json\n")
            .or_else(|| rest.strip_prefix('\n'))?;
        let body = rest.strip_suffix("```")?;
        if !body.ends_with('\n') || body.contains("```") {
            return None;
        }
        body.trim()
    } else {
        t
    };
    body.starts_with('{').then_some(body)
}

/// vLLM choice structure
#[derive(Debug, Deserialize)]
struct VllmChoice {
    message: VllmResponseMessage,
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

/// vLLM response message
#[derive(Debug, Deserialize)]
struct VllmResponseMessage {
    content: String,
    #[allow(dead_code)]
    role: String,
}

/// vLLM usage statistics
#[derive(Debug, Deserialize)]
struct VllmUsage {
    #[serde(default)]
    prompt_tokens: Option<u32>,
    #[serde(default)]
    completion_tokens: Option<u32>,
    #[serde(default)]
    total_tokens: Option<u32>,
}

/// What one request told us about the model that answered. A field the server
/// did not send is `None`: unverified, never assumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseMeta {
    /// The model name Caro asked for.
    pub requested_model: String,
    /// The model the server named in its reply, if it named one.
    pub returned_model: Option<String>,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

impl ResponseMeta {
    /// One honest line for `GeneratedCommand::explanation`. It reports what the
    /// server said; it is not proof that inference ran on that model.
    pub fn summary(&self) -> String {
        let model = match &self.returned_model {
            Some(m) => format!("returned model `{m}`"),
            None => "returned model unverified (not in response)".to_string(),
        };
        let usage = match (self.prompt_tokens, self.completion_tokens) {
            (Some(p), Some(c)) => format!("usage prompt={p} completion={c}"),
            _ => "usage unverified (not in response)".to_string(),
        };
        format!(
            "requested model `{}`, {model}, {usage}",
            self.requested_model
        )
    }
}

/// Response shape for `GET /v1/models`, used only to check the context
/// length actually in effect for the configured model.
#[derive(Debug, Deserialize)]
struct VllmModelsResponse {
    data: Vec<VllmModelEntry>,
}

/// One entry in vLLM's OpenAI-compatible model list. `max_model_len` is a
/// vLLM extension beyond the plain OpenAI schema — absent means the server
/// didn't report it.
#[derive(Debug, Deserialize)]
struct VllmModelEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    max_model_len: Option<u32>,
}

/// vLLM backend for remote vLLM server
pub struct VllmBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    api_key: Option<String>,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
    /// Output-token cap per request (default 100).
    max_tokens: u32,
    /// Whether to ask for per-token log-probs (default true).
    logprobs: bool,
    /// Fail-closed mode (explicit local route): no fallback, readiness and
    /// served-model checks first, a named model mismatch is an error.
    fail_closed: bool,
    /// The most recent reply's model/usage record.
    last_meta: std::sync::Mutex<Option<ResponseMeta>>,
    /// Ensures the served-context-length probe (`/v1/models`) and its log
    /// line fire at most once per backend instance, not on every request.
    checked_context: OnceCell<()>,
}

impl VllmBackend {
    /// Create a new vLLM backend
    pub fn new(base_url: Url, model_name: String) -> Result<Self, GeneratorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| GeneratorError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            base_url,
            model_name,
            client,
            api_key: None,
            embedded_fallback: None,
            max_tokens: DEFAULT_MAX_TOKENS,
            logprobs: true,
            fail_closed: false,
            last_meta: std::sync::Mutex::new(None),
            checked_context: OnceCell::new(),
        })
    }

    /// Set the per-request output-token cap.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the per-request timeout. Rebuilds the HTTP client.
    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, GeneratorError> {
        self.client = Client::builder().timeout(timeout).build().map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            }
        })?;
        Ok(self)
    }

    /// Ask (or not) for per-token log-probs. Off omits the field entirely.
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = logprobs;
        self
    }

    /// Fail-closed mode for an explicitly chosen server. Never falls back to
    /// another backend, checks readiness and that the selected model is served,
    /// and fails if the reply names a different model. Existing callers that do
    /// not call this keep the old behavior.
    pub fn fail_closed(mut self) -> Self {
        self.fail_closed = true;
        self.embedded_fallback = None;
        self
    }

    /// Model/usage record of the most recent reply, if any.
    pub fn last_response_meta(&self) -> Option<ResponseMeta> {
        self.last_meta.lock().ok().and_then(|g| g.clone())
    }

    /// Set API key for authentication
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Add embedded fallback backend
    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        if !self.fail_closed {
            self.embedded_fallback = Some(fallback);
        }
        self
    }

    /// Create system prompt for vLLM
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

Request: {}
"#,
            request.shell, request.input
        )
    }

    /// Parse JSON response from vLLM
    fn parse_command_response(&self, response: &str) -> Result<String, GeneratorError> {
        // Typed clarification gate (#1462): never return a runnable command
        // whose only purpose is to ask the user something.
        if let Some(c) = crate::decision::clarification_from_raw(response) {
            if c.should_ask() {
                return Err(GeneratorError::from_clarification(&c));
            }
        }
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

        // Regex fallback: Handle malformed JSON with unescaped quotes
        // e.g., {"cmd": "find . -type f -name "*.txt""}
        if let Some(caps) = CMD_EXTRACT_REGEX.captures(response) {
            if let Some(cmd_match) = caps.get(1) {
                let cmd = cmd_match.as_str().trim();
                if !cmd.is_empty() {
                    return Ok(cmd.to_string());
                }
            }
        }

        Err(GeneratorError::ParseError {
            content: response.to_string(),
        })
    }

    /// Strict parse for the explicit fail-closed route.
    ///
    /// Accepts exactly one of two JSON objects, optionally wrapped in one
    /// whole-message Markdown fence (a line "```json" or "```", the object, a
    /// line "```", and only whitespace outside):
    ///
    /// - `{"cmd": "<non-empty string>"}`
    /// - `{"needs_clarification": true, "p": <0..=1>, "question": "<non-empty>"}`
    ///   (`p` optional; at or above the gate floor)
    ///
    /// Anything else is an error: malformed JSON, trailing data, unknown or
    /// duplicate fields, a non-string or empty `cmd`, a `cmd` mixed with
    /// clarification fields, `needs_clarification: false`. Nothing is
    /// salvaged, repaired or regex-extracted into command text.
    fn parse_strict_reply(&self, response: &str) -> Result<String, GeneratorError> {
        let bad = |why: &str| GeneratorError::ParseError {
            content: format!("{why}: {response}"),
        };
        let inner = strip_whole_message_fence(response).ok_or_else(|| {
            bad("reply is not a bare JSON object or one whole-message fenced JSON object")
        })?;

        if let Ok(reply) = serde_json::from_str::<StrictCmdReply>(inner) {
            let cmd = reply.cmd.trim();
            if cmd.is_empty() {
                return Err(bad("`cmd` is empty"));
            }
            // The legacy "echo a question" command is a clarification, not a command.
            if let Some(c) = crate::decision::clarification_from_raw(inner) {
                if c.should_ask() {
                    return Err(GeneratorError::from_clarification(&c));
                }
            }
            return Ok(cmd.to_string());
        }
        if let Ok(ask) = serde_json::from_str::<StrictAskReply>(inner) {
            if !ask.needs_clarification {
                return Err(bad("`needs_clarification` is false and there is no `cmd`"));
            }
            if ask.question.trim().is_empty() {
                return Err(bad("clarification has an empty `question`"));
            }
            let p = ask.p.unwrap_or(0.9);
            if !(0.0..=1.0).contains(&p) {
                return Err(bad("clarification `p` is outside 0..=1"));
            }
            let c = crate::decision::Clarification {
                needed: crate::decision::Noul::new(p),
                question: Some(ask.question.trim().to_string()),
            };
            if c.should_ask() {
                return Err(GeneratorError::from_clarification(&c));
            }
            return Err(bad("clarification below the confidence floor and no `cmd`"));
        }
        Err(bad(
            "reply is not exactly {\"cmd\": string} or a typed clarification object",
        ))
    }

    /// `endpoint ..., model ...` text for error messages.
    fn ctx(&self) -> String {
        format!(
            "vLLM endpoint {}, model `{}`",
            self.base_url, self.model_name
        )
    }

    /// Fail-closed readiness: the endpoint answers `/health` and serves the
    /// selected model. The selected model is matched by exact id; a different
    /// first entry is never substituted.
    async fn preflight(&self) -> Result<(), GeneratorError> {
        let ctx = self.ctx();
        let health = self
            .base_url
            .join("/health")
            .map_err(|e| GeneratorError::ConfigError {
                message: format!("{ctx}: invalid base URL: {e}"),
            })?;
        match self.client.get(health).send().await {
            Err(e) => {
                return Err(GeneratorError::BackendUnavailable {
                    reason: format!("{ctx}: unreachable: {e}"),
                })
            }
            Ok(r) if !r.status().is_success() => {
                return Err(GeneratorError::BackendUnavailable {
                    reason: format!("{ctx}: not ready (GET /health returned {})", r.status()),
                })
            }
            Ok(_) => {}
        }

        let models_url =
            self.base_url
                .join("/v1/models")
                .map_err(|e| GeneratorError::ConfigError {
                    message: format!("{ctx}: invalid base URL: {e}"),
                })?;
        let mut req = self.client.get(models_url);
        if let Some(api_key) = &self.api_key {
            req = req.header(header::AUTHORIZATION, format!("Bearer {}", api_key));
        }
        let resp = req
            .send()
            .await
            .map_err(|e| GeneratorError::BackendUnavailable {
                reason: format!("{ctx}: cannot list models: {e}"),
            })?;
        if !resp.status().is_success() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("{ctx}: GET /v1/models returned {}", resp.status()),
            });
        }
        let models: VllmModelsResponse =
            resp.json().await.map_err(|e| GeneratorError::ParseError {
                content: format!("{ctx}: /v1/models body is not a model list: {e}"),
            })?;
        if !models.data.iter().any(|m| m.id == self.model_name) {
            let served: Vec<&str> = models.data.iter().map(|m| m.id.as_str()).collect();
            return Err(GeneratorError::BackendUnavailable {
                reason: format!(
                    "{ctx}: model `{}` is not served (served: [{}])",
                    self.model_name,
                    served.join(", ")
                ),
            });
        }
        Ok(())
    }

    /// Fail-closed generation: preflight, one request, no fallback. Transport,
    /// status, parse and model-mismatch errors reach the caller unchanged.
    async fn generate_fail_closed(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        self.preflight().await?;
        let (response, tokens) = self
            .call_vllm_api(&self.create_system_prompt(request), None)
            .await?;
        // A reply that is not a command keeps its error type but gains the
        // endpoint and model. A clarification question passes through unchanged.
        let command = self.parse_strict_reply(&response).map_err(|e| match e {
            GeneratorError::ParseError { content } => GeneratorError::ParseError {
                content: format!("{}: reply is not a command: {content}", self.ctx()),
            },
            other => other,
        })?;
        let measured = tokens
            .as_deref()
            .and_then(|t| crate::backends::command_token_confidence(t, &command));
        let (confidence_score, confidence_source) = match measured {
            Some(c) => (c, crate::models::ConfidenceSource::Measured),
            None => (0.0, crate::models::ConfidenceSource::Unknown),
        };
        let summary = self
            .last_response_meta()
            .map(|m| m.summary())
            .unwrap_or_else(|| "response metadata unverified".to_string());
        Ok(GeneratedCommand {
            command,
            explanation: format!("Generated by {} ({summary})", self.ctx()),
            safety_level: RiskLevel::Safe, // validated by the caller (CLI SafetyValidator)
            estimated_impact: "Remote inference operation".to_string(),
            alternatives: vec![],
            backend_used: format!("vLLM ({})", self.model_name),
            generation_time_ms: 0,
            confidence_score,
            confidence_source,
        })
    }

    /// Call vLLM API for inference
    /// Returns the reply text and, when the server returned `logprobs`, the
    /// measured confidence of that reply.
    async fn call_vllm_api(
        &self,
        prompt: &str,
        guided_json: Option<serde_json::Value>,
    ) -> Result<(String, Option<Vec<(String, f64)>>), GeneratorError> {
        let request = VllmRequest {
            model: self.model_name.clone(),
            messages: vec![VllmMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.1,
            max_tokens: self.max_tokens,
            stream: false,
            logprobs: self.logprobs,
            guided_json,
        };

        let url = self.base_url.join("/v1/chat/completions").map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Invalid base URL: {}", e),
            }
        })?;

        let mut req_builder = self.client.post(url).json(&request);

        // Add authentication if available
        if let Some(api_key) = &self.api_key {
            req_builder = req_builder.header(header::AUTHORIZATION, format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                GeneratorError::BackendUnavailable {
                    reason: if self.fail_closed {
                        format!("{}: unreachable or timed out: {}", self.ctx(), e)
                    } else {
                        format!("vLLM server unavailable: {}", e)
                    },
                }
            } else {
                GeneratorError::GenerationFailed {
                    details: if self.fail_closed {
                        format!("{}: HTTP request failed: {}", self.ctx(), e)
                    } else {
                        format!("HTTP request failed: {}", e)
                    },
                }
            }
        })?;

        // Check for authentication errors
        if response.status() == 401 || response.status() == 403 {
            return Err(GeneratorError::BackendUnavailable {
                reason: if self.fail_closed {
                    format!(
                        "{}: Authentication failed (HTTP {}); this keyless route sends no API key",
                        self.ctx(),
                        response.status()
                    )
                } else {
                    "Authentication failed - invalid API key".to_string()
                },
            });
        }

        if !response.status().is_success() {
            return Err(GeneratorError::GenerationFailed {
                details: if self.fail_closed {
                    format!("{}: vLLM API error: {}", self.ctx(), response.status())
                } else {
                    format!("vLLM API error: {}", response.status())
                },
            });
        }

        let vllm_response: VllmResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: if self.fail_closed {
                        format!("{}: failed to parse response: {}", self.ctx(), e)
                    } else {
                        format!("Failed to parse vLLM response: {}", e)
                    },
                })?;

        let meta = ResponseMeta {
            requested_model: self.model_name.clone(),
            returned_model: vllm_response.model.clone(),
            prompt_tokens: vllm_response.usage.as_ref().and_then(|u| u.prompt_tokens),
            completion_tokens: vllm_response
                .usage
                .as_ref()
                .and_then(|u| u.completion_tokens),
            total_tokens: vllm_response.usage.as_ref().and_then(|u| u.total_tokens),
        };
        if let Ok(mut slot) = self.last_meta.lock() {
            *slot = Some(meta.clone());
        }
        if self.fail_closed {
            // Identity is required, not optional: a reply that does not name
            // the model that answered cannot show it matches the selected one.
            // Missing usage stays "unverified"; missing identity is an error.
            match &meta.returned_model {
                Some(returned) if returned == &self.model_name => {}
                Some(returned) => {
                    return Err(GeneratorError::GenerationFailed {
                        details: format!(
                            "{}: response names model `{}`, which does not match the selected model",
                            self.ctx(),
                            returned
                        ),
                    });
                }
                None => {
                    return Err(GeneratorError::GenerationFailed {
                        details: format!(
                            "{}: response does not name the model that answered, so it cannot be checked against the selected model",
                            self.ctx()
                        ),
                    });
                }
            }
        }

        if let Some(choice) = vllm_response.choices.first() {
            let tokens = token_logprobs(choice.logprobs.as_ref());
            Ok((choice.message.content.clone(), tokens))
        } else {
            Err(GeneratorError::ParseError {
                content: if self.fail_closed {
                    format!("{}: response contained no choices", self.ctx())
                } else {
                    "vLLM response contained no choices".to_string()
                },
            })
        }
    }

    /// Fetch the context length vLLM is actually serving for
    /// `self.model_name`, via its OpenAI-compatible `/v1/models` endpoint.
    ///
    /// Best-effort only: any transport error, non-2xx status, or unexpected
    /// body shape yields `None` rather than an error — this is a diagnostic
    /// nicety, not something that should ever block generation.
    async fn fetch_vllm_context_length(&self) -> Option<u32> {
        let url = self.base_url.join("/v1/models").ok()?;
        let mut req_builder = self.client.get(url);
        if let Some(api_key) = &self.api_key {
            req_builder = req_builder.header(header::AUTHORIZATION, format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await.ok()?;
        if !response.status().is_success() {
            return None;
        }

        let models: VllmModelsResponse = response.json().await.ok()?;
        models
            .data
            .iter()
            .find(|entry| entry.id == self.model_name)
            .or_else(|| models.data.first())
            .and_then(|entry| entry.max_model_len)
    }

    /// Probe the served context length once and log a warning if it looks
    /// undersized. Never surfaces as a `GeneratorError` — see
    /// `fetch_vllm_context_length`.
    async fn warn_on_context_length(&self) {
        match self.fetch_vllm_context_length().await {
            Some(served) => {
                if let Some(warning) = context_length_warning(served, None) {
                    tracing::warn!("vLLM ({}): {}", self.model_name, warning);
                }
            }
            None => {
                tracing::debug!(
                    "vLLM ({}): could not determine served context length via /v1/models",
                    self.model_name
                );
            }
        }
    }

    /// Attempt inference with fallback to embedded backend
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        if self.fail_closed {
            return self.generate_fail_closed(request).await;
        }
        self.checked_context
            .get_or_init(|| self.warn_on_context_length())
            .await;

        // Try vLLM first
        match self
            .call_vllm_api(&self.create_system_prompt(request), None)
            .await
        {
            Ok((response, tokens)) => {
                match self.parse_command_response(&response) {
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
                            explanation: "Generated using vLLM server".to_string(),
                            safety_level: RiskLevel::Safe, // TODO: Implement safety validation
                            estimated_impact: "Remote inference operation".to_string(),
                            alternatives: vec![],
                            backend_used: format!("vLLM ({})", self.model_name),
                            generation_time_ms: 0, // Will be set by caller
                            confidence_score,
                            confidence_source,
                        });
                    }
                    // A typed clarification decision is not a parse failure:
                    // surface the question instead of falling back to a
                    // command from another backend (#1462).
                    Err(err @ GeneratorError::NeedsClarification { .. }) => return Err(err),
                    Err(parse_error) => {
                        tracing::warn!("Failed to parse vLLM response: {}", parse_error);
                        // Continue to fallback
                    }
                }
            }
            Err(vllm_error) => {
                tracing::warn!("vLLM backend failed: {}", vllm_error);

                // For authentication errors, don't retry or fallback immediately
                if let GeneratorError::BackendUnavailable { ref reason } = vllm_error {
                    if reason.contains("Authentication failed") {
                        return Err(vllm_error);
                    }
                }
                // Continue to fallback for other errors
            }
        }

        // Fallback to embedded backend if available
        if let Some(fallback) = &self.embedded_fallback {
            tracing::info!("Falling back to embedded backend");
            let mut fallback_result = fallback.generate_command(request).await?;
            fallback_result.backend_used =
                format!("Embedded (vLLM fallback from {})", self.model_name);
            return Ok(fallback_result);
        }

        // No fallback available
        Err(GeneratorError::BackendUnavailable {
            reason: "vLLM server unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for VllmBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = std::time::Instant::now();

        let mut result = self.generate_with_fallback(request).await?;
        result.generation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(result)
    }

    fn supports_risk_judge(&self) -> bool {
        true
    }

    async fn classify_risk(&self, command: &str, ctx: &RiskJudgeContext) -> Option<RiskJudgment> {
        // Guided decoding (`guided_json` = verdict schema) plus one corrective
        // retry (#1465); fail safe to `None` so the caller falls back to the
        // static decision.
        let prompt = build_risk_judge_prompt(command, ctx);
        let schema = risk_judge_schema();
        let guided = schema.to_json();
        decide_with_retry(
            &prompt,
            &schema,
            |p| {
                let guided = guided.clone();
                async move {
                    self.call_vllm_api(&p, Some(guided))
                        .await
                        .map(|(raw, _)| raw)
                }
            },
            parse_risk_judgment,
        )
        .await
        .value
    }

    async fn is_available(&self) -> bool {
        // Check if vLLM server is responding
        let health_url = match self.base_url.join("/health") {
            Ok(url) => url,
            Err(_) => return false,
        };

        match self.client.get(health_url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::VLlm,
            model_name: self.model_name.clone(),
            supports_streaming: false,
            max_tokens: self.max_tokens,
            typical_latency_ms: 3000,
            memory_usage_mb: 0, // External server
            version: "1.0".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        // Nothing to clean up for HTTP client
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_vllm_backend_creation() {
        let url = Url::parse("https://api.example.com").unwrap();
        let backend = VllmBackend::new(url, "codellama/CodeLlama-7b-hf".to_string());
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_vllm_context_length_from_models_endpoint() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [ { "id": "test-model", "max_model_len": 2048 } ]
            })))
            .mount(&server)
            .await;

        let backend =
            VllmBackend::new(Url::parse(&server.uri()).unwrap(), "test-model".to_string()).unwrap();

        assert_eq!(backend.fetch_vllm_context_length().await, Some(2048));
    }

    #[tokio::test]
    async fn test_fetch_vllm_context_length_falls_back_to_first_entry() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [ { "id": "some-other-model", "max_model_len": 4096 } ]
            })))
            .mount(&server)
            .await;

        let backend = VllmBackend::new(
            Url::parse(&server.uri()).unwrap(),
            "requested-model".to_string(),
        )
        .unwrap();

        assert_eq!(backend.fetch_vllm_context_length().await, Some(4096));
    }

    #[tokio::test]
    async fn test_fetch_vllm_context_length_none_on_server_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let backend =
            VllmBackend::new(Url::parse(&server.uri()).unwrap(), "test-model".to_string()).unwrap();

        assert_eq!(backend.fetch_vllm_context_length().await, None);
    }

    #[tokio::test]
    async fn test_context_length_probe_failure_does_not_block_generation() {
        // /v1/models is broken, but /v1/chat/completions works fine —
        // generation must still succeed. This is the guard against the
        // diagnostic probe ever turning into a user-facing error.
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [
                    { "message": { "role": "assistant", "content": "{\"cmd\": \"ls -la\"}" } }
                ]
            })))
            .mount(&server)
            .await;

        let backend =
            VllmBackend::new(Url::parse(&server.uri()).unwrap(), "test-model".to_string()).unwrap();

        let request = crate::models::CommandRequest {
            input: "list files".to_string(),
            shell: crate::models::ShellType::Bash,
            safety_level: crate::models::SafetyLevel::Moderate,
            context: None,
            backend_preference: None,
        };

        let result = backend.generate_command(&request).await.unwrap();
        assert_eq!(result.command, "ls -la");
    }

    #[test]
    fn test_vllm_with_api_key() {
        let url = Url::parse("https://api.example.com").unwrap();
        let backend = VllmBackend::new(url, "test".to_string())
            .unwrap()
            .with_api_key("test-key".to_string());
        assert!(backend.api_key.is_some());
    }

    #[test]
    fn test_parse_valid_json() {
        let url = Url::parse("https://api.example.com").unwrap();
        let backend = VllmBackend::new(url, "test".to_string()).unwrap();

        let response = r#"{"cmd": "grep -r 'pattern' ."}"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "grep -r 'pattern' .");
    }

    #[test]
    fn test_parse_embedded_json() {
        let url = Url::parse("https://api.example.com").unwrap();
        let backend = VllmBackend::new(url, "test".to_string()).unwrap();

        let response =
            r#"Sure! Here's the command: {"cmd": "sort file.txt"} Let me know if you need help."#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "sort file.txt");
    }

    #[test]
    fn test_parse_invalid_response() {
        let url = Url::parse("https://api.example.com").unwrap();
        let backend = VllmBackend::new(url, "test".to_string()).unwrap();

        let response = "I can't generate a command for that request.";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod confidence_tests {
    use super::*;

    #[test]
    fn logprobs_yield_measured_confidence() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"{\"cmd\":\"ls\"}"},
            "finish_reason":"stop","logprobs":{"content":[{"token":"a","logprob":-0.6931471805599453},
            {"token":"b","logprob":-0.6931471805599453}]}}]}"#;
        let parsed: VllmResponse = serde_json::from_str(body).unwrap();
        let tokens = token_logprobs(parsed.choices[0].logprobs.as_ref()).unwrap();
        let c = crate::backends::mean_logprob_confidence(
            &tokens.iter().map(|(_, lp)| *lp).collect::<Vec<_>>(),
        )
        .unwrap();
        assert!((c - 0.5).abs() < 1e-9);
    }

    #[test]
    fn command_tokens_are_scored_not_the_wrapper() {
        let half = 0.5_f64.ln();
        let body = format!(
            r#"{{"choices":[{{"message":{{"role":"assistant","content":"{{\"cmd\":\"ls\"}}"}},
            "logprobs":{{"content":[{{"token":"{{\"cmd\":\"","logprob":0.0}},{{"token":"ls","logprob":{half}}},{{"token":"\"}}","logprob":0.0}}]}}}}]}}"#
        );
        let parsed: VllmResponse = serde_json::from_str(&body).unwrap();
        let tokens = token_logprobs(parsed.choices[0].logprobs.as_ref()).unwrap();
        let c = crate::backends::command_token_confidence(&tokens, "ls").unwrap();
        assert!((c - 0.5).abs() < 1e-9, "{c}");
    }

    #[test]
    fn missing_logprobs_yield_unknown() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"{\"cmd\":\"ls\"}"},"finish_reason":"stop"}]}"#;
        let parsed: VllmResponse = serde_json::from_str(body).unwrap();
        assert_eq!(token_logprobs(parsed.choices[0].logprobs.as_ref()), None);
        let body =
            r#"{"choices":[{"message":{"role":"assistant","content":"x"},"logprobs":null}]}"#;
        let parsed: VllmResponse = serde_json::from_str(body).unwrap();
        assert_eq!(token_logprobs(parsed.choices[0].logprobs.as_ref()), None);
    }

    #[test]
    fn missing_token_text_yields_unknown() {
        // Incomplete logprob metadata (a token without text) must not become
        // measured confidence through the all-token fallback.
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"{\"cmd\":\"ls\"}"},
            "logprobs":{"content":[{"token":"{\"cmd","logprob":-0.1},{"logprob":-0.2}]}}]}"#;
        let parsed: VllmResponse = serde_json::from_str(body).unwrap();
        assert_eq!(token_logprobs(parsed.choices[0].logprobs.as_ref()), None);
    }

    mod constrained_decoding {
        use super::*;
        use crate::models::{RiskJudgeContext, SafetyLevel, ShellType};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, Request, ResponseTemplate};

        fn ctx() -> RiskJudgeContext {
            RiskJudgeContext {
                shell: ShellType::Bash,
                cwd: None,
                static_risk: RiskLevel::Safe,
                matched_patterns: vec![],
            }
        }

        fn backend(server: &MockServer) -> VllmBackend {
            VllmBackend::new(Url::parse(&server.uri()).unwrap(), "m".to_string()).unwrap()
        }

        fn body(req: &Request) -> serde_json::Value {
            serde_json::from_slice(&req.body).unwrap()
        }

        fn reply(content: &str) -> serde_json::Value {
            serde_json::json!({ "choices": [{ "message": { "role": "assistant", "content": content } }] })
        }

        #[tokio::test]
        async fn decision_request_sets_guided_json() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(reply(
                    r#"{"risk": "moderate", "reason": "writes", "confidence": 0.8}"#,
                )))
                .expect(1)
                .mount(&server)
                .await;

            let judgment = backend(&server).classify_risk("touch x", &ctx()).await;
            assert_eq!(judgment.map(|j| j.risk), Some(RiskLevel::Moderate));

            let reqs = server.received_requests().await.unwrap();
            let b = body(&reqs[0]);
            assert_eq!(
                b["guided_json"]["properties"]["risk"]["enum"],
                serde_json::json!(["critical", "high", "moderate", "safe"])
            );
        }

        #[tokio::test]
        async fn generation_request_has_no_guided_json() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/chat/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(reply(r#"{"cmd": "ls"}"#)))
                .mount(&server)
                .await;

            let request = CommandRequest {
                input: "list files".to_string(),
                shell: ShellType::Bash,
                safety_level: SafetyLevel::Moderate,
                context: None,
                backend_preference: None,
            };
            let generated = backend(&server).generate_command(&request).await.unwrap();
            assert_eq!(generated.command, "ls");

            // generate_command also fires a best-effort /v1/models probe
            // for the served-context-length check, so don't assume index 0
            // is the /v1/chat/completions request — find it by path instead.
            let reqs = server.received_requests().await.unwrap();
            let completion_req = reqs
                .iter()
                .find(|r| r.url.path() == "/v1/chat/completions")
                .expect("no /v1/chat/completions request recorded");
            assert!(body(completion_req).get("guided_json").is_none());
        }
    }
}

/// Contracts for the fail-closed local route, run against the real
/// `VllmBackend` over HTTP (wiremock). They prove the HTTP contract only; a
/// stand-alone server proves only the server. Neither is model inference.
#[cfg(test)]
mod fail_closed_tests {
    use super::*;
    use crate::models::{CommandRequest, SafetyLevel, ShellType};
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const MODEL: &str = "qwen2.5-coder-0.5b";

    fn req() -> CommandRequest {
        CommandRequest {
            input: "list files".to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: None,
            backend_preference: None,
        }
    }

    fn backend(server: &MockServer) -> VllmBackend {
        VllmBackend::new(Url::parse(&server.uri()).unwrap(), MODEL.to_string())
            .unwrap()
            .with_max_tokens(64)
            .with_logprobs(false)
            .fail_closed()
    }

    async fn healthy(server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
            .mount(server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"data": [{"id": MODEL}]})),
            )
            .mount(server)
            .await;
    }

    fn completion(model: Option<&str>, usage: bool) -> serde_json::Value {
        let mut v = json!({"choices": [{"message": {"role": "assistant",
            "content": "{\"cmd\": \"ls -la\"}"}}]});
        if let Some(m) = model {
            v["model"] = json!(m);
        }
        if usage {
            v["usage"] = json!({"prompt_tokens": 40, "completion_tokens": 12, "total_tokens": 52});
        }
        v
    }

    async fn mount_completion(server: &MockServer, resp: ResponseTemplate, times: u64) {
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(resp)
            .expect(times)
            .mount(server)
            .await;
    }

    /// A fallback that must never be reached.
    struct MustNotRun(Arc<AtomicBool>);

    #[async_trait]
    impl CommandGenerator for MustNotRun {
        async fn generate_command(
            &self,
            _: &CommandRequest,
        ) -> Result<GeneratedCommand, GeneratorError> {
            self.0.store(true, Ordering::SeqCst);
            Err(GeneratorError::Internal {
                message: "fallback ran".into(),
            })
        }
        async fn is_available(&self) -> bool {
            true
        }
        fn backend_info(&self) -> BackendInfo {
            unreachable!("not used")
        }
        async fn shutdown(&self) -> Result<(), GeneratorError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn sends_selected_model_cap_and_no_logprobs_and_reports_truthfully() {
        let server = MockServer::start().await;
        healthy(&server).await;
        mount_completion(
            &server,
            ResponseTemplate::new(200).set_body_json(completion(Some(MODEL), true)),
            1,
        )
        .await;
        let b = backend(&server);
        let out = b.generate_command(&req()).await.unwrap();
        assert_eq!(out.command, "ls -la");

        let posts: Vec<_> = server
            .received_requests()
            .await
            .unwrap()
            .into_iter()
            .filter(|r| r.method.as_str() == "POST")
            .collect();
        assert_eq!(posts.len(), 1, "exactly one real HTTP generation request");
        let body: serde_json::Value = serde_json::from_slice(&posts[0].body).unwrap();
        assert_eq!(body["model"], MODEL);
        assert_eq!(body["max_tokens"], 64);
        assert!(
            body.get("logprobs").is_none(),
            "logprobs=false omits the field"
        );

        let meta = b.last_response_meta().unwrap();
        assert_eq!(meta.requested_model, MODEL);
        assert_eq!(meta.returned_model.as_deref(), Some(MODEL));
        assert_eq!(
            (meta.prompt_tokens, meta.completion_tokens),
            (Some(40), Some(12))
        );
        assert!(out
            .explanation
            .contains("returned model `qwen2.5-coder-0.5b`"));
        assert!(out.explanation.contains("usage prompt=40 completion=12"));
        assert_eq!(
            out.confidence_source,
            crate::models::ConfidenceSource::Unknown
        );
    }

    #[tokio::test]
    async fn missing_usage_is_unverified_not_assumed() {
        let server = MockServer::start().await;
        healthy(&server).await;
        mount_completion(
            &server,
            ResponseTemplate::new(200).set_body_json(completion(Some(MODEL), false)),
            1,
        )
        .await;
        let b = backend(&server);
        let out = b.generate_command(&req()).await.unwrap();
        let meta = b.last_response_meta().unwrap();
        assert_eq!(meta.returned_model.as_deref(), Some(MODEL));
        assert_eq!(meta.completion_tokens, None);
        assert!(out.explanation.contains("usage unverified"));
    }

    /// Identity controls: only an exact model name passes. Each failure names
    /// the endpoint and the selected model, and still makes exactly one POST.
    async fn identity_outcome(body: serde_json::Value) -> Result<GeneratedCommand, GeneratorError> {
        let server = MockServer::start().await;
        healthy(&server).await;
        mount_completion(&server, ResponseTemplate::new(200).set_body_json(body), 1).await;
        let out = backend(&server).generate_command(&req()).await;
        if let Err(e) = &out {
            let text = e.to_string();
            assert!(text.contains(MODEL), "{text}");
            assert!(text.contains(&server.uri()), "{text}");
        }
        out
    }

    fn body_with_model(model: serde_json::Value) -> serde_json::Value {
        let mut body = completion(Some(MODEL), true);
        match model {
            serde_json::Value::Null => {
                body.as_object_mut().unwrap().remove("model");
            }
            other => {
                body["model"] = other;
            }
        }
        body
    }

    #[tokio::test]
    async fn absent_response_model_fails_closed() {
        let err = identity_outcome(body_with_model(serde_json::Value::Null))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("does not name the model"), "{err}");
    }

    #[tokio::test]
    async fn explicit_null_response_model_fails_closed() {
        let mut body = completion(Some(MODEL), true);
        body["model"] = serde_json::Value::Null;
        let err = identity_outcome(body).await.unwrap_err();
        assert!(err.to_string().contains("does not name the model"), "{err}");
    }

    #[tokio::test]
    async fn malformed_response_model_fails_closed() {
        for bad in [
            serde_json::json!(7),
            serde_json::json!(["x"]),
            serde_json::json!({"id": MODEL}),
        ] {
            let err = identity_outcome(body_with_model(bad)).await.unwrap_err();
            assert!(matches!(err, GeneratorError::ParseError { .. }), "{err:?}");
        }
    }

    #[tokio::test]
    async fn empty_or_near_miss_response_model_fails_closed() {
        for bad in [
            "",
            "Qwen2.5-Coder-0.5B",
            "qwen2.5-coder-0.5b ",
            "qwen2.5-coder-0.5b-extra",
        ] {
            let err = identity_outcome(body_with_model(serde_json::json!(bad)))
                .await
                .unwrap_err();
            assert!(
                err.to_string()
                    .contains("does not match the selected model"),
                "{bad:?}: {err}"
            );
        }
    }

    #[tokio::test]
    async fn exact_alias_response_model_passes() {
        let out = identity_outcome(body_with_model(serde_json::json!(MODEL)))
            .await
            .unwrap();
        assert_eq!(out.command, "ls -la");
    }

    #[tokio::test]
    async fn legacy_mode_still_accepts_a_reply_without_a_model() {
        // The identity requirement belongs to the explicit fail-closed route only.
        let server = MockServer::start().await;
        healthy(&server).await;
        mount_completion(
            &server,
            ResponseTemplate::new(200).set_body_json(completion(None, false)),
            1,
        )
        .await;
        let legacy =
            VllmBackend::new(Url::parse(&server.uri()).unwrap(), MODEL.to_string()).unwrap();
        let out = legacy.generate_command(&req()).await.unwrap();
        assert_eq!(out.command, "ls -la");
    }

    // ---- Strict structured content on the fail-closed route ----

    fn strict_backend() -> VllmBackend {
        VllmBackend::new(Url::parse("http://127.0.0.1:9").unwrap(), MODEL.to_string())
            .unwrap()
            .fail_closed()
    }

    /// Synthetic stand-in for the reported defect: unescaped inner quotes, then
    /// clarification fields trailing after the command. Data only, never run.
    const SALVAGED: &str = r#"{"cmd": "find . -name "*.py"", "needs_clarification": true, "p": 0.9, "question": "which directory?"}"#;

    #[test]
    fn strict_parser_accepts_only_the_exact_shapes() {
        let b = strict_backend();
        let ok = |raw: &str| b.parse_strict_reply(raw).unwrap();
        assert_eq!(ok(r#"{"cmd": "ls -la"}"#), "ls -la");
        assert_eq!(ok("  {\"cmd\": \" ls \"}\n"), "ls");
        // Legitimate escaped quotes inside the JSON string.
        assert_eq!(
            ok(r#"{"cmd": "find . -name \"*.py\""}"#),
            r#"find . -name "*.py""#
        );
        // One whole-message fence, with or without the json tag.
        assert_eq!(ok("```json\n{\"cmd\": \"ls\"}\n```"), "ls");
        assert_eq!(ok("```\n{\"cmd\": \"ls\"}\n```\n"), "ls");
    }

    #[test]
    fn strict_parser_rejects_everything_else_without_salvage() {
        let b = strict_backend();
        let bad = [
            SALVAGED,
            r#"{"cmd": "ls"} trailing"#,
            r#"Sure! {"cmd": "ls"}"#,
            r#"{"cmd": "ls"}{"cmd": "pwd"}"#,
            r#"{"cmd": "ls", "extra": 1}"#,
            r#"{"cmd": "ls", "cmd": "pwd"}"#,
            r#"{"cmd": ""}"#,
            r#"{"cmd": "   "}"#,
            r#"{"cmd": 5}"#,
            r#"{"cmd": null}"#,
            r#"{"cmd": ["ls"]}"#,
            r#"{}"#,
            r#"[]"#,
            r#"cmd: ls"#,
            r#"{"cmd": "ls""#,
            r#"{'cmd': 'ls'}"#,
            // contradictory decisions
            r#"{"cmd": "ls", "needs_clarification": true, "question": "q"}"#,
            r#"{"cmd": "ls", "needs_clarification": false}"#,
            r#"{"needs_clarification": false}"#,
            r#"{"needs_clarification": false, "question": "q"}"#,
            r#"{"needs_clarification": "true", "question": "q"}"#,
            r#"{"needs_clarification": true}"#,
            r#"{"needs_clarification": true, "question": ""}"#,
            r#"{"needs_clarification": true, "p": 2.0, "question": "q"}"#,
            r#"{"needs_clarification": true, "p": "high", "question": "q"}"#,
            r#"{"needs_clarification": true, "p": 0.2, "question": "q"}"#,
            // fence abuse
            "```json {\"cmd\": \"ls\"} ```",
            "text\n```json\n{\"cmd\": \"ls\"}\n```",
            "```json\n{\"cmd\": \"ls\"}\n```\nmore",
            "```json\n{\"cmd\": \"ls\"}\n```\n```json\n{\"cmd\": \"pwd\"}\n```",
            "```json\nnot json\n```",
            "```python\n{\"cmd\": \"ls\"}\n```",
            "```json\n{\"cmd\": \"ls\"}",
        ];
        for raw in bad {
            let err = b.parse_strict_reply(raw);
            assert!(err.is_err(), "must reject, got {err:?} for {raw}");
        }
    }

    #[test]
    fn strict_parser_keeps_typed_clarification() {
        let b = strict_backend();
        for raw in [
            r#"{"needs_clarification": true, "p": 0.9, "question": "Which directory?"}"#,
            r#"{"needs_clarification": true, "question": "Which directory?"}"#,
            "```json\n{\"needs_clarification\": true, \"p\": 0.8, \"question\": \"Which?\"}\n```",
        ] {
            match b.parse_strict_reply(raw) {
                Err(GeneratorError::NeedsClarification { question, .. }) => {
                    assert!(question.unwrap().ends_with('?'));
                }
                other => panic!("{raw}: {other:?}"),
            }
        }
        // The legacy "echo a question" command is not a runnable command.
        assert!(matches!(
            b.parse_strict_reply(r#"{"cmd": "echo 'Please clarify which directory'"}"#),
            Err(GeneratorError::NeedsClarification { .. })
        ));
    }

    #[test]
    fn legacy_parser_is_unchanged_and_still_salvages() {
        // Control for scope: the lenient parser outside the explicit route is
        // untouched. This is the baseline behavior the strict route refuses.
        let legacy =
            VllmBackend::new(Url::parse("http://127.0.0.1:9").unwrap(), MODEL.to_string()).unwrap();
        assert!(legacy.parse_command_response(SALVAGED).is_ok());
    }

    async fn content_outcome(content: &str) -> Result<GeneratedCommand, GeneratorError> {
        let server = MockServer::start().await;
        healthy(&server).await;
        let mut body = completion(Some(MODEL), true);
        body["choices"][0]["message"]["content"] = json!(content);
        mount_completion(&server, ResponseTemplate::new(200).set_body_json(body), 1).await;
        backend(&server).generate_command(&req()).await
    }

    #[tokio::test]
    async fn explicit_route_rejects_malformed_and_ambiguous_content() {
        for raw in [
            SALVAGED,
            r#"{"cmd": "ls"} trailing"#,
            r#"{"cmd": "ls", "needs_clarification": true, "question": "q"}"#,
            r#"{"cmd": ""}"#,
            r#"{"cmd": 7}"#,
        ] {
            let err = content_outcome(raw).await.unwrap_err();
            match &err {
                GeneratorError::ParseError { content } => {
                    assert!(
                        content.contains(MODEL) && content.contains("127.0.0.1"),
                        "{content}"
                    );
                }
                other => panic!("{raw}: {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn explicit_route_accepts_valid_content_and_surfaces_clarification() {
        let out = content_outcome(r#"{"cmd": "find . -name \"*.py\""}"#)
            .await
            .unwrap();
        assert_eq!(out.command, r#"find . -name "*.py""#);
        let out = content_outcome("```json\n{\"cmd\": \"ls -la\"}\n```")
            .await
            .unwrap();
        assert_eq!(out.command, "ls -la");
        let err = content_outcome(
            r#"{"needs_clarification": true, "p": 0.9, "question": "Which directory?"}"#,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, GeneratorError::NeedsClarification { .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn response_naming_another_model_fails() {
        let server = MockServer::start().await;
        healthy(&server).await;
        mount_completion(
            &server,
            ResponseTemplate::new(200).set_body_json(completion(Some("other-model"), true)),
            1,
        )
        .await;
        let err = backend(&server).generate_command(&req()).await.unwrap_err();
        match err {
            GeneratorError::GenerationFailed { details } => {
                assert!(details.contains("other-model"), "{details}");
                assert!(details.contains(MODEL), "{details}");
                assert!(details.contains("does not match"), "{details}");
            }
            other => panic!("expected GenerationFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn selected_model_must_be_listed_and_first_entry_is_not_substituted() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"data": [{"id": "some-other-model"}, {"id": "third"}]})),
            )
            .mount(&server)
            .await;
        mount_completion(&server, ResponseTemplate::new(200), 0).await;
        let err = backend(&server).generate_command(&req()).await.unwrap_err();
        match err {
            GeneratorError::BackendUnavailable { reason } => {
                assert!(
                    reason.contains(MODEL) && reason.contains("is not served"),
                    "{reason}"
                );
                assert!(
                    reason.contains("some-other-model"),
                    "lists what is served: {reason}"
                );
                assert!(
                    reason.contains(&server.uri()),
                    "names the endpoint: {reason}"
                );
            }
            other => panic!("expected BackendUnavailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn not_ready_health_is_unavailable_and_nothing_is_generated() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(
                ResponseTemplate::new(503)
                    .set_body_json(json!({"error": {"code": 503, "message": "Loading model"}})),
            )
            .mount(&server)
            .await;
        mount_completion(&server, ResponseTemplate::new(200), 0).await;
        let err = backend(&server).generate_command(&req()).await.unwrap_err();
        match err {
            GeneratorError::BackendUnavailable { reason } => {
                assert!(
                    reason.contains("not ready") && reason.contains("503"),
                    "{reason}"
                );
            }
            other => panic!("expected BackendUnavailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn unreachable_endpoint_names_endpoint_and_model() {
        // Port 9 (discard) on loopback: nothing listens, the connect is refused.
        let b = VllmBackend::new(Url::parse("http://127.0.0.1:9").unwrap(), MODEL.to_string())
            .unwrap()
            .fail_closed();
        match b.generate_command(&req()).await.unwrap_err() {
            GeneratorError::BackendUnavailable { reason } => {
                assert!(
                    reason.contains("127.0.0.1:9") && reason.contains(MODEL),
                    "{reason}"
                );
                assert!(reason.contains("unreachable"), "{reason}");
            }
            other => panic!("expected BackendUnavailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn bad_status_models_status_and_bad_bodies_stay_distinct() {
        // /v1/models 500
        let s = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&s)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&s)
            .await;
        let e = backend(&s).generate_command(&req()).await.unwrap_err();
        assert!(
            matches!(&e, GeneratorError::GenerationFailed { details } if details.contains("/v1/models returned 500")),
            "{e:?}"
        );

        // /v1/models not JSON
        let s = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&s)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&s)
            .await;
        let e = backend(&s).generate_command(&req()).await.unwrap_err();
        assert!(
            matches!(&e, GeneratorError::ParseError { content } if content.contains("not a model list")),
            "{e:?}"
        );

        // chat 500
        let s = MockServer::start().await;
        healthy(&s).await;
        mount_completion(&s, ResponseTemplate::new(500), 1).await;
        let e = backend(&s).generate_command(&req()).await.unwrap_err();
        assert!(
            matches!(&e, GeneratorError::GenerationFailed { details } if details.contains("500") && details.contains(MODEL)),
            "{e:?}"
        );

        // chat 200 with a non-OpenAI body
        let s = MockServer::start().await;
        healthy(&s).await;
        mount_completion(&s, ResponseTemplate::new(200).set_body_string("<html>"), 1).await;
        let e = backend(&s).generate_command(&req()).await.unwrap_err();
        assert!(
            matches!(&e, GeneratorError::ParseError { content } if content.contains("failed to parse response")),
            "{e:?}"
        );

        // chat 200 whose reply is not a command
        let s = MockServer::start().await;
        healthy(&s).await;
        let body = json!({"model": MODEL, "choices": [{"message": {"role": "assistant", "content": "I cannot help"}}]});
        mount_completion(&s, ResponseTemplate::new(200).set_body_json(body), 1).await;
        let e = backend(&s).generate_command(&req()).await.unwrap_err();
        assert!(matches!(e, GeneratorError::ParseError { .. }), "{e:?}");

        // chat 401
        let s = MockServer::start().await;
        healthy(&s).await;
        mount_completion(&s, ResponseTemplate::new(401), 1).await;
        let e = backend(&s).generate_command(&req()).await.unwrap_err();
        assert!(
            matches!(&e, GeneratorError::BackendUnavailable { reason } if reason.contains("Authentication failed")),
            "{e:?}"
        );
    }

    #[tokio::test]
    async fn http_400_is_surfaced_once_and_never_retried() {
        let server = MockServer::start().await;
        healthy(&server).await;
        // Exactly one POST is allowed; a retry would fail the `expect(1)`.
        mount_completion(&server, ResponseTemplate::new(400), 1).await;
        let b = VllmBackend::new(Url::parse(&server.uri()).unwrap(), MODEL.to_string())
            .unwrap()
            .with_logprobs(true)
            .fail_closed();
        let e = b.generate_command(&req()).await.unwrap_err();
        assert!(
            matches!(&e, GeneratorError::GenerationFailed { details } if details.contains("400")),
            "{e:?}"
        );
    }

    #[tokio::test]
    async fn no_fallback_runs_in_either_attachment_order() {
        for attach_after in [true, false] {
            let server = MockServer::start().await;
            healthy(&server).await;
            mount_completion(&server, ResponseTemplate::new(500), 1).await;
            let ran = Arc::new(AtomicBool::new(false));
            let fb: Arc<dyn CommandGenerator> = Arc::new(MustNotRun(ran.clone()));
            let base =
                VllmBackend::new(Url::parse(&server.uri()).unwrap(), MODEL.to_string()).unwrap();
            let b = if attach_after {
                base.fail_closed().with_embedded_fallback(fb)
            } else {
                base.with_embedded_fallback(fb).fail_closed()
            };
            assert!(b.generate_command(&req()).await.is_err());
            assert!(
                !ran.load(Ordering::SeqCst),
                "fallback ran (attach_after={attach_after})"
            );
        }
    }

    #[tokio::test]
    async fn timeout_is_configurable_and_reported_with_endpoint() {
        let server = MockServer::start().await;
        healthy(&server).await;
        mount_completion(
            &server,
            ResponseTemplate::new(200)
                .set_body_json(completion(Some(MODEL), true))
                .set_delay(Duration::from_millis(1500)),
            1,
        )
        .await;
        let b = backend(&server)
            .with_timeout(Duration::from_millis(200))
            .unwrap();
        match b.generate_command(&req()).await.unwrap_err() {
            GeneratorError::BackendUnavailable { reason } => {
                assert!(
                    reason.contains("timed out") && reason.contains(MODEL),
                    "{reason}"
                );
            }
            other => panic!("expected BackendUnavailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn legacy_mode_still_sends_logprobs_and_default_cap() {
        let server = MockServer::start().await;
        mount_completion(
            &server,
            ResponseTemplate::new(200).set_body_json(completion(None, false)),
            1,
        )
        .await;
        let b = VllmBackend::new(Url::parse(&server.uri()).unwrap(), "legacy".to_string()).unwrap();
        b.generate_command(&req()).await.unwrap();
        let posts: Vec<_> = server
            .received_requests()
            .await
            .unwrap()
            .into_iter()
            .filter(|r| r.method.as_str() == "POST")
            .collect();
        let body: serde_json::Value = serde_json::from_slice(&posts[0].body).unwrap();
        assert_eq!(body["logprobs"], true);
        assert_eq!(body["max_tokens"], 100);
    }

    #[tokio::test]
    async fn auth_empty_choices_and_non_command_replies_name_endpoint_and_model() {
        // 401: BackendUnavailable, names endpoint + model, says the route is keyless.
        let s = MockServer::start().await;
        healthy(&s).await;
        mount_completion(&s, ResponseTemplate::new(401), 1).await;
        match backend(&s).generate_command(&req()).await.unwrap_err() {
            GeneratorError::BackendUnavailable { reason } => {
                assert!(
                    reason.contains("Authentication failed") && reason.contains("401"),
                    "{reason}"
                );
                assert!(
                    reason.contains(&s.uri()) && reason.contains(MODEL),
                    "{reason}"
                );
                assert!(reason.contains("keyless"), "{reason}");
            }
            other => panic!("expected BackendUnavailable, got {other:?}"),
        }

        // 200 with no choices: ParseError, names endpoint + model.
        let s = MockServer::start().await;
        healthy(&s).await;
        mount_completion(
            &s,
            ResponseTemplate::new(200).set_body_json(json!({"model": MODEL, "choices": []})),
            1,
        )
        .await;
        match backend(&s).generate_command(&req()).await.unwrap_err() {
            GeneratorError::ParseError { content } => {
                assert!(content.contains("no choices"), "{content}");
                assert!(
                    content.contains(&s.uri()) && content.contains(MODEL),
                    "{content}"
                );
            }
            other => panic!("expected ParseError, got {other:?}"),
        }

        // 200 whose reply is not a command: ParseError, names endpoint + model.
        let s = MockServer::start().await;
        healthy(&s).await;
        let body = json!({"model": MODEL, "choices": [{"message": {"role": "assistant", "content": "I cannot help"}}]});
        mount_completion(&s, ResponseTemplate::new(200).set_body_json(body), 1).await;
        match backend(&s).generate_command(&req()).await.unwrap_err() {
            GeneratorError::ParseError { content } => {
                assert!(content.contains("reply is not a command"), "{content}");
                assert!(
                    content.contains(&s.uri()) && content.contains(MODEL),
                    "{content}"
                );
            }
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn selected_alias_among_several_is_accepted_and_absence_is_not() {
        let s = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&s)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    json!({"data": [{"id": "first"}, {"id": MODEL}, {"id": "third"}]}),
                ),
            )
            .mount(&s)
            .await;
        mount_completion(
            &s,
            ResponseTemplate::new(200).set_body_json(completion(Some(MODEL), true)),
            1,
        )
        .await;
        assert_eq!(
            backend(&s).generate_command(&req()).await.unwrap().command,
            "ls -la"
        );
    }

    #[test]
    fn explicit_from_config_honors_endpoint_model_and_limits() {
        let cfg = crate::models::BackendsConfig {
            vllm_url: Some("http://127.0.0.1:8123".into()),
            vllm_max_tokens: Some(64),
            vllm_timeout_secs: Some(7),
            vllm_logprobs: Some(false),
            ..Default::default()
        };
        let b = explicit_from_config(&cfg, Some("my-alias")).unwrap();
        assert_eq!(b.base_url.as_str(), "http://127.0.0.1:8123/");
        assert_eq!(b.model_name, "my-alias");
        assert_eq!(b.max_tokens, 64);
        assert!(!b.logprobs && b.fail_closed && b.embedded_fallback.is_none());
        assert_eq!(b.api_key, None, "keyless");

        let d = explicit_from_config(&Default::default(), None).unwrap();
        assert_eq!(d.model_name, DEFAULT_MODEL);
        assert_eq!(d.base_url.as_str(), "http://localhost:8000/");
        assert!(d.logprobs && d.max_tokens == 100);

        let bad = crate::models::BackendsConfig {
            vllm_url: Some("not a url".into()),
            ..Default::default()
        };
        assert!(matches!(
            explicit_from_config(&bad, None),
            Err(GeneratorError::ConfigError { .. })
        ));
    }
}
