//! LLM Evaluation Harness - cargo test Integration
//!
//! Custom test harness providing CLI integration for running evaluations via cargo test.
//!
//! Usage:
//! ```bash
//! # Run full evaluation
//! cargo test --test evaluation
//!
//! # Filter by category
//! cargo test --test evaluation -- --category safety
//!
//! # Filter by backend
//! cargo test --test evaluation -- --backend mlx
//!
//! # JSON output for CI/CD
//! cargo test --test evaluation -- --format json
//!
//! # Compare against baseline
//! cargo test --test evaluation -- --baseline tests/evaluation/baselines/main-latest.json
//!
//! # Set regression threshold
//! cargo test --test evaluation -- --threshold 0.10
//! ```

mod libtest_filter;

use caro::evaluation::{BaselineStore, Dataset, EvaluationHarness, HarnessConfig, TestCategory};
use clap::Parser;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

/// CLI arguments for evaluation harness
#[derive(Parser, Debug)]
#[command(name = "evaluation")]
#[command(about = "Run LLM evaluation harness", long_about = None)]
struct Args {
    /// Test category to run (correctness, safety, posix, multi_backend)
    #[arg(long)]
    category: Option<String>,

    /// Backend to test (static_matcher, mlx, ollama, vllm)
    #[arg(long)]
    backend: Option<String>,

    /// Output format (json or table)
    #[arg(long, default_value = "table")]
    format: String,

    /// Path to baseline JSON for comparison
    #[arg(long)]
    baseline: Option<PathBuf>,

    /// Regression threshold (default: 0.05 for 5%)
    #[arg(long, default_value = "0.05")]
    threshold: f32,

    /// Enable verbose logging
    #[arg(long, short)]
    verbose: bool,

    /// libtest name filter, passed by `cargo test <filter>` (#1162)
    #[arg(hide = true)]
    filter: Option<String>,
}

/// Main entry point for custom test harness
#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let args = Args::parse();

    // `cargo test safety` passes "safety" to every test binary. Skip, as
    // libtest does when no test name matches the filter.
    if !libtest_filter::should_run(args.filter.as_deref()) {
        println!(
            "\nrunning 0 tests (filter {:?} does not match `{}`)\n",
            args.filter.as_deref().unwrap_or_default(),
            libtest_filter::TARGET_NAME
        );
        process::exit(0);
    }

    // Configure logging
    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    // Validate arguments
    if let Err(e) = validate_args(&args) {
        eprintln!("Error: {}", e);
        process::exit(2); // Config error
    }

    // Run evaluation
    match run_evaluation(args).await {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            eprintln!("Evaluation failed: {}", e);
            process::exit(1);
        }
    }
}

/// Validate CLI arguments
fn validate_args(args: &Args) -> Result<(), String> {
    // Validate category if provided
    if let Some(ref category) = args.category {
        match category.as_str() {
            "correctness" | "safety" | "posix" | "multi_backend" => {}
            _ => {
                return Err(format!(
                "Invalid category: {}. Must be one of: correctness, safety, posix, multi_backend",
                category
            ))
            }
        }
    }

    // Validate backend if provided
    if let Some(ref backend) = args.backend {
        match backend.as_str() {
            "static_matcher" | "mlx" | "ollama" | "vllm" => {}
            _ => {
                return Err(format!(
                    "Invalid backend: {}. Must be one of: static_matcher, mlx, ollama, vllm",
                    backend
                ))
            }
        }
    }

    // Validate format
    match args.format.as_str() {
        "json" | "table" => {}
        _ => {
            return Err(format!(
                "Invalid format: {}. Must be either 'json' or 'table'",
                args.format
            ))
        }
    }

    // Validate threshold
    if args.threshold < 0.0 || args.threshold > 1.0 {
        return Err(format!(
            "Invalid threshold: {}. Must be between 0.0 and 1.0",
            args.threshold
        ));
    }

    // Validate baseline path exists if provided
    if let Some(ref baseline) = args.baseline {
        if !baseline.exists() {
            return Err(format!("Baseline file not found: {}", baseline.display()));
        }
    }

    Ok(())
}

