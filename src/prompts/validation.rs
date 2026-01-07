//! Command Validation Rules for Generated Shell Commands
//!
//! This module provides validation rules to check if generated shell commands
//! are safe, compatible with the target platform, and properly formatted.
//!
//! # Validation Categories
//!
//! 1. **Schema Validation**: Response format (JSON or QUESTION)
//! 2. **Tool Allowlist**: Commands are in the allowed tool list
//! 3. **Feature Compatibility**: Flags are supported by the platform
//! 4. **Safety Checks**: Dangerous patterns are detected
//! 5. **Quoting/Syntax**: Paths with spaces are properly quoted
//! 6. **Complexity Budget**: Pipeline stages within limit
//!
//! # Example
//!
//! ```rust
//! use caro::prompts::validation::{CommandValidator, ValidationResult};
//! use caro::prompts::capability_profile::CapabilityProfile;
//!
//! let profile = CapabilityProfile::ubuntu();
//! let validator = CommandValidator::new(profile);
//!
//! let result = validator.validate("ls -la");
//! assert!(result.is_valid());
//! ```

use super::capability_profile::{CapabilityProfile, StatFormat};
use regex::Regex;
use std::collections::HashSet;

/// Result of command validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the command is valid
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings (non-fatal)
    pub warnings: Vec<ValidationWarning>,
    /// Risk level assessment
    pub risk_level: RiskLevel,
}

impl ValidationResult {
    /// Create a valid result with no errors
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            risk_level: RiskLevel::Safe,
        }
    }

    /// Create an invalid result with errors
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
            risk_level: RiskLevel::High,
        }
    }

    /// Check if result is valid
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Get error messages as a single string
    pub fn error_message(&self) -> String {
        self.errors
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// A validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error code for programmatic handling
    pub code: ValidationErrorCode,
    /// Human-readable error message
    pub message: String,
    /// The offending part of the command (if applicable)
    pub context: Option<String>,
}

