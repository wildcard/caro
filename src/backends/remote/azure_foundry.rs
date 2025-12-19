// Azure Foundry backend implementation

use async_trait::async_trait;
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

/// Azure Foundry API request format (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct AzureFoundryRequest {
    model: String,
    messages: Vec<AzureFoundryMessage>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

/// Azure Foundry message format
#[derive(Debug, Serialize)]
struct AzureFoundryMessage {
    role: String,
    content: String,
}

/// Response format specification for JSON mode
#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

/// Azure Foundry API response format
#[derive(Debug, Deserialize)]
struct AzureFoundryResponse {
    choices: Vec<AzureFoundryChoice>,
    #[allow(dead_code)]
    usage: Option<AzureFoundryUsage>,
    #[allow(dead_code)]
    id: Option<String>,
    #[allow(dead_code)]
    model: Option<String>,
}

/// Azure Foundry choice structure
#[derive(Debug, Deserialize)]
struct AzureFoundryChoice {
    message: AzureFoundryResponseMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
    #[allow(dead_code)]
    index: Option<u32>,
}

/// Azure Foundry response message
#[derive(Debug, Deserialize)]
struct AzureFoundryResponseMessage {
    content: String,
    #[allow(dead_code)]
    role: String,
}

/// Azure Foundry usage statistics
#[derive(Debug, Deserialize)]
struct AzureFoundryUsage {
    #[allow(dead_code)]
    prompt_tokens: u32,
    #[allow(dead_code)]
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

/// Azure Foundry backend for Azure AI services
pub struct AzureFoundryBackend {
    endpoint: Url,
    model_name: String,
    client: Client,
    api_key: String,
    api_version: String,
    timeout: Duration,
    max_retries: u32,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

impl AzureFoundryBackend {
    /// Create a new Azure Foundry backend
    pub fn new(
        endpoint: Url,
        model_name: String,
        api_key: String,
    ) -> Result<Self, GeneratorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| GeneratorError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            endpoint,
            model_name,
            client,
            api_key,
            api_version: "2024-05-01-preview".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            embedded_fallback: None,
        })
    }

    /// Set API version for Azure Foundry
    pub fn with_api_version(mut self, api_version: String) -> Self {
        self.api_version = api_version;
        self
    }

    /// Set timeout for API requests
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum retry attempts
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Add embedded fallback backend
    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }

    /// Create system prompt for Azure Foundry
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

    /// Parse JSON response from Azure Foundry
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

    /// Call Azure Foundry API for inference
    async fn call_azure_foundry_api(&self, prompt: &str) -> Result<String, GeneratorError> {
        let request = AzureFoundryRequest {
            model: self.model_name.clone(),
            messages: vec![AzureFoundryMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.1,
            max_tokens: 100,
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
        };

        // Construct the Azure Foundry endpoint URL
        // Format: https://<endpoint>/v1/chat/completions?api-version=<version>
        let mut url = self
            .endpoint
            .join("/v1/chat/completions")
            .map_err(|e| GeneratorError::ConfigError {
                message: format!("Invalid endpoint URL: {}", e),
            })?;

        url.query_pairs_mut()
            .append_pair("api-version", &self.api_version);

        let req_builder = self
            .client
            .post(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request);

        let response = req_builder.send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                GeneratorError::BackendUnavailable {
                    reason: format!("Azure Foundry unavailable: {}", e),
                }
            } else {
                GeneratorError::GenerationFailed {
                    details: format!("HTTP request failed: {}", e),
                }
            }
        })?;

        // Check for authentication errors
        let status = response.status();
        if status == 401 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Authentication failed - invalid API key".to_string(),
            });
        } else if status == 403 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Access forbidden - check your permissions".to_string(),
            });
        } else if status == 404 {
            return Err(GeneratorError::ConfigError {
                message: "Endpoint not found - check your Azure Foundry endpoint URL".to_string(),
            });
        } else if status == 429 {
            return Err(GeneratorError::GenerationFailed {
                details: "Rate limit exceeded - please try again later".to_string(),
            });
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(GeneratorError::GenerationFailed {
                details: format!("Azure Foundry API error ({}): {}", status, error_text),
            });
        }

        let azure_response: AzureFoundryResponse = response.json().await.map_err(|e| {
            GeneratorError::ParseError {
                content: format!("Failed to parse Azure Foundry response: {}", e),
            }
        })?;

        if let Some(choice) = azure_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(GeneratorError::ParseError {
                content: "Azure Foundry response contained no choices".to_string(),
            })
        }
    }

    /// Attempt inference with fallback to embedded backend
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Try Azure Foundry first
        match self
            .call_azure_foundry_api(&self.create_system_prompt(request))
            .await
        {
            Ok(response) => {
                match self.parse_command_response(&response) {
                    Ok(command) => {
                        return Ok(GeneratedCommand {
                            command,
                            explanation: "Generated using Azure Foundry".to_string(),
                            safety_level: RiskLevel::Safe, // TODO: Implement safety validation
                            estimated_impact: "Remote inference operation".to_string(),
                            alternatives: vec![],
                            backend_used: format!("Azure Foundry ({})", self.model_name),
                            generation_time_ms: 0, // Will be set by caller
                            confidence_score: 0.85,
                        });
                    }
                    Err(parse_error) => {
                        tracing::warn!("Failed to parse Azure Foundry response: {}", parse_error);
                        // Continue to fallback
                    }
                }
            }
            Err(azure_error) => {
                tracing::warn!("Azure Foundry backend failed: {}", azure_error);

                // For authentication/config errors, don't retry or fallback immediately
                match azure_error {
                    GeneratorError::BackendUnavailable { ref reason }
                        if reason.contains("Authentication failed")
                            || reason.contains("Access forbidden") =>
                    {
                        return Err(azure_error);
                    }
                    GeneratorError::ConfigError { .. } => {
                        return Err(azure_error);
                    }
                    _ => {
                        // Continue to fallback for other errors
                    }
                }
            }
        }

        // Fallback to embedded backend if available
        if let Some(fallback) = &self.embedded_fallback {
            tracing::info!("Falling back to embedded backend");
            let mut fallback_result = fallback.generate_command(request).await?;
            fallback_result.backend_used = format!(
                "Embedded (Azure Foundry fallback from {})",
                self.model_name
            );
            return Ok(fallback_result);
        }

        // No fallback available
        Err(GeneratorError::BackendUnavailable {
            reason: "Azure Foundry unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for AzureFoundryBackend {
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
        // Check if Azure Foundry is responding
        // We'll try a simple health check by calling the endpoint
        let health_url = match self.endpoint.join("/health") {
            Ok(url) => url,
            Err(_) => return false,
        };

        match self
            .client
            .get(health_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::AzureFoundry,
            model_name: self.model_name.clone(),
            supports_streaming: false,
            max_tokens: 100,
            typical_latency_ms: 3000,
            memory_usage_mb: 0, // External service
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
    fn test_azure_foundry_backend_creation() {
        let url = Url::parse("https://my-foundry.eastus.inference.ml.azure.com").unwrap();
        let backend = AzureFoundryBackend::new(url, "gpt-4o".to_string(), "test-key".to_string());
        assert!(backend.is_ok());
    }

    #[test]
    fn test_azure_foundry_with_api_version() {
        let url = Url::parse("https://my-foundry.eastus.inference.ml.azure.com").unwrap();
        let backend = AzureFoundryBackend::new(url, "gpt-4o".to_string(), "test-key".to_string())
            .unwrap()
            .with_api_version("2024-06-01".to_string());
        assert_eq!(backend.api_version, "2024-06-01");
    }

    #[test]
    fn test_parse_valid_json() {
        let url = Url::parse("https://my-foundry.eastus.inference.ml.azure.com").unwrap();
        let backend = AzureFoundryBackend::new(url, "gpt-4o".to_string(), "test-key".to_string())
            .unwrap();

        let response = r#"{"cmd": "find . -name '*.rs'"}"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "find . -name '*.rs'");
    }

    #[test]
    fn test_parse_embedded_json() {
        let url = Url::parse("https://my-foundry.eastus.inference.ml.azure.com").unwrap();
        let backend = AzureFoundryBackend::new(url, "gpt-4o".to_string(), "test-key".to_string())
            .unwrap();

        let response =
            r#"Here's your command: {"cmd": "ls -la"} Hope this helps!"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "ls -la");
    }

    #[test]
    fn test_parse_invalid_response() {
        let url = Url::parse("https://my-foundry.eastus.inference.ml.azure.com").unwrap();
        let backend = AzureFoundryBackend::new(url, "gpt-4o".to_string(), "test-key".to_string())
            .unwrap();

        let response = "I cannot generate a command for that.";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_timeout_configuration() {
        let url = Url::parse("https://my-foundry.eastus.inference.ml.azure.com").unwrap();
        let backend = AzureFoundryBackend::new(url, "gpt-4o".to_string(), "test-key".to_string())
            .unwrap()
            .with_timeout(Duration::from_secs(60));
        assert_eq!(backend.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_max_retries_configuration() {
        let url = Url::parse("https://my-foundry.eastus.inference.ml.azure.com").unwrap();
        let backend = AzureFoundryBackend::new(url, "gpt-4o".to_string(), "test-key".to_string())
            .unwrap()
            .with_max_retries(5);
        assert_eq!(backend.max_retries, 5);
    }
}