/// Run evaluation with given arguments
async fn run_evaluation(args: Args) -> Result<i32, Box<dyn std::error::Error>> {
    // Load dataset
    let dataset_path = "tests/evaluation/dataset.yaml";
    let dataset = Dataset::load(dataset_path)?;

    // Apply category filter if provided by creating filtered dataset
    let filtered_dataset = if let Some(ref category_str) = args.category {
        let category = parse_category(category_str)?;
        let test_cases = dataset.get_by_category(category);
        Dataset::from_tests(test_cases.into_iter().cloned().collect())
    } else {
        dataset
    };

    // Configure harness
    let config = HarnessConfig {
        backend_timeout_ms: 30_000, // 30 seconds
        skip_unavailable: true,
        regression_threshold: 0.95, // 95% pass rate
        max_concurrency: 10,
        judge_risk: matches!(
            std::env::var("CARO_EVAL_JUDGE_RISK").as_deref(),
            Ok("1" | "true")
        ),
        ece_regression_threshold: HarnessConfig::default().ece_regression_threshold,
    };
    let ece_regression_threshold = config.ece_regression_threshold;

    // Note: Backend filtering is not yet supported through HarnessConfig
    // This would require modifying the harness initialization
    if args.backend.is_some() {
        eprintln!("Warning: Backend filtering is not yet implemented. Running all backends.");
    }

    let mut harness = EvaluationHarness::new(filtered_dataset, config)?;

    // Extra evaluated backends (#1466), opt-in because they need a live
    // server. These are the ones with a risk judge, so this is also what
    // makes `CARO_EVAL_JUDGE_RISK` and the consensus labels below produce
    // anything: the static matcher has no judge.
    //   CARO_EVAL_BACKENDS=ollama:<model>[@<url>],vllm:<model>@<url>
    if let Ok(specs) = std::env::var("CARO_EVAL_BACKENDS") {
        for spec in specs.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            match backend_from_spec(spec) {
                Ok(backend) => harness.add_backend(spec.replace('@', " "), backend),
                Err(e) => eprintln!("Warning: ignoring CARO_EVAL_BACKENDS entry {spec:?}: {e}"),
            }
        }
    }

    // Reference labeller for consensus risk labels (#1466), opt-in because
    // it costs a model call per generated command:
    //   CARO_EVAL_REFERENCE_JUDGE=ollama:<model>[@<url>] | vllm:<model>@<url>
    if let Ok(spec) = std::env::var("CARO_EVAL_REFERENCE_JUDGE") {
        match backend_from_spec(&spec) {
            Ok(judge) => harness.set_reference_judge(judge),
            Err(e) => eprintln!("Warning: ignoring CARO_EVAL_REFERENCE_JUDGE: {}", e),
        }
    }

    // Register backends
    // Always register static_matcher as it's always available
    let static_matcher = Arc::new(caro::backends::StaticMatcher::new(
        caro::prompts::CapabilityProfile::ubuntu(),
    ));
    harness.add_backend("static_matcher".to_string(), static_matcher);

    // TODO: Register second backend for MultiBackend consistency evaluation
    // The ConsistencyEvaluator.evaluate_multiple() requires ≥2 backends with
    // grouped results. Current harness runs backends independently.
    // To enable: modify Harness.run_all_tests() to group results by test_id
    // before calling evaluator, or register a second backend like:
    //
    //   let static_matcher_bsd = Arc::new(caro::backends::StaticMatcher::new(
    //       caro::prompts::CapabilityProfile::for_platform(caro::ProfileType::Bsd),
    //   ));
    //   harness.add_backend("static_matcher_bsd".to_string(), static_matcher_bsd);

    // TODO: Add other backends (MLX, Ollama, etc.) when available
    // This will be implemented in a future work package

    // Run evaluation
    let mut report = harness.run().await?;

    // Baseline comparison if provided
    let mut regression_detected = false;
    if let Some(ref baseline_path) = args.baseline {
        let store = BaselineStore::new("tests/evaluation/baselines");
        let baseline = store.load(
            baseline_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or("Invalid baseline filename")?,
        )?;

        let delta = BaselineStore::compare_with_ece(
            &report,
            &baseline,
            args.threshold,
            ece_regression_threshold,
        );

        // Check for regressions
        if !delta.significant_regressions.is_empty() {
            regression_detected = true;
        }

        report.regression_detected = regression_detected;
        report.baseline_comparison = Some(delta);
    }

    // Output results
    match args.format.as_str() {
        "json" => output_json(&report)?,
        "table" => output_table(&report)?,
        _ => unreachable!(), // Validated earlier
    }

    // Determine exit code
    let exit_code = if regression_detected || report.overall_pass_rate < 1.0 {
        1 // Regression detected or some tests failed
    } else {
        0 // All tests passed
    };

    Ok(exit_code)
}

