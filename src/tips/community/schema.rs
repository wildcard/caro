//! Schema validation for cheatsheet contributions
//!
//! Validates cheatsheet YAML format against the expected schema
//! before allowing submission to the community knowledge base.

use crate::tips::kb::{Cheatsheet, KbTipCategory};
use regex::Regex;
use std::collections::HashSet;

/// Cheatsheet schema validator
#[derive(Debug, Clone)]
pub struct SchemaValidator {
    /// Maximum number of aliases per cheatsheet
    max_aliases: usize,
    /// Maximum number of tips per cheatsheet
    max_tips: usize,
    /// Valid shell types
    valid_shells: HashSet<String>,
    /// Regex patterns that must be valid
    validate_patterns: bool,
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaValidator {
    /// Create a new schema validator with default settings
    pub fn new() -> Self {
        let valid_shells: HashSet<String> = ["zsh", "bash", "fish", "sh", "all"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            max_aliases: 500,
            max_tips: 200,
            valid_shells,
            validate_patterns: true,
        }
    }

    /// Set maximum aliases allowed
    pub fn with_max_aliases(mut self, max: usize) -> Self {
        self.max_aliases = max;
        self
    }

    /// Set maximum tips allowed
    pub fn with_max_tips(mut self, max: usize) -> Self {
        self.max_tips = max;
        self
    }

    /// Disable pattern validation
    pub fn without_pattern_validation(mut self) -> Self {
        self.validate_patterns = false;
        self
    }

    /// Validate a YAML string
    pub fn validate_yaml(&self, yaml: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Parse YAML
        let cheatsheet: Cheatsheet = match serde_yaml::from_str(yaml) {
            Ok(c) => c,
            Err(e) => {
                return ValidationResult {
                    is_valid: false,
                    errors: vec![ValidationError::ParseError(format!("Invalid YAML: {}", e))],
                    warnings: vec![],
                    cheatsheet: None,
                };
            }
        };

        // Validate the parsed cheatsheet
        self.validate_cheatsheet(&cheatsheet, &mut errors, &mut warnings);

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            cheatsheet: Some(cheatsheet),
        }
    }

