//! WP12-13: Model-Specific Intelligence & Capability Matrix Tests
//!
//! These tests validate model profiling, capability matrix generation,
//! and model recommendation systems for optimal backend selection.

/// WP12: Test model profile data structure
#[tokio::test]
async fn test_model_profile_creation() {
    use caro_evaluation::model_profiling::{CategoryPerformance, ModelProfile};

    let profile = ModelProfile {
        model_name: "smollm-135m".to_string(),
        total_tests: 55,
        passed_tests: 25,
        overall_pass_rate: 0.45,
        category_performance: vec![
            CategoryPerformance {
                category: "file_operations".to_string(),
                passed: 17,
                total: 20,
                pass_rate: 0.85,
            },
            CategoryPerformance {
                category: "regex_patterns".to_string(),
                passed: 3,
                total: 20,
                pass_rate: 0.15,
            },
        ],
        strengths: vec!["file_operations".to_string(), "simple_commands".to_string()],
        weaknesses: vec!["regex_patterns".to_string(), "complex_pipes".to_string()],
    };

    assert_eq!(profile.model_name, "smollm-135m");
    assert_eq!(profile.overall_pass_rate, 0.45);
    assert_eq!(profile.category_performance.len(), 2);
    assert_eq!(profile.strengths.len(), 2);
    assert_eq!(profile.weaknesses.len(), 2);
}

/// WP12: Test failure pattern extraction
#[tokio::test]
async fn test_extract_failure_patterns() {
    use caro_evaluation::model_profiling::ModelProfiler;
    use caro_evaluation::test_runner::TestResult;

    let profiler = ModelProfiler::new();

    let test_results = vec![
        TestResult {
            test_id: "test_001".to_string(),
            backend: "smollm".to_string(),
            category: "regex".to_string(),
            passed: false,
            actual_output: "grep 'pattern'".to_string(),
            expected_output: "grep -E 'pattern'".to_string(),
            failure_reason: Some("Missing -E flag for extended regex".to_string()),
        },
        TestResult {
            test_id: "test_002".to_string(),
            backend: "smollm".to_string(),
            category: "regex".to_string(),
            passed: false,
            actual_output: "grep 'test|prod'".to_string(),
            expected_output: "grep -E 'test|prod'".to_string(),
            failure_reason: Some("Missing -E flag for extended regex".to_string()),
        },
    ];

    let patterns = profiler.extract_failure_patterns(&test_results);

    assert!(!patterns.is_empty(), "Should extract failure patterns");
    assert!(
        patterns.iter().any(|p| p.contains("Missing -E flag")),
        "Should identify -E flag pattern"
    );
}

/// WP12: Test model profile report generation
#[tokio::test]
async fn test_generate_profile_report() {
    use caro_evaluation::model_profiling::{CategoryPerformance, ModelProfile, ModelProfiler};

    let profiler = ModelProfiler::new();

    let profile = ModelProfile {
        model_name: "smollm-135m".to_string(),
        total_tests: 55,
        passed_tests: 25,
        overall_pass_rate: 0.45,
        category_performance: vec![CategoryPerformance {
            category: "file_operations".to_string(),
            passed: 17,
            total: 20,
            pass_rate: 0.85,
        }],
        strengths: vec!["file_operations".to_string()],
        weaknesses: vec!["regex_patterns".to_string()],
    };

    let report = profiler.generate_report(&profile);

    assert!(
        report.contains("smollm-135m"),
        "Report should include model name"
    );
    assert!(report.contains("45"), "Report should include pass rate");
    assert!(
        report.contains("Strengths"),
        "Report should have strengths section"
    );
    assert!(
        report.contains("Weaknesses"),
        "Report should have weaknesses section"
    );
    assert!(
        report.contains("file_operations"),
        "Report should list strong categories"
    );
}

