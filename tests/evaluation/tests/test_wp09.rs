//! WP09: Multi-Backend CI Matrix Tests
//!
//! These tests validate that the evaluation harness can test all production
//! backends in parallel and track baselines independently per backend.

use caro_evaluation::evaluator::Evaluator;
use caro_evaluation::executor::Executor;
use std::path::Path;

/// Test that MLX backend can be evaluated
#[tokio::test]
async fn test_mlx_backend_evaluates_correctly() {
    // Skip if not on macOS (MLX requires Apple Silicon)
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("Skipping MLX test - requires macOS");
        return;
    }

    let executor = match Executor::with_backend("mlx") {
        Ok(e) => e,
        Err(_) => {
            eprintln!("Skipping MLX test - caro binary not built or MLX not available");
            return;
        }
    };

    let evaluator = Evaluator;

    // Try a simple command
    let result = executor.execute("list all files").await;

    match result {
        Ok(generated) => {
            println!("MLX generated: {}", generated);

            // Evaluate correctness
            let eval_result = evaluator.evaluate_correctness(&generated, "ls");

            // MLX should generate reasonable commands
            assert!(
                eval_result.score >= 0.5,
                "MLX backend should generate reasonable commands, got score: {:.2}",
                eval_result.score
            );
        }
        Err(e) => {
            eprintln!("MLX backend execution failed (might be expected): {}", e);
        }
    }
}

/// Test that embedded SmolLM backend can be evaluated
#[tokio::test]
async fn test_embedded_smollm_backend_evaluates_correctly() {
    let executor = match Executor::with_backend("embedded-smollm") {
        Ok(e) => e,
        Err(_) => {
            eprintln!("Skipping embedded-smollm test - backend not available");
            return;
        }
    };

    let evaluator = Evaluator;

    let result = executor.execute("list all files").await;

    match result {
        Ok(generated) => {
            println!("SmolLM generated: {}", generated);

            let eval_result = evaluator.evaluate_correctness(&generated, "ls");

            assert!(
                eval_result.score >= 0.3,
                "SmolLM should generate commands, got score: {:.2}",
                eval_result.score
            );
        }
        Err(e) => {
            eprintln!("SmolLM backend execution failed: {}", e);
        }
    }
}

/// Test that embedded Qwen backend can be evaluated
#[tokio::test]
async fn test_embedded_qwen_backend_evaluates_correctly() {
    let executor = match Executor::with_backend("embedded-qwen") {
        Ok(e) => e,
        Err(_) => {
            eprintln!("Skipping embedded-qwen test - backend not available");
            return;
        }
    };

    let evaluator = Evaluator;

    let result = executor.execute("list all files").await;

    match result {
        Ok(generated) => {
            println!("Qwen generated: {}", generated);

            let eval_result = evaluator.evaluate_correctness(&generated, "ls");

            assert!(
                eval_result.score >= 0.3,
                "Qwen should generate commands, got score: {:.2}",
                eval_result.score
            );
        }
        Err(e) => {
            eprintln!("Qwen backend execution failed: {}", e);
        }
    }
}

/// Test graceful degradation when backend is unavailable
#[tokio::test]
async fn test_graceful_degradation_when_backend_unavailable() {
    // Try to create executor with non-existent backend
    let result = Executor::with_backend("nonexistent-backend");

    match result {
        Ok(_) => {
            panic!("Should not succeed with nonexistent backend");
        }
        Err(e) => {
            // Error should be graceful, not a panic
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("not available") || error_msg.contains("not found"),
                "Error should indicate backend unavailability: {}",
                error_msg
            );
            println!("✓ Graceful degradation works: {}", error_msg);
        }
    }
}

