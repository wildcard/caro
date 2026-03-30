//! Safety module - Command safety validation and risk assessment
//!
//! This module provides comprehensive validation of shell commands to detect
//! potentially dangerous operations before execution.
//!
//! # Architecture
//!
//! - **Decision Pipeline**: 4-stage evaluation inspired by Claude Code's auto mode
//! - **Safe Patterns**: Known-safe commands auto-approved without validation
//! - **Pattern Database**: 60+ pre-compiled regex patterns covering Critical/High/Moderate risks
//! - **Context-Aware Matching**: Distinguishes between dangerous commands and safe string literals
//! - **Performance**: Patterns compiled once at startup using `once_cell::Lazy` (30x speedup)
//! - **Extensibility**: Supports custom patterns via `SafetyConfig`
//!
//! # Example
//!
//! ```no_run
//! use caro::safety::{SafetyValidator, SafetyConfig};
//! use caro::models::ShellType;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let validator = SafetyValidator::new(SafetyConfig::moderate())?;
//! let result = validator.validate_command("rm -rf /", ShellType::Bash).await?;
//!
//! assert!(!result.allowed); // Dangerous command blocked
//! assert_eq!(result.risk_level, caro::models::RiskLevel::Critical);
//! # Ok(())
//! # }
//! ```

mod patterns;

use serde::{Deserialize, Serialize};

use crate::models::{RiskLevel, SafetyLevel, ShellType};

pub use patterns::{
    get_compiled_patterns_for_shell, get_patterns_by_risk, get_patterns_for_shell, is_known_safe,
    validate_patterns,
};

/// Main safety validator for analyzing command safety
#[derive(Debug)]
pub struct SafetyValidator {
    config: SafetyConfig,
    /// Original pattern definitions (used for Debug output, not validation)
    #[allow(dead_code)]
    patterns: Vec<DangerPattern>,
    /// Cached compiled regex patterns for performance
    compiled_patterns: Vec<(regex::Regex, RiskLevel, String)>,
}

/// Configuration for safety validation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub safety_level: SafetyLevel,
    pub max_command_length: usize,
    pub custom_patterns: Vec<DangerPattern>,
    pub allowlist_patterns: Vec<String>,
}

/// Result of safety validation for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub allowed: bool,
    pub risk_level: RiskLevel,
    pub explanation: String,
    pub warnings: Vec<String>,
    pub matched_patterns: Vec<String>,
    pub confidence_score: f32,
}

/// Pattern definition for dangerous command detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerPattern {
    pub pattern: String,
    pub risk_level: RiskLevel,
    pub description: String,
    pub shell_specific: Option<ShellType>,
}

impl SafetyValidator {
    /// Create new validator with given configuration
    pub fn new(config: SafetyConfig) -> Result<Self, ValidationError> {
        // Validate built-in patterns at startup and log any errors
        if let Err(errors) = patterns::validate_patterns() {
            for error in &errors {
                eprintln!("WARN: Invalid built-in safety pattern: {}", error);
            }
            // Continue execution - patterns are pre-validated during development,
            // this is a defensive check for runtime detection of any issues
        }

        // Validate configuration
        if config.max_command_length == 0 {
            return Err(ValidationError::InvalidConfig {
                message: format!(
                    "max_command_length must be positive, got {}",
                    config.max_command_length
                ),
            });
        }

        // Validate custom patterns can compile
        for pattern in &config.custom_patterns {
            if let Err(e) = regex::Regex::new(&pattern.pattern) {
                eprintln!(
                    "WARN: Invalid custom safety pattern '{}': {}",
                    pattern.pattern, e
                );
                return Err(ValidationError::PatternError {
                    pattern: format!("{}: {}", pattern.pattern, e),
                });
            }
        }

        // Pre-compile all custom patterns for performance
        let mut compiled_patterns = Vec::new();
        for pattern in &config.custom_patterns {
            match regex::Regex::new(&pattern.pattern) {
                Ok(regex) => {
                    compiled_patterns.push((
                        regex,
                        pattern.risk_level,
                        pattern.description.clone(),
                    ));
                }
                Err(e) => {
                    return Err(ValidationError::PatternError {
                        pattern: format!("{}: {}", pattern.pattern, e),
                    });
                }
            }
        }

        let patterns = config.custom_patterns.clone();

        Ok(Self {
            config,
            patterns,
            compiled_patterns,
        })
    }

