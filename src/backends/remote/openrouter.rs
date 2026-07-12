// OpenRouter backend implementation
//
// OpenRouter provides a unified API for 100+ LLMs (OpenAI-compatible endpoint).
// Docs: https://openrouter.ai/docs

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

const DEFAULT_ENDPOINT: &str = "https://openrouter.ai/api/v1";
const DEFAULT_MODEL: &str = "qwen/qwen3-coder";
const DEFAULT_MAX_TOKENS: u32 = 512;
const DEFAULT_TEMPERATURE: f32 = 0.1;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

static CMD_EXTRACT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{\s*"cmd"\s*:\s*"(.+)"\s*\}"#).expect("Invalid regex pattern"));

#[derive(Clone)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl std::fmt::Debug for OpenRouterConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenRouterConfig")
            .field("api_key", &"[REDACTED]")
            .field("model", &self.model)
            .field("endpoint", &self.endpoint)
            .field("max_tokens", &self.max_tokens)
            .field("temperature", &self.temperature)
            .finish()
    }
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: DEFAULT_MODEL.to_string(),
            endpoint: DEFAULT_ENDPOINT.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
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

pub struct OpenRouterBackend {
    config: OpenRouterConfig,
    client: Client,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

impl OpenRouterBackend {
    pub fn new(config: OpenRouterConfig) -> Result<Self, GeneratorError> {
        if config.api_key.is_empty() {
            return Err(GeneratorError::ConfigError {
                message: "OpenRouter API key is required (set OPENROUTER_API_KEY)".to_string(),
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
        })
    }

    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
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
7. If the request is unclear, generate "echo 'Please clarify your request'"
"#,
            request.shell
        )
    }

    fn parse_command_response(&self, response: &str) -> Result<String, GeneratorError> {
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
    ) -> Result<String, GeneratorError> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: format!("<user_request>{}</user_request>", user_input),
                },
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: false,
        };

        let url = format!("{}/chat/completions", self.config.endpoint);

        let response = self
            .client
            .post(&url)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.config.api_key),
            )
            .header("HTTP-Referer", "https://github.com/wildcard/caro")
            .header("X-Title", "caro")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    GeneratorError::BackendUnavailable {
                        reason: format!("OpenRouter unavailable: {}", e),
                    }
                } else {
                    GeneratorError::GenerationFailed {
                        details: format!("HTTP request failed: {}", e),
                    }
                }
            })?;

        if response.status() == 401 || response.status() == 403 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "OpenRouter authentication failed - check OPENROUTER_API_KEY".to_string(),
            });
        }

        if !response.status().is_success() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("OpenRouter API error: {}", response.status()),
            });
        }

        let chat_response: ChatResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse OpenRouter response: {}", e),
                })?;

        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(GeneratorError::ParseError {
                content: "OpenRouter response contained no choices".to_string(),
            })
        }
    }

    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        match self
            .call_api(&self.create_system_prompt(request), &request.input)
            .await
        {
            Ok(response) => match self.parse_command_response(&response) {
                Ok(command) => {
                    return Ok(GeneratedCommand {
                        command,
                        explanation: "Generated using OpenRouter".to_string(),
                        safety_level: RiskLevel::Moderate,
                        estimated_impact: "Remote inference operation".to_string(),
                        alternatives: vec![],
                        backend_used: format!("OpenRouter ({})", self.config.model),
                        generation_time_ms: 0,
                        confidence_score: 0.85,
                    });
                }
                Err(parse_error) => {
                    tracing::warn!("Failed to parse OpenRouter response: {}", parse_error);
                }
            },
            Err(api_error) => {
                tracing::warn!("OpenRouter backend failed: {}", api_error);
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
            fallback_result.backend_used =
                format!("Embedded (OpenRouter fallback from {})", self.config.model);
            return Ok(fallback_result);
        }

        Err(GeneratorError::BackendUnavailable {
            reason: "OpenRouter unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for OpenRouterBackend {
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

    async fn is_available(&self) -> bool {
        let url = format!("{}/models", self.config.endpoint);
        let mut req = self.client.get(&url);
        if !self.config.api_key.is_empty() {
            req = req.header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.config.api_key),
            );
        }
        match req.send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::OpenRouter,
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
        let config = OpenRouterConfig::default();
        assert_eq!(config.model, "qwen/qwen3-coder");
        assert_eq!(config.endpoint, "https://openrouter.ai/api/v1");
        assert_eq!(config.max_tokens, 512);
    }

    #[test]
    fn test_openrouter_requires_api_key() {
        let config = OpenRouterConfig::default();
        assert!(OpenRouterBackend::new(config).is_err());
    }

    #[test]
    fn test_openrouter_creation() {
        let config = OpenRouterConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        assert!(OpenRouterBackend::new(config).is_ok());
    }

    #[test]
    fn test_parse_valid_json() {
        let config = OpenRouterConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let backend = OpenRouterBackend::new(config).unwrap();
        let response = r#"{"cmd": "grep -r 'pattern' ."}"#;
        assert_eq!(
            backend.parse_command_response(response).unwrap(),
            "grep -r 'pattern' ."
        );
    }

    #[test]
    fn test_parse_embedded_json() {
        let config = OpenRouterConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let backend = OpenRouterBackend::new(config).unwrap();
        let response = r#"Here: {"cmd": "sort file.txt"} done"#;
        assert_eq!(
            backend.parse_command_response(response).unwrap(),
            "sort file.txt"
        );
    }

    #[test]
    fn test_parse_invalid_response() {
        let config = OpenRouterConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let backend = OpenRouterBackend::new(config).unwrap();
        assert!(backend.parse_command_response("no command here").is_err());
    }

    #[test]
    fn test_backend_info() {
        let config = OpenRouterConfig {
            api_key: "test-key".to_string(),
            model: "qwen/qwen3-coder".to_string(),
            ..Default::default()
        };
        let backend = OpenRouterBackend::new(config).unwrap();
        let info = backend.backend_info();
        assert_eq!(info.backend_type, BackendType::OpenRouter);
        assert_eq!(info.model_name, "qwen/qwen3-coder");
    }
}
