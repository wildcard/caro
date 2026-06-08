// Mesh-LLM distributed mesh backend implementation
//
// Mesh-LLM (https://github.com/Mesh-LLM/mesh-llm) is a peer-to-peer mesh that
// pools GPU and memory across multiple machines behind a single
// OpenAI-compatible API at http://localhost:9337/v1. Requests are routed to a
// node that can serve the requested model; large models can be split across
// peers ("skippy stage splits").
//
// Because the API is OpenAI-compatible, this backend mirrors the vLLM/Exo
// backends. The distinguishing feature is the `"model": "mesh"` request, which
// fans the query across every model in the mesh (a Mixture-of-Agents mode).
// For single-shot command generation we default to `"mesh"` so the mesh
// auto-routes to whatever model is loaded; users can pin a specific model via
// `[backends].mesh_url` + `--model-name`.

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

/// Regex pattern to extract command from malformed JSON with unescaped quotes
/// Handles cases like: {"cmd": "find . -type f -name "*.txt""}
static CMD_EXTRACT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{\s*"cmd"\s*:\s*"(.+)"\s*\}"#).expect("Invalid regex pattern"));

/// Default port for the Mesh-LLM local node API
pub const MESH_DEFAULT_PORT: u16 = 9337;

/// Default model: `"mesh"` triggers the mesh's auto-routing / MoA fan-out
pub const MESH_DEFAULT_MODEL: &str = "mesh";

/// Mesh-LLM API request format (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct MeshRequest {
    model: String,
    messages: Vec<MeshMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct MeshMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MeshResponse {
    choices: Vec<MeshChoice>,
}

#[derive(Debug, Deserialize)]
struct MeshChoice {
    message: MeshResponseMessage,
}

#[derive(Debug, Deserialize)]
struct MeshResponseMessage {
    content: String,
}

/// Mesh-LLM mesh backend for pooled distributed inference
pub struct MeshBackend {
    base_url: Url,
    model_name: String,
    client: Client,
    api_key: Option<String>,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

impl MeshBackend {
    /// Create a new Mesh-LLM backend pointed at `base_url`.
    pub fn new(base_url: Url, model_name: String) -> Result<Self, GeneratorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // mesh routing + split stages add latency
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

    /// Create a Mesh-LLM backend with the default local node URL and model.
    pub fn with_defaults() -> Result<Self, GeneratorError> {
        let url = Url::parse(&format!("http://localhost:{}", MESH_DEFAULT_PORT)).map_err(|e| {
            GeneratorError::ConfigError {
                message: format!("Failed to parse default URL: {}", e),
            }
        })?;
        Self::new(url, MESH_DEFAULT_MODEL.to_string())
    }

    /// Set the owner-control API key for a private mesh (if required).
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Add an embedded fallback backend used when the mesh is unreachable.
    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Build the command-generation system prompt (shared `{"cmd": ...}` contract).
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

    /// Parse the model response into a bare command string (4-tier strategy).
    fn parse_command_response(&self, response: &str) -> Result<String, GeneratorError> {
        // 1. Structured JSON
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                if !cmd.is_empty() {
                    return Ok(cmd.trim().to_string());
                }
            }
        }

