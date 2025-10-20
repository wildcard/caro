// Docker sandbox for isolated command execution

use async_trait::async_trait;
use eval_core::ShellType;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::sandbox::{ExecutionContext, ExecutionOutput, Sandbox, SandboxError};

/// Docker sandbox implementation
pub struct DockerSandbox {
    /// Docker image to use
    image: String,

    /// Whether to enable network access
    network: bool,

    /// Memory limit in megabytes
    memory_limit_mb: Option<u64>,

    /// CPU limit (number of cores)
    cpu_limit: Option<f64>,

    /// Whether to remove containers after execution
    cleanup: bool,
}

impl DockerSandbox {
    /// Create a new Docker sandbox with default settings
    pub fn new() -> Self {
        Self {
            image: "alpine:latest".to_string(),
            network: false,
            memory_limit_mb: Some(512),
            cpu_limit: Some(1.0),
            cleanup: true,
        }
    }

    /// Set the Docker image to use
    pub fn with_image(mut self, image: String) -> Self {
        self.image = image;
        self
    }

    /// Enable network access
    pub fn with_network(mut self, enabled: bool) -> Self {
        self.network = enabled;
        self
    }

    /// Set memory limit
    pub fn with_memory_limit(mut self, mb: u64) -> Self {
        self.memory_limit_mb = Some(mb);
        self
    }

    /// Set CPU limit
    pub fn with_cpu_limit(mut self, cores: f64) -> Self {
        self.cpu_limit = Some(cores);
        self
    }

    /// Disable container cleanup (useful for debugging)
    pub fn no_cleanup(mut self) -> Self {
        self.cleanup = false;
        self
    }

    /// Check if Docker is installed and accessible
    async fn check_docker_available() -> bool {
        let output = Command::new("docker")
            .arg("--version")
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Pull Docker image if not available
    async fn ensure_image(&self) -> Result<(), SandboxError> {
        debug!("Checking if Docker image {} is available", self.image);

        let check_output = Command::new("docker")
            .args(&["image", "inspect", &self.image])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match check_output {
            Ok(status) if status.success() => {
                debug!("Image {} already available", self.image);
                Ok(())
            }
            _ => {
                debug!("Pulling Docker image {}", self.image);

                let pull_output = Command::new("docker")
                    .args(&["pull", &self.image])
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .status()
                    .map_err(|e| {
                        SandboxError::InitializationFailed(format!(
                            "Failed to pull Docker image: {}",
                            e
                        ))
                    })?;

                if !pull_output.success() {
                    return Err(SandboxError::InitializationFailed(format!(
                        "Failed to pull Docker image {}",
                        self.image
                    )));
                }

                Ok(())
            }
        }
    }

    /// Get the shell command for the given shell type
    fn get_shell_command(&self, shell: &ShellType) -> Vec<String> {
        match shell {
            ShellType::Bash => vec!["sh".to_string(), "-c".to_string()],
            ShellType::Zsh => vec!["sh".to_string(), "-c".to_string()], // Alpine uses sh
            ShellType::Fish => vec!["sh".to_string(), "-c".to_string()],
            ShellType::PowerShell | ShellType::Cmd => {
                vec!["sh".to_string(), "-c".to_string()] // Fallback to sh for Windows shells
            }
        }
    }

    /// Build combined command with setup and main command
    fn build_combined_command(&self, context: &ExecutionContext) -> String {
        let mut script = String::new();

        // Add setup commands
        for setup_cmd in &context.setup_commands {
            script.push_str(setup_cmd);
            script.push_str(" && ");
        }

        // Add main command
        script.push_str(&context.command);

        script
    }
}

impl Default for DockerSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Sandbox for DockerSandbox {
    async fn is_available(&self) -> bool {
        Self::check_docker_available().await
    }