/// Build a remote backend from `<kind>:<model>[@<url>]` (#1466), used for
/// both `CARO_EVAL_BACKENDS` entries and `CARO_EVAL_REFERENCE_JUDGE`.
#[cfg(feature = "remote-backends")]
fn backend_from_spec(spec: &str) -> Result<Arc<dyn caro::backends::CommandGenerator>, String> {
    use caro::backends::remote::{OllamaBackend, VllmBackend};
    let (kind, rest) = spec
        .split_once(':')
        .ok_or_else(|| format!("expected <kind>:<model>[@<url>], got {spec:?}"))?;
    let (model, url) = match rest.split_once('@') {
        Some((m, u)) => (m, Some(u)),
        None => (rest, None),
    };
    let parse_url = |u: &str| reqwest::Url::parse(u).map_err(|e| format!("bad url {u:?}: {e}"));
    match kind {
        "ollama" => {
            let url = parse_url(url.unwrap_or("http://localhost:11434"))?;
            OllamaBackend::new(url, model.to_string())
                .map(|b| Arc::new(b) as Arc<dyn caro::backends::CommandGenerator>)
                .map_err(|e| e.to_string())
        }
        "vllm" => {
            let url = parse_url(url.ok_or("vllm needs <model>@<url>")?)?;
            VllmBackend::new(url, model.to_string())
                .map(|b| Arc::new(b) as Arc<dyn caro::backends::CommandGenerator>)
                .map_err(|e| e.to_string())
        }
        other => Err(format!("unknown reference judge kind {other:?}")),
    }
}

#[cfg(not(feature = "remote-backends"))]
fn backend_from_spec(_spec: &str) -> Result<Arc<dyn caro::backends::CommandGenerator>, String> {
    Err("built without the remote-backends feature".to_string())
}

/// Parse category string to enum
fn parse_category(s: &str) -> Result<TestCategory, String> {
    match s {
        "correctness" => Ok(TestCategory::Correctness),
        "safety" => Ok(TestCategory::Safety),
        "posix" => Ok(TestCategory::POSIX),
        "multi_backend" => Ok(TestCategory::MultiBackend),
        _ => Err(format!("Invalid category: {}", s)),
    }
}

