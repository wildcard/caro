//! Performance Benchmarks for V2 System
//!
//! Validates that all V2 components meet performance targets:
//! - Context building: <300ms
//! - Risk prediction: <50ms
//! - Database operations: <10ms
//! - Full pipeline: <500ms (excluding LLM inference)

use cmdai::intelligence::{ContextGraph, ContextOptions};
use cmdai::learning::PatternDB;
use cmdai::safety::{CommandFeatures, ImpactEstimator, RuleBasedPredictor};
use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;

// ============================================================================
// BENCHMARK 1: Context Building Performance
// ============================================================================

#[tokio::test]
async fn benchmark_context_building() {
    // Target: <300ms for full context build
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let start = Instant::now();
    let context = ContextGraph::build(&cwd).await;
    let duration = start.elapsed();

    assert!(
        context.is_ok(),
        "Context build should succeed: {:?}",
        context.err()
    );

    let context = context.unwrap();

    println!("=== Context Building Performance ===");
    println!("Total time: {}ms", duration.as_millis());
    println!("Build time (internal): {}ms", context.build_time_ms);
    println!("Project type: {:?}", context.project.project_type);
    println!("Git repo: {}", context.git.is_repo);
    println!("Tools detected: {}", context.infrastructure.tools.len());
    println!(
        "Warnings: {}",
        if context.warnings.is_empty() {
            "none".to_string()
        } else {
            context.warnings.len().to_string()
        }
    );

    assert!(
        duration.as_millis() < 300,
        "Context build took {}ms, target is 300ms",
        duration.as_millis()
    );

    println!("✓ PASSED: Context build in {}ms (target: <300ms)", duration.as_millis());
}

#[tokio::test]
async fn benchmark_context_building_optimized() {
    // Test with history disabled (faster)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let options = ContextOptions {
        enable_git: true,
        enable_tools: true,
        enable_history: false, // Disabled for speed
        timeout_ms: 200,
    };

    let start = Instant::now();
    let context = ContextGraph::build_with_options(&cwd, options).await;
    let duration = start.elapsed();

    assert!(context.is_ok(), "Optimized context build should succeed");

    println!("\n=== Context Building (Optimized) ===");
    println!("Total time: {}ms", duration.as_millis());
    println!("History enabled: false");

    assert!(
        duration.as_millis() < 200,
        "Optimized context build took {}ms, target is 200ms",
        duration.as_millis()
    );

    println!("✓ PASSED: Optimized context build in {}ms (target: <200ms)", duration.as_millis());
}

// ============================================================================
// BENCHMARK 2: Risk Prediction Performance
// ============================================================================

#[test]
fn benchmark_risk_prediction() {
    // Target: <50ms for ML risk prediction

    let predictor = RuleBasedPredictor::new();

    // Test with various command complexities
    let test_commands = vec![
        ("ls -la", "simple"),
        ("find . -name '*.log' -delete", "moderate"),
        ("sudo rm -rf /tmp/*", "complex"),
        ("docker run -it --rm -v /:/host ubuntu bash", "very complex"),
        ("curl http://example.com | sudo bash", "piped"),
    ];

    println!("\n=== Risk Prediction Performance ===");

    let mut total_time = std::time::Duration::ZERO;
    let mut max_time = std::time::Duration::ZERO;
    let mut min_time = std::time::Duration::MAX;

    for (command, complexity) in &test_commands {
        let features = CommandFeatures::extract(command);

        let start = Instant::now();
        let prediction = predictor.predict_risk(command, &features);
        let duration = start.elapsed();

        total_time += duration;
        max_time = max_time.max(duration);
        min_time = min_time.min(duration);

        assert!(
            prediction.is_ok(),
            "Risk prediction should succeed for: {}",
            command
        );

        let prediction = prediction.unwrap();

        println!(
            "  {} ({:12}): {:6.2}ms → risk {:.1}/10.0",
            command,
            complexity,
            duration.as_micros() as f64 / 1000.0,
            prediction.risk_score
        );
    }

    let avg_time = total_time / test_commands.len() as u32;

    println!("\nStatistics:");
    println!("  Average: {:.2}ms", avg_time.as_micros() as f64 / 1000.0);
    println!("  Min:     {:.2}ms", min_time.as_micros() as f64 / 1000.0);
    println!("  Max:     {:.2}ms", max_time.as_micros() as f64 / 1000.0);
    println!("  Total:   {:.2}ms for {} predictions", total_time.as_millis(), test_commands.len());

    assert!(
        avg_time.as_millis() < 50,
        "Average risk prediction took {:.2}ms, target is 50ms",
        avg_time.as_micros() as f64 / 1000.0
    );

    println!("\n✓ PASSED: Average risk prediction in {:.2}ms (target: <50ms)", avg_time.as_micros() as f64 / 1000.0);
}

