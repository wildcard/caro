//! Core data models for the LLM Evaluation Harness
//!
//! This module defines all entities used throughout the evaluation system:
//! - Test cases and categories
//! - Evaluation results and benchmarks
//! - Backend profiles and configurations
//! - Baseline comparison and regression detection

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test category enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestCategory {
    /// Validates functional accuracy of generated commands
    Correctness,
    /// Validates that dangerous commands are properly blocked
    Safety,
    /// Validates POSIX compliance and shell portability
    #[serde(rename = "posix")]
    POSIX,
    /// Validates consistency across different inference backends
    MultiBackend,
    /// Validates observed runtime behavior by executing the generated command
    /// in a disposable sandbox (see `tools/exec-harness/PROTOCOL.md`)
    Execution,
}

/// Validation rule for test case evaluation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRule {
    /// Generated command must exactly match expected_command
    ExactMatch,
    /// Generated command is functionally equivalent
    CommandEquivalence,
    /// Generated command matches validation_pattern regex
    PatternMatch,
    /// Command must be blocked by safety validation
    MustBeBlocked,
    /// Command must not be blocked
    MustExecute,
}

/// Test difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    /// Straightforward commands with clear expected output
    Easy,
    /// Requires understanding of options/flags
    Medium,
    /// Complex multi-step commands or edge cases
    Hard,
}

/// Backend evaluation priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationPriority {
    /// Run all tests (100% sampling rate)
    Deep,
    /// Run representative sample (~25% sampling rate)
    Basic,
    /// Run only critical tests (~10% sampling rate)
    Minimal,
}

/// Error type categorization for test failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Backend failed to generate command
    GenerationFailure,
    /// Backend exceeded timeout threshold
    Timeout,
    /// Generated command failed validation rules
    ValidationFailure,
    /// Dangerous command not blocked
    SafetyViolation,
    /// Command doesn't match expected output
    IncorrectOutput,
    /// Non-POSIX compliant shell syntax
    #[serde(rename = "posix_violation")]
    POSIXViolation,
    /// Different backends produced inconsistent results
    BackendInconsistency,
}

/// How faithfully the tier-0 engine (just-bash) can execute a case's command.
///
/// just-bash is neither GNU nor BSD userland. This label keeps dialect gaps
/// honest — a `partial`/`unsupported` case measures engine coverage, never
/// command correctness. See `tools/exec-harness/PROTOCOL.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier0Support {
    /// Command runs faithfully on tier 0
    Supported,
    /// Command runs, but some flags/behavior differ from real userland
    Partial,
    /// Command (or a flag it needs) is not implemented by tier 0
    Unsupported,
}

/// Observable effects an execution-grounded case expects from running the
/// generated command in a sandbox. Every populated field is one scored
/// criterion; an omitted `exit_code` defaults to expecting 0.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExpectedEffects {
    /// Expected exit code (None = expect 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,

    /// Regex the captured stdout must match
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_pattern: Option<String>,

    /// Sandbox paths the command must create
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_created: Vec<String>,

    /// Sandbox paths the command must remove
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_removed: Vec<String>,

    /// Sandbox paths whose content the command must change
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_modified: Vec<String>,
}

/// Execution-grounding data for `TestCategory::Execution` cases: the sandbox
/// fixture to seed, the effects to assert, and the tier-0 fidelity label.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExecutionSpec {
    /// Files seeded into the sandbox workspace before execution
    /// (relative path → content)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub fixture_files: HashMap<String, String>,

    /// Effects the command must produce
    #[serde(default)]
    pub expected: ExpectedEffects,

    /// Tier-0 engine compatibility (None is treated as `supported`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier0: Option<Tier0Support>,
}

/// Single labeled evaluation test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Unique identifier (e.g., "safety-001")
    pub id: String,

    /// Test category
    pub category: TestCategory,

    /// Natural language command description
    pub input_request: String,

    /// Expected shell command output (required for correctness/posix tests)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_command: Option<String>,

    /// Expected behavior (e.g., "blocked", "executed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_behavior: Option<String>,

    /// How to validate result
    pub validation_rule: ValidationRule,

    /// Regex pattern for pattern validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_pattern: Option<String>,

    /// Metadata tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Test difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,

    /// Origin (e.g., "beta-testing", "manual")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Additional context or documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Execution-grounding data (Execution category only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionSpec>,
}

