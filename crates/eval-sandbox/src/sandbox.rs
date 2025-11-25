// Core sandbox trait and types

use async_trait::async_trait;
use eval_core::{SandboxConfig, ShellType};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Sandbox execution errors
#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Sandbox initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Cleanup failed: {0}")]
    CleanupFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Setup command failed: {0}")]
    SetupFailed(String),

    #[error("Sandbox not available: {0}")]
    NotAvailable(String),
}

/// Execution context for sandbox
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Command to execute
    pub command: String,

    /// Shell type
    pub shell: ShellType,

    /// Working directory (within sandbox)
    pub working_dir: PathBuf,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Timeout for execution
    pub timeout: Duration,

    /// Setup commands to run before main command
    pub setup_commands: Vec<String>,
}

impl ExecutionContext {
    pub fn new(command: String, shell: ShellType) -> Self {
        Self {
            command,
            shell,
            working_dir: PathBuf::from("/tmp/cmdai-test"),
            env: HashMap::new(),
            timeout: Duration::from_secs(5),
            setup_commands: Vec::new(),
        }
    }

    pub fn from_config(command: String, shell: ShellType, config: &SandboxConfig) -> Self {
        let mut ctx = Self::new(command, shell);
        ctx.timeout = Duration::from_millis(config.timeout_ms);
        ctx.env = config.env.clone();
        ctx.setup_commands = config.working_dir_setup.clone();
        ctx
    }
}

/// Output from sandbox execution
#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    /// Exit code
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution time
    pub execution_time: Duration,

    /// Working directory path (host filesystem)
    pub working_dir: PathBuf,

    /// Files created during execution
    pub created_files: Vec<PathBuf>,

    /// Files modified during execution
    pub modified_files: Vec<PathBuf>,

    /// Whether execution timed out
    pub timed_out: bool,
}

/// Sandbox trait for executing commands in isolated environments
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Check if this sandbox backend is available on the current system
    async fn is_available(&self) -> bool;

    /// Execute a command in the sandbox
    async fn execute(&self, context: ExecutionContext) -> Result<ExecutionOutput, SandboxError>;

    /// Get the name of this sandbox backend
    fn name(&self) -> &'static str;

    /// Get a description of this sandbox backend
    fn description(&self) -> &'static str;
}
