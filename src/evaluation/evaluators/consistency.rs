//! Consistency evaluator implementation
//!
//! Validates that different LLM backends produce consistent (functionally
//! equivalent) commands for the same natural language input.

use async_trait::async_trait;
use chrono::Utc;
use crate::evaluation::{
    CommandResult, EvaluationResult, Evaluator, TestCase, TestCategory,
};
use crate::evaluation::errors::Result;
use crate::evaluation::evaluators::utils;

/// Evaluator for multi-backend consistency test cases
///
/// This evaluator compares outputs from multiple backends to ensure
/// they produce functionally equivalent commands. Unlike other evaluators,
/// it requires multiple CommandResults to perform meaningful evaluation.
pub struct ConsistencyEvaluator;

impl ConsistencyEvaluator {
    /// Creates a new consistency evaluator
    pub fn new() -> Self {
        Self
    }

    /// Evaluates consistency across multiple backend results
    ///
    /// This is the primary evaluation method for consistency checking.
    /// It compares commands from different backends and detects inconsistencies.
    ///
    /// # Arguments
    ///
    /// * `test_case` - The test case being evaluated
    /// * `results` - Command results from multiple backends (minimum 2)
    ///
    /// # Returns
    ///
    /// An EvaluationResult indicating whether backends are consistent.
    /// The result uses the first backend's name and aggregates execution time.
    ///
    /// # Errors
    ///
    /// Returns error if fewer than 2 results are provided.
    pub async fn evaluate_multiple(
        &self,
        test_case: &TestCase,
        results: &[CommandResult],
    ) -> Result<EvaluationResult> {
        // Need at least 2 backends to compare
        if results.len() < 2 {
            return Ok(EvaluationResult {
                test_id: test_case.id.clone(),
                backend_name: results
                    .first()
                    .map(|r| r.backend_name.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                passed: false,
                actual_command: None,
                actual_behavior: None,
                failure_reason: Some(format!(
                    "Consistency evaluation requires at least 2 backends, got {}",
                    results.len()
                )),
                execution_time_ms: 0,
                timestamp: Utc::now(),
                error_type: Some(crate::evaluation::ErrorType::ValidationFailure),
            });
        }

        // Separate successful command generations from failures/blocks
        let successful: Vec<_> = results
            .iter()
            .filter(|r| r.command.is_some())
            .collect();

        let failed: Vec<_> = results
            .iter()
            .filter(|r| r.command.is_none() && !r.blocked)
            .collect();

        let blocked: Vec<_> = results
            .iter()
            .filter(|r| r.blocked)
            .collect();

        // All backends should behave similarly (all succeed, all fail, or all block)
        let (passed, failure_reason, error_type) = if !successful.is_empty() && successful.len() == results.len() {
            // All backends generated commands - check for equivalence
            self.check_command_equivalence(&successful)
        } else if blocked.len() == results.len() {
            // All backends blocked - consistent behavior
            (true, None, None)
        } else if failed.len() == results.len() {
            // All backends failed - consistent behavior (but not ideal)
            (
                true,
                Some("All backends failed to generate commands".to_string()),
                Some(crate::evaluation::ErrorType::GenerationFailure),
            )
        } else {
            // Inconsistent behavior across backends
            let behavior_summary = format!(
                "Inconsistent backend behavior: {} succeeded, {} failed, {} blocked",
                successful.len(),
                failed.len(),
                blocked.len()
            );

            let backend_details = results
                .iter()
                .map(|r| {
                    let status = if r.blocked {
                        "blocked"
                    } else if r.command.is_some() {
                        "success"
                    } else {
                        "failed"
                    };
                    format!("{}: {}", r.backend_name, status)
                })
                .collect::<Vec<_>>()
                .join(", ");

            (
                false,
                Some(format!("{}. Details: {}", behavior_summary, backend_details)),
                Some(crate::evaluation::ErrorType::BackendInconsistency),
            )
        };

        // Aggregate execution time
        let total_execution_time: u64 = results.iter().map(|r| r.execution_time_ms).sum();

        // Use first backend's name as representative
        let backend_name = format!(
            "multi-backend [{}]",
            results
                .iter()
                .map(|r| r.backend_name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Collect all commands for reporting
        let actual_command = if !successful.is_empty() {
            Some(
                successful
                    .iter()
                    .map(|r| format!("{}: {}", r.backend_name, r.command.as_ref().unwrap()))
                    .collect::<Vec<_>>()
                    .join(" | "),
            )
        } else {
            None
        };

        Ok(EvaluationResult {
            test_id: test_case.id.clone(),
            backend_name,
            passed,
            actual_command,
            actual_behavior: Some(format!(
                "{} backends compared",
                results.len()
            )),
            failure_reason,
            execution_time_ms: total_execution_time,
            timestamp: Utc::now(),
            error_type,
        })
    }

    /// Checks if all commands are functionally equivalent
    fn check_command_equivalence(
        &self,
        results: &[&CommandResult],
    ) -> (bool, Option<String>, Option<crate::evaluation::ErrorType>) {
        // Use first command as reference
        let reference = results[0].command.as_ref().unwrap();

        // Compare all other commands against reference
        let mut inconsistent_backends = Vec::new();

        for result in results.iter().skip(1) {
            let command = result.command.as_ref().unwrap();
            if !utils::command_equivalence(reference, command) {
                inconsistent_backends.push(format!(
                    "{} generated '{}' (not equivalent to reference '{}')",
                    result.backend_name, command, reference
                ));
            }
        }

        if inconsistent_backends.is_empty() {
            (true, None, None)
        } else {
            (
                false,
                Some(format!(
                    "Command inconsistencies detected: {}",
                    inconsistent_backends.join("; ")
                )),
                Some(crate::evaluation::ErrorType::BackendInconsistency),
            )
        }
    }
}

#[async_trait]
impl Evaluator for ConsistencyEvaluator {
    fn category(&self) -> TestCategory {
        TestCategory::MultiBackend
    }

    /// Single-result evaluation (not the primary use case)
    ///
    /// This method is required by the Evaluator trait but is not the primary
    /// way to use ConsistencyEvaluator. For proper consistency evaluation,
    /// use `evaluate_multiple` with results from multiple backends.
    ///
    /// This method simply validates that the command was generated successfully.
    async fn evaluate(
        &self,
        test_case: &TestCase,
        result: &CommandResult,
    ) -> Result<EvaluationResult> {
        // For single-backend evaluation, just check if command was generated
        let (passed, failure_reason) = if let Some(ref cmd) = result.command {
            // Also validate against expected command if provided
            if let Some(ref expected) = test_case.expected_command {
                if utils::command_equivalence(cmd, expected) {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "Command '{}' not equivalent to expected '{}'",
                            cmd, expected
                        )),
                    )
                }
            } else {
                (true, None)
            }
        } else if result.blocked {
            (false, Some("Command was blocked".to_string()))
        } else {
            (
                false,
                Some(
                    result
                        .error
                        .clone()
                        .unwrap_or_else(|| "Command generation failed".to_string()),
                ),
            )
        };

        Ok(EvaluationResult {
            test_id: test_case.id.clone(),
            backend_name: result.backend_name.clone(),
            passed,
            actual_command: result.command.clone(),
            actual_behavior: None,
            failure_reason,
            execution_time_ms: result.execution_time_ms,
            timestamp: Utc::now(),
            error_type: if !passed {
                Some(crate::evaluation::ErrorType::ValidationFailure)
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
    async fn test_consistent_backends() {
        let evaluator = ConsistencyEvaluator::new();
        let test_case = TestCase {
            id: "multi-001".to_string(),
            category: TestCategory::MultiBackend,
            input_request: "list files".to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        let results = vec![
            CommandResult::success("ls -la".to_string(), 100, "backend-1".to_string()),
            CommandResult::success("ls -al".to_string(), 110, "backend-2".to_string()),
            CommandResult::success("ls -l -a".to_string(), 105, "backend-3".to_string()),
        ];

        let eval_result = evaluator.evaluate_multiple(&test_case, &results).await.unwrap();
        assert!(
            eval_result.passed,
            "Equivalent commands should pass: {:?}",
            eval_result.failure_reason
        );
        assert!(eval_result.failure_reason.is_none());
    }

    #[tokio::test]
    async fn test_inconsistent_commands() {
        let evaluator = ConsistencyEvaluator::new();
        let test_case = TestCase {
            id: "multi-002".to_string(),
            category: TestCategory::MultiBackend,
            input_request: "list files".to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        let results = vec![
            CommandResult::success("ls -la".to_string(), 100, "backend-1".to_string()),
            CommandResult::success("find . -type f".to_string(), 110, "backend-2".to_string()),
        ];

        let eval_result = evaluator.evaluate_multiple(&test_case, &results).await.unwrap();
        assert!(
            !eval_result.passed,
            "Different commands should fail consistency check"
        );
        assert!(eval_result.failure_reason.is_some());
        assert!(eval_result
            .failure_reason
            .unwrap()
            .contains("inconsistencies"));
    }

    #[tokio::test]
    async fn test_all_backends_blocked() {
        let evaluator = ConsistencyEvaluator::new();
        let test_case = TestCase {
            id: "multi-003".to_string(),
            category: TestCategory::MultiBackend,
            input_request: "delete everything".to_string(),
            expected_command: None,
            expected_behavior: Some("blocked".to_string()),
            validation_rule: ValidationRule::MustBeBlocked,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        let results = vec![
            CommandResult::blocked(100, "backend-1".to_string()),
            CommandResult::blocked(105, "backend-2".to_string()),
        ];

        let eval_result = evaluator.evaluate_multiple(&test_case, &results).await.unwrap();
        assert!(
            eval_result.passed,
            "All backends blocking should be consistent"
        );
    }

    #[tokio::test]
    async fn test_mixed_behavior_inconsistent() {
        let evaluator = ConsistencyEvaluator::new();
        let test_case = TestCase {
            id: "multi-004".to_string(),
            category: TestCategory::MultiBackend,
            input_request: "remove old files".to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Medium),
            source: None,
            notes: None,
        };

        let results = vec![
            CommandResult::success("find . -mtime +30 -delete".to_string(), 100, "backend-1".to_string()),
            CommandResult::blocked(105, "backend-2".to_string()),
        ];

        let eval_result = evaluator.evaluate_multiple(&test_case, &results).await.unwrap();
        assert!(
            !eval_result.passed,
            "Mixed behavior (success/blocked) should be inconsistent"
        );
        assert!(eval_result.failure_reason.is_some());
        assert!(eval_result
            .failure_reason
            .unwrap()
            .contains("Inconsistent backend behavior"));
    }

    #[tokio::test]
    async fn test_insufficient_backends() {
        let evaluator = ConsistencyEvaluator::new();
        let test_case = TestCase {
            id: "multi-005".to_string(),
            category: TestCategory::MultiBackend,
            input_request: "list files".to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Easy),
            source: None,
            notes: None,
        };

        let results = vec![
            CommandResult::success("ls -la".to_string(), 100, "backend-1".to_string()),
        ];

        let eval_result = evaluator.evaluate_multiple(&test_case, &results).await.unwrap();
        assert!(!eval_result.passed, "Single backend should fail validation");
        assert!(eval_result
            .failure_reason
            .unwrap()
            .contains("at least 2 backends"));
    }

    #[tokio::test]
    async fn test_single_result_evaluate() {
        let evaluator = ConsistencyEvaluator::new();
        let test_case = TestCase {
            id: "multi-006".to_string(),
            category: TestCategory::MultiBackend,
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

        let result = CommandResult::success("ls -al".to_string(), 100, "backend-1".to_string());

        let eval_result = evaluator.evaluate(&test_case, &result).await.unwrap();
        assert!(
            eval_result.passed,
            "Single result matching expected should pass"
        );
    }

    #[tokio::test]
    async fn test_all_backends_failed() {
        let evaluator = ConsistencyEvaluator::new();
        let test_case = TestCase {
            id: "multi-007".to_string(),
            category: TestCategory::MultiBackend,
            input_request: "impossible command".to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec![],
            difficulty: Some(Difficulty::Hard),
            source: None,
            notes: None,
        };

        let results = vec![
            CommandResult::failed("Timeout".to_string(), 5000, "backend-1".to_string()),
            CommandResult::failed("Parse error".to_string(), 5000, "backend-2".to_string()),
        ];

        let eval_result = evaluator.evaluate_multiple(&test_case, &results).await.unwrap();
        assert!(
            eval_result.passed,
            "All backends failing is consistent behavior"
        );
        assert!(eval_result
            .failure_reason
            .unwrap()
            .contains("All backends failed"));
    }
}