/// WP12: Test model profiler with real test results
#[tokio::test]
async fn test_model_profiler_from_results() {
    use caro_evaluation::model_profiling::ModelProfiler;
    use caro_evaluation::test_runner::TestResult;

    let profiler = ModelProfiler::new();

    let results = vec![
        TestResult {
            test_id: "test_001".to_string(),
            backend: "smollm".to_string(),
            category: "file_ops".to_string(),
            passed: true,
            actual_output: "find . -type f".to_string(),
            expected_output: "find . -type f".to_string(),
            failure_reason: None,
        },
        TestResult {
            test_id: "test_002".to_string(),
            backend: "smollm".to_string(),
            category: "file_ops".to_string(),
            passed: true,
            actual_output: "ls -la".to_string(),
            expected_output: "ls -la".to_string(),
            failure_reason: None,
        },
        TestResult {
            test_id: "test_003".to_string(),
            backend: "smollm".to_string(),
            category: "regex".to_string(),
            passed: false,
            actual_output: "grep 'test'".to_string(),
            expected_output: "grep -E 'test|prod'".to_string(),
            failure_reason: Some("Wrong regex syntax".to_string()),
        },
    ];

    let profile = profiler.build_profile("smollm", &results);

    assert_eq!(profile.model_name, "smollm");
    assert_eq!(profile.total_tests, 3);
    assert_eq!(profile.passed_tests, 2);
    assert!((profile.overall_pass_rate - 0.667).abs() < 0.01);
}

/// WP12: Test model recommendation system
#[tokio::test]
async fn test_model_recommendation() {
    use caro_evaluation::model_profiling::{CategoryPerformance, ModelProfile, ModelProfiler};

    let profiler = ModelProfiler::new();

    let profiles = vec![
        ModelProfile {
            model_name: "smollm".to_string(),
            total_tests: 55,
            passed_tests: 25,
            overall_pass_rate: 0.45,
            category_performance: vec![CategoryPerformance {
                category: "file_operations".to_string(),
                passed: 17,
                total: 20,
                pass_rate: 0.85,
            }],
            strengths: vec!["file_operations".to_string()],
            weaknesses: vec!["regex".to_string()],
        },
        ModelProfile {
            model_name: "qwen".to_string(),
            total_tests: 55,
            passed_tests: 34,
            overall_pass_rate: 0.62,
            category_performance: vec![CategoryPerformance {
                category: "regex".to_string(),
                passed: 15,
                total: 20,
                pass_rate: 0.75,
            }],
            strengths: vec!["regex".to_string()],
            weaknesses: vec![],
        },
    ];

    let recommendation = profiler.recommend_model(&profiles, "file_operations");

    assert_eq!(
        recommendation, "smollm",
        "Should recommend smollm for file operations (85% vs lower)"
    );
}

/// WP13: Test capability matrix creation
#[tokio::test]
async fn test_capability_matrix_creation() {
    use caro_evaluation::capability_matrix::{CapabilityMatrix, ModelCapability};

    let matrix = CapabilityMatrix {
        categories: vec![
            "correctness".to_string(),
            "safety".to_string(),
            "posix".to_string(),
        ],
        models: vec![
            ModelCapability {
                model_name: "smollm".to_string(),
                capabilities: vec![
                    ("correctness".to_string(), 0.45),
                    ("safety".to_string(), 0.85),
                    ("posix".to_string(), 0.60),
                ],
            },
            ModelCapability {
                model_name: "qwen".to_string(),
                capabilities: vec![
                    ("correctness".to_string(), 0.62),
                    ("safety".to_string(), 0.78),
                    ("posix".to_string(), 0.75),
                ],
            },
        ],
    };

    assert_eq!(matrix.categories.len(), 3);
    assert_eq!(matrix.models.len(), 2);

    // Verify smollm capabilities
    let smollm = &matrix.models[0];
    assert_eq!(smollm.model_name, "smollm");
    assert_eq!(smollm.capabilities.len(), 3);
    assert_eq!(smollm.capabilities[1].1, 0.85); // Safety
}

/// WP13: Test capability matrix JSON serialization
#[tokio::test]
async fn test_capability_matrix_serialization() {
    use caro_evaluation::capability_matrix::{CapabilityMatrix, ModelCapability};

    let matrix = CapabilityMatrix {
        categories: vec!["correctness".to_string()],
        models: vec![ModelCapability {
            model_name: "smollm".to_string(),
            capabilities: vec![("correctness".to_string(), 0.45)],
        }],
    };

    let json = serde_json::to_string(&matrix).expect("Should serialize to JSON");

    assert!(json.contains("smollm"));
    assert!(json.contains("correctness"));
    assert!(json.contains("0.45"));

    // Test deserialization
    let deserialized: CapabilityMatrix =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(deserialized.categories.len(), 1);
    assert_eq!(deserialized.models[0].model_name, "smollm");
}

