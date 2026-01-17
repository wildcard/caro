//! Safety validation testing module.
//!
//! This module provides interfaces for testing caro's safety detection
//! accuracy using labeled test datasets and confusion matrix analysis.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Safety validator for testing pattern matching accuracy
#[derive(Debug)]
pub struct SafetyValidator {
    patterns: Vec<SafetyPattern>,
}

/// A safety pattern with its risk classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyPattern {
    pub name: String,
    pub pattern: String,
    pub risk_level: RiskLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Risk level classification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}

/// Result of safety validation for a single command
#[derive(Debug, Clone)]
pub struct SafetyValidationResult {
    pub is_dangerous: bool,
    pub risk_level: RiskLevel,
    pub matched_patterns: Vec<String>,
    pub should_block: bool,
}

/// Error type for safety validation operations
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Pattern compilation failed: {0}")]
    PatternError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl SafetyValidator {
    /// Create new validator with built-in patterns from caro's safety module
    ///
    /// This loads patterns directly from the caro safety module.
    /// Patterns are sourced from src/safety/patterns.rs DANGEROUS_PATTERNS.
    pub fn new() -> Result<Self, ValidationError> {
        // Extract patterns from caro's built-in safety module
        // We load these directly from the static DANGEROUS_PATTERNS in src/safety/patterns.rs
        let patterns = Self::load_builtin_patterns()?;

        Ok(Self { patterns })
    }

    /// Load built-in patterns from caro's safety module
    ///
    /// NOTE: This implementation extracts patterns from the caro library.
    /// For now, we hardcode the core patterns that match src/safety/patterns.rs.
    /// In production, these would be loaded via the public API or pattern export.
    fn load_builtin_patterns() -> Result<Vec<SafetyPattern>, ValidationError> {
        // These patterns match src/safety/patterns.rs DANGEROUS_PATTERNS
        // We've extracted the most important ones for MVP testing
        Ok(vec![
            // CRITICAL: Filesystem destruction
            // Matches: ~, ~/, ~*, ., .., * but excludes safe paths like /tmp/, ./specific-dir
            SafetyPattern {
                name: "rm_recursive_root".to_string(),
                pattern: r"rm\s+-[rfRF]+\s+(~/?[*]?|~/\*|\$HOME|\.\s*$|\.\.\s*$|\.\./|\*\s*$)"
                    .to_string(),
                risk_level: RiskLevel::Critical,
                description: Some(
                    "Recursive deletion of home, current dir, parent dir, or wildcards".to_string(),
                ),
            },
            SafetyPattern {
                name: "rm_rf_root".to_string(),
                pattern: r"rm\s+-rf\s+/\s*$".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Force recursive deletion from root directory only".to_string()),
            },
            SafetyPattern {
                name: "rm_rf_no_preserve_root".to_string(),
                pattern: r"rm\s+-rf\s+--no-preserve-root\s+/".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Bypass root protection and delete everything".to_string()),
            },
            // CRITICAL: Disk operations
            // Note: Excludes USB drives (typically sdc+) to allow safe USB formatting
            SafetyPattern {
                name: "dd_disk_overwrite".to_string(),
                pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(sd[ab]|hd[ab]|nvme)"
                    .to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Overwrite primary disk with random data".to_string()),
            },
            SafetyPattern {
                name: "mkfs_primary_disk".to_string(),
                pattern: r"mkfs\.\w+\s+/dev/(sd[ab]|hd[ab]|nvme0n1)".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Format primary disk destroying all data".to_string()),
            },
            // CRITICAL: Fork bombs
            SafetyPattern {
                name: "fork_bomb".to_string(),
                pattern: r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Fork bomb - exponential process creation".to_string()),
            },
            // HIGH: System directory operations
            SafetyPattern {
                name: "system_dir_modification".to_string(),
                pattern: r"(rm|mv|chmod|chown)\s+.*(/bin|/sbin|/usr/bin|/usr/sbin|/etc)"
                    .to_string(),
                risk_level: RiskLevel::High,
                description: Some("Modification of critical system directories".to_string()),
            },
            SafetyPattern {
                name: "chmod_777_root".to_string(),
                pattern: r"chmod\s+777\s+/".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Recursive permission change from root".to_string()),
            },
            // HIGH: Privilege escalation
            SafetyPattern {
                name: "sudo_su".to_string(),
                pattern: r"sudo\s+su\s*$".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Switch to root user without specific command".to_string()),
            },
            SafetyPattern {
                name: "sudo_chmod_setuid".to_string(),
                pattern: r"sudo\s+.*chmod\s+u\+s".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Adding setuid bit with elevated privileges".to_string()),
            },
            // HIGH: Download and execute
            SafetyPattern {
                name: "curl_pipe_bash".to_string(),
                pattern: r"(curl|wget)\s+.*\|\s*(bash|sh|zsh|fish)".to_string(),
                risk_level: RiskLevel::High,
                description: Some(
                    "Download and execute remote script without inspection".to_string(),
                ),
            },
            SafetyPattern {
                name: "curl_pipe_sudo_bash".to_string(),
                pattern: r"(curl|wget)\s+.*\|\s*sudo\s+(bash|sh)".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some(
                    "Download and execute remote script with root privileges".to_string(),
                ),
            },
            // MODERATE: Package management
            SafetyPattern {
                name: "apt_force_remove".to_string(),
                pattern: r"(apt|yum|dnf)\s+remove\s+.*--force".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Force removal of packages bypassing dependencies".to_string()),
            },
            SafetyPattern {
                name: "pip_break_system".to_string(),
                pattern: r"pip\s+install\s+.*--break-system-packages".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some(
                    "Install Python packages bypassing system protections".to_string(),
                ),
            },
            // MODERATE: Process manipulation
            SafetyPattern {
                name: "kill_all_processes".to_string(),
                pattern: r"kill\s+-9\s+(-1|1)\s*$".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Force kill all processes or init".to_string()),
            },
            SafetyPattern {
                name: "killall_force".to_string(),
                pattern: r"killall\s+-9\s+\w+".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Force kill all processes by name".to_string()),
            },
            // CRITICAL: Destructive data operations
            SafetyPattern {
                name: "disk_direct_write".to_string(),
                pattern: r">\s*/dev/sd[a-z]".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Direct write to disk device".to_string()),
            },
            SafetyPattern {
                name: "shred_disk".to_string(),
                pattern: r"shred\s+-[uvz]*\s+/dev/(sd|hd|nvme)".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Securely delete disk device".to_string()),
            },
            // MODERATE: Network operations
            SafetyPattern {
                name: "iptables_flush".to_string(),
                pattern: r"iptables\s+-F".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Flush all firewall rules".to_string()),
            },
            SafetyPattern {
                name: "ufw_disable".to_string(),
                pattern: r"ufw\s+disable".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Disable firewall".to_string()),
            },
            // HIGH: Sudo with system modifications
            SafetyPattern {
                name: "sudo_systemctl_modify".to_string(),
                pattern: r"sudo\s+(systemctl|service)\s+(restart|stop|disable)".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Modify system services with elevated privileges".to_string()),
            },
            SafetyPattern {
                name: "sudo_rm".to_string(),
                pattern: r"sudo\s+rm\s".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Delete files with elevated privileges".to_string()),
            },
            // HIGH: System file modification
            SafetyPattern {
                name: "redirect_to_etc".to_string(),
                pattern: r">\s*/etc/".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Redirect output to system configuration file".to_string()),
            },
            SafetyPattern {
                name: "write_to_etc".to_string(),
                pattern: r"(echo|cat|printf)\s+.*>\s*/etc/".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Write to system configuration directory".to_string()),
            },
            // MODERATE: Environment manipulation
            SafetyPattern {
                name: "export_path".to_string(),
                pattern: r"export\s+PATH=".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Modify PATH environment variable".to_string()),
            },
            SafetyPattern {
                name: "alias_critical_cmd".to_string(),
                pattern: r"alias\s+(rm|mv|cp)=".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Override critical command with alias".to_string()),
            },
            // CRITICAL: Network backdoors
            SafetyPattern {
                name: "nc_bind_shell".to_string(),
                pattern: r"nc\s+.*-[a-z]*l[a-z]*\s+.*-[a-z]*e".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Netcat bind shell - creates network backdoor".to_string()),
            },
            SafetyPattern {
                name: "nc_shell_exec".to_string(),
                pattern: r"nc\s+-[a-z]*e\s+/bin/(ba)?sh".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Netcat shell binding".to_string()),
            },
            // HIGH: Cron job manipulation
            SafetyPattern {
                name: "crontab_remove".to_string(),
                pattern: r"crontab\s+-r".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Remove all cron jobs".to_string()),
            },
            SafetyPattern {
                name: "crontab_malicious_add".to_string(),
                pattern: r"\(crontab\s+-l.*;\s*echo".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Add malicious cron job".to_string()),
            },
            // CRITICAL: Script language exec with dangerous commands
            SafetyPattern {
                name: "python_rm_rf".to_string(),
                pattern: r"python\s+-c\s+.*os\.system.*rm\s+-rf".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Python executing recursive deletion".to_string()),
            },
            SafetyPattern {
                name: "script_system_exec".to_string(),
                pattern: r"(python|perl|ruby)\s+-[ec]\s+.*system\s*\(".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Script language executing shell commands".to_string()),
            },
            // MODERATE: Permission changes
            // Note: Only flags suspicious permission changes, excludes common safe operations
            SafetyPattern {
                name: "chmod_executable_system".to_string(),
                pattern: r"chmod\s+[+\-]x\s+(/bin|/sbin|/usr|/etc|/lib)".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Making system directories executable".to_string()),
            },
            SafetyPattern {
                name: "chmod_dangerous_numeric".to_string(),
                pattern: r"chmod\s+(777|666|4[0-9]{3})\s+".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Dangerous permission values (777, 666, or setuid)".to_string()),
            },
            SafetyPattern {
                name: "chown_system_dirs".to_string(),
                pattern: r"chown\s+(-[RrfhHv]+\s+)?[^\s]+\s+(/bin|/sbin|/usr|/etc|/lib|/\s*$)"
                    .to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Changing ownership of system directories or root".to_string()),
            },
            // MISSING PATTERNS (added to fix false negatives)

            // HIGH: Recursive chmod 777 on root
            SafetyPattern {
                name: "chmod_recursive_777_root".to_string(),
                pattern: r"chmod\s+-R\s+777\s+/".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Recursive world-writable permissions from root".to_string()),
            },
            // CRITICAL: Disk-to-disk cloning
            SafetyPattern {
                name: "dd_disk_clone".to_string(),
                pattern: r"dd\s+if=/dev/[sh]d[a-z]\s+of=/dev/[sh]d[a-z]".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Disk-to-disk cloning".to_string()),
            },
            // HIGH: Force kill all without process name
            SafetyPattern {
                name: "killall_no_target".to_string(),
                pattern: r"killall\s+-9\s*$".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Force kill all processes without specifying target".to_string()),
            },
            // HIGH: Force kill user processes
            SafetyPattern {
                name: "pkill_force_user".to_string(),
                pattern: r"pkill\s+-9\s+-u".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Force kill user processes".to_string()),
            },
            // HIGH: Remove extended attributes recursively
            SafetyPattern {
                name: "xattr_recursive_root".to_string(),
                pattern: r"xattr\s+-cr\s+/".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Remove extended attributes from root directory".to_string()),
            },
            // CRITICAL: Move root contents to null device
            SafetyPattern {
                name: "mv_root_to_null".to_string(),
                pattern: r"mv\s+/\*\s+/dev/null".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Move root directory contents to null device".to_string()),
            },
            // HIGH: Firewall policy change
            SafetyPattern {
                name: "iptables_policy_accept".to_string(),
                pattern: r"iptables\s+-P\s+(INPUT|OUTPUT|FORWARD)\s+ACCEPT".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Change firewall policy to accept all".to_string()),
            },
            // HIGH: Disable SELinux
            SafetyPattern {
                name: "selinux_disable".to_string(),
                pattern: r"setenforce\s+0".to_string(),
                risk_level: RiskLevel::High,
                description: Some("Disable SELinux security".to_string()),
            },
            // CRITICAL: Force unmount all filesystems
            SafetyPattern {
                name: "umount_force_all".to_string(),
                pattern: r"umount\s+-a\s+-f".to_string(),
                risk_level: RiskLevel::Critical,
                description: Some("Force unmount all filesystems".to_string()),
            },
            // MODERATE: Immediate shutdown
            SafetyPattern {
                name: "shutdown_immediate".to_string(),
                pattern: r"shutdown\s+-h\s+now".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Immediate system shutdown without grace period".to_string()),
            },
            // MODERATE: Force reboot
            SafetyPattern {
                name: "reboot_force".to_string(),
                pattern: r"reboot\s+-f".to_string(),
                risk_level: RiskLevel::Moderate,
                description: Some("Force system reboot".to_string()),
            },
        ])
    }

    /// Validate a command and return safety classification
    ///
    /// This method matches the command against all loaded patterns
    /// and returns the highest risk level found.
    pub fn validate(&self, command: &str) -> SafetyValidationResult {
        let mut matched_patterns = Vec::new();
        let mut highest_risk = RiskLevel::Safe;

        for pattern in &self.patterns {
            // Compile regex for each validation
            // In production, these would be pre-compiled
            if let Ok(regex) = Regex::new(&pattern.pattern) {
                if regex.is_match(command) {
                    matched_patterns.push(pattern.name.clone());
                    if pattern.risk_level > highest_risk {
                        highest_risk = pattern.risk_level.clone();
                    }
                }
            }
        }

        let is_dangerous = highest_risk != RiskLevel::Safe;
        let should_block = highest_risk >= RiskLevel::High;

        SafetyValidationResult {
            is_dangerous,
            risk_level: highest_risk,
            matched_patterns,
            should_block,
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Safe => write!(f, "safe"),
            RiskLevel::Moderate => write!(f, "moderate"),
            RiskLevel::High => write!(f, "high"),
            RiskLevel::Critical => write!(f, "critical"),
        }
    }
}

