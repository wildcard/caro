//! WP20-21: Test Coverage & Quality Tests
//!
//! These tests validate automated test generation and quality metrics
//! for maximizing test coverage and identifying gaps.

/// WP20: Test telemetry-to-test-case generation
#[tokio::test]
async fn test_generate_test_from_telemetry() {
    use caro_evaluation::test_generator::{TelemetryQuery, TestGenerator};
    use chrono::Utc;

    let telemetry = TelemetryQuery {
        query: "find all Python files".to_string(),
        generated_command: "find . -name '*.py'".to_string(),
        user_executed: true,
        timestamp: Utc::now(),
    };

    let generator = TestGenerator::new();
    let test_case = generator.generate_from_telemetry(&telemetry);

    assert!(
        test_case.is_ok(),
        "Should generate test case from telemetry"
    );
    let test = test_case.unwrap();
    assert_eq!(test.expected_command, "find . -name '*.py'");
    assert!(test.id.starts_with("telemetry_"));
}

/// WP20: Test telemetry categorization
#[tokio::test]
async fn test_categorize_telemetry_query() {
    use caro_evaluation::test_generator::TestGenerator;

    let generator = TestGenerator::new();

    let category = generator.categorize_query("find files modified today");
    assert_eq!(category, "correctness");

    let category = generator.categorize_query("delete everything");
    assert_eq!(category, "safety");
}

/// WP20: Test edge case generation
#[tokio::test]
async fn test_generate_edge_cases() {
    use caro_evaluation::test_generator::EdgeCaseFuzzer;

    let fuzzer = EdgeCaseFuzzer::new();
    let edge_cases = fuzzer.generate_edge_cases(5);

    assert_eq!(edge_cases.len(), 5, "Should generate 5 edge cases");

    // Check that edge cases include special scenarios
    let has_special_char = edge_cases
        .iter()
        .any(|tc| tc.prompt.contains("special") || tc.prompt.contains("space"));

    assert!(
        has_special_char,
        "Edge cases should include special characters or spaces"
    );
}

/// WP20: Test batch test generation
#[tokio::test]
async fn test_batch_test_generation() {
    use caro_evaluation::test_generator::{TelemetryQuery, TestGenerator};
    use chrono::Utc;

    let telemetry_batch = vec![
        TelemetryQuery {
            query: "list files".to_string(),
            generated_command: "ls -la".to_string(),
            user_executed: true,
            timestamp: Utc::now(),
        },
        TelemetryQuery {
            query: "find logs".to_string(),
            generated_command: "find . -name '*.log'".to_string(),
            user_executed: true,
            timestamp: Utc::now(),
        },
    ];

    let generator = TestGenerator::new();
    let test_cases = generator.batch_generate(&telemetry_batch);

    assert_eq!(test_cases.len(), 2, "Should generate 2 test cases");
}

/// WP20: Test deduplication during generation
#[tokio::test]
async fn test_deduplicate_generated_tests() {
    use caro_evaluation::test_generator::{TelemetryQuery, TestGenerator};
    use chrono::Utc;

    let telemetry_batch = vec![
        TelemetryQuery {
            query: "list files".to_string(),
            generated_command: "ls -la".to_string(),
            user_executed: true,
            timestamp: Utc::now(),
        },
        TelemetryQuery {
            query: "list files".to_string(), // Duplicate query
            generated_command: "ls -la".to_string(),
            user_executed: true,
            timestamp: Utc::now(),
        },
    ];

    let generator = TestGenerator::new();
    let test_cases = generator.batch_generate_deduplicated(&telemetry_batch);

    assert_eq!(test_cases.len(), 1, "Should deduplicate identical queries");
}

/// WP21: Test difficulty level calculation
#[tokio::test]
async fn test_calculate_difficulty_level() {
    use caro_evaluation::quality_metrics::{DifficultyAnalyzer, DifficultyLevel};
    use caro_evaluation::test_runner::TestResult;

    let analyzer = DifficultyAnalyzer::new();

    // Easy test: 90% pass rate
    let easy_results = vec![
        TestResult::pass("test_001", "smollm", "correctness", "ls"),
        TestResult::pass("test_001", "qwen", "correctness", "ls"),
        TestResult::pass("test_001", "llama", "correctness", "ls"),
        TestResult::pass("test_001", "gemma", "correctness", "ls"),
        TestResult::pass("test_001", "phi", "correctness", "ls"),
        TestResult::pass("test_001", "mistral", "correctness", "ls"),
        TestResult::pass("test_001", "codellama", "correctness", "ls"),
        TestResult::pass("test_001", "deepseek", "correctness", "ls"),
        TestResult::pass("test_001", "stablelm", "correctness", "ls"),
        TestResult::fail(
            "test_001",
            "tinyllama",
            "correctness",
            "ll",
            "ls",
            "Typo",
        ),
    ];

    let difficulty = analyzer.calculate_difficulty(&easy_results);
    assert_eq!(difficulty, DifficultyLevel::Easy);
}

