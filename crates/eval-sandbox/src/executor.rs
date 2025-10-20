// High-level sandbox executor

use eval_core::{SandboxBackend, SandboxConfig, ShellType};
use std::sync::Arc;
use tracing::debug;

use crate::docker::DockerSandbox;
use crate::local::LocalSandbox;
use crate::sandbox::{ExecutionContext, ExecutionOutput, Sandbox, SandboxError};

/// Unified sandbox executor that selects the appropriate backend
pub struct SandboxExecutor {
    local: Arc<LocalSandbox>,
    docker: Arc<DockerSandbox>,
}

impl SandboxExecutor {
    /// Create a new sandbox executor with default backends
    pub fn new() -> Self {
        Self {
            local: Arc::new(LocalSandbox::new()),
            docker: Arc::new(DockerSandbox::new()),
        }
    }

    /// Create an executor with custom backends
    pub fn with_backends(local: LocalSandbox, docker: DockerSandbox) -> Self {
        Self {
            local: Arc::new(local),
            docker: Arc::new(docker),
        }
    }

    /// Execute a command using the specified sandbox configuration
    pub async fn execute(
        &self,
        command: String,
        shell: ShellType,
        config: &SandboxConfig,
    ) -> Result<ExecutionOutput, SandboxError> {
        let context = ExecutionContext::from_config(command, shell, config);

        let backend: Arc<dyn Sandbox> = match config.backend {
            SandboxBackend::Local => self.local.clone(),
            SandboxBackend::Docker => {
                // Check if Docker is available, fallback to local if not
                if self.docker.is_available().await {
                    self.docker.clone()
                } else {
                    debug!("Docker not available, falling back to local sandbox");
                    self.local.clone()
                }
            }
            SandboxBackend::Firejail => {
                // Firejail not implemented yet, fallback to local
                debug!("Firejail not implemented, falling back to local sandbox");
                self.local.clone()
            }
        };

        debug!("Using sandbox backend: {}", backend.name());
        backend.execute(context).await
    }

    /// Execute a simple command with default configuration
    pub async fn execute_simple(
        &self,
        command: String,
        shell: ShellType,
    ) -> Result<ExecutionOutput, SandboxError> {
        let config = SandboxConfig::default();
        self.execute(command, shell, &config).await
    }

    /// Get the appropriate sandbox backend
    pub async fn get_backend(
        &self,
        backend_type: SandboxBackend,
    ) -> Arc<dyn Sandbox> {
        match backend_type {
            SandboxBackend::Local => self.local.clone(),
            SandboxBackend::Docker => {
                if self.docker.is_available().await {
                    self.docker.clone()
                } else {
                    debug!("Docker not available, using local sandbox");
                    self.local.clone()
                }
            }
            SandboxBackend::Firejail => {
                debug!("Firejail not implemented, using local sandbox");
                self.local.clone()
            }
        }
    }

    /// Check if a specific backend is available
    pub async fn is_backend_available(&self, backend: SandboxBackend) -> bool {
        match backend {
            SandboxBackend::Local => self.local.is_available().await,
            SandboxBackend::Docker => self.docker.is_available().await,
            SandboxBackend::Firejail => false, // Not implemented
        }
    }
}

impl Default for SandboxExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_executor_local_backend() {
        let executor = SandboxExecutor::new();

        let mut config = SandboxConfig::default();
        config.backend = SandboxBackend::Local;

        let result = executor
            .execute("echo 'test'".to_string(), ShellType::Bash, &config)
            .await
            .unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("test"));
    }

    #[tokio::test]
    async fn test_executor_backend_availability() {
        let executor = SandboxExecutor::new();

        assert!(executor.is_backend_available(SandboxBackend::Local).await);

        // Docker availability depends on system
        let docker_available = executor.is_backend_available(SandboxBackend::Docker).await;
        println!("Docker available: {}", docker_available);

        // Firejail not implemented
        assert!(!executor.is_backend_available(SandboxBackend::Firejail).await);
    }

    #[tokio::test]
    async fn test_executor_simple() {
        let executor = SandboxExecutor::new();

        let result = executor
            .execute_simple("pwd".to_string(), ShellType::Bash)
            .await
            .unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(!result.stdout.is_empty());
    }
}