    async fn execute(&self, context: ExecutionContext) -> Result<ExecutionOutput, SandboxError> {
        // Ensure Docker is available
        if !self.is_available().await {
            return Err(SandboxError::NotAvailable(
                "Docker is not installed or not accessible".to_string(),
            ));
        }

        // Ensure image is available
        self.ensure_image().await?;

        // Create temporary directory for volume mount
        let temp_dir = tempfile::tempdir().map_err(|e| {
            SandboxError::InitializationFailed(format!("Failed to create temp directory: {}", e))
        })?;

        let working_dir = temp_dir.path().to_path_buf();
        debug!("Created sandbox volume at: {:?}", working_dir);

        // Build Docker command
        let mut cmd = Command::new("docker");
        cmd.args(&["run", "--rm"]);

        // Set working directory mount
        cmd.arg("-v");
        cmd.arg(format!("{}:/workspace", working_dir.display()));
        cmd.arg("-w").arg("/workspace");

        // Network configuration
        if !self.network {
            cmd.arg("--network").arg("none");
        }

        // Resource limits
        if let Some(memory_mb) = self.memory_limit_mb {
            cmd.arg("--memory").arg(format!("{}m", memory_mb));
        }

        if let Some(cpu_limit) = self.cpu_limit {
            cmd.arg("--cpus").arg(cpu_limit.to_string());
        }

        // Environment variables
        for (key, value) in &context.env {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        // Image
        cmd.arg(&self.image);

        // Shell and command
        let shell_cmd = self.get_shell_command(&context.shell);
        for arg in &shell_cmd {
            cmd.arg(arg);
        }

        let combined_command = self.build_combined_command(&context);
        cmd.arg(&combined_command);

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        debug!(
            "Executing Docker command: docker run ... {} {} '{}'",
            self.image, shell_cmd[0], combined_command
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
                            "Docker command execution failed: {}",
                            e
                        )));
                    }
                },
                Err(e) => {
                    return Err(SandboxError::ExecutionFailed(format!(
                        "Docker task join failed: {}",
                        e
                    )));
                }
            },
            Err(_) => {
                warn!("Docker command timed out after {:?}", context.timeout);
                (
                    -1,
                    String::new(),
                    format!("Command timed out after {:?}", context.timeout),
                    true,
                )
            }
        };

        // List created and modified files in the working directory
        let created_files = self.list_files(&working_dir);
        let modified_files = Vec::new(); // Docker always starts fresh

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

        // Cleanup is automatic with --rm flag
        // Temp directory cleanup
        if let Err(e) = temp_dir.close() {
            warn!("Failed to cleanup temp directory: {}", e);
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "docker"
    }

    fn description(&self) -> &'static str {
        "Docker sandbox with full isolation"
    }
}

impl DockerSandbox {
    fn list_files(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                } else if path.is_dir() {
                    files.extend(self.list_files(&path));
                }
            }
        }

        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_docker_availability() {
        let sandbox = DockerSandbox::new();
        // This test will pass if Docker is installed, skip otherwise
        let available = sandbox.is_available().await;
        println!("Docker available: {}", available);
    }

    #[tokio::test]
    #[ignore] // Run only when Docker is available
    async fn test_docker_sandbox_basic() {
        let sandbox = DockerSandbox::new();

        if !sandbox.is_available().await {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let context = ExecutionContext {
            command: "echo 'hello from docker'".to_string(),
            shell: ShellType::Bash,
            working_dir: PathBuf::from("/workspace"),
            env: Default::default(),
            timeout: Duration::from_secs(10),
            setup_commands: vec![],
        };

        let result = sandbox.execute(context).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello from docker"));
        assert!(!result.timed_out);
    }

    #[tokio::test]
    #[ignore] // Run only when Docker is available
    async fn test_docker_sandbox_isolation() {
        let sandbox = DockerSandbox::new();

        if !sandbox.is_available().await {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let context = ExecutionContext {
            command: "touch test_file.txt && ls -la".to_string(),
            shell: ShellType::Bash,
            working_dir: PathBuf::from("/workspace"),
            env: Default::default(),
            timeout: Duration::from_secs(10),
            setup_commands: vec![],
        };

        let result = sandbox.execute(context).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(!result.created_files.is_empty());
    }
}