/// WP13: Test capability matrix report generation
#[tokio::test]
async fn test_capability_matrix_report() {
    use caro_evaluation::capability_matrix::{CapabilityMatrix, CapabilityMatrixBuilder};

    let builder = CapabilityMatrixBuilder::new();

    let matrix = CapabilityMatrix {
        categories: vec![
            "correctness".to_string(),
            "safety".to_string(),
            "posix".to_string(),
        ],
        models: vec![
            caro_evaluation::capability_matrix::ModelCapability {
                model_name: "smollm".to_string(),
                capabilities: vec![
                    ("correctness".to_string(), 0.45),
                    ("safety".to_string(), 0.85),
                    ("posix".to_string(), 0.60),
                ],
            },
            caro_evaluation::capability_matrix::ModelCapability {
                model_name: "qwen".to_string(),
                capabilities: vec![
                    ("correctness".to_string(), 0.62),
                    ("safety".to_string(), 0.78),
                    ("posix".to_string(), 0.75),
                ],
            },
        ],
    };

    let report = builder.generate_table_report(&matrix);

    assert!(report.contains("Model"), "Should have header row");
    assert!(report.contains("smollm"), "Should include smollm");
    assert!(report.contains("qwen"), "Should include qwen");
    assert!(report.contains("correctness"), "Should include categories");
    assert!(
        report.contains("45%") || report.contains("0.45"),
        "Should show percentages"
    );
}

/// WP13: Test finding best model for category
#[tokio::test]
async fn test_find_best_model_for_category() {
    use caro_evaluation::capability_matrix::{CapabilityMatrix, CapabilityMatrixBuilder};

    let builder = CapabilityMatrixBuilder::new();

    let matrix = CapabilityMatrix {
        categories: vec!["safety".to_string(), "correctness".to_string()],
        models: vec![
            caro_evaluation::capability_matrix::ModelCapability {
                model_name: "smollm".to_string(),
                capabilities: vec![
                    ("safety".to_string(), 0.85),
                    ("correctness".to_string(), 0.45),
                ],
            },
            caro_evaluation::capability_matrix::ModelCapability {
                model_name: "qwen".to_string(),
                capabilities: vec![
                    ("safety".to_string(), 0.78),
                    ("correctness".to_string(), 0.62),
                ],
            },
        ],
    };

    let best_for_safety = builder.find_best_model(&matrix, "safety");
    assert_eq!(best_for_safety, Some("smollm".to_string()));

    let best_for_correctness = builder.find_best_model(&matrix, "correctness");
    assert_eq!(best_for_correctness, Some("qwen".to_string()));
}

/// WP13: Test capability matrix from test results
#[tokio::test]
async fn test_build_capability_matrix_from_results() {
    use caro_evaluation::capability_matrix::CapabilityMatrixBuilder;
    use caro_evaluation::test_runner::TestResult;

    let builder = CapabilityMatrixBuilder::new();

    let results = vec![
        TestResult {
            test_id: "test_001".to_string(),
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            passed: true,
            actual_output: "ls".to_string(),
            expected_output: "ls".to_string(),
            failure_reason: None,
        },
        TestResult {
            test_id: "test_002".to_string(),
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            passed: false,
            actual_output: "find".to_string(),
            expected_output: "find .".to_string(),
            failure_reason: Some("Missing argument".to_string()),
        },
        TestResult {
            test_id: "test_003".to_string(),
            backend: "qwen".to_string(),
            category: "correctness".to_string(),
            passed: true,
            actual_output: "ls".to_string(),
            expected_output: "ls".to_string(),
            failure_reason: None,
        },
        TestResult {
            test_id: "test_004".to_string(),
            backend: "qwen".to_string(),
            category: "correctness".to_string(),
            passed: true,
            actual_output: "find .".to_string(),
            expected_output: "find .".to_string(),
            failure_reason: None,
        },
    ];

    let matrix = builder.build_from_results(&results);

    assert_eq!(matrix.categories.len(), 1);
    assert_eq!(matrix.categories[0], "correctness");
    assert_eq!(matrix.models.len(), 2);

    // Verify smollm: 1/2 = 50%
    let smollm = matrix
        .models
        .iter()
        .find(|m| m.model_name == "smollm")
        .unwrap();
    assert_eq!(smollm.capabilities[0].1, 0.5);

    // Verify qwen: 2/2 = 100%
    let qwen = matrix
        .models
        .iter()
        .find(|m| m.model_name == "qwen")
        .unwrap();
    assert_eq!(qwen.capabilities[0].1, 1.0);
}
