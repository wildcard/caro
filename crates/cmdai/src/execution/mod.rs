//! Execution module for capturing runtime context and shell detection
//!
//! Provides execution environment capture with sensitive data filtering.

use crate::models::{ExecutionContext as ExecutionContextModel, Platform, ShellType};
use std::collections::HashMap;
use std::path::PathBuf;

mod shell;
pub use shell::{PlatformDetector, ShellDetector};

/// Execution-related errors
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Failed to get current directory: {0}")]
    CurrentDirError(#[from] std::io::Error),

    #[error("Current directory not accessible: {0}")]
    CurrentDirNotAccessible(String),

    #[error("Environment variable error: {0}")]
    EnvVarError(String),

    #[error("Invalid execution context: {0}")]
    InvalidContext(String),
}

impl From<String> for ExecutionError {
    fn from(s: String) -> Self {
        ExecutionError::InvalidContext(s)
    }
}

/// Wrapper for ExecutionContext with additional methods
pub struct ExecutionContext {
    inner: ExecutionContextModel,
}

impl ExecutionContext {
    /// Capture current execution context from the environment
    pub fn capture() -> Result<Self, ExecutionError> {
        let current_dir = std::env::current_dir()?;
        let shell_type = ShellType::detect();
        let platform = Platform::detect();

        let inner = ExecutionContextModel::new(current_dir, shell_type, platform)?;

        Ok(Self { inner })
    }

    /// Create a new execution context with custom values
    pub fn new(
        current_dir: PathBuf,
        shell_type: ShellType,
        platform: Platform,
    ) -> Result<Self, ExecutionError> {
        let inner = ExecutionContextModel::new(current_dir, shell_type, platform)
            .map_err(ExecutionError::InvalidContext)?;

        Ok(Self { inner })
    }

    /// Get the current directory
    pub fn current_dir(&self) -> &std::path::Path {
        &self.inner.current_dir
    }

    /// Get the shell type
    pub fn shell_type(&self) -> ShellType {
        self.inner.shell_type
    }

    /// Get the platform
    pub fn platform(&self) -> Platform {
        self.inner.platform
    }

    /// Get the username
    pub fn username(&self) -> &str {
        &self.inner.username
    }

    /// Get the hostname
    pub fn hostname(&self) -> &str {
        &self.inner.hostname
    }

    /// Check if an environment variable exists
    pub fn has_env_var(&self, key: &str) -> bool {
        self.inner.environment_vars.contains_key(key)
    }

    /// Get the environment variables map
    pub fn environment_vars(&self) -> &HashMap<String, String> {
        &self.inner.environment_vars
    }

    /// Get an environment variable value
    pub fn get_env_var(&self, key: &str) -> Option<&str> {
        self.inner.environment_vars.get(key).map(|s| s.as_str())
    }

    /// Get the timestamp when context was captured
    pub fn captured_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.inner.captured_at
    }

    /// Convert to prompt context string for LLM
    pub fn to_prompt_context(&self) -> String {
        self.inner.to_prompt_context()
    }

    /// Get the inner model for serialization
    pub fn into_inner(self) -> ExecutionContextModel {
        self.inner
    }

    /// Get a reference to the inner model
    pub fn as_inner(&self) -> &ExecutionContextModel {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_capture() {
        let result = ExecutionContext::capture();
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(context.current_dir().is_absolute());
        assert!(!context.username().is_empty());
        assert!(!context.hostname().is_empty());
    }

    #[test]
    fn test_execution_context_new() {
        let test_dir = PathBuf::from("/tmp/test");
        let result = ExecutionContext::new(test_dir.clone(), ShellType::Bash, Platform::Linux);

        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.current_dir(), test_dir.as_path());
        assert_eq!(context.shell_type(), ShellType::Bash);
        assert_eq!(context.platform(), Platform::Linux);
    }

    #[test]
    fn test_context_filters_sensitive_vars() {
        std::env::set_var("TEST_API_KEY", "secret");

        let context = ExecutionContext::capture().unwrap();

        // API_KEY should be filtered
        assert!(!context.has_env_var("TEST_API_KEY"));

        std::env::remove_var("TEST_API_KEY");
    }
}