/// Outcome of running one test case on one backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Reference to TestCase.id
    pub test_id: String,

    /// Backend that generated the command
    pub backend_name: String,

    /// Whether test passed validation
    pub passed: bool,

    /// Command generated by backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_command: Option<String>,

    /// Observed behavior (e.g., "blocked", "executed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_behavior: Option<String>,

    /// Why test failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,

    /// Time taken to generate command (milliseconds)
    pub execution_time_ms: u64,

    /// When evaluation occurred
    pub timestamp: DateTime<Utc>,

    /// Category of failure if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<ErrorType>,

    /// Estimated input (prompt) tokens for this generation (~chars/4).
    ///
    /// 0 for results that predate cost instrumentation (serde default keeps
    /// older baseline JSON loadable). Populated centrally by the harness.
    #[serde(default)]
    pub est_tokens_in: u32,

    /// Estimated output (command) tokens for this generation (~chars/4).
    #[serde(default)]
    pub est_tokens_out: u32,

    /// Estimated USD cost for this single generation. 0.0 for local/self-hosted
    /// backends; non-zero for hosted frontier APIs. See [`crate::evaluation::pricing`].
    #[serde(default)]
    pub est_cost_usd: f64,

    /// Number of rubric criteria this result satisfied.
    ///
    /// For single-criterion cases this stays 0 and the mean-score is derived
    /// from `passed` (see [`EvaluationResult::score`]). For multi-criterion
    /// cases the evaluator sets `criteria_passed`/`criteria_total` so a result
    /// can be "8 of 10" — the article's distinction between *all-pass*
    /// (production) and *mean-score* (sensitivity).
    #[serde(default)]
    pub criteria_passed: u32,

    /// Total rubric criteria evaluated for this case (0 = single-criterion).
    #[serde(default)]
    pub criteria_total: u32,
}

impl EvaluationResult {
    /// Mean-score for this result: the fraction of rubric criteria passed.
    ///
    /// Single-criterion results (`criteria_total == 0`) score 1.0 when
    /// `passed` and 0.0 otherwise, so a dataset with no multi-criterion cases
    /// has `mean_score == pass_rate` by construction.
    pub fn score(&self) -> f32 {
        if self.criteria_total > 0 {
            self.criteria_passed as f32 / self.criteria_total as f32
        } else if self.passed {
            1.0
        } else {
            0.0
        }
    }
}

/// Configuration for a specific inference backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendProfile {
    /// Backend identifier (e.g., "mlx", "static_matcher")
    pub name: String,

    /// Human-readable name (e.g., "MLX (Apple Silicon)")
    pub display_name: String,

    /// Whether backend is enabled for evaluation
    pub enabled: bool,

    /// Max time per command generation (milliseconds)
    pub timeout_ms: u64,

    /// Platform requirements (e.g., "macos", "cuda")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_features: Vec<String>,

    /// Deep testing vs basic coverage
    pub evaluation_priority: EvaluationPriority,

    /// Fraction of tests to run (0.0-1.0)
    #[serde(default = "default_sampling_rate")]
    pub test_sampling_rate: f32,
}

fn default_sampling_rate() -> f32 {
    1.0
}

/// Aggregated results for a single test category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    /// Which category these results are for
    pub category: TestCategory,

    /// All-pass rate: fraction of tests where *every* criterion passed (0.0-1.0).
    /// This is the production-readiness metric.
    pub pass_rate: f32,

    /// Mean-score: average fraction of criteria passed across tests (0.0-1.0).
    ///
    /// The sensitivity metric. Equals `pass_rate` for single-criterion datasets;
    /// diverges (sits above `pass_rate`) once multi-criterion cases exist, since
    /// a "8 of 10" result lifts the mean but not the all-pass rate.
    #[serde(default)]
    pub mean_score: f32,

    /// Tests in this category
    pub total_tests: usize,

    /// Tests passed
    pub passed: usize,

    /// Tests failed
    pub failed: usize,

    /// Average time per test (milliseconds)
    pub avg_execution_time_ms: u64,
}

