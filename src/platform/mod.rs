//! Platform detection and context for command validation
//!
//! This module provides enhanced platform detection including:
//! - OS and version information
//! - Shell type and version
//! - Architecture details
//! - Utility availability (GNU vs BSD coreutils)
//! - Platform-specific notes for command generation

use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

/// Utility type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityType {
    /// GNU coreutils (Linux standard)
    Gnu,
    /// BSD utilities (macOS, BSD)
    Bsd,
    /// Busybox (embedded systems)
    Busybox,
    /// Unknown or mixed
    Unknown,
}

/// Enhanced platform context for command validation
#[derive(Debug, Clone)]
pub struct PlatformContext {
    os: String,
    os_version: String,
    arch: String,
    shell: String,
    shell_version: String,
    posix_compliant: bool,
    has_gnu_coreutils: bool,
    has_bsd_utils: bool,
    available_tools: HashMap<String, String>,
    utility_type: UtilityType,
}

impl PlatformContext {
    /// Detect platform context from the current environment
    pub async fn detect() -> Result<Self, PlatformContextError> {
        let os = detect_os();
        let os_version = detect_os_version().await?;
        let arch = detect_arch();
        let shell = detect_shell();
        let shell_version = detect_shell_version(&shell).await;
        let posix_compliant = is_posix_compliant(&os);

        // Detect utilities in parallel
        let (has_gnu, has_bsd, available_tools) = tokio::join!(
            detect_gnu_coreutils(),
            detect_bsd_utils(),
            detect_available_tools()
        );

        let utility_type = determine_utility_type(has_gnu, has_bsd);

        Ok(Self {
            os,
            os_version,
            arch,
            shell,
            shell_version,
            posix_compliant,
            has_gnu_coreutils: has_gnu,
            has_bsd_utils: has_bsd,
            available_tools,
            utility_type,
        })
    }

    /// Create a builder for custom platform context
    pub fn builder() -> PlatformContextBuilder {
        PlatformContextBuilder::new()
    }

    // Getters
    pub fn os(&self) -> &str {
        &self.os
    }

    pub fn os_version(&self) -> &str {
        &self.os_version
    }

    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn shell(&self) -> &str {
        &self.shell
    }

    pub fn shell_version(&self) -> &str {
        &self.shell_version
    }

    pub fn is_posix_compliant(&self) -> bool {
        self.posix_compliant
    }

    pub fn has_gnu_coreutils(&self) -> bool {
        self.has_gnu_coreutils
    }

    pub fn has_bsd_utils(&self) -> bool {
        self.has_bsd_utils
    }

    pub fn available_tools(&self) -> &HashMap<String, String> {
        &self.available_tools
    }

    pub fn utility_type(&self) -> UtilityType {
        self.utility_type
    }

    /// Generate platform-specific notes for LLM prompt
    pub fn platform_notes(&self) -> Vec<String> {
        let mut notes = Vec::new();

        if self.os == "macos" && self.has_bsd_utils {
            notes.push("macOS uses BSD utilities with different flags than GNU".to_string());
            notes.push("netstat: use -an (not -ano)".to_string());
            notes.push("sed: use -i '' (not -i) for in-place edits".to_string());
            notes.push("date: BSD format specifiers differ from GNU".to_string());
        }

        if self.os == "linux" && self.has_gnu_coreutils {
            notes.push("Linux uses GNU coreutils".to_string());
            notes.push("Commands support long options (--help)".to_string());
        }

        if self.utility_type == UtilityType::Busybox {
            notes.push("Using Busybox utilities (limited feature set)".to_string());
        }

        notes
    }

    /// Convert platform context to prompt string for LLM
    pub fn to_prompt_string(&self) -> String {
        let mut prompt = format!(
            "OS: {} {}\nArchitecture: {}\nShell: {}",
            self.os, self.os_version, self.arch, self.shell
        );

        if !self.shell_version.is_empty() {
            prompt.push_str(&format!(" {}", self.shell_version));
        }

        prompt.push_str(&format!("\nUtilities: {:?}", self.utility_type));

        if !self.platform_notes().is_empty() {
            prompt.push_str("\n\nPlatform-specific notes:\n");
            for note in self.platform_notes() {
                prompt.push_str(&format!("- {}\n", note));
            }
        }

        prompt
    }
}

/// Builder for PlatformContext
pub struct PlatformContextBuilder {
    os: Option<String>,
    os_version: Option<String>,
    arch: Option<String>,
    shell: Option<String>,
    shell_version: Option<String>,
    posix_compliant: Option<bool>,
    has_gnu_coreutils: Option<bool>,
    has_bsd_utils: Option<bool>,
    available_tools: Option<HashMap<String, String>>,
}

