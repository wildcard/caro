//! Suggestions module - Proactive query suggestions based on user environment
//!
//! This module provides personalized command suggestions by analyzing:
//! - Shell history patterns (bash, zsh, fish)
//! - Installed tools via PATH analysis
//! - Current environment and context
//! - Git repository state (Caro loves Git!)
//!
//! For new users, it provides educational suggestions to help learn the terminal.

mod analyzer;
mod defaults;
mod environment;
mod generator;
mod history;
mod profile;
mod tools;

pub use analyzer::AnalysisCoordinator;
pub use defaults::{get_beginner_suggestions, get_git_suggestions};
pub use environment::{EnvironmentAnalyzer, EnvironmentInsights, ProjectType};
pub use generator::SuggestionGenerator;
pub use history::{CommandPatterns, HistoryAnalyzer, HistoryEntry};
pub use profile::{ExperienceLevel, ProfileManager, UserProfile, Workflow};
pub use tools::{DetectedTool, ToolCategory, ToolsAnalyzer};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A suggested query for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedQuery {
    /// The natural language query to suggest
    pub query: String,

    /// Short description of what this does
    pub description: String,

    /// Why this was suggested
    pub reason: SuggestionReason,

    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,

    /// Category for grouping
    pub category: QueryCategory,
}

impl SuggestedQuery {
    /// Create a new suggested query
    pub fn new(
        query: impl Into<String>,
        description: impl Into<String>,
        reason: SuggestionReason,
        category: QueryCategory,
    ) -> Self {
        Self {
            query: query.into(),
            description: description.into(),
            reason,
            relevance: 0.5,
            category,
        }
    }

    /// Set relevance score
    pub fn with_relevance(mut self, relevance: f32) -> Self {
        self.relevance = relevance.clamp(0.0, 1.0);
        self
    }
}

/// Reason why a query was suggested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionReason {
    /// Based on shell history patterns
    HistoryPattern { command: String },

    /// Based on detected tool
    DetectedTool { tool: String },

    /// Based on current directory context
    DirectoryContext { context: String },

    /// Based on time of day patterns
    TimeBasedHabit,

    /// Generic suggestion for new users
    NewUserOnboarding,

    /// Git-specific (Caro loves Git!)
    GitWorkflow { state: String },
}

impl std::fmt::Display for SuggestionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HistoryPattern { command } => write!(f, "Based on your use of '{}'", command),
            Self::DetectedTool { tool } => write!(f, "You have {} installed", tool),
            Self::DirectoryContext { context } => write!(f, "Based on {}", context),
            Self::TimeBasedHabit => write!(f, "Based on your usage patterns"),
            Self::NewUserOnboarding => write!(f, "Great for learning the terminal"),
            Self::GitWorkflow { state } => write!(f, "Git: {}", state),
        }
    }
}

/// Category for grouping suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryCategory {
    FileOperations,
    Git,
    Docker,
    Development,
    System,
    Network,
    Learning,
}

impl std::fmt::Display for QueryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileOperations => write!(f, "Files"),
            Self::Git => write!(f, "Git"),
            Self::Docker => write!(f, "Docker"),
            Self::Development => write!(f, "Dev"),
            Self::System => write!(f, "System"),
            Self::Network => write!(f, "Network"),
            Self::Learning => write!(f, "Learn"),
        }
    }
}

/// Error type for suggestions module
#[derive(Debug, thiserror::Error)]
pub enum SuggestionError {
    #[error("Failed to read history file: {0}")]
    HistoryReadError(String),

    #[error("Failed to parse history: {0}")]
    HistoryParseError(String),

    #[error("Profile error: {0}")]
    ProfileError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Analysis timeout")]
    Timeout,
}

/// Result type for suggestions module
pub type Result<T> = std::result::Result<T, SuggestionError>;

/// Configuration for the suggestions system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionsConfig {
    /// Maximum number of suggestions to generate
    pub max_suggestions: usize,

    /// Analysis timeout in seconds
    pub analysis_timeout_secs: u64,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,

    /// Minimum relevance score to include
    pub min_relevance: f32,

    /// Enable history analysis
    pub analyze_history: bool,

    /// Enable tools analysis
    pub analyze_tools: bool,

    /// Enable environment analysis
    pub analyze_environment: bool,
}

impl Default for SuggestionsConfig {
    fn default() -> Self {
        Self {
            max_suggestions: 5,
            analysis_timeout_secs: 2,
            cache_ttl_secs: 3600, // 1 hour
            min_relevance: 0.3,
            analyze_history: true,
            analyze_tools: true,
            analyze_environment: true,
        }
    }
}

/// Get the path to the .caro data directory
pub fn get_caro_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        SuggestionError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        ))
    })?;

    let caro_dir = home.join(".caro");

    // Create directory if it doesn't exist
    if !caro_dir.exists() {
        std::fs::create_dir_all(&caro_dir)?;
    }

    Ok(caro_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggested_query_creation() {
        let query = SuggestedQuery::new(
            "list files",
            "Show files in directory",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::FileOperations,
        );

        assert_eq!(query.query, "list files");
        assert_eq!(query.relevance, 0.5);
    }

    #[test]
    fn test_relevance_clamping() {
        let query = SuggestedQuery::new(
            "test",
            "test",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::Learning,
        )
        .with_relevance(1.5);

        assert_eq!(query.relevance, 1.0);
    }

    #[test]
    fn test_default_config() {
        let config = SuggestionsConfig::default();
        assert_eq!(config.max_suggestions, 5);
        assert!(config.analyze_history);
    }
}