/// Aggregated results for a single backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendResult {
    /// Backend identifier
    pub backend_name: String,

    /// All-pass rate for this backend (0.0-1.0)
    pub pass_rate: f32,

    /// Mean-score for this backend: average fraction of criteria passed (0.0-1.0).
    #[serde(default)]
    pub mean_score: f32,

    /// Tests run on this backend
    pub total_tests: usize,

    /// Tests passed
    pub passed: usize,

    /// Tests failed
    pub failed: usize,

    /// Tests that timed out
    pub timeouts: usize,

    /// Average time per test (milliseconds)
    pub avg_execution_time_ms: u64,

    /// Pass rate per category for this backend
    pub category_breakdown: HashMap<TestCategory, f32>,

    /// Total estimated USD cost across all tests for this backend.
    #[serde(default)]
    pub total_cost_usd: f64,

    /// Estimated USD cost per *passed* test (`total_cost_usd / passed`).
    ///
    /// This is the article's headline comparison axis: a backend is only
    /// "cheaper" if it costs less per task it actually got right. 0.0 when no
    /// tests passed.
    #[serde(default)]
    pub cost_per_passed_task: f64,

    /// Total estimated input tokens across all tests for this backend.
    #[serde(default)]
    pub total_tokens_in: u64,

    /// Total estimated output tokens across all tests for this backend.
    #[serde(default)]
    pub total_tokens_out: u64,
}

/// Aggregated results from a complete evaluation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Unique identifier for this run (ISO 8601 timestamp)
    pub run_id: String,

    /// When evaluation started
    pub timestamp: DateTime<Utc>,

    /// Git branch evaluated
    pub branch: String,

    /// Git commit hash
    pub commit_sha: String,

    /// Overall all-pass rate across all tests (0.0-1.0)
    pub overall_pass_rate: f32,

    /// Overall mean-score across all tests (0.0-1.0). Equals `overall_pass_rate`
    /// for single-criterion datasets; the article's sensitivity metric.
    #[serde(default)]
    pub overall_mean_score: f32,

    /// Total number of tests executed
    pub total_tests: usize,

    /// Number of tests passed
    pub total_passed: usize,

    /// Number of tests failed
    pub total_failed: usize,

    /// Per-category aggregated results
    pub category_results: HashMap<TestCategory, CategoryResult>,

    /// Per-backend aggregated results
    pub backend_results: HashMap<String, BackendResult>,

    /// Total evaluation runtime (milliseconds)
    pub execution_time_ms: u64,

    /// Total estimated USD cost across all backends and tests in this run.
    #[serde(default)]
    pub total_cost_usd: f64,

    /// Whether pass rate dropped vs baseline
    pub regression_detected: bool,

    /// Delta from previous run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_comparison: Option<BaselineDelta>,
}

/// Comparison between current run and stored baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineDelta {
    /// Reference to baseline run
    pub baseline_run_id: String,

    /// Baseline git commit
    pub baseline_commit_sha: String,

    /// Change in overall pass rate
    pub overall_delta: f32,

    /// Change per category
    pub category_deltas: HashMap<TestCategory, f32>,

    /// Change per backend
    pub backend_deltas: HashMap<String, f32>,

    /// Threshold for regression detection (e.g., 0.05)
    pub regression_threshold: f32,

    /// Categories/backends with significant drops
    pub significant_regressions: Vec<String>,
}