    /// Check if command contains dangerous pattern in executable context
    ///
    /// This function prevents false positives by distinguishing between dangerous
    /// commands and safe string literals. For example:
    /// - `rm -rf /` → dangerous (returns true)
    /// - `echo 'rm -rf /' > script.sh` → safe (returns false, in quotes)
    ///
    /// # Algorithm
    ///
    /// For each pattern match, counts unescaped quotes before the match position.
    /// If an odd number of quotes precedes the match, it's inside a string literal.
    ///
    /// # Limitations
    ///
    /// - Does not handle nested quotes: `echo "it's safe"` might be misclassified
    /// - Does not handle hex escapes: `\x27` (single quote)
    /// - Does not handle double-escaped quotes: `\\'`
    ///
    /// These edge cases are rare in practice and can be addressed if needed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let pattern = Regex::new(r"rm\s+-rf\s+/").unwrap();
    /// assert!(is_dangerous_in_context("rm -rf /", &pattern));
    /// assert!(!is_dangerous_in_context("echo 'rm -rf /'", &pattern));
    /// ```
    fn is_dangerous_in_context(command: &str, pattern_regex: &regex::Regex) -> bool {
        if !pattern_regex.is_match(command) {
            return false;
        }

        // Find all matches and check if any are in executable context
        for mat in pattern_regex.find_iter(command) {
            let match_start = mat.start();
            let before = &command[..match_start];

            // Count unescaped quotes before the match
            let single_quotes = before.matches('\'').count() - before.matches("\\'").count();
            let double_quotes = before.matches('"').count() - before.matches("\\\"").count();

            // If odd number of quotes, we're inside a string literal
            if single_quotes % 2 == 1 || double_quotes % 2 == 1 {
                continue;
            }

            // Match is in executable context (not quoted)
            return true;
        }

        false
    }

