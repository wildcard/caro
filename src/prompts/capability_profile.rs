//! Capability Profile Detection for Shell Command Generation
//!
//! This module provides comprehensive platform capability detection to generate
//! optimal shell commands for each user's environment. The detected capabilities
//! are used to configure the SmolLM prompt template with appropriate tool flags
//! and command patterns.
//!
//! # Supported Profiles
//!
//! - `gnu-linux`: GNU coreutils + findutils (Ubuntu, Debian, Fedora, RHEL, Arch)
//! - `bsd`: BSD utilities (macOS, FreeBSD, OpenBSD)
//! - `busybox`: BusyBox utilities (Alpine, embedded systems)
//! - `hybrid`: Mixed environments (Git Bash, MSYS2, Cygwin)
//!
//! # Example
//!
//! ```rust
//! use caro::prompts::capability_profile::CapabilityProfile;
//!
//! #[tokio::main]
//! async fn main() {
//!     let profile = CapabilityProfile::detect().await;
//!     println!("Profile: {:?}", profile.profile_type);
//!     println!("find -printf support: {}", profile.find_printf);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

/// Profile type classification based on detected userland
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileType {
    /// GNU coreutils + GNU findutils (Linux standard)
    GnuLinux,
    /// BSD utilities (macOS, BSDs)
    Bsd,
    /// BusyBox utilities (Alpine, embedded)
    Busybox,
    /// Mixed/hybrid environments (Git Bash, MSYS2, Cygwin)
    Hybrid,
    /// Unknown profile
    Unknown,
}

impl std::fmt::Display for ProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileType::GnuLinux => write!(f, "gnu-linux"),
            ProfileType::Bsd => write!(f, "bsd"),
            ProfileType::Busybox => write!(f, "busybox"),
            ProfileType::Hybrid => write!(f, "hybrid"),
            ProfileType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Detected shell type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectedShell {
    Bash,
    Zsh,
    Fish,
    Dash,
    Sh,
    Ash,
    PowerShell,
    Cmd,
    Unknown(String),
}

impl std::fmt::Display for DetectedShell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectedShell::Bash => write!(f, "bash"),
            DetectedShell::Zsh => write!(f, "zsh"),
            DetectedShell::Fish => write!(f, "fish"),
            DetectedShell::Dash => write!(f, "dash"),
            DetectedShell::Sh => write!(f, "sh"),
            DetectedShell::Ash => write!(f, "ash"),
            DetectedShell::PowerShell => write!(f, "powershell"),
            DetectedShell::Cmd => write!(f, "cmd"),
            DetectedShell::Unknown(s) => write!(f, "{}", s),
        }
    }
}

/// Comprehensive capability profile for the current system
///
/// This structure contains all detected capabilities that affect shell command
/// generation. It is used by the SmolLM prompt system to select appropriate
/// command templates and flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProfile {
    /// Profile type classification
    pub profile_type: ProfileType,

    /// Operating system name (e.g., "Ubuntu", "macOS", "Alpine Linux")
    pub os_name: String,

    /// OS version string
    pub os_version: String,

    /// Full uname output
    pub uname: String,

    /// Path to /bin/sh (or equivalent)
    pub shell_sh: String,

    /// Detected default shell
    pub detected_shell: DetectedShell,

    /// Available tools as comma-separated list
    pub tools: Vec<String>,

    // Feature flags - these determine which command patterns can be used
    /// Whether `find -printf` is supported (GNU findutils)
    pub find_printf: bool,

    /// Whether `find -print0` is supported
    pub find_print0: bool,

    /// Whether `sort -h` (human-numeric sort) is supported
    pub sort_h: bool,

    /// Whether `xargs -0` (null-delimited input) is supported
    pub xargs_0: bool,

    /// Whether `grep -R` (recursive grep) is supported
    pub grep_r: bool,

    /// Whether `grep -P` (Perl regex) is supported
    pub grep_p: bool,

    /// Stat format type: "gnu" for `stat -c`, "bsd" for `stat -f`, "none" if unsupported
    pub stat_format: StatFormat,

    /// Whether `sed -i` works without argument (GNU) or needs '' (BSD)
    pub sed_inplace_gnu: bool,

    /// Whether `du --max-depth` is supported (GNU) vs `-d` (BSD)
    pub du_max_depth: bool,

    /// Whether `date --date` is supported (GNU) vs `-v` (BSD)
    pub date_gnu_format: bool,

    /// Whether `readlink -f` is supported
    pub readlink_f: bool,

    /// Whether `ps --sort` is supported
    pub ps_sort: bool,

    /// Whether `ls --sort` is supported
    pub ls_sort: bool,

    /// Whether `awk` is mawk (fast), gawk (GNU), or nawk (BSD)
    pub awk_type: AwkType,

    /// Extended tool metadata
    pub tool_versions: HashMap<String, String>,
}

