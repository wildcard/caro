//! Data models for the history tracking system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Maximum total size of history storage in bytes (default: 100MB)
    pub max_history_size_bytes: u64,

    /// Maximum size for individual output fields before summarization (default: 10KB)
    pub max_output_size_bytes: usize,

    /// Whether history tracking is enabled
    pub enabled: bool,

    /// Number of days to retain history (0 = unlimited)
    pub retention_days: u32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_history_size_bytes: 100 * 1024 * 1024, // 100 MB
            max_output_size_bytes: 10 * 1024,          // 10 KB
            enabled: true,
            retention_days: 0, // Unlimited by default, size-based rotation takes precedence
        }
    }
}

/// Represents a single request/response record in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRecord {
    /// Unique identifier for this record
    pub id: String,

    /// Timestamp when the request was made
    pub timestamp: DateTime<Utc>,

    /// The natural language input from the user
    pub user_input: String,

    /// The shell command that was inferred/generated
    pub inferred_command: String,

    /// Whether the user chose to execute the command
    pub user_executed: bool,

    /// Outcome of the execution (if executed)
    pub execution_outcome: Option<ExecutionOutcome>,

    /// Exit code from the execution (if executed)
    pub exit_code: Option<i32>,

    /// Full execution output (may be truncated if too large)
    pub execution_output: Option<String>,

    /// Summarized version of the output (for large outputs)
    pub execution_output_summarized: Option<String>,

    /// Version of Caro that processed this request
    pub caro_version: String,

    /// Version of the prompt template used
    pub prompt_version: String,

    /// Time taken for inference in milliseconds
    pub inference_time_ms: u64,

    /// Time taken for execution in milliseconds (if executed)
    pub execution_time_ms: Option<u64>,

    /// Total time from request to completion in milliseconds
    pub total_time_ms: u64,

    /// Backend used for inference (e.g., "mlx", "ollama", "vllm")
    pub backend_used: String,

    /// Model used for inference
    pub model_used: Option<String>,

    /// Confidence score from the model (0.0 to 1.0)
    pub confidence_score: Option<f64>,

    /// Risk level assessed for the command
    pub risk_level: Option<String>,

    /// Shell type used
    pub shell_type: String,

    /// Whether the command was blocked by safety checks
    pub blocked_by_safety: bool,

    /// Reason for blocking (if blocked)
    pub block_reason: Option<String>,

    /// Any warnings generated during processing
    pub warnings: Vec<String>,

    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RequestRecord {
    /// Create a new request record with minimal required fields
    pub fn new(
        user_input: String,
        inferred_command: String,
        caro_version: String,
        prompt_version: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user_input,
            inferred_command,
            user_executed: false,
            execution_outcome: None,
            exit_code: None,
            execution_output: None,
            execution_output_summarized: None,
            caro_version,
            prompt_version,
            inference_time_ms: 0,
            execution_time_ms: None,
            total_time_ms: 0,
            backend_used: String::new(),
            model_used: None,
            confidence_score: None,
            risk_level: None,
            shell_type: String::new(),
            blocked_by_safety: false,
            block_reason: None,
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Builder method to set execution result
    pub fn with_execution(
        mut self,
        executed: bool,
        outcome: ExecutionOutcome,
        exit_code: i32,
        output: String,
        execution_time_ms: u64,
    ) -> Self {
        self.user_executed = executed;
        self.execution_outcome = Some(outcome);
        self.exit_code = Some(exit_code);
        self.execution_output = Some(output);
        self.execution_time_ms = Some(execution_time_ms);
        self
    }

    /// Builder method to set timing information
    pub fn with_timing(mut self, inference_ms: u64, total_ms: u64) -> Self {
        self.inference_time_ms = inference_ms;
        self.total_time_ms = total_ms;
        self
    }

    /// Builder method to set backend information
    pub fn with_backend(mut self, backend: String, model: Option<String>) -> Self {
        self.backend_used = backend;
        self.model_used = model;
        self
    }

    /// Builder method to set confidence and risk
    pub fn with_assessment(mut self, confidence: f64, risk_level: String) -> Self {
        self.confidence_score = Some(confidence);
        self.risk_level = Some(risk_level);
        self
    }

    /// Builder method to set shell type
    pub fn with_shell(mut self, shell: String) -> Self {
        self.shell_type = shell;
        self
    }

    /// Builder method to set safety block information
    pub fn with_block(mut self, blocked: bool, reason: Option<String>) -> Self {
        self.blocked_by_safety = blocked;
        self.block_reason = reason;
        self
    }

    /// Builder method to add warnings
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    /// Add metadata field
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
}

/// Outcome of command execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionOutcome {
    /// Command executed successfully (exit code 0)
    Success,
    /// Command executed but failed (non-zero exit code)
    Failed,
    /// Command execution timed out
    Timeout,
    /// Command was cancelled by user
    Cancelled,
    /// Command was not executed (user declined)
    NotExecuted,
}

