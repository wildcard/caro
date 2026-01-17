//! Integration tests for safety validation.
//!
//! These tests validate the end-to-end safety validation flow from
//! dataset loading through pattern matching to confusion matrix metrics.

use caro_evaluation::dataset::TestDataset;
use caro_evaluation::safety_validator::{analyze_errors, ConfusionMatrix, RiskLevel, SafetyValidator};
use std::path::Path;

#[test]
fn test_evaluate_dangerous_patterns_dataset() {
    let dataset_path = Path::new("datasets/safety/dangerous_patterns.json");

    // Skip if dataset doesn't exist
    if !dataset_path.exists() {
        eprintln!("Skipping test - dataset not found: {:?}", dataset_path);
        return;
    }

    let dataset = TestDataset::load_from_file(dataset_path)
        .expect("Failed to load dangerous_patterns dataset");

    let validator = SafetyValidator::new().expect("Failed to create validator");

    println!("\n=== Dangerous Patterns Dataset ===");
    println!("Total test cases: {}", dataset.test_cases.len());

    let mut actual_results = Vec::new();
    let mut expected_labels = Vec::new();

    for test_case in &dataset.test_cases {
        let result = validator.validate(&test_case.expected_command);
        actual_results.push(result);
        expected_labels.push(RiskLevel::from_str(&test_case.risk_level));
    }

    let matrix = ConfusionMatrix::from_results(actual_results.clone(), expected_labels);

    println!("\nConfusion Matrix:");
    println!("  True Positives:  {} (dangerous correctly flagged)", matrix.true_positives);
    println!("  False Positives: {} (safe incorrectly flagged)", matrix.false_positives);
    println!("  True Negatives:  {} (safe correctly allowed)", matrix.true_negatives);
    println!("  False Negatives: {} (dangerous missed)", matrix.false_negatives);

    println!("\nMetrics:");
    println!("  Precision: {:.2}%", matrix.precision() * 100.0);
    println!("  Recall:    {:.2}%", matrix.recall() * 100.0);
    println!("  F1-Score:  {:.3}", matrix.f1_score());
    println!("  Accuracy:  {:.2}%", matrix.accuracy() * 100.0);

    // Analyze errors
    let error_analysis = analyze_errors(&dataset.test_cases, &actual_results);

    if !error_analysis.false_positives.is_empty() {
        println!("\nFalse Positives (safe commands flagged as dangerous):");
        for (id, prompt, patterns) in &error_analysis.false_positives {
            println!("  - {}: {}", id, prompt);
            println!("    Matched patterns: {:?}", patterns);
        }
    }

    if !error_analysis.false_negatives.is_empty() {
        println!("\nFalse Negatives (dangerous commands missed):");
        for (id, prompt, command) in &error_analysis.false_negatives {
            println!("  - {}: {}", id, prompt);
            println!("    Command: {}", command);
        }
    }

    // For dangerous_patterns dataset, all commands should be detected as dangerous
    // We expect high recall (ideally 100%) since all test cases are dangerous
    assert!(
        matrix.recall() >= 0.90,
        "Recall below 90% - dangerous commands being missed: {:.2}%",
        matrix.recall() * 100.0
    );

    // For this dataset, precision might be affected if we're over-detecting
    // but we still want reasonable precision
    assert!(
        matrix.precision() >= 0.80,
        "Precision below 80% - too many false positives: {:.2}%",
        matrix.precision() * 100.0
    );
}

#[test]
fn test_evaluate_false_positives_dataset() {
    let dataset_path = Path::new("datasets/safety/false_positives.json");

    // Skip if dataset doesn't exist
    if !dataset_path.exists() {
        eprintln!("Skipping test - dataset not found: {:?}", dataset_path);
        return;
    }

    let dataset = TestDataset::load_from_file(dataset_path)
        .expect("Failed to load false_positives dataset");

    let validator = SafetyValidator::new().expect("Failed to create validator");

    println!("\n=== False Positives Dataset ===");
    println!("Total test cases: {}", dataset.test_cases.len());

    let mut actual_results = Vec::new();
    let mut expected_labels = Vec::new();

    for test_case in &dataset.test_cases {
        let result = validator.validate(&test_case.expected_command);
        actual_results.push(result);
        expected_labels.push(RiskLevel::from_str(&test_case.risk_level));
    }

    let matrix = ConfusionMatrix::from_results(actual_results.clone(), expected_labels);

    println!("\nConfusion Matrix:");
    println!("  True Positives:  {} (dangerous correctly flagged)", matrix.true_positives);
    println!("  False Positives: {} (safe incorrectly flagged)", matrix.false_positives);
    println!("  True Negatives:  {} (safe correctly allowed)", matrix.true_negatives);
    println!("  False Negatives: {} (dangerous missed)", matrix.false_negatives);

    println!("\nMetrics:");
    println!("  Precision: {:.2}%", matrix.precision() * 100.0);
    println!("  Recall:    {:.2}%", matrix.recall() * 100.0);
    println!("  F1-Score:  {:.3}", matrix.f1_score());
    println!("  Accuracy:  {:.2}%", matrix.accuracy() * 100.0);

    // Analyze errors
    let error_analysis = analyze_errors(&dataset.test_cases, &actual_results);

    if !error_analysis.false_positives.is_empty() {
        println!("\nFalse Positives (safe commands flagged as dangerous):");
        for (id, prompt, patterns) in &error_analysis.false_positives {
            println!("  - {}: {}", id, prompt);
            println!("    Matched patterns: {:?}", patterns);
        }
    }

    if !error_analysis.false_negatives.is_empty() {
        println!("\nFalse Negatives (dangerous commands missed):");
        for (id, prompt, command) in &error_analysis.false_negatives {
            println!("  - {}: {}", id, prompt);
            println!("    Command: {}", command);
        }
    }

    // For false_positives dataset, all commands are safe
    // We expect high precision (no false positives) and 100% specificity (TN rate)
    // Calculate specificity: TN / (TN + FP)
    let total_negatives = matrix.true_negatives + matrix.false_positives;
    let specificity = if total_negatives > 0 {
        matrix.true_negatives as f64 / total_negatives as f64
    } else {
        0.0
    };

    println!("\n  Specificity (TN rate): {:.2}%", specificity * 100.0);

    // We want high specificity - at least 85% of safe commands should be correctly allowed
    assert!(
        specificity >= 0.85,
        "Specificity below 85% - too many safe commands flagged: {:.2}%",
        specificity * 100.0
    );

    // False positive rate should be low (< 15%)
    let false_positive_rate = if total_negatives > 0 {
        matrix.false_positives as f64 / total_negatives as f64
    } else {
        0.0
    };

    println!("  False Positive Rate: {:.2}%", false_positive_rate * 100.0);

    assert!(
        false_positive_rate <= 0.15,
        "False positive rate above 15%: {:.2}%",
        false_positive_rate * 100.0
    );
}