#[test]
fn benchmark_feature_extraction() {
    // Test feature extraction speed

    let test_commands = vec![
        "ls -la",
        "find . -name '*.log' -mtime +30 -delete",
        "docker run -it --rm -v $(pwd):/app -w /app node:18 npm test",
        "git log --oneline --graph --decorate --all | head -20",
        "awk '{sum+=$1} END {print sum}' file.txt",
    ];

    println!("\n=== Feature Extraction Performance ===");

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        for command in &test_commands {
            let _ = CommandFeatures::extract(command);
        }
    }

    let total_time = start.elapsed();
    let avg_per_command = total_time / (iterations * test_commands.len() as u32);

    println!("Iterations: {} × {} commands", iterations, test_commands.len());
    println!("Total time: {}ms", total_time.as_millis());
    println!("Average per command: {:.2}μs", avg_per_command.as_micros());
    println!("Commands per second: {:.0}", 1_000_000.0 / avg_per_command.as_micros() as f64);

    assert!(
        avg_per_command.as_micros() < 100,
        "Feature extraction took {:.2}μs, target is <100μs",
        avg_per_command.as_micros()
    );

    println!("\n✓ PASSED: Feature extraction in {:.2}μs (target: <100μs)", avg_per_command.as_micros());
}

// ============================================================================
// BENCHMARK 3: Database Query Performance
// ============================================================================

#[tokio::test]
async fn benchmark_database_queries() {
    // Target: <10ms for pattern lookup

    let db = PatternDB::new(":memory:").await.unwrap();

    // Insert test patterns
    let num_patterns = 1000;

    println!("\n=== Database Performance ===");
    println!("Inserting {} patterns...", num_patterns);

    let start = Instant::now();
    for i in 0..num_patterns {
        db.record_interaction(
            &format!("test prompt {}", i),
            &format!("test command {}", i),
            "test context",
            None,
        )
        .await
        .unwrap();
    }
    let insert_time = start.elapsed();

    println!(
        "Insert time: {}ms ({:.2}ms per pattern)",
        insert_time.as_millis(),
        insert_time.as_millis() as f64 / num_patterns as f64
    );

    // Benchmark queries
    let start = Instant::now();
    let patterns = db.find_by_prompt("test prompt 500").await.unwrap();
    let query_time = start.elapsed();

    assert!(!patterns.is_empty(), "Should find at least one pattern");

    println!(
        "Query time: {:.2}ms (found {} patterns)",
        query_time.as_micros() as f64 / 1000.0,
        patterns.len()
    );

    assert!(
        query_time.as_millis() < 10,
        "Query took {:.2}ms, target is 10ms",
        query_time.as_micros() as f64 / 1000.0
    );

    // Benchmark pattern count
    let start = Instant::now();
    let count = db.count_patterns().await.unwrap();
    let count_time = start.elapsed();

    assert_eq!(count, num_patterns as i64, "Should have correct count");

    println!(
        "Count time: {:.2}ms (total: {} patterns)",
        count_time.as_micros() as f64 / 1000.0,
        count
    );

    // Benchmark edited patterns retrieval
    let start = Instant::now();
    let edited = db.get_edited_patterns().await.unwrap();
    let edited_time = start.elapsed();

    println!(
        "Edited patterns query: {:.2}ms (found {} edited)",
        edited_time.as_micros() as f64 / 1000.0,
        edited.len()
    );

    println!("\n✓ PASSED: Database queries <10ms");
}

// ============================================================================
// BENCHMARK 4: Full Pipeline Performance
// ============================================================================