impl std::fmt::Display for ExecutionOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
            Self::Timeout => write!(f, "timeout"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::NotExecuted => write!(f, "not_executed"),
        }
    }
}

/// Represents a version of the prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersion {
    /// Version identifier (semantic versioning recommended)
    pub version: String,

    /// Hash of the prompt template content
    pub content_hash: String,

    /// Human-readable description of changes
    pub description: Option<String>,

    /// When this version was registered
    pub registered_at: DateTime<Utc>,

    /// Whether this is the currently active version
    pub is_active: bool,
}

impl PromptVersion {
    /// Create a new prompt version
    pub fn new(version: String, content_hash: String) -> Self {
        Self {
            version,
            content_hash,
            description: None,
            registered_at: Utc::now(),
            is_active: true,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// History manifest tracking all records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryManifest {
    /// Manifest format version
    pub version: String,

    /// Index of record IDs with metadata for quick lookup
    pub records: HashMap<String, RecordMetadata>,

    /// Total size of all records in bytes
    pub total_size_bytes: u64,

    /// Maximum allowed size in bytes
    pub max_size_bytes: u64,

    /// When the manifest was last updated
    pub last_updated: DateTime<Utc>,

    /// Total number of records ever created
    pub total_records_created: u64,

    /// Registered prompt versions
    pub prompt_versions: Vec<PromptVersion>,

    /// Statistics about the history
    pub stats: HistoryStats,
}

impl HistoryManifest {
    /// Create a new empty manifest
    pub fn new(max_size_bytes: u64) -> Self {
        Self {
            version: "1.0.0".to_string(),
            records: HashMap::new(),
            total_size_bytes: 0,
            max_size_bytes,
            last_updated: Utc::now(),
            total_records_created: 0,
            prompt_versions: vec![PromptVersion::new(
                "1.0.0".to_string(),
                "initial".to_string(),
            )],
            stats: HistoryStats::default(),
        }
    }
}

/// Metadata about a record stored in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    /// Size of the record in bytes
    pub size_bytes: u64,

    /// When the record was created
    pub created_at: DateTime<Utc>,

    /// When the record was last accessed
    pub last_accessed: DateTime<Utc>,
}

/// Statistics about the history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryStats {
    /// Total number of records currently stored
    pub total_records: usize,

    /// Total size of all records in bytes
    pub total_size_bytes: u64,

    /// Number of successful executions
    pub successful_executions: u64,

    /// Number of failed executions
    pub failed_executions: u64,

    /// Number of commands not executed by user
    pub not_executed: u64,

    /// Number of commands blocked by safety
    pub blocked_by_safety: u64,

    /// Average inference time in milliseconds
    pub avg_inference_time_ms: f64,

    /// Oldest record timestamp
    pub oldest_record: Option<DateTime<Utc>>,

    /// Newest record timestamp
    pub newest_record: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_record_creation() {
        let record = RequestRecord::new(
            "list files".to_string(),
            "ls -la".to_string(),
            "0.1.0".to_string(),
            "1.0.0".to_string(),
        );

        assert!(!record.id.is_empty());
        assert_eq!(record.user_input, "list files");
        assert_eq!(record.inferred_command, "ls -la");
        assert!(!record.user_executed);
    }

    #[test]
    fn test_request_record_builder() {
        let record = RequestRecord::new(
            "delete temp".to_string(),
            "rm -rf /tmp/*".to_string(),
            "0.1.0".to_string(),
            "1.0.0".to_string(),
        )
        .with_execution(true, ExecutionOutcome::Success, 0, "".to_string(), 100)
        .with_timing(500, 600)
        .with_backend("mock".to_string(), Some("test-model".to_string()))
        .with_assessment(0.95, "moderate".to_string())
        .with_shell("bash".to_string());

        assert!(record.user_executed);
        assert_eq!(record.execution_outcome, Some(ExecutionOutcome::Success));
        assert_eq!(record.inference_time_ms, 500);
        assert_eq!(record.backend_used, "mock");
        assert_eq!(record.shell_type, "bash");
    }

    #[test]
    fn test_history_config_default() {
        let config = HistoryConfig::default();

        assert_eq!(config.max_history_size_bytes, 100 * 1024 * 1024);
        assert!(config.enabled);
    }

    #[test]
    fn test_execution_outcome_display() {
        assert_eq!(ExecutionOutcome::Success.to_string(), "success");
        assert_eq!(ExecutionOutcome::Failed.to_string(), "failed");
    }
}