impl PlatformContextBuilder {
    pub fn new() -> Self {
        Self {
            os: None,
            os_version: None,
            arch: None,
            shell: None,
            shell_version: None,
            posix_compliant: None,
            has_gnu_coreutils: None,
            has_bsd_utils: None,
            available_tools: None,
        }
    }

    pub fn os(mut self, os: impl Into<String>) -> Self {
        self.os = Some(os.into());
        self
    }

    pub fn os_version(mut self, version: impl Into<String>) -> Self {
        self.os_version = Some(version.into());
        self
    }

    pub fn arch(mut self, arch: impl Into<String>) -> Self {
        self.arch = Some(arch.into());
        self
    }

    pub fn shell(mut self, shell: impl Into<String>) -> Self {
        self.shell = Some(shell.into());
        self
    }

    pub fn shell_version(mut self, version: impl Into<String>) -> Self {
        self.shell_version = Some(version.into());
        self
    }

    pub fn posix_compliant(mut self, posix: bool) -> Self {
        self.posix_compliant = Some(posix);
        self
    }

    pub fn has_gnu_coreutils(mut self, has_gnu: bool) -> Self {
        self.has_gnu_coreutils = Some(has_gnu);
        self
    }

    pub fn has_bsd_utils(mut self, has_bsd: bool) -> Self {
        self.has_bsd_utils = Some(has_bsd);
        self
    }

    pub fn available_tools(mut self, tools: HashMap<String, String>) -> Self {
        self.available_tools = Some(tools);
        self
    }

    pub fn build(self) -> Result<PlatformContext, PlatformContextError> {
        let os = self
            .os
            .ok_or_else(|| PlatformContextError::MissingField("os".to_string()))?;
        let arch = self
            .arch
            .ok_or_else(|| PlatformContextError::MissingField("arch".to_string()))?;
        let shell = self
            .shell
            .ok_or_else(|| PlatformContextError::MissingField("shell".to_string()))?;

        if os.is_empty() {
            return Err(PlatformContextError::EmptyField("os".to_string()));
        }
        if arch.is_empty() {
            return Err(PlatformContextError::EmptyField("arch".to_string()));
        }
        if shell.is_empty() {
            return Err(PlatformContextError::EmptyField("shell".to_string()));
        }

        let has_gnu = self.has_gnu_coreutils.unwrap_or(false);
        let has_bsd = self.has_bsd_utils.unwrap_or(false);
        let utility_type = determine_utility_type(has_gnu, has_bsd);
        let posix_compliant = self
            .posix_compliant
            .unwrap_or_else(|| is_posix_compliant(&os));

        Ok(PlatformContext {
            os,
            os_version: self.os_version.unwrap_or_default(),
            arch,
            shell,
            shell_version: self.shell_version.unwrap_or_default(),
            posix_compliant,
            has_gnu_coreutils: has_gnu,
            has_bsd_utils: has_bsd,
            available_tools: self.available_tools.unwrap_or_default(),
            utility_type,
        })
    }
}

impl Default for PlatformContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors related to platform context detection
#[derive(Debug, thiserror::Error)]
pub enum PlatformContextError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Empty field: {0}")]
    EmptyField(String),

    #[error("Detection failed: {0}")]
    DetectionFailed(String),

    #[error("Command execution error: {0}")]
    CommandError(String),

    #[error("Timeout during detection")]
    Timeout,
}

// Detection functions

fn detect_os() -> String {
    if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        "unknown".to_string()
    }
}

async fn detect_os_version() -> Result<String, PlatformContextError> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) =
            run_command_with_timeout("sw_vers", &["-productVersion"], Duration::from_secs(1)).await
        {
            return Ok(output.trim().to_string());
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try /etc/os-release first
        if let Ok(contents) = tokio::fs::read_to_string("/etc/os-release").await {
            for line in contents.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    let version = line
                        .trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"')
                        .to_string();
                    return Ok(version);
                }
            }
        }

        // Fallback to uname
        if let Ok(output) = run_command_with_timeout("uname", &["-r"], Duration::from_secs(1)).await
        {
            return Ok(output.trim().to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = run_command_with_timeout("ver", &[], Duration::from_secs(1)).await {
            return Ok(output.trim().to_string());
        }
    }

    Ok("unknown".to_string())
}

fn detect_arch() -> String {
    std::env::consts::ARCH.to_string()
}

fn detect_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if let Some(name) = shell.split('/').next_back() {
            return name.to_string();
        }
    }

    #[cfg(target_os = "windows")]
    {
        if std::env::var("PSModulePath").is_ok() {
            return "powershell".to_string();
        }
        return "cmd".to_string();
    }

    "sh".to_string()
}