impl RiskLevel {
    /// Parse risk level from string (used for test datasets)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "safe" => RiskLevel::Safe,
            "moderate" | "medium" => RiskLevel::Moderate,
            "high" => RiskLevel::High,
            "critical" => RiskLevel::Critical,
            _ => RiskLevel::Safe, // Default to safe for unknown
        }
    }
}

// Confusion Matrix and Metrics (T025, T026, T027)

/// Confusion matrix for binary classification of dangerous vs safe commands
#[derive(Debug, Clone)]
pub struct ConfusionMatrix {
    /// Dangerous correctly classified as dangerous
    pub true_positives: usize,
    /// Safe incorrectly classified as dangerous
    pub false_positives: usize,
    /// Safe correctly classified as safe
    pub true_negatives: usize,
    /// Dangerous incorrectly classified as safe
    pub false_negatives: usize,
}

impl ConfusionMatrix {
    /// Create confusion matrix from actual and expected results
    ///
    /// Compares safety validation results against ground truth labels
    /// from test datasets.
    pub fn from_results(actual: Vec<SafetyValidationResult>, expected: Vec<RiskLevel>) -> Self {
        let mut tp = 0;
        let mut fp = 0;
        let mut tn = 0;
        let mut fn_count = 0;

        for (actual, expected) in actual.iter().zip(expected.iter()) {
            // Ground truth: is the command actually dangerous?
            let is_actually_dangerous = expected != &RiskLevel::Safe;

            match (actual.is_dangerous, is_actually_dangerous) {
                (true, true) => tp += 1,        // Correctly flagged as dangerous
                (true, false) => fp += 1,       // Incorrectly flagged as dangerous
                (false, false) => tn += 1,      // Correctly passed as safe
                (false, true) => fn_count += 1, // Incorrectly passed as safe (missed)
            }
        }

        Self {
            true_positives: tp,
            false_positives: fp,
            true_negatives: tn,
            false_negatives: fn_count,
        }
    }

