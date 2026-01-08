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
            pattern: r"rm\s+(-[rfRF]*\s+)*(/|~|\$HOME|/\*|~/\*|\*|\.|\.\/|\.\/\*|\.\*)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of root, home, or current directory".to_string(),
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
}