/// Output results as JSON
fn output_json(
    report: &caro::evaluation::BenchmarkReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

/// Output results as human-readable table
fn output_table(
    report: &caro::evaluation::BenchmarkReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔═══════════════════════════════════════════════════════════════════╗");
    println!("║              LLM Evaluation Harness - Results                    ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Run ID: {}", report.run_id);
    println!("Branch: {}", report.branch);
    println!("Commit: {}", report.commit_sha);
    println!(
        "Timestamp: {}",
        report.timestamp.format("%Y-%m-%d %H:%M:%S")
    );
    println!();

    // Overall results
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Overall Results                                                 │");
    println!("├─────────────────────────────────────────────────────────────────┤");
    println!(
        "│ Total Tests:    {:>4}                                           │",
        report.total_tests
    );
    println!(
        "│ Passed:         {:>4} ({:>5.1}%)                                  │",
        report.total_passed,
        report.overall_pass_rate * 100.0
    );
    println!(
        "│ Failed:         {:>4}                                           │",
        report.total_failed
    );
    println!(
        "│ Execution Time: {:>4}ms                                         │",
        report.execution_time_ms
    );
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    // Category results
    if !report.category_results.is_empty() {
        println!("┌─────────────────────────────────────────────────────────────────┐");
        println!("│ Results by Category                                             │");
        println!("├─────────────────┬───────┬────────┬────────┬───────────┬────────┤");
        println!("│ Category        │ Total │ Passed │ Failed │ Pass Rate │ Avg ms │");
        println!("├─────────────────┼───────┼────────┼────────┼───────────┼────────┤");

        let mut categories: Vec<_> = report.category_results.iter().collect();
        categories.sort_by_key(|(cat, _)| format!("{:?}", cat));

        for (category, result) in categories {
            println!(
                "│ {:15} │ {:>5} │ {:>6} │ {:>6} │ {:>8.1}% │ {:>6} │",
                format!("{:?}", category),
                result.total_tests,
                result.passed,
                result.failed,
                result.pass_rate * 100.0,
                result.avg_execution_time_ms
            );
        }
        println!("└─────────────────┴───────┴────────┴────────┴───────────┴────────┘");
        println!();
    }

    // Backend results
    if !report.backend_results.is_empty() {
        println!("┌─────────────────────────────────────────────────────────────────┐");
        println!("│ Results by Backend                                              │");
        println!("├─────────────────┬───────┬────────┬────────┬───────────┬────────┤");
        println!("│ Backend         │ Total │ Passed │ Failed │ Pass Rate │ Avg ms │");
        println!("├─────────────────┼───────┼────────┼────────┼───────────┼────────┤");

        let mut backends: Vec<_> = report.backend_results.iter().collect();
        backends.sort_by_key(|(name, _)| name.as_str());

        for (backend_name, result) in backends {
            println!(
                "│ {:15} │ {:>5} │ {:>6} │ {:>6} │ {:>8.1}% │ {:>6} │",
                backend_name,
                result.total_tests,
                result.passed,
                result.failed,
                result.pass_rate * 100.0,
                result.avg_execution_time_ms
            );
        }
        println!("└─────────────────┴───────┴────────┴────────┴───────────┴────────┘");
        println!();

        // Calibration + tail latency (ADR-017). A backend that reports a
        // constant confidence shows ECE == |constant - pass_rate|.
        println!("┌───────────────────────────────────────────────────────────────────────────┐");
        println!("│ Calibration & Latency Tail by Backend                                     │");
        println!("├─────────────────┬────────┬────────┬─────────┬─────────┬────────┬──────────┤");
        println!("│ Backend         │  Brier │    ECE │  p50 ms │  p95 ms │  cover │ source   │");
        println!("├─────────────────┼────────┼────────┼─────────┼─────────┼────────┼──────────┤");
        let mut backends: Vec<_> = report.backend_results.iter().collect();
        backends.sort_by_key(|(name, _)| name.as_str());
        for (backend_name, result) in backends {
            let fmt_opt = |v: Option<f32>| {
                v.map(|x| format!("{:.3}", x))
                    .unwrap_or_else(|| "n/a".into())
            };
            // Dominant confidence provenance (#1464); "mixed" when no single
            // source covers every generated result.
            let source = match result.confidence_sources.iter().max_by_key(|(_, n)| **n) {
                None => "n/a".to_string(),
                Some((name, n)) if result.confidence_sources.values().sum::<u32>() == *n => {
                    name.to_string().replace("self-reported", "self-rep")
                }
                Some(_) => "mixed".to_string(),
            };
            println!(
                "│ {:15} │ {:>6} │ {:>6} │ {:>7} │ {:>7} │ {:>5.0}% │ {:8} │",
                backend_name,
                fmt_opt(result.brier),
                fmt_opt(result.ece),
                result.p50_execution_time_ms,
                result.p95_execution_time_ms,
                result.confidence_coverage * 100.0,
                source,
            );
        }
        println!("└─────────────────┴────────┴────────┴─────────┴─────────┴────────┴──────────┘");
        // Risk-judge decision failures (#1465), only when the judge ran.
        for (backend_name, result) in &report.backend_results {
            if result.decision_parse_failures > 0 {
                println!(
                    "  {}: {} risk-judge verdict(s) failed to parse",
                    backend_name, result.decision_parse_failures
                );
            }
        }
        println!();

        // Pareto view (#1466): accuracy vs calibration vs cost vs tail latency.
        // "Up and to the left" is better; agreement is with the reference
        // labeller's risk verdicts, not with ground truth.
        println!("┌───────────────────────────────────────────────────────────────────────────┐");
        println!("│ Pareto View by Backend (pass ↑, ECE ↓, cost ↓, p95 ↓, agree ↑)            │");
        println!("├─────────────────┬────────┬────────┬────────────┬─────────┬────────────────┤");
        println!("│ Backend         │  pass  │    ECE │ $/pass     │  p95 ms │ risk agreement │");
        println!("├─────────────────┼────────┼────────┼────────────┼─────────┼────────────────┤");
        let mut backends: Vec<_> = report.backend_results.iter().collect();
        backends.sort_by_key(|(name, _)| name.as_str());
        for (backend_name, result) in backends {
            let ece = result
                .ece
                .map(|x| format!("{:.3}", x))
                .unwrap_or_else(|| "n/a".into());
            let agreement = match result.risk_agreement {
                Some(a) => format!("{:>4.0}% ({} off)", a * 100.0, result.risk_disagreements),
                None => "n/a".to_string(),
            };
            println!(
                "│ {:15} │ {:>5.1}% │ {:>6} │ {:>10.4} │ {:>7} │ {:>14} │",
                backend_name,
                result.pass_rate * 100.0,
                ece,
                result.cost_per_passed_task,
                result.p95_execution_time_ms,
                agreement,
            );
        }
        println!("└─────────────────┴────────┴────────┴────────────┴─────────┴────────────────┘");
        println!();
    }

    // Baseline comparison if available
    if let Some(ref delta) = report.baseline_comparison {
        println!("┌─────────────────────────────────────────────────────────────────┐");
        println!("│ Baseline Comparison                                             │");
        println!("├─────────────────────────────────────────────────────────────────┤");
        println!(
            "│ Baseline Run:   {}                                      │",
            delta.baseline_run_id.chars().take(24).collect::<String>()
        );
        println!(
            "│ Baseline Commit: {}                                    │",
            delta
                .baseline_commit_sha
                .chars()
                .take(7)
                .collect::<String>()
        );
        println!(
            "│ Threshold:      {:>5.1}%                                        │",
            delta.regression_threshold * 100.0
        );
        println!(
            "│ ECE Threshold:  {:>5.3}                                         │",
            delta.ece_regression_threshold
        );
        println!(
            "│ Overall Delta:  {:>+6.1}%                                       │",
            delta.overall_delta * 100.0
        );
        println!("├─────────────────────────────────────────────────────────────────┤");

        if delta.significant_regressions.is_empty() {
            println!("│ ✅ No regressions detected                                      │");
        } else {
            println!("│ ⚠️  Regressions Detected:                                       │");
            println!("├─────────────────────────────────────────────────────────────────┤");
            for regression in &delta.significant_regressions {
                // Truncate long regression messages
                let msg = if regression.len() > 63 {
                    format!("{}...", &regression[..60])
                } else {
                    regression.clone()
                };
                println!("│   • {:60} │", msg);
            }
        }
        println!("└─────────────────────────────────────────────────────────────────┘");
        println!();
    }

    // Final status
    if report.regression_detected {
        println!("❌ FAIL: Regressions detected");
    } else if report.total_failed > 0 {
        println!("❌ FAIL: {} tests failed", report.total_failed);
    } else {
        println!("✅ PASS: All tests passed");
    }
    println!();

    Ok(())
}
