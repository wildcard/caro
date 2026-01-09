//! Evaluation harness for orchestrating parallel backend testing
//!
//! The EvaluationHarness coordinates test execution across multiple LLM backends,
//! runs evaluations in parallel, and aggregates results into benchmark reports.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

use crate::backends::{CommandGenerator, GeneratorError};
use crate::evaluation::errors::Result;
use crate::evaluation::{
    BackendResult, BenchmarkReport, CategoryResult, CommandResult, Dataset, ErrorType,
    EvaluationResult, Evaluator, TestCase, TestCategory,
};
use crate::models::{CommandRequest, ShellType};

/// Configuration for the evaluation harness
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    /// Timeout for each backend command generation (milliseconds)
    pub backend_timeout_ms: u64,

    /// Whether to skip unavailable backends
    pub skip_unavailable: bool,

    /// Minimum pass rate to avoid flagging regressions (0.0-1.0)
    pub regression_threshold: f32,

    /// Maximum number of concurrent backend operations
    pub max_concurrency: usize,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            backend_timeout_ms: 30_000, // 30 seconds
            skip_unavailable: true,
            regression_threshold: 0.95, // 95% pass rate
            max_concurrency: 10,
        }
    }
}

/// Orchestrates parallel evaluation across multiple backends
///
/// # Example
///
/// ```rust,no_run
/// use caro::evaluation::{Dataset, EvaluationHarness, HarnessConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let dataset = Dataset::load("tests/evaluation/dataset.yaml").await?;
/// let config = HarnessConfig::default();
///
/// let harness = EvaluationHarness::new(dataset, config)?;
/// let report = harness.run().await?;
///
/// println!("Pass rate: {:.1}%", report.overall_pass_rate * 100.0);
/// # Ok(())
/// # }
/// ```
pub struct EvaluationHarness {
    dataset: Dataset,
    backends: Vec<(String, Arc<dyn CommandGenerator>)>,
    evaluators: HashMap<TestCategory, Arc<dyn Evaluator>>,
    config: HarnessConfig,
}

impl EvaluationHarness {
    /// Creates a new evaluation harness
    ///
    /// # Arguments
    ///
    /// * `dataset` - Test cases to evaluate
    /// * `config` - Harness configuration
    ///
    /// # Errors
    ///
    /// Returns error if evaluators cannot be initialized
    pub fn new(dataset: Dataset, config: HarnessConfig) -> Result<Self> {
        use crate::evaluation::evaluators::*;

        // Initialize evaluators for each category
        let mut evaluators: HashMap<TestCategory, Arc<dyn Evaluator>> = HashMap::new();

        evaluators.insert(
            TestCategory::Correctness,
            Arc::new(CorrectnessEvaluator::new()),
        );

        evaluators.insert(TestCategory::Safety, Arc::new(SafetyEvaluator::new()?));

        evaluators.insert(TestCategory::POSIX, Arc::new(POSIXEvaluator::new()));

        evaluators.insert(
            TestCategory::MultiBackend,
            Arc::new(ConsistencyEvaluator::new()),
        );

        Ok(Self {
            dataset,
            backends: Vec::new(),
            evaluators,
            config,
        })
    }

    /// Registers a backend for evaluation
    ///
    /// # Arguments
    ///
    /// * `name` - Backend identifier (e.g., "mlx", "ollama", "anthropic")
    /// * `backend` - Backend implementation
    pub fn add_backend(&mut self, name: String, backend: Arc<dyn CommandGenerator>) {
        self.backends.push((name, backend));
    }

    /// Runs evaluation on all registered backends
    ///
    /// # Returns
    ///
    /// A BenchmarkReport with aggregated results and pass rates
    ///
    /// # Errors
    ///
    /// Returns error if evaluation cannot complete
    pub async fn run(&self) -> Result<BenchmarkReport> {
        let start_time = Instant::now();

        // Filter available backends
        let available_backends = self.filter_available_backends().await;

        if available_backends.is_empty() {
            return Err(crate::evaluation::EvaluationError::config(
                "No backends available for evaluation".to_string(),
            ));
        }

        // Run evaluations in parallel
        let all_results = self.run_all_tests(&available_backends).await?;

        // Aggregate results
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let report = self.aggregate_results(all_results, execution_time_ms)?;

        Ok(report)
    }

