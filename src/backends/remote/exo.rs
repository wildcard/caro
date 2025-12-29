// Exo distributed cluster backend implementation
//
// Exo (https://github.com/exo-explore/exo) is a distributed AI inference framework
// that allows running LLMs across multiple devices. It provides an OpenAI-compatible
// API at http://localhost:52415/v1/chat/completions.
//
// Features:
// - Automatic peer discovery on local networks
// - RDMA over Thunderbolt for high-speed inter-device communication
// - Distributed model execution across multiple Apple Silicon devices
// - Support for large models like llama-3.1-405b across device clusters

use async_trait::async_trait;
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

/// Default port for exo cluster API
pub const EXO_DEFAULT_PORT: u16 = 52415;

/// Default model for exo cluster
pub const EXO_DEFAULT_MODEL: &str = "llama-3.2-3b";

/// Exo API request format (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct ExoRequest {
    model: String,
    messages: Vec<ExoMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

/// Exo message format
#[derive(Debug, Serialize)]
struct ExoMessage {
    role: String,
    content: String,
}

/// Exo API response format
#[derive(Debug, Deserialize)]
struct ExoResponse {
    choices: Vec<ExoChoice>,
    #[allow(dead_code)]
    usage: Option<ExoUsage>,
    #[allow(dead_code)]
    model: Option<String>,
}

/// Exo choice structure
#[derive(Debug, Deserialize)]
struct ExoChoice {
    message: ExoResponseMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

/// Exo response message
#[derive(Debug, Deserialize)]
struct ExoResponseMessage {
    content: String,
    #[allow(dead_code)]
    role: String,
}

/// Exo usage statistics
#[derive(Debug, Deserialize)]
struct ExoUsage {
    #[allow(dead_code)]
    prompt_tokens: u32,
    #[allow(dead_code)]
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

/// Exo model pool response
#[derive(Debug, Deserialize)]
pub struct ExoModelPool {
    pub models: Vec<ExoModelInfo>,
}

/// Exo model information
#[derive(Debug, Clone, Deserialize)]
pub struct ExoModelInfo {
    pub id: String,
    #[serde(default)]
    pub object: String,
    #[serde(default)]
    pub owned_by: String,
}

/// Exo cluster backend for distributed inference
pub struct ExoBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    api_key: Option<String>,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

impl ExoBackend {
    /// Create a new Exo backend with default settings
    ///
    /// Connects to exo cluster at http://localhost:52415 by default
    pub fn new(base_url: Url, model_name: String) -> Result<Self, GeneratorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Longer timeout for distributed inference
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

