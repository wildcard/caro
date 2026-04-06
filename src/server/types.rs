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

// ─── Knowledge API types ───

/// Request to search the knowledge index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeSearchParams {
    /// Search query
    pub q: String,
    /// Max results (default: 10)
    #[serde(default = "default_knowledge_limit")]
    pub limit: u32,
}

/// A single knowledge search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeResult {
    pub input: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub similarity: f32,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_type: Option<String>,
}

/// Response from knowledge search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeSearchResponse {
    pub results: Vec<ApiKnowledgeResult>,
    pub total: usize,
}

/// Request to record a successful command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeRecordRequest {
    pub input: String,
    pub command: String,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default)]
    pub agent_id: Option<String>,
}

/// Response from knowledge record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeRecordResponse {
    pub status: ApiStatus,
    pub message: String,
}

/// Response for knowledge export (markdown or JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeExportResponse {
    pub status: ApiStatus,
    pub entries: Vec<ApiKnowledgeResult>,
    pub total: usize,
}

/// Request to import knowledge from markdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeImportRequest {
    pub entries: Vec<ApiKnowledgeImportEntry>,
}

/// A single entry to import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeImportEntry {
    pub input: String,
    pub command: String,
    #[serde(default)]
    pub context: Option<String>,
}

/// Response from knowledge import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKnowledgeImportResponse {
    pub status: ApiStatus,
    pub imported: usize,
    pub skipped: usize,
}

// ─── WebSocket message types ───

/// WebSocket message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Client requests command generation
    CommandRequest {
        id: String,
        input: String,
        #[serde(default)]
        agent_id: Option<String>,
    },
    /// Server responds with generated command
    CommandResult {
        id: String,
        status: ApiStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        explanation: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        risk_level: Option<RiskLevel>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        warnings: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// Client requests command execution
    ExecutionRequest {
        id: String,
        command: String,
        confirmed: bool,
    },
    /// Server responds with execution result
    ExecutionResult {
        id: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
        execution_time_ms: u64,
    },
    /// Knowledge update notification
    KnowledgeUpdate {
        entries: Vec<ApiKnowledgeResult>,
    },
    /// Heartbeat (bidirectional)
    Heartbeat,
    /// Error message
    Error {
        message: String,
    },
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

fn default_knowledge_limit() -> u32 {
    10
}

fn default_success() -> bool {
    true
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

    #[test]
    fn test_knowledge_search_params_defaults() {
        let json = r#"{"q": "docker"}"#;
        let params: ApiKnowledgeSearchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.q, "docker");
        assert_eq!(params.limit, 10);
    }

    #[test]
    fn test_knowledge_record_request() {
        let json = r#"{"input": "list files", "command": "ls -la"}"#;
        let req: ApiKnowledgeRecordRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.input, "list files");
        assert_eq!(req.command, "ls -la");
        assert!(req.success); // default true
        assert!(req.context.is_none());
    }

    #[test]
    fn test_ws_message_command_request_roundtrip() {
        let msg = WsMessage::CommandRequest {
            id: "req-1".to_string(),
            input: "list files".to_string(),
            agent_id: Some("devops".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"command_request\""));
        let decoded: WsMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            WsMessage::CommandRequest { id, input, agent_id } => {
                assert_eq!(id, "req-1");
                assert_eq!(input, "list files");
                assert_eq!(agent_id.as_deref(), Some("devops"));
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_ws_message_heartbeat_roundtrip() {
        let msg = WsMessage::Heartbeat;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"heartbeat"}"#);
        let decoded: WsMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, WsMessage::Heartbeat));
    }

    #[test]
    fn test_ws_message_execution_result() {
        let msg = WsMessage::ExecutionResult {
            id: "req-2".to_string(),
            exit_code: 0,
            stdout: "hello\n".to_string(),
            stderr: String::new(),
            execution_time_ms: 5,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"execution_result\""));
        assert!(json.contains("\"exit_code\":0"));
    }
}
