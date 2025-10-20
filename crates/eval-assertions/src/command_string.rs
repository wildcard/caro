// Command-string level validation

use eval_core::{AssertionFailure, CommandStringAssertions, TestCase};
use regex::Regex;
use std::collections::HashMap;

/// Validates command strings against assertion rules
pub struct CommandStringValidator {
    /// Compiled regex cache for performance
    regex_cache: HashMap<String, Regex>,
}

impl CommandStringValidator {
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
        }
    }

    /// Validate a generated command against command-string assertions
    pub fn validate(
        &mut self,
        generated_command: &str,
        _test_case: &TestCase,
        assertions: &CommandStringAssertions,
    ) -> Result<Vec<AssertionFailure>, String> {
        let mut failures = Vec::new();

        // Check denylist patterns
        for pattern in &assertions.denylist {
            if self.matches_pattern(generated_command, pattern) {
                failures.push(AssertionFailure {
                    assertion_type: "denylist".to_string(),
                    expected: format!("Command must NOT contain: {}", pattern),
                    actual: generated_command.to_string(),
                    message: format!("Forbidden pattern '{}' found in command", pattern),
                });
            }
        }

        // Check allowlist patterns (at least one must match)
        if !assertions.allowlist.is_empty() {
            let has_required_pattern = assertions
                .allowlist
                .iter()
                .any(|pattern| self.matches_pattern(generated_command, pattern));

            if !has_required_pattern {
                failures.push(AssertionFailure {
                    assertion_type: "allowlist".to_string(),
                    expected: format!("Command must contain one of: {:?}", assertions.allowlist),
                    actual: generated_command.to_string(),
                    message: "Command does not contain any required patterns".to_string(),
                });
            }
        }

        // Check required flags
        for flag in &assertions.required_flags {
            if !generated_command.contains(flag) {
                failures.push(AssertionFailure {
                    assertion_type: "required_flag".to_string(),
                    expected: format!("Command must contain flag: {}", flag),
                    actual: generated_command.to_string(),
                    message: format!("Required flag '{}' not found", flag),
                });
            }
        }

        // Check maximum length
        if let Some(max_length) = assertions.max_length {
            if generated_command.len() > max_length {
                failures.push(AssertionFailure {
                    assertion_type: "max_length".to_string(),
                    expected: format!("Command length <= {}", max_length),
                    actual: format!("Length: {}", generated_command.len()),
                    message: format!(
                        "Command too long: {} > {}",
                        generated_command.len(),
                        max_length
                    ),
                });
            }
        }

        // Check minimum length
        if let Some(min_length) = assertions.min_length {
            if generated_command.len() < min_length {
                failures.push(AssertionFailure {
                    assertion_type: "min_length".to_string(),
                    expected: format!("Command length >= {}", min_length),
                    actual: format!("Length: {}", generated_command.len()),
                    message: format!(
                        "Command too short: {} < {}",
                        generated_command.len(),
                        min_length
                    ),
                });
            }
        }

        Ok(failures)
    }

    /// Check if command matches a pattern (supports regex or literal)
    fn matches_pattern(&mut self, command: &str, pattern: &str) -> bool {
        // Try literal match first (faster)
        if command.contains(pattern) {
            return true;
        }

        // Try regex match
        if let Some(regex) = self.get_or_compile_regex(pattern) {
            regex.is_match(command)
        } else {
            false
        }
    }

    /// Get or compile regex from cache
    fn get_or_compile_regex(&mut self, pattern: &str) -> Option<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            if let Ok(regex) = Regex::new(pattern) {
                self.regex_cache.insert(pattern.to_string(), regex);
            } else {
                return None;
            }
        }
        self.regex_cache.get(pattern)
    }
}

impl Default for CommandStringValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eval_core::{DifficultyLevel, SafetyLevel, ShellType};

    fn create_test_case() -> TestCase {
        TestCase {
            id: "test_001".to_string(),
            category: "test".to_string(),
            subcategory: "test".to_string(),
            shell: ShellType::Bash,
            difficulty: DifficultyLevel::Basic,
            input: "test".to_string(),
            expected_commands: vec![],
            explanation: "test".to_string(),
            tags: vec![],
            safety_level: SafetyLevel::Safe,
            sandbox: None,
            assertions: None,
        }
    }

    #[test]
    fn test_denylist_validation() {
        let mut validator = CommandStringValidator::new();
        let test_case = create_test_case();

        let assertions = CommandStringAssertions {
            denylist: vec!["rm -rf".to_string(), "sudo".to_string()],
            ..Default::default()
        };

        let failures = validator
            .validate("rm -rf /tmp", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion_type, "denylist");

        let failures = validator
            .validate("ls -la", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 0);
    }

    #[test]
    fn test_allowlist_validation() {
        let mut validator = CommandStringValidator::new();
        let test_case = create_test_case();

        let assertions = CommandStringAssertions {
            allowlist: vec!["find".to_string(), "grep".to_string()],
            ..Default::default()
        };

        let failures = validator
            .validate("find . -name '*.txt'", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 0);

        let failures = validator
            .validate("ls -la", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion_type, "allowlist");
    }

    #[test]
    fn test_required_flags() {
        let mut validator = CommandStringValidator::new();
        let test_case = create_test_case();

        let assertions = CommandStringAssertions {
            required_flags: vec!["-type".to_string(), "-name".to_string()],
            ..Default::default()
        };

        let failures = validator
            .validate("find . -type f -name '*.txt'", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 0);

        let failures = validator
            .validate("find . -name '*.txt'", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn test_length_constraints() {
        let mut validator = CommandStringValidator::new();
        let test_case = create_test_case();

        let assertions = CommandStringAssertions {
            max_length: Some(20),
            min_length: Some(5),
            ..Default::default()
        };

        let failures = validator
            .validate("ls -la", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 0);

        let failures = validator
            .validate("ls", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion_type, "min_length");

        let failures = validator
            .validate("ls -la /very/long/path/that/exceeds/maximum", &test_case, &assertions)
            .unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion_type, "max_length");
    }
}
