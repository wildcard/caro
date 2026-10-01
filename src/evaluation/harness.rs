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
use crate::evaluation::calibration::{
    decision_failure_count, risk_agreement, source_counts, CalibrationRollup, LatencyPercentiles,
};
use crate::evaluation::errors::Result;
use crate::evaluation::{
    BackendResult, BenchmarkReport, CategoryResult, CommandResult, Dataset, ErrorType,
    EvaluationResult, Evaluator, TestCase, TestCategory,
};
use crate::models::{CommandRequest, RiskJudgeContext, ShellType};

/// Aggregated estimated cost over a set of evaluation results.
///
/// Shared by the per-backend (`run_backend`) and full-run aggregation paths so
/// the rollup math lives in one place. `cost_per_passed_task` is the article's
/// headline comparison axis — cost normalized by tasks actually passed.
#[derive(Debug, Clone, Copy, Default)]
struct CostRollup {
    total_cost_usd: f64,
    cost_per_passed_task: f64,
    total_tokens_in: u64,
    total_tokens_out: u64,
}

impl CostRollup {
    /// Roll up cost/token totals from any iterator of results, normalizing cost
    /// by the number that `passed`. Single-pass so it works with both owned
    /// (`&[EvaluationResult]`) and borrowed (`Vec<&EvaluationResult>`) sets.
    fn of<'a>(results: impl IntoIterator<Item = &'a EvaluationResult>, passed: usize) -> Self {
        let mut total_cost_usd = 0.0_f64;
        let mut total_tokens_in = 0_u64;
        let mut total_tokens_out = 0_u64;
        for r in results {
            total_cost_usd += r.est_cost_usd;
            total_tokens_in += r.est_tokens_in as u64;
            total_tokens_out += r.est_tokens_out as u64;
        }
        let cost_per_passed_task = if passed > 0 {
            total_cost_usd / passed as f64
        } else {
            0.0
        };
        Self {
            total_cost_usd,
            cost_per_passed_task,
            total_tokens_in,
            total_tokens_out,
        }
    }
}