async fn detect_shell_version(shell: &str) -> String {
    let result = match shell {
        "bash" => run_command_with_timeout("bash", &["--version"], Duration::from_secs(1)).await,
        "zsh" => run_command_with_timeout("zsh", &["--version"], Duration::from_secs(1)).await,
        "fish" => run_command_with_timeout("fish", &["--version"], Duration::from_secs(1)).await,
        _ => Err(PlatformContextError::DetectionFailed(
            "Unsupported shell".to_string(),
        )),
    };

    if let Ok(output) = result {
        // Parse first line and extract version number
        if let Some(first_line) = output.lines().next() {
            // Extract version pattern (e.g., "5.1.16" or "3.1.2")
            for word in first_line.split_whitespace() {
                if word.chars().next().is_some_and(|c| c.is_numeric()) {
                    return word.to_string();
                }
            }
        }
    }

    String::new()
}

fn is_posix_compliant(os: &str) -> bool {
    matches!(os, "macos" | "linux")
}

async fn detect_gnu_coreutils() -> bool {
    // Try to run `ls --version` - GNU coreutils respond with version info
    if let Ok(output) =
        run_command_with_timeout("ls", &["--version"], Duration::from_millis(500)).await
    {
        output.to_lowercase().contains("gnu")
    } else {
        false
    }
}

async fn detect_bsd_utils() -> bool {
    // BSD utils typically don't support --version
    if (run_command_with_timeout("ls", &["--version"], Duration::from_millis(500)).await).is_ok() {
        false // If --version works, it's likely GNU
    } else {
        // BSD utils will fail on --version, check if ls exists normally
        run_command_with_timeout("ls", &["-d", "."], Duration::from_millis(500))
            .await
            .is_ok()
    }
}

async fn detect_available_tools() -> HashMap<String, String> {
    let mut tools = HashMap::new();

    let common_utils = [
        "ls", "cat", "grep", "find", "sed", "awk", "sort", "uniq", "wc", "head", "tail", "cut",
        "tr", "chmod", "chown", "ps", "netstat", "df", "du", "tar", "gzip", "curl", "wget",
    ];

    // Check each utility in parallel
    let futures: Vec<_> = common_utils
        .iter()
        .map(|util| async move {
            let version = detect_tool_version(util).await;
            (*util, version)
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    for (util, version) in results {
        if !version.is_empty() || tool_exists(util).await {
            tools.insert(util.to_string(), version);
        }
    }

    tools
}

async fn detect_tool_version(tool: &str) -> String {
    // Try common version flags
    for flag in &["--version", "-v", "-V", "version"] {
        if let Ok(output) =
            run_command_with_timeout(tool, &[flag], Duration::from_millis(500)).await
        {
            // Extract version from first line
            if let Some(first_line) = output.lines().next() {
                return first_line.trim().to_string();
            }
        }
    }

    String::new()
}

async fn tool_exists(tool: &str) -> bool {
    run_command_with_timeout("which", &[tool], Duration::from_millis(500))
        .await
        .is_ok()
}

fn determine_utility_type(has_gnu: bool, has_bsd: bool) -> UtilityType {
    match (has_gnu, has_bsd) {
        (true, false) => UtilityType::Gnu,
        (false, true) => UtilityType::Bsd,
        (true, true) => UtilityType::Gnu, // Prefer GNU if both
        (false, false) => UtilityType::Unknown,
    }
}

async fn run_command_with_timeout(
    cmd: &str,
    args: &[&str],
    duration: Duration,
) -> Result<String, PlatformContextError> {
    let future = tokio::task::spawn_blocking({
        let cmd = cmd.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        move || {
            Command::new(&cmd)
                .args(&args)
                .output()
                .map_err(|e| PlatformContextError::CommandError(e.to_string()))
        }
    });

    let result = timeout(duration, future)
        .await
        .map_err(|_| PlatformContextError::Timeout)?
        .map_err(|e| PlatformContextError::CommandError(e.to_string()))??;

    if result.status.success() {
        Ok(String::from_utf8_lossy(&result.stdout).to_string())
    } else {
        Err(PlatformContextError::CommandError(format!(
            "Command {} failed with status {}",
            cmd, result.status
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_os() {
        let os = detect_os();
        assert!(["macos", "linux", "windows"].contains(&os.as_str()));
    }

    #[tokio::test]
    async fn test_detect_arch() {
        let arch = detect_arch();
        assert!(!arch.is_empty());
    }

    #[tokio::test]
    async fn test_detect_shell() {
        let shell = detect_shell();
        assert!(!shell.is_empty());
    }

    #[test]
    fn test_builder() {
        let ctx = PlatformContext::builder()
            .os("linux")
            .arch("x86_64")
            .shell("bash")
            .build()
            .unwrap();

        assert_eq!(ctx.os(), "linux");
        assert_eq!(ctx.arch(), "x86_64");
        assert_eq!(ctx.shell(), "bash");
    }
}