/// Format type for stat command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatFormat {
    /// GNU stat with `-c` format
    Gnu,
    /// BSD stat with `-f` format
    Bsd,
    /// No stat format support
    None,
}

impl std::fmt::Display for StatFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatFormat::Gnu => write!(f, "gnu"),
            StatFormat::Bsd => write!(f, "bsd"),
            StatFormat::None => write!(f, "none"),
        }
    }
}

/// Type of awk implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AwkType {
    /// GNU awk
    Gawk,
    /// Fast awk (common on Ubuntu/Debian)
    Mawk,
    /// Traditional awk (BSD)
    Nawk,
    /// BusyBox awk
    BusyboxAwk,
    /// Unknown awk
    Unknown,
}

impl Default for CapabilityProfile {
    fn default() -> Self {
        Self {
            profile_type: ProfileType::Unknown,
            os_name: "unknown".to_string(),
            os_version: "unknown".to_string(),
            uname: "unknown".to_string(),
            shell_sh: "/bin/sh".to_string(),
            detected_shell: DetectedShell::Sh,
            tools: Vec::new(),
            find_printf: false,
            find_print0: false,
            sort_h: false,
            xargs_0: false,
            grep_r: false,
            grep_p: false,
            stat_format: StatFormat::None,
            sed_inplace_gnu: false,
            du_max_depth: false,
            date_gnu_format: false,
            readlink_f: false,
            ps_sort: false,
            ls_sort: false,
            awk_type: AwkType::Unknown,
            tool_versions: HashMap::new(),
        }
    }
}

impl CapabilityProfile {
    /// Detect capabilities from the current environment
    pub async fn detect() -> Self {
        let mut profile = CapabilityProfile::default();

        // Detect OS identity
        profile.detect_os_identity().await;

        // Detect shell
        profile.detect_shell().await;

        // Detect available tools
        profile.detect_tools().await;

        // Detect feature flags in parallel
        profile.detect_features().await;

        // Determine profile type
        profile.determine_profile_type().await;

        profile
    }

    /// Create a profile for a specific platform (useful for testing or cross-compilation)
    pub fn for_platform(platform: ProfileType) -> Self {
        let mut profile = CapabilityProfile {
            profile_type: platform,
            ..Default::default()
        };

        match platform {
            ProfileType::GnuLinux => {
                profile.os_name = "Ubuntu".to_string();
                profile.find_printf = true;
                profile.find_print0 = true;
                profile.sort_h = true;
                profile.xargs_0 = true;
                profile.grep_r = true;
                profile.grep_p = true;
                profile.stat_format = StatFormat::Gnu;
                profile.sed_inplace_gnu = true;
                profile.du_max_depth = true;
                profile.date_gnu_format = true;
                profile.readlink_f = true;
                profile.ps_sort = true;
                profile.ls_sort = true;
                profile.awk_type = AwkType::Gawk;
            }
            ProfileType::Bsd => {
                profile.os_name = "macOS".to_string();
                profile.find_printf = false;
                profile.find_print0 = true;
                profile.sort_h = false;
                profile.xargs_0 = true;
                profile.grep_r = true;
                profile.grep_p = false;
                profile.stat_format = StatFormat::Bsd;
                profile.sed_inplace_gnu = false;
                profile.du_max_depth = false;
                profile.date_gnu_format = false;
                profile.readlink_f = false;
                profile.ps_sort = false;
                profile.ls_sort = false;
                profile.awk_type = AwkType::Nawk;
            }
            ProfileType::Busybox => {
                profile.os_name = "Alpine Linux".to_string();
                profile.find_printf = false;
                profile.find_print0 = true;
                profile.sort_h = false;
                profile.xargs_0 = true;
                profile.grep_r = true;
                profile.grep_p = false;
                profile.stat_format = StatFormat::None;
                profile.sed_inplace_gnu = false;
                profile.du_max_depth = false;
                profile.date_gnu_format = false;
                profile.readlink_f = true;
                profile.ps_sort = false;
                profile.ls_sort = false;
                profile.awk_type = AwkType::BusyboxAwk;
            }
            _ => {}
        }

        profile
    }

