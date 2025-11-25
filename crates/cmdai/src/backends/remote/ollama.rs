// Ollama local server backend implementation

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

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
    done: bool,
    #[allow(dead_code)]
    context: Option<Vec<i32>>,
    #[allow(dead_code)]
    created_at: Option<String>,
}

/// Ollama backend for local Ollama server
pub struct OllamaBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
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

    /// Attempt inference with fallback to embedded backend
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
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

    #[test]
    fn test_ollama_backend_creation() {
        let url = Url::parse("http://localhost:11434").unwrap();
        let backend = OllamaBackend::new(url, "codellama:7b".to_string());
        assert!(backend.is_ok());
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
