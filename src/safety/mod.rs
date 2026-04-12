//! Safety module - Command safety validation and risk assessment
//!
//! This module provides comprehensive validation of shell commands to detect
//! potentially dangerous operations before execution.
//!
//! # Architecture
//!
//! - **Pattern Database**: 52 pre-compiled regex patterns covering Critical/High/Moderate risks
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

pub mod cache;
pub mod expansion;
mod patterns;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::models::{RiskLevel, SafetyLevel, ShellType};

pub use cache::{CacheKey, SafetyDecisionCache};
pub use expansion::{ExpansionDetector, ExpansionKind, ShellExpansion};
pub use patterns::{
    get_compiled_patterns_for_shell, get_patterns_by_risk, get_patterns_for_shell,
    validate_patterns,
};

/// Environment variable used to track recursive caro invocations
///
/// Inspired by OpenEndpointSecurity's "self-mute" capability, which prevents
/// clients from triggering events in response to their own actions. For caro,
/// we use this to detect and refuse execution when a generated command would
/// invoke caro itself, preventing infinite recursion.
pub const RECURSION_DEPTH_ENV: &str = "CARO_RECURSION_DEPTH";

/// Maximum allowed recursion depth before caro refuses to run
///
/// A legitimate caro invocation has depth 0. Depth 1 means caro was spawned
/// by another caro process (e.g., via a shell script). Depth 2 would indicate
/// a recursion loop and is refused.
pub const MAX_RECURSION_DEPTH: u32 = 2;

/// Check if a command string would recursively invoke caro
///
/// Detects `caro` (and the legacy name `cmdai`) as a standalone command token,
/// not as a substring. This avoids false positives on names like `cargo` or
/// `scaro` that happen to contain `caro`.
///
/// # Algorithm
///
/// Tokenizes the command on shell separators (`|`, `;`, `&`, `` ` ``, whitespace)
/// and checks if any resulting token is exactly `caro` or `cmdai`.
///
/// # Example
///
/// ```
/// use caro::safety::detect_recursive_invocation;
///
/// assert!(detect_recursive_invocation("caro list files"));
/// assert!(detect_recursive_invocation("ls | caro filter"));
/// assert!(detect_recursive_invocation("true && caro do stuff"));
/// assert!(!detect_recursive_invocation("cargo build"));
/// assert!(!detect_recursive_invocation("echo scaro"));
/// ```
pub fn detect_recursive_invocation(command: &str) -> bool {
    // Split on shell metacharacters that separate commands
    let separators = |c: char| {
        matches!(
            c,
            ' ' | '\t' | '|' | ';' | '&' | '`' | '(' | ')' | '\n' | '\r'
        )
    };
    command
        .split(separators)
        .any(|token| token == "caro" || token == "cmdai")
}

/// Main safety validator for analyzing command safety
#[derive(Debug)]
pub struct SafetyValidator {
    config: SafetyConfig,
    /// Original pattern definitions (used for Debug output, not validation)
    #[allow(dead_code)]
    patterns: Vec<DangerPattern>,
    /// Cached compiled regex patterns for performance
    compiled_patterns: Vec<(regex::Regex, RiskLevel, String)>,
    /// OES-inspired LRU+TTL cache for validation decisions.
    ///
    /// `None` when caching is disabled. Caching is a pure optimization:
    /// misses fall through to the normal pattern-matching path.
    decision_cache: Option<SafetyDecisionCache>,
}

/// Configuration for safety validation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub safety_level: SafetyLevel,
    pub max_command_length: usize,
    pub custom_patterns: Vec<DangerPattern>,
    pub allowlist_patterns: Vec<String>,
    /// Maximum time (milliseconds) allowed for safety validation.
    ///
    /// OES-inspired fail-safe: if validation cannot complete within this
    /// budget (e.g. due to regex catastrophic backtracking on a crafted
    /// input), the command is blocked as a precaution. Unlike OES's
    /// default-ALLOW policy for auth timeouts (appropriate for kernel
    /// operations that must never hang the system), caro uses default-DENY
    /// because a CLI tool should fail closed when it cannot confirm safety.
    ///
    /// Defaults to 500ms, which is generous for the 52+ regex patterns.
    #[serde(default = "default_validation_timeout_ms")]
    pub validation_timeout_ms: u64,

    /// Whether to enable the OES-inspired LRU decision cache.
    ///
    /// When enabled, repeated validations of the same command return
    /// cached results, avoiding re-running all regex patterns. Caching
    /// is a pure optimization: on miss, validation falls through to the
    /// normal path.
    #[serde(default = "default_enable_cache")]
    pub enable_cache: bool,

    /// Maximum entries in the decision cache (LRU eviction)
    #[serde(default = "default_cache_capacity")]
    pub cache_capacity: usize,

    /// Decision cache TTL in seconds
    #[serde(default = "default_cache_ttl_secs")]
    pub cache_ttl_secs: u64,
}

