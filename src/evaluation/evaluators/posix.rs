//! POSIX evaluator implementation
//!
//! Validates that generated commands comply with POSIX standards and
//! avoid bash-specific or GNU-only features.

use async_trait::async_trait;
use chrono::Utc;
use crate::evaluation::{
    CommandResult, EvaluationResult, Evaluator, TestCase, TestCategory,
};
use crate::evaluation::errors::Result;
use crate::evaluation::evaluators::utils;

/// Evaluator for POSIX compliance test cases
pub struct POSIXEvaluator;

impl POSIXEvaluator {
    /// Creates a new POSIX evaluator
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Evaluator for POSIXEvaluator {
    fn category(&self) -> TestCategory {
        TestCategory::POSIX
    }

    async fn evaluate(
        &self,
        test_case: &TestCase,
        result: &CommandResult,
    ) -> Result<EvaluationResult> {
        // If command generation failed, test fails
        let actual_command = match &result.command {
            Some(cmd) => cmd.clone(),
            None => {
                return Ok(EvaluationResult {
                    test_id: test_case.id.clone(),
                    backend_name: result.backend_name.clone(),
                    passed: false,
                    actual_command: None,
                    actual_behavior: None,
                    failure_reason: Some(
                        result
                            .error
                            .clone()
                            .unwrap_or_else(|| "Command generation failed".to_string()),
                    ),
                    execution_time_ms: result.execution_time_ms,
                    timestamp: Utc::now(),
                    error_type: Some(crate::evaluation::ErrorType::GenerationFailure),
                });
            }
        };

        // Check for POSIX compliance violations
        let violations = utils::check_posix_compliance(&actual_command);

        let (passed, failure_reason) = if violations.is_empty() {
            // Also check if it matches the expected command (if provided)
            if let Some(ref expected) = test_case.expected_command {
                if utils::command_equivalence(&actual_command, expected) {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "Command doesn't match expected POSIX-compliant command. Expected: '{}', got: '{}'",
                            expected, actual_command
                        )),
                    )
                }
            } else {
                // No expected command, just check for violations
                (true, None)
            }
        } else {
            (
                false,
                Some(format!(
                    "POSIX violations detected: {}",
                    violations.join("; ")
                )),
            )
        };

        Ok(EvaluationResult {
            test_id: test_case.id.clone(),
            backend_name: result.backend_name.clone(),
            passed,
            actual_command: Some(actual_command),
            actual_behavior: None,
            failure_reason,
            execution_time_ms: result.execution_time_ms,
            timestamp: Utc::now(),
            error_type: if !passed {
                Some(crate::evaluation::ErrorType::POSIXViolation)
            } else {
                None
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::{Difficulty, TestCase, TestCategory, ValidationRule};

    #[tokio::test]
    async fn test_posix_compliant_command() {
        let evaluator = POSIXEvaluator::new();
        let test_case = TestCase {
            id: "posix-001".to_string(),
            category: TestCategory::POSIX,
            input_request: "find files modified today".to_string(),
            expected_command: Some("find . -mtime 0".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::ExactMatch,
            validation_pattern: None,
            tags: vec!["posix".to_string()],
            difficulty: Some(Difficulty::Medium),
            source: None,
            notes: None,
        };

        let result = CommandResult::success("find . -mtime 0".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(eval_result.passed, "POSIX-compliant command should pass");
        assert!(eval_result.failure_reason.is_none());
    }

    #[tokio::test]
    async fn test_gnu_extension_detected() {
        let evaluator = POSIXEvaluator::new();
        let test_case = TestCase {
            id: "posix-002".to_string(),
            category: TestCategory::POSIX,
            input_request: "find files modified yesterday".to_string(),
            expected_command: Some("find . -mtime 1".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec!["posix".to_string()],
            difficulty: Some(Difficulty::Medium),
            source: None,
            notes: None,
        };

        // Command uses GNU extension (-mtime -1)
        let result = CommandResult::success("find . -mtime -1".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(
            !eval_result.passed,
            "GNU extension should be detected as violation"
        );
        assert!(eval_result.failure_reason.is_some());
        assert!(eval_result
            .failure_reason
            .unwrap()
            .contains("GNU extension"));
    }

    #[tokio::test]
    async fn test_bash_specific_feature() {
        let evaluator = POSIXEvaluator::new();
        let test_case = TestCase {
            id: "posix-003".to_string(),
            category: TestCategory::POSIX,
            input_request: "test if file exists".to_string(),
            expected_command: Some("[ -f file.txt ]".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec!["posix".to_string()],
            difficulty: Some(Difficulty::Medium),
            source: None,
            notes: None,
        };

        // Command uses bash-specific [[ operator
        let result = CommandResult::success("[[ -f file.txt ]]".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(
            !eval_result.passed,
            "Bash-specific feature should be detected"
        );
        assert!(eval_result.failure_reason.is_some());
        assert!(eval_result
            .failure_reason
            .unwrap()
            .contains("Bash-specific"));
    }

    #[tokio::test]
    async fn test_long_options_detected() {
        let evaluator = POSIXEvaluator::new();
        let test_case = TestCase {
            id: "posix-004".to_string(),
            category: TestCategory::POSIX,
            input_request: "list all files".to_string(),
            expected_command: Some("ls -a".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec!["posix".to_string()],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        // Command uses GNU long options
        let result = CommandResult::success("ls --all".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(!eval_result.passed, "GNU long options should be detected");
    }

    #[tokio::test]
    async fn test_generation_failure() {
        let evaluator = POSIXEvaluator::new();
        let test_case = TestCase {
            id: "posix-005".to_string(),
            category: TestCategory::POSIX,
            input_request: "find files".to_string(),
            expected_command: Some("find . -type f".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::ExactMatch,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        let result = CommandResult::failed("Timeout".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(!eval_result.passed);
        assert!(eval_result.failure_reason.is_some());
    }
}
