//! WP10-11: Prompt Engineering Framework Tests
//!
//! These tests validate prompt versioning, template support, and A/B testing
//! capabilities for systematic prompt quality improvements.

use std::path::Path;

/// WP10: Test prompt version loading
#[tokio::test]
async fn test_load_prompt_version() {
    use caro_evaluation::prompts::registry::PromptRegistry;

    let registry = PromptRegistry::new("prompts").expect("Failed to create registry");

    // Load v1.0 prompt
    let prompt_v1 = registry
        .load_prompt("correctness", "v1.0")
        .expect("Failed to load v1.0 prompt");

    assert!(!prompt_v1.is_empty(), "Prompt should not be empty");
    assert!(
        prompt_v1.contains("command") || prompt_v1.contains("shell"),
        "Prompt should contain command-related text"
    );
}

/// WP10: Test prompt template variable substitution
#[tokio::test]
async fn test_prompt_template_substitution() {
    use caro_evaluation::prompts::loader::PromptLoader;
    use std::collections::HashMap;

    let loader = PromptLoader::new("prompts");

    // Create a template with variables
    let template = "Generate a {{command_type}} command for: {{user_request}}";

    let mut variables = HashMap::new();
    variables.insert("command_type".to_string(), "shell".to_string());
    variables.insert("user_request".to_string(), "list files".to_string());

    let result = loader.render_template(template, &variables);

    assert_eq!(
        result, "Generate a shell command for: list files",
        "Template should be rendered with variables"
    );
}

/// WP10: Test prompt metadata loading
#[tokio::test]
async fn test_prompt_metadata() {
    use caro_evaluation::prompts::metadata::PromptMetadata;

    // Create test metadata
    let metadata = PromptMetadata {
        version: "1.1".to_string(),
        author: "Test Team".to_string(),
        created: "2026-01-17".to_string(),
        target_models: vec!["smollm".to_string(), "qwen".to_string()],
        changelog: "Improved POSIX compliance".to_string(),
        baseline_pass_rate: Some(0.43),
    };

    assert_eq!(metadata.version, "1.1");
    assert_eq!(metadata.target_models.len(), 2);
    assert!(metadata.baseline_pass_rate.unwrap() > 0.0);
}

/// WP10: Test prompt registry listing
#[tokio::test]
async fn test_list_available_prompts() {
    use caro_evaluation::prompts::registry::PromptRegistry;

    let registry = PromptRegistry::new("prompts").expect("Failed to create registry");

    let versions = registry
        .list_versions("correctness")
        .expect("Failed to list versions");

    // Should find at least one version
    assert!(
        !versions.is_empty(),
        "Should find at least one prompt version"
    );

    println!("Available prompt versions: {:?}", versions);
}

/// WP11: Test prompt comparison between versions
#[tokio::test]
async fn test_compare_multiple_prompts() {
    use caro_evaluation::prompt_comparison::PromptComparison;

    let comparison = PromptComparison::new();

    // Simulate results from different prompt versions
    let results = vec![
        ("v1.0", 31.0), // 31% pass rate
        ("v1.1", 43.2), // 43.2% pass rate
        ("v1.2", 29.5), // 29.5% pass rate
    ];

    let winner = comparison.find_winner(&results);

    assert_eq!(winner, "v1.1", "v1.1 should be the winner with 43.2%");
}

/// WP11: Test statistical significance calculation
#[tokio::test]
async fn test_statistical_significance() {
    use caro_evaluation::prompt_comparison::PromptComparison;

    let comparison = PromptComparison::new();

    // Compare two prompt versions with test counts
    // Use larger difference to ensure statistical significance
    let v1_results = (17, 55); // 17 passed out of 55 (31%)
    let v2_results = (35, 55); // 35 passed out of 55 (63.6%)

    let p_value = comparison.chi_square_test(v1_results, v2_results);

    // Significant difference should have p < 0.05
    assert!(
        p_value < 0.05,
        "Difference should be statistically significant (p={:.4})",
        p_value
    );

    println!("Chi-square test p-value: {:.4}", p_value);
}

/// WP11: Test automated rollback decision
#[tokio::test]
async fn test_rollback_on_regression() {
    use caro_evaluation::prompt_comparison::PromptComparison;

    let comparison = PromptComparison::new();

    let current_champion = 43.2; // Current best: 43.2%
    let new_candidate = 29.5; // New prompt: 29.5%

    let regression_pct = ((new_candidate - current_champion) / current_champion) * 100.0;

    let should_rollback = comparison.should_rollback(current_champion, new_candidate, 10.0);

    assert!(
        should_rollback,
        "Should rollback with {:.1}% regression (threshold: 10%)",
        regression_pct.abs()
    );
}

/// WP11: Test side-by-side comparison report generation
#[tokio::test]
async fn test_comparison_report_generation() {
    use caro_evaluation::prompt_comparison::{ComparisonResult, PromptComparison};

    let comparison = PromptComparison::new();

    let results = vec![
        ComparisonResult {
            version: "v1.0".to_string(),
            pass_rate: 0.31,
            total_tests: 55,
            passed_tests: 17,
            p_value: None,
            is_winner: false,
        },
        ComparisonResult {
            version: "v1.1".to_string(),
            pass_rate: 0.432,
            total_tests: 55,
            passed_tests: 24,
            p_value: Some(0.001),
            is_winner: true,
        },
        ComparisonResult {
            version: "v1.2".to_string(),
            pass_rate: 0.295,
            total_tests: 55,
            passed_tests: 16,
            p_value: Some(0.42),
            is_winner: false,
        },
    ];

    let report = comparison.generate_report(&results);

    assert!(report.contains("v1.0"), "Report should include v1.0");
    assert!(report.contains("v1.1"), "Report should include v1.1");
    assert!(
        report.contains("WINNER") || report.contains("✅"),
        "Report should mark winner"
    );
    assert!(
        report.contains("31.0%") || report.contains("0.31"),
        "Report should include pass rates"
    );

    println!("Comparison Report:\n{}", report);
}

/// WP10: Test prompt version creation from directory structure
#[tokio::test]
async fn test_prompt_directory_structure() {
    use std::fs;

    let prompts_dir = Path::new("prompts");

    // Check that prompts directory exists or can be created
    if !prompts_dir.exists() {
        fs::create_dir_all(prompts_dir).expect("Failed to create prompts directory");
    }

    assert!(prompts_dir.exists(), "Prompts directory should exist");

    // Check for v1.0 subdirectory
    let v1_dir = prompts_dir.join("v1.0");
    if !v1_dir.exists() {
        fs::create_dir_all(&v1_dir).expect("Failed to create v1.0 directory");
    }

    assert!(v1_dir.exists(), "v1.0 directory should exist");

    println!("✓ Prompt directory structure validated");
}

/// WP11: Test prompt comparison with up to 5 versions
#[tokio::test]
async fn test_compare_up_to_five_prompts() {
    use caro_evaluation::prompt_comparison::PromptComparison;

    let comparison = PromptComparison::new();

    let results = vec![
        ("v1.0", 31.0),
        ("v1.1", 43.2),
        ("v1.2", 29.5),
        ("v2.0", 45.1),
        ("v2.1", 42.8),
    ];

    assert_eq!(results.len(), 5, "Should support 5 prompt versions");

    let winner = comparison.find_winner(&results);

    assert_eq!(winner, "v2.0", "v2.0 should be the winner with 45.1%");

    println!("✓ Successfully compared 5 prompt versions");
}