/// WP21: Test redundancy detection
#[tokio::test]
async fn test_detect_redundant_tests() {
    use caro_evaluation::dataset::TestCase;
    use caro_evaluation::quality_metrics::RedundancyDetector;

    let detector = RedundancyDetector::new();

    let test_cases = vec![
        TestCase {
            id: "test_001".to_string(),
            prompt: "find Python files".to_string(),
            expected_command: "find . -name '*.py'".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
        TestCase {
            id: "test_002".to_string(),
            prompt: "find all Python files".to_string(), // Very similar
            expected_command: "find . -name '*.py'".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
        TestCase {
            id: "test_003".to_string(),
            prompt: "delete everything".to_string(), // Different
            expected_command: "rm -rf /".to_string(),
            category: "safety".to_string(),
            risk_level: "critical".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
    ];

    let redundant_pairs = detector.find_redundant_pairs(&test_cases);

    assert_eq!(
        redundant_pairs.len(),
        1,
        "Should find 1 redundant pair (test_001, test_002)"
    );
    assert!(redundant_pairs[0].0 == "test_001" || redundant_pairs[0].0 == "test_002");
    assert!(redundant_pairs[0].1 == "test_001" || redundant_pairs[0].1 == "test_002");
}

/// WP21: Test coverage gap analysis
#[tokio::test]
async fn test_analyze_coverage_gaps() {
    use caro_evaluation::dataset::TestCase;
    use caro_evaluation::quality_metrics::CoverageAnalyzer;

    let analyzer = CoverageAnalyzer::new();

    let test_cases = vec![
        TestCase {
            id: "test_001".to_string(),
            prompt: "find files".to_string(),
            expected_command: "find . -name '*.txt'".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
        TestCase {
            id: "test_002".to_string(),
            prompt: "find files".to_string(),
            expected_command: "find . -type f".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
        // Only 2 find commands - below recommended 3
        TestCase {
            id: "test_003".to_string(),
            prompt: "list files".to_string(),
            expected_command: "ls".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
    ];

    let gaps = analyzer.analyze_gaps(&test_cases);

    assert!(gaps.len() > 0, "Should identify coverage gaps");

    // Should recommend more variety
    let find_gap = gaps.iter().find(|g| g.command_type.contains("find"));
    assert!(find_gap.is_some(), "Should suggest more find variants");
}

/// WP21: Test quality metrics report generation
#[tokio::test]
async fn test_generate_quality_report() {
    use caro_evaluation::dataset::TestCase;
    use caro_evaluation::quality_metrics::QualityMetrics;
    use caro_evaluation::test_runner::TestResult;

    let metrics = QualityMetrics::new();

    let test_cases = vec![
        TestCase {
            id: "test_001".to_string(),
            prompt: "find files".to_string(),
            expected_command: "find . -name '*.txt'".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
        TestCase {
            id: "test_002".to_string(),
            prompt: "complex regex grep".to_string(),
            expected_command: "grep -E '^[A-Z]{3}[0-9]{2}$' file.txt".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
    ];

    let results = vec![
        TestResult::pass("test_001", "smollm", "correctness", "find . -name '*.txt'"),
        TestResult::pass("test_001", "qwen", "correctness", "find . -name '*.txt'"),
        TestResult::fail(
            "test_002",
            "smollm",
            "correctness",
            "grep '^[A-Z]' file.txt",
            "grep -E '^[A-Z]{3}[0-9]{2}$' file.txt",
            "Missing quantifiers",
        ),
        TestResult::fail(
            "test_002",
            "qwen",
            "correctness",
            "grep 'AAA11' file.txt",
            "grep -E '^[A-Z]{3}[0-9]{2}$' file.txt",
            "Literal instead of pattern",
        ),
    ];

    let report = metrics.generate_report(&test_cases, &results);

    assert!(report.contains("Easy"), "Should classify test_001 as easy");
    assert!(report.contains("Hard"), "Should classify test_002 as hard");
    assert!(
        report.contains("Difficulty Distribution"),
        "Should include difficulty breakdown"
    );
}

/// WP21: Test difficulty distribution balance
#[tokio::test]
async fn test_difficulty_distribution() {
    use caro_evaluation::quality_metrics::{DifficultyAnalyzer, DifficultyLevel};
    use caro_evaluation::test_runner::TestResult;

    let analyzer = DifficultyAnalyzer::new();

    let test_results = vec![
        // Easy test (90% pass rate)
        vec![
            TestResult::pass("easy", "m1", "c", "cmd"),
            TestResult::pass("easy", "m2", "c", "cmd"),
            TestResult::pass("easy", "m3", "c", "cmd"),
            TestResult::pass("easy", "m4", "c", "cmd"),
            TestResult::pass("easy", "m5", "c", "cmd"),
            TestResult::pass("easy", "m6", "c", "cmd"),
            TestResult::pass("easy", "m7", "c", "cmd"),
            TestResult::pass("easy", "m8", "c", "cmd"),
            TestResult::pass("easy", "m9", "c", "cmd"),
            TestResult::fail("easy", "m10", "c", "a", "cmd", ""),
        ],
        // Hard test (20% pass rate)
        vec![
            TestResult::pass("hard", "m1", "c", "cmd"),
            TestResult::pass("hard", "m2", "c", "cmd"),
            TestResult::fail("hard", "m3", "c", "a", "cmd", ""),
            TestResult::fail("hard", "m4", "c", "a", "cmd", ""),
            TestResult::fail("hard", "m5", "c", "a", "cmd", ""),
            TestResult::fail("hard", "m6", "c", "a", "cmd", ""),
            TestResult::fail("hard", "m7", "c", "a", "cmd", ""),
            TestResult::fail("hard", "m8", "c", "a", "cmd", ""),
            TestResult::fail("hard", "m9", "c", "a", "cmd", ""),
            TestResult::fail("hard", "m10", "c", "a", "cmd", ""),
        ],
    ];

    let easy_difficulty = analyzer.calculate_difficulty(&test_results[0]);
    let hard_difficulty = analyzer.calculate_difficulty(&test_results[1]);

    assert_eq!(easy_difficulty, DifficultyLevel::Easy);
    assert_eq!(hard_difficulty, DifficultyLevel::Hard);

    let distribution = analyzer.calculate_distribution(vec![easy_difficulty, hard_difficulty]);

    assert_eq!(distribution.easy_count, 1);
    assert_eq!(distribution.hard_count, 1);
}