impl ValidationError {
    pub fn new(code: ValidationErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Error codes for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorCode {
    /// Invalid response format (not JSON or QUESTION)
    InvalidFormat,
    /// Tool not in allowlist
    ToolNotAllowed,
    /// Flag not supported on this platform
    FlagNotSupported,
    /// Dangerous command pattern detected
    DangerousCommand,
    /// Path with spaces not quoted
    UnquotedPath,
    /// Pipeline too complex
    ComplexityExceeded,
    /// Command output detected (model hallucinated output)
    OutputHallucination,
    /// Empty command
    EmptyCommand,
    /// Syntax error
    SyntaxError,
}

/// A validation warning (non-fatal)
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ValidationWarning {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Risk level for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Safe, non-destructive command
    Safe,
    /// Potentially modifying (chmod, chown, etc.)
    Moderate,
    /// Destructive or dangerous (rm, dd, etc.)
    High,
    /// Critical system-level danger
    Critical,
}

/// Validator for generated shell commands
pub struct CommandValidator {
    profile: CapabilityProfile,
    tool_allowlist: HashSet<String>,
    max_pipeline_stages: usize,
    allow_destructive: bool,
    dangerous_patterns: Vec<DangerousPattern>,
    flag_rules: Vec<FlagRule>,
}

impl CommandValidator {
    /// Create a new validator with the given capability profile
    pub fn new(profile: CapabilityProfile) -> Self {
        let tool_allowlist = Self::default_tool_allowlist();
        let dangerous_patterns = Self::default_dangerous_patterns();
        let flag_rules = Self::flag_rules_for_profile(&profile);

        Self {
            profile,
            tool_allowlist,
            max_pipeline_stages: 4,
            allow_destructive: false,
            dangerous_patterns,
            flag_rules,
        }
    }

    /// Set the tool allowlist
    pub fn with_tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tool_allowlist = tools.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Set maximum pipeline stages
    pub fn max_pipeline_stages(mut self, max: usize) -> Self {
        self.max_pipeline_stages = max;
        self
    }

    /// Allow destructive commands
    pub fn allow_destructive(mut self, allow: bool) -> Self {
        self.allow_destructive = allow;
        self
    }

    /// Validate a command string
    pub fn validate(&self, command: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Empty command check
        if command.trim().is_empty() {
            result.add_error(ValidationError::new(
                ValidationErrorCode::EmptyCommand,
                "Command is empty",
            ));
            return result;
        }

        // Check for output hallucination (model generating fake output)
        if self.is_output_hallucination(command) {
            result.add_error(ValidationError::new(
                ValidationErrorCode::OutputHallucination,
                "Response contains hallucinated command output instead of a command",
            ));
            return result;
        }

        // Check pipeline complexity
        let pipeline_count = self.count_pipeline_stages(command);
        if pipeline_count > self.max_pipeline_stages {
            result.add_error(
                ValidationError::new(
                    ValidationErrorCode::ComplexityExceeded,
                    format!(
                        "Pipeline has {} stages, maximum is {}",
                        pipeline_count, self.max_pipeline_stages
                    ),
                )
                .with_context(command.to_string()),
            );
        }

        // Check each command in the pipeline
        let commands = self.split_commands(command);
        for cmd in &commands {
            self.validate_single_command(cmd, &mut result);
        }

        // Check dangerous patterns
        if !self.allow_destructive {
            self.check_dangerous_patterns(command, &mut result);
        }

        // Check quoting for paths with spaces
        self.check_quoting(command, &mut result);

        // Assess overall risk level (but don't downgrade if dangerous patterns already set Critical)
        if result.risk_level != RiskLevel::Critical {
            result.risk_level = self.assess_risk_level(command);
        }

        result
    }

    /// Validate the raw model response (before parsing)
    pub fn validate_response(&self, response: &str) -> ValidationResult {
        let trimmed = response.trim();

        // Check for QUESTION format
        if trimmed.starts_with("QUESTION:") {
            return ValidationResult::valid();
        }

        // Check for JSON format
        if trimmed.starts_with('{') || trimmed.contains("\"cmd\":") {
            // Try to extract the command
            if let Some(cmd) = self.extract_command_from_json(trimmed) {
                return self.validate(&cmd);
            } else {
                let mut result = ValidationResult::valid();
                result.add_error(ValidationError::new(
                    ValidationErrorCode::InvalidFormat,
                    "Could not parse command from JSON response",
                ));
                return result;
            }
        }

        // Neither QUESTION nor JSON format
        let mut result = ValidationResult::valid();
        result.add_error(ValidationError::new(
            ValidationErrorCode::InvalidFormat,
            "Response must be JSON ({\"cmd\": \"...\"}) or QUESTION: ...",
        ));
        result
    }

    fn extract_command_from_json(&self, response: &str) -> Option<String> {
        // Try full JSON parse
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                return Some(cmd.to_string());
            }
        }

        // Try to extract from partial JSON
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_part = &response[start..=end];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_part) {
                    if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                        return Some(cmd.to_string());
                    }
                }
            }
        }

        // Try to extract from "cmd": "..." pattern
        let re = Regex::new(r#""cmd"\s*:\s*"([^"]+)"#).ok()?;
        if let Some(caps) = re.captures(response) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }

        None
    }

    fn validate_single_command(&self, command: &str, result: &mut ValidationResult) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let tool = parts[0];

        // Check tool allowlist
        if !self.tool_allowlist.contains(tool) {
            result.add_error(
                ValidationError::new(
                    ValidationErrorCode::ToolNotAllowed,
                    format!("Tool '{}' is not in the allowed list", tool),
                )
                .with_context(tool.to_string()),
            );
        }

        // Check flag compatibility
        for flag_rule in &self.flag_rules {
            if flag_rule.tool == tool && parts.iter().any(|p| *p == flag_rule.flag) {
                if !flag_rule.supported {
                    result.add_error(
                        ValidationError::new(
                            ValidationErrorCode::FlagNotSupported,
                            format!(
                                "Flag '{}' for '{}' is not supported on {} ({})",
                                flag_rule.flag, tool, self.profile.profile_type, flag_rule.reason
                            ),
                        )
                        .with_context(command.to_string()),
                    );
                }
            }
        }
    }

    fn check_dangerous_patterns(&self, command: &str, result: &mut ValidationResult) {
        for pattern in &self.dangerous_patterns {
            if pattern.regex.is_match(command) {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::DangerousCommand,
                        format!("Dangerous pattern detected: {}", pattern.description),
                    )
                    .with_context(command.to_string()),
                );
                result.risk_level = pattern.risk_level;
            }
        }
    }

    fn check_quoting(&self, command: &str, result: &mut ValidationResult) {
        // Simple heuristic: if there's a path-like string with spaces that's not quoted
        // We look for patterns like /path/with spaces that aren't inside quotes
        let mut in_double_quote = false;
        let mut in_single_quote = false;
        let chars: Vec<char> = command.chars().collect();

        for i in 0..chars.len() {
            let c = chars[i];
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };

            if c == '"' && prev != Some('\\') && !in_single_quote {
                in_double_quote = !in_double_quote;
            } else if c == '\'' && prev != Some('\\') && !in_double_quote {
                in_single_quote = !in_single_quote;
            }
        }

        // If we have unbalanced quotes, that might be an issue
        if in_double_quote || in_single_quote {
            result.add_warning(ValidationWarning::new("Unbalanced quotes in command"));
        }

        // Check for common patterns that might need quoting
        // Look for paths followed by spaces that aren't quoted
        // Using raw string literal with # delimiters to avoid escaping issues
        if let Ok(re) = Regex::new(r#"(?:^|\s)(/[^\s"']+)\s+([^\s|;&>]+)"#) {
            for caps in re.captures_iter(command) {
                if let (Some(path), Some(rest)) = (caps.get(1), caps.get(2)) {
                    // Check if this looks like an unquoted path with spaces
                    let path_str = path.as_str();
                    let rest_str = rest.as_str();
                    // If the rest doesn't look like a flag and the path looks like a file path
                    if !rest_str.starts_with('-') && path_str.contains('/') {
                        // This might be an unquoted path with spaces
                        result.add_warning(
                            ValidationWarning::new("Path may contain unquoted spaces")
                                .with_suggestion("Quote paths with spaces: \"path with spaces\""),
                        );
                        break;
                    }
                }
            }
        }
    }

    fn is_output_hallucination(&self, text: &str) -> bool {
        // Patterns that indicate the model is generating fake output instead of a command
        let hallucination_patterns = [
            r"^total \d+",           // ls output
            r"^\d+\s+\d+\s+\d+",     // numeric output like wc
            r"^-[rwx-]{9}",          // ls -l output
            r"^drwx",                // directory listing
            r"^\s*\d+\.\d+%",        // percentage output
            r"^Here is the output:", // LLM preamble
            r"^The command output:", // LLM preamble
            r"^Output:",             // LLM preamble
        ];

        for pattern in hallucination_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(text.trim()) {
                    return true;
                }
            }
        }

        false
    }

    fn count_pipeline_stages(&self, command: &str) -> usize {
        // Count pipes, but not escaped or quoted ones
        let mut count = 1;
        let mut in_quotes = false;
        let mut in_single_quotes = false;
        let chars: Vec<char> = command.chars().collect();

        for i in 0..chars.len() {
            let c = chars[i];
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };

            if c == '"' && prev != Some('\\') {
                in_quotes = !in_quotes;
            } else if c == '\'' && prev != Some('\\') {
                in_single_quotes = !in_single_quotes;
            } else if c == '|' && !in_quotes && !in_single_quotes && prev != Some('\\') {
                // Make sure it's not ||
                let next = chars.get(i + 1);
                if next != Some(&'|') && prev != Some('|') {
                    count += 1;
                }
            }
        }

        count
    }

    fn split_commands(&self, command: &str) -> Vec<String> {
        // Split on pipes, semicolons, and && (but not inside quotes)
        let mut commands = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut in_single_quotes = false;
        let chars: Vec<char> = command.chars().collect();

        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };

            if c == '"' && prev != Some('\\') {
                in_quotes = !in_quotes;
                current.push(c);
            } else if c == '\'' && prev != Some('\\') {
                in_single_quotes = !in_single_quotes;
                current.push(c);
            } else if !in_quotes && !in_single_quotes {
                if c == '|' && chars.get(i + 1) != Some(&'|') && prev != Some('|') {
                    // Pipe separator
                    if !current.trim().is_empty() {
                        commands.push(current.trim().to_string());
                    }
                    current = String::new();
                } else if c == ';' || (c == '&' && chars.get(i + 1) == Some(&'&')) {
                    // Command separator
                    if !current.trim().is_empty() {
                        commands.push(current.trim().to_string());
                    }
                    current = String::new();
                    if c == '&' {
                        i += 1; // Skip second &
                    }
                } else {
                    current.push(c);
                }
            } else {
                current.push(c);
            }

            i += 1;
        }

        if !current.trim().is_empty() {
            commands.push(current.trim().to_string());
        }

        commands
    }

    fn assess_risk_level(&self, command: &str) -> RiskLevel {
        let critical_patterns = [
            r"rm\s+-rf\s+/",
            r"dd\s+.*of=/dev/",
            r"mkfs",
            r":\(\)\{\s*:\|:\&\s*\};:",
            r"fork\s+bomb",
            r">\s*/dev/sd",
        ];

        let high_patterns = [
            r"\brm\b",
            r"\bdd\b",
            r"chmod\s+-R",
            r"chown\s+-R",
            r"sudo\s+rm",
        ];

        let moderate_patterns = [
            r"\bmv\b",
            r"\bchmod\b",
            r"\bchown\b",
            r">\s+\S+", // Redirect that could overwrite
        ];

        for pattern in critical_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(command) {
                    return RiskLevel::Critical;
                }
            }
        }

        for pattern in high_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(command) {
                    return RiskLevel::High;
                }
            }
        }

        for pattern in moderate_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(command) {
                    return RiskLevel::Moderate;
                }
            }
        }

        RiskLevel::Safe
    }

    fn default_tool_allowlist() -> HashSet<String> {
        [
            "ls", "find", "grep", "awk", "sed", "sort", "head", "tail", "xargs", "cat", "wc",
            "cut", "tr", "uniq", "tee", "diff", "stat", "du", "df", "file", "readlink",
            "basename", "dirname", "realpath", "date", "ps", "lsof", "netstat", "ss",
            "tar", "gzip", "gunzip", "bzip2", "curl", "wget", "nc", "jq", "yq",
            "echo", "printf", "test", "true", "false", "pwd", "env", "which",
            "mkdir", "touch", "cp", "mv", "rm", "chmod", "chown", "ln",
            "sh", "bash", "zsh",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn default_dangerous_patterns() -> Vec<DangerousPattern> {
        vec![
            DangerousPattern {
                regex: Regex::new(r"rm\s+-rf\s+/\s*$").unwrap(),
                description: "Recursive delete of root directory".to_string(),
                risk_level: RiskLevel::Critical,
            },
            DangerousPattern {
                regex: Regex::new(r":\(\)\s*\{\s*:\|:\&\s*\}\s*;:").unwrap(),
                description: "Fork bomb".to_string(),
                risk_level: RiskLevel::Critical,
            },
            DangerousPattern {
                regex: Regex::new(r"dd\s+.*of=/dev/[sh]d[a-z]").unwrap(),
                description: "Direct disk write".to_string(),
                risk_level: RiskLevel::Critical,
            },
            DangerousPattern {
                regex: Regex::new(r"mkfs").unwrap(),
                description: "Filesystem creation".to_string(),
                risk_level: RiskLevel::Critical,
            },
            DangerousPattern {
                regex: Regex::new(r"curl.*\|\s*sh").unwrap(),
                description: "Download and execute".to_string(),
                risk_level: RiskLevel::Critical,
            },
            DangerousPattern {
                regex: Regex::new(r"wget.*\|\s*sh").unwrap(),
                description: "Download and execute".to_string(),
                risk_level: RiskLevel::Critical,
            },
            DangerousPattern {
                regex: Regex::new(r">\s*/etc/passwd").unwrap(),
                description: "Overwrite system file".to_string(),
                risk_level: RiskLevel::Critical,
            },
            DangerousPattern {
                regex: Regex::new(r">\s*/etc/shadow").unwrap(),
                description: "Overwrite system file".to_string(),
                risk_level: RiskLevel::Critical,
            },
            DangerousPattern {
                regex: Regex::new(r"chmod\s+777\s+/").unwrap(),
                description: "Open permissions on root".to_string(),
                risk_level: RiskLevel::High,
            },
        ]
    }

    fn flag_rules_for_profile(profile: &CapabilityProfile) -> Vec<FlagRule> {
        let mut rules = Vec::new();

        // find -printf
        if !profile.find_printf {
            rules.push(FlagRule {
                tool: "find".to_string(),
                flag: "-printf".to_string(),
                supported: false,
                reason: "GNU findutils required".to_string(),
            });
        }

        // sort -h
        if !profile.sort_h {
            rules.push(FlagRule {
                tool: "sort".to_string(),
                flag: "-h".to_string(),
                supported: false,
                reason: "GNU coreutils required for human-readable sorting".to_string(),
            });
        }

        // grep -P
        if !profile.grep_p {
            rules.push(FlagRule {
                tool: "grep".to_string(),
                flag: "-P".to_string(),
                supported: false,
                reason: "Perl regex not supported; use -E for extended regex".to_string(),
            });
        }

        // stat format
        match profile.stat_format {
            StatFormat::Gnu => {
                rules.push(FlagRule {
                    tool: "stat".to_string(),
                    flag: "-f".to_string(),
                    supported: false,
                    reason: "Use -c for GNU stat".to_string(),
                });
            }
            StatFormat::Bsd => {
                rules.push(FlagRule {
                    tool: "stat".to_string(),
                    flag: "-c".to_string(),
                    supported: false,
                    reason: "Use -f for BSD stat".to_string(),
                });
            }
            StatFormat::None => {
                rules.push(FlagRule {
                    tool: "stat".to_string(),
                    flag: "-c".to_string(),
                    supported: false,
                    reason: "stat format options not supported".to_string(),
                });
                rules.push(FlagRule {
                    tool: "stat".to_string(),
                    flag: "-f".to_string(),
                    supported: false,
                    reason: "stat format options not supported".to_string(),
                });
            }
        }

        // du --max-depth
        if !profile.du_max_depth {
            rules.push(FlagRule {
                tool: "du".to_string(),
                flag: "--max-depth".to_string(),
                supported: false,
                reason: "Use -d for BSD du".to_string(),
            });
        }

        // date --date
        if !profile.date_gnu_format {
            rules.push(FlagRule {
                tool: "date".to_string(),
                flag: "--date".to_string(),
                supported: false,
                reason: "Use -v for BSD date".to_string(),
            });
        }

        // ps --sort
        if !profile.ps_sort {
            rules.push(FlagRule {
                tool: "ps".to_string(),
                flag: "--sort".to_string(),
                supported: false,
                reason: "ps --sort not available; pipe to sort instead".to_string(),
            });
        }

        // ls --sort
        if !profile.ls_sort {
            rules.push(FlagRule {
                tool: "ls".to_string(),
                flag: "--sort".to_string(),
                supported: false,
                reason: "Use -S or -t flags instead".to_string(),
            });
        }

        rules
    }
}

/// A dangerous command pattern to detect
struct DangerousPattern {
    regex: Regex,
    description: String,
    risk_level: RiskLevel,
}

/// A flag rule for a specific tool
struct FlagRule {
    tool: String,
    flag: String,
    supported: bool,
    reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::capability_profile::ProfileType;

    #[test]
    fn test_valid_command() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile);

        let result = validator.validate("ls -la");
        assert!(result.is_valid());
        assert_eq!(result.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_dangerous_command() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile);

        let result = validator.validate("rm -rf /");
        assert!(!result.is_valid());
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_fork_bomb() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile);

        let result = validator.validate(":() { :|:& };:");
        assert!(!result.is_valid());
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_flag_compatibility_bsd() {
        let profile = CapabilityProfile::for_platform(ProfileType::Bsd);
        let validator = CommandValidator::new(profile);

        // find -printf not available on BSD
        let result = validator.validate("find . -printf '%s %p\\n'");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.code == ValidationErrorCode::FlagNotSupported));
    }

    #[test]
    fn test_pipeline_counting() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile).max_pipeline_stages(2);

        // Two stages - should pass
        let result = validator.validate("find . -type f | wc -l");
        assert!(result.is_valid());

        // Three stages - should fail
        let result = validator.validate("find . -type f | sort | wc -l");
        assert!(!result.is_valid());
    }

    #[test]
    fn test_tool_allowlist() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile)
            .with_tools(["ls", "cat"]);

        let result = validator.validate("ls -la");
        assert!(result.is_valid());

        let result = validator.validate("find . -type f");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.code == ValidationErrorCode::ToolNotAllowed));
    }

    #[test]
    fn test_output_hallucination() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile);

        // This looks like command output, not a command
        let result = validator.validate("total 12\ndrwxr-xr-x 2 user user 4096 Jan 1 12:00 .");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.code == ValidationErrorCode::OutputHallucination));
    }

    #[test]
    fn test_validate_json_response() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile);

        // Valid JSON response
        let result = validator.validate_response(r#"{"cmd": "ls -la"}"#);
        assert!(result.is_valid());

        // Valid QUESTION response
        let result = validator.validate_response("QUESTION: Do you want to delete files?");
        assert!(result.is_valid());

        // Invalid response
        let result = validator.validate_response("Here is the command: ls -la");
        assert!(!result.is_valid());
    }

    #[test]
    fn test_destructive_allowed() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile).allow_destructive(true);

        // rm should be allowed when destructive is enabled
        let result = validator.validate("rm file.txt");
        // Risk level should still be High, but no dangerous pattern error
        assert!(result.errors.iter().all(|e| e.code != ValidationErrorCode::DangerousCommand));
    }

    #[test]
    fn test_risk_assessment() {
        let profile = CapabilityProfile::ubuntu();
        let validator = CommandValidator::new(profile).allow_destructive(true);

        assert_eq!(validator.validate("ls -la").risk_level, RiskLevel::Safe);
        assert_eq!(validator.validate("chmod 644 file.txt").risk_level, RiskLevel::Moderate);
        assert_eq!(validator.validate("rm file.txt").risk_level, RiskLevel::High);
    }
}
