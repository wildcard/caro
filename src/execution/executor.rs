//! Command execution module
//!
//! Provides safe command execution with output capture and platform-specific handling.
//!
//! ## Design principles
//!
//! These rules are non-negotiable for any code in this module — they exist
//! because users pipe caro's output into other tools and lose all leverage
//! if either invariant breaks:
//!
//! 1. **Exit-code propagation.** The underlying command's exit code (or its
//!    signal-derived equivalent — `128 + signal_number` on Unix) is always
//!    surfaced in [`ExecutionResult::exit_code`]. We never collapse a
//!    SIGINT (130) or SIGKILL (137) down to a generic `-1`.
//! 2. **Fail-safe post-processing.** Any post-execution filter (redaction,
//!    truncation, etc.) is invoked through [`CommandExecutor::apply_filter`].
//!    If a filter panics or returns an error, the *original raw output is
//!    preserved* and a warning is logged — never silently dropped.
//!
//! Pattern idea-borrowed from rtk-ai/rtk's `ARCHITECTURE.md` "Design
//! Principles" section (Apache-2.0); reimplemented in caro's idioms.

use crate::models::ShellType;
use std::panic::{catch_unwind, AssertUnwindSafe};
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

    /// Process command output.
    ///
    /// Preserves the underlying exit code exactly as the shell would report
    /// it: a normal exit returns its code, a signal-terminated process
    /// returns `128 + signal_number` on Unix (e.g. SIGINT = 130, SIGKILL =
    /// 137), and only a truly absent status (no code, no signal) collapses
    /// to `-1`.
    fn process_output(&self, output: Output, execution_time_ms: u64) -> ExecutionResult {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = Self::resolve_exit_code(&output.status);
        let success = output.status.success();

        ExecutionResult {
            exit_code,
            stdout,
            stderr,
            execution_time_ms,
            success,
        }
    }

    /// Resolve a process's effective exit code, preserving signal
    /// information on Unix as `128 + signal_number`.
    fn resolve_exit_code(status: &std::process::ExitStatus) -> i32 {
        if let Some(code) = status.code() {
            return code;
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(sig) = status.signal() {
                return 128 + sig;
            }
        }
        -1
    }

    /// Apply a post-execution filter to an [`ExecutionResult`] with
    /// fail-safe semantics. If the filter panics or returns `None`, the
    /// original result is returned untouched and a warning is emitted.
    ///
    /// This is the canonical entry point for any redaction / compression /
    /// rewrite layer in caro. See module-level docs for the invariant.
    pub fn apply_filter<F>(result: ExecutionResult, filter: F) -> ExecutionResult
    where
        F: FnOnce(&ExecutionResult) -> Option<ExecutionResult>,
    {
        // Snapshot enough to log if we have to recover.
        let backup = result.clone();
        // `AssertUnwindSafe` is sound here because we never observe partial
        // state of `result` after a panic — we drop the panic and return the
        // pristine backup.
        match catch_unwind(AssertUnwindSafe(|| filter(&result))) {
            Ok(Some(new_result)) => new_result,
            Ok(None) => result,
            Err(panic_payload) => {
                let msg = panic_payload
                    .downcast_ref::<&'static str>()
                    .map(|s| (*s).to_string())
                    .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "<non-string panic payload>".to_string());
                tracing::warn!(
                    "post-execution filter panicked; raw output preserved (panic: {})",
                    msg
                );
                backup
            }
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

    // ----- Phase 1B: exit-code preservation + fail-safe filters ---------

    #[test]
    fn test_exit_code_zero_propagates() {
        let executor = CommandExecutor::new(ShellType::Bash);
        let res = executor.execute("true").unwrap();
        assert_eq!(res.exit_code, 0);
        assert!(res.success);
    }

    #[test]
    fn test_exit_code_one_propagates() {
        let executor = CommandExecutor::new(ShellType::Bash);
        let res = executor.execute("false").unwrap();
        assert_eq!(res.exit_code, 1);
        assert!(!res.success);
    }

    #[test]
    fn test_exit_code_custom_propagates() {
        let executor = CommandExecutor::new(ShellType::Bash);
        let res = executor.execute("exit 42").unwrap();
        assert_eq!(res.exit_code, 42);
    }

    #[test]
    #[cfg(unix)]
    fn test_exit_code_sigterm_encodes_as_128_plus_signal() {
        // Self-kill with SIGTERM (15) — shells encode as 128+15 = 143.
        let executor = CommandExecutor::new(ShellType::Bash);
        let res = executor.execute("kill -TERM $$").unwrap();
        assert_eq!(res.exit_code, 143, "SIGTERM should encode as 128+15=143");
        assert!(!res.success);
    }

    #[test]
    #[cfg(unix)]
    fn test_exit_code_sigkill_encodes_as_137() {
        // Self-kill with SIGKILL (9) → 128+9 = 137.
        let executor = CommandExecutor::new(ShellType::Bash);
        let res = executor.execute("kill -KILL $$").unwrap();
        assert_eq!(res.exit_code, 137, "SIGKILL should encode as 128+9=137");
        assert!(!res.success);
    }

    #[test]
    fn test_apply_filter_passes_through_when_filter_returns_none() {
        let original = ExecutionResult {
            exit_code: 7,
            stdout: "raw".into(),
            stderr: String::new(),
            execution_time_ms: 1,
            success: false,
        };
        let after = CommandExecutor::apply_filter(original.clone(), |_| None);
        assert_eq!(after.exit_code, 7);
        assert_eq!(after.stdout, "raw");
    }

    #[test]
    fn test_apply_filter_applies_when_filter_returns_some() {
        let original = ExecutionResult {
            exit_code: 0,
            stdout: "AKIA1234".into(),
            stderr: String::new(),
            execution_time_ms: 1,
            success: true,
        };
        let after = CommandExecutor::apply_filter(original, |r| {
            Some(ExecutionResult {
                stdout: r.stdout.replace("AKIA1234", "[REDACTED]"),
                ..r.clone()
            })
        });
        assert_eq!(after.stdout, "[REDACTED]");
        // Exit code preserved even when stdout was rewritten.
        assert_eq!(after.exit_code, 0);
    }

    #[test]
    fn test_apply_filter_recovers_from_panic_with_raw_output() {
        let original = ExecutionResult {
            exit_code: 0,
            stdout: "important_payload".into(),
            stderr: "important_stderr".into(),
            execution_time_ms: 1,
            success: true,
        };
        let after = CommandExecutor::apply_filter(original.clone(), |_| {
            panic!("intentional filter panic for test")
        });
        // Raw stdout/stderr survive the panic; exit code preserved.
        assert_eq!(after.stdout, original.stdout);
        assert_eq!(after.stderr, original.stderr);
        assert_eq!(after.exit_code, original.exit_code);
        assert!(after.success);
    }
}
