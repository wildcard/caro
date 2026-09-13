// Ollama local server backend implementation

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;

use crate::backends::remote::context_length_warning;
use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};

/// Regex pattern to extract command from malformed JSON with unescaped quotes
/// Handles cases like: {"cmd": "find . -type f -name "*.txt""}
static CMD_EXTRACT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{\s*"cmd"\s*:\s*"(.+)"\s*\}"#).expect("Invalid regex pattern"));
use crate::models::{CommandRequest, GeneratedCommand, RiskJudgeContext, RiskJudgment, RiskLevel};
use crate::prompts::{build_risk_judge_prompt, parse_risk_judgment};

/// Ollama API request format
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

/// Ollama inference options
#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_k: u32,
    top_p: f32,
    num_predict: u32,
}

/// Ollama API response format
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
    #[allow(dead_code)]
    context: Option<Vec<i32>>,
    #[allow(dead_code)]
    created_at: Option<String>,
}

/// Response shape for `POST /api/show`, used only to check the context
/// length actually in effect for the configured model.
#[derive(Debug, Deserialize)]
struct OllamaShowResponse {
    #[serde(default)]
    parameters: Option<String>,
    #[serde(default)]
    model_info: Option<serde_json::Value>,
}

/// Ollama's own default context length when nothing overrides it via a
/// Modelfile `PARAMETER num_ctx` line.
const OLLAMA_DEFAULT_NUM_CTX: u32 = 2048;

/// Parse a `num_ctx <value>` line out of `/api/show`'s `parameters` string,
/// which lists one explicit Modelfile override per line (e.g.
/// `"num_ctx 8192\nstop \"</s>\"\n"`). Returns `None` when no such line is
/// present, or if present but not a valid integer.
fn parse_num_ctx(parameters: &str) -> Option<u32> {
    parameters.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        if parts.next()? == "num_ctx" {
            parts.next()?.parse::<u32>().ok()
        } else {
            None
        }
    })
}

/// Extract a model's declared/max context length from `/api/show`'s
/// `model_info` object. The key is architecture-prefixed (e.g.
/// `"llama.context_length"`, `"qwen2.context_length"`), so this scans for
/// any key ending in `.context_length` rather than hardcoding an arch name.
fn extract_declared_context_length(model_info: &serde_json::Value) -> Option<u32> {
    let object = model_info.as_object()?;
    object.iter().find_map(|(key, value)| {
        if key.ends_with(".context_length") {
            value.as_u64().and_then(|n| u32::try_from(n).ok())
        } else {
            None
        }
    })
}

/// Ollama backend for local Ollama server
pub struct OllamaBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
    /// Ensures the served-context-length probe (`/api/show`) and its log
    /// line fire at most once per backend instance, not on every request.
    checked_context: OnceCell<()>,
}

impl OllamaBackend {
    /// Create a new Ollama backend
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
            embedded_fallback: None,
            checked_context: OnceCell::new(),
        })
    }

    /// Add embedded fallback backend
    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }

    /// Create system prompt for Ollama
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
7. If the request is unclear, generate "echo 'Please clarify your request'"