/// Mean-score over a set of results: the average per-result `score()` (fraction
/// of criteria passed). For single-criterion datasets this equals the all-pass
/// rate; it diverges (sits above pass_rate) once multi-criterion results exist.
fn mean_score_of<'a>(results: impl IntoIterator<Item = &'a EvaluationResult>) -> f32 {
    let mut sum = 0.0_f32;
    let mut n = 0_u32;
    for r in results {
        sum += r.score();
        n += 1;
    }
    if n > 0 {
        sum / n as f32
    } else {
        0.0
    }
}

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

    /// Also ask each backend's risk judge (`classify_risk`) about every
    /// generated command and count verdicts that fail to parse (#1465).
    /// Off by default: it doubles the calls per test and CI has no judge.
    pub judge_risk: bool,

    /// ECE rise over the baseline that counts as a regression (#1466),
    /// applied by [`BaselineStore::compare_with_ece`](crate::evaluation::BaselineStore::compare_with_ece).
    pub ece_regression_threshold: f32,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            backend_timeout_ms: 30_000, // 30 seconds
            skip_unavailable: true,
            regression_threshold: 0.95, // 95% pass rate
            max_concurrency: 10,
            judge_risk: false,
            ece_regression_threshold: 0.05,
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
    /// Reference labeller for risk verdicts (#1466): asked about every
    /// generated command so local verdicts can be scored for agreement.
    reference_judge: Option<Arc<dyn CommandGenerator>>,
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
            reference_judge: None,
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

    /// Set the reference labeller (#1466): a backend with a risk judge whose
    /// verdicts are stored as `reference_risk` on every result. Its labels
    /// are a model's opinion, not ground truth.
    pub fn set_reference_judge(&mut self, judge: Arc<dyn CommandGenerator>) {
        self.reference_judge = Some(judge);
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
            mean_score: mean_score_of(all_results.iter()),
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

        self.enrich_costs(&mut all_results);

        // Calculate backend metrics
        let total = all_results.len();
        let passed = all_results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let pass_rate = if total > 0 {
            passed as f32 / total as f32
        } else {
            0.0
        };
        let cost = CostRollup::of(all_results.iter(), passed);
        let calibration = CalibrationRollup::of(all_results.iter());
        let latency = LatencyPercentiles::of(all_results.iter());
        let (agreement, disagreements) = risk_agreement(all_results.iter());

        Ok(BackendResult {
            backend_name: backend_name.to_string(),
            pass_rate,
            mean_score: mean_score_of(all_results.iter()),
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
            total_cost_usd: cost.total_cost_usd,
            cost_per_passed_task: cost.cost_per_passed_task,
            total_tokens_in: cost.total_tokens_in,
            total_tokens_out: cost.total_tokens_out,
            brier: calibration.brier,
            ece: calibration.ece,
            confidence_coverage: calibration.coverage,
            p50_execution_time_ms: latency.p50_ms,
            p95_execution_time_ms: latency.p95_ms,
            confidence_sources: source_counts(all_results.iter()),
            decision_parse_failures: decision_failure_count(all_results.iter()),
            risk_agreement: agreement,
            risk_disagreements: disagreements,
        })
    }

    /// Filters backends to only those currently available
    async fn filter_available_backends(&self) -> Vec<(String, Arc<dyn CommandGenerator>)> {
        let mut available = Vec::new();

        for (name, backend) in &self.backends {
            if backend.is_available().await || !self.config.skip_unavailable {
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
        // Bound in-flight backend work by `max_concurrency`: the permit is
        // taken before spawning and held for generation, the optional judge
        // pass (#1465) and evaluation.
        let limiter = Arc::new(tokio::sync::Semaphore::new(
            self.config.max_concurrency.max(1),
        ));

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
                let judge_risk = self.config.judge_risk;
                let reference_judge = self.reference_judge.clone();
                let permit = limiter
                    .clone()
                    .acquire_owned()
                    .await
                    .expect("semaphore is never closed");

                // Spawn parallel task for this backend
                let task = tokio::spawn(async move {
                    let _permit = permit;

                    // Run backend with timeout
                    let mut command_result = match timeout(
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
                                confidence: None,
                                confidence_source: None,
                                decision_failed: None,
                                local_risk: None,
                                reference_risk: None,
                            }
                        }
                    };
                    if judge_risk {
                        Self::judge_command_for_test(&mut command_result, &backend, timeout_ms)
                            .await;
                    }
                    if let Some(judge) = &reference_judge {
                        Self::reference_label_for_test(&mut command_result, judge, timeout_ms)
                            .await;
                    }

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

        self.enrich_costs(&mut results);
        Ok(results)
    }

    /// Populate estimated token/cost fields on each result.
    ///
    /// Implements the Fireworks "cost as a first-class metric" idea: every
    /// generation gets an estimated USD figure so backends are comparable on
    /// price-per-task, not just pass-rate. Local/self-hosted backends price to
    /// $0; hosted frontier APIs price to a positive figure. Token counts are
    /// estimated from text length — see [`crate::evaluation::pricing`].
    fn enrich_costs(&self, results: &mut [EvaluationResult]) {
        let input_by_id: HashMap<&str, &str> = self
            .dataset
            .test_cases()
            .iter()
            .map(|tc| (tc.id.as_str(), tc.input_request.as_str()))
            .collect();

        for r in results.iter_mut() {
            let input = input_by_id.get(r.test_id.as_str()).copied().unwrap_or("");
            let output = r.actual_command.as_deref().unwrap_or("");
            let (tokens_in, tokens_out, cost) =
                crate::evaluation::pricing::estimate_generation_cost(
                    &r.backend_name,
                    input,
                    output,
                );
            r.est_tokens_in = tokens_in;
            r.est_tokens_out = tokens_out;
            r.est_cost_usd = cost;
        }
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
                    est_tokens_in: 0,
                    est_tokens_out: 0,
                    est_cost_usd: 0.0,
                    criteria_passed: 0,
                    criteria_total: 0,
                    confidence: None,
                    confidence_source: None,
                    decision_failed: None,
                    local_risk: None,
                    reference_risk: None,
                };
            }
        };

        // Generate command with timeout
        let mut command_result = match timeout(
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
                confidence: None,
                confidence_source: None,
                decision_failed: None,
                local_risk: None,
                reference_risk: None,
            },
        };
        if self.config.judge_risk {
            Self::judge_command_for_test(
                &mut command_result,
                &backend,
                self.config.backend_timeout_ms,
            )
            .await;
        }
        if let Some(judge) = &self.reference_judge {
            Self::reference_label_for_test(
                &mut command_result,
                judge,
                self.config.backend_timeout_ms,
            )
            .await;
        }

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
                est_tokens_in: 0,
                est_tokens_out: 0,
                est_cost_usd: 0.0,
                criteria_passed: 0,
                criteria_total: 0,
                confidence: command_result.confidence,
                confidence_source: command_result.confidence_source,
                decision_failed: None,
                local_risk: None,
                reference_risk: None,
            },
        }
    }

    /// Optional decision pass (#1465): ask the backend's risk judge about a
    /// generated command under its own `timeout_ms` budget, separate from
    /// generation's, and record `decision_failed`. Backends without a judge
    /// (`supports_risk_judge() == false`) are left at `None` so they never
    /// count as failures; a judge timeout counts as one. The judge sees the
    /// same context the CLI gives it, minus the static verdict, which the
    /// harness does not compute.
    async fn judge_command_for_test(
        result: &mut CommandResult,
        backend: &Arc<dyn CommandGenerator>,
        timeout_ms: u64,
    ) {
        let Some(command) = result.command.as_deref() else {
            return;
        };
        if !backend.supports_risk_judge() {
            return;
        }
        let ctx = RiskJudgeContext {
            shell: ShellType::Bash,
            cwd: None,
            static_risk: crate::models::RiskLevel::Safe,
            matched_patterns: vec![],
        };
        let verdict = timeout(
            Duration::from_millis(timeout_ms),
            backend.classify_risk(command, &ctx),
        )
        .await;
        result.decision_failed = Some(!matches!(verdict, Ok(Some(_))));
        result.local_risk = verdict.ok().flatten();
    }

    /// Reference labelling (#1466): ask the configured reference judge for
    /// its verdict on the generated command, under the same per-call budget.
    async fn reference_label_for_test(
        result: &mut CommandResult,
        judge: &Arc<dyn CommandGenerator>,
        timeout_ms: u64,
    ) {
        let Some(command) = result.command.as_deref() else {
            return;
        };
        if !judge.supports_risk_judge() {
            return;
        }
        let ctx = RiskJudgeContext {
            shell: ShellType::Bash,
            cwd: None,
            static_risk: crate::models::RiskLevel::Safe,
            matched_patterns: vec![],
        };
        result.reference_risk = timeout(
            Duration::from_millis(timeout_ms),
            judge.classify_risk(command, &ctx),
        )
        .await
        .ok()
        .flatten();
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

                let confidence = generated
                    .has_confidence()
                    .then_some(generated.confidence_score);
                CommandResult {
                    command: Some(generated.command),
                    blocked: false,
                    error: None,
                    execution_time_ms,
                    backend_name: backend_name.to_string(),
                    confidence,
                    confidence_source: Some(generated.confidence_source),
                    decision_failed: None,
                    local_risk: None,
                    reference_risk: None,
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
                    confidence: None,
                    confidence_source: None,
                    decision_failed: None,
                    local_risk: None,
                    reference_risk: None,
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
        let total_cost_usd: f64 = results.iter().map(|r| r.est_cost_usd).sum();
        let overall_mean_score = mean_score_of(results.iter());

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
                    mean_score: mean_score_of(category_tests.iter().copied()),
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
            let cost = CostRollup::of(backend_tests.iter().copied(), passed);
            let calibration = CalibrationRollup::of(backend_tests.iter().copied());
            let latency = LatencyPercentiles::of(backend_tests.iter().copied());
            let (agreement, disagreements) = risk_agreement(backend_tests.iter().copied());

            backend_results.insert(
                backend_name.clone(),
                BackendResult {
                    backend_name: backend_name.clone(),
                    total_tests: total,
                    passed,
                    failed,
                    pass_rate,
                    mean_score: mean_score_of(backend_tests.iter().copied()),
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
                    total_cost_usd: cost.total_cost_usd,
                    cost_per_passed_task: cost.cost_per_passed_task,
                    total_tokens_in: cost.total_tokens_in,
                    total_tokens_out: cost.total_tokens_out,
                    brier: calibration.brier,
                    ece: calibration.ece,
                    confidence_coverage: calibration.coverage,
                    p50_execution_time_ms: latency.p50_ms,
                    p95_execution_time_ms: latency.p95_ms,
                    confidence_sources: source_counts(backend_tests.iter().copied()),
                    decision_parse_failures: decision_failure_count(backend_tests.iter().copied()),
                    risk_agreement: agreement,
                    risk_disagreements: disagreements,
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
            overall_mean_score,
            total_tests,
            total_passed,
            total_failed,
            category_results,
            backend_results,
            total_cost_usd,
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
        /// `Some(verdict)` = judge supported (`None` inside = judge failed);
        /// `None` = no judge at all.
        judge: Option<Option<crate::models::RiskJudgment>>,
    }

    impl MockBackend {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                available: true,
                should_fail: false,
                should_timeout: false,
                judge: None,
            }
        }

        fn unavailable(name: &str) -> Self {
            Self {
                name: name.to_string(),
                available: false,
                should_fail: false,
                should_timeout: false,
                judge: None,
            }
        }

        fn with_judge(name: &str, verdict: Option<crate::models::RiskJudgment>) -> Self {
            Self {
                judge: Some(verdict),
                ..Self::new(name)
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
                confidence_source: crate::models::ConfidenceSource::SelfReported,
            })
        }

        fn supports_risk_judge(&self) -> bool {
            self.judge.is_some()
        }

        async fn classify_risk(
            &self,
            _command: &str,
            _ctx: &RiskJudgeContext,
        ) -> Option<crate::models::RiskJudgment> {
            self.judge.clone().flatten()
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
        // Single-criterion dataset: mean-score equals the all-pass rate by
        // construction (the article's two metrics coincide until multi-criterion
        // cases exist).
        assert!((report.overall_mean_score - report.overall_pass_rate).abs() < 1e-6);
        for backend in report.backend_results.values() {
            assert!((backend.mean_score - backend.pass_rate).abs() < 1e-6);
        }
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
    async fn test_hosted_backend_accrues_estimated_cost() {
        let dataset = create_simple_dataset();
        let mut harness = EvaluationHarness::new(dataset, HarnessConfig::default()).unwrap();
        // Registered name resolves to a hosted (priced) backend in the pricing table.
        harness.add_backend(
            "claude-opus-eval".to_string(),
            Arc::new(MockBackend::new("mock")),
        );

        let report = harness.run().await.unwrap();

        assert!(
            report.total_cost_usd > 0.0,
            "hosted backend should accrue non-zero estimated cost"
        );
        let backend = report.backend_results.get("claude-opus-eval").unwrap();
        assert!(backend.total_cost_usd > 0.0);
        assert!(backend.total_tokens_in > 0);
        assert!(backend.total_tokens_out > 0);
        // cost_per_passed_task is cost normalized by passed count (>= 0 always).
        assert!(backend.cost_per_passed_task >= 0.0);
        if backend.passed > 0 {
            assert!(backend.cost_per_passed_task > 0.0);
        }
    }

    #[tokio::test]
    async fn test_local_backend_is_free_but_counts_tokens() {
        let dataset = create_simple_dataset();
        let mut harness = EvaluationHarness::new(dataset, HarnessConfig::default()).unwrap();
        // "embedded" resolves to a local (free) backend in the pricing table.
        harness.add_backend("embedded".to_string(), Arc::new(MockBackend::new("mock")));

        let report = harness.run().await.unwrap();

        assert_eq!(report.total_cost_usd, 0.0, "local backend must be free");
        let backend = report.backend_results.get("embedded").unwrap();
        assert_eq!(backend.total_cost_usd, 0.0);
        // Tokens are still counted even though the cost is zero.
        assert!(backend.total_tokens_in > 0);
        assert!(backend.total_tokens_out > 0);
        assert_eq!(backend.cost_per_passed_task, 0.0);
    }

    #[tokio::test]
    async fn test_harness_skips_unavailable_backend() {
        let dataset = create_simple_dataset();
        let config = HarnessConfig {
            skip_unavailable: true,
            ..Default::default()
        };

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
    async fn reference_labels_score_agreement_per_backend() {
        let dataset = create_simple_dataset();
        let config = HarnessConfig {
            judge_risk: true,
            ..Default::default()
        };
        let verdict = |risk| crate::models::RiskJudgment {
            risk,
            reason: "mock".to_string(),
            confidence: 0.9,
        };

        let mut harness = EvaluationHarness::new(dataset, config).unwrap();
        harness.set_reference_judge(Arc::new(MockBackend::with_judge(
            "reference",
            Some(verdict(crate::models::RiskLevel::Safe)),
        )));
        harness.add_backend(
            "agrees".to_string(),
            Arc::new(MockBackend::with_judge(
                "agrees",
                Some(verdict(crate::models::RiskLevel::Safe)),
            )),
        );
        harness.add_backend(
            "disagrees".to_string(),
            Arc::new(MockBackend::with_judge(
                "disagrees",
                Some(verdict(crate::models::RiskLevel::High)),
            )),
        );
        harness.add_backend(
            "no_judge".to_string(),
            Arc::new(MockBackend::new("no_judge")),
        );

        let report = harness.run().await.unwrap();
        let agreement = |name: &str| {
            let r = &report.backend_results[name];
            (r.risk_agreement, r.risk_disagreements)
        };
        assert_eq!(agreement("agrees"), (Some(1.0), 0));
        assert_eq!(agreement("disagrees"), (Some(0.0), 2));
        assert_eq!(
            agreement("no_judge"),
            (None, 0),
            "reference alone is not agreement"
        );
    }

    #[tokio::test]
    async fn judge_failures_counted_only_for_backends_with_a_judge() {
        let dataset = create_simple_dataset();
        let config = HarnessConfig {
            judge_risk: true,
            ..Default::default()
        };

        let mut harness = EvaluationHarness::new(dataset, config).unwrap();
        harness.add_backend(
            "no_judge".to_string(),
            Arc::new(MockBackend::new("no_judge")),
        );
        harness.add_backend(
            "judge_ok".to_string(),
            Arc::new(MockBackend::with_judge(
                "judge_ok",
                Some(crate::models::RiskJudgment {
                    risk: crate::models::RiskLevel::Safe,
                    reason: "mock".to_string(),
                    confidence: 0.9,
                }),
            )),
        );
        harness.add_backend(
            "judge_fails".to_string(),
            Arc::new(MockBackend::with_judge("judge_fails", None)),
        );

        let report = harness.run().await.unwrap();
        let failures = |name: &str| report.backend_results[name].decision_parse_failures;
        assert_eq!(failures("no_judge"), 0, "no judge is not a failed judge");
        assert_eq!(failures("judge_ok"), 0);
        assert_eq!(failures("judge_fails"), 2, "one per test case");
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
