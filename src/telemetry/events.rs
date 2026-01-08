//! Telemetry event types and session management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Anonymous session identifier (rotates daily)
///
/// Generated from hash of machine ID + current date, ensuring:
/// - No personal identification possible
/// - Daily rotation prevents long-term tracking
/// - Consistent within a day for session correlation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionId(String);

impl SessionId {
    /// Generate anonymous session ID from machine ID + date
    ///
    /// Format: first 16 characters of SHA256(machine_id + date)
    pub fn generate() -> Self {
        let machine_id = machine_uid::get().unwrap_or_else(|_| "unknown".to_string());
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let combined = format!("{}{}", machine_id, date);

        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        SessionId(hash[..16].to_string())
    }

    /// Get the session ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Telemetry event types
///
/// All event types are designed to collect metadata only, never content.
/// Privacy guarantees:
/// - No command text or natural language input
/// - No file paths or directory structures
/// - No environment variables or secrets
/// - No personally identifiable information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventType {
    /// Session started
    SessionStart {
        /// Caro version (e.g., "1.1.0-beta")
        version: String,
        /// OS platform (e.g., "macos", "linux")
        platform: String,
        /// Shell type (e.g., "bash", "zsh", "fish")
        shell_type: String,
        /// Available backends (e.g., ["embedded", "ollama"])
        backend_available: Vec<String>,
    },

    /// Session ended
    SessionEnd {
        /// Total session duration in milliseconds
        duration_ms: u64,
        /// Number of commands generated
        commands_generated: u32,
        /// Number of commands executed
        commands_executed: u32,
    },

    /// Command generation attempted
    CommandGeneration {
        /// Backend used (e.g., "embedded", "ollama", "static")
        backend: String,
        /// Generation duration in milliseconds
        duration_ms: u64,
        /// Whether generation succeeded
        success: bool,
        /// Error category if failed (e.g., "timeout", "parse_error")
        error_category: Option<String>,
    },

    /// Safety validation performed
    SafetyValidation {
        /// Risk level detected (e.g., "critical", "high", "medium", "low")
        risk_level: String,
        /// Action taken (e.g., "allowed", "blocked", "warned")
        action_taken: String,
        /// Pattern category if matched (e.g., "destructive", "privilege_escalation")
        pattern_category: Option<String>,
    },

    /// Backend error occurred
    BackendError {
        /// Backend that errored (e.g., "embedded", "ollama")
        backend: String,
        /// Error category (e.g., "model_load_failed", "inference_timeout")
        error_category: String,
        /// Whether error is recoverable
        recoverable: bool,
    },
}

/// Telemetry event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event ID
    pub id: Uuid,
    /// Session identifier (anonymous, daily rotation)
    pub session_id: SessionId,
    /// Event timestamp (UTC)
    pub timestamp: DateTime<Utc>,
    /// Event type and data
    #[serde(flatten)]
    pub event_type: EventType,
}

impl Event {
    /// Create a new telemetry event
    pub fn new(session_id: SessionId, event_type: EventType) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            timestamp: Utc::now(),
            event_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_generation() {
        let id1 = SessionId::generate();
        let id2 = SessionId::generate();

        // IDs should be same within same day
        assert_eq!(id1, id2);

        // ID should be 16 characters
        assert_eq!(id1.as_str().len(), 16);
    }

    #[test]
    fn test_event_creation() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id.clone(),
            EventType::SessionStart {
                version: "1.0.0".to_string(),
                platform: "linux".to_string(),
                shell_type: "bash".to_string(),
                backend_available: vec!["embedded".to_string()],
            },
        );

        assert_eq!(event.session_id, session_id);
        assert!(matches!(event.event_type, EventType::SessionStart { .. }));
    }

    #[test]
    fn test_event_serialization() {
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::CommandGeneration {
                backend: "embedded".to_string(),
                duration_ms: 1500,
                success: true,
                error_category: None,
            },
        );

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("command_generation"));
        assert!(json.contains("embedded"));

        // Verify no sensitive data patterns
        assert!(!json.contains("/Users"));
        assert!(!json.contains("PATH="));
    }
}