Request: {}
"#,
            request.shell, request.input
        )
    }

    /// Parse JSON response from Ollama
    fn parse_command_response(&self, response: &str) -> Result<String, GeneratorError> {
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

    /// Call Ollama API for inference
    async fn call_ollama_api(&self, prompt: &str) -> Result<String, GeneratorError> {
        let request = OllamaRequest {
            model: self.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature: 0.1,
                top_k: 10,
                top_p: 0.3,
                num_predict: 100,
            },
        };

        let url = self
            .base_url
            .join("/api/generate")
            .map_err(|e| GeneratorError::ConfigError {
                message: format!("Invalid base URL: {}", e),
            })?;

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    GeneratorError::BackendUnavailable {
                        reason: format!("Ollama server unavailable: {}", e),
                    }
                } else {
                    GeneratorError::GenerationFailed {
                        details: format!("HTTP request failed: {}", e),
                    }
                }
            })?;

        if !response.status().is_success() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("Ollama API error: {}", response.status()),
            });
        }

        let ollama_response: OllamaResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse Ollama response: {}", e),
                })?;

        Ok(ollama_response.response)
    }

    /// Fetch the context length actually in effect for `self.model_name`,
    /// plus the model's own declared/max context length when Ollama's
    /// `model_info` exposes it. Returns `(served, declared)`.
    ///
    /// Best-effort only: any transport error, non-2xx status, or unexpected
    /// body shape yields `None` rather than an error — this is a diagnostic
    /// nicety, not something that should ever block generation.
    async fn fetch_ollama_context_lengths(&self) -> Option<(u32, Option<u32>)> {
        let url = self.base_url.join("/api/show").ok()?;
        let body = serde_json::json!({ "model": self.model_name });

        let response = self.client.post(url).json(&body).send().await.ok()?;
        if !response.status().is_success() {
            return None;
        }

        let show: OllamaShowResponse = response.json().await.ok()?;
        let served = show
            .parameters
            .as_deref()
            .and_then(parse_num_ctx)
            .unwrap_or(OLLAMA_DEFAULT_NUM_CTX);
        let declared = show
            .model_info
            .as_ref()
            .and_then(extract_declared_context_length);

        Some((served, declared))
    }

    /// Probe the served context length once and log a warning if it looks
    /// undersized. Never surfaces as a `GeneratorError` — see
    /// `fetch_ollama_context_lengths`.
    async fn warn_on_context_length(&self) {
        if let Some((served, declared)) = self.fetch_ollama_context_lengths().await {
            if let Some(warning) = context_length_warning(served, declared) {
                tracing::warn!("Ollama ({}): {}", self.model_name, warning);
            }
        } else {
            tracing::debug!(
                "Ollama ({}): could not determine served context length via /api/show",
                self.model_name
            );
        }
    }

    /// Attempt inference with fallback to embedded backend
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        self.checked_context
            .get_or_init(|| self.warn_on_context_length())
            .await;

        // Try Ollama first
        match self
            .call_ollama_api(&self.create_system_prompt(request))
            .await
        {
            Ok(response) => {
                match self.parse_command_response(&response) {
                    Ok(command) => {
                        return Ok(GeneratedCommand {
                            command,
                            explanation: "Generated using Ollama local server".to_string(),
                            safety_level: RiskLevel::Safe, // TODO: Implement safety validation
                            estimated_impact: "Low impact local operation".to_string(),
                            alternatives: vec![],
                            backend_used: format!("Ollama ({})", self.model_name),
                            generation_time_ms: 0, // Will be set by caller
                            confidence_score: 0.8,
                        });
                    }
                    Err(parse_error) => {
                        tracing::warn!("Failed to parse Ollama response: {}", parse_error);
                        // Continue to fallback
                    }
                }
            }
            Err(ollama_error) => {
                tracing::warn!("Ollama backend failed: {}", ollama_error);
                // Continue to fallback
            }
        }

        // Fallback to embedded backend if available
        if let Some(fallback) = &self.embedded_fallback {
            tracing::info!("Falling back to embedded backend");
            let mut fallback_result = fallback.generate_command(request).await?;
            fallback_result.backend_used =
                format!("Embedded (Ollama fallback from {})", self.model_name);
            return Ok(fallback_result);
        }

        // No fallback available
        Err(GeneratorError::BackendUnavailable {
            reason: "Ollama server unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for OllamaBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = std::time::Instant::now();

        let mut result = self.generate_with_fallback(request).await?;
        result.generation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(result)
    }

    async fn classify_risk(&self, command: &str, ctx: &RiskJudgeContext) -> Option<RiskJudgment> {
        // Reuse the existing prompt→text path; fail safe to `None` on any error
        // so the caller falls back to the static decision.
        let prompt = build_risk_judge_prompt(command, ctx);
        let raw = self.call_ollama_api(&prompt).await.ok()?;
        parse_risk_judgment(&raw)
    }

    async fn is_available(&self) -> bool {
        // Check if Ollama server is responding
        let health_url = match self.base_url.join("/api/tags") {
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
            backend_type: BackendType::Ollama,
            model_name: self.model_name.clone(),
            supports_streaming: false,
            max_tokens: 100,
            typical_latency_ms: 2000,
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
    fn test_ollama_backend_creation() {
        let url = Url::parse("http://localhost:11434").unwrap();
        let backend = OllamaBackend::new(url, "codellama:7b".to_string());
        assert!(backend.is_ok());
    }

    #[test]
    fn test_parse_num_ctx_finds_explicit_override() {
        assert_eq!(parse_num_ctx("num_ctx 8192\nstop \"</s>\"\n"), Some(8192));
    }

    #[test]
    fn test_parse_num_ctx_absent() {
        assert_eq!(parse_num_ctx("stop \"</s>\"\ntemperature 0.1\n"), None);
    }

    #[test]
    fn test_parse_num_ctx_empty_string() {
        assert_eq!(parse_num_ctx(""), None);
    }

    #[test]
    fn test_parse_num_ctx_malformed_value_does_not_panic() {
        assert_eq!(parse_num_ctx("num_ctx not-a-number\n"), None);
    }

    #[test]
    fn test_extract_declared_context_length_ignores_arch_prefix() {
        let model_info = serde_json::json!({ "llama.context_length": 131_072_u64 });
        assert_eq!(extract_declared_context_length(&model_info), Some(131_072));

        let model_info = serde_json::json!({ "qwen2.context_length": 32_768_u64 });
        assert_eq!(extract_declared_context_length(&model_info), Some(32_768));
    }

    #[test]
    fn test_extract_declared_context_length_missing_key() {
        let model_info = serde_json::json!({ "llama.vocab_size": 32_000_u64 });
        assert_eq!(extract_declared_context_length(&model_info), None);
    }

    #[tokio::test]
    async fn test_fetch_ollama_context_lengths_reports_declared_and_served() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/show"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "parameters": "num_ctx 2048\n",
                "model_info": { "llama.context_length": 131_072_u64 }
            })))
            .mount(&server)
            .await;

        let backend =
            OllamaBackend::new(Url::parse(&server.uri()).unwrap(), "test-model".to_string())
                .unwrap();

        let result = backend.fetch_ollama_context_lengths().await;
        assert_eq!(result, Some((2048, Some(131_072))));
    }

    #[tokio::test]
    async fn test_fetch_ollama_context_lengths_defaults_when_no_override() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/show"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "parameters": "temperature 0.1\n",
                "model_info": { "llama.context_length": 8192_u64 }
            })))
            .mount(&server)
            .await;

        let backend =
            OllamaBackend::new(Url::parse(&server.uri()).unwrap(), "test-model".to_string())
                .unwrap();

        let result = backend.fetch_ollama_context_lengths().await;
        assert_eq!(result, Some((OLLAMA_DEFAULT_NUM_CTX, Some(8192))));
    }

    #[tokio::test]
    async fn test_fetch_ollama_context_lengths_none_on_server_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/show"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let backend =
            OllamaBackend::new(Url::parse(&server.uri()).unwrap(), "test-model".to_string())
                .unwrap();

        assert_eq!(backend.fetch_ollama_context_lengths().await, None);
    }

    #[tokio::test]
    async fn test_context_length_probe_failure_does_not_block_generation() {
        // /api/show is broken, but /api/generate works fine — generation
        // must still succeed. This is the guard against the diagnostic
        // probe ever turning into a user-facing error.
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/show"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": "{\"cmd\": \"ls -la\"}",
                "done": true
            })))
            .mount(&server)
            .await;

        let backend =
            OllamaBackend::new(Url::parse(&server.uri()).unwrap(), "test-model".to_string())
                .unwrap();

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
    fn test_parse_valid_json() {
        let url = Url::parse("http://localhost:11434").unwrap();
        let backend = OllamaBackend::new(url, "test".to_string()).unwrap();

        let response = r#"{"cmd": "ls -la"}"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "ls -la");
    }

    #[test]
    fn test_parse_embedded_json() {
        let url = Url::parse("http://localhost:11434").unwrap();
        let backend = OllamaBackend::new(url, "test".to_string()).unwrap();

        let response = r#"Here is the command: {"cmd": "find . -name '*.rs'"} Hope this helps!"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "find . -name '*.rs'");
    }

    #[test]
    fn test_parse_invalid_response() {
        let url = Url::parse("http://localhost:11434").unwrap();
        let backend = OllamaBackend::new(url, "test".to_string()).unwrap();

        let response = "This is not a valid JSON response";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }
}
