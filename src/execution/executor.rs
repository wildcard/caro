//! Command execution module
//!
//! Provides safe command execution with output capture and platform-specific handling.

use crate::models::ShellType;
use crate::sandbox::NonoSandbox;
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
    /// Optional Nono sandbox wrapper
    sandbox: Option<NonoSandbox>,
}

impl CommandExecutor {
    /// Create a new command executor for the specified shell
    pub fn new(shell_type: ShellType) -> Self {
        Self {
            shell_type,
            timeout_ms: None,
            sandbox: None,
        }
    }

    /// Set execution timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Wrap execution in a Nono kernel sandbox.
    ///
    /// If `nono` is not found in PATH at execution time, a warning is emitted
    /// and execution proceeds without sandboxing.
    pub fn with_sandbox(mut self, sandbox: NonoSandbox) -> Self {
        self.sandbox = Some(sandbox);
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

    /// Create shell command based on platform and shell type.
    ///
    /// When a [`NonoSandbox`] is configured and `nono` is available in PATH,
    /// the command is wrapped as `nono run [...] -- <shell> -c <command>`.
    /// If `nono` is configured but not found, a warning is logged and execution
    /// proceeds without sandboxing.
    fn create_shell_command(&self, command: &str) -> Result<Command, ExecutorError> {
        // Determine the inner shell binary and its flag style
        let (shell_bin, shell_flag) = match self.shell_type {
            ShellType::PowerShell => ("powershell", "-Command"),
            ShellType::Cmd => ("cmd", "/C"),
            ShellType::Bash => ("bash", "-c"),
            ShellType::Zsh => ("zsh", "-c"),
            ShellType::Fish => ("fish", "-c"),
            ShellType::Sh => ("sh", "-c"),
            ShellType::Unknown => {
                #[cfg(unix)]
                { ("sh", "-c") }
                #[cfg(windows)]
                { ("cmd", "/C") }
                #[cfg(not(any(unix, windows)))]
                {
                    return Err(ExecutorError::InvalidCommand(
                        "Unknown platform".to_string(),
                    ));
                }
            }
        };

        // PowerShell uses `-NoProfile` as an extra flag
        let extra_flags: &[&str] = if self.shell_type == ShellType::PowerShell {
            &["-NoProfile"]
        } else {
            &[]
        };

        // Optionally wrap in Nono sandbox
        if let Some(ref sandbox) = self.sandbox {
            if NonoSandbox::is_available() {
                let inner_args: Vec<&str> = extra_flags
                    .iter()
                    .copied()
                    .chain([shell_flag, command])
                    .collect();
                let (prog, args) = sandbox.wrap(shell_bin, &inner_args);
                let mut cmd = Command::new(prog);
                cmd.args(args);
                return Ok(cmd);
            } else {
                tracing::warn!(
                    "nono not found in PATH — running command without sandbox. \
                     Install nono (https://github.com/always-further/nono) to enable sandboxing."
                );
            }
        }

        // Plain execution (no sandbox)
        let mut cmd = Command::new(shell_bin);
        for flag in extra_flags {
            cmd.arg(flag);
        }
        cmd.arg(shell_flag).arg(command);
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
        // Use platform-appropriate shell
        #[cfg(windows)]
        let executor = CommandExecutor::new(ShellType::Cmd);
        #[cfg(not(windows))]
        let executor = CommandExecutor::new(ShellType::Bash);

        #[cfg(windows)]
        let result = executor.execute("echo Hello, World!");
        #[cfg(not(windows))]
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
        // Use platform-appropriate shell
        #[cfg(windows)]
        let executor = CommandExecutor::new(ShellType::Cmd);
        #[cfg(not(windows))]
        let executor = CommandExecutor::new(ShellType::Bash);

        #[cfg(windows)]
        let result = executor.execute("echo error message 1>&2");
        #[cfg(not(windows))]
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
        // Use platform-appropriate shell and sleep command
        #[cfg(windows)]
        let executor = CommandExecutor::new(ShellType::PowerShell);
        #[cfg(not(windows))]
        let executor = CommandExecutor::new(ShellType::Bash);

        #[cfg(windows)]
        let result = executor.execute("Start-Sleep -Milliseconds 100");
        #[cfg(not(windows))]
        let result = executor.execute("sleep 0.1");

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        // Execution time should be at least 100ms
        assert!(exec_result.execution_time_ms >= 100);
    }
}