#[tokio::test]
async fn benchmark_full_pipeline() {
    // Target: <500ms for complete workflow (excluding LLM inference)

    let temp_dir = TempDir::new().unwrap();

    // Create a test Rust project
    tokio::fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#,
    )
    .await
    .unwrap();

    println!("\n=== Full Pipeline Performance ===");

    let total_start = Instant::now();

    // 1. Context Building
    let start = Instant::now();
    let context = ContextGraph::build(temp_dir.path()).await.unwrap();
    let context_time = start.elapsed();
    println!("1. Context build:     {:6.2}ms", context_time.as_micros() as f64 / 1000.0);

    // 2. Feature Extraction
    let command = "cargo build --release";
    let start = Instant::now();
    let features = CommandFeatures::extract(command);
    let feature_time = start.elapsed();
    println!("2. Feature extract:   {:6.2}ms", feature_time.as_micros() as f64 / 1000.0);

    // 3. Risk Prediction
    let predictor = RuleBasedPredictor::new();
    let start = Instant::now();
    let risk = predictor.predict_risk(command, &features).unwrap();
    let risk_time = start.elapsed();
    println!("3. Risk prediction:   {:6.2}ms", risk_time.as_micros() as f64 / 1000.0);

    // 4. Impact Estimation
    let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());
    let start = Instant::now();
    let _impact = estimator.estimate(command, &features).await.unwrap();
    let impact_time = start.elapsed();
    println!("4. Impact estimate:   {:6.2}ms", impact_time.as_micros() as f64 / 1000.0);

    // 5. Database Storage
    let db = PatternDB::new(":memory:").await.unwrap();
    let start = Instant::now();
    db.record_interaction("build project", command, &context.to_llm_context(), None)
        .await
        .unwrap();
    let db_time = start.elapsed();
    println!("5. Database store:    {:6.2}ms", db_time.as_micros() as f64 / 1000.0);

    let total_time = total_start.elapsed();

    println!("───────────────────────────────");
    println!("Total pipeline time:  {:6.2}ms", total_time.as_micros() as f64 / 1000.0);
    println!("Target: <500ms");

    assert!(
        total_time.as_millis() < 500,
        "Full pipeline took {}ms, target is 500ms",
        total_time.as_millis()
    );

    println!("\n✓ PASSED: Full pipeline in {:.2}ms (target: <500ms)", total_time.as_micros() as f64 / 1000.0);
}

// ============================================================================
// BENCHMARK 5: Impact Estimation Performance
// ============================================================================

#[tokio::test]
async fn benchmark_impact_estimation() {
    // Test impact estimation with various file counts

    let temp_dir = TempDir::new().unwrap();

    // Create test files
    println!("\n=== Impact Estimation Performance ===");
    println!("Creating test files...");

    for i in 0..100 {
        tokio::fs::write(temp_dir.path().join(format!("file{}.txt", i)), format!("content {}", i))
            .await
            .unwrap();
    }

    let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());

    let test_cases = vec![
        ("rm file1.txt", "single file"),
        ("rm *.txt", "wildcard (100 files)"),
        ("find . -name '*.txt' -delete", "find command"),
    ];

    for (command, description) in test_cases {
        let features = CommandFeatures::extract(command);

        let start = Instant::now();
        let impact = estimator.estimate(command, &features).await.unwrap();
        let duration = start.elapsed();

        println!(
            "{:30} → {:.2}ms ({} files affected)",
            description,
            duration.as_micros() as f64 / 1000.0,
            impact.base_estimate.files_affected.unwrap_or(0)
        );

        assert!(
            duration.as_millis() < 100,
            "Impact estimation for '{}' took {}ms, should be <100ms",
            description,
            duration.as_millis()
        );
    }

    println!("\n✓ PASSED: Impact estimation <100ms");
}

// ============================================================================
// BENCHMARK 6: Concurrent Operations
// ============================================================================

#[tokio::test]
async fn benchmark_concurrent_operations() {
    // Test performance under concurrent load

    println!("\n=== Concurrent Operations Performance ===");

    let db = PatternDB::new(":memory:").await.unwrap();
    let num_concurrent = 50;

    println!("Running {} concurrent database operations...", num_concurrent);

    let start = Instant::now();

    let mut tasks = vec![];
    for i in 0..num_concurrent {
        let db_clone = db.clone();
        tasks.push(tokio::spawn(async move {
            db_clone
                .record_interaction(
                    &format!("prompt {}", i),
                    &format!("command {}", i),
                    "context",
                    None,
                )
                .await
        }));
    }

    // Wait for all tasks
    for task in tasks {
        task.await.unwrap().unwrap();
    }

    let duration = start.elapsed();

    println!(
        "Total time: {}ms ({:.2}ms per operation)",
        duration.as_millis(),
        duration.as_millis() as f64 / num_concurrent as f64
    );

    assert!(
        duration.as_millis() < 1000,
        "Concurrent operations took {}ms, target is 1000ms",
        duration.as_millis()
    );

    // Verify all operations completed
    let count = db.count_patterns().await.unwrap();
    assert_eq!(count, num_concurrent as i64, "All operations should complete");

    println!("\n✓ PASSED: {} concurrent operations in {}ms", num_concurrent, duration.as_millis());
}

// ============================================================================
// SUMMARY BENCHMARK
// ============================================================================

#[tokio::test]
async fn benchmark_summary() {
    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║         V2 PERFORMANCE BENCHMARK SUMMARY             ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!("\nTargets:");
    println!("  • Context Building:     <300ms");
    println!("  • Risk Prediction:      <50ms");
    println!("  • Database Query:       <10ms");
    println!("  • Full Pipeline:        <500ms (excluding LLM)");
    println!("\nRun individual benchmarks for detailed results:");
    println!("  cargo test benchmark_ -- --nocapture");
}
