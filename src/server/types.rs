//! API request and response types for caro-server.
//!
//! These types define the JSON API contract between caro-server and its clients
//! (cabinet agents, TypeScript clients, etc.).

use crate::models::{RiskLevel, SafetyLevel, ShellType};
use serde::{Deserialize, Serialize};

/// Request to generate a shell command from natural language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCommandRequest {
    /// Natural language description of desired command
    pub input: String,

    /// Target shell type (default: bash)
    #[serde(default = "default_shell")]
    pub shell: ShellType,

    /// Safety validation level (default: moderate)
    #[serde(default)]
    pub safety_level: SafetyLevel,

    /// Working directory or additional context
    #[serde(default)]
    pub context: Option<String>,

    /// Client-provided request ID for correlation
    #[serde(default = "generate_request_id")]
    pub request_id: String,

    /// Cabinet agent identifier (for logging/audit)
    #[serde(default)]
    pub agent_id: Option<String>,
}

/// Response from command generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCommandResponse {
    /// Correlation ID matching the request
    pub request_id: String,

    /// Status: "ok", "blocked", or "error"
    pub status: ApiStatus,

    /// The generated shell command (present when status is ok or blocked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Human-readable explanation of what the command does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,

    /// Assessed risk level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,

    /// Description of estimated impact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_impact: Option<String>,

    /// Alternative commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<String>>,

    /// Name of the backend that generated this command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_used: Option<String>,

    /// Time taken to generate the command in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_time_ms: Option<u64>,

    /// Confidence score (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<f64>,

    /// Safety warnings
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,

    /// Error message (present when status is error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Reason for blocking (present when status is blocked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Request to execute a previously generated command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiExecuteRequest {
    /// The command to execute
    pub command: String,

    /// Explicit safety confirmation (required)
    pub confirmed: bool,

    /// Correlation ID
    #[serde(default = "generate_request_id")]
    pub request_id: String,

    /// Execution timeout in milliseconds (default: 30000)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

/// Response from command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiExecuteResponse {
    pub request_id: String,
    pub status: ApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiHealthResponse {
    pub status: String,
    pub version: String,
    pub backends: BackendStatus,
    pub safety_patterns: usize,
    pub uptime_seconds: u64,
}

/// Backend availability status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    pub static_matcher: bool,
    pub embedded: bool,
    pub ollama: bool,
    pub claude: bool,
}

/// API response status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
    Ok,
    Blocked,
    Error,
}

/// Error response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub status: ApiStatus,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

fn default_shell() -> ShellType {
    ShellType::Bash
}

fn default_timeout() -> u64 {
    30_000
}

fn generate_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_request_deserialize_minimal() {
        let json = r#"{"input": "list files"}"#;
        let req: ApiCommandRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.input, "list files");
        assert_eq!(req.shell, ShellType::Bash);
        assert_eq!(req.safety_level, SafetyLevel::Moderate);
        assert!(req.context.is_none());
        assert!(!req.request_id.is_empty());
    }

    #[test]
    fn test_command_request_deserialize_full() {
        let json = r#"{
            "input": "find large files",
            "shell": "zsh",
            "safety_level": "strict",
            "context": "/home/user",
            "request_id": "req-123",
            "agent_id": "devops"
        }"#;
        let req: ApiCommandRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.input, "find large files");
        assert_eq!(req.shell, ShellType::Zsh);
        assert_eq!(req.safety_level, SafetyLevel::Strict);
        assert_eq!(req.context.as_deref(), Some("/home/user"));
        assert_eq!(req.request_id, "req-123");
        assert_eq!(req.agent_id.as_deref(), Some("devops"));
    }

    #[test]
    fn test_command_response_serialize_ok() {
        let resp = ApiCommandResponse {
            request_id: "req-123".to_string(),
            status: ApiStatus::Ok,
            command: Some("ls -la".to_string()),
            explanation: Some("List all files".to_string()),
            risk_level: Some(RiskLevel::Safe),
            estimated_impact: Some("Read-only".to_string()),
            alternatives: Some(vec!["ls -l".to_string()]),
            backend_used: Some("static_matcher".to_string()),
            generation_time_ms: Some(2),
            confidence_score: Some(0.95),
            warnings: vec![],
            error: None,
            reason: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"command\":\"ls -la\""));
        // Empty warnings and None fields should be omitted
        assert!(!json.contains("\"warnings\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_command_response_serialize_blocked() {
        let resp = ApiCommandResponse {
            request_id: "req-456".to_string(),
            status: ApiStatus::Blocked,
            command: Some("rm -rf /".to_string()),
            explanation: None,
            risk_level: Some(RiskLevel::Critical),
            estimated_impact: None,
            alternatives: None,
            backend_used: None,
            generation_time_ms: None,
            confidence_score: None,
            warnings: vec!["Recursive deletion of root filesystem".to_string()],
            error: None,
            reason: Some("Command blocked by safety validation".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"blocked\""));
        assert!(json.contains("\"reason\""));
    }

    #[test]
    fn test_execute_request_defaults() {
        let json = r#"{"command": "echo hello", "confirmed": true}"#;
        let req: ApiExecuteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.command, "echo hello");
        assert!(req.confirmed);
        assert_eq!(req.timeout_ms, 30_000);
        assert!(!req.request_id.is_empty());
    }

    #[test]
    fn test_health_response_serialize() {
        let resp = ApiHealthResponse {
            status: "ok".to_string(),
            version: "1.2.0".to_string(),
            backends: BackendStatus {
                static_matcher: true,
                embedded: true,
                ollama: false,
                claude: false,
            },
            safety_patterns: 52,
            uptime_seconds: 3600,
        };
        let json = serde_json::to_string_pretty(&resp).unwrap();
        assert!(json.contains("\"safety_patterns\": 52"));
    }
}