impl TestCase {
    /// Validates the test case structure
    pub fn validate(&self) -> Result<(), String> {
        // ID format validation
        if !self.id.contains('-') {
            return Err(format!(
                "Test ID '{}' must follow format {{category}}-{{number}}",
                self.id
            ));
        }

        // Input request validation
        if self.input_request.is_empty() {
            return Err(format!("Test {} has empty input_request", self.id));
        }
        if self.input_request.len() > 500 {
            return Err(format!("Test {} input_request exceeds 500 chars", self.id));
        }

        // Validation rule requirements
        match self.validation_rule {
            ValidationRule::PatternMatch if self.validation_pattern.is_none() => {
                return Err(format!(
                    "Test {} requires validation_pattern for PatternMatch rule",
                    self.id
                ));
            }
            ValidationRule::MustBeBlocked
                if self.expected_behavior.as_deref() != Some("blocked") =>
            {
                return Err(format!(
                    "Test {} should have expected_behavior='blocked' for MustBeBlocked rule",
                    self.id
                ));
            }
            ValidationRule::ExactMatch | ValidationRule::CommandEquivalence
                if matches!(
                    self.category,
                    TestCategory::Correctness | TestCategory::POSIX
                ) && self.expected_command.is_none() =>
            {
                return Err(format!(
                    "Test {} requires expected_command for {:?} category",
                    self.id, self.category
                ));
            }
            _ => {}
        }

        // Tag validation
        if self.tags.len() > 10 {
            return Err(format!("Test {} has more than 10 tags", self.id));
        }
        for tag in &self.tags {
            if tag.len() > 50 {
                return Err(format!(
                    "Test {} has tag exceeding 50 chars: {}",
                    self.id, tag
                ));
            }
        }

        Ok(())
    }
}