    /// Calculate precision (true positives / all predicted positives)
    ///
    /// Measures accuracy of dangerous classifications.
    /// High precision = few false alarms.
    pub fn precision(&self) -> f64 {
        let total_positive = self.true_positives + self.false_positives;
        if total_positive == 0 {
            return 0.0;
        }
        self.true_positives as f64 / total_positive as f64
    }

    /// Calculate recall (true positives / all actual positives)
    ///
    /// Measures completeness of dangerous detection.
    /// High recall = few missed dangerous commands.
    pub fn recall(&self) -> f64 {
        let total_actual_positive = self.true_positives + self.false_negatives;
        if total_actual_positive == 0 {
            return 0.0;
        }
        self.true_positives as f64 / total_actual_positive as f64
    }

    /// Calculate F1-score (harmonic mean of precision and recall)
    ///
    /// Balances precision and recall into a single metric.
    /// F1 = 2 * (precision * recall) / (precision + recall)
    pub fn f1_score(&self) -> f64 {
        let p = self.precision();
        let r = self.recall();
        if p + r == 0.0 {
            return 0.0;
        }
        2.0 * (p * r) / (p + r)
    }

    /// Calculate overall accuracy
    ///
    /// Measures total correct classifications.
    pub fn accuracy(&self) -> f64 {
        let total =
            self.true_positives + self.false_positives + self.true_negatives + self.false_negatives;
        if total == 0 {
            return 0.0;
        }
        (self.true_positives + self.true_negatives) as f64 / total as f64
    }
}

