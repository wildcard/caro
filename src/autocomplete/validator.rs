//! Validator agent - Verify inferred arguments are correct
//!
//! This module validates that suggested completions match the expected types
//! and constraints defined in the completion context.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::context::{ArgumentSpec, CommandContext, CompletionType};
use super::AutocompleteError;

/// Agent that validates argument suggestions
pub struct ArgumentValidator {
    config: ValidatorConfig,
}

/// Configuration for validation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Whether to check file existence for File arguments
    pub check_file_existence: bool,
    /// Whether to check directory existence for Directory arguments
    pub check_directory_existence: bool,
    /// Whether to validate patterns strictly
    pub strict_pattern_matching: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            check_file_existence: true,
            check_directory_existence: true,
            strict_pattern_matching: false,
        }
    }
}

/// Result of argument validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the argument is valid
    pub is_valid: bool,
    /// Validation errors if any
    pub errors: Vec<String>,
    /// Validation warnings (non-fatal issues)
    pub warnings: Vec<String>,
    /// Confidence in the validation result (0.0 to 1.0)
    pub confidence: f32,
}

impl ArgumentValidator {
    /// Create new argument validator
    pub fn new(config: ValidatorConfig) -> Result<Self, AutocompleteError> {
        Ok(Self { config })
    }

    /// Validate an argument value against the command context
    pub async fn validate(
        &self,
        value: &str,
        context: &CommandContext,
    ) -> Result<ValidationResult, AutocompleteError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Get the expected argument spec from context
        let arg_spec = self.get_expected_spec(context)?;

        if let Some(spec) = arg_spec {
            self.validate_against_spec(value, &spec, &mut errors, &mut warnings)
                .await?;
        } else {
            // No spec available, do basic validation
            warnings.push("No argument specification available for validation".to_string());
        }

