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
            pattern: r"rm\s+(-[rfRF]*\s+)*(/|~|\$HOME|/\*|~/\*)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of root or home directory".to_string(),
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
            pattern: r"sudo\s+su(\s+-.*|$)".to_string(),
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

    #[test]
    fn test_pattern_engine_creation() {
        let engine = PatternEngine::new().expect("Should create pattern engine");
        assert!(!engine.dangerous_patterns.patterns.is_empty());
        assert!(!engine.safe_patterns.patterns.is_empty());
        assert!(!engine.context_patterns.patterns.is_empty());
    }

    #[test]
    fn test_pattern_engine_analysis() {
        let engine = PatternEngine::new().expect("Should create pattern engine");

        // Test safe command
        let safe_result = engine.analyze_command("ls -la", ShellType::Bash);
        assert!(!safe_result.matched);
        assert_eq!(safe_result.risk_level, RiskLevel::Safe);
        assert!(safe_result.safety_score < 0.1);

        // Test dangerous command
        let dangerous_result = engine.analyze_command("rm -rf /", ShellType::Bash);
        assert!(dangerous_result.matched);
        assert_eq!(dangerous_result.risk_level, RiskLevel::Critical);
        assert!(dangerous_result.safety_score > 0.3); // Should be high risk
        assert!(!dangerous_result.matched_patterns.is_empty());
    }

    #[test]
    fn test_compiled_patterns_filtering() {
        let engine = PatternEngine::new().expect("Should create pattern engine");

        // Test risk-based filtering
        let critical_patterns = engine.dangerous_patterns.get_by_risk(RiskLevel::Critical);
        assert!(!critical_patterns.is_empty());

        // Test category-based filtering
        let dangerous_patterns = engine.dangerous_patterns.get_by_category("dangerous");
        assert!(!dangerous_patterns.is_empty());

        // Test enabled patterns
        let enabled_patterns = engine.dangerous_patterns.get_enabled();
        assert!(!enabled_patterns.is_empty());
        assert!(enabled_patterns.iter().all(|p| p.enabled));
    }

    #[test]
    fn test_custom_pattern_addition() {
        let mut engine = PatternEngine::new().expect("Should create pattern engine");
        let initial_count = engine.dangerous_patterns.patterns.len();

        let custom_pattern = DangerPattern {
            pattern: r"test_custom_danger".to_string(),
            risk_level: RiskLevel::High,
            description: "Test custom dangerous pattern".to_string(),
            shell_specific: None,
        };

        engine
            .add_custom_pattern(custom_pattern, "test")
            .expect("Should add custom pattern");
        assert_eq!(engine.dangerous_patterns.patterns.len(), initial_count + 1);

        // Test that the custom pattern works
        let result = engine.analyze_command("test_custom_danger", ShellType::Bash);
        assert!(result.matched);
        assert_eq!(result.risk_level, RiskLevel::High);
    }
}

/// Production-ready pattern engine with compiled regex patterns for advanced safety validation
#[derive(Debug)]
pub struct PatternEngine {
    pub dangerous_patterns: CompiledPatterns,
    pub safe_patterns: CompiledPatterns,
    pub context_patterns: CompiledPatterns,
}

/// Compiled pattern collection with metadata for efficient matching
#[derive(Debug)]
pub struct CompiledPatterns {
    /// Pre-compiled regex patterns with associated metadata
    pub patterns: Vec<AdvancedCompiledPattern>,
    /// Pattern lookup by category for faster filtering
    pub by_category: std::collections::HashMap<String, Vec<usize>>,
    /// Pattern lookup by risk level for efficient risk-based filtering
    pub by_risk: std::collections::HashMap<RiskLevel, Vec<usize>>,
}

/// Individual compiled pattern with metadata and performance tracking
#[derive(Debug)]
pub struct AdvancedCompiledPattern {
    pub regex: Regex,
    pub risk_level: RiskLevel,
    pub description: String,
    pub category: String,
    pub shell_specific: Option<ShellType>,
    pub enabled: bool,
    /// Performance metrics (number of matches, average match time)
    pub match_count: std::sync::atomic::AtomicU64,
    pub total_match_time_ns: std::sync::atomic::AtomicU64,
}

/// Pattern matching result with detailed context
#[derive(Debug, Clone)]
pub struct PatternMatchResult {
    pub matched: bool,
    pub risk_level: RiskLevel,
    pub matched_patterns: Vec<PatternMatch>,
    pub total_match_time_ns: u64,
    pub safety_score: f64, // 0.0 = safe, 1.0 = critical
}