        // 2. Embedded JSON substring
        if let (Some(start), Some(end)) = (response.find('{'), response.rfind('}')) {
            if start <= end {
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

        // 3. Line-scan for `cmd:` prefix
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

        // 4. Regex fallback for malformed JSON with unescaped quotes
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

    /// Call the mesh's OpenAI-compatible chat-completions endpoint.
    async fn call_mesh_api(&self, prompt: &str) -> Result<String, GeneratorError> {
        let request = MeshRequest {
            model: self.model_name.clone(),
            messages: vec![MeshMessage {
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
        if let Some(api_key) = &self.api_key {
            req_builder = req_builder.header(header::AUTHORIZATION, format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                GeneratorError::BackendUnavailable {
                    reason: format!("Mesh-LLM node unavailable: {}", e),
                }
            } else {
                GeneratorError::GenerationFailed {
                    details: format!("HTTP request failed: {}", e),
                }
            }
        })?;

        if response.status() == 401 || response.status() == 403 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "Authentication failed - invalid mesh API key".to_string(),
            });
        }

        if !response.status().is_success() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("Mesh-LLM API error: {}", response.status()),
            });
        }

        let mesh_response: MeshResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse Mesh-LLM response: {}", e),
                })?;

        if let Some(choice) = mesh_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(GeneratorError::ParseError {
                content: "Mesh-LLM response contained no choices".to_string(),
            })
        }
    }

    /// Attempt mesh inference, falling back to the embedded backend on failure.
    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        match self.call_mesh_api(&self.create_system_prompt(request)).await {
            Ok(response) => match self.parse_command_response(&response) {
                Ok(command) => {
                    return Ok(GeneratedCommand {
                        command,
                        explanation: "Generated using Mesh-LLM distributed mesh".to_string(),
                        safety_level: RiskLevel::Safe, // validated downstream
                        estimated_impact: "Pooled distributed inference operation".to_string(),
                        alternatives: vec![],
                        backend_used: format!("Mesh-LLM ({})", self.model_name),
                        generation_time_ms: 0, // set by caller
                        confidence_score: 0.85,
                    });
                }
                Err(parse_error) => {
                    tracing::warn!("Failed to parse Mesh-LLM response: {}", parse_error);
                }
            },
            Err(mesh_error) => {
                tracing::warn!("Mesh-LLM failed: {}", mesh_error);
                if let GeneratorError::BackendUnavailable { ref reason } = mesh_error {
                    if reason.contains("Authentication failed") {
                        return Err(mesh_error);
                    }
                }
            }
        }

        if let Some(fallback) = &self.embedded_fallback {
            tracing::info!("Falling back to embedded backend");
            let mut fallback_result = fallback.generate_command(request).await?;
            fallback_result.backend_used =
                format!("Embedded (Mesh-LLM fallback from {})", self.model_name);
            return Ok(fallback_result);
        }

        Err(GeneratorError::BackendUnavailable {
            reason: "Mesh-LLM node unavailable and no fallback configured".to_string(),
        })
    }
}

#[async_trait]
impl CommandGenerator for MeshBackend {
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
        // OpenAI-standard model listing endpoint doubles as a health probe.
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
            backend_type: BackendType::Mesh,
            model_name: self.model_name.clone(),
            supports_streaming: true,
            max_tokens: 100,
            typical_latency_ms: 2500, // routing + possible stage splits
            memory_usage_mb: 0,       // external mesh
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
    fn test_mesh_backend_creation() {
        let url = Url::parse("http://localhost:9337").unwrap();
        assert!(MeshBackend::new(url, "mesh".to_string()).is_ok());
    }

    #[test]
    fn test_mesh_with_defaults() {
        let backend = MeshBackend::with_defaults().unwrap();
        assert_eq!(backend.model_name(), MESH_DEFAULT_MODEL);
        assert!(backend.base_url().as_str().contains("9337"));
    }

    #[test]
    fn test_mesh_with_api_key() {
        let url = Url::parse("http://localhost:9337").unwrap();
        let backend = MeshBackend::new(url, "mesh".to_string())
            .unwrap()
            .with_api_key("owner-token".to_string());
        assert!(backend.api_key.is_some());
    }

    #[test]
    fn test_parse_valid_json() {
        let url = Url::parse("http://localhost:9337").unwrap();
        let backend = MeshBackend::new(url, "mesh".to_string()).unwrap();
        let response = r#"{"cmd": "grep -r 'pattern' ."}"#;
        assert_eq!(backend.parse_command_response(response).unwrap(), "grep -r 'pattern' .");
    }

    #[test]
    fn test_parse_embedded_json() {
        let url = Url::parse("http://localhost:9337").unwrap();
        let backend = MeshBackend::new(url, "mesh".to_string()).unwrap();
        let response = r#"Here you go: {"cmd": "sort file.txt"} done."#;
        assert_eq!(backend.parse_command_response(response).unwrap(), "sort file.txt");
    }

    #[test]
    fn test_parse_invalid_response() {
        let url = Url::parse("http://localhost:9337").unwrap();
        let backend = MeshBackend::new(url, "mesh".to_string()).unwrap();
        assert!(backend.parse_command_response("no command here").is_err());
    }

    #[test]
    fn test_backend_info() {
        let url = Url::parse("http://localhost:9337").unwrap();
        let backend = MeshBackend::new(url, "mesh".to_string()).unwrap();
        let info = backend.backend_info();
        assert_eq!(info.backend_type, BackendType::Mesh);
        assert_eq!(info.model_name, "mesh");
    }
}
