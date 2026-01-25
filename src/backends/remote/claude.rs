// Claude (Anthropic) API backend implementation
// Uses Claude Haiku 4.5 for fast, cost-effective command generation

use async_trait::async_trait;
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

/// Default Claude model - Haiku 4.5 for fast, efficient command generation
pub const DEFAULT_CLAUDE_MODEL: &str = "claude-haiku-4-5-20251101";

/// Claude API base URL
pub const CLAUDE_API_URL: &str = "https://api.anthropic.com";

/// Current Anthropic API version
pub const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Claude Messages API request format
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    temperature: f32,
}

/// Claude message format
#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API response format
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    stop_reason: Option<String>,
    #[allow(dead_code)]
    usage: ClaudeUsage,
}

/// Claude content block
#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

/// Claude usage statistics
#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    #[allow(dead_code)]
    input_tokens: u32,
    #[allow(dead_code)]
    output_tokens: u32,
}

/// Claude API error response
#[derive(Debug, Deserialize)]
struct ClaudeError {
    error: ClaudeErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ClaudeErrorDetail {
    message: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    error_type: String,
}

/// Claude backend for Anthropic's Claude API
///
/// Uses Claude Haiku 4.5 by default for fast, cost-effective command generation.
/// This backend is ideal when running within Claude Code or when direct API access
/// is available.
pub struct ClaudeBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    api_key: String,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

impl ClaudeBackend {
    /// Create a new Claude backend with API key
    pub fn new(api_key: String) -> Result<Self, GeneratorError> {
        Self::with_model(api_key, DEFAULT_CLAUDE_MODEL.to_string())
    }

    /// Create a new Claude backend with a specific model
    pub fn with_model(api_key: String, model_name: String) -> Result<Self, GeneratorError> {
        if api_key.is_empty() {
            return Err(GeneratorError::ConfigError {
                message: "Claude API key cannot be empty".to_string(),
            });
        }

        let base_url = Url::parse(CLAUDE_API_URL).map_err(|e| GeneratorError::ConfigError {
            message: format!("Invalid Claude API URL: {}", e),
        })?;

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
            api_key,
            embedded_fallback: None,
        })
    }

    /// Create a new Claude backend from environment variable
    pub fn from_env() -> Result<Self, GeneratorError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| GeneratorError::ConfigError {
            message: "ANTHROPIC_API_KEY environment variable not set".to_string(),
        })?;
        Self::new(api_key)
    }

    /// Add embedded fallback backend
    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }

    /// Set a custom base URL (for testing or proxy)
    pub fn with_base_url(mut self, base_url: Url) -> Self {
        self.base_url = base_url;
        self
    }

    /// Create system prompt for Claude
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

