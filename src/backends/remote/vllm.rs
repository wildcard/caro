// vLLM server backend implementation

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::decision::decide_with_retry;
use crate::models::{RiskJudgeContext, RiskJudgment};
use crate::prompts::{build_risk_judge_prompt, parse_risk_judgment, risk_judge_schema};

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
    #[allow(dead_code)]
    usage: Option<VllmUsage>,
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
    #[allow(dead_code)]
    prompt_tokens: u32,
    #[allow(dead_code)]
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

/// vLLM backend for remote vLLM server
pub struct VllmBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    api_key: Option<String>,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
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
        })
    }

    /// Set API key for authentication
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Add embedded fallback backend
    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
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
            max_tokens: 100,
            stream: false,
            logprobs: true,
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
                    reason: format!("vLLM server unavailable: {}", e),
                }
            } else {
                GeneratorError::GenerationFailed {
                    details: format!("HTTP request failed: {}", e),
                }
            }
        })?;

        // Check for authentication errors
        if response.status() == 401 || response.status() == 403 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Authentication failed - invalid API key".to_string(),
            });
        }

        if !response.status().is_success() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("vLLM API error: {}", response.status()),
            });
        }

        let vllm_response: VllmResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse vLLM response: {}", e),
                })?;

        if let Some(choice) = vllm_response.choices.first() {
            let tokens = token_logprobs(choice.logprobs.as_ref());
            Ok((choice.message.content.clone(), tokens))
        } else {
            Err(GeneratorError::ParseError {
                content: "vLLM response contained no choices".to_string(),
            })
        }
    }

    /// Attempt inference with fallback to embedded backend
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
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
            max_tokens: 100,
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

    #[test]
    fn test_vllm_backend_creation() {
        let url = Url::parse("https://api.example.com").unwrap();
        let backend = VllmBackend::new(url, "codellama/CodeLlama-7b-hf".to_string());
        assert!(backend.is_ok());
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

            let reqs = server.received_requests().await.unwrap();
            assert!(body(&reqs[0]).get("guided_json").is_none());
        }
    }
}
