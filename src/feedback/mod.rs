//! Feedback System for cmdai
//!
//! This module provides a comprehensive feedback system that allows users to:
//! - Submit feedback about errors or issues they encounter
//! - Automatically capture rich context (environment, command, error info)
//! - Create GitHub issues with structured information
//! - Track the status of submitted feedback
//!
//! # Architecture
//!
//! The feedback system consists of several components:
//!
//! - **types**: Data structures for feedback, context, and status
//! - **capture**: Context capture engine for collecting environment information
//! - **redaction**: Sensitive data redaction to protect user privacy
//! - **github**: GitHub API client for creating issues
//! - **storage**: Local SQLite database for tracking feedback
//! - **tui**: Terminal UI for interactive feedback submission
//!
//! # Example
//!
//! ```no_run
//! use caro::feedback::{FeedbackId, FeedbackStatus};
//!
//! // Generate a new feedback ID
//! let id = FeedbackId::generate();
//! println!("Created feedback: {}", id);
//! ```

pub mod capture;
pub mod github;
pub mod redaction;
pub mod storage;
pub mod tui;
pub mod types;

// Re-export commonly used types
pub use types::{
    CommandInfo, EnvironmentInfo, ErrorInfo, Feedback, FeedbackContext, FeedbackId,
    FeedbackIdError, FeedbackStatus, GitContext, GitHubIssueRequest, HistoryEntry, SystemState,
};

pub use capture::capture_context;
pub use github::GitHubClient;
pub use redaction::{redact_context, redact_sensitive_data};
pub use storage::FeedbackDatabase;
pub use tui::run_feedback_interface;

use thiserror::Error;

/// Feedback system errors
#[derive(Debug, Error)]
pub enum FeedbackError {
    /// Error capturing context
    #[error("Failed to capture context: {0}")]
    CaptureError(String),

    /// Error with GitHub API
    #[error("GitHub API error: {0}")]
    GitHubError(String),

    /// Error with local storage
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Error with user input
    #[error("Invalid input: {0}")]
    InputError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<std::io::Error> for FeedbackError {
    fn from(err: std::io::Error) -> Self {
        FeedbackError::CaptureError(err.to_string())
    }
}

impl From<serde_json::Error> for FeedbackError {
    fn from(err: serde_json::Error) -> Self {
        FeedbackError::SerializationError(err.to_string())
    }
}

impl From<rusqlite::Error> for FeedbackError {
    fn from(err: rusqlite::Error) -> Self {
        FeedbackError::StorageError(err.to_string())
    }
}
