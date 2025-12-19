//! Command execution module
//!
//! Provides safe command execution with output capture and platform-specific handling.

use crate::models::ShellType;
use std::process::{Command, Output, Stdio};
use std::time::Instant;

/// Result of command execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub success: bool,
}

/// Command executor errors
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Failed to spawn command: {0}")]
    SpawnError(String),

    #[error("Failed to wait for command: {0}")]
    WaitError(String),

    #[error("Command execution timeout after {0}ms")]
    Timeout(u64),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),
}

/// Command executor for running shell commands
pub struct CommandExecutor {
    shell_type: ShellType,
    timeout_ms: Option<u64>,
}

impl CommandExecutor {
    /// Create a new command executor for the specified shell
    pub fn new(shell_type: ShellType) -> Self {
        Self {
            shell_type,
            timeout_ms: None,
        }
    }

    /// Set execution timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Execute a command and capture output
    pub fn execute(&self, command: &str) -> Result<ExecutionResult, ExecutorError> {
        let start_time = Instant::now();

        // Create the appropriate shell command based on platform
        let mut cmd = self.create_shell_command(command)?;

        // Configure stdio
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        // Execute the command
        let output = cmd
            .output()
            .map_err(|e| ExecutorError::SpawnError(format!("Failed to execute command: {}", e)))?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Check for timeout
        if let Some(timeout) = self.timeout_ms {
            if execution_time_ms > timeout {
                return Err(ExecutorError::Timeout(timeout));
            }
        }

        Ok(self.process_output(output, execution_time_ms))
    }

    /// Create shell command based on platform and shell type
    fn create_shell_command(&self, command: &str) -> Result<Command, ExecutorError> {
        let cmd = match self.shell_type {
            ShellType::PowerShell => {
                let mut c = Command::new("powershell");
                c.arg("-NoProfile").arg("-Command").arg(command);
                c
            }
            ShellType::Cmd => {
                let mut c = Command::new("cmd");
                c.arg("/C").arg(command);
                c
            }
            ShellType::Bash => {
                let mut c = Command::new("bash");
                c.arg("-c").arg(command);
                c
            }
            ShellType::Zsh => {
                let mut c = Command::new("zsh");
                c.arg("-c").arg(command);
                c
            }
            ShellType::Fish => {
                let mut c = Command::new("fish");
                c.arg("-c").arg(command);
                c
            }
            ShellType::Sh => {
                let mut c = Command::new("sh");
                c.arg("-c").arg(command);
                c
            }
            ShellType::Unknown => {
                // Default to sh on Unix-like systems, cmd on Windows
                #[cfg(unix)]
                {
                    let mut c = Command::new("sh");
                    c.arg("-c").arg(command);
                    c
                }
                #[cfg(windows)]
                {
                    let mut c = Command::new("cmd");
                    c.arg("/C").arg(command);
                    c
                }
                #[cfg(not(any(unix, windows)))]
                {
                    return Err(ExecutorError::InvalidCommand(
                        "Unknown platform".to_string(),
                    ));
                }
            }
        };

        Ok(cmd)
    }

    /// Process command output
    fn process_output(&self, output: Output, execution_time_ms: u64) -> ExecutionResult {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        ExecutionResult {
            exit_code,
            stdout,
            stderr,
            execution_time_ms,
            success,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_executor_simple_command() {
        let executor = CommandExecutor::new(ShellType::Bash);
        let result = executor.execute("echo 'Hello, World!'");

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.success);
        assert_eq!(exec_result.exit_code, 0);
        assert!(exec_result.stdout.contains("Hello, World!"));
    }

    #[test]
    fn test_command_executor_error_command() {
        let executor = CommandExecutor::new(ShellType::Bash);
        let result = executor.execute("exit 1");

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(!exec_result.success);
        assert_eq!(exec_result.exit_code, 1);
    }

    #[test]
    fn test_command_executor_with_stderr() {
        let executor = CommandExecutor::new(ShellType::Bash);
        let result = executor.execute("echo 'error message' >&2");

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.stderr.contains("error message"));
    }

    #[test]
    #[cfg(unix)]
    fn test_different_shells() {
        // Test with sh
        let executor_sh = CommandExecutor::new(ShellType::Sh);
        let result = executor_sh.execute("echo 'sh test'");
        assert!(result.is_ok());

        // Test with bash if available
        if Command::new("bash").arg("--version").output().is_ok() {
            let executor_bash = CommandExecutor::new(ShellType::Bash);
            let result = executor_bash.execute("echo 'bash test'");
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_execution_time_tracking() {
        let executor = CommandExecutor::new(ShellType::Bash);
        let result = executor.execute("sleep 0.1");

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        // Execution time should be at least 100ms
        assert!(exec_result.execution_time_ms >= 100);
    }
}
