//! Command scanner for discovering installed commands

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use tracing::{debug, info, warn};

/// Scan system for installed commands
pub fn scan_installed_commands() -> Result<Vec<String>> {
    info!("Scanning system for installed commands");

    let mut commands = HashSet::new();

    // Method 1: Scan PATH directories
    let path_commands = scan_path_directories()?;
    commands.extend(path_commands);

    // Method 2: Check common utilities
    let common_commands = get_common_commands();
    commands.extend(common_commands.into_iter().filter(|cmd| command_exists(cmd)));

    // Method 3: Scan common man page directories
    let man_commands = scan_man_directories()?;
    commands.extend(man_commands);

    let mut result: Vec<String> = commands.into_iter().collect();
    result.sort();

    info!("Found {} installed commands", result.len());
    Ok(result)
}

/// Scan directories in PATH environment variable
fn scan_path_directories() -> Result<HashSet<String>> {
    let mut commands = HashSet::new();

    let path_var = env::var("PATH").unwrap_or_default();
    let paths: Vec<&str> = path_var.split(':').collect();

    debug!("Scanning {} PATH directories", paths.len());

    for path_str in paths {
        let path = Path::new(path_str);
        if !path.exists() || !path.is_dir() {
            continue;
        }

        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        // Only include regular files and symlinks
                        if file_type.is_file() || file_type.is_symlink() {
                            if let Some(name) = entry.file_name().to_str() {
                                // Skip common non-commands
                                if !is_likely_command(name) {
                                    continue;
                                }

                                // Check if executable
                                if is_executable(&entry.path()) {
                                    commands.insert(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Could not read directory {}: {}", path_str, e);
            }
        }
    }

    Ok(commands)
}

/// Check if a file is executable
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        // Check if any execute bit is set
        return mode & 0o111 != 0;
    }

    false
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    // On Windows, check file extension
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str().unwrap_or("").to_lowercase().as_str(),
            "exe" | "bat" | "cmd" | "ps1"
        )
    } else {
        false
    }
}

/// Check if filename is likely a command (not a library or config file)
fn is_likely_command(name: &str) -> bool {
    // Skip files with common non-command extensions
    if name.ends_with(".so")
        || name.ends_with(".a")
        || name.ends_with(".dylib")
        || name.ends_with(".conf")
        || name.ends_with(".txt")
    {
        return false;
    }

    // Skip hidden files
    if name.starts_with('.') {
        return false;
    }

    // Skip common non-command patterns
    if name.contains("lib") && name.contains(".so") {
        return false;
    }

    true
}

