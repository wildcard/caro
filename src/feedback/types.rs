//! Feedback system data types
//!
//! This module defines all the data structures used by the feedback system,
//! including feedback IDs, context information, and status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Unique identifier for feedback submissions (format: fb-XXXXXX)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeedbackId(String);

impl FeedbackId {
    /// Generate a new random feedback ID
    ///
    /// Format: fb-XXXXXX where X is alphanumeric (lowercase letters and digits)
    pub fn generate() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Generate a pseudo-random 6-character alphanumeric string
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        // Mix timestamp with some randomness from memory addresses
        let random_component = std::ptr::null::<u8>() as usize;
        let seed = timestamp ^ (random_component as u128);

        // Convert to base36 (0-9, a-z) and take 6 characters
        let chars: Vec<char> = "0123456789abcdefghijklmnopqrstuvwxyz".chars().collect();
        let mut result = String::with_capacity(6);
        let mut n = seed;

        for _ in 0..6 {
            let idx = (n % 36) as usize;
            result.push(chars[idx]);
            n /= 36;
        }

        Self(format!("fb-{}", result))
    }

    /// Parse from string with validation
    ///
    /// # Arguments
    /// * `s` - String to parse (must be format fb-XXXXXX)
    ///
    /// # Returns
    /// Result containing the FeedbackId or an error message
    pub fn parse(s: &str) -> Result<Self, FeedbackIdError> {
        // Must start with "fb-"
        if !s.starts_with("fb-") {
            return Err(FeedbackIdError::InvalidPrefix);
        }

        let suffix = &s[3..];

        // Must have exactly 6 characters after prefix
        if suffix.len() != 6 {
            return Err(FeedbackIdError::InvalidLength {
                expected: 6,
                actual: suffix.len(),
            });
        }

        // Must be alphanumeric lowercase
        if !suffix.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err(FeedbackIdError::InvalidCharacters);
        }

        Ok(Self(s.to_string()))
    }

    /// Get the inner string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FeedbackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for FeedbackId {
    type Err = FeedbackIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Error type for FeedbackId parsing
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FeedbackIdError {
    #[error("Feedback ID must start with 'fb-'")]
    InvalidPrefix,

    #[error("Feedback ID suffix must be {expected} characters, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Feedback ID must contain only lowercase alphanumeric characters")]
    InvalidCharacters,
}

/// Complete feedback context captured from user's environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackContext {
    /// Timestamp when context was captured
    pub timestamp: DateTime<Utc>,

    /// Version of cmdai/caro that generated this feedback
    pub cmdai_version: String,

    /// Environment information
    pub environment: EnvironmentInfo,

    /// Command-related information
    pub command_info: CommandInfo,

    /// Error information if feedback is about an error
    pub error_info: Option<ErrorInfo>,

    /// System state information
    pub system_state: SystemState,

    /// Git repository context if in a git repo
    pub git_context: Option<GitContext>,
}

/// Environment information from the user's system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system name (e.g., "macos", "linux", "windows")
    pub os: String,

    /// Operating system version (e.g., "14.1")
    pub os_version: String,

    /// CPU architecture (e.g., "arm64", "x86_64")
    pub arch: String,

    /// Shell being used (e.g., "zsh", "bash")
    pub shell: String,

    /// Terminal emulator (e.g., "iTerm.app", "Terminal")
    pub terminal: String,

    /// Rust version if available
    pub rust_version: Option<String>,
}

/// Information about the command that triggered the feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Original natural language prompt from user
    pub user_prompt: String,

    /// Generated shell command
    pub generated_command: String,

    /// Backend used for generation (e.g., "mlx", "embedded", "static")
    pub backend: String,

    /// Model ID or path used
    pub model: Option<String>,

    /// Recent command history (limited to prevent data leak)
    #[serde(default)]
    pub command_history: Vec<HistoryEntry>,
}

/// A single entry in command history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Natural language prompt
    pub prompt: String,

    /// Generated command
    pub command: String,

    /// When this command was generated
    pub timestamp: DateTime<Utc>,

    /// Whether the command was successful
    pub success: bool,
}