        let is_valid = errors.is_empty();
        let confidence = if is_valid {
            if warnings.is_empty() {
                1.0
            } else {
                0.8
            }
        } else {
            0.0
        };

        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
            confidence,
        })
    }

    /// Get expected argument spec from context
    fn get_expected_spec(&self, context: &CommandContext) -> Result<Option<ArgumentSpec>, AutocompleteError> {
        match context.completion_type {
            CompletionType::FlagValue => {
                // Find the flag spec for the previous token
                if let Some(sig) = &context.signature {
                    if context.tokens.len() >= 2 {
                        let flag_token = &context.tokens[context.tokens.len() - 1];

                        // Search in subcommands
                        for subcmd in &sig.subcommands {
                            for flag in &subcmd.flags {
                                if Self::flag_matches_token(flag, flag_token) {
                                    return Ok(flag.value_spec.clone());
                                }
                            }
                        }

                        // Search in global flags
                        for flag in &sig.global_flags {
                            if Self::flag_matches_token(flag, flag_token) {
                                return Ok(flag.value_spec.clone());
                            }
                        }
                    }
                }
                Ok(None)
            }
            CompletionType::Argument => {
                // Find the positional argument spec
                if let Some(sig) = &context.signature {
                    if let Some(subcmd_name) = context.tokens.get(1) {
                        if let Some(subcmd) = sig.subcommands.iter().find(|s| &s.name == subcmd_name) {
                            // Calculate which positional argument we're at
                            let arg_index = self.calculate_arg_index(context);
                            return Ok(subcmd.arguments.get(arg_index).cloned());
                        }
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Calculate which positional argument index we're completing
    fn calculate_arg_index(&self, context: &CommandContext) -> usize {
        // Count non-flag tokens after the subcommand
        let mut arg_count = 0;
        let mut skip_next = false;

        for (i, token) in context.tokens.iter().enumerate() {
            if i <= 1 {
                // Skip command and subcommand
                continue;
            }

            if skip_next {
                skip_next = false;
                continue;
            }

            if token.starts_with('-') {
                // This is a flag, check if it takes a value
                // For simplicity, assume flags with '=' don't consume next token
                if !token.contains('=') {
                    skip_next = true;
                }
            } else {
                arg_count += 1;
            }
        }

        arg_count
    }

    /// Check if a flag matches a token
    fn flag_matches_token(flag: &super::context::FlagSpec, token: &str) -> bool {
        if let Some(short) = flag.short {
            if token == format!("-{}", short) {
                return true;
            }
        }
        if let Some(long) = &flag.long {
            if token == format!("--{}", long) || token.starts_with(&format!("--{}=", long)) {
                return true;
            }
        }
        false
    }

    /// Validate value against argument specification
    pub async fn validate_against_spec(
        &self,
        value: &str,
        spec: &ArgumentSpec,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), AutocompleteError> {
        match spec {
            ArgumentSpec::String { pattern, .. } => {
                if let Some(pattern) = pattern {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        if !regex.is_match(value) {
                            errors.push(format!("Value does not match pattern: {}", pattern));
                        }
                    } else {
                        warnings.push(format!("Invalid pattern in spec: {}", pattern));
                    }
                }
            }
            ArgumentSpec::File {
                must_exist,
                extensions,
            } => {
                let path = Path::new(value);

                if *must_exist && self.config.check_file_existence {
                    if !path.exists() {
                        errors.push(format!("File does not exist: {}", value));
                    } else if !path.is_file() {
                        errors.push(format!("Path is not a file: {}", value));
                    }
                }

                if let Some(exts) = extensions {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if !exts.contains(&ext.to_string()) {
                            errors.push(format!(
                                "Invalid file extension. Expected one of: {}",
                                exts.join(", ")
                            ));
                        }
                    } else {
                        warnings.push("File has no extension".to_string());
                    }
                }
            }
            ArgumentSpec::Directory { must_exist } => {
                let path = Path::new(value);

                if *must_exist && self.config.check_directory_existence {
                    if !path.exists() {
                        errors.push(format!("Directory does not exist: {}", value));
                    } else if !path.is_dir() {
                        errors.push(format!("Path is not a directory: {}", value));
                    }
                }
            }
            ArgumentSpec::Enum { values } => {
                if !values.contains(&value.to_string()) {
                    errors.push(format!(
                        "Invalid value. Must be one of: {}",
                        values.join(", ")
                    ));
                }
            }
            ArgumentSpec::Integer { min, max } => {
                match value.parse::<i64>() {
                    Ok(num) => {
                        if let Some(min) = min {
                            if num < *min {
                                errors.push(format!("Value must be at least {}", min));
                            }
                        }
                        if let Some(max) = max {
                            if num > *max {
                                errors.push(format!("Value must be at most {}", max));
                            }
                        }
                    }
                    Err(_) => {
                        errors.push(format!("Invalid integer: {}", value));
                    }
                }
            }
            ArgumentSpec::Boolean => {
                let lower = value.to_lowercase();
                if !["true", "false", "1", "0", "yes", "no"].contains(&lower.as_str()) {
                    errors.push(format!("Invalid boolean value: {}", value));
                }
            }
        }

        Ok(())
    }

    /// Validate multiple values in batch
    pub async fn validate_batch(
        &self,
        values: &[String],
        context: &CommandContext,
    ) -> Result<Vec<ValidationResult>, AutocompleteError> {
        let mut results = Vec::with_capacity(values.len());

        for value in values {
            let result = self.validate(value, context).await?;
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocomplete::context::{CommandSignature, FlagSpec, SubcommandSpec};

    #[tokio::test]
    async fn test_validate_string_pattern() {
        let validator = ArgumentValidator::new(ValidatorConfig::default()).unwrap();

        let spec = ArgumentSpec::String {
            pattern: Some(r"^\d{3}-\d{3}-\d{4}$".to_string()),
            examples: vec![],
        };

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Valid phone number
        validator
            .validate_against_spec("123-456-7890", &spec, &mut errors, &mut warnings)
            .await
            .unwrap();
        assert!(errors.is_empty());

        // Invalid phone number
        let mut errors = Vec::new();
        validator
            .validate_against_spec("not-a-phone", &spec, &mut errors, &mut warnings)
            .await
            .unwrap();
        assert!(!errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_integer_range() {
        let validator = ArgumentValidator::new(ValidatorConfig::default()).unwrap();

        let spec = ArgumentSpec::Integer {
            min: Some(0),
            max: Some(100),
        };

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Valid integer
        validator
            .validate_against_spec("50", &spec, &mut errors, &mut warnings)
            .await
            .unwrap();
        assert!(errors.is_empty());

        // Too large
        let mut errors = Vec::new();
        validator
            .validate_against_spec("150", &spec, &mut errors, &mut warnings)
            .await
            .unwrap();
        assert!(!errors.is_empty());

        // Not an integer
        let mut errors = Vec::new();
        validator
            .validate_against_spec("abc", &spec, &mut errors, &mut warnings)
            .await
            .unwrap();
        assert!(!errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_enum() {
        let validator = ArgumentValidator::new(ValidatorConfig::default()).unwrap();

        let spec = ArgumentSpec::Enum {
            values: vec!["red".to_string(), "green".to_string(), "blue".to_string()],
        };

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Valid value
        validator
            .validate_against_spec("red", &spec, &mut errors, &mut warnings)
            .await
            .unwrap();
        assert!(errors.is_empty());

        // Invalid value
        let mut errors = Vec::new();
        validator
            .validate_against_spec("yellow", &spec, &mut errors, &mut warnings)
            .await
            .unwrap();
        assert!(!errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_boolean() {
        let validator = ArgumentValidator::new(ValidatorConfig::default()).unwrap();

        let spec = ArgumentSpec::Boolean;

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Valid boolean values
        for value in &["true", "false", "yes", "no", "1", "0"] {
            errors.clear();
            warnings.clear();
            validator
                .validate_against_spec(value, &spec, &mut errors, &mut warnings)
                .await
                .unwrap();
            assert!(errors.is_empty(), "Expected {} to be valid boolean", value);
        }

        // Invalid boolean
        let mut errors = Vec::new();
        validator
            .validate_against_spec("maybe", &spec, &mut errors, &mut warnings)
            .await
            .unwrap();
        assert!(!errors.is_empty());
    }
}
