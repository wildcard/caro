//! Execution module for capturing runtime context and shell detection
//!
//! Provides execution environment capture with sensitive data filtering.

use crate::models::{ExecutionContext as ExecutionContextModel, Platform, ShellType};
use std::collections::HashMap;
use std::path::PathBuf;

mod executor;
mod shell;

pub use executor::{CommandExecutor, ExecutionResult, ExecutorError};
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
    ///
    /// Automatically captures the current directory, shell type, platform, username,
    /// hostname, and environment variables (with sensitive data filtered).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::execution::ExecutionContext;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = ExecutionContext::capture()?;
    ///
    /// println!("Current directory: {}", context.current_dir().display());
    /// println!("Shell: {:?}", context.shell_type());
    /// println!("Platform: {:?}", context.platform());
    /// println!("Username: {}", context.username());
    /// println!("Hostname: {}", context.hostname());
    ///
    /// // Access environment variables (sensitive vars like API keys are filtered)
    /// if let Some(path) = context.get_env_var("PATH") {
    ///     println!("PATH: {}", path);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ExecutionError::CurrentDirError` if the current directory cannot be accessed.
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
    ///
    /// Generates a formatted string containing execution context information
    /// suitable for inclusion in LLM prompts. This helps the LLM generate
    /// commands appropriate for the user's environment.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::execution::ExecutionContext;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = ExecutionContext::capture()?;
    ///
    /// // Get context string for LLM prompt
    /// let prompt_context = context.to_prompt_context();
    /// println!("Context for LLM:\n{}", prompt_context);
    ///
    /// // Example output:
    /// // Current directory: /home/user/projects
    /// // Shell: Bash
    /// // Platform: Linux
    /// // Username: user
    /// // Hostname: mycomputer
    /// # Ok(())
    /// # }
    /// ```
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
        // Use platform-appropriate test directory, shell, and platform
        #[cfg(windows)]
        let test_dir = PathBuf::from("C:\\Users\\test");
        #[cfg(not(windows))]
        let test_dir = PathBuf::from("/tmp/test");

        #[cfg(windows)]
        let shell_type = ShellType::PowerShell;
        #[cfg(not(windows))]
        let shell_type = ShellType::Bash;

        #[cfg(windows)]
        let platform = Platform::Windows;
        #[cfg(target_os = "macos")]
        let platform = Platform::MacOS;
        #[cfg(all(not(windows), not(target_os = "macos")))]
        let platform = Platform::Linux;

        let result = ExecutionContext::new(test_dir.clone(), shell_type, platform);

        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.current_dir(), test_dir.as_path());
        assert_eq!(context.shell_type(), shell_type);
        assert_eq!(context.platform(), platform);
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