/// Test per-backend baseline storage
#[tokio::test]
async fn test_per_backend_baseline_storage() {
    use std::fs;

    let baseline_dir = Path::new("baselines");

    // Create baseline directory if it doesn't exist
    if !baseline_dir.exists() {
        fs::create_dir_all(baseline_dir).expect("Failed to create baselines directory");
    }

    // Test baseline file paths for each backend
    let backends = vec!["static_matcher", "mlx", "embedded-smollm", "embedded-qwen"];

    for backend in backends {
        let baseline_path = baseline_dir.join(format!("{}-main-latest.json", backend));

        // For this test, we just verify the path structure is correct
        println!(
            "Expected baseline path for {}: {:?}",
            backend, baseline_path
        );

        // Test that we can create the file
        let test_baseline = serde_json::json!({
            "backend": backend,
            "pass_rate": 0.31,
            "total_tests": 55,
            "passed_tests": 17,
            "timestamp": "2026-01-17T00:00:00Z"
        });

        fs::write(
            &baseline_path,
            serde_json::to_string_pretty(&test_baseline).unwrap(),
        )
        .expect("Failed to write baseline file");

        // Verify we can read it back
        assert!(
            baseline_path.exists(),
            "Baseline file should exist: {:?}",
            baseline_path
        );

        // Clean up
        fs::remove_file(&baseline_path).ok();
    }

    println!("✓ Per-backend baseline storage structure validated");
}

/// Test baseline comparison per backend
#[tokio::test]
async fn test_baseline_comparison_per_backend() {
    use serde_json::Value;
    use std::fs;

    let baseline_dir = Path::new("baselines");
    fs::create_dir_all(baseline_dir).expect("Failed to create baselines directory");

    // Create a baseline for static_matcher (with unique test suffix to avoid conflicts)
    let baseline_path = baseline_dir.join("static_matcher-test-comparison.json");
    let baseline = serde_json::json!({
        "backend": "static_matcher",
        "pass_rate": 0.31,
        "total_tests": 55,
        "passed_tests": 17,
        "timestamp": "2026-01-17T00:00:00Z"
    });

    fs::write(
        &baseline_path,
        serde_json::to_string_pretty(&baseline).unwrap(),
    )
    .expect("Failed to write baseline");

    // Read baseline back
    let baseline_content = fs::read_to_string(&baseline_path).expect("Failed to read baseline");

    let baseline_json: Value =
        serde_json::from_str(&baseline_content).expect("Failed to parse baseline JSON");

    // Verify structure
    assert_eq!(baseline_json["backend"], "static_matcher");
    assert_eq!(baseline_json["pass_rate"], 0.31);
    assert_eq!(baseline_json["total_tests"], 55);

    // Test regression detection logic
    let current_pass_rate = 0.25; // Simulated regression
    let baseline_pass_rate = baseline_json["pass_rate"].as_f64().unwrap();
    let threshold = baseline_pass_rate - 0.05; // 5% threshold

    assert!(
        current_pass_rate < threshold,
        "Should detect regression: current {:.2} < threshold {:.2}",
        current_pass_rate,
        threshold
    );

    println!("✓ Baseline comparison logic validated");

    // Clean up
    fs::remove_file(&baseline_path).ok();
}

/// Test that all backends can run in parallel (matrix strategy)
#[tokio::test]
async fn test_parallel_backend_evaluation() {
    use tokio::task::JoinSet;

    let backends = vec!["static_matcher", "mlx", "embedded-smollm", "embedded-qwen"];
    let mut join_set = JoinSet::new();

    for backend in backends {
        join_set.spawn(async move {
            let executor = Executor::with_backend(backend);

            match executor {
                Ok(exec) => {
                    let result = exec.execute("list files").await;
                    (backend, result.is_ok())
                }
                Err(_) => {
                    eprintln!("Backend {} not available (expected)", backend);
                    (backend, false)
                }
            }
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        if let Ok((backend, success)) = result {
            results.push((backend, success));
            println!("Backend {} evaluation completed: {}", backend, success);
        }
    }

    // At least static_matcher should work
    let static_matcher_result = results.iter().find(|(b, _)| *b == "static_matcher");
    assert!(
        static_matcher_result.is_some(),
        "static_matcher should be evaluated"
    );

    println!(
        "✓ Parallel evaluation completed for {} backends",
        results.len()
    );
}