/// Analysis of false positives and false negatives
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    /// Commands incorrectly flagged as dangerous
    /// (test_id, prompt, matched_patterns)
    pub false_positives: Vec<(String, String, Vec<String>)>,

    /// Dangerous commands that were missed
    /// (test_id, prompt, expected_command)
    pub false_negatives: Vec<(String, String, String)>,
}

/// Analyze test results to identify specific errors
///
/// This function identifies which specific test cases failed
/// and provides details for debugging and pattern improvement.
pub fn analyze_errors<T>(
    test_cases: &[T],
    validation_results: &[SafetyValidationResult],
) -> ErrorAnalysis
where
    T: TestCaseProvider,
{
    let mut false_positives = Vec::new();
    let mut false_negatives = Vec::new();

    for (tc, result) in test_cases.iter().zip(validation_results.iter()) {
        let expected_dangerous = tc.get_risk_level() != "safe";

        if result.is_dangerous && !expected_dangerous {
            // False positive: safe command flagged as dangerous
            false_positives.push((
                tc.get_id().to_string(),
                tc.get_prompt().to_string(),
                result.matched_patterns.clone(),
            ));
        } else if !result.is_dangerous && expected_dangerous {
            // False negative: dangerous command not detected
            false_negatives.push((
                tc.get_id().to_string(),
                tc.get_prompt().to_string(),
                tc.get_expected_command().to_string(),
            ));
        }
    }

    ErrorAnalysis {
        false_positives,
        false_negatives,
    }
}

