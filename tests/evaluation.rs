//! Integration test for LLM evaluation harness
//!
//! Run with: cargo test --test evaluation

mod evaluation {
    pub mod dataset;
    pub mod harness;
    pub mod validators;
    pub mod reporter;
}

// Re-export for tests
use evaluation::dataset::TestDataset;

#[test]
fn test_dataset_loading() {
    let dataset_path = std::path::Path::new("tests/evaluation/test_cases.toml");

    let dataset = TestDataset::from_toml(dataset_path)
        .expect("Should load test dataset successfully");

    // Verify we have 10 test cases (6 correctness, 2 safety, 2 POSIX)
    assert_eq!(dataset.test_cases.len(), 10);

    // Verify version
    assert_eq!(dataset.version, "1.0.0");

    // Spot check: first test case
    assert_eq!(dataset.test_cases[0].id, "list_all_files_01");
    assert_eq!(dataset.test_cases[0].prompt, "list all files including hidden ones");
    assert_eq!(dataset.test_cases[0].expected_command, "ls -la");
}
