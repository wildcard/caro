//! WP14-15: Product Feedback Loops Tests
//!
//! These tests validate automated issue creation and pattern extraction
//! for continuous improvement of the evaluation system.

/// WP14: Test regression detection
#[tokio::test]
async fn test_detect_regression() {
    use caro_evaluation::issue_automation::RegressionDetector;
    use caro_evaluation::model_profiling::ModelProfile;

    let detector = RegressionDetector::new(10.0); // 10% threshold

    let baseline = ModelProfile {
        model_name: "smollm".to_string(),
        total_tests: 55,
        passed_tests: 47,
        overall_pass_rate: 0.85,
        category_performance: vec![],
        strengths: vec![],
        weaknesses: vec![],
    };

    let current = ModelProfile {
        model_name: "smollm".to_string(),
        total_tests: 55,
        passed_tests: 23,
        overall_pass_rate: 0.42,
        category_performance: vec![],
        strengths: vec![],
        weaknesses: vec![],
    };

    let regression = detector.detect_regression(&baseline, &current);

    assert!(regression.is_some(), "Should detect regression");
    let reg = regression.unwrap();
    assert_eq!(reg.model, "smollm");
    assert!((reg.regression_pct - 51.0).abs() < 2.0); // ~51% regression (0.85→0.42)
    assert!(reg.is_critical);
}

/// WP14: Test issue template generation
#[tokio::test]
async fn test_generate_issue_template() {
    use caro_evaluation::issue_automation::{IssueGenerator, RegressionInfo};

    let generator = IssueGenerator::new();

    let regression = RegressionInfo {
        model: "smollm".to_string(),
        test_category: "correctness".to_string(),
        baseline_pass_rate: 0.85,
        current_pass_rate: 0.42,
        regression_pct: 43.0,
        is_critical: true,
        sample_failures: vec![
            ("test_001".to_string(), "find . -mtime -1".to_string()),
            ("test_002".to_string(), "grep TODO".to_string()),
        ],
    };

    let issue = generator.generate_issue_template(&regression);

    assert!(issue.title.contains("Regression"));
    assert!(issue.title.contains("smollm"));
    assert!(issue.body.contains("85%"));
    assert!(issue.body.contains("42%"));
    assert!(issue.body.contains("-43%"));
    assert!(issue.labels.contains(&"regression".to_string()));
    assert!(issue.labels.contains(&"backend:smollm".to_string()));
    assert!(issue.labels.contains(&"priority:critical".to_string())); // 43% >= 30%
}

/// WP14: Test priority assignment
#[tokio::test]
async fn test_assign_priority() {
    use caro_evaluation::issue_automation::RegressionDetector;

    let detector = RegressionDetector::new(10.0);

    // Critical: >30% regression
    assert_eq!(
        detector.assign_priority(50.0),
        "priority:critical",
        "50% regression should be critical"
    );

    // High: 20-30%
    assert_eq!(
        detector.assign_priority(25.0),
        "priority:high",
        "25% regression should be high"
    );

    // Medium: 10-20%
    assert_eq!(
        detector.assign_priority(15.0),
        "priority:medium",
        "15% regression should be medium"
    );

    // Low: <10%
    assert_eq!(
        detector.assign_priority(5.0),
        "priority:low",
        "5% regression should be low"
    );
}

/// WP14: Test label generation
#[tokio::test]
async fn test_generate_labels() {
    use caro_evaluation::issue_automation::IssueGenerator;

    let generator = IssueGenerator::new();

    let labels = generator.generate_labels("smollm", "correctness", 35.0);

    assert!(labels.contains(&"regression".to_string()));
    assert!(labels.contains(&"backend:smollm".to_string()));
    assert!(labels.contains(&"category:correctness".to_string()));
    assert!(labels.contains(&"priority:critical".to_string()));
}

/// WP14: Test issue creation dry-run mode
#[tokio::test]
async fn test_issue_creation_dry_run() {
    use caro_evaluation::issue_automation::{IssueAutomation, RegressionInfo};

    let automation = IssueAutomation::new(true); // dry_run = true

    let regression = RegressionInfo {
        model: "qwen".to_string(),
        test_category: "safety".to_string(),
        baseline_pass_rate: 0.90,
        current_pass_rate: 0.65,
        regression_pct: 25.0,
        is_critical: false,
        sample_failures: vec![],
    };

    let result = automation.create_issue(&regression).await;

    assert!(result.is_ok(), "Dry run should succeed");
    assert!(
        result.unwrap().contains("DRY RUN"),
        "Should indicate dry run"
    );
}