/// Get list of common Unix/Linux commands
fn get_common_commands() -> Vec<String> {
    vec![
        // Core utilities
        "ls", "cat", "cp", "mv", "rm", "mkdir", "rmdir", "touch", "chmod", "chown", "ln",
        "pwd", "cd", "echo", "printf", "test", "true", "false",
        // Text processing
        "grep", "egrep", "fgrep", "sed", "awk", "cut", "tr", "sort", "uniq", "wc", "head",
        "tail", "less", "more", "file", "diff", "patch", "comm", "join", "paste", "column",
        "expand", "unexpand", "fold", "fmt", "pr", "nl",
        // File operations
        "find", "locate", "updatedb", "which", "whereis", "type", "basename", "dirname",
        "readlink", "realpath", "stat", "du", "df", "mount", "umount",
        // Archiving and compression
        "tar", "gzip", "gunzip", "bzip2", "bunzip2", "xz", "unxz", "zip", "unzip", "rar",
        "unrar", "7z",
        // Network tools
        "curl", "wget", "nc", "netcat", "telnet", "ssh", "scp", "rsync", "ftp", "sftp",
        "ping", "traceroute", "tracepath", "mtr", "nslookup", "dig", "host", "whois",
        "netstat", "ss", "lsof", "tcpdump", "nmap", "iptables", "ip", "ifconfig", "route",
        // Process management
        "ps", "top", "htop", "kill", "killall", "pkill", "pgrep", "nice", "renice", "nohup",
        "bg", "fg", "jobs", "wait", "at", "cron", "crontab", "watch",
        // System information
        "uname", "hostname", "uptime", "date", "cal", "w", "who", "whoami", "id", "groups",
        "last", "lastlog", "users", "finger",
        // User management
        "useradd", "usermod", "userdel", "groupadd", "groupmod", "groupdel", "passwd",
        "chage", "su", "sudo", "visudo",
        // System administration
        "systemctl", "service", "journalctl", "dmesg", "sysctl", "shutdown", "reboot",
        "halt", "poweroff", "init", "telinit",
        // Package management (various distros)
        "apt", "apt-get", "dpkg", "yum", "dnf", "rpm", "zypper", "pacman", "emerge",
        "brew", "port",
        // Development tools
        "git", "svn", "hg", "make", "cmake", "gcc", "g++", "clang", "clang++", "ld", "ar",
        "nm", "objdump", "strip", "gdb", "lldb", "strace", "ltrace", "valgrind",
        // Editors
        "vim", "vi", "nano", "emacs", "ed", "ex",
        // Shells
        "bash", "sh", "zsh", "fish", "dash", "ksh", "tcsh", "csh",
        // Scripting
        "python", "python3", "ruby", "perl", "php", "node", "lua",
        // Disk utilities
        "fdisk", "gdisk", "parted", "mkfs", "fsck", "e2fsck", "resize2fs", "tune2fs",
        "blkid", "lsblk", "dd", "sync",
        // Miscellaneous
        "xargs", "env", "printenv", "export", "alias", "unalias", "history", "fc", "clear",
        "reset", "tput", "stty", "tee", "mktemp", "sleep", "timeout", "yes", "seq", "bc",
        "dc", "expr", "factor", "jq", "yq", "xmllint", "pandoc",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

/// Scan man page directories for additional commands
fn scan_man_directories() -> Result<HashSet<String>> {
    let mut commands = HashSet::new();

    let man_dirs = vec![
        "/usr/share/man/man1",
        "/usr/share/man/man8",
        "/usr/local/share/man/man1",
        "/usr/local/share/man/man8",
        "/opt/local/share/man/man1",
    ];

    for man_dir in man_dirs {
        let path = Path::new(man_dir);
        if !path.exists() || !path.is_dir() {
            continue;
        }

        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        // Extract command name from man page filename
                        // Examples: grep.1, grep.1.gz, systemctl.8
                        if let Some(cmd) = extract_command_from_manpage(name) {
                            commands.insert(cmd);
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Could not read man directory {}: {}", man_dir, e);
            }
        }
    }

    Ok(commands)
}

/// Extract command name from man page filename
fn extract_command_from_manpage(filename: &str) -> Option<String> {
    // Man page filenames are like: command.1, command.1.gz, command.8.gz
    let name = filename
        .trim_end_matches(".gz")
        .trim_end_matches(".bz2")
        .trim_end_matches(".xz");

    // Remove section number
    if let Some(dot_pos) = name.rfind('.') {
        let cmd = &name[..dot_pos];
        if !cmd.is_empty() {
            return Some(cmd.to_string());
        }
    }

    None
}

/// Check if a command exists on the system
fn command_exists(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_likely_command() {
        assert!(is_likely_command("ls"));
        assert!(is_likely_command("grep"));
        assert!(is_likely_command("my-tool"));

        assert!(!is_likely_command(".hidden"));
        assert!(!is_likely_command("libfoo.so"));
        assert!(!is_likely_command("config.conf"));
    }

    #[test]
    fn test_extract_command_from_manpage() {
        assert_eq!(
            extract_command_from_manpage("grep.1"),
            Some("grep".to_string())
        );
        assert_eq!(
            extract_command_from_manpage("grep.1.gz"),
            Some("grep".to_string())
        );
        assert_eq!(
            extract_command_from_manpage("systemctl.8.gz"),
            Some("systemctl".to_string())
        );
        assert_eq!(extract_command_from_manpage("noext"), None);
    }

    #[test]
    fn test_scan_installed_commands() {
        let commands = scan_installed_commands().unwrap();

        // Should find at least some basic commands
        assert!(!commands.is_empty());

        // Common commands should be present on most systems
        assert!(commands.contains(&"ls".to_string()));
    }

    #[test]
    fn test_command_exists() {
        // ls should exist on all Unix-like systems
        assert!(command_exists("ls"));

        // This command almost certainly doesn't exist
        assert!(!command_exists("this_command_definitely_does_not_exist_12345"));
    }
}