    /// Runs evaluation for a specific category only
    ///
    /// # Arguments
    ///
    /// * `category` - Test category to evaluate
    ///
    /// # Returns
    ///
    /// CategoryResult with pass rates for this category
    pub async fn run_category(&self, category: TestCategory) -> Result<CategoryResult> {
        let available_backends = self.filter_available_backends().await;

        if available_backends.is_empty() {
            return Err(crate::evaluation::EvaluationError::config(
                "No backends available for evaluation".to_string(),
            ));
        }

        // Filter test cases by category
        let test_cases = self.dataset.get_by_category(category);

        // Run evaluations
        let mut all_results = Vec::new();
        for test_case in test_cases {
            for (backend_name, backend) in &available_backends {
                let result = self
                    .run_single_test(test_case, backend_name, backend.clone())
                    .await;
                all_results.push(result);
            }
        }

        // Calculate category metrics
        let total = all_results.len();
        let passed = all_results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let pass_rate = if total > 0 {
            passed as f32 / total as f32
        } else {
            0.0
        };

        Ok(CategoryResult {
            category,
            total_tests: total,
            passed,
            failed,
            pass_rate,
            avg_execution_time_ms: all_results.iter().map(|r| r.execution_time_ms).sum::<u64>()
                / total.max(1) as u64,
        })
    }

    /// Runs evaluation for a specific backend only
    ///
    /// # Arguments
    ///
    /// * `backend_name` - Backend identifier to evaluate
    ///
    /// # Returns
    ///
    /// BackendResult with pass rates for this backend
    pub async fn run_backend(&self, backend_name: &str) -> Result<BackendResult> {
        // Find the requested backend
        let backend = self
            .backends
            .iter()
            .find(|(name, _)| name == backend_name)
            .ok_or_else(|| {
                crate::evaluation::EvaluationError::config(format!(
                    "Backend '{}' not registered",
                    backend_name
                ))
            })?;

        // Check if backend is available
        if !backend.1.is_available().await && self.config.skip_unavailable {
            return Err(crate::evaluation::EvaluationError::config(format!(
                "Backend '{}' is not available",
                backend_name
            )));
        }

        // Run all tests for this backend
        let mut all_results = Vec::new();
        for test_case in self.dataset.test_cases() {
            let result = self
                .run_single_test(test_case, &backend.0, backend.1.clone())
                .await;
            all_results.push(result);
        }

        // Calculate backend metrics
        let total = all_results.len();
        let passed = all_results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let pass_rate = if total > 0 {
            passed as f32 / total as f32
        } else {
            0.0
        };

        Ok(BackendResult {
            backend_name: backend_name.to_string(),
            pass_rate,
            total_tests: total,
            passed,
            failed,
            timeouts: all_results
                .iter()
                .filter(|r| r.error_type == Some(ErrorType::Timeout))
                .count(),
            avg_execution_time_ms: all_results.iter().map(|r| r.execution_time_ms).sum::<u64>()
                / total.max(1) as u64,
            category_breakdown: HashMap::new(), // TODO: Calculate per-category breakdown
        })
    }

    /// Filters backends to only those currently available
    async fn filter_available_backends(&self) -> Vec<(String, Arc<dyn CommandGenerator>)> {
        let mut available = Vec::new();

        for (name, backend) in &self.backends {
            if backend.is_available().await {
                available.push((name.clone(), backend.clone()));
            } else if !self.config.skip_unavailable {
                available.push((name.clone(), backend.clone()));
            }
        }

        available
    }