#[test]
fn test_combined_safety_evaluation() {
    let dangerous_path = Path::new("datasets/safety/dangerous_patterns.json");
    let false_positives_path = Path::new("datasets/safety/false_positives.json");

    // Skip if datasets don't exist
    if !dangerous_path.exists() || !false_positives_path.exists() {
        eprintln!("Skipping test - datasets not found");
        return;
    }

    let dangerous_dataset = TestDataset::load_from_file(dangerous_path)
        .expect("Failed to load dangerous_patterns dataset");
    let false_positives_dataset = TestDataset::load_from_file(false_positives_path)
        .expect("Failed to load false_positives dataset");

    let validator = SafetyValidator::new().expect("Failed to create validator");

    println!("\n=== Combined Safety Evaluation ===");
    println!("Dangerous commands: {}", dangerous_dataset.test_cases.len());
    println!("Safe commands: {}", false_positives_dataset.test_cases.len());

    // Combine both datasets
    let mut all_test_cases = Vec::new();
    let mut actual_results = Vec::new();
    let mut expected_labels = Vec::new();

    // Add dangerous commands
    for test_case in &dangerous_dataset.test_cases {
        let result = validator.validate(&test_case.expected_command);
        all_test_cases.push(test_case.clone());
        actual_results.push(result);
        expected_labels.push(RiskLevel::from_str(&test_case.risk_level));
    }

    // Add safe commands
    for test_case in &false_positives_dataset.test_cases {
        let result = validator.validate(&test_case.expected_command);
        all_test_cases.push(test_case.clone());
        actual_results.push(result);
        expected_labels.push(RiskLevel::from_str(&test_case.risk_level));
    }

    let matrix = ConfusionMatrix::from_results(actual_results.clone(), expected_labels);

    println!("\nCombined Confusion Matrix:");
    println!("  True Positives:  {} (dangerous correctly flagged)", matrix.true_positives);
    println!("  False Positives: {} (safe incorrectly flagged)", matrix.false_positives);
    println!("  True Negatives:  {} (safe correctly allowed)", matrix.true_negatives);
    println!("  False Negatives: {} (dangerous missed)", matrix.false_negatives);

    println!("\nCombined Metrics:");
    println!("  Precision: {:.2}%", matrix.precision() * 100.0);
    println!("  Recall:    {:.2}%", matrix.recall() * 100.0);
    println!("  F1-Score:  {:.3}", matrix.f1_score());
    println!("  Accuracy:  {:.2}%", matrix.accuracy() * 100.0);

    // Analyze errors
    let error_analysis = analyze_errors(&all_test_cases, &actual_results);

    println!("\nError Summary:");
    println!("  False Positives: {}", error_analysis.false_positives.len());
    println!("  False Negatives: {}", error_analysis.false_negatives.len());

    if !error_analysis.false_positives.is_empty() {
        println!("\nFalse Positives (safe commands flagged as dangerous):");
        for (id, prompt, patterns) in error_analysis.false_positives.iter().take(5) {
            println!("  - {}: {}", id, prompt);
            println!("    Matched patterns: {:?}", patterns);
        }
        if error_analysis.false_positives.len() > 5 {
            println!("  ... and {} more", error_analysis.false_positives.len() - 5);
        }
    }

    if !error_analysis.false_negatives.is_empty() {
        println!("\nFalse Negatives (dangerous commands missed):");
        for (id, prompt, command) in error_analysis.false_negatives.iter().take(5) {
            println!("  - {}: {}", id, prompt);
            println!("    Command: {}", command);
        }
        if error_analysis.false_negatives.len() > 5 {
            println!("  ... and {} more", error_analysis.false_negatives.len() - 5);
        }
    }

    // Success criteria from spec.md (SC-002):
    // - Precision >= 85%
    // - Recall >= 90%
    assert!(
        matrix.precision() >= 0.85,
        "Precision below 85% (SC-002): {:.2}%",
        matrix.precision() * 100.0
    );

    assert!(
        matrix.recall() >= 0.90,
        "Recall below 90% (SC-002): {:.2}%",
        matrix.recall() * 100.0
    );

    println!("\nSafety validation meets success criteria (SC-002)");
    println!("  Precision: {:.2}% (target: >=85%)", matrix.precision() * 100.0);
    println!("  Recall: {:.2}% (target: >=90%)", matrix.recall() * 100.0);
}
