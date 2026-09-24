//! Command execution module
//!
//! Provides safe command execution with output capture and platform-specific handling.

use crate::models::ShellType;
use std::io::Read;
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

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

        let output = match self.timeout_ms {
            Some(timeout) => Self::output_with_deadline(cmd, timeout)?,
            None => cmd.output().map_err(|e| {
                ExecutorError::SpawnError(format!("Failed to execute command: {}", e))
            })?,
        };

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(self.process_output(output, execution_time_ms))
    }

    /// Run the command, killing it (and its whole process group on Unix)
    /// once `timeout_ms` elapses.
    fn output_with_deadline(mut cmd: Command, timeout_ms: u64) -> Result<Output, ExecutorError> {
        // Put the shell in its own process group so a timeout also kills the
        // grandchildren it forks (e.g. `sleep` in `sleep 5; echo done`).
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);

        let mut child = cmd
            .spawn()
            .map_err(|e| ExecutorError::SpawnError(format!("Failed to execute command: {}", e)))?;

        // Drain pipes on threads so a chatty command can't block on a full pipe.
        let (tx, rx) = mpsc::channel();
        Self::drain(child.stdout.take(), true, tx.clone());
        Self::drain(child.stderr.take(), false, tx);

        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    Self::kill_tree(&mut child);
                    return Err(ExecutorError::Timeout(timeout_ms));
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(e) => return Err(ExecutorError::WaitError(e.to_string())),
            }
        };

        // The shell has exited, but a background descendant can still hold
        // stdout/stderr open, so keep enforcing the deadline while draining.
        let (mut stdout, mut stderr) = (None, None);
        while stdout.is_none() || stderr.is_none() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match rx.recv_timeout(remaining) {
                Ok((true, buf)) => stdout = Some(buf),
                Ok((false, buf)) => stderr = Some(buf),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    Self::kill_tree(&mut child);
                    return Err(ExecutorError::Timeout(timeout_ms));
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        Ok(Output {
            status,
            stdout: stdout.unwrap_or_default(),
            stderr: stderr.unwrap_or_default(),
        })
    }

    /// Read `pipe` to EOF on a thread and send `(is_stdout, bytes)` on `tx`.
    fn drain<R: Read + Send + 'static>(
        pipe: Option<R>,
        is_stdout: bool,
        tx: mpsc::Sender<(bool, Vec<u8>)>,
    ) {
        thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut pipe) = pipe {
                let _ = pipe.read_to_end(&mut buf);
            }
            let _ = tx.send((is_stdout, buf));
        });
    }

    /// SIGKILL the child's whole process group (Unix), then reap the child.
    fn kill_tree(child: &mut Child) {
        #[cfg(unix)]
        {
            // The child leads its own group (process_group(0)), so its pid is
            // the pgid. ESRCH means the group already exited, which is fine.
            let pgid = child.id() as libc::pid_t;
            // SAFETY: kill(2) takes plain integers and has no memory effects.
            if unsafe { libc::kill(-pgid, libc::SIGKILL) } != 0 {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() != Some(libc::ESRCH) {
                    eprintln!("caro: failed to kill process group {}: {}", pgid, err);
                }
            }
        }
        let _ = child.kill();
        let _ = child.wait();
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

    #[test]
    #[cfg(unix)]
    fn test_timeout_kills_hung_command() {
        // Regression guard: the timeout used to be checked only after the
        // child exited, so a hung command was never killed. The compound
        // command forces bash to fork `sleep` as a grandchild.
        let executor = CommandExecutor::new(ShellType::Bash).with_timeout(300);
        let start = Instant::now();
        let result = executor.execute("sleep 5; echo done");

        assert!(matches!(result, Err(ExecutorError::Timeout(300))));
        assert!(start.elapsed().as_millis() < 2000);
    }

    #[test]
    #[cfg(unix)]
    fn test_timeout_kills_background_descendant_holding_pipes() {
        // The shell exits at once, but the backgrounded `sleep` inherits
        // stdout/stderr and keeps them open. The deadline must still apply
        // while draining output.
        let executor = CommandExecutor::new(ShellType::Bash).with_timeout(300);
        let start = Instant::now();
        let result = executor.execute("sleep 5 &");

        assert!(matches!(result, Err(ExecutorError::Timeout(300))));
        assert!(start.elapsed().as_millis() < 2000);
    }

    #[test]
    #[cfg(unix)]
    fn test_timeout_not_triggered_for_fast_command() {
        let executor = CommandExecutor::new(ShellType::Bash).with_timeout(5000);
        let exec_result = executor.execute("echo fast; echo err >&2").unwrap();

        assert!(exec_result.success);
        assert!(exec_result.stdout.contains("fast"));
        assert!(exec_result.stderr.contains("err"));
    }
}
