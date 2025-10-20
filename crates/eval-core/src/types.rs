// Core types for evaluation framework

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Shell type for command generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellType {
    #[serde(rename = "bash")]
    Bash,
    #[serde(rename = "zsh")]
    Zsh,
    #[serde(rename = "fish")]
    Fish,
    #[serde(rename = "powershell")]
    PowerShell,
    #[serde(rename = "cmd")]
    Cmd,
}

impl Default for ShellType {
    fn default() -> Self {
        ShellType::Bash
    }
}

/// Difficulty level for test cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "intermediate")]
    Intermediate,
    #[serde(rename = "advanced")]
    Advanced,
    #[serde(rename = "expert")]
    Expert,
}

/// Safety level for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    #[serde(rename = "safe")]
    Safe,
    #[serde(rename = "moderate")]
    Moderate,
    #[serde(rename = "dangerous")]
    Dangerous,
}

/// Sandbox backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxBackend {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "docker")]
    Docker,
    #[serde(rename = "firejail")]
    Firejail,
}

impl Default for SandboxBackend {
    fn default() -> Self {
        SandboxBackend::Local
    }
}

/// Sandbox configuration for test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SandboxConfig {
    /// Backend to use for sandboxing
    pub backend: SandboxBackend,

    /// Commands to run before test execution (setup)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub working_dir_setup: Vec<String>,

    /// Environment variables to set
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Timeout in milliseconds
    pub timeout_ms: u64,

    /// Docker-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerConfig>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            backend: SandboxBackend::Local,
            working_dir_setup: Vec::new(),
            env: HashMap::new(),
            timeout_ms: 5000,
            docker: None,
        }
    }
}

/// Docker-specific sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerConfig {
    /// Docker image to use
    pub image: String,

    /// Whether to enable network access
    pub network: bool,

    /// Memory limit in megabytes
    pub memory_limit_mb: Option<u64>,

    /// CPU limit (number of cores)
    pub cpu_limit: Option<f64>,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            image: "alpine:latest".to_string(),
            network: false,
            memory_limit_mb: Some(512),
            cpu_limit: Some(1.0),
        }
    }
}

/// Command-string assertion configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CommandStringAssertions {
    /// Patterns that must NOT appear in the command
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub denylist: Vec<String>,

    /// Patterns that MUST appear in the command
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowlist: Vec<String>,

    /// Flags that must be present
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required_flags: Vec<String>,

    /// Maximum command length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Minimum command length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
}

/// Runtime assertion configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RuntimeAssertions {
    /// Allowed exit codes (empty means any code is allowed)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed_exit_codes: Vec<i32>,

    /// Regex pattern that stdout must match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_regex: Option<String>,

    /// Regex pattern that stderr must match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_regex: Option<String>,

    /// Whether stdout must be empty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_empty: Option<bool>,

    /// Whether stderr must be empty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_empty: Option<bool>,

    /// File expectations
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub expected_files: Vec<FileExpectation>,

    /// Directories where writes are not allowed
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub no_writes_outside: Vec<String>,

    /// Maximum execution time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_execution_time_ms: Option<u64>,
}

/// File existence expectation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExpectation {
    /// Path to the file (relative to working directory)
    pub path: String,

    /// Whether the file should exist
    pub should_exist: bool,

    /// Optional content regex to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_regex: Option<String>,

    /// Optional minimum file size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_size: Option<u64>,

    /// Optional maximum file size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<u64>,
}

/// Combined assertion configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AssertionConfig {
    /// Command-string level assertions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_string: Option<CommandStringAssertions>,

    /// Runtime assertions (require sandbox execution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeAssertions>,
}
