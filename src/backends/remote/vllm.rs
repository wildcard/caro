// vLLM server backend implementation

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};

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
7. If the request is unclear, generate "echo 'Please clarify your request'"

Request: {}
"#,
            request.shell, request.input
        )
    }

    /// Parse JSON response from vLLM
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

    /// Call vLLM API for inference
    async fn call_vllm_api(&self, prompt: &str) -> Result<String, GeneratorError> {
        let request = VllmRequest {
            model: self.model_name.clone(),
            messages: vec![VllmMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.1,
            max_tokens: 100,
            stream: false,
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
            Ok(choice.message.content.clone())
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
            .call_vllm_api(&self.create_system_prompt(request))
            .await
        {
            Ok(response) => {
                match self.parse_command_response(&response) {
                    Ok(command) => {
                        return Ok(GeneratedCommand {
                            command,
                            explanation: "Generated using vLLM server".to_string(),
                            safety_level: RiskLevel::Safe, // TODO: Implement safety validation
                            estimated_impact: "Remote inference operation".to_string(),
                            alternatives: vec![],
                            backend_used: format!("vLLM ({})", self.model_name),
                            generation_time_ms: 0, // Will be set by caller
                            confidence_score: 0.85,
                        });
                    }
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
