use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};
use anyhow::Result;

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

    /// Distribution identifier for index matching (e.g., "ubuntu-22.04", "macos-14")
    pub distro_id: String,

    /// Whether network is available for downloading indexes
    pub network_available: bool,
}

impl ExecutionContext {
    /// Detect current execution context
    pub fn detect() -> Self {
        let os = std::env::consts::OS.to_string();
        let os_version = Self::get_os_version();
        let distribution = Self::detect_distribution();

        Self {
            distro_id: Self::generate_distro_id(&os, &os_version, &distribution),
            network_available: Self::check_network_available(),
            os,
            arch: std::env::consts::ARCH.to_string(),
            os_version,
            distribution,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            shell: Self::detect_shell(),
            user: std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
            available_commands: Self::scan_available_commands(),
        }
    }
    
    /// Get platform-specific rules for command generation
    pub fn get_platform_rules(&self) -> String {
        match self.os.as_str() {
            "macos" => self.get_macos_rules(),
            "linux" => self.get_linux_rules(),
            _ => String::from("Use POSIX-compliant commands")
        }
    }
    
    fn get_macos_rules(&self) -> String {
        format!(r#"macOS {} (BSD-style commands):
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
- Current directory: {}"#, self.os_version, self.cwd.display())
    }
    
    fn get_linux_rules(&self) -> String {
        let dist = self.distribution.as_deref().unwrap_or("Linux");
        format!(r#"{} (GNU coreutils):
- ps: Can use 'ps aux --sort=-pcpu' for CPU sorting
- Network: Use 'ss -tuln' for listening ports
- df: Can use 'df -h --sort=size' for size sorting
- find: Use 'find .' for current directory
- sed: Use 'sed -i' for in-place editing
- du: Can use 'du -h --max-depth=1'
- date: GNU date, use 'date --date="7 days ago"'
- readlink: Use 'readlink -f' for canonical path
- stat: Use 'stat --format=%s'
- Current directory: {}"#, dist, self.cwd.display())
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
                    content.lines()
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
            .and_then(|shell| {
                shell.rsplit('/').next().map(|s| s.to_string())
            })
            .unwrap_or_else(|| "bash".to_string())
    }
    
    fn scan_available_commands() -> Vec<String> {
        let common_commands = vec![
            "ps", "top", "kill", "killall",
            "find", "grep", "egrep", "fgrep", "sed", "awk",
            "sort", "head", "tail", "cut", "tr", "wc", "xargs", "uniq",
            "ls", "cat", "less", "more", "file",
            "df", "du", "lsof", "netstat", "ss", "ifconfig", "ip",
            "git", "curl", "wget", "nc", "telnet",
            "tar", "gzip", "gunzip", "zip", "unzip", "bzip2",
            "chmod", "chown", "chgrp", "umask",
            "date", "cal", "uptime", "w", "who", "whoami",
            "env", "export", "echo", "printf",
            "jq", "yq", "xmllint",
        ];
        
        common_commands.iter()
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

    /// Generate distribution identifier for index matching
    fn generate_distro_id(os: &str, os_version: &str, distribution: &Option<String>) -> String {
        match os {
            "macos" => {
                // Extract major version from os_version (e.g., "14.2.1" -> "14")
                let major = os_version.split('.').next().unwrap_or("unknown");
                format!("macos-{}", major)
            }
            "linux" => {
                // Try to extract distribution name and version
                if let Some(distro) = distribution {
                    // Parse from "Ubuntu 22.04 LTS" -> "ubuntu-22.04"
                    Self::parse_linux_distro_id(distro)
                } else {
                    "linux-unknown".to_string()
                }
            }
            "windows" => {
                // Windows version detection
                let major = os_version.split('.').next().unwrap_or("unknown");
                format!("windows-{}", major)
            }
            _ => format!("{}-unknown", os),
        }
    }

    /// Parse Linux distribution ID from PRETTY_NAME
    fn parse_linux_distro_id(pretty_name: &str) -> String {
        let lower = pretty_name.to_lowercase();

        // Ubuntu 22.04 LTS -> ubuntu-22.04
        if lower.contains("ubuntu") {
            if let Some(version) = Self::extract_version(&lower) {
                return format!("ubuntu-{}", version);
            }
            return "ubuntu-unknown".to_string();
        }

        // Debian GNU/Linux 12 -> debian-12
        if lower.contains("debian") {
            if let Some(version) = Self::extract_version(&lower) {
                return format!("debian-{}", version);
            }
            return "debian-unknown".to_string();
        }

        // Fedora Linux 38 -> fedora-38
        if lower.contains("fedora") {
            if let Some(version) = Self::extract_version(&lower) {
                return format!("fedora-{}", version);
            }
            return "fedora-unknown".to_string();
        }

        // CentOS Linux 7 -> centos-7
        if lower.contains("centos") {
            if let Some(version) = Self::extract_version(&lower) {
                return format!("centos-{}", version);
            }
            return "centos-unknown".to_string();
        }

        // Fallback
        "linux-unknown".to_string()
    }

    /// Extract version number from string (e.g., "Ubuntu 22.04" -> "22.04")
    fn extract_version(text: &str) -> Option<String> {
        // Look for pattern like "22.04" or "22"
        for word in text.split_whitespace() {
            // Check if word looks like a version number
            if word.chars().next()?.is_ascii_digit() {
                // Take major.minor (e.g., "22.04")
                let parts: Vec<&str> = word.split('.').take(2).collect();
                if !parts.is_empty() {
                    return Some(parts.join("."));
                }
            }
        }
        None
    }

    /// Check if network is available
    fn check_network_available() -> bool {
        #[cfg(feature = "vector-store")]
        {
            Self::check_network_impl()
        }

        #[cfg(not(feature = "vector-store"))]
        {
            true // Assume available if feature disabled
        }
    }

    #[cfg(feature = "vector-store")]
    fn check_network_impl() -> bool {
        use dns_lookup::lookup_host;

        // Try to resolve common hostnames
        let test_hosts = vec![
            "github.com",
            "raw.githubusercontent.com",
            "api.github.com",
        ];

        for host in test_hosts {
            if lookup_host(host).is_ok() {
                return true;
            }
        }

        false
    }
    
    /// Get context summary for system prompt
    pub fn get_prompt_context(&self) -> String {
        format!(r#"EXECUTION ENVIRONMENT:
- Platform: {} {} ({})
- Architecture: {}
- Shell: {}
- Current Directory: {}
- User: {}

AVAILABLE COMMANDS: {}

PLATFORM-SPECIFIC RULES:
{}"#,
            self.os,
            self.os_version,
            self.distribution.as_deref().unwrap_or("Unknown"),
            self.arch,
            self.shell,
            self.cwd.display(),
            self.user,
            self.available_commands.join(", "),
            self.get_platform_rules()
        )
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