    /// Validate a parsed cheatsheet
    pub fn validate_cheatsheet(
        &self,
        cheatsheet: &Cheatsheet,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<String>,
    ) {
        // Check name
        if cheatsheet.name.trim().is_empty() {
            errors.push(ValidationError::RequiredField("name".to_string()));
        } else if cheatsheet.name.len() > 100 {
            errors.push(ValidationError::InvalidValue {
                field: "name".to_string(),
                message: "Name must be 100 characters or less".to_string(),
            });
        }

        // Check version format
        if !is_valid_semver(&cheatsheet.version) {
            warnings.push(format!(
                "Version '{}' is not valid semver format",
                cheatsheet.version
            ));
        }

        // Check shells
        for shell in &cheatsheet.shells {
            if !self.valid_shells.contains(&shell.to_lowercase()) {
                errors.push(ValidationError::InvalidValue {
                    field: "shells".to_string(),
                    message: format!("Invalid shell type: {}", shell),
                });
            }
        }

        // Validate aliases
        if cheatsheet.aliases.len() > self.max_aliases {
            errors.push(ValidationError::LimitExceeded {
                field: "aliases".to_string(),
                limit: self.max_aliases,
                actual: cheatsheet.aliases.len(),
            });
        }

        let mut alias_names = HashSet::new();
        for (i, alias) in cheatsheet.aliases.iter().enumerate() {
            // Check for duplicate names
            if !alias_names.insert(&alias.name) {
                errors.push(ValidationError::DuplicateEntry {
                    entry_type: "alias".to_string(),
                    name: alias.name.clone(),
                });
            }

            // Validate alias name
            if alias.name.trim().is_empty() {
                errors.push(ValidationError::RequiredField(format!(
                    "aliases[{}].name",
                    i
                )));
            } else if !is_valid_alias_name(&alias.name) {
                errors.push(ValidationError::InvalidValue {
                    field: format!("aliases[{}].name", i),
                    message: format!("Invalid alias name: '{}'", alias.name),
                });
            }

            // Validate expansion
            if alias.expansion.trim().is_empty() {
                errors.push(ValidationError::RequiredField(format!(
                    "aliases[{}].expansion",
                    i
                )));
            }

            // Check for dangerous patterns
            if contains_dangerous_pattern(&alias.expansion) {
                warnings.push(format!(
                    "Alias '{}' expansion may contain dangerous patterns",
                    alias.name
                ));
            }
        }

        // Validate tips
        if cheatsheet.tips.len() > self.max_tips {
            errors.push(ValidationError::LimitExceeded {
                field: "tips".to_string(),
                limit: self.max_tips,
                actual: cheatsheet.tips.len(),
            });
        }

        let mut tip_ids = HashSet::new();
        for (i, tip) in cheatsheet.tips.iter().enumerate() {
            // Check for duplicate IDs
            if !tip_ids.insert(&tip.id) {
                errors.push(ValidationError::DuplicateEntry {
                    entry_type: "tip".to_string(),
                    name: tip.id.clone(),
                });
            }

            // Validate tip ID
            if tip.id.trim().is_empty() {
                errors.push(ValidationError::RequiredField(format!("tips[{}].id", i)));
            } else if !is_valid_tip_id(&tip.id) {
                errors.push(ValidationError::InvalidValue {
                    field: format!("tips[{}].id", i),
                    message: format!("Invalid tip ID: '{}'. Use kebab-case.", tip.id),
                });
            }

            // Validate pattern
            if tip.pattern.trim().is_empty() {
                errors.push(ValidationError::RequiredField(format!(
                    "tips[{}].pattern",
                    i
                )));
            } else if tip.is_regex && self.validate_patterns {
                if let Err(e) = Regex::new(&tip.pattern) {
                    errors.push(ValidationError::InvalidValue {
                        field: format!("tips[{}].pattern", i),
                        message: format!("Invalid regex: {}", e),
                    });
                }
            }

            // Validate message
            if tip.message.trim().is_empty() {
                errors.push(ValidationError::RequiredField(format!(
                    "tips[{}].message",
                    i
                )));
            } else if tip.message.len() > 500 {
                warnings.push(format!("Tip '{}' message is over 500 characters", tip.id));
            }

            // Validate category
            if !is_valid_category(&tip.category) {
                warnings.push(format!(
                    "Tip '{}' has unusual category: {:?}",
                    tip.id, tip.category
                ));
            }
        }

        // Validate plugins
        for (i, plugin) in cheatsheet.plugins.iter().enumerate() {
            if plugin.name.trim().is_empty() {
                errors.push(ValidationError::RequiredField(format!(
                    "plugins[{}].name",
                    i
                )));
            }

            if plugin.description.trim().is_empty() {
                errors.push(ValidationError::RequiredField(format!(
                    "plugins[{}].description",
                    i
                )));
            }
        }

        // Content quality checks
        if cheatsheet.aliases.is_empty() && cheatsheet.tips.is_empty() {
            warnings.push("Cheatsheet has no aliases or tips".to_string());
        }
    }

    /// Get the JSON schema for cheatsheets
    pub fn json_schema() -> &'static str {
        include_str!("cheatsheet.schema.json")
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the cheatsheet is valid
    pub is_valid: bool,
    /// Validation errors (must fix)
    pub errors: Vec<ValidationError>,
    /// Validation warnings (should consider)
    pub warnings: Vec<String>,
    /// Parsed cheatsheet if successful
    pub cheatsheet: Option<Cheatsheet>,
}

impl ValidationResult {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Format errors and warnings for display
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        if self.is_valid {
            report.push_str("✓ Cheatsheet is valid\n");
        } else {
            report.push_str("✗ Cheatsheet has validation errors\n");
        }

        if !self.errors.is_empty() {
            report.push_str("\nErrors:\n");
            for error in &self.errors {
                report.push_str(&format!("  • {}\n", error));
            }
        }

        if !self.warnings.is_empty() {
            report.push_str("\nWarnings:\n");
            for warning in &self.warnings {
                report.push_str(&format!("  • {}\n", warning));
            }
        }

        if let Some(ref cs) = self.cheatsheet {
            report.push_str("\nSummary:\n");
            report.push_str(&format!("  Name: {}\n", cs.name));
            report.push_str(&format!("  Aliases: {}\n", cs.aliases.len()));
            report.push_str(&format!("  Tips: {}\n", cs.tips.len()));
            report.push_str(&format!("  Plugins: {}\n", cs.plugins.len()));
        }

        report
    }
}

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// YAML parsing error
    ParseError(String),
    /// Required field is missing
    RequiredField(String),
    /// Field value is invalid
    InvalidValue { field: String, message: String },
    /// Limit exceeded
    LimitExceeded {
        field: String,
        limit: usize,
        actual: usize,
    },
    /// Duplicate entry
    DuplicateEntry { entry_type: String, name: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "{}", msg),
            Self::RequiredField(field) => write!(f, "Required field missing: {}", field),
            Self::InvalidValue { field, message } => write!(f, "{}: {}", field, message),
            Self::LimitExceeded {
                field,
                limit,
                actual,
            } => {
                write!(f, "{} exceeds limit: {} (max {})", field, actual, limit)
            }
            Self::DuplicateEntry { entry_type, name } => {
                write!(f, "Duplicate {} entry: {}", entry_type, name)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Check if a string is valid semver
fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

/// Check if an alias name is valid
fn is_valid_alias_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 50 {
        return false;
    }
    // Must start with letter or underscore, can contain alphanumeric, underscore, dash
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").unwrap();
    re.is_match(name)
}

