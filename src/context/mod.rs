use crate::starship_context::{GitContext, ProjectType, StarshipContext};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// Complete execution context for command generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Operating system (Darwin, Linux, Windows)
    pub os: String,

    /// Architecture (arm64, x86_64)
    pub arch: String,

    /// OS version (14.2.1 for macOS, 6.5.0 for Linux)
    pub os_version: String,

    /// Distribution name (macOS Sonoma, Ubuntu 22.04, etc.)
    pub distribution: Option<String>,

    /// Current working directory
    pub cwd: PathBuf,

    /// Current shell (zsh, bash, fish)
    pub shell: String,

    /// Current user
    pub user: String,

    /// Available commands on system
    pub available_commands: Vec<String>,

    /// Git repository context (from starship)
    #[serde(default)]
    pub git: GitContext,

    /// Detected project type (from starship)
    #[serde(default = "default_project_type")]
    pub project_type: ProjectType,

    /// Notable files in current directory
    #[serde(default)]
    pub notable_files: Vec<String>,

    /// Terminal width in columns
    #[serde(default)]
    pub terminal_width: usize,
}

fn default_project_type() -> ProjectType {
    ProjectType::Generic
}

impl ExecutionContext {
    /// Detect current execution context with enhanced starship integration
    pub fn detect() -> Self {
        // Get enhanced context from starship
        let starship_ctx = StarshipContext::detect();

        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            os_version: Self::get_os_version(),
            distribution: Self::detect_distribution(),
            cwd: starship_ctx.current_dir.clone(),
            shell: starship_ctx.shell.to_string(),
            user: std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
            available_commands: Self::scan_available_commands(),
            git: starship_ctx.git,
            project_type: starship_ctx.project_type,
            notable_files: starship_ctx.notable_files,
            terminal_width: starship_ctx.terminal_width,
        }
    }

    /// Detect current execution context without starship (faster, for basic use)
    pub fn detect_basic() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            os_version: Self::get_os_version(),
            distribution: Self::detect_distribution(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            shell: Self::detect_shell(),
            user: std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
            available_commands: Self::scan_available_commands(),
            git: GitContext::default(),
            project_type: ProjectType::Generic,
            notable_files: Vec::new(),
            terminal_width: 0,
        }
    }

    /// Get platform-specific rules for command generation
    pub fn get_platform_rules(&self) -> String {
        match self.os.as_str() {
            "macos" => self.get_macos_rules(),
            "linux" => self.get_linux_rules(),
            "windows" => self.get_windows_rules(),
            _ => String::from("Use POSIX-compliant commands"),
        }
    }

    fn get_macos_rules(&self) -> String {
        format!(
            r#"macOS {} (BSD-style commands):
- ps: Use 'ps aux' (no --sort flag), pipe to 'sort -nrk 3,3' for CPU sorting
- Network: Use 'lsof -iTCP -sTCP:LISTEN' or 'netstat' (NOT ss - not available)
- df: Use 'df -h' (no --sort flag), pipe to 'sort -k5 -hr' for size sorting
- find: Use 'find .' for current directory (avoid 'find /' - permission errors)
- sed: Use 'sed -i ""' for in-place editing (note the empty string after -i)
- du: Use 'du -h' (no --max-depth), use '-d' flag instead: 'du -d 1 -h'
- date: BSD date format, use 'date -v-7d' not 'date --date'
- readlink: Use 'readlink' (no -f flag), use 'greadlink -f' if available
- stat: Use 'stat -f %s' not 'stat --format'
- xargs: Works same as Linux
- Current directory: {}"#,
            self.os_version,
            self.cwd.display()
        )
    }

    fn get_linux_rules(&self) -> String {
        let dist = self.distribution.as_deref().unwrap_or("Linux");
        format!(
            r#"{} (GNU coreutils):
- ps: Can use 'ps aux --sort=-pcpu' for CPU sorting
- Network: Use 'ss -tuln' for listening ports
- df: Can use 'df -h --sort=size' for size sorting
- find: Use 'find .' for current directory
- sed: Use 'sed -i' for in-place editing
- du: Can use 'du -h --max-depth=1'
- date: GNU date, use 'date --date="7 days ago"'
- readlink: Use 'readlink -f' for canonical path
- stat: Use 'stat --format=%s'
- Current directory: {}"#,
            dist,
            self.cwd.display()
        )
    }

    fn get_windows_rules(&self) -> String {
        format!(
            r#"Windows {} (PowerShell/CMD):
- Listing: Use 'dir' or 'Get-ChildItem' (PowerShell)
- Network: Use 'netstat -an' or 'Get-NetTCPConnection' (PowerShell)
- Processes: Use 'tasklist' or 'Get-Process' (PowerShell)
- Find files: Use 'dir /s' or 'Get-ChildItem -Recurse' (PowerShell)
- Environment: Use 'set' or '$env:' (PowerShell)
- Disk usage: Use 'dir' or 'Get-ChildItem | Measure-Object' (PowerShell)
- Date/time: Use 'date /t' and 'time /t' or 'Get-Date' (PowerShell)
- File content: Use 'type' or 'Get-Content' (PowerShell)
- Paths: Use backslashes '\' for paths (e.g., C:\Users\)
- Current directory: {}"#,
            self.os_version,
            self.cwd.display()
        )
    }

    fn get_os_version() -> String {
        #[cfg(target_os = "macos")]
        {
            Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("uname")
                .arg("-r")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            String::new()
        }
    }

    fn detect_distribution() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            Command::new("sw_vers")
                .arg("-productName")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|name| {
                    let version = Self::get_os_version();
                    format!("{} {}", name.trim(), version)
                })
        }

        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/etc/os-release")
                .ok()
                .and_then(|content| {
                    content
                        .lines()
                        .find(|line| line.starts_with("PRETTY_NAME="))
                        .map(|line| {
                            line.trim_start_matches("PRETTY_NAME=")
                                .trim_matches('"')
                                .to_string()
                        })
                })
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }

    fn detect_shell() -> String {
        std::env::var("SHELL")
            .ok()
            .and_then(|shell| shell.rsplit('/').next().map(|s| s.to_string()))
            .unwrap_or_else(|| "bash".to_string())
    }

    fn scan_available_commands() -> Vec<String> {
        let common_commands = vec![
            "ps", "top", "kill", "killall", "find", "grep", "egrep", "fgrep", "sed", "awk", "sort",
            "head", "tail", "cut", "tr", "wc", "xargs", "uniq", "ls", "cat", "less", "more",
            "file", "df", "du", "lsof", "netstat", "ss", "ifconfig", "ip", "git", "curl", "wget",
            "nc", "telnet", "tar", "gzip", "gunzip", "zip", "unzip", "bzip2", "chmod", "chown",
            "chgrp", "umask", "date", "cal", "uptime", "w", "who", "whoami", "env", "export",
            "echo", "printf", "jq", "yq", "xmllint",
        ];

        common_commands
            .iter()
            .filter(|cmd| Self::command_exists(cmd))
            .map(|s| s.to_string())
            .collect()
    }

    fn command_exists(command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get context summary for system prompt
    pub fn get_prompt_context(&self) -> String {
        let mut context = format!(
            r#"EXECUTION ENVIRONMENT:
- Platform: {} {} ({})
- Architecture: {}
- Shell: {}
- Current Directory: {}
- User: {}
- Terminal Width: {} columns"#,
            self.os,
            self.os_version,
            self.distribution.as_deref().unwrap_or("Unknown"),
            self.arch,
            self.shell,
            self.cwd.display(),
            self.user,
            self.terminal_width
        );

        // Add git context if in a repository
        if self.git.is_git_repo {
            context.push_str("\n\nGIT REPOSITORY:");
            if let Some(ref branch) = self.git.branch {
                context.push_str(&format!("\n- Branch: {}", branch));
            }
            if let Some(ref state) = self.git.state {
                if state != "Clean" {
                    context.push_str(&format!("\n- State: {}", state));
                }
            }
            if let Some(ref remote) = self.git.remote_name {
                context.push_str(&format!("\n- Remote: {}", remote));
            }
            if let Some(ref root) = self.git.repo_root {
                context.push_str(&format!("\n- Repo Root: {}", root.display()));
            }
        }

        // Add project type context
        if self.project_type != ProjectType::Generic {
            context.push_str(&format!("\n\nPROJECT TYPE: {}", self.project_type));
            let hints = self.project_type.command_hints();
            if !hints.is_empty() {
                context.push_str("\nCommon commands:");
                for hint in hints.iter().take(3) {
                    context.push_str(&format!("\n  - {}", hint));
                }
            }
        }

        // Add notable files
        if !self.notable_files.is_empty() {
            context.push_str(&format!(
                "\n\nNOTABLE FILES: {}",
                self.notable_files.join(", ")
            ));
        }

        context.push_str(&format!(
            "\n\nAVAILABLE COMMANDS: {}\n\nPLATFORM-SPECIFIC RULES:\n{}",
            self.available_commands.join(", "),
            self.get_platform_rules()
        ));

        context
    }

    /// Check if we're in a git repository
    pub fn is_git_repo(&self) -> bool {
        self.git.is_git_repo
    }

    /// Get the current git branch
    pub fn git_branch(&self) -> Option<&str> {
        self.git.branch.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_detection() {
        let context = ExecutionContext::detect();

        assert!(!context.os.is_empty());
        assert!(!context.arch.is_empty());
        assert!(!context.shell.is_empty());
        assert!(!context.available_commands.is_empty());
    }

    #[test]
    fn test_platform_rules() {
        let context = ExecutionContext::detect();
        let rules = context.get_platform_rules();

        assert!(!rules.is_empty());
        assert!(rules.contains("Current directory"));
    }
}