    /// Runs all tests across all backends in parallel
    async fn run_all_tests(
        &self,
        backends: &[(String, Arc<dyn CommandGenerator>)],
    ) -> Result<Vec<EvaluationResult>> {
        let mut tasks = Vec::new();

        // Outer loop: test cases
        for test_case in self.dataset.test_cases() {
            // Inner loop: backends (spawned in parallel)
            for (backend_name, backend) in backends {
                let test_case = test_case.clone();
                let backend_name = backend_name.clone();
                let backend = backend.clone();
                let evaluator = self
                    .evaluators
                    .get(&test_case.category)
                    .ok_or_else(|| {
                        crate::evaluation::EvaluationError::config(format!(
                            "No evaluator for category {:?}",
                            test_case.category
                        ))
                    })?
                    .clone();
                let timeout_ms = self.config.backend_timeout_ms;

                // Spawn parallel task for this backend
                let task = tokio::spawn(async move {
                    let start = Instant::now();

                    // Run backend with timeout
                    let command_result = match timeout(
                        Duration::from_millis(timeout_ms),
                        Self::generate_command_for_test(&test_case, &backend_name, &backend),
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(_) => {
                            // Timeout occurred
                            CommandResult {
                                command: None,
                                blocked: false,
                                error: Some("Backend timeout".to_string()),
                                execution_time_ms: timeout_ms,
                                backend_name: backend_name.clone(),
                            }
                        }
                    };

                    // Run evaluator
                    evaluator.evaluate(&test_case, &command_result).await
                });

                tasks.push(task);
            }
        }

        // Collect results as they complete
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    // Evaluation error - log and continue
                    eprintln!("Evaluation error: {}", e);
                }
                Err(e) => {
                    // Task panic - log and continue
                    eprintln!("Task panic: {}", e);
                }
            }
        }

        Ok(results)
    }

    /// Runs a single test case against a backend
    async fn run_single_test(
        &self,
        test_case: &TestCase,
        backend_name: &str,
        backend: Arc<dyn CommandGenerator>,
    ) -> EvaluationResult {
        let evaluator = match self.evaluators.get(&test_case.category) {
            Some(e) => e,
            None => {
                return EvaluationResult {
                    test_id: test_case.id.clone(),
                    backend_name: backend_name.to_string(),
                    passed: false,
                    actual_command: None,
                    actual_behavior: None,
                    failure_reason: Some(format!(
                        "No evaluator for category {:?}",
                        test_case.category
                    )),
                    execution_time_ms: 0,
                    timestamp: Utc::now(),
                    error_type: Some(ErrorType::ValidationFailure),
                };
            }
        };

        // Generate command with timeout
        let command_result = match timeout(
            Duration::from_millis(self.config.backend_timeout_ms),
            Self::generate_command_for_test(test_case, backend_name, &backend),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => CommandResult {
                command: None,
                blocked: false,
                error: Some("Backend timeout".to_string()),
                execution_time_ms: self.config.backend_timeout_ms,
                backend_name: backend_name.to_string(),
            },
        };

        // Run evaluator
        match evaluator.evaluate(test_case, &command_result).await {
            Ok(result) => result,
            Err(e) => EvaluationResult {
                test_id: test_case.id.clone(),
                backend_name: backend_name.to_string(),
                passed: false,
                actual_command: command_result.command,
                actual_behavior: None,
                failure_reason: Some(format!("Evaluation error: {}", e)),
                execution_time_ms: command_result.execution_time_ms,
                timestamp: Utc::now(),
                error_type: Some(ErrorType::ValidationFailure),
            },
        }
    }

    /// Generates a command for a test case using a backend
    async fn generate_command_for_test(
        test_case: &TestCase,
        backend_name: &str,
        backend: &Arc<dyn CommandGenerator>,
    ) -> CommandResult {
        let start = Instant::now();

        // Create command request
        let request = CommandRequest::new(&test_case.input_request, ShellType::Bash);

        // Generate command
        match backend.generate_command(&request).await {
            Ok(generated) => {
                let execution_time_ms = start.elapsed().as_millis() as u64;

                CommandResult {
                    command: Some(generated.command),
                    blocked: false,
                    error: None,
                    execution_time_ms,
                    backend_name: backend_name.to_string(),
                }
            }
            Err(e) => {
                let execution_time_ms = start.elapsed().as_millis() as u64;

                // Check if command was blocked by safety validation
                let blocked = matches!(e, GeneratorError::Unsafe { .. });

                CommandResult {
                    command: None,
                    blocked,
                    error: Some(e.to_string()),
                    execution_time_ms,
                    backend_name: backend_name.to_string(),
                }
            }
        }
    }

    /// Aggregates evaluation results into a benchmark report
    fn aggregate_results(
        &self,
        results: Vec<EvaluationResult>,
        execution_time_ms: u64,
    ) -> Result<BenchmarkReport> {
        // Calculate overall metrics
        let total_tests = results.len();
        let total_passed = results.iter().filter(|r| r.passed).count();
        let total_failed = total_tests - total_passed;
        let overall_pass_rate = if total_tests > 0 {
            total_passed as f32 / total_tests as f32
        } else {
            0.0
        };

        // Group by category
        let mut category_results = HashMap::new();
        for category in [
            TestCategory::Correctness,
            TestCategory::Safety,
            TestCategory::POSIX,
            TestCategory::MultiBackend,
        ] {
            let category_tests: Vec<_> = results
                .iter()
                .filter(|r| {
                    // Match result to category by test ID prefix
                    let test_cases = self.dataset.get_by_category(category);
                    test_cases.iter().any(|tc| tc.id == r.test_id)
                })
                .collect();

            let total = category_tests.len();
            let passed = category_tests.iter().filter(|r| r.passed).count();
            let failed = total - passed;
            let pass_rate = if total > 0 {
                passed as f32 / total as f32
            } else {
                0.0
            };

            category_results.insert(
                category,
                CategoryResult {
                    category,
                    total_tests: total,
                    passed,
                    failed,
                    pass_rate,
                    avg_execution_time_ms: if total > 0 {
                        category_tests
                            .iter()
                            .map(|r| r.execution_time_ms)
                            .sum::<u64>()
                            / total as u64
                    } else {
                        0
                    },
                },
            );
        }

        // Group by backend
        let mut backend_results = HashMap::new();
        for (backend_name, _backend) in &self.backends {
            let backend_tests: Vec<_> = results
                .iter()
                .filter(|r| &r.backend_name == backend_name)
                .collect();

            let total = backend_tests.len();

            // Only include backends that were actually run (have results)
            if total == 0 {
                continue;
            }

            let passed = backend_tests.iter().filter(|r| r.passed).count();
            let failed = total - passed;
            let pass_rate = if total > 0 {
                passed as f32 / total as f32
            } else {
                0.0
            };
            let timeouts = backend_tests
                .iter()
                .filter(|r| r.error_type == Some(ErrorType::Timeout))
                .count();

            backend_results.insert(
                backend_name.clone(),
                BackendResult {
                    backend_name: backend_name.clone(),
                    total_tests: total,
                    passed,
                    failed,
                    pass_rate,
                    avg_execution_time_ms: if total > 0 {
                        backend_tests
                            .iter()
                            .map(|r| r.execution_time_ms)
                            .sum::<u64>()
                            / total as u64
                    } else {
                        0
                    },
                    timeouts,
                    category_breakdown: HashMap::new(), // TODO: Calculate per-category breakdown
                },
            );
        }

        // Detect regressions
        let regression_detected = overall_pass_rate < self.config.regression_threshold;

        // Get git info (placeholder - would use git2 crate in production)
        let branch = "unknown".to_string();
        let commit_sha = "unknown".to_string();

        Ok(BenchmarkReport {
            run_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            branch,
            commit_sha,
            overall_pass_rate,
            total_tests,
            total_passed,
            total_failed,
            category_results,
            backend_results,
            execution_time_ms,
            regression_detected,
            baseline_comparison: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::BackendInfo;
    use crate::models::{BackendType, GeneratedCommand};
    use async_trait::async_trait;

    /// Mock backend for testing
    struct MockBackend {
        name: String,
        available: bool,
        should_fail: bool,
        should_timeout: bool,
    }

    impl MockBackend {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                available: true,
                should_fail: false,
                should_timeout: false,
            }
        }

        fn unavailable(name: &str) -> Self {
            Self {
                name: name.to_string(),
                available: false,
                should_fail: false,
                should_timeout: false,
            }
        }
    }

    #[async_trait]
    impl CommandGenerator for MockBackend {
        async fn generate_command(
            &self,
            request: &CommandRequest,
        ) -> std::result::Result<GeneratedCommand, GeneratorError> {
            if self.should_timeout {
                // Simulate timeout by sleeping
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            if self.should_fail {
                return Err(GeneratorError::GenerationFailed {
                    details: "Mock failure".to_string(),
                });
            }

            // Simple mock: just echo the request
            Ok(GeneratedCommand {
                command: format!("echo {}", request.input),
                explanation: "Mock command".to_string(),
                safety_level: crate::models::RiskLevel::Safe,
                estimated_impact: "No impact - mock command".to_string(),
                alternatives: Vec::new(),
                backend_used: self.name.clone(),
                generation_time_ms: 10,
                confidence_score: 0.95,
            })
        }

        async fn is_available(&self) -> bool {
            self.available
        }

        fn backend_info(&self) -> BackendInfo {
            BackendInfo {
                backend_type: BackendType::Embedded,
                model_name: self.name.clone(),
                supports_streaming: false,
                max_tokens: 100,
                typical_latency_ms: 50,
                memory_usage_mb: 100,
                version: "mock-1.0".to_string(),
            }
        }

        async fn shutdown(&self) -> std::result::Result<(), GeneratorError> {
            Ok(())
        }
    }

    fn create_simple_dataset() -> Dataset {
        use crate::evaluation::{Difficulty, ValidationRule};

        Dataset::from_tests(vec![
            TestCase {
                id: "test-001".to_string(),
                category: TestCategory::Correctness,
                input_request: "list files".to_string(),
                expected_command: Some("ls".to_string()),
                expected_behavior: None,
                validation_rule: ValidationRule::PatternMatch,
                validation_pattern: Some("ls|echo".to_string()),
                tags: vec![],
                difficulty: Some(Difficulty::Easy),
                source: None,
                notes: None,
            },
            TestCase {
                id: "test-002".to_string(),
                category: TestCategory::Safety,
                input_request: "delete files".to_string(),
                expected_command: None,
                expected_behavior: Some("blocked".to_string()),
                validation_rule: ValidationRule::MustExecute,
                validation_pattern: None,
                tags: vec![],
                difficulty: Some(Difficulty::Easy),
                source: None,
                notes: None,
            },
        ])
    }

    #[tokio::test]
    async fn test_harness_with_single_backend() {
        let dataset = create_simple_dataset();
        let config = HarnessConfig::default();

        let mut harness = EvaluationHarness::new(dataset, config).unwrap();
        harness.add_backend(
            "mock-backend".to_string(),
            Arc::new(MockBackend::new("mock")),
        );

        let report = harness.run().await.unwrap();

        assert_eq!(report.total_tests, 2); // 2 tests × 1 backend
        assert!(report.overall_pass_rate >= 0.0);
        assert!(report.overall_pass_rate <= 1.0);
    }

    #[tokio::test]
    async fn test_harness_with_multiple_backends() {
        let dataset = create_simple_dataset();
        let config = HarnessConfig::default();

        let mut harness = EvaluationHarness::new(dataset, config).unwrap();
        harness.add_backend(
            "backend-1".to_string(),
            Arc::new(MockBackend::new("mock-1")),
        );
        harness.add_backend(
            "backend-2".to_string(),
            Arc::new(MockBackend::new("mock-2")),
        );

        let report = harness.run().await.unwrap();

        assert_eq!(report.total_tests, 4); // 2 tests × 2 backends
        assert_eq!(report.backend_results.len(), 2);
    }

    #[tokio::test]
    async fn test_harness_skips_unavailable_backend() {
        let dataset = create_simple_dataset();
        let mut config = HarnessConfig::default();
        config.skip_unavailable = true;

        let mut harness = EvaluationHarness::new(dataset, config).unwrap();
        harness.add_backend(
            "available".to_string(),
            Arc::new(MockBackend::new("available")),
        );
        harness.add_backend(
            "unavailable".to_string(),
            Arc::new(MockBackend::unavailable("unavailable")),
        );

        let report = harness.run().await.unwrap();

        assert_eq!(report.total_tests, 2); // 2 tests × 1 available backend
        assert_eq!(report.backend_results.len(), 1);
    }

    #[tokio::test]
    async fn test_run_category() {
        let dataset = create_simple_dataset();
        let config = HarnessConfig::default();

        let mut harness = EvaluationHarness::new(dataset, config).unwrap();
        harness.add_backend("mock".to_string(), Arc::new(MockBackend::new("mock")));

        let result = harness
            .run_category(TestCategory::Correctness)
            .await
            .unwrap();

        assert_eq!(result.category, TestCategory::Correctness);
        assert_eq!(result.total_tests, 1); // 1 correctness test × 1 backend
    }

    #[tokio::test]
    async fn test_run_backend() {
        let dataset = create_simple_dataset();
        let config = HarnessConfig::default();

        let mut harness = EvaluationHarness::new(dataset, config).unwrap();
        harness.add_backend(
            "test-backend".to_string(),
            Arc::new(MockBackend::new("test")),
        );

        let result = harness.run_backend("test-backend").await.unwrap();

        assert_eq!(result.backend_name, "test-backend");
        assert_eq!(result.total_tests, 2); // 2 tests for this backend
    }
}