    /// Validate a single command for safety
    pub async fn validate_command(
        &self,
        command: &str,
        shell: ShellType,
    ) -> Result<ValidationResult, ValidationError> {
        // Check command length
        if command.len() > self.config.max_command_length {
            return Ok(ValidationResult {
                allowed: false,
                risk_level: RiskLevel::Moderate,
                explanation: format!(
                    "Command exceeds maximum length of {} characters",
                    self.config.max_command_length
                ),
                warnings: vec![format!(
                    "Command is {} characters long (max: {})",
                    command.len(),
                    self.config.max_command_length
                )],
                matched_patterns: vec![],
                confidence_score: 1.0,
            });
        }

        // Check allowlist patterns first
        for allow_pattern in &self.config.allowlist_patterns {
            if let Ok(regex) = regex::Regex::new(allow_pattern) {
                if regex.is_match(command) {
                    return Ok(ValidationResult {
                        allowed: true,
                        risk_level: RiskLevel::Safe,
                        explanation: "Command matches allowlist pattern".to_string(),
                        warnings: vec![],
                        matched_patterns: vec![allow_pattern.clone()],
                        confidence_score: 1.0,
                    });
                }
            }
        }

        // Get pre-compiled patterns for this shell type
        let built_in_patterns = patterns::get_compiled_patterns_for_shell(shell);
        let mut matched = Vec::new();
        let mut highest_risk = RiskLevel::Safe;
        let mut warnings = Vec::new();

        // Check against built-in compiled patterns (fast!)
        for (regex, risk_level, description, _) in built_in_patterns {
            if Self::is_dangerous_in_context(command, regex) {
                // Normalize to lowercase for consistent .contains() matching in tests
                // Original case is preserved in warnings for readability
                matched.push(description.to_lowercase());
                if *risk_level > highest_risk {
                    highest_risk = *risk_level;
                }
                warnings.push(format!("{}: {}", risk_level, description));
            }
        }

        // Check pre-compiled custom patterns
        for (regex, risk_level, description) in &self.compiled_patterns {
            if Self::is_dangerous_in_context(command, regex) {
                // Normalize to lowercase for consistent .contains() matching in tests
                // Original case is preserved in warnings for readability
                matched.push(description.to_lowercase());
                if *risk_level > highest_risk {
                    highest_risk = *risk_level;
                }
                warnings.push(format!("{}: {}", risk_level, description));
            }
        }

        // Determine if command is allowed based on safety level
        // Commands are not allowed if either blocked OR require confirmation
        let requires_confirm = highest_risk.requires_confirmation(self.config.safety_level);
        let is_blocked = highest_risk.is_blocked(self.config.safety_level);
        let allowed = !is_blocked && !requires_confirm;

        // Generate explanation
        let explanation = if matched.is_empty() {
            "No dangerous patterns detected".to_string()
        } else {
            // Include specific risk types in explanation
            let risk_keywords: Vec<&str> = matched
                .iter()
                .flat_map(|desc| {
                    let lower = desc.to_lowercase();
                    let mut keywords = Vec::new();
                    if lower.contains("delet") {
                        keywords.push("deletion");
                    }
                    if lower.contains("remov") {
                        keywords.push("removal");
                    }
                    if lower.contains("recursive") {
                        keywords.push("recursive");
                    }
                    if lower.contains("privilege")
                        || lower.contains("root")
                        || lower.contains("sudo")
                    {
                        keywords.push("privilege escalation");
                    }
                    if lower.contains("network") || lower.contains("backdoor") {
                        keywords.push("network");
                    }
                    if lower.contains("disk") || lower.contains("format") {
                        keywords.push("disk");
                    }
                    keywords
                })
                .collect();

            let risk_types = if risk_keywords.is_empty() {
                String::new()
            } else {
                let unique: std::collections::HashSet<_> = risk_keywords.into_iter().collect();
                format!(" ({})", unique.into_iter().collect::<Vec<_>>().join(", "))
            };

            format!(
                "Detected {} dangerous pattern(s) at {} risk level{}",
                matched.len(),
                highest_risk,
                risk_types
            )
        };

        // Calculate confidence score based on pattern matches
        let confidence_score = if matched.is_empty() {
            0.95 // High confidence for safe commands
        } else {
            1.0 // Very confident about dangerous patterns
        };

        Ok(ValidationResult {
            allowed,
            risk_level: highest_risk,
            explanation,
            warnings,
            matched_patterns: matched,
            confidence_score,
        })
    }

    /// Validate multiple commands efficiently
    pub async fn validate_batch(
        &self,
        commands: &[String],
        shell: ShellType,
    ) -> Result<Vec<ValidationResult>, ValidationError> {
        let mut results = Vec::with_capacity(commands.len());

        for command in commands {
            let result = self.validate_command(command, shell).await?;
            results.push(result);
        }

        Ok(results)
    }
}

impl SafetyConfig {
    /// Create strict safety configuration (blocks High and Critical)
    pub fn strict() -> Self {
        Self {
            safety_level: SafetyLevel::Strict,
            max_command_length: 1000,
            custom_patterns: Vec::new(),
            allowlist_patterns: Vec::new(),
        }
    }

    /// Create moderate safety configuration (blocks Critical only)
    pub fn moderate() -> Self {
        Self {
            safety_level: SafetyLevel::Moderate,
            max_command_length: 5000,
            custom_patterns: Vec::new(),
            allowlist_patterns: Vec::new(),
        }
    }

    /// Create permissive safety configuration (warns but allows all)
    pub fn permissive() -> Self {
        Self {
            safety_level: SafetyLevel::Permissive,
            max_command_length: 10000,
            custom_patterns: Vec::new(),
            allowlist_patterns: Vec::new(),
        }
    }

    /// Create safety configuration from a SafetyLevel
    ///
    /// This is the primary way to convert CLI safety level to backend config.
    pub fn from_level(level: SafetyLevel) -> Self {
        match level {
            SafetyLevel::Strict => Self::strict(),
            SafetyLevel::Moderate => Self::moderate(),
            SafetyLevel::Permissive => Self::permissive(),
        }
    }

