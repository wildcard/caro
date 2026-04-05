// quant.cpp backend implementation
//
// quant.cpp (https://github.com/quantumaikr/quant.cpp) is a C-based LLM inference
// engine that achieves 7x longer context windows through KV cache compression with
// ~0% perplexity degradation. It provides an OpenAI-compatible API server.
//
// Features:
// - 4-bit uniform KV quantization (3.8x compression)
// - Delta encoding with 3-bit keys (8.5x compression)
// - QK-norm aware compression (3.5x compression)
// - GGUF model format (same as llama.cpp / caro embedded backend)
// - Cross-platform: Metal (Apple Silicon), AVX2 (x86), NEON (ARM)

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

/// Default port for quant.cpp HTTP server
pub const QUANTCPP_DEFAULT_PORT: u16 = 8080;

/// Default model name (user should configure based on their loaded model)
pub const QUANTCPP_DEFAULT_MODEL: &str = "default";

/// KV cache compression mode for quant.cpp
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KvCompressionMode {
    /// 4-bit uniform quantization (3.8x compression, ~0% perplexity loss)
    #[default]
    FourBit,
    /// Delta encoding with 3-bit keys (8.5x compression, ~1.3% perplexity impact)
    Delta,
    /// QK-norm aware compression (3.5x, for norm-sensitive architectures like Gemma)
    QkNorm,
}

impl std::fmt::Display for KvCompressionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FourBit => write!(f, "4-bit"),
            Self::Delta => write!(f, "delta"),
            Self::QkNorm => write!(f, "qk-norm"),
        }
    }
}

/// quant.cpp API request format (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct QuantCppRequest {
    model: String,
    messages: Vec<QuantCppMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

/// quant.cpp message format
#[derive(Debug, Serialize)]
struct QuantCppMessage {
    role: String,
    content: String,
}

/// quant.cpp API response format
#[derive(Debug, Deserialize)]
struct QuantCppResponse {
    choices: Vec<QuantCppChoice>,
    #[allow(dead_code)]
    usage: Option<QuantCppUsage>,
}

