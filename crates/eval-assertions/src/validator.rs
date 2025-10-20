// Unified validation interface

use eval_core::{AssertionConfig, AssertionFailure, TestCase};
use eval_sandbox::ExecutionOutput;

use crate::command_string::CommandStringValidator;
use crate::runtime::RuntimeValidator;

/// Validation result aggregating all assertion failures
pub struct ValidationResult {
    pub passed: bool,
    pub failures: Vec<AssertionFailure>,
}

/// Unified assertion validator combining command-string and runtime validation
pub struct AssertionValidator {
    command_string: CommandStringValidator,
    runtime: RuntimeValidator,
}

impl AssertionValidator {
    pub fn new() -> Self {
        Self {
            command_string: CommandStringValidator::new(),
            runtime: RuntimeValidator::new(),
        }
    }

    /// Validate all assertions for a test case
    pub fn validate(
        &mut self,
        generated_command: &str,
        test_case: &TestCase,
        assertions: &AssertionConfig,
        execution_output: Option<&ExecutionOutput>,
    ) -> ValidationResult {
        let mut all_failures = Vec::new();

        // Validate command-string assertions
        if let Some(cmd_assertions) = &assertions.command_string {
            match self.command_string.validate(generated_command, test_case, cmd_assertions) {
                Ok(failures) => all_failures.extend(failures),
                Err(e) => {
                    all_failures.push(AssertionFailure {
                        assertion_type: "validation_error".to_string(),
                        expected: "Valid validation".to_string(),
                        actual: "Validation error".to_string(),
                        message: format!("Command-string validation error: {}", e),
                    });
                }
            }
        }

        // Validate runtime assertions if execution output is available
        if let (Some(runtime_assertions), Some(output)) = (&assertions.runtime, execution_output) {
            match self.runtime.validate(output, runtime_assertions) {
                Ok(failures) => all_failures.extend(failures),
                Err(e) => {
                    all_failures.push(AssertionFailure {
                        assertion_type: "validation_error".to_string(),
                        expected: "Valid validation".to_string(),
                        actual: "Validation error".to_string(),
                        message: format!("Runtime validation error: {}", e),
                    });
                }
            }
        }

        ValidationResult {
            passed: all_failures.is_empty(),
            failures: all_failures,
        }
    }
}

impl Default for AssertionValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eval_core::{CommandStringAssertions, DifficultyLevel, RuntimeAssertions, SafetyLevel, ShellType};
    use std::path::PathBuf;
    use std::time::Duration;

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
    fn test_command_string_only() {
        let mut validator = AssertionValidator::new();
        let test_case = create_test_case();

        let assertions = AssertionConfig {
            command_string: Some(CommandStringAssertions {
                denylist: vec!["rm -rf".to_string()],
                ..Default::default()
            }),
            runtime: None,
        };

        let result = validator.validate("ls -la", &test_case, &assertions, None);
        assert!(result.passed);
        assert_eq!(result.failures.len(), 0);

        let result = validator.validate("rm -rf /", &test_case, &assertions, None);
        assert!(!result.passed);
        assert_eq!(result.failures.len(), 1);
    }

    #[test]
    fn test_runtime_only() {
        let mut validator = AssertionValidator::new();
        let test_case = create_test_case();

        let assertions = AssertionConfig {
            command_string: None,
            runtime: Some(RuntimeAssertions {
                allowed_exit_codes: vec![0],
                ..Default::default()
            }),
        };

        let output = ExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: Duration::from_millis(100),
            working_dir: PathBuf::from("/tmp"),
            created_files: vec![],
            modified_files: vec![],
            timed_out: false,
        };

        let result = validator.validate("ls -la", &test_case, &assertions, Some(&output));
        assert!(result.passed);
    }

    #[test]
    fn test_combined_validation() {
        let mut validator = AssertionValidator::new();
        let test_case = create_test_case();

        let assertions = AssertionConfig {
            command_string: Some(CommandStringAssertions {
                required_flags: vec!["-la".to_string()],
                ..Default::default()
            }),
            runtime: Some(RuntimeAssertions {
                allowed_exit_codes: vec![0],
                stdout_empty: Some(false),
                ..Default::default()
            }),
        };

        let output = ExecutionOutput {
            exit_code: 0,
            stdout: "file1.txt\n".to_string(),
            stderr: String::new(),
            execution_time: Duration::from_millis(100),
            working_dir: PathBuf::from("/tmp"),
            created_files: vec![],
            modified_files: vec![],
            timed_out: false,
        };

        let result = validator.validate("ls -la", &test_case, &assertions, Some(&output));
        assert!(result.passed);
        assert_eq!(result.failures.len(), 0);
    }
}
