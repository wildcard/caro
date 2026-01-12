//! Correctness evaluator implementation
//!
//! Validates that generated commands match expected functionality using
//! ExactMatch, CommandEquivalence, or PatternMatch rules.

use crate::evaluation::errors::Result;
use crate::evaluation::evaluators::utils;
use crate::evaluation::{
    CommandResult, EvaluationResult, Evaluator, TestCase, TestCategory, ValidationRule,
};
use async_trait::async_trait;
use chrono::Utc;

/// Evaluator for correctness test cases
pub struct CorrectnessEvaluator;

impl Default for CorrectnessEvaluator {
    fn default() -> Self {
        Self
    }
}

impl CorrectnessEvaluator {
    /// Creates a new correctness evaluator
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Evaluator for CorrectnessEvaluator {
    fn category(&self) -> TestCategory {
        TestCategory::Correctness
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

        // Evaluate based on validation rule
        let (passed, failure_reason) = match &test_case.validation_rule {
            ValidationRule::ExactMatch => {
                evaluate_exact_match(&actual_command, &test_case.expected_command)
            }
            ValidationRule::CommandEquivalence => {
                evaluate_command_equivalence(&actual_command, &test_case.expected_command)
            }
            ValidationRule::PatternMatch => {
                evaluate_pattern_match(&actual_command, &test_case.validation_pattern)
            }
            _ => (
                false,
                Some(format!(
                    "Invalid validation rule {:?} for correctness test",
                    test_case.validation_rule
                )),
            ),
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
                Some(crate::evaluation::ErrorType::IncorrectOutput)
            } else {
                None
            },
        })
    }
}

fn evaluate_exact_match(actual: &str, expected: &Option<String>) -> (bool, Option<String>) {
    match expected {
        Some(exp) => {
            if actual == exp {
                (true, None)
            } else {
                (
                    false,
                    Some(format!("Expected exactly '{}', got '{}'", exp, actual)),
                )
            }
        }
        None => (
            false,
            Some("Expected command not specified for ExactMatch".to_string()),
        ),
    }
}

fn evaluate_command_equivalence(actual: &str, expected: &Option<String>) -> (bool, Option<String>) {
    match expected {
        Some(exp) => {
            if utils::command_equivalence(actual, exp) {
                (true, None)
            } else {
                (
                    false,
                    Some(format!(
                        "Commands not equivalent: expected '{}', got '{}'",
                        exp, actual
                    )),
                )
            }
        }
        None => (
            false,
            Some("Expected command not specified for CommandEquivalence".to_string()),
        ),
    }
}

fn evaluate_pattern_match(actual: &str, pattern: &Option<String>) -> (bool, Option<String>) {
    match pattern {
        Some(pat) => {
            if utils::matches_pattern(actual, pat) {
                (true, None)
            } else {
                (
                    false,
                    Some(format!(
                        "Command '{}' does not match pattern '{}'",
                        actual, pat
                    )),
                )
            }
        }
        None => (
            false,
            Some("Validation pattern not specified for PatternMatch".to_string()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::{Difficulty, TestCase, TestCategory, ValidationRule};

    #[tokio::test]
    async fn test_exact_match_success() {
        let evaluator = CorrectnessEvaluator::new();
        let test_case = TestCase {
            id: "test-001".to_string(),
            category: TestCategory::Correctness,
            input_request: "list files".to_string(),
            expected_command: Some("ls -la".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::ExactMatch,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        let result = CommandResult::success("ls -la".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(eval_result.passed);
        assert!(eval_result.failure_reason.is_none());
    }

    #[tokio::test]
    async fn test_exact_match_failure() {
        let evaluator = CorrectnessEvaluator::new();
        let test_case = TestCase {
            id: "test-001".to_string(),
            category: TestCategory::Correctness,
            input_request: "list files".to_string(),
            expected_command: Some("ls -la".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::ExactMatch,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        let result = CommandResult::success("ls -l".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(!eval_result.passed);
        assert!(eval_result.failure_reason.is_some());
    }

    #[tokio::test]
    async fn test_command_equivalence() {
        let evaluator = CorrectnessEvaluator::new();
        let test_case = TestCase {
            id: "test-002".to_string(),
            category: TestCategory::Correctness,
            input_request: "list files".to_string(),
            expected_command: Some("ls -la".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        // Test equivalent command (different flag order)
        let result = CommandResult::success("ls -al".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(eval_result.passed);
    }

    #[tokio::test]
    async fn test_pattern_match_success() {
        let evaluator = CorrectnessEvaluator::new();
        let test_case = TestCase {
            id: "test-003".to_string(),
            category: TestCategory::Correctness,
            input_request: "find Python files".to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: ValidationRule::PatternMatch,
            validation_pattern: Some(r"find.*\.py".to_string()),
            tags: vec![],
            difficulty: Some(Difficulty::Medium),
            source: None,
            notes: None,
        };

        let result =
            CommandResult::success("find . -name '*.py'".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(eval_result.passed);
    }

    #[tokio::test]
    async fn test_generation_failure() {
        let evaluator = CorrectnessEvaluator::new();
        let test_case = TestCase {
            id: "test-004".to_string(),
            category: TestCategory::Correctness,
            input_request: "list files".to_string(),
            expected_command: Some("ls -la".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::ExactMatch,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        let result = CommandResult::failed("Backend timeout".to_string(), 100, "test".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(!eval_result.passed);
        assert!(eval_result.failure_reason.is_some());
    }
}