/// quant.cpp choice structure
#[derive(Debug, Deserialize)]
struct QuantCppChoice {
    message: QuantCppResponseMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

/// quant.cpp response message
#[derive(Debug, Deserialize)]
struct QuantCppResponseMessage {
    content: String,
    #[allow(dead_code)]
    role: String,
}

/// quant.cpp usage statistics
#[derive(Debug, Deserialize)]
struct QuantCppUsage {
    #[allow(dead_code)]
    prompt_tokens: u32,
    #[allow(dead_code)]
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

/// quant.cpp backend for inference with KV cache compression
pub struct QuantCppBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    api_key: Option<String>,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
    kv_compression: KvCompressionMode,
}

impl QuantCppBackend {
    /// Create a new quant.cpp backend
    ///
    /// Connects to quant.cpp server at the specified URL
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
            kv_compression: KvCompressionMode::default(),
        })
    }

    /// Create a new quant.cpp backend with default URL and model
    pub fn with_defaults() -> Result<Self, GeneratorError> {
        let url =
            Url::parse(&format!("http://localhost:{}", QUANTCPP_DEFAULT_PORT)).map_err(|e| {
                GeneratorError::ConfigError {
                    message: format!("Failed to parse default URL: {}", e),
                }
            })?;
        Self::new(url, QUANTCPP_DEFAULT_MODEL.to_string())
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

    /// Set KV cache compression mode
    pub fn with_kv_compression(mut self, mode: KvCompressionMode) -> Self {
        self.kv_compression = mode;
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

    /// Get the KV compression mode
    pub fn kv_compression(&self) -> KvCompressionMode {
        self.kv_compression
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

    /// Parse JSON response from quant.cpp
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

    /// Call quant.cpp API for inference
    async fn call_quantcpp_api(&self, prompt: &str) -> Result<String, GeneratorError> {
        let request = QuantCppRequest {
            model: self.model_name.clone(),
            messages: vec![QuantCppMessage {
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
                    reason: format!("quant.cpp server unavailable: {}", e),
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
                details: format!("quant.cpp API error: {}", response.status()),
            });
        }

        let quantcpp_response: QuantCppResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse quant.cpp response: {}", e),
                })?;

        if let Some(choice) = quantcpp_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(GeneratorError::ParseError {
                content: "quant.cpp response contained no choices".to_string(),
            })
        }
    }

    /// Attempt inference with fallback to embedded backend
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Try quant.cpp server first
        match self
            .call_quantcpp_api(&self.create_system_prompt(request))
            .await
        {
            Ok(response) => {
                match self.parse_command_response(&response) {
                    Ok(command) => {
                        return Ok(GeneratedCommand {
                            command,
                            explanation: format!(
                                "Generated using quant.cpp ({} KV compression)",
                                self.kv_compression
                            ),
                            safety_level: RiskLevel::Safe, // Safety validation done by caller
                            estimated_impact: "Local inference with KV cache compression"
                                .to_string(),
                            alternatives: vec![],
                            backend_used: format!(
                                "quant.cpp ({}, {})",
                                self.model_name, self.kv_compression
                            ),
                            generation_time_ms: 0, // Will be set by caller
                            confidence_score: 0.85,
                        });
                    }
                    Err(parse_error) => {
                        tracing::warn!("Failed to parse quant.cpp response: {}", parse_error);
                        // Continue to fallback
                    }
                }
            }
            Err(quantcpp_error) => {
                tracing::warn!("quant.cpp server failed: {}", quantcpp_error);

                // For authentication errors, don't fallback
                if let GeneratorError::BackendUnavailable { ref reason } = quantcpp_error {
                    if reason.contains("Authentication failed") {
                        return Err(quantcpp_error);
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
                format!("Embedded (quant.cpp fallback from {})", self.model_name);
            return Ok(fallback_result);
        }

        // No fallback available
        Err(GeneratorError::BackendUnavailable {
            reason: "quant.cpp server unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for QuantCppBackend {
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
        // Check if quant.cpp server is responding via models endpoint
        let health_url = match self.base_url.join("/v1/models") {
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
            backend_type: BackendType::QuantCpp,
            model_name: self.model_name.clone(),
            supports_streaming: true, // quant.cpp supports SSE streaming
            max_tokens: 100,
            typical_latency_ms: 1500, // Slightly faster than vLLM due to KV optimization
            memory_usage_mb: 0,       // External server
            version: "0.5".to_string(),
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
    fn test_quantcpp_backend_creation() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = QuantCppBackend::new(url, "qwen2.5-coder-1.5b".to_string());
        assert!(backend.is_ok());
    }

    #[test]
    fn test_quantcpp_with_defaults() {
        let backend = QuantCppBackend::with_defaults();
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert_eq!(backend.model_name(), QUANTCPP_DEFAULT_MODEL);
        assert!(backend.base_url().as_str().contains("8080"));
    }

    #[test]
    fn test_quantcpp_with_api_key() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = QuantCppBackend::new(url, "test".to_string())
            .unwrap()
            .with_api_key("test-key".to_string());
        assert!(backend.api_key.is_some());
    }

    #[test]
    fn test_quantcpp_with_kv_compression() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = QuantCppBackend::new(url, "test".to_string())
            .unwrap()
            .with_kv_compression(KvCompressionMode::Delta);
        assert_eq!(backend.kv_compression(), KvCompressionMode::Delta);
    }

    #[test]
    fn test_default_kv_compression() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = QuantCppBackend::new(url, "test".to_string()).unwrap();
        assert_eq!(backend.kv_compression(), KvCompressionMode::FourBit);
    }

    #[test]
    fn test_parse_valid_json() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = QuantCppBackend::new(url, "test".to_string()).unwrap();

        let response = r#"{"cmd": "grep -r 'pattern' ."}"#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "grep -r 'pattern' .");
    }

    #[test]
    fn test_parse_embedded_json() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = QuantCppBackend::new(url, "test".to_string()).unwrap();

        let response =
            r#"Sure! Here's the command: {"cmd": "sort file.txt"} Let me know if you need help."#;
        let result = backend.parse_command_response(response);
        assert_eq!(result.unwrap(), "sort file.txt");
    }

    #[test]
    fn test_parse_invalid_response() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = QuantCppBackend::new(url, "test".to_string()).unwrap();

        let response = "I can't generate a command for that request.";
        let result = backend.parse_command_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_info() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let backend = QuantCppBackend::new(url, "qwen2.5-coder-1.5b".to_string()).unwrap();

        let info = backend.backend_info();
        assert_eq!(info.backend_type, BackendType::QuantCpp);
        assert_eq!(info.model_name, "qwen2.5-coder-1.5b");
        assert!(info.supports_streaming);
        assert_eq!(info.typical_latency_ms, 1500);
    }

    #[test]
    fn test_kv_compression_display() {
        assert_eq!(format!("{}", KvCompressionMode::FourBit), "4-bit");
        assert_eq!(format!("{}", KvCompressionMode::Delta), "delta");
        assert_eq!(format!("{}", KvCompressionMode::QkNorm), "qk-norm");
    }
}
