// Evaluation results and metrics

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Result of executing a single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    /// Test case ID
    pub test_case_id: String,

    /// Generated command
    pub generated_command: String,

    /// Command-string level accuracy
    pub command_accuracy: CommandAccuracy,

    /// Runtime execution results (if sandbox was used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_result: Option<RuntimeResult>,

    /// Performance metrics
    pub performance: PerformanceMetrics,

    /// Error message if generation/execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Overall pass/fail status
    pub passed: bool,
}

/// Command-string level accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAccuracy {
    /// Exact match with any expected command
    pub exact_match: bool,

    /// Semantic equivalence (same functionality, different syntax)
    pub semantic_match: bool,

    /// Functional similarity score (0.0 to 1.0)
    pub functional_score: f64,

    /// Safety score (0.0 to 1.0)
    pub safety_score: f64,

    /// POSIX compliance
    pub posix_compliance: bool,

    /// Overall command-string score (0.0 to 1.0)
    pub overall_score: f64,
}

impl CommandAccuracy {
    pub fn calculate(
        exact_match: bool,
        semantic_match: bool,
        functional_score: f64,
        safety_score: f64,
        posix_compliance: bool,
    ) -> Self {
        let overall_score = if exact_match {
            1.0
        } else if semantic_match {
            0.9
        } else {
            (functional_score * 0.7 + safety_score * 0.3).min(0.85)
        };

        Self {
            exact_match,
            semantic_match,
            functional_score,
            safety_score,
            posix_compliance,
            overall_score,
        }
    }

    pub fn passed(&self) -> bool {
        self.exact_match || self.semantic_match || self.overall_score >= 0.7
    }
}

/// Runtime execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResult {
    /// Exit code
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Assertion results
    pub assertions_passed: bool,

    /// Failed assertions with details
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failed_assertions: Vec<AssertionFailure>,

    /// Files created during execution
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub created_files: Vec<String>,

    /// Files modified during execution
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modified_files: Vec<String>,
}

/// Assertion failure details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionFailure {
    /// Type of assertion that failed
    pub assertion_type: String,

    /// Expected value
    pub expected: String,

    /// Actual value
    pub actual: String,

    /// Human-readable message
    pub message: String,
}

/// Performance metrics for test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total inference time (command generation)
    pub inference_time: Duration,

    /// Memory usage in megabytes
    pub memory_usage_mb: f64,

    /// Tokens per second (if available)
    pub tokens_per_second: f64,

    /// Sandbox setup time (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_setup_time: Option<Duration>,

    /// Command execution time (if sandbox was used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_execution_time: Option<Duration>,
}

/// Aggregated evaluation results for a dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Total number of test cases
    pub total_cases: usize,

    /// Number of exact matches
    pub exact_matches: usize,

    /// Number of semantic matches
    pub semantic_matches: usize,

    /// Number of failures
    pub failures: usize,

    /// Overall accuracy percentage (0.0 to 100.0)
    pub overall_accuracy: f64,

    /// Average inference time in milliseconds
    pub avg_inference_time_ms: u64,

    /// Average functional score
    pub avg_functional_score: f64,

    /// Average safety score
    pub avg_safety_score: f64,

    /// Runtime test statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_stats: Option<RuntimeStats>,

    /// Individual test results
    pub test_results: Vec<TestCaseResult>,

    /// Per-category breakdown
    pub category_breakdown: Vec<CategoryResult>,
}

/// Runtime testing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    /// Number of runtime tests executed
    pub total_runtime_tests: usize,

    /// Number of runtime tests passed
    pub runtime_tests_passed: usize,

    /// Number of runtime tests failed
    pub runtime_tests_failed: usize,

    /// Runtime accuracy percentage
    pub runtime_accuracy: f64,

    /// Average execution time
    pub avg_execution_time_ms: u64,

    /// Number of assertion failures
    pub total_assertion_failures: usize,
}

/// Per-category results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    /// Category name
    pub category: String,

    /// Number of test cases
    pub total: usize,

    /// Number passed
    pub passed: usize,

    /// Accuracy percentage
    pub accuracy: f64,

    /// Average inference time
    pub avg_inference_time_ms: u64,
}

impl EvaluationResult {
    /// Create a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "Total: {}, Exact: {}, Semantic: {}, Failed: {}, Accuracy: {:.1}%",
            self.total_cases,
            self.exact_matches,
            self.semantic_matches,
            self.failures,
            self.overall_accuracy
        )
    }

    /// Check if the evaluation meets a minimum accuracy threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.overall_accuracy >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_accuracy_calculation() {
        let acc = CommandAccuracy::calculate(true, true, 1.0, 1.0, true);
        assert_eq!(acc.overall_score, 1.0);
        assert!(acc.passed());

        let acc2 = CommandAccuracy::calculate(false, true, 0.8, 0.9, true);
        assert_eq!(acc2.overall_score, 0.9);
        assert!(acc2.passed());

        let acc3 = CommandAccuracy::calculate(false, false, 0.5, 0.6, false);
        assert!(acc3.overall_score < 0.7);
    }

    #[test]
    fn test_result_serialization() {
        let result = TestCaseResult {
            test_case_id: "test_001".to_string(),
            generated_command: "ls -la".to_string(),
            command_accuracy: CommandAccuracy::calculate(true, true, 1.0, 1.0, true),
            runtime_result: None,
            performance: PerformanceMetrics {
                inference_time: Duration::from_millis(100),
                memory_usage_mb: 50.0,
                tokens_per_second: 0.0,
                sandbox_setup_time: None,
                command_execution_time: None,
            },
            error: None,
            passed: true,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TestCaseResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.test_case_id, deserialized.test_case_id);
    }
}
