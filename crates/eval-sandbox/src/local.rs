// Local sandbox using temporary directories and process isolation

use async_trait::async_trait;
use eval_core::ShellType;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::sandbox::{ExecutionContext, ExecutionOutput, Sandbox, SandboxError};

/// Local sandbox implementation using temporary directories
pub struct LocalSandbox {
    /// Whether to clean up temp directories after execution
    cleanup: bool,
}

impl LocalSandbox {
    /// Create a new local sandbox
    pub fn new() -> Self {
        Self { cleanup: true }
    }

    /// Create a local sandbox without cleanup (useful for debugging)
    pub fn no_cleanup() -> Self {
        Self { cleanup: false }
    }

    /// Get the shell command and args for the given shell type
    fn get_shell_command(&self, shell: &ShellType) -> (&str, Vec<&str>) {
        match shell {
            ShellType::Bash => ("bash", vec!["-c"]),
            ShellType::Zsh => ("zsh", vec!["-c"]),
            ShellType::Fish => ("fish", vec!["-c"]),
            ShellType::PowerShell => ("pwsh", vec!["-Command"]),
            ShellType::Cmd => ("cmd", vec!["/C"]),
        }
    }

    /// Run setup commands in the sandbox
    async fn run_setup(
        &self,
        setup_commands: &[String],
        working_dir: &Path,
        shell: &ShellType,
    ) -> Result<(), SandboxError> {
        for setup_cmd in setup_commands {
            debug!("Running setup command: {}", setup_cmd);

            let (shell_cmd, shell_args) = self.get_shell_command(shell);
            let mut cmd = Command::new(shell_cmd);

            for arg in shell_args {
                cmd.arg(arg);
            }

            cmd.arg(setup_cmd)
                .current_dir(working_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::piped());

            let output = cmd.output().map_err(|e| {
                SandboxError::SetupFailed(format!("Failed to run setup command: {}", e))
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(SandboxError::SetupFailed(format!(
                    "Setup command failed: {}\nCommand: {}\nError: {}",
                    output.status, setup_cmd, stderr
                )));
            }
        }

        Ok(())
    }

    /// Get list of files in directory (recursively)
    fn list_files(&self, dir: &Path) -> Result<HashSet<PathBuf>, std::io::Error> {
        let mut files = HashSet::new();

        if !dir.exists() {
            return Ok(files);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                files.extend(self.list_files(&path)?);
            } else {
                files.insert(path);
            }
        }

        Ok(files)
    }
}

impl Default for LocalSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Sandbox for LocalSandbox {
    async fn is_available(&self) -> bool {
        // Local sandbox is always available
        true
    }

    async fn execute(&self, context: ExecutionContext) -> Result<ExecutionOutput, SandboxError> {
        // Create temporary directory
        let temp_dir = tempfile::tempdir().map_err(|e| {
            SandboxError::InitializationFailed(format!("Failed to create temp directory: {}", e))
        })?;

        let working_dir = temp_dir.path().to_path_buf();
        debug!("Created sandbox at: {:?}", working_dir);

        // Get files before execution
        let files_before = self.list_files(&working_dir).unwrap_or_default();

        // Run setup commands
        if !context.setup_commands.is_empty() {
            self.run_setup(&context.setup_commands, &working_dir, &context.shell)
                .await?;
        }

        // Prepare command execution
        let (shell_cmd, shell_args) = self.get_shell_command(&context.shell);
        let mut cmd = Command::new(shell_cmd);

        for arg in shell_args {
            cmd.arg(arg);
        }

        cmd.arg(&context.command)
            .current_dir(&working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        for (key, value) in &context.env {
            cmd.env(key, value);
        }

        debug!(
            "Executing command in {}: {}",
            shell_cmd, context.command
        );

        // Execute with timeout
        let start_time = Instant::now();
        let output_future = tokio::task::spawn_blocking(move || cmd.output());

        let output_result = timeout(context.timeout, output_future).await;

        let execution_time = start_time.elapsed();

        let (exit_code, stdout, stderr, timed_out) = match output_result {
            Ok(join_result) => match join_result {
                Ok(io_result) => match io_result {
                    Ok(output) => {
                        let exit_code = output.status.code().unwrap_or(-1);
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        (exit_code, stdout, stderr, false)
                    }
                    Err(e) => {
                        return Err(SandboxError::ExecutionFailed(format!(
                            "Command execution failed: {}",
                            e
                        )));
                    }
                },
                Err(e) => {
                    return Err(SandboxError::ExecutionFailed(format!(
                        "Task join failed: {}",
                        e
                    )));
                }
            },
            Err(_) => {
                warn!("Command timed out after {:?}", context.timeout);
                (
                    -1,
                    String::new(),
                    format!("Command timed out after {:?}", context.timeout),
                    true,
                )
            }
        };

        // Get files after execution
        let files_after = self.list_files(&working_dir).unwrap_or_default();

        // Determine created and modified files
        let created_files: Vec<PathBuf> = files_after
            .difference(&files_before)
            .cloned()
            .collect();

        // For simplicity, we consider all pre-existing files as potentially modified
        // In a more sophisticated implementation, we'd check modification times
        let modified_files: Vec<PathBuf> = files_before
            .intersection(&files_after)
            .cloned()
            .collect();

        let output = ExecutionOutput {
            exit_code,
            stdout,
            stderr,
            execution_time,
            working_dir: working_dir.clone(),
            created_files,
            modified_files,
            timed_out,
        };

        // Cleanup if enabled
        if self.cleanup {
            if let Err(e) = temp_dir.close() {
                warn!("Failed to cleanup temp directory: {}", e);
            }
        } else {
            // Persist the temp directory
            std::mem::forget(temp_dir);
            debug!("Sandbox directory persisted (not cleaned up)");
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "local"
    }

    fn description(&self) -> &'static str {
        "Local sandbox using temporary directories"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_local_sandbox_basic() {
        let sandbox = LocalSandbox::new();
        assert!(sandbox.is_available().await);

        let context = ExecutionContext {
            command: "echo 'hello world'".to_string(),
            shell: ShellType::Bash,
            working_dir: PathBuf::from("/tmp"),
            env: Default::default(),
            timeout: Duration::from_secs(5),
            setup_commands: vec![],
        };

        let result = sandbox.execute(context).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello world"));
        assert!(!result.timed_out);
    }

    #[tokio::test]
    async fn test_local_sandbox_timeout() {
        let sandbox = LocalSandbox::new();

        let context = ExecutionContext {
            command: "sleep 10".to_string(),
            shell: ShellType::Bash,
            working_dir: PathBuf::from("/tmp"),
            env: Default::default(),
            timeout: Duration::from_millis(100),
            setup_commands: vec![],
        };

        let result = sandbox.execute(context).await.unwrap();
        assert!(result.timed_out);
    }

    #[tokio::test]
    async fn test_local_sandbox_file_creation() {
        let sandbox = LocalSandbox::new();

        let context = ExecutionContext {
            command: "touch test_file.txt".to_string(),
            shell: ShellType::Bash,
            working_dir: PathBuf::from("/tmp"),
            env: Default::default(),
            timeout: Duration::from_secs(5),
            setup_commands: vec![],
        };

        let result = sandbox.execute(context).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(!result.created_files.is_empty());
    }
}