/// WP15: Test failure pattern extraction
#[tokio::test]
async fn test_extract_failure_patterns() {
    use caro_evaluation::pattern_extraction::PatternExtractor;
    use caro_evaluation::test_runner::TestResult;

    let extractor = PatternExtractor::new();

    let failures = vec![
        TestResult::fail(
            "test_001",
            "smollm",
            "correctness",
            "find . -mtime -1",
            "find . -mtime 0",
            "Using -mtime -1 instead of -mtime 0",
        ),
        TestResult::fail(
            "test_002",
            "smollm",
            "correctness",
            "find . -mtime -1",
            "find . -mtime 0",
            "Using -mtime -1 instead of -mtime 0",
        ),
        TestResult::fail(
            "test_003",
            "qwen",
            "correctness",
            "grep TODO",
            "grep -n TODO",
            "Missing -n flag for line numbers",
        ),
    ];

    let patterns = extractor.extract_patterns(&failures);

    assert!(!patterns.is_empty(), "Should extract patterns");
    assert_eq!(patterns.len(), 2, "Should find 2 unique patterns");

    let mtime_pattern = patterns
        .iter()
        .find(|p| p.description.contains("-mtime"))
        .unwrap();
    assert_eq!(mtime_pattern.count, 2, "mtime pattern should occur twice");
}

/// WP15: Test safety pattern gap detection
#[tokio::test]
async fn test_detect_safety_gaps() {
    use caro_evaluation::pattern_extraction::PatternExtractor;

    let extractor = PatternExtractor::new();

    let blocked_commands = vec![
        "shred -u secret.txt".to_string(),
        "truncate -s 0 file.txt".to_string(),
        "dd if=/dev/zero of=disk.img".to_string(),
    ];

    let existing_patterns = vec!["rm -rf", "mkfs", "> /dev/"];

    let gaps = extractor.detect_safety_gaps(&blocked_commands, &existing_patterns);

    assert!(!gaps.is_empty(), "Should detect safety gaps");
    assert!(
        gaps.iter().any(|g| g.contains("shred")),
        "Should detect shred gap"
    );
    assert!(
        gaps.iter().any(|g| g.contains("truncate")),
        "Should detect truncate gap"
    );
}

/// WP15: Test test case suggestions
#[tokio::test]
async fn test_suggest_test_cases() {
    use caro_evaluation::pattern_extraction::PatternExtractor;
    use caro_evaluation::test_runner::TestResult;

    let extractor = PatternExtractor::new();

    let failures = vec![TestResult::fail(
        "test_001",
        "smollm",
        "correctness",
        "find",
        "find . -type f",
        "Missing path and type",
    )];

    let suggestions = extractor.suggest_test_cases(&failures);

    assert!(!suggestions.is_empty(), "Should suggest test cases");
    assert!(
        suggestions.iter().any(|s| s.contains("find")),
        "Should suggest find-related test"
    );
}

/// WP15: Test insights report generation
#[tokio::test]
async fn test_generate_insights_report() {
    use caro_evaluation::pattern_extraction::{FailurePattern, PatternExtractor};

    let extractor = PatternExtractor::new();

    let patterns = vec![
        FailurePattern {
            description: "BSD vs GNU flag confusion".to_string(),
            count: 15,
            examples: vec!["find . -mtime -1".to_string()],
            suggested_fix: "Add POSIX examples to prompt".to_string(),
        },
        FailurePattern {
            description: "Missing -n flag for line numbers".to_string(),
            count: 8,
            examples: vec!["grep TODO".to_string()],
            suggested_fix: "Include -n in grep examples".to_string(),
        },
    ];

    let safety_gaps = vec!["shred".to_string(), "truncate".to_string()];
    let test_suggestions = vec!["securely delete file".to_string()];

    let report = extractor.generate_insights_report(&patterns, &safety_gaps, &test_suggestions);

    assert!(report.contains("Insights"));
    assert!(report.contains("Common Failure Modes"));
    assert!(report.contains("BSD vs GNU"));
    assert!(report.contains("15"));
    assert!(report.contains("Safety Pattern Gaps"));
    assert!(report.contains("shred"));
    assert!(report.contains("Suggested Test Cases"));
}

/// WP15: Test failure grouping by category
#[tokio::test]
async fn test_group_failures_by_category() {
    use caro_evaluation::pattern_extraction::PatternExtractor;
    use caro_evaluation::test_runner::TestResult;

    let extractor = PatternExtractor::new();

    let failures = vec![
        TestResult::fail(
            "test_001",
            "smollm",
            "correctness",
            "ls",
            "ls -la",
            "Missing flags",
        ),
        TestResult::fail(
            "test_002",
            "smollm",
            "correctness",
            "find",
            "find .",
            "Missing path",
        ),
        TestResult::fail(
            "test_003",
            "qwen",
            "safety",
            "rm *",
            "blocked",
            "Dangerous pattern",
        ),
    ];

    let grouped = extractor.group_by_category(&failures);

    assert_eq!(grouped.len(), 2, "Should have 2 categories");
    assert_eq!(grouped["correctness"].len(), 2);
    assert_eq!(grouped["safety"].len(), 1);
}