/// Error information captured when feedback is about an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Exit code if available
    pub exit_code: Option<i32>,

    /// Standard error output
    pub stderr: String,

    /// Standard output
    pub stdout: String,

    /// Error message
    pub error_message: String,

    /// Error type/category
    pub error_type: Option<String>,
}

/// Current system state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    /// Available inference backends
    pub available_backends: Vec<String>,

    /// Cache directory path
    pub cache_dir: PathBuf,

    /// Config file path if exists
    pub config_file: Option<PathBuf>,

    /// Whether running in CI/CD environment
    pub is_ci: bool,

    /// Whether stdin is a TTY
    pub is_interactive: bool,
}

/// Git repository context if feedback is submitted from within a repo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    /// Remote repository URL (if configured)
    pub repo_url: Option<String>,

    /// Current branch name
    pub current_branch: String,

    /// Whether there are uncommitted changes
    pub has_uncommitted_changes: bool,

    /// Last commit hash (short form)
    pub last_commit_hash: Option<String>,
}

/// User's feedback submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    /// Unique feedback identifier
    pub id: FeedbackId,

    /// When feedback was submitted
    pub timestamp: DateTime<Utc>,

    /// User's description of the issue
    pub user_description: String,

    /// Optional reproduction steps
    pub reproduction_steps: Option<String>,

    /// Captured context
    pub context: FeedbackContext,

    /// URL to the created GitHub issue (if submitted)
    pub github_issue_url: Option<String>,

    /// Current status of the feedback
    pub status: FeedbackStatus,
}

/// Status of a feedback submission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FeedbackStatus {
    /// Feedback has been submitted locally
    #[default]
    Submitted,

    /// Feedback has been triaged by maintainers
    Triaged,

    /// Work is in progress on the issue
    InProgress,

    /// A fix is available (in development or beta)
    FixAvailable,

    /// The issue has been resolved
    Resolved,
}

impl fmt::Display for FeedbackStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Submitted => write!(f, "submitted"),
            Self::Triaged => write!(f, "triaged"),
            Self::InProgress => write!(f, "in_progress"),
            Self::FixAvailable => write!(f, "fix_available"),
            Self::Resolved => write!(f, "resolved"),
        }
    }
}

impl std::str::FromStr for FeedbackStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "submitted" => Ok(Self::Submitted),
            "triaged" => Ok(Self::Triaged),
            "in_progress" | "inprogress" => Ok(Self::InProgress),
            "fix_available" | "fixavailable" => Ok(Self::FixAvailable),
            "resolved" => Ok(Self::Resolved),
            _ => Err(format!("Unknown feedback status: {}", s)),
        }
    }
}

