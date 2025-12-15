//! Context Intelligence Engine - V2 Phase 1
//!
//! This module provides comprehensive context awareness for cmdai, enabling
//! intelligent command generation based on project type, git state, available
//! tools, and shell history.
//!
//! ## Architecture
//!
//! - `ContextGraph`: Core aggregator that builds complete context
//! - `ProjectParser`: Detects project types and parses metadata
//! - `GitAnalyzer`: Extracts git repository state
//! - `ToolDetector`: Identifies available infrastructure tools
//! - `HistoryAnalyzer`: Analyzes shell history patterns
//!
//! ## Performance
//!
//! All analyzers are designed to run in parallel with a target of <300ms total.

pub mod context_graph;
pub mod git_analyzer;
pub mod history_analyzer;
pub mod project_parser;
pub mod tool_detector;

// Re-export main types
pub use context_graph::ContextGraph;
pub use git_analyzer::{GitAnalyzer, GitContext};
pub use history_analyzer::{HistoryAnalyzer, HistoryContext};
pub use project_parser::{ProjectContext, ProjectParser, ProjectType};
pub use tool_detector::{InfrastructureContext, Tool, ToolDetector};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during context analysis
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ContextError {
    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("Git error: {message}")]
    GitError { message: String },

    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Context analysis timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl From<std::io::Error> for ContextError {
    fn from(err: std::io::Error) -> Self {
        ContextError::IoError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for ContextError {
    fn from(err: serde_json::Error) -> Self {
        ContextError::ParseError {
            message: err.to_string(),
        }
    }
}

/// Environment context captured from the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    pub shell: String,
    pub platform: String,
    pub cwd: String,
    pub user: String,
    pub hostname: String,
}

impl EnvironmentContext {
    /// Build environment context from current system state
    pub fn build() -> Result<Self, ContextError> {
        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| "unknown".to_string())
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string();

        let platform = std::env::consts::OS.to_string();

        let cwd = std::env::current_dir()
            .map_err(|e| ContextError::IoError {
                message: format!("Failed to get current directory: {}", e),
            })?
            .to_string_lossy()
            .to_string();

        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(Self {
            shell,
            platform,
            cwd,
            user,
            hostname,
        })
    }

    /// Convert to LLM-friendly string
    pub fn to_llm_context(&self) -> String {
        format!(
            "Shell: {}\nPlatform: {}\nWorking Directory: {}\nUser: {}@{}",
            self.shell, self.platform, self.cwd, self.user, self.hostname
        )
    }
}

/// Options for context building
#[derive(Debug, Clone)]
pub struct ContextOptions {
    /// Enable git analysis
    pub enable_git: bool,
    /// Enable tool detection
    pub enable_tools: bool,
    /// Enable history analysis
    pub enable_history: bool,
    /// Timeout for entire context build in milliseconds
    pub timeout_ms: u64,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            enable_git: true,
            enable_tools: true,
            enable_history: true,
            timeout_ms: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_context_build() {
        let env = EnvironmentContext::build().expect("Should build environment context");
        assert!(!env.shell.is_empty());
        assert!(!env.platform.is_empty());
        assert!(!env.cwd.is_empty());
    }

    #[test]
    fn test_environment_context_to_llm_string() {
        let env = EnvironmentContext::build().expect("Should build");
        let llm_str = env.to_llm_context();
        assert!(llm_str.contains("Shell:"));
        assert!(llm_str.contains("Platform:"));
        assert!(llm_str.contains("Working Directory:"));
    }
}