fn default_validation_timeout_ms() -> u64 {
    500
}

fn default_enable_cache() -> bool {
    true
}

fn default_cache_capacity() -> usize {
    256
}

fn default_cache_ttl_secs() -> u64 {
    60
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

        // Initialize decision cache if enabled
        let decision_cache = if config.enable_cache {
            Some(SafetyDecisionCache::new(
                config.cache_capacity,
                std::time::Duration::from_secs(config.cache_ttl_secs),
            ))
        } else {
            None
        };

        Ok(Self {
            config,
            patterns,
            compiled_patterns,
            decision_cache,
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

    /// Build a fail-closed ValidationResult when the validation budget is exceeded.
    ///
    /// OES uses per-client timeouts with a default-ALLOW policy because blocking
    /// a kernel thread forever would be catastrophic. For a CLI tool, the right
    /// failure mode is the opposite: default-DENY so the user never executes a
    /// command we could not finish analyzing.
    fn timeout_result(budget_ms: u64, elapsed: std::time::Duration) -> ValidationResult {
        ValidationResult {
            allowed: false,
            risk_level: RiskLevel::High,
            explanation: format!(
                "Safety validation exceeded budget of {}ms (elapsed {}ms) - command blocked as precaution",
                budget_ms,
                elapsed.as_millis()
            ),
            warnings: vec![format!(
                "Validation timeout after {}ms - blocked by fail-safe default",
                elapsed.as_millis()
            )],
            matched_patterns: vec!["validation_timeout".to_string()],
            confidence_score: 1.0,
        }
    }

    /// Validate a single command for safety
    ///
    /// Emits structured tracing events on completion for observability.
    /// Inspired by OpenEndpointSecurity's DTrace probes (auth-allow,
    /// auth-deny, auth-timeout, cache-hit/miss), we emit:
    /// - `safety.decision`: allowed | blocked | timeout
    /// - `safety.risk_level`: string representation of the risk level
    /// - `safety.patterns_matched`: count of matched patterns
    /// - `safety.duration_us`: elapsed microseconds
    /// - `safety.shell`: shell type
    /// - `safety.command_len`: command length in bytes
    ///
    /// **Privacy**: the raw command text is NOT logged, only its length and
    /// derived metadata. Users can opt in to verbose debug logging via
    /// `RUST_LOG=caro::safety=debug` for local debugging.
    pub async fn validate_command(
        &self,
        command: &str,
        shell: ShellType,
    ) -> Result<ValidationResult, ValidationError> {
        // OES-inspired timeout budget. The Rust `regex` crate guarantees
        // linear-time execution (no catastrophic backtracking), so the
        // realistic risk is a pathologically long list of custom patterns
        // or an unexpected performance regression. We check elapsed time
        // at each pattern iteration and fail CLOSED (default-deny) on
        // timeout, which is the appropriate fail-safe for a CLI tool.
        let start = std::time::Instant::now();
        let budget = std::time::Duration::from_millis(self.config.validation_timeout_ms);
        let timeout_exceeded = || start.elapsed() > budget;

        debug!(
            target: "caro::safety",
            shell = %shell,
            command_len = command.len(),
            budget_ms = self.config.validation_timeout_ms,
            "safety validation begin"
        );

        // OES-inspired cache lookup. Compute the key once; we'll reuse it
        // below for insertion on miss.
        let cache_key = self
            .decision_cache
            .as_ref()
            .map(|_| CacheKey::new(command, shell, self.config.safety_level));
        if let (Some(cache), Some(key)) = (&self.decision_cache, cache_key) {
            if let Some(cached) = cache.get(&key) {
                info!(
                    target: "caro::safety",
                    decision = if cached.allowed { "allowed" } else { "blocked" },
                    risk_level = %cached.risk_level,
                    cache_hit = true,
                    shell = %shell,
                    command_len = command.len(),
                    "safety validation cache hit"
                );
                return Ok(cached);
            }
        }

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

        // OES-inspired: detect shell expansions before pattern matching.
        // The pattern database cannot see inside $(...) or `...`, so we
        // raise the risk level when expansions that execute commands are
        // present. This is the "kernel-boundary validation" principle
        // applied to shell metacharacters.
        let expansions = ExpansionDetector::new(shell).detect(command);
        for exp in &expansions {
            if exp.kind.executes_command() {
                // Command-executing expansion: raise risk
                let exp_risk = match self.config.safety_level {
                    SafetyLevel::Strict => RiskLevel::High,
                    SafetyLevel::Moderate => RiskLevel::Moderate,
                    SafetyLevel::Permissive => RiskLevel::Moderate,
                };
                if exp_risk > highest_risk {
                    highest_risk = exp_risk;
                }
                matched.push(format!("shell {}", exp.kind.name()));
                warnings.push(format!(
                    "{}: shell {} can hide dangerous operations ({})",
                    exp_risk,
                    exp.kind.name(),
                    exp.description
                ));
            } else {
                // Variable/parameter/arithmetic expansion: informational warning only
                warnings.push(format!(
                    "Info: {} detected ({})",
                    exp.kind.name(),
                    exp.description
                ));
            }
        }

        // Check against built-in compiled patterns (fast!)
        for (regex, risk_level, description, _) in built_in_patterns {
            if timeout_exceeded() {
                let elapsed = start.elapsed();
                info!(
                    target: "caro::safety",
                    decision = "timeout",
                    budget_ms = self.config.validation_timeout_ms,
                    duration_us = elapsed.as_micros() as u64,
                    "safety validation timeout - failing closed"
                );
                return Ok(Self::timeout_result(
                    self.config.validation_timeout_ms,
                    elapsed,
                ));
            }
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
            if timeout_exceeded() {
                let elapsed = start.elapsed();
                info!(
                    target: "caro::safety",
                    decision = "timeout",
                    budget_ms = self.config.validation_timeout_ms,
                    duration_us = elapsed.as_micros() as u64,
                    "safety validation timeout - failing closed"
                );
                return Ok(Self::timeout_result(
                    self.config.validation_timeout_ms,
                    elapsed,
                ));
            }
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

        // OES-inspired observability: emit a structured event capturing the
        // safety decision. This replaces the previously-silent validation
        // path and enables debugging, auditing, and metrics collection.
        let decision = if allowed {
            "allowed"
        } else if requires_confirm {
            "requires_confirmation"
        } else {
            "blocked"
        };
        let duration_us = start.elapsed().as_micros();
        info!(
            target: "caro::safety",
            decision = decision,
            risk_level = %highest_risk,
            patterns_matched = matched.len(),
            duration_us = duration_us as u64,
            cache_hit = false,
            shell = %shell,
            command_len = command.len(),
            "safety validation complete"
        );

        let result = ValidationResult {
            allowed,
            risk_level: highest_risk,
            explanation,
            warnings,
            matched_patterns: matched,
            confidence_score,
        };

        // Store in cache on success. We do NOT cache timeout results
        // (transient) or results for commands that failed pattern
        // compilation upstream (those return Err above).
        if let (Some(cache), Some(key)) = (&self.decision_cache, cache_key) {
            cache.insert(key, result.clone());
        }

        Ok(result)
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
            validation_timeout_ms: default_validation_timeout_ms(),
            enable_cache: default_enable_cache(),
            cache_capacity: default_cache_capacity(),
            cache_ttl_secs: default_cache_ttl_secs(),
        }
    }

    /// Create moderate safety configuration (blocks Critical only)
    pub fn moderate() -> Self {
        Self {
            safety_level: SafetyLevel::Moderate,
            max_command_length: 5000,
            custom_patterns: Vec::new(),
            allowlist_patterns: Vec::new(),
            validation_timeout_ms: default_validation_timeout_ms(),
            enable_cache: default_enable_cache(),
            cache_capacity: default_cache_capacity(),
            cache_ttl_secs: default_cache_ttl_secs(),
        }
    }

    /// Create permissive safety configuration (warns but allows all)
    pub fn permissive() -> Self {
        Self {
            safety_level: SafetyLevel::Permissive,
            max_command_length: 10000,
            custom_patterns: Vec::new(),
            allowlist_patterns: Vec::new(),
            validation_timeout_ms: default_validation_timeout_ms(),
            enable_cache: default_enable_cache(),
            cache_capacity: default_cache_capacity(),
            cache_ttl_secs: default_cache_ttl_secs(),
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
            validation_timeout_ms: default_validation_timeout_ms(),
            enable_cache: default_enable_cache(),
            cache_capacity: default_cache_capacity(),
            cache_ttl_secs: default_cache_ttl_secs(),
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