/// GitHub issue creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssueRequest {
    /// Issue title
    pub title: String,

    /// Issue body (markdown)
    pub body: String,

    /// Labels to apply
    pub labels: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // FeedbackId Tests
    // ==========================================================================

    #[test]
    fn test_feedback_id_generation() {
        let id1 = FeedbackId::generate();
        let id2 = FeedbackId::generate();

        // IDs should start with fb-
        assert!(id1.as_str().starts_with("fb-"), "ID should start with fb-");
        assert!(id2.as_str().starts_with("fb-"), "ID should start with fb-");

        // IDs should be 9 characters total (fb- + 6 chars)
        assert_eq!(id1.as_str().len(), 9, "ID should be 9 characters");
        assert_eq!(id2.as_str().len(), 9, "ID should be 9 characters");

        // Suffix should be lowercase alphanumeric
        let suffix1 = &id1.as_str()[3..];
        assert!(
            suffix1.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "Suffix should be lowercase alphanumeric"
        );
    }

    #[test]
    fn test_feedback_id_parse_valid() {
        let id = FeedbackId::parse("fb-abc123").unwrap();
        assert_eq!(id.as_str(), "fb-abc123");
    }

    #[test]
    fn test_feedback_id_parse_invalid_prefix() {
        let result = FeedbackId::parse("id-abc123");
        assert!(matches!(result, Err(FeedbackIdError::InvalidPrefix)));
    }

    #[test]
    fn test_feedback_id_parse_invalid_length() {
        let result = FeedbackId::parse("fb-abc");
        assert!(matches!(
            result,
            Err(FeedbackIdError::InvalidLength { expected: 6, actual: 3 })
        ));

        let result2 = FeedbackId::parse("fb-abcdefgh");
        assert!(matches!(
            result2,
            Err(FeedbackIdError::InvalidLength { expected: 6, actual: 8 })
        ));
    }

    #[test]
    fn test_feedback_id_parse_invalid_characters() {
        // Uppercase not allowed
        let result = FeedbackId::parse("fb-ABC123");
        assert!(matches!(result, Err(FeedbackIdError::InvalidCharacters)));

        // Special characters not allowed
        let result2 = FeedbackId::parse("fb-abc-12");
        assert!(matches!(result2, Err(FeedbackIdError::InvalidCharacters)));
    }

    #[test]
    fn test_feedback_id_display() {
        let id = FeedbackId::parse("fb-abc123").unwrap();
        assert_eq!(format!("{}", id), "fb-abc123");
    }

    #[test]
    fn test_feedback_id_from_str() {
        let id: FeedbackId = "fb-xyz789".parse().unwrap();
        assert_eq!(id.as_str(), "fb-xyz789");
    }

    // ==========================================================================
    // FeedbackContext Serialization Tests
    // ==========================================================================

    #[test]
    fn test_context_serialization() {
        let context = FeedbackContext {
            timestamp: Utc::now(),
            cmdai_version: "1.0.0".to_string(),
            environment: EnvironmentInfo {
                os: "macos".to_string(),
                os_version: "14.1".to_string(),
                arch: "arm64".to_string(),
                shell: "zsh".to_string(),
                terminal: "iTerm.app".to_string(),
                rust_version: Some("1.75.0".to_string()),
            },
            command_info: CommandInfo {
                user_prompt: "list files".to_string(),
                generated_command: "ls -la".to_string(),
                backend: "static".to_string(),
                model: None,
                command_history: vec![],
            },
            error_info: None,
            system_state: SystemState {
                available_backends: vec!["static".to_string(), "embedded".to_string()],
                cache_dir: PathBuf::from("/home/user/.cache/caro"),
                config_file: Some(PathBuf::from("/home/user/.config/caro/config.toml")),
                is_ci: false,
                is_interactive: true,
            },
            git_context: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&context).expect("Should serialize to JSON");

        // Deserialize back
        let deserialized: FeedbackContext =
            serde_json::from_str(&json).expect("Should deserialize from JSON");

        assert_eq!(context.cmdai_version, deserialized.cmdai_version);
        assert_eq!(context.environment.os, deserialized.environment.os);
        assert_eq!(
            context.command_info.user_prompt,
            deserialized.command_info.user_prompt
        );
    }

    #[test]
    fn test_context_serialization_with_all_fields() {
        let context = FeedbackContext {
            timestamp: Utc::now(),
            cmdai_version: "1.1.0".to_string(),
            environment: EnvironmentInfo {
                os: "linux".to_string(),
                os_version: "6.1.0".to_string(),
                arch: "x86_64".to_string(),
                shell: "bash".to_string(),
                terminal: "gnome-terminal".to_string(),
                rust_version: None,
            },
            command_info: CommandInfo {
                user_prompt: "find large files".to_string(),
                generated_command: "find . -size +100M".to_string(),
                backend: "embedded".to_string(),
                model: Some("smollm-135m".to_string()),
                command_history: vec![HistoryEntry {
                    prompt: "previous command".to_string(),
                    command: "ls".to_string(),
                    timestamp: Utc::now(),
                    success: true,
                }],
            },
            error_info: Some(ErrorInfo {
                exit_code: Some(1),
                stderr: "error occurred".to_string(),
                stdout: "partial output".to_string(),
                error_message: "Command failed".to_string(),
                error_type: Some("ExecutionError".to_string()),
            }),
            system_state: SystemState {
                available_backends: vec!["static".to_string()],
                cache_dir: PathBuf::from("/tmp/cache"),
                config_file: None,
                is_ci: true,
                is_interactive: false,
            },
            git_context: Some(GitContext {
                repo_url: Some("https://github.com/user/repo".to_string()),
                current_branch: "main".to_string(),
                has_uncommitted_changes: true,
                last_commit_hash: Some("abc1234".to_string()),
            }),
        };

        let json = serde_json::to_string_pretty(&context).expect("Should serialize");
        let deserialized: FeedbackContext =
            serde_json::from_str(&json).expect("Should deserialize");

        assert!(deserialized.error_info.is_some());
        assert!(deserialized.git_context.is_some());
        assert_eq!(
            deserialized.error_info.as_ref().unwrap().exit_code,
            Some(1)
        );
        assert_eq!(
            deserialized.git_context.as_ref().unwrap().current_branch,
            "main"
        );
    }

    // ==========================================================================
    // FeedbackStatus Tests
    // ==========================================================================

    #[test]
    fn test_feedback_status_display() {
        assert_eq!(format!("{}", FeedbackStatus::Submitted), "submitted");
        assert_eq!(format!("{}", FeedbackStatus::Triaged), "triaged");
        assert_eq!(format!("{}", FeedbackStatus::InProgress), "in_progress");
        assert_eq!(format!("{}", FeedbackStatus::FixAvailable), "fix_available");
        assert_eq!(format!("{}", FeedbackStatus::Resolved), "resolved");
    }

    #[test]
    fn test_feedback_status_parse() {
        assert_eq!(
            "submitted".parse::<FeedbackStatus>().unwrap(),
            FeedbackStatus::Submitted
        );
        assert_eq!(
            "TRIAGED".parse::<FeedbackStatus>().unwrap(),
            FeedbackStatus::Triaged
        );
        assert_eq!(
            "in_progress".parse::<FeedbackStatus>().unwrap(),
            FeedbackStatus::InProgress
        );
        assert_eq!(
            "inprogress".parse::<FeedbackStatus>().unwrap(),
            FeedbackStatus::InProgress
        );
        assert_eq!(
            "fix_available".parse::<FeedbackStatus>().unwrap(),
            FeedbackStatus::FixAvailable
        );
        assert_eq!(
            "resolved".parse::<FeedbackStatus>().unwrap(),
            FeedbackStatus::Resolved
        );
    }

    #[test]
    fn test_feedback_status_parse_invalid() {
        assert!("unknown".parse::<FeedbackStatus>().is_err());
    }

    // ==========================================================================
    // Feedback Tests
    // ==========================================================================

    #[test]
    fn test_feedback_serialization() {
        let feedback = Feedback {
            id: FeedbackId::parse("fb-abc123").unwrap(),
            timestamp: Utc::now(),
            user_description: "The command didn't work".to_string(),
            reproduction_steps: Some("1. Run caro list files\n2. See error".to_string()),
            context: FeedbackContext {
                timestamp: Utc::now(),
                cmdai_version: "1.0.0".to_string(),
                environment: EnvironmentInfo {
                    os: "macos".to_string(),
                    os_version: "14.0".to_string(),
                    arch: "arm64".to_string(),
                    shell: "zsh".to_string(),
                    terminal: "Terminal.app".to_string(),
                    rust_version: None,
                },
                command_info: CommandInfo {
                    user_prompt: "list files".to_string(),
                    generated_command: "ls".to_string(),
                    backend: "static".to_string(),
                    model: None,
                    command_history: vec![],
                },
                error_info: None,
                system_state: SystemState {
                    available_backends: vec![],
                    cache_dir: PathBuf::from("/tmp"),
                    config_file: None,
                    is_ci: false,
                    is_interactive: true,
                },
                git_context: None,
            },
            github_issue_url: None,
            status: FeedbackStatus::Submitted,
        };

        let json = serde_json::to_string(&feedback).expect("Should serialize");
        let deserialized: Feedback = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(feedback.id, deserialized.id);
        assert_eq!(feedback.user_description, deserialized.user_description);
        assert_eq!(feedback.status, deserialized.status);
    }

    // ==========================================================================
    // GitHubIssueRequest Tests
    // ==========================================================================

    #[test]
    fn test_github_issue_request_serialization() {
        let request = GitHubIssueRequest {
            title: "[Bug] Command generation failed".to_string(),
            body: "## Description\n\nThe command failed".to_string(),
            labels: vec!["bug".to_string(), "user-feedback".to_string()],
        };

        let json = serde_json::to_string(&request).expect("Should serialize");
        assert!(json.contains("user-feedback"));

        let deserialized: GitHubIssueRequest =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(request.title, deserialized.title);
        assert_eq!(request.labels.len(), 2);
    }
}