    /// Create a new Exo backend with default URL and model
    pub fn with_defaults() -> Result<Self, GeneratorError> {
        let url = Url::parse(&format!("http://localhost:{}", EXO_DEFAULT_PORT)).map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Failed to parse default URL: {}", e),
            }
        })?;
        Self::new(url, EXO_DEFAULT_MODEL.to_string())
    }

    /// Set API key for authentication (if required)
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Add embedded fallback backend
    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }

    /// Get the base URL
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Get the model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// List available models in the exo cluster
    pub async fn list_models(&self) -> Result<Vec<ExoModelInfo>, GeneratorError> {
        let url = self.base_url.join("/v1/models").map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Invalid base URL: {}", e),
            }
        })?;

        let response = self.client.get(url).send().await.map_err(|e| {
            GeneratorError::BackendUnavailable {
                reason: format!("Failed to list models: {}", e),
            }
        })?;

        if !response.status().is_success() {
            return Err(GeneratorError::BackendUnavailable {
                reason: format!("Failed to list models: HTTP {}", response.status()),
            });
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ExoModelInfo>,
        }

        let models_response: ModelsResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse models response: {}", e),
                })?;

        Ok(models_response.data)
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

    /// Parse JSON response from exo
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

    /// Call exo API for inference
    async fn call_exo_api(&self, prompt: &str) -> Result<String, GeneratorError> {
        let request = ExoRequest {
            model: self.model_name.clone(),
            messages: vec![ExoMessage {
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
                    reason: format!("Exo cluster unavailable: {}", e),
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
                details: format!("Exo API error: {}", response.status()),
            });
        }

        let exo_response: ExoResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse Exo response: {}", e),
                })?;

        if let Some(choice) = exo_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(GeneratorError::ParseError {
                content: "Exo response contained no choices".to_string(),
            })
        }
    }

    /// Attempt inference with fallback to embedded backend
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Try exo cluster first
        match self
            .call_exo_api(&self.create_system_prompt(request))
            .await
        {
            Ok(response) => {
                match self.parse_command_response(&response) {
                    Ok(command) => {
                        return Ok(GeneratedCommand {
                            command,
                            explanation: "Generated using Exo distributed cluster".to_string(),
                            safety_level: RiskLevel::Safe, // TODO: Implement safety validation
                            estimated_impact: "Distributed inference operation".to_string(),
                            alternatives: vec![],
                            backend_used: format!("Exo ({})", self.model_name),
                            generation_time_ms: 0, // Will be set by caller
                            confidence_score: 0.85,
                        });
                    }
                    Err(parse_error) => {
                        tracing::warn!("Failed to parse Exo response: {}", parse_error);
                        // Continue to fallback
                    }
                }
            }
            Err(exo_error) => {
                tracing::warn!("Exo cluster failed: {}", exo_error);

                // For authentication errors, don't retry or fallback immediately
                if let GeneratorError::BackendUnavailable { ref reason } = exo_error {
                    if reason.contains("Authentication failed") {
                        return Err(exo_error);
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
                format!("Embedded (Exo fallback from {})", self.model_name);
            return Ok(fallback_result);
        }

        // No fallback available
        Err(GeneratorError::BackendUnavailable {
            reason: "Exo cluster unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for ExoBackend {
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
        // Check if exo cluster is responding via health endpoint
        let health_url = match self.base_url.join("/healthcheck") {
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
            backend_type: BackendType::Exo,
            model_name: self.model_name.clone(),
            supports_streaming: true, // Exo supports streaming
            max_tokens: 100,
            typical_latency_ms: 2000, // Distributed inference can be fast with RDMA
            memory_usage_mb: 0,       // External cluster
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
    fn test_exo_backend_creation() {
        let url = Url::parse("http://localhost:52415").unwrap();
        let backend = ExoBackend::new(url, "llama-3.2-3b".to_string());
        assert!(backend.is_ok());
    }

    #[test]
    fn test_exo_with_defaults() {
        let backend = ExoBackend::with_defaults();
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert_eq!(backend.model_name(), EXO_DEFAULT_MODEL);
        assert!(backend.base_url().as_str().contains("52415"));
    }

    #[test]
    fn test_exo_with_api_key() {
        let url = Url::parse("http://localhost:52415").unwrap();
        let backend = ExoBackend::new(url, "test".to_string())
            .unwrap()
            .with_api_key("test-key".to_string());
        assert!(backend.api_key.is_some());
    }

    #[test]
    fn test_parse_valid_json() {
        let url = Url::parse("http://localhost:52415").unwrap();
        let backend = ExoBackend::new(url, "test".to_string()).unwrap();

        let response = r#"{"cmd": "grep -r 'pattern' ."}"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "grep -r 'pattern' .");
    }

    #[test]
    fn test_parse_embedded_json() {
        let url = Url::parse("http://localhost:52415").unwrap();
        let backend = ExoBackend::new(url, "test".to_string()).unwrap();

        let response =
            r#"Sure! Here's the command: {"cmd": "sort file.txt"} Let me know if you need help."#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "sort file.txt");
    }

    #[test]
    fn test_parse_invalid_response() {
        let url = Url::parse("http://localhost:52415").unwrap();
        let backend = ExoBackend::new(url, "test".to_string()).unwrap();

        let response = "I can't generate a command for that request.";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_info() {
        let url = Url::parse("http://localhost:52415").unwrap();
        let backend = ExoBackend::new(url, "llama-3.2-3b".to_string()).unwrap();

        let info = backend.backend_info();
        assert_eq!(info.backend_type, BackendType::Exo);
        assert_eq!(info.model_name, "llama-3.2-3b");
        assert!(info.supports_streaming);
    }
}