impl BackendProfile {
    /// Validates the backend profile configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Backend name cannot be empty".to_string());
        }

        if self.timeout_ms == 0 || self.timeout_ms > 30000 {
            return Err(format!(
                "Backend {} timeout must be between 1-30000ms",
                self.name
            ));
        }

        if !(0.0..=1.0).contains(&self.test_sampling_rate) {
            return Err(format!(
                "Backend {} sampling rate must be 0.0-1.0",
                self.name
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_category_serde() {
        let categories = vec![
            (TestCategory::Correctness, "\"correctness\""),
            (TestCategory::Safety, "\"safety\""),
            (TestCategory::POSIX, "\"posix\""),
            (TestCategory::MultiBackend, "\"multi_backend\""),
            (TestCategory::Execution, "\"execution\""),
        ];

        for (category, expected_json) in categories {
            let json = serde_json::to_string(&category).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: TestCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, category);
        }
    }

    #[test]
    fn test_validation_rule_serde() {
        let rules = vec![
            (ValidationRule::ExactMatch, "\"exact_match\""),
            (
                ValidationRule::CommandEquivalence,
                "\"command_equivalence\"",
            ),
            (ValidationRule::PatternMatch, "\"pattern_match\""),
            (ValidationRule::MustBeBlocked, "\"must_be_blocked\""),
            (ValidationRule::MustExecute, "\"must_execute\""),
        ];

        for (rule, expected_json) in rules {
            let json = serde_json::to_string(&rule).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: ValidationRule = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, rule);
        }
    }

    #[test]
    fn test_difficulty_serde() {
        let difficulties = vec![
            (Difficulty::Easy, "\"easy\""),
            (Difficulty::Medium, "\"medium\""),
            (Difficulty::Hard, "\"hard\""),
        ];

        for (diff, expected_json) in difficulties {
            let json = serde_json::to_string(&diff).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: Difficulty = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, diff);
        }
    }

    #[test]
    fn test_evaluation_priority_serde() {
        let priorities = vec![
            (EvaluationPriority::Deep, "\"deep\""),
            (EvaluationPriority::Basic, "\"basic\""),
            (EvaluationPriority::Minimal, "\"minimal\""),
        ];

        for (priority, expected_json) in priorities {
            let json = serde_json::to_string(&priority).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: EvaluationPriority = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, priority);
        }
    }

    #[test]
    fn test_error_type_serde() {
        let error_types = vec![
            (ErrorType::GenerationFailure, "\"generation_failure\""),
            (ErrorType::Timeout, "\"timeout\""),
            (ErrorType::ValidationFailure, "\"validation_failure\""),
            (ErrorType::SafetyViolation, "\"safety_violation\""),
            (ErrorType::IncorrectOutput, "\"incorrect_output\""),
            (ErrorType::POSIXViolation, "\"posix_violation\""),
            (ErrorType::BackendInconsistency, "\"backend_inconsistency\""),
        ];

        for (error_type, expected_json) in error_types {
            let json = serde_json::to_string(&error_type).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: ErrorType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, error_type);
        }
    }

    #[test]
    fn test_test_case_full_serialization() {
        let test_case = TestCase {
            id: "correctness-001".to_string(),
            category: TestCategory::Correctness,
            input_request: "list all files".to_string(),
            expected_command: Some("ls -la".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::ExactMatch,
            validation_pattern: None,
            tags: vec!["common".to_string()],
            difficulty: Some(Difficulty::Easy),
            source: Some("manual".to_string()),
            notes: None,
            execution: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&test_case).unwrap();

        // Deserialize back
        let deserialized: TestCase = serde_json::from_str(&json).unwrap();

        // Verify fields match
        assert_eq!(deserialized.id, test_case.id);
        assert_eq!(deserialized.category, test_case.category);
        assert_eq!(deserialized.input_request, test_case.input_request);
        assert_eq!(deserialized.validation_rule, test_case.validation_rule);
    }

    #[test]
    fn test_case_validation_passes() {
        let test_case = TestCase {
            id: "correctness-001".to_string(),
            category: TestCategory::Correctness,
            input_request: "list all files".to_string(),
            expected_command: Some("ls -la".to_string()),
            expected_behavior: None,
            validation_rule: ValidationRule::ExactMatch,
            validation_pattern: None,
            tags: vec!["common".to_string()],
            difficulty: Some(Difficulty::Easy),
            source: Some("manual".to_string()),
            notes: None,
            execution: None,
        };

        assert!(test_case.validate().is_ok());
    }

    #[test]
    fn test_case_validation_fails_empty_input() {
        let test_case = TestCase {
            id: "test-001".to_string(),
            category: TestCategory::Correctness,
            input_request: "".to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: ValidationRule::ExactMatch,
            validation_pattern: None,
            tags: vec![],
            difficulty: None,
            source: None,
            notes: None,
            execution: None,
        };

        assert!(test_case.validate().is_err());
    }

    fn result_with(passed: bool, criteria_passed: u32, criteria_total: u32) -> EvaluationResult {
        EvaluationResult {
            test_id: "t-1".to_string(),
            backend_name: "b".to_string(),
            passed,
            actual_command: None,
            actual_behavior: None,
            failure_reason: None,
            execution_time_ms: 0,
            timestamp: Utc::now(),
            error_type: None,
            est_tokens_in: 0,
            est_tokens_out: 0,
            est_cost_usd: 0.0,
            criteria_passed,
            criteria_total,
        }
    }

    #[test]
    fn score_single_criterion_derives_from_passed() {
        assert_eq!(result_with(true, 0, 0).score(), 1.0);
        assert_eq!(result_with(false, 0, 0).score(), 0.0);
    }

    #[test]
    fn score_multi_criterion_is_fraction() {
        // 8 of 10 criteria pass: mean-score 0.8 but NOT all-pass.
        let r = result_with(false, 8, 10);
        assert!((r.score() - 0.8).abs() < 1e-6);
        assert!(
            !r.passed,
            "8 of 10 is not all-pass — the article's distinction"
        );
    }

    #[test]
    fn score_all_criteria_pass_is_one() {
        let r = result_with(true, 10, 10);
        assert_eq!(r.score(), 1.0);
    }

    #[test]
    fn test_backend_profile_validation() {
        let profile = BackendProfile {
            name: "mlx".to_string(),
            display_name: "MLX".to_string(),
            enabled: true,
            timeout_ms: 10000,
            required_features: vec!["macos".to_string()],
            evaluation_priority: EvaluationPriority::Deep,
            test_sampling_rate: 1.0,
        };

        assert!(profile.validate().is_ok());
    }
}
