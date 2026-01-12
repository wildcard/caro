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
}

/// Main entry point for custom test harness
#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let args = Args::parse();

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
    };

    // Note: Backend filtering is not yet supported through HarnessConfig
    // This would require modifying the harness initialization
    if args.backend.is_some() {
        eprintln!("Warning: Backend filtering is not yet implemented. Running all backends.");
    }

    let mut harness = EvaluationHarness::new(filtered_dataset, config)?;

    // Register backends
    // Always register static_matcher as it's always available
    let static_matcher = Arc::new(caro::backends::StaticMatcher::new(
        caro::prompts::CapabilityProfile::ubuntu(),
    ));
    harness.add_backend("static_matcher".to_string(), static_matcher);

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

        let delta = BaselineStore::compare(&report, &baseline, args.threshold);

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
    let exit_code = if regression_detected {
        1 // Regression detected
    } else if report.overall_pass_rate < 1.0 {
        1 // Some tests failed
    } else {
        0 // All tests passed
    };

    Ok(exit_code)
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
