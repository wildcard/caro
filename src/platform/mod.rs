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

/// BSD-family OS flavor.
///
/// Identifies the underlying OS lineage independently of `UtilityType`.
/// macOS with Homebrew GNU coreutils stays `BsdFlavor::MacOs` even though
/// its capability profile flips to `UtilityType::Gnu` — the kernel family
/// is still Darwin and BSD-derived utilities (chflags, newfs, sysctl)
/// remain available.
///
/// Inspired by the FDD-book ch. 29 (Portability and Driver Abstraction):
/// portability decisions hinge on the kernel family, not the userland
/// veneer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BsdFlavor {
    /// FreeBSD (kern.ostype = "FreeBSD")
    FreeBsd,
    /// OpenBSD
    OpenBsd,
    /// NetBSD
    NetBsd,
    /// macOS / Darwin
    MacOs,
    /// DragonFly BSD
    DragonFlyBsd,
    /// Detected as BSD-family but the specific flavor was not recognized
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
    /// BSD-family flavor (None for non-BSD operating systems).
    /// Orthogonal to `utility_type` — see `BsdFlavor` docs.
    bsd_flavor: Option<BsdFlavor>,
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
        let bsd_flavor = detect_bsd_flavor(&os).await;

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
            bsd_flavor,
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

    /// BSD-family flavor (FreeBSD/OpenBSD/NetBSD/macOS/DragonFly).
    /// Returns `None` for non-BSD operating systems (Linux, Windows, etc.).
    pub fn bsd_flavor(&self) -> Option<BsdFlavor> {
        self.bsd_flavor
    }

    /// True if the OS is in the BSD lineage. Decoupled from userland —
    /// macOS with Homebrew GNU coreutils still reports `true` here.
    pub fn is_bsd_family(&self) -> bool {
        self.bsd_flavor.is_some()
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

        // BSD-flavor-specific guidance for the LLM. These notes are scoped
        // to features that exist only on the named flavor — they do not
        // duplicate the generic BSD-utility notes above.
        if let Some(flavor) = self.bsd_flavor {
            match flavor {
                BsdFlavor::FreeBsd => {
                    notes.push(
                        "FreeBSD: package manager is `pkg` (pkg install/info/delete)".to_string(),
                    );
                    notes.push(
                        "FreeBSD: containers via `jail` / `jls` (not Docker by default)"
                            .to_string(),
                    );
                    notes.push(
                        "FreeBSD: filesystem ops use `gpart`/`newfs`; ZFS is first-class"
                            .to_string(),
                    );
                }
                BsdFlavor::OpenBsd => {
                    notes.push("OpenBSD: package manager is `pkg_add`/`pkg_delete`".to_string());
                    notes.push(
                        "OpenBSD: firewall is `pf` (not iptables); config in /etc/pf.conf"
                            .to_string(),
                    );
                    notes.push(
                        "OpenBSD: prefer `doas` over `sudo` (not installed by default)".to_string(),
                    );
                }
                BsdFlavor::NetBsd => {
                    notes.push(
                        "NetBSD: package manager is `pkgsrc` (pkg_add/pkg_info/pkg_delete)"
                            .to_string(),
                    );
                    notes.push(
                        "NetBSD: rc.d service management via /etc/rc.d/<service>".to_string(),
                    );
                }
                BsdFlavor::MacOs => {
                    // macOS-specific notes already covered above by the
                    // `os == "macos"` branch; nothing extra to add here.
                }
                BsdFlavor::DragonFlyBsd => {
                    notes.push(
                        "DragonFly BSD: HAMMER2 filesystem; package manager is `pkg`".to_string(),
                    );
                }
                BsdFlavor::Unknown => {
                    notes.push("BSD-family OS detected but flavor unrecognized".to_string());
                }
            }
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

        if let Some(flavor) = self.bsd_flavor {
            prompt.push_str(&format!("\nBSD flavor: {:?}", flavor));
        }

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
    bsd_flavor: Option<BsdFlavor>,
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
            bsd_flavor: None,
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

    /// Set the BSD-family flavor explicitly. Useful in tests and in
    /// non-async code paths where uname-based detection is not available.
    pub fn bsd_flavor(mut self, flavor: BsdFlavor) -> Self {
        self.bsd_flavor = Some(flavor);
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

        // If the caller didn't set bsd_flavor explicitly, infer it from the
        // OS string for known BSD-family identifiers. This keeps existing
        // callers working without forcing every test to set the flavor.
        let bsd_flavor = self.bsd_flavor.or_else(|| infer_bsd_flavor_from_os(&os));

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
            bsd_flavor,
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
    } else if cfg!(target_os = "freebsd") {
        "freebsd".to_string()
    } else if cfg!(target_os = "openbsd") {
        "openbsd".to_string()
    } else if cfg!(target_os = "netbsd") {
        "netbsd".to_string()
    } else if cfg!(target_os = "dragonfly") {
        "dragonfly".to_string()
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
        // ver is a shell builtin, must run through cmd
        if let Ok(output) =
            run_command_with_timeout("cmd", &["/c", "ver"], Duration::from_secs(1)).await
        {
            let trimmed = output.trim();
            // Extract version number from output like "Microsoft Windows [Version 10.0.20348.2340]"
            if let Some(start) = trimmed.find('[') {
                if let Some(end) = trimmed.find(']') {
                    let version_part = &trimmed[start + 1..end];
                    if let Some(version) = version_part.strip_prefix("Version ") {
                        return Ok(version.to_string());
                    }
                }
            }
            // If parsing fails, return the whole output
            return Ok(trimmed.to_string());
        }

        // Fallback: Try wmic (older Windows)
        if let Ok(output) = run_command_with_timeout(
            "wmic",
            &["os", "get", "Version", "/value"],
            Duration::from_secs(2),
        )
        .await
        {
            for line in output.lines() {
                if let Some(version) = line.strip_prefix("Version=") {
                    return Ok(version.trim().to_string());
                }
            }
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
    matches!(
        os,
        "macos" | "linux" | "freebsd" | "openbsd" | "netbsd" | "dragonfly"
    )
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

/// Map the canonical OS string (`os`) to a `BsdFlavor`. Returns `None` for
/// non-BSD operating systems. Used by the builder when no flavor is
/// explicitly supplied.
fn infer_bsd_flavor_from_os(os: &str) -> Option<BsdFlavor> {
    match os {
        "macos" | "darwin" => Some(BsdFlavor::MacOs),
        "freebsd" => Some(BsdFlavor::FreeBsd),
        "openbsd" => Some(BsdFlavor::OpenBsd),
        "netbsd" => Some(BsdFlavor::NetBsd),
        "dragonfly" | "dragonflybsd" => Some(BsdFlavor::DragonFlyBsd),
        _ => None,
    }
}

/// Detect the BSD flavor for the current host.
///
/// On non-BSD targets (`linux`, `windows`, `unknown`) returns `None`
/// without touching the filesystem or running any commands. On BSD-family
/// targets (`macos` and the BSD compile-target gate) returns a flavor
/// derived from the OS string. We deliberately avoid shelling out to
/// `uname -s` on hot paths: the compile-time `cfg!` answer is already
/// authoritative for binaries built on that target, and `os` here is
/// driven by the same `cfg!` checks.
async fn detect_bsd_flavor(os: &str) -> Option<BsdFlavor> {
    infer_bsd_flavor_from_os(os)
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
        assert!(
            [
                "macos",
                "linux",
                "windows",
                "freebsd",
                "openbsd",
                "netbsd",
                "dragonfly",
            ]
            .contains(&os.as_str()),
            "Unrecognized OS string from detect_os(): {}",
            os
        );
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