/// Check if a tip ID is valid (kebab-case)
fn is_valid_tip_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 100 {
        return false;
    }
    let re = Regex::new(r"^[a-z][a-z0-9-]*[a-z0-9]$|^[a-z]$").unwrap();
    re.is_match(id)
}

/// Check for dangerous command patterns
fn contains_dangerous_pattern(cmd: &str) -> bool {
    let dangerous = [
        "rm -rf /",
        "rm -rf ~",
        "mkfs",
        "dd if=/dev/zero",
        ":(){",
        "chmod 777 /",
        "sudo rm -rf",
        "> /dev/sda",
    ];

    let lower = cmd.to_lowercase();
    dangerous.iter().any(|p| lower.contains(p))
}

/// Check if category is valid
fn is_valid_category(category: &KbTipCategory) -> bool {
    !matches!(category, KbTipCategory::General) || true // All categories are valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_semver() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("0.1.0"));
        assert!(is_valid_semver("10.20.30"));
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("1.0.0.0"));
        assert!(!is_valid_semver("v1.0.0"));
    }

    #[test]
    fn test_valid_alias_name() {
        assert!(is_valid_alias_name("gst"));
        assert!(is_valid_alias_name("git_status"));
        assert!(is_valid_alias_name("my-alias"));
        assert!(is_valid_alias_name("_private"));
        assert!(!is_valid_alias_name(""));
        assert!(!is_valid_alias_name("123"));
        assert!(!is_valid_alias_name("-start"));
    }

    #[test]
    fn test_valid_tip_id() {
        assert!(is_valid_tip_id("git-status"));
        assert!(is_valid_tip_id("use-alias"));
        assert!(is_valid_tip_id("tip1"));
        assert!(is_valid_tip_id("a"));
        assert!(!is_valid_tip_id(""));
        assert!(!is_valid_tip_id("Git-Status")); // uppercase
        assert!(!is_valid_tip_id("-start"));
        assert!(!is_valid_tip_id("end-"));
    }

    #[test]
    fn test_dangerous_patterns() {
        assert!(contains_dangerous_pattern("rm -rf /"));
        assert!(contains_dangerous_pattern("sudo rm -rf /home"));
        assert!(!contains_dangerous_pattern("rm -rf ./build"));
        assert!(!contains_dangerous_pattern("ls -la"));
    }

    #[test]
    fn test_validate_valid_yaml() {
        let yaml = r#"
name: Test Cheatsheet
version: 1.0.0
shells: [zsh]
aliases:
  - name: gst
    expansion: git status
tips:
  - id: use-gst
    pattern: git status
    message: Use gst instead!
"#;

        let validator = SchemaValidator::new();
        let result = validator.validate_yaml(yaml);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_empty_name() {
        // When name is missing from YAML, serde fills it with default "Unnamed"
        // We should validate that explicitly empty names are rejected
        let yaml = r#"
name: ""
version: 1.0.0
aliases: []
"#;

        let validator = SchemaValidator::new();
        let result = validator.validate_yaml(yaml);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::RequiredField(f) if f == "name")));
    }

    #[test]
    fn test_validate_invalid_regex() {
        let yaml = r#"
name: Test
version: 1.0.0
tips:
  - id: bad-regex
    pattern: "[invalid("
    is_regex: true
    message: Bad pattern
"#;

        let validator = SchemaValidator::new();
        let result = validator.validate_yaml(yaml);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validate_duplicate_aliases() {
        let yaml = r#"
name: Test
version: 1.0.0
aliases:
  - name: gst
    expansion: git status
  - name: gst
    expansion: git stash
"#;

        let validator = SchemaValidator::new();
        let result = validator.validate_yaml(yaml);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateEntry { .. })));
    }

    #[test]
    fn test_validation_report() {
        let yaml = r#"
name: Test
version: 1.0.0
aliases:
  - name: gst
    expansion: git status
"#;

        let validator = SchemaValidator::new();
        let result = validator.validate_yaml(yaml);
        let report = result.format_report();

        assert!(report.contains("valid"));
        assert!(report.contains("Name: Test"));
        assert!(report.contains("Aliases: 1"));
    }
}
