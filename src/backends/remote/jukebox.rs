// vLLM Jukebox multi-model server backend implementation
//
// vLLM Jukebox is an OpenAI-compatible HTTP server that supports automatic
// model swapping. This backend takes advantage of its multi-model capabilities.
// See: https://github.com/erans/vllm-jukebox

use async_trait::async_trait;
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel};

/// OpenAI-compatible chat completion request
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

/// Chat message format
#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible chat completion response
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    #[allow(dead_code)]
    usage: Option<UsageInfo>,
    #[allow(dead_code)]
    model: Option<String>,
}

/// Chat choice structure
#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ResponseMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

/// Response message content
#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
    #[allow(dead_code)]
    role: String,
}

/// Token usage statistics
#[derive(Debug, Deserialize)]
struct UsageInfo {
    #[allow(dead_code)]
    prompt_tokens: u32,
    #[allow(dead_code)]
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

/// Jukebox status response
#[derive(Debug, Deserialize)]
pub struct JukeboxStatus {
    /// Current mode (swap or scheduler)
    pub mode: Option<String>,
    /// Currently loaded model (in swap mode)
    pub current_model: Option<String>,
    /// Number of running instances (in scheduler mode)
    pub running_instances: Option<u32>,
}

/// Model information from /v1/models endpoint
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

/// Individual model info
#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
    #[allow(dead_code)]
    object: Option<String>,
}

/// vLLM Jukebox backend for multi-model inference server
///
/// This backend connects to a vLLM Jukebox server which can automatically
/// swap between multiple models based on the request. It supports both
/// "swap mode" (single instance, model swapping) and "scheduler mode"
/// (multiple concurrent instances with LRU eviction).
pub struct JukeboxBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    api_key: Option<String>,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
    /// Request timeout in seconds
    request_timeout: Duration,
}