Do not include any text before or after the JSON object."#,
            request.shell
        )
    }

    /// Parse JSON response from Claude
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

        Err(GeneratorError::ParseError {
            content: response.to_string(),
        })
    }

    /// Call Claude API for inference
    async fn call_claude_api(&self, request: &CommandRequest) -> Result<String, GeneratorError> {
        let claude_request = ClaudeRequest {
            model: self.model_name.clone(),
            max_tokens: 100,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: request.input.clone(),
            }],
            system: Some(self.create_system_prompt(request)),
            temperature: 0.1,
        };

        let url = self.base_url.join("/v1/messages").map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Invalid base URL: {}", e),
            }
        })?;

        let response = self
            .client
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .json(&claude_request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    GeneratorError::BackendUnavailable {
                        reason: format!("Claude API unavailable: {}", e),
                    }
                } else {
                    GeneratorError::GenerationFailed {
                        details: format!("HTTP request failed: {}", e),
                    }
                }
            })?;

        // Check for authentication errors
        if response.status() == 401 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Authentication failed - invalid API key".to_string(),
            });
        }

        if response.status() == 403 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Access denied - check API key permissions".to_string(),
            });
        }

        if !response.status().is_success() {
            // Try to parse error response
            let status = response.status();
            if let Ok(error_response) = response.json::<ClaudeError>().await {
                return Err(GeneratorError::GenerationFailed {
                    details: format!("Claude API error ({}): {}", status, error_response.error.message),
                });
            }
            return Err(GeneratorError::GenerationFailed {
                details: format!("Claude API error: {}", status),
            });
        }

        let claude_response: ClaudeResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse Claude response: {}", e),
                })?;

        // Extract text from response content
        for content in claude_response.content {
            if content.content_type == "text" {
                if let Some(text) = content.text {
                    return Ok(text);
                }
            }
        }

        Err(GeneratorError::ParseError {
            content: "Claude response contained no text content".to_string(),
        })
    }

    /// Attempt inference with fallback to embedded backend
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Try Claude first
        match self.call_claude_api(request).await {
            Ok(response) => {
                match self.parse_command_response(&response) {
                    Ok(command) => {
                        return Ok(GeneratedCommand {
                            command,
                            explanation: format!("Generated using Claude ({})", self.model_name),
                            safety_level: RiskLevel::Safe, // TODO: Implement safety validation
                            estimated_impact: "Remote inference via Anthropic API".to_string(),
                            alternatives: vec![],
                            backend_used: format!("Claude ({})", self.model_name),
                            generation_time_ms: 0, // Will be set by caller
                            confidence_score: 0.95, // Claude typically has high confidence
                        });
                    }
                    Err(parse_error) => {
                        tracing::warn!("Failed to parse Claude response: {}", parse_error);
                        // Continue to fallback
                    }
                }
            }
            Err(claude_error) => {
                tracing::warn!("Claude backend failed: {}", claude_error);

                // For authentication errors, don't retry or fallback immediately
                if let GeneratorError::BackendUnavailable { ref reason } = claude_error {
                    if reason.contains("Authentication failed") || reason.contains("Access denied") {
                        return Err(claude_error);
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
                format!("Embedded (Claude fallback from {})", self.model_name);
            return Ok(fallback_result);
        }

        // No fallback available
        Err(GeneratorError::BackendUnavailable {
            reason: "Claude API unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for ClaudeBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = std::time::Instant::now();

        let mut result = self.generate_with_fallback(request).await?;
        result.generation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(result)
    }

    async fn is_available(&self) -> bool {
        // For Claude, we check if we have a valid API key format
        // Actual availability is checked on first request
        !self.api_key.is_empty() && self.api_key.starts_with("sk-ant-")
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Claude,
            model_name: self.model_name.clone(),
            supports_streaming: false, // Could be enabled in future
            max_tokens: 100,
            typical_latency_ms: 500, // Haiku is very fast
            memory_usage_mb: 0, // External API
            version: ANTHROPIC_API_VERSION.to_string(),
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
    fn test_claude_backend_creation() {
        let backend = ClaudeBackend::new("sk-ant-test-key".to_string());
        assert!(backend.is_ok());
    }

    #[test]
    fn test_claude_backend_empty_key() {
        let backend = ClaudeBackend::new("".to_string());
        assert!(backend.is_err());
    }

    #[test]
    fn test_claude_with_custom_model() {
        let backend = ClaudeBackend::with_model(
            "sk-ant-test-key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
        );
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert_eq!(backend.model_name, "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_parse_valid_json() {
        let backend = ClaudeBackend::new("sk-ant-test-key".to_string()).unwrap();

        let response = r#"{"cmd": "grep -r 'pattern' ."}"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "grep -r 'pattern' .");
    }

    #[test]
    fn test_parse_embedded_json() {
        let backend = ClaudeBackend::new("sk-ant-test-key".to_string()).unwrap();

        let response =
            r#"Here's the command: {"cmd": "sort file.txt"} Let me know if you need help."#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "sort file.txt");
    }

    #[test]
    fn test_parse_invalid_response() {
        let backend = ClaudeBackend::new("sk-ant-test-key".to_string()).unwrap();

        let response = "I can't generate a command for that request.";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_model() {
        assert_eq!(DEFAULT_CLAUDE_MODEL, "claude-haiku-4-5-20251101");
    }

    #[tokio::test]
    async fn test_is_available_valid_key() {
        let backend = ClaudeBackend::new("sk-ant-valid-key-format".to_string()).unwrap();
        assert!(backend.is_available().await);
    }

    #[tokio::test]
    async fn test_is_available_invalid_key() {
        let backend = ClaudeBackend::new("invalid-key-format".to_string()).unwrap();
        assert!(!backend.is_available().await);
    }
}