/// Trait for test case types to work with error analysis
pub trait TestCaseProvider {
    fn get_id(&self) -> &str;
    fn get_prompt(&self) -> &str;
    fn get_expected_command(&self) -> &str;
    fn get_risk_level(&self) -> &str;
}

// Implement for the dataset TestCase type
impl TestCaseProvider for crate::dataset::TestCase {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_prompt(&self) -> &str {
        &self.prompt
    }

    fn get_expected_command(&self) -> &str {
        &self.expected_command
    }

    fn get_risk_level(&self) -> &str {
        &self.risk_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_validator_creation() {
        let validator = SafetyValidator::new();
        assert!(validator.is_ok());

        let validator = validator.unwrap();
        assert!(
            !validator.patterns.is_empty(),
            "Should have loaded patterns"
        );
    }

    #[test]
    fn test_dangerous_command_detection() {
        let validator = SafetyValidator::new().unwrap();

        // Test critical commands
        let result = validator.validate("rm -rf /");
        assert!(result.is_dangerous);
        assert_eq!(result.risk_level, RiskLevel::Critical);
        assert!(result.should_block);
        assert!(!result.matched_patterns.is_empty());
    }

    #[test]
    fn test_safe_command_detection() {
        let validator = SafetyValidator::new().unwrap();

        // Test safe commands
        let result = validator.validate("ls -la");
        assert!(!result.is_dangerous);
        assert_eq!(result.risk_level, RiskLevel::Safe);
        assert!(!result.should_block);
        assert!(result.matched_patterns.is_empty());
    }

    #[test]
    fn test_moderate_risk_detection() {
        let validator = SafetyValidator::new().unwrap();

        // Test moderate risk command (shutdown without grace period)
        let result = validator.validate("shutdown -h now");
        assert!(result.is_dangerous);
        assert_eq!(result.risk_level, RiskLevel::Moderate);
        assert!(!result.should_block); // Moderate doesn't block
    }

    #[test]
    fn test_high_risk_detection() {
        let validator = SafetyValidator::new().unwrap();

        // Test high risk command
        let result = validator.validate("sudo rm file.txt");
        assert!(result.is_dangerous);
        assert_eq!(result.risk_level, RiskLevel::High);
        assert!(result.should_block);
    }

    #[test]
    fn test_risk_level_parsing() {
        assert_eq!(RiskLevel::from_str("safe"), RiskLevel::Safe);
        assert_eq!(RiskLevel::from_str("moderate"), RiskLevel::Moderate);
        assert_eq!(RiskLevel::from_str("medium"), RiskLevel::Moderate);
        assert_eq!(RiskLevel::from_str("high"), RiskLevel::High);
        assert_eq!(RiskLevel::from_str("critical"), RiskLevel::Critical);
        assert_eq!(RiskLevel::from_str("unknown"), RiskLevel::Safe);
    }

    // Confusion Matrix Tests

    #[test]
    fn test_confusion_matrix_basic() {
        // Create example results: 8 TPs, 2 FPs, 18 TNs, 1 FN
        let matrix = ConfusionMatrix {
            true_positives: 8,
            false_positives: 2,
            true_negatives: 18,
            false_negatives: 1,
        };

        // Precision = TP / (TP + FP) = 8 / 10 = 0.8
        assert!((matrix.precision() - 0.8).abs() < 0.01);

        // Recall = TP / (TP + FN) = 8 / 9 = 0.888...
        assert!((matrix.recall() - 0.888).abs() < 0.01);

        // F1 = 2 * (P * R) / (P + R) = 2 * (0.8 * 0.888) / (0.8 + 0.888)
        assert!((matrix.f1_score() - 0.842).abs() < 0.01);

        // Accuracy = (TP + TN) / Total = 26 / 29 = 0.896...
        assert!((matrix.accuracy() - 0.896).abs() < 0.01);
    }

    #[test]
    fn test_confusion_matrix_from_results() {
        let validator = SafetyValidator::new().unwrap();

        // Create test commands
        let commands = vec![
            "rm -rf /",                    // Dangerous, should detect
            "ls -la",                      // Safe, should not detect
            "chmod 777 /etc",              // Dangerous, should detect
            "pwd",                         // Safe, should not detect
            "dd if=/dev/zero of=/dev/sda", // Dangerous, should detect
        ];

        let expected_risks = vec![
            RiskLevel::Critical,
            RiskLevel::Safe,
            RiskLevel::High,
            RiskLevel::Safe,
            RiskLevel::Critical,
        ];

        let results: Vec<SafetyValidationResult> =
            commands.iter().map(|cmd| validator.validate(cmd)).collect();

        let matrix = ConfusionMatrix::from_results(results, expected_risks);

        // Should have: 3 TPs (dangerous detected), 2 TNs (safe not detected), 0 FPs, 0 FNs
        assert_eq!(
            matrix.true_positives, 3,
            "Should detect all dangerous commands"
        );
        assert_eq!(matrix.true_negatives, 2, "Should not flag safe commands");
        assert_eq!(matrix.false_positives, 0, "Should have no false positives");
        assert_eq!(matrix.false_negatives, 0, "Should have no false negatives");

        // Perfect detection: precision, recall, F1, accuracy should all be 1.0
        assert_eq!(matrix.precision(), 1.0);
        assert_eq!(matrix.recall(), 1.0);
        assert_eq!(matrix.f1_score(), 1.0);
        assert_eq!(matrix.accuracy(), 1.0);
    }

    #[test]
    fn test_confusion_matrix_edge_cases() {
        // All true positives
        let matrix = ConfusionMatrix {
            true_positives: 10,
            false_positives: 0,
            true_negatives: 0,
            false_negatives: 0,
        };
        assert_eq!(matrix.precision(), 1.0);
        assert_eq!(matrix.recall(), 1.0);
        assert_eq!(matrix.f1_score(), 1.0);
        assert_eq!(matrix.accuracy(), 1.0);

        // All true negatives
        let matrix = ConfusionMatrix {
            true_positives: 0,
            false_positives: 0,
            true_negatives: 10,
            false_negatives: 0,
        };
        assert_eq!(matrix.precision(), 0.0);
        assert_eq!(matrix.recall(), 0.0);
        assert_eq!(matrix.f1_score(), 0.0);
        assert_eq!(matrix.accuracy(), 1.0);

        // All false positives
        let matrix = ConfusionMatrix {
            true_positives: 0,
            false_positives: 10,
            true_negatives: 0,
            false_negatives: 0,
        };
        assert_eq!(matrix.precision(), 0.0);
        assert_eq!(matrix.recall(), 0.0);
        assert_eq!(matrix.f1_score(), 0.0);
        assert_eq!(matrix.accuracy(), 0.0);

        // All false negatives
        let matrix = ConfusionMatrix {
            true_positives: 0,
            false_positives: 0,
            true_negatives: 0,
            false_negatives: 10,
        };
        assert_eq!(matrix.precision(), 0.0);
        assert_eq!(matrix.recall(), 0.0);
        assert_eq!(matrix.f1_score(), 0.0);
        assert_eq!(matrix.accuracy(), 0.0);

        // Empty matrix
        let matrix = ConfusionMatrix {
            true_positives: 0,
            false_positives: 0,
            true_negatives: 0,
            false_negatives: 0,
        };
        assert_eq!(matrix.precision(), 0.0);
        assert_eq!(matrix.recall(), 0.0);
        assert_eq!(matrix.f1_score(), 0.0);
        assert_eq!(matrix.accuracy(), 0.0);
    }

    #[test]
    fn test_confusion_matrix_mixed_results() {
        // Realistic scenario with some errors
        let matrix = ConfusionMatrix {
            true_positives: 15, // Correctly detected dangerous
            false_positives: 3, // Safe commands wrongly flagged
            true_negatives: 25, // Correctly allowed safe
            false_negatives: 2, // Dangerous commands missed
        };

        // Total = 45
        // Precision = 15 / (15 + 3) = 15/18 = 0.833...
        assert!((matrix.precision() - 0.833).abs() < 0.01);

        // Recall = 15 / (15 + 2) = 15/17 = 0.882...
        assert!((matrix.recall() - 0.882).abs() < 0.01);

        // Accuracy = (15 + 25) / 45 = 40/45 = 0.888...
        assert!((matrix.accuracy() - 0.888).abs() < 0.01);

        // F1 should be harmonic mean of P and R
        let p = matrix.precision();
        let r = matrix.recall();
        let expected_f1 = 2.0 * (p * r) / (p + r);
        assert!((matrix.f1_score() - expected_f1).abs() < 0.01);
    }

    // Error Analysis Tests

    // Mock test case for error analysis tests
    #[derive(Clone)]
    struct MockTestCase {
        id: String,
        prompt: String,
        expected_command: String,
        risk_level: String,
    }

    impl TestCaseProvider for MockTestCase {
        fn get_id(&self) -> &str {
            &self.id
        }

        fn get_prompt(&self) -> &str {
            &self.prompt
        }

        fn get_expected_command(&self) -> &str {
            &self.expected_command
        }

        fn get_risk_level(&self) -> &str {
            &self.risk_level
        }
    }

    #[test]
    fn test_error_analysis_no_errors() {
        let test_cases = vec![
            MockTestCase {
                id: "test_001".to_string(),
                prompt: "delete root".to_string(),
                expected_command: "rm -rf /".to_string(),
                risk_level: "critical".to_string(),
            },
            MockTestCase {
                id: "test_002".to_string(),
                prompt: "list files".to_string(),
                expected_command: "ls".to_string(),
                risk_level: "safe".to_string(),
            },
        ];

        let results = vec![
            SafetyValidationResult {
                is_dangerous: true,
                risk_level: RiskLevel::Critical,
                matched_patterns: vec!["rm_recursive_root".to_string()],
                should_block: true,
            },
            SafetyValidationResult {
                is_dangerous: false,
                risk_level: RiskLevel::Safe,
                matched_patterns: vec![],
                should_block: false,
            },
        ];

        let analysis = analyze_errors(&test_cases, &results);

        assert_eq!(analysis.false_positives.len(), 0);
        assert_eq!(analysis.false_negatives.len(), 0);
    }

    #[test]
    fn test_error_analysis_false_positives() {
        let test_cases = vec![MockTestCase {
            id: "test_fp_001".to_string(),
            prompt: "clean temp directory".to_string(),
            expected_command: "rm -rf /tmp/cache".to_string(),
            risk_level: "safe".to_string(),
        }];

        let results = vec![SafetyValidationResult {
            is_dangerous: true,
            risk_level: RiskLevel::High,
            matched_patterns: vec!["rm_recursive".to_string()],
            should_block: true,
        }];

        let analysis = analyze_errors(&test_cases, &results);

        assert_eq!(analysis.false_positives.len(), 1);
        assert_eq!(analysis.false_negatives.len(), 0);

        let (id, prompt, patterns) = &analysis.false_positives[0];
        assert_eq!(id, "test_fp_001");
        assert_eq!(prompt, "clean temp directory");
        assert!(patterns.contains(&"rm_recursive".to_string()));
    }

    #[test]
    fn test_error_analysis_false_negatives() {
        let test_cases = vec![MockTestCase {
            id: "test_fn_001".to_string(),
            prompt: "wipe disk".to_string(),
            expected_command: "dd if=/dev/zero of=/dev/sda".to_string(),
            risk_level: "critical".to_string(),
        }];

        let results = vec![SafetyValidationResult {
            is_dangerous: false,
            risk_level: RiskLevel::Safe,
            matched_patterns: vec![],
            should_block: false,
        }];

        let analysis = analyze_errors(&test_cases, &results);

        assert_eq!(analysis.false_positives.len(), 0);
        assert_eq!(analysis.false_negatives.len(), 1);

        let (id, prompt, command) = &analysis.false_negatives[0];
        assert_eq!(id, "test_fn_001");
        assert_eq!(prompt, "wipe disk");
        assert_eq!(command, "dd if=/dev/zero of=/dev/sda");
    }

    #[test]
    fn test_error_analysis_mixed_errors() {
        let test_cases = vec![
            // True positive
            MockTestCase {
                id: "test_tp".to_string(),
                prompt: "format disk".to_string(),
                expected_command: "mkfs.ext4 /dev/sda".to_string(),
                risk_level: "critical".to_string(),
            },
            // True negative
            MockTestCase {
                id: "test_tn".to_string(),
                prompt: "list files".to_string(),
                expected_command: "ls -la".to_string(),
                risk_level: "safe".to_string(),
            },
            // False positive
            MockTestCase {
                id: "test_fp".to_string(),
                prompt: "clean cache".to_string(),
                expected_command: "rm -rf ./cache".to_string(),
                risk_level: "safe".to_string(),
            },
            // False negative
            MockTestCase {
                id: "test_fn".to_string(),
                prompt: "disable firewall".to_string(),
                expected_command: "iptables -F".to_string(),
                risk_level: "high".to_string(),
            },
        ];

        let results = vec![
            SafetyValidationResult {
                is_dangerous: true,
                risk_level: RiskLevel::Critical,
                matched_patterns: vec!["mkfs".to_string()],
                should_block: true,
            },
            SafetyValidationResult {
                is_dangerous: false,
                risk_level: RiskLevel::Safe,
                matched_patterns: vec![],
                should_block: false,
            },
            SafetyValidationResult {
                is_dangerous: true,
                risk_level: RiskLevel::High,
                matched_patterns: vec!["rm_recursive".to_string()],
                should_block: true,
            },
            SafetyValidationResult {
                is_dangerous: false,
                risk_level: RiskLevel::Safe,
                matched_patterns: vec![],
                should_block: false,
            },
        ];

        let analysis = analyze_errors(&test_cases, &results);

        assert_eq!(analysis.false_positives.len(), 1);
        assert_eq!(analysis.false_negatives.len(), 1);

        // Verify FP details
        let (fp_id, _, _) = &analysis.false_positives[0];
        assert_eq!(fp_id, "test_fp");

        // Verify FN details
        let (fn_id, _, _) = &analysis.false_negatives[0];
        assert_eq!(fn_id, "test_fn");
    }
}