impl JukeboxBackend {
    /// Create a new vLLM Jukebox backend
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the Jukebox server (e.g., http://localhost:8080)
    /// * `model_name` - Name of the model to use for inference
    ///
    /// # Example
    /// ```no_run
    /// use reqwest::Url;
    /// use caro::backends::remote::JukeboxBackend;
    ///
    /// let backend = JukeboxBackend::new(
    ///     Url::parse("http://localhost:8080").unwrap(),
    ///     "qwen".to_string()
    /// ).unwrap();
    /// ```
    pub fn new(base_url: Url, model_name: String) -> Result<Self, GeneratorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Longer timeout for model swapping
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
            request_timeout: Duration::from_secs(60),
        })
    }

    /// Set API key for authentication
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Add embedded fallback backend for when Jukebox is unavailable
    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }

    /// Set custom request timeout (useful for large models that take time to swap)
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        // Recreate client with new timeout
        if let Ok(client) = Client::builder().timeout(timeout).build() {
            self.client = client;
        }
        self
    }

    /// Get current server status
    ///
    /// Returns information about the server mode and currently loaded model
    pub async fn get_status(&self) -> Result<JukeboxStatus, GeneratorError> {
        let url = self.base_url.join("/status").map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Invalid base URL: {}", e),
            }
        })?;

        let response = self.client.get(url).send().await.map_err(|e| {
            GeneratorError::BackendUnavailable {
                reason: format!("Failed to connect to Jukebox server: {}", e),
            }
        })?;

        if !response.status().is_success() {
            return Err(GeneratorError::BackendUnavailable {
                reason: format!("Status endpoint returned: {}", response.status()),
            });
        }

        response.json().await.map_err(|e| GeneratorError::ParseError {
            content: format!("Failed to parse status response: {}", e),
        })
    }

    /// List available models from the server
    pub async fn list_models(&self) -> Result<Vec<String>, GeneratorError> {
        let url = self.base_url.join("/v1/models").map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Invalid base URL: {}", e),
            }
        })?;

        let response = self.client.get(url).send().await.map_err(|e| {
            GeneratorError::BackendUnavailable {
                reason: format!("Failed to connect to Jukebox server: {}", e),
            }
        })?;

        if !response.status().is_success() {
            return Err(GeneratorError::BackendUnavailable {
                reason: format!("Models endpoint returned: {}", response.status()),
            });
        }

        let models_response: ModelsResponse =
            response.json().await.map_err(|e| GeneratorError::ParseError {
                content: format!("Failed to parse models response: {}", e),
            })?;

        Ok(models_response.data.into_iter().map(|m| m.id).collect())
    }

    /// Create system prompt for command generation
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

    /// Parse JSON response to extract command
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

    /// Call Jukebox API for chat completion
    async fn call_api(&self, prompt: &str) -> Result<String, GeneratorError> {
        let request = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages: vec![ChatMessage {
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

        // Add authentication if configured
        if let Some(api_key) = &self.api_key {
            req_builder = req_builder.header(header::AUTHORIZATION, format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                GeneratorError::BackendUnavailable {
                    reason: format!("Jukebox server unavailable: {}", e),
                }
            } else {
                GeneratorError::GenerationFailed {
                    details: format!("HTTP request failed: {}", e),
                }
            }
        })?;

        // Handle authentication errors
        if response.status() == 401 || response.status() == 403 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Authentication failed - invalid API key".to_string(),
            });
        }

        // Handle model not found (could happen if model isn't configured in Jukebox)
        if response.status() == 404 {
            return Err(GeneratorError::ConfigError {
                message: format!("Model '{}' not found on Jukebox server", self.model_name),
            });
        }

        // Handle service unavailable (model might be swapping)
        if response.status() == 503 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Jukebox server is busy (possibly swapping models)".to_string(),
            });
        }

        if !response.status().is_success() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("Jukebox API error: {}", response.status()),
            });
        }

        let completion_response: ChatCompletionResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse Jukebox response: {}", e),
                })?;

        if let Some(choice) = completion_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(GeneratorError::ParseError {
                content: "Jukebox response contained no choices".to_string(),
            })
        }
    }

    /// Generate command with automatic fallback
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Try Jukebox first
        match self.call_api(&self.create_system_prompt(request)).await {
            Ok(response) => match self.parse_command_response(&response) {
                Ok(command) => {
                    return Ok(GeneratedCommand {
                        command,
                        explanation: "Generated using vLLM Jukebox server".to_string(),
                        safety_level: RiskLevel::Safe, // TODO: Implement safety validation
                        estimated_impact: "Remote inference operation".to_string(),
                        alternatives: vec![],
                        backend_used: format!("vLLM Jukebox ({})", self.model_name),
                        generation_time_ms: 0, // Set by caller
                        confidence_score: 0.85,
                    });
                }
                Err(parse_error) => {
                    tracing::warn!("Failed to parse Jukebox response: {}", parse_error);
                    // Continue to fallback
                }
            },
            Err(jukebox_error) => {
                tracing::warn!("Jukebox backend failed: {}", jukebox_error);

                // For authentication errors, don't fallback
                if let GeneratorError::BackendUnavailable { ref reason } = jukebox_error {
                    if reason.contains("Authentication failed") {
                        return Err(jukebox_error);
                    }
                }
                // For config errors (model not found), don't fallback
                if matches!(jukebox_error, GeneratorError::ConfigError { .. }) {
                    return Err(jukebox_error);
                }
                // Continue to fallback for other errors
            }
        }

        // Fallback to embedded backend if available
        if let Some(fallback) = &self.embedded_fallback {
            tracing::info!("Falling back to embedded backend");
            let mut fallback_result = fallback.generate_command(request).await?;
            fallback_result.backend_used =
                format!("Embedded (Jukebox fallback from {})", self.model_name);
            return Ok(fallback_result);
        }

        // No fallback available
        Err(GeneratorError::BackendUnavailable {
            reason: "Jukebox server unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for JukeboxBackend {
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
        // Check if Jukebox server is responding via health endpoint
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
            backend_type: BackendType::VllmJukebox,
            model_name: self.model_name.clone(),
            supports_streaming: true, // Jukebox supports streaming
            max_tokens: 100,
            typical_latency_ms: 5000, // Slightly higher due to potential model swapping
            memory_usage_mb: 0,       // External server
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
    fn test_jukebox_backend_creation() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = JukeboxBackend::new(url, "qwen".to_string());
        assert!(backend.is_ok());
    }

    #[test]
    fn test_jukebox_with_api_key() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = JukeboxBackend::new(url, "test-model".to_string())
            .unwrap()
            .with_api_key("secret-key".to_string());
        assert!(backend.api_key.is_some());
    }

    #[test]
    fn test_jukebox_with_timeout() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = JukeboxBackend::new(url, "test-model".to_string())
            .unwrap()
            .with_timeout(Duration::from_secs(120));
        assert_eq!(backend.request_timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_parse_valid_json() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = JukeboxBackend::new(url, "test".to_string()).unwrap();

        let response = r#"{"cmd": "find . -name '*.rs'"}"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "find . -name '*.rs'");
    }

    #[test]
    fn test_parse_embedded_json() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = JukeboxBackend::new(url, "test".to_string()).unwrap();

        let response = r#"Here's the command: {"cmd": "ls -la"} Hope that helps!"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "ls -la");
    }

    #[test]
    fn test_parse_invalid_response() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = JukeboxBackend::new(url, "test".to_string()).unwrap();

        let response = "I cannot generate that command.";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_info() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = JukeboxBackend::new(url, "qwen".to_string()).unwrap();
        let info = backend.backend_info();

        assert_eq!(info.backend_type, BackendType::VllmJukebox);
        assert_eq!(info.model_name, "qwen");
        assert!(info.supports_streaming);
    }
}