    /// Create an Ubuntu-optimized profile (the default target)
    pub fn ubuntu() -> Self {
        let mut profile = Self::for_platform(ProfileType::GnuLinux);
        profile.os_name = "Ubuntu".to_string();
        profile.os_version = "22.04".to_string();
        profile.detected_shell = DetectedShell::Bash;
        profile.shell_sh = "/bin/dash".to_string();
        profile.awk_type = AwkType::Mawk; // Ubuntu uses mawk by default
        profile.tools = vec![
            "ls", "find", "grep", "awk", "sed", "sort", "head", "tail", "xargs", "stat", "du",
            "wc", "cat", "tar", "gzip", "curl", "cut", "tr", "uniq", "tee", "diff", "patch",
            "chmod", "chown",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        profile
    }

    async fn detect_os_identity(&mut self) {
        // Get uname
        if let Ok(output) = run_command("uname", &["-srm"]).await {
            self.uname = output.trim().to_string();
        }

        // Get OS name and version
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = tokio::fs::read_to_string("/etc/os-release").await {
                for line in contents.lines() {
                    if let Some(name) = line.strip_prefix("NAME=") {
                        self.os_name = name.trim_matches('"').to_string();
                    }
                    if let Some(version) = line.strip_prefix("VERSION_ID=") {
                        self.os_version = version.trim_matches('"').to_string();
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(name) = run_command("sw_vers", &["-productName"]).await {
                self.os_name = name.trim().to_string();
            }
            if let Ok(version) = run_command("sw_vers", &["-productVersion"]).await {
                self.os_version = version.trim().to_string();
            }
        }
    }

    async fn detect_shell(&mut self) {
        // Detect /bin/sh target
        #[cfg(unix)]
        {
            if let Ok(output) = run_command("readlink", &["-f", "/bin/sh"]).await {
                self.shell_sh = output.trim().to_string();
            } else if let Ok(output) = run_command("ls", &["-la", "/bin/sh"]).await {
                // Parse symlink from ls output
                if let Some(target) = output.split("->").nth(1) {
                    self.shell_sh = target.trim().to_string();
                }
            }
        }

        // Detect user's default shell
        if let Ok(shell) = std::env::var("SHELL") {
            let shell_name = shell.rsplit('/').next().unwrap_or(&shell);
            self.detected_shell = match shell_name {
                "bash" => DetectedShell::Bash,
                "zsh" => DetectedShell::Zsh,
                "fish" => DetectedShell::Fish,
                "dash" => DetectedShell::Dash,
                "sh" => DetectedShell::Sh,
                "ash" => DetectedShell::Ash,
                other => DetectedShell::Unknown(other.to_string()),
            };
        }

        // Windows detection
        #[cfg(windows)]
        {
            if std::env::var("PSModulePath").is_ok() {
                self.detected_shell = DetectedShell::PowerShell;
            } else {
                self.detected_shell = DetectedShell::Cmd;
            }
        }
    }

    async fn detect_tools(&mut self) {
        let tool_candidates = [
            "ls", "find", "grep", "awk", "sed", "sort", "head", "tail", "xargs", "stat", "du",
            "wc", "cat", "tar", "gzip", "curl", "cut", "tr", "uniq", "tee", "diff", "patch",
            "chmod", "chown", "mkdir", "rm", "cp", "mv", "touch", "file", "readlink", "basename",
            "dirname", "realpath", "date", "df", "ps", "kill", "top", "wget", "jq", "yq", "nc",
            "netstat", "ss", "lsof",
        ];

        let mut tools = Vec::new();
        for tool in tool_candidates {
            if tool_exists(tool).await {
                tools.push(tool.to_string());
            }
        }
        self.tools = tools;

        // Detect awk type
        if let Ok(version) = run_command("awk", &["--version"]).await {
            let version_lower = version.to_lowercase();
            if version_lower.contains("gnu awk") || version_lower.contains("gawk") {
                self.awk_type = AwkType::Gawk;
            } else if version_lower.contains("mawk") {
                self.awk_type = AwkType::Mawk;
            } else if version_lower.contains("busybox") {
                self.awk_type = AwkType::BusyboxAwk;
            }
        } else {
            // BSD awk typically doesn't support --version
            self.awk_type = AwkType::Nawk;
        }
    }

    async fn detect_features(&mut self) {
        // Run feature probes in parallel
        let (
            find_printf,
            find_print0,
            sort_h,
            xargs_0,
            grep_r,
            grep_p,
            stat_format,
            sed_inplace,
            du_max_depth,
            date_gnu,
            readlink_f,
            ps_sort,
            ls_sort,
        ) = tokio::join!(
            probe_find_printf(),
            probe_find_print0(),
            probe_sort_h(),
            probe_xargs_0(),
            probe_grep_r(),
            probe_grep_p(),
            probe_stat_format(),
            probe_sed_inplace_gnu(),
            probe_du_max_depth(),
            probe_date_gnu(),
            probe_readlink_f(),
            probe_ps_sort(),
            probe_ls_sort(),
        );

        self.find_printf = find_printf;
        self.find_print0 = find_print0;
        self.sort_h = sort_h;
        self.xargs_0 = xargs_0;
        self.grep_r = grep_r;
        self.grep_p = grep_p;
        self.stat_format = stat_format;
        self.sed_inplace_gnu = sed_inplace;
        self.du_max_depth = du_max_depth;
        self.date_gnu_format = date_gnu;
        self.readlink_f = readlink_f;
        self.ps_sort = ps_sort;
        self.ls_sort = ls_sort;
    }

    async fn determine_profile_type(&mut self) {
        // Check for BusyBox first
        if tool_exists("busybox").await {
            // Check if core tools are symlinks to busybox
            if let Ok(output) = run_command("ls", &["-la", "/bin/ls"]).await {
                if output.contains("busybox") {
                    self.profile_type = ProfileType::Busybox;
                    return;
                }
            }
        }

        // Check for GNU coreutils
        if let Ok(output) = run_command("ls", &["--version"]).await {
            if output.to_lowercase().contains("gnu") || output.contains("coreutils") {
                self.profile_type = ProfileType::GnuLinux;
                return;
            }
        }

        // Check for GNU findutils
        if let Ok(output) = run_command("find", &["--version"]).await {
            if output.to_lowercase().contains("gnu") || output.contains("findutils") {
                self.profile_type = ProfileType::GnuLinux;
                return;
            }
        }

        // macOS / BSD detection - on macOS, we immediately know it's BSD
        #[cfg(target_os = "macos")]
        {
            self.profile_type = ProfileType::Bsd;
        }

        // Return early on macOS to avoid unreachable code
        #[cfg(target_os = "macos")]
        return;

        // If ls --version fails but ls works, likely BSD (non-macOS platforms)
        #[cfg(not(target_os = "macos"))]
        if run_command("ls", &["--version"]).await.is_err()
            && run_command("ls", &["-d", "."]).await.is_ok()
        {
            self.profile_type = ProfileType::Bsd;
            return;
        }

        // Check for hybrid environments (Git Bash, MSYS2, Cygwin)
        // Only check on non-macOS platforms since macOS already returned above
        #[cfg(not(target_os = "macos"))]
        {
            if std::env::var("MSYSTEM").is_ok()
                || std::env::var("CYGWIN").is_ok()
                || self.uname.to_lowercase().contains("mingw")
                || self.uname.to_lowercase().contains("cygwin")
            {
                self.profile_type = ProfileType::Hybrid;
                return;
            }

            self.profile_type = ProfileType::Unknown;
        }
    }

    /// Convert profile to a format string for embedding in prompts
    pub fn to_prompt_format(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("PROFILE={}", self.profile_type));
        lines.push(format!("OS_NAME={}", self.os_name));
        lines.push(format!("OS_VERSION={}", self.os_version));
        lines.push(format!("UNAME={}", self.uname));
        lines.push(format!("SHELL_SH={}", self.shell_sh));
        lines.push(format!("TOOLS={}", self.tools.join(",")));
        lines.push(format!("FIND_PRINTF={}", self.find_printf));
        lines.push(format!("FIND_PRINT0={}", self.find_print0));
        lines.push(format!("SORT_H={}", self.sort_h));
        lines.push(format!("XARGS_0={}", self.xargs_0));
        lines.push(format!("GREP_R={}", self.grep_r));
        lines.push(format!("GREP_P={}", self.grep_p));
        lines.push(format!("STAT_FORMAT={}", self.stat_format));
        lines.push(format!("SED_INPLACE_GNU={}", self.sed_inplace_gnu));
        lines.push(format!("DU_MAX_DEPTH={}", self.du_max_depth));
        lines.push(format!("DATE_GNU_FORMAT={}", self.date_gnu_format));
        lines.push(format!("READLINK_F={}", self.readlink_f));
        lines.push(format!("PS_SORT={}", self.ps_sort));
        lines.push(format!("LS_SORT={}", self.ls_sort));

        lines.join("\n")
    }

    /// Generate capability notes for the LLM
    pub fn capability_notes(&self) -> Vec<String> {
        let mut notes = Vec::new();

        match self.profile_type {
            ProfileType::GnuLinux => {
                notes.push("GNU coreutils and findutils available".to_string());
                notes.push("Long options (--help, --version) supported".to_string());
            }
            ProfileType::Bsd => {
                notes.push("BSD utilities - some GNU flags not available".to_string());
                notes.push("Use short options where possible for portability".to_string());
            }
            ProfileType::Busybox => {
                notes.push("BusyBox utilities - limited feature set".to_string());
                notes.push("Use simplest command forms".to_string());
            }
            ProfileType::Hybrid => {
                notes.push("Hybrid environment - verify command availability".to_string());
            }
            ProfileType::Unknown => {
                notes.push("Unknown environment - use POSIX-compliant commands".to_string());
            }
        }

        // Feature-specific notes
        if !self.find_printf {
            notes.push("find -printf not available; use stat or ls for metadata".to_string());
        }

        if !self.sort_h {
            notes.push("sort -h not available; avoid human-readable sorting".to_string());
        }

        if !self.grep_p {
            notes.push("grep -P not available; use extended regex (-E) instead".to_string());
        }

        if !self.sed_inplace_gnu {
            notes.push("sed -i requires '' suffix on BSD (sed -i '' 's/.../...')".to_string());
        }

        if !self.du_max_depth {
            notes.push("du --max-depth not available; use du -d N instead".to_string());
        }

        if !self.date_gnu_format {
            notes.push("date --date not available; use date -v on BSD".to_string());
        }

        if !self.ps_sort {
            notes.push("ps --sort not available; pipe to sort instead".to_string());
        }

        notes
    }
}

// Helper functions for running commands

async fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let result = timeout(Duration::from_millis(500), async {
        tokio::task::spawn_blocking({
            let cmd = cmd.to_string();
            let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            move || Command::new(&cmd).args(&args).output()
        })
        .await
    })
    .await;

    match result {
        Ok(Ok(Ok(output))) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        _ => Err("Command failed".to_string()),
    }
}

async fn tool_exists(tool: &str) -> bool {
    run_command("which", &[tool]).await.is_ok()
}

// Feature probe functions

async fn probe_find_printf() -> bool {
    run_command("find", &[".", "-maxdepth", "0", "-printf", "%p\\n"])
        .await
        .is_ok()
}

async fn probe_find_print0() -> bool {
    run_command("find", &[".", "-maxdepth", "0", "-print0"])
        .await
        .is_ok()
}

async fn probe_sort_h() -> bool {
    let result = timeout(Duration::from_millis(500), async {
        tokio::task::spawn_blocking(|| {
            use std::process::Stdio;
            let mut child = Command::new("sh")
                .args(["-c", "printf '1K\\n2K\\n' | sort -h"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match &mut child {
                Ok(c) => c.wait().map(|s| s.success()).unwrap_or(false),
                Err(_) => false,
            }
        })
        .await
    })
    .await;

    matches!(result, Ok(Ok(true)))
}

async fn probe_xargs_0() -> bool {
    let result = timeout(Duration::from_millis(500), async {
        tokio::task::spawn_blocking(|| {
            use std::process::Stdio;
            let mut child = Command::new("sh")
                .args(["-c", "printf 'x\\0' | xargs -0 printf '%s'"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match &mut child {
                Ok(c) => c.wait().map(|s| s.success()).unwrap_or(false),
                Err(_) => false,
            }
        })
        .await
    })
    .await;

    matches!(result, Ok(Ok(true)))
}

async fn probe_grep_r() -> bool {
    run_command("grep", &["-R", "--help"]).await.is_ok()
        || run_command("grep", &["-r", ".", "-l", "-e", "^$", "/dev/null"])
            .await
            .is_ok()
}

async fn probe_grep_p() -> bool {
    let result = timeout(Duration::from_millis(500), async {
        tokio::task::spawn_blocking(|| {
            use std::process::Stdio;
            let mut child = Command::new("sh")
                .args(["-c", "echo test | grep -P 'test'"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match &mut child {
                Ok(c) => c.wait().map(|s| s.success()).unwrap_or(false),
                Err(_) => false,
            }
        })
        .await
    })
    .await;

    matches!(result, Ok(Ok(true)))
}

async fn probe_stat_format() -> StatFormat {
    // Try GNU stat
    if run_command("stat", &["--version"]).await.is_ok() {
        return StatFormat::Gnu;
    }

    // Try BSD stat
    if run_command("stat", &["-f", "%N", "."]).await.is_ok() {
        return StatFormat::Bsd;
    }

    StatFormat::None
}

async fn probe_sed_inplace_gnu() -> bool {
    // GNU sed accepts -i without argument
    // BSD sed requires -i ''
    // We check by looking at --version
    if let Ok(output) = run_command("sed", &["--version"]).await {
        output.to_lowercase().contains("gnu")
    } else {
        false
    }
}

async fn probe_du_max_depth() -> bool {
    run_command("du", &["--max-depth=0", "."]).await.is_ok()
}

async fn probe_date_gnu() -> bool {
    run_command("date", &["--date=now"]).await.is_ok()
}

async fn probe_readlink_f() -> bool {
    run_command("readlink", &["-f", "."]).await.is_ok()
}

async fn probe_ps_sort() -> bool {
    run_command("ps", &["--sort=pid", "-e"]).await.is_ok()
}

async fn probe_ls_sort() -> bool {
    run_command("ls", &["--sort=size", "."]).await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ubuntu_profile() {
        let profile = CapabilityProfile::ubuntu();

        assert_eq!(profile.profile_type, ProfileType::GnuLinux);
        assert_eq!(profile.os_name, "Ubuntu");
        assert!(profile.find_printf);
        assert!(profile.sort_h);
        assert!(profile.grep_r);
        assert_eq!(profile.stat_format, StatFormat::Gnu);
    }

    #[test]
    fn test_bsd_profile() {
        let profile = CapabilityProfile::for_platform(ProfileType::Bsd);

        assert_eq!(profile.profile_type, ProfileType::Bsd);
        assert!(!profile.find_printf);
        assert!(!profile.sort_h);
        assert_eq!(profile.stat_format, StatFormat::Bsd);
    }

    #[test]
    fn test_profile_to_prompt_format() {
        let profile = CapabilityProfile::ubuntu();
        let format = profile.to_prompt_format();

        assert!(format.contains("PROFILE=gnu-linux"));
        assert!(format.contains("OS_NAME=Ubuntu"));
        assert!(format.contains("FIND_PRINTF=true"));
    }

    #[test]
    fn test_capability_notes() {
        let profile = CapabilityProfile {
            profile_type: ProfileType::Bsd,
            find_printf: false,
            sort_h: false,
            sed_inplace_gnu: false,
            ..Default::default()
        };

        let notes = profile.capability_notes();

        assert!(notes.iter().any(|n| n.contains("BSD")));
        assert!(notes.iter().any(|n| n.contains("find -printf")));
        assert!(notes.iter().any(|n| n.contains("sed -i")));
    }

    #[tokio::test]
    async fn test_detect_profile() {
        // This test actually runs on the current system
        let profile = CapabilityProfile::detect().await;

        // Profile should be detected
        assert_ne!(profile.profile_type, ProfileType::Unknown);

        // Should have some tools
        assert!(!profile.tools.is_empty());

        // Should be able to format for prompt
        let format = profile.to_prompt_format();
        assert!(format.contains("PROFILE="));
    }
}
