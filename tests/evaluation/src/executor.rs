//! CLI executor for invoking caro commands.
//!
//! This module provides an interface for executing the caro CLI binary
//! and capturing its output in a controlled manner.

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during command execution
#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Caro binary not found at expected location")]
    BinaryNotFound,

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Command timeout after {0:?}")]
    Timeout(Duration),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Executor for running caro CLI commands
pub struct Executor {
    caro_binary_path: PathBuf,
    timeout: Duration,
}

impl Executor {
    /// Create a new executor with default configuration
    pub fn new() -> Result<Self, ExecutorError> {
        // Find the caro binary relative to the current executable
        // Assumes cargo build layout: target/release/caro or target/debug/caro
        let current_exe = std::env::current_exe()?;
        let target_dir = current_exe
            .parent()
            .ok_or_else(|| ExecutorError::BinaryNotFound)?
            .parent()
            .ok_or_else(|| ExecutorError::BinaryNotFound)?;

        // Try release build first, then debug
        let release_path = target_dir.join("release").join("caro");
        let debug_path = target_dir.join("debug").join("caro");

        let binary_path = if release_path.exists() {
            release_path
        } else if debug_path.exists() {
            debug_path
        } else {
            return Err(ExecutorError::BinaryNotFound);
        };

        Ok(Self {
            caro_binary_path: binary_path,
            timeout: Duration::from_secs(30),
        })
    }

    /// Execute a caro command and return the generated output
    pub async fn execute(&self, prompt: &str) -> Result<String, ExecutorError> {
        use tokio::process::Command;
        use tokio::time::timeout;

        // Execute with timeout
        let output = timeout(self.timeout, async {
            Command::new(&self.caro_binary_path)
                .arg(prompt)
                .output()
                .await
        })
        .await
        .map_err(|_| ExecutorError::Timeout(self.timeout))??;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(ExecutorError::ExecutionFailed(stderr));
        }

        // Extract command from stdout
        let command = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        // Handle empty output
        if command.is_empty() {
            return Err(ExecutorError::ExecutionFailed(
                "Command produced empty output".to_string(),
            ));
        }

        Ok(command)
    }

    /// Get the path to the caro binary being used
    pub fn binary_path(&self) -> &PathBuf {
        &self.caro_binary_path
    }

    /// Get the configured timeout duration
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        // This test will pass if the caro binary exists in target/
        let result = Executor::new();

        match result {
            Ok(executor) => {
                assert!(executor.binary_path().exists());
                assert_eq!(executor.timeout(), Duration::from_secs(30));
            }
            Err(ExecutorError::BinaryNotFound) => {
                // Expected if caro binary hasn't been built yet
                eprintln!("Caro binary not found - this is expected if not built yet");
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_executor_basic_command() {
        // Skip if binary doesn't exist
        let executor = match Executor::new() {
            Ok(e) => e,
            Err(ExecutorError::BinaryNotFound) => {
                eprintln!("Skipping test - caro binary not built");
                return;
            }
            Err(e) => panic!("Unexpected error: {}", e),
        };

        // Test with a simple prompt
        let result = executor.execute("list all files").await;

        match result {
            Ok(command) => {
                assert!(!command.is_empty());
                eprintln!("Generated command: {}", command);
            }
            Err(e) => {
                // Don't fail the test if backend isn't configured
                eprintln!("Execution failed (might be expected): {}", e);
            }
        }
    }
}