    /// Add custom dangerous pattern with deferred validation
    ///
    /// This method adds a pattern to the config but performs full validation
    /// only when `SafetyValidator::new()` is called. This allows building
    /// configurations that may contain invalid patterns (e.g., from external sources)
    /// and handling all validation errors at once during validator creation.
    ///
    /// # Behavior
    ///
    /// - Returns `Ok(())` if the pattern regex compiles successfully
    /// - Returns `Err(ValidationError::PatternError)` if regex is invalid
    /// - **Important**: Even on error, the pattern is still added to `custom_patterns`
    ///   to allow deferred validation by `SafetyValidator::new()`
    ///
    /// # Example
    ///
    /// ```
    /// use caro::safety::{SafetyConfig, DangerPattern};
    /// use caro::models::RiskLevel;
    ///
    /// let mut config = SafetyConfig::default();
    /// let pattern = DangerPattern {
    ///     pattern: r"deploy.*production".to_string(),
    ///     risk_level: RiskLevel::High,
    ///     description: "Production deployment".to_string(),
    ///     shell_specific: None,
    /// };
    ///
    /// // Pattern is validated here, but also during SafetyValidator::new()
    /// let result = config.add_custom_pattern(pattern);
    /// assert!(result.is_ok());
    /// ```
    pub fn add_custom_pattern(&mut self, pattern: DangerPattern) -> Result<(), ValidationError> {
        // Quick validation check for immediate feedback
        if let Err(e) = regex::Regex::new(&pattern.pattern) {
            // Add pattern anyway for deferred validation (see method docs)
            self.custom_patterns.push(pattern);
            return Err(ValidationError::PatternError {
                pattern: format!("{}: {}", &self.custom_patterns.last().unwrap().pattern, e),
            });
        }

        self.custom_patterns.push(pattern);
        Ok(())
    }

    /// Add allowlist pattern
    pub fn add_allowlist_pattern(&mut self, pattern: impl Into<String>) {
        self.allowlist_patterns.push(pattern.into());
    }
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            safety_level: SafetyLevel::Moderate,
            max_command_length: 1000,
            custom_patterns: Vec::new(),
            allowlist_patterns: Vec::new(),
        }
    }
}

/// Errors that can occur during safety validation
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ValidationError {
    #[error("Safety validation not implemented yet")]
    NotImplemented,

    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Pattern compilation failed: {pattern}")]
    PatternError { pattern: String },

    #[error("Validation timeout")]
    Timeout,

    #[error("Internal validation error: {message}")]
    Internal { message: String },
}

// Types are already public, no re-export needed

/// Tiered safety decision inspired by Claude Code's auto mode.
///
/// Instead of a binary allow/block, commands get one of four outcomes.
/// This enables graceful degradation: uncertain commands get confirmation
/// prompts instead of hard blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyDecision {
    /// Safe to execute immediately (matches known-safe pattern or allowlist)
    Allow {
        reason: String,
    },
    /// Probably safe, but show a warning to the user
    AllowWithWarning {
        warning: String,
    },
    /// Risky command — ask user for confirmation before executing
    AskConfirmation {
        risk_level: RiskLevel,
        explanation: String,
        matched_patterns: Vec<String>,
    },
    /// Dangerous command — refuse to execute
    Block {
        risk_level: RiskLevel,
        explanation: String,
        matched_patterns: Vec<String>,
    },
}

impl SafetyDecision {
    /// Whether this decision allows execution (with or without warning)
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. } | Self::AllowWithWarning { .. })
    }

    /// Whether this decision requires user interaction before proceeding
    pub fn needs_confirmation(&self) -> bool {
        matches!(self, Self::AskConfirmation { .. })
    }

    /// Whether this decision blocks execution entirely
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Block { .. })
    }
}

/// Decision pipeline that evaluates commands through multiple stages.
///
/// Inspired by Claude Code's auto mode decision flow:
/// 1. Known-safe patterns → auto-approve
/// 2. User allowlist → auto-approve
/// 3. Danger pattern matching → block or confirm
/// 4. Default → allow with no warnings
///
/// This replaces direct use of `SafetyValidator` for the primary flow,
/// adding a fast-path for safe commands and nuanced outcomes for risky ones.
pub struct DecisionPipeline {
    validator: SafetyValidator,
}

impl DecisionPipeline {
    /// Create a new decision pipeline wrapping an existing validator
    pub fn new(validator: SafetyValidator) -> Self {
        Self { validator }
    }

