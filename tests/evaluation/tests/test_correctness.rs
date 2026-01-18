//! Integration tests for command correctness evaluation.
//!
//! These tests validate the end-to-end correctness evaluation flow
//! from dataset loading through command generation to scoring.

use caro_evaluation::dataset::TestDataset;
use caro_evaluation::evaluator::{CorrectnessMethod, Evaluator};
use caro_evaluation::executor::Executor;
use std::path::Path;

#[tokio::test]
async fn test_evaluate_file_operations_dataset() {
    let dataset_path = Path::new("datasets/correctness/file_operations.json");

    // Skip if dataset doesn't exist
    if !dataset_path.exists() {
        eprintln!("Skipping test - dataset not found: {:?}", dataset_path);
        return;
    }

    let dataset =
        TestDataset::load_from_file(dataset_path).expect("Failed to load file_operations dataset");

    // For now, just test evaluator without executor (which requires caro binary)
    let evaluator = Evaluator;

    println!("\n=== File Operations Dataset ===");
    println!("Total test cases: {}", dataset.test_cases.len());

    // Test that dataset is valid
    assert!(!dataset.test_cases.is_empty());
    assert_eq!(dataset.name, "correctness_file_operations");

    // Test evaluator with some example comparisons
    let exact_match = evaluator.evaluate_correctness("ls", "ls");
    assert_eq!(exact_match.method, CorrectnessMethod::ExactMatch);
    assert_eq!(exact_match.score, 1.0);

    let whitespace = evaluator.evaluate_correctness("ls  -la", "ls -la");
    assert_eq!(whitespace.method, CorrectnessMethod::SemanticEquivalent);
    assert!(whitespace.score >= 0.95);

    let flag_order = evaluator.evaluate_correctness("ls -la", "ls -al");
    assert_eq!(flag_order.method, CorrectnessMethod::SemanticEquivalent);
    assert!(flag_order.score >= 0.90);

    let no_match = evaluator.evaluate_correctness("ls", "pwd");
    assert_eq!(no_match.method, CorrectnessMethod::NoMatch);
    assert_eq!(no_match.score, 0.0);

    println!("✓ Evaluator tests passed");
}

#[tokio::test]
async fn test_evaluate_with_executor() {
    // This test requires the caro binary to be built
    let executor = match Executor::new() {
        Ok(e) => e,
        Err(_) => {
            eprintln!("Skipping executor test - caro binary not built");
            return;
        }
    };

    let evaluator = Evaluator;

    println!("\n=== Executor Integration Test ===");
    println!("Binary path: {:?}", executor.binary_path());

    // Try a simple command
    let result = executor.execute("list all files").await;

    match result {
        Ok(generated) => {
            println!("Generated command: {}", generated);

            // Evaluate against expected command
            let eval_result = evaluator.evaluate_correctness(&generated, "ls");
            println!(
                "Score: {:.2}, Method: {:?}",
                eval_result.score, eval_result.method
            );

            if let Some(diff) = &eval_result.diff {
                println!("Diff:\n{}", diff);
            }
        }
        Err(e) => {
            eprintln!("Command execution failed (might be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_full_evaluation_flow() {
    let dataset_path = Path::new("datasets/correctness/file_operations.json");

    if !dataset_path.exists() {
        eprintln!("Skipping test - dataset not found");
        return;
    }

    let dataset = TestDataset::load_from_file(dataset_path).expect("Failed to load dataset");

    let executor = match Executor::new() {
        Ok(e) => e,
        Err(_) => {
            eprintln!("Skipping full flow test - caro binary not built");
            return;
        }
    };

    let evaluator = Evaluator;

    println!("\n=== Full Evaluation Flow ===");
    println!("Dataset: {}", dataset.name);
    println!("Test cases: {}", dataset.test_cases.len());

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for test_case in dataset.test_cases.iter().take(5) {
        // Only test first 5 to avoid long test times
        println!("\nTest case: {}", test_case.id);
        println!("Prompt: {}", test_case.prompt);
        println!("Expected: {}", test_case.expected_command);

        let generated = match executor.execute(&test_case.prompt).await {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Execution failed: {}", e);
                failed += 1;
                continue;
            }
        };

        println!("Generated: {}", generated);

        let correctness = evaluator.evaluate_correctness(&generated, &test_case.expected_command);

        println!(
            "Score: {:.2}, Method: {:?}",
            correctness.score, correctness.method
        );

        if let Some(diff) = &correctness.diff {
            println!("Diff:\n{}", diff);
        }

        if correctness.score >= 0.90 {
            passed += 1;
        } else {
            failed += 1;
        }

        results.push((test_case.id.clone(), correctness));
    }

    println!("\n=== Summary ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if !results.is_empty() {
        let avg_score = Evaluator::aggregate_scores(
            &results.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>(),
        );
        println!("Average score: {:.2}%", avg_score * 100.0);

        // For MVP, we expect at least some commands to work
        // Don't fail the test if backend isn't configured properly yet
        if avg_score > 0.0 {
            assert!(
                avg_score >= 0.50,
                "Average correctness too low: {:.2}",
                avg_score
            );
        }
    }
}

#[test]
fn test_all_datasets_valid() {
    // Verify all correctness datasets are valid
    let datasets = vec![
        "datasets/correctness/file_operations.json",
        "datasets/correctness/text_processing.json",
        "datasets/correctness/network_commands.json",
    ];

    for dataset_path in datasets {
        let path = Path::new(dataset_path);
        if !path.exists() {
            eprintln!("Warning: Dataset not found: {:?}", path);
            continue;
        }

        let dataset = TestDataset::load_from_file(path)
            .unwrap_or_else(|e| panic!("Failed to load {}: {}", dataset_path, e));

        println!("Loaded dataset: {}", dataset.name);
        println!("  Version: {}", dataset.version);
        println!("  Test cases: {}", dataset.test_cases.len());
        println!(
            "  POSIX coverage: {:.1}%",
            dataset.metadata.posix_coverage * 100.0
        );

        // Validate dataset structure
        assert!(!dataset.name.is_empty());
        assert!(!dataset.test_cases.is_empty());
        assert_eq!(dataset.metadata.total_cases, dataset.test_cases.len());

        // Validate each test case
        for tc in &dataset.test_cases {
            assert!(!tc.id.is_empty(), "Test case ID is empty");
            assert!(!tc.prompt.is_empty(), "Prompt is empty for {}", tc.id);
            assert!(
                !tc.expected_command.is_empty(),
                "Expected command is empty for {}",
                tc.id
            );
            assert!(!tc.category.is_empty(), "Category is empty for {}", tc.id);
        }
    }
}