/// Individual pattern match with context
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_description: String,
    pub risk_level: RiskLevel,
    pub matched_text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub category: String,
}

impl PatternEngine {
    /// Create a new pattern engine with all default patterns
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let dangerous_patterns = Self::compile_dangerous_patterns()?;
        let safe_patterns = Self::compile_safe_patterns()?;
        let context_patterns = Self::compile_context_patterns()?;

        Ok(Self {
            dangerous_patterns,
            safe_patterns,
            context_patterns,
        })
    }

    /// Create pattern engine with custom pattern sets
    pub fn with_custom_patterns(
        dangerous: Vec<DangerPattern>,
        safe: Vec<DangerPattern>,
        context: Vec<DangerPattern>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dangerous_patterns = Self::compile_pattern_set(dangerous, "dangerous")?;
        let safe_patterns = Self::compile_pattern_set(safe, "safe")?;
        let context_patterns = Self::compile_pattern_set(context, "context")?;

        Ok(Self {
            dangerous_patterns,
            safe_patterns,
            context_patterns,
        })
    }

    /// Analyze command for dangerous patterns with detailed results
    pub fn analyze_command(&self, command: &str, shell: ShellType) -> PatternMatchResult {
        let start_time = std::time::Instant::now();
        let mut matches = Vec::new();
        let mut max_risk = RiskLevel::Safe;
        let mut total_score = 0.0;

        // Check dangerous patterns
        for (_i, pattern) in self.dangerous_patterns.patterns.iter().enumerate() {
            if !pattern.enabled {
                continue;
            }

            if let Some(shell_specific) = pattern.shell_specific {
                if shell_specific != shell {
                    continue;
                }
            }

            if let Some(captures) = pattern.regex.find(command) {
                pattern
                    .match_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let pattern_match = PatternMatch {
                    pattern_description: pattern.description.clone(),
                    risk_level: pattern.risk_level,
                    matched_text: captures.as_str().to_string(),
                    start_pos: captures.start(),
                    end_pos: captures.end(),
                    category: pattern.category.clone(),
                };

                matches.push(pattern_match);

                // Update maximum risk level
                if pattern.risk_level as u8 > max_risk as u8 {
                    max_risk = pattern.risk_level;
                }

                // Add to safety score (weighted by risk level)
                total_score += match pattern.risk_level {
                    RiskLevel::Low => 0.2,
                    RiskLevel::Safe => 0.1,
                    RiskLevel::Medium => 0.3,
                    RiskLevel::Moderate => 0.4,
                    RiskLevel::High => 0.7,
                    RiskLevel::Critical => 1.0,
                };
            }
        }

        let elapsed = start_time.elapsed();

        // Update performance metrics
        for pattern_match in &matches {
            if let Some(pattern) = self
                .dangerous_patterns
                .patterns
                .iter()
                .find(|p| p.description == pattern_match.pattern_description)
            {
                pattern.total_match_time_ns.fetch_add(
                    elapsed.as_nanos() as u64 / matches.len() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
            }
        }

        // Calculate final safety score (normalize and clamp)
        let safety_score = (total_score / 3.0f64).clamp(0.0, 1.0);

        PatternMatchResult {
            matched: !matches.is_empty(),
            risk_level: max_risk,
            matched_patterns: matches,
            total_match_time_ns: elapsed.as_nanos() as u64,
            safety_score,
        }
    }

    /// Get performance statistics for all patterns
    pub fn get_performance_stats(&self) -> Vec<PatternPerformanceStats> {
        self.dangerous_patterns
            .patterns
            .iter()
            .map(|pattern| {
                let match_count = pattern
                    .match_count
                    .load(std::sync::atomic::Ordering::Relaxed);
                let total_time = pattern
                    .total_match_time_ns
                    .load(std::sync::atomic::Ordering::Relaxed);
                let avg_time = if match_count > 0 {
                    total_time / match_count
                } else {
                    0
                };

                PatternPerformanceStats {
                    description: pattern.description.clone(),
                    category: pattern.category.clone(),
                    match_count,
                    average_match_time_ns: avg_time,
                }
            })
            .collect()
    }

    /// Add custom pattern at runtime
    pub fn add_custom_pattern(
        &mut self,
        pattern: DangerPattern,
        category: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let regex = Regex::new(&pattern.pattern)?;
        let compiled = AdvancedCompiledPattern {
            regex,
            risk_level: pattern.risk_level,
            description: pattern.description,
            category: category.to_string(),
            shell_specific: pattern.shell_specific,
            enabled: true,
            match_count: std::sync::atomic::AtomicU64::new(0),
            total_match_time_ns: std::sync::atomic::AtomicU64::new(0),
        };

        let index = self.dangerous_patterns.patterns.len();
        self.dangerous_patterns.patterns.push(compiled);

        // Update indices
        self.dangerous_patterns
            .by_category
            .entry(category.to_string())
            .or_insert_with(Vec::new)
            .push(index);

        self.dangerous_patterns
            .by_risk
            .entry(pattern.risk_level)
            .or_insert_with(Vec::new)
            .push(index);

        Ok(())
    }

    /// Compile dangerous patterns from the built-in set
    fn compile_dangerous_patterns() -> Result<CompiledPatterns, Box<dyn std::error::Error>> {
        Self::compile_pattern_set(DANGEROUS_PATTERNS.clone(), "dangerous")
    }

    /// Compile safe patterns (patterns that indicate safe operations)
    fn compile_safe_patterns() -> Result<CompiledPatterns, Box<dyn std::error::Error>> {
        let safe_patterns = vec![
            DangerPattern {
                pattern:
                    r"^(ls|pwd|whoami|date|echo|cat|less|more|head|tail|grep|find|which|man)\s"
                        .to_string(),
                risk_level: RiskLevel::Safe,
                description: "Safe read-only commands".to_string(),
                shell_specific: None,
            },
            DangerPattern {
                pattern: r"--help|--version|-h|-v".to_string(),
                risk_level: RiskLevel::Safe,
                description: "Help and version flags".to_string(),
                shell_specific: None,
            },
        ];

        Self::compile_pattern_set(safe_patterns, "safe")
    }

    /// Compile context patterns (patterns that provide execution context)
    fn compile_context_patterns() -> Result<CompiledPatterns, Box<dyn std::error::Error>> {
        let context_patterns = vec![
            DangerPattern {
                pattern: r"sudo\s+".to_string(),
                risk_level: RiskLevel::Moderate,
                description: "Elevated privileges".to_string(),
                shell_specific: None,
            },
            DangerPattern {
                pattern: r"\|\s*sudo\s+".to_string(),
                risk_level: RiskLevel::Moderate,
                description: "Piped sudo command".to_string(),
                shell_specific: None,
            },
        ];

        Self::compile_pattern_set(context_patterns, "context")
    }

    /// Compile a set of patterns into CompiledPatterns
    fn compile_pattern_set(
        patterns: Vec<DangerPattern>,
        category: &str,
    ) -> Result<CompiledPatterns, Box<dyn std::error::Error>> {
        let mut compiled_patterns = Vec::new();
        let mut by_category = std::collections::HashMap::new();
        let mut by_risk = std::collections::HashMap::new();

        for (i, pattern) in patterns.iter().enumerate() {
            let regex = Regex::new(&pattern.pattern)?;

            let compiled = AdvancedCompiledPattern {
                regex,
                risk_level: pattern.risk_level,
                description: pattern.description.clone(),
                category: category.to_string(),
                shell_specific: pattern.shell_specific,
                enabled: true,
                match_count: std::sync::atomic::AtomicU64::new(0),
                total_match_time_ns: std::sync::atomic::AtomicU64::new(0),
            };

            compiled_patterns.push(compiled);

            // Update indices
            by_category
                .entry(category.to_string())
                .or_insert_with(Vec::new)
                .push(i);

            by_risk
                .entry(pattern.risk_level)
                .or_insert_with(Vec::new)
                .push(i);
        }

        Ok(CompiledPatterns {
            patterns: compiled_patterns,
            by_category,
            by_risk,
        })
    }
}

impl Default for PatternEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default PatternEngine")
    }
}

/// Performance statistics for a pattern
#[derive(Debug, Clone)]
pub struct PatternPerformanceStats {
    pub description: String,
    pub category: String,
    pub match_count: u64,
    pub average_match_time_ns: u64,
}

impl CompiledPatterns {
    /// Get patterns by risk level
    pub fn get_by_risk(&self, risk: RiskLevel) -> Vec<&AdvancedCompiledPattern> {
        if let Some(indices) = self.by_risk.get(&risk) {
            indices.iter().map(|&i| &self.patterns[i]).collect()
        } else {
            Vec::new()
        }
    }

    /// Get patterns by category
    pub fn get_by_category(&self, category: &str) -> Vec<&AdvancedCompiledPattern> {
        if let Some(indices) = self.by_category.get(category) {
            indices.iter().map(|&i| &self.patterns[i]).collect()
        } else {
            Vec::new()
        }
    }

    /// Get enabled patterns only
    pub fn get_enabled(&self) -> Vec<&AdvancedCompiledPattern> {
        self.patterns.iter().filter(|p| p.enabled).collect()
    }
}