    /// Create a pipeline with default moderate safety config
    pub fn default_config() -> Result<Self, ValidationError> {
        let validator = SafetyValidator::new(SafetyConfig::default())?;
        Ok(Self { validator })
    }

    /// Evaluate a command through the full decision pipeline
    pub async fn evaluate(
        &self,
        command: &str,
        shell: ShellType,
    ) -> Result<SafetyDecision, ValidationError> {
        // Stage 1: Check known-safe patterns (fast path)
        if let Some(reason) = is_known_safe(command) {
            return Ok(SafetyDecision::Allow {
                reason: reason.to_string(),
            });
        }

        // Stage 2: Check user allowlist (from SafetyConfig)
        for allow_pattern in &self.validator.config.allowlist_patterns {
            if let Ok(regex) = regex::Regex::new(allow_pattern) {
                if regex.is_match(command) {
                    return Ok(SafetyDecision::Allow {
                        reason: "Command matches user allowlist".to_string(),
                    });
                }
            }
        }

        // Stage 3: Full danger pattern validation
        let result = self.validator.validate_command(command, shell).await?;

        if result.matched_patterns.is_empty() {
            // No dangerous patterns matched — safe to execute
            return Ok(SafetyDecision::Allow {
                reason: "No dangerous patterns detected".to_string(),
            });
        }

        // Determine outcome based on risk level and safety config
        let safety_level = self.validator.config.safety_level;

        if result.risk_level.is_blocked(safety_level) {
            Ok(SafetyDecision::Block {
                risk_level: result.risk_level,
                explanation: result.explanation,
                matched_patterns: result.matched_patterns,
            })
        } else if result.risk_level.requires_confirmation(safety_level) {
            Ok(SafetyDecision::AskConfirmation {
                risk_level: result.risk_level,
                explanation: result.explanation,
                matched_patterns: result.matched_patterns,
            })
        } else {
            // Patterns matched but below confirmation threshold — warn only
            Ok(SafetyDecision::AllowWithWarning {
                warning: result.explanation,
            })
        }
    }

    /// Access the underlying validator for direct use
    pub fn validator(&self) -> &SafetyValidator {
        &self.validator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pipeline(safety_level: SafetyLevel) -> DecisionPipeline {
        let config = SafetyConfig::from_level(safety_level);
        let validator = SafetyValidator::new(config).unwrap();
        DecisionPipeline::new(validator)
    }

    // --- SafetyDecision tests ---

    #[test]
    fn test_safety_decision_is_allowed() {
        let allow = SafetyDecision::Allow {
            reason: "test".to_string(),
        };
        assert!(allow.is_allowed());
        assert!(!allow.needs_confirmation());
        assert!(!allow.is_blocked());
    }

    #[test]
    fn test_safety_decision_allow_with_warning() {
        let warn = SafetyDecision::AllowWithWarning {
            warning: "careful".to_string(),
        };
        assert!(warn.is_allowed());
        assert!(!warn.needs_confirmation());
        assert!(!warn.is_blocked());
    }

    #[test]
    fn test_safety_decision_ask_confirmation() {
        let ask = SafetyDecision::AskConfirmation {
            risk_level: RiskLevel::High,
            explanation: "risky".to_string(),
            matched_patterns: vec!["test".to_string()],
        };
        assert!(!ask.is_allowed());
        assert!(ask.needs_confirmation());
        assert!(!ask.is_blocked());
    }

    #[test]
    fn test_safety_decision_block() {
        let block = SafetyDecision::Block {
            risk_level: RiskLevel::Critical,
            explanation: "dangerous".to_string(),
            matched_patterns: vec!["test".to_string()],
        };
        assert!(!block.is_allowed());
        assert!(!block.needs_confirmation());
        assert!(block.is_blocked());
    }

    // --- DecisionPipeline tests ---

    #[tokio::test]
    async fn test_pipeline_allows_safe_commands() {
        let pipeline = make_pipeline(SafetyLevel::Strict);

        let decision = pipeline.evaluate("ls -la", ShellType::Bash).await.unwrap();
        assert!(decision.is_allowed(), "ls -la should be auto-allowed");

        let decision = pipeline.evaluate("git status", ShellType::Bash).await.unwrap();
        assert!(decision.is_allowed(), "git status should be auto-allowed");

        let decision = pipeline.evaluate("cargo test", ShellType::Bash).await.unwrap();
        assert!(decision.is_allowed(), "cargo test should be auto-allowed");
    }

    #[tokio::test]
    async fn test_pipeline_blocks_critical_commands() {
        let pipeline = make_pipeline(SafetyLevel::Moderate);

        let decision = pipeline.evaluate("rm -rf /", ShellType::Bash).await.unwrap();
        assert!(decision.is_blocked(), "rm -rf / should be blocked");
    }

    #[tokio::test]
    async fn test_pipeline_asks_confirmation_for_risky() {
        let pipeline = make_pipeline(SafetyLevel::Moderate);

        let decision = pipeline
            .evaluate("git push --force origin main", ShellType::Bash)
            .await
            .unwrap();
        // With moderate safety, HIGH risk should require confirmation
        assert!(
            decision.needs_confirmation() || decision.is_blocked(),
            "Force push should need confirmation or be blocked"
        );
    }

    #[tokio::test]
    async fn test_pipeline_safe_commands_bypass_validation() {
        // Even with strict safety, known-safe commands should pass through
        let pipeline = make_pipeline(SafetyLevel::Strict);

        let decision = pipeline.evaluate("pwd", ShellType::Bash).await.unwrap();
        assert!(
            decision.is_allowed(),
            "pwd should be allowed even in strict mode"
        );
    }

    #[tokio::test]
    async fn test_pipeline_chained_commands_not_safe() {
        let pipeline = make_pipeline(SafetyLevel::Strict);

        // Even though "ls" is safe, "ls && rm -rf /" should not be auto-allowed
        let decision = pipeline
            .evaluate("ls && rm -rf /", ShellType::Bash)
            .await
            .unwrap();
        assert!(
            !decision.is_allowed() || decision == SafetyDecision::AllowWithWarning { warning: String::new() },
            "Chained commands with dangerous content should not be auto-allowed: {:?}",
            decision
        );
    }

    #[tokio::test]
    async fn test_pipeline_unknown_command_allowed() {
        let pipeline = make_pipeline(SafetyLevel::Moderate);

        // An unknown but harmless command should be allowed
        let decision = pipeline
            .evaluate("my-custom-tool --output file.txt", ShellType::Bash)
            .await
            .unwrap();
        assert!(
            decision.is_allowed(),
            "Unknown harmless commands should be allowed"
        );
    }

    #[tokio::test]
    async fn test_pipeline_allowlist_overrides() {
        let mut config = SafetyConfig::strict();
        config.add_allowlist_pattern(r"^docker\s+run\s+--rm");
        let validator = SafetyValidator::new(config).unwrap();
        let pipeline = DecisionPipeline::new(validator);

        let decision = pipeline
            .evaluate("docker run --rm alpine echo hello", ShellType::Bash)
            .await
            .unwrap();
        assert!(decision.is_allowed(), "Allowlisted commands should be allowed");
    }

    #[tokio::test]
    async fn test_pipeline_new_patterns_terraform_destroy() {
        let pipeline = make_pipeline(SafetyLevel::Moderate);

        let decision = pipeline
            .evaluate("terraform destroy", ShellType::Bash)
            .await
            .unwrap();
        assert!(
            decision.needs_confirmation() || decision.is_blocked(),
            "terraform destroy should not be auto-allowed: {:?}",
            decision
        );
    }

    #[tokio::test]
    async fn test_pipeline_new_patterns_drop_database() {
        let pipeline = make_pipeline(SafetyLevel::Moderate);

        // Test with unquoted SQL (e.g., piped to psql)
        let decision = pipeline
            .evaluate("echo DROP TABLE users | psql", ShellType::Bash)
            .await
            .unwrap();
        // The "DROP TABLE" part is in executable context (before the pipe)
        // Note: chained commands bypass safe-pattern fast path,
        // so this goes through full validation
        assert!(
            !decision.is_allowed()
                || matches!(decision, SafetyDecision::AllowWithWarning { .. }),
            "DROP TABLE should trigger at least a warning: {:?}",
            decision
        );
    }
}
