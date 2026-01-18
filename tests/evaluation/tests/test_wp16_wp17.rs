//! WP16-17: Fine-Tuning Integration & Training Metrics Tests
//!
//! These tests validate training dataset export and training metrics
//! correlation for closing the evaluation → training → evaluation loop.

/// WP16: Test OpenAI JSONL format export
#[tokio::test]
async fn test_export_openai_format() {
    use caro_evaluation::dataset_export::{DatasetExporter, ExportFormat};
    use caro_evaluation::test_runner::TestResult;

    let exporter = DatasetExporter::new();

    let results = vec![
        TestResult::pass("test_001", "smollm", "correctness", "find . -type f"),
        TestResult::pass("test_002", "smollm", "correctness", "ls -la"),
    ];

    let exported = exporter
        .export(&results, ExportFormat::OpenAI, None)
        .expect("Should export");

    // Each line should be valid JSON
    for line in exported.lines() {
        let parsed: serde_json::Value = serde_json::from_str(line).expect("Should be valid JSON");
        assert!(parsed.get("messages").is_some(), "Should have messages");

        let messages = parsed["messages"].as_array().unwrap();
        assert!(messages.len() >= 2, "Should have at least system + user");

        // Verify structure
        assert_eq!(messages[0]["role"], "system");
        assert!(messages[0]["content"].is_string());
    }
}

/// WP16: Test preference pairs export
#[tokio::test]
async fn test_export_preference_pairs() {
    use caro_evaluation::dataset_export::{DatasetExporter, ExportFormat};
    use caro_evaluation::test_runner::TestResult;

    let exporter = DatasetExporter::new();

    let results = vec![
        TestResult::pass("test_001", "smollm", "correctness", "find . -type f"),
        TestResult::fail(
            "test_001",
            "qwen",
            "correctness",
            "ls -R",
            "find . -type f",
            "Wrong command",
        ),
    ];

    let exported = exporter
        .export(&results, ExportFormat::PreferencePairs, None)
        .expect("Should export");

    // Should generate preference pairs
    for line in exported.lines() {
        let parsed: serde_json::Value = serde_json::from_str(line).expect("Should be valid JSON");

        assert!(parsed.get("prompt").is_some());
        assert!(parsed.get("chosen").is_some());
        assert!(parsed.get("rejected").is_some());

        // Chosen should be the passing command
        assert_eq!(parsed["chosen"], "find . -type f");
        // Rejected should be the failing command
        assert_eq!(parsed["rejected"], "ls -R");
    }
}

/// WP16: Test filtering only passing tests
#[tokio::test]
async fn test_export_only_passing() {
    use caro_evaluation::dataset_export::{DatasetExporter, ExportFormat, FilterOptions};
    use caro_evaluation::test_runner::TestResult;

    let exporter = DatasetExporter::new();

    let results = vec![
        TestResult::pass("test_001", "smollm", "correctness", "find . -type f"),
        TestResult::fail(
            "test_002",
            "smollm",
            "correctness",
            "ls",
            "ls -la",
            "Missing flags",
        ),
        TestResult::pass("test_003", "smollm", "correctness", "grep foo file"),
    ];

    let filter = FilterOptions {
        only_passing: true,
        only_failing: false,
        min_confidence: None,
    };

    let exported = exporter
        .export(&results, ExportFormat::OpenAI, Some(filter))
        .expect("Should export");

    // Should only have 2 passing tests
    assert_eq!(exported.lines().count(), 2);
}

/// WP16: Test filtering only failing tests
#[tokio::test]
async fn test_export_only_failing() {
    use caro_evaluation::dataset_export::{DatasetExporter, ExportFormat, FilterOptions};
    use caro_evaluation::test_runner::TestResult;

    let exporter = DatasetExporter::new();

    let results = vec![
        TestResult::pass("test_001", "smollm", "correctness", "find . -type f"),
        TestResult::fail(
            "test_002",
            "smollm",
            "correctness",
            "ls",
            "ls -la",
            "Missing flags",
        ),
        TestResult::fail(
            "test_003",
            "smollm",
            "correctness",
            "grep foo",
            "grep -n foo file",
            "Missing args",
        ),
    ];

    let filter = FilterOptions {
        only_passing: false,
        only_failing: true,
        min_confidence: None,
    };

    let exported = exporter
        .export(&results, ExportFormat::OpenAI, Some(filter))
        .expect("Should export");

    // Should only have 2 failing tests
    assert_eq!(exported.lines().count(), 2);
}

/// WP16: Test dataset validation
#[tokio::test]
async fn test_validate_export_format() {
    use caro_evaluation::dataset_export::DatasetExporter;

    let exporter = DatasetExporter::new();

    let valid_openai =
        r#"{"messages":[{"role":"system","content":"test"},{"role":"user","content":"query"}]}"#;

    assert!(
        exporter.validate_openai_format(valid_openai),
        "Should validate correct OpenAI format"
    );

    let invalid_openai = r#"{"invalid":"data"}"#;

    assert!(
        !exporter.validate_openai_format(invalid_openai),
        "Should reject invalid format"
    );
}

/// WP17: Test experiment tracking creation
#[tokio::test]
async fn test_create_experiment() {
    use caro_evaluation::training_tracker::{Experiment, TrainingTracker};

    let tracker = TrainingTracker::new();

    let experiment = Experiment {
        id: "exp-001".to_string(),
        model: "smollm-135m".to_string(),
        baseline_pass_rate: 0.31,
        checkpoints: vec![],
        created_at: "2026-01-18".to_string(),
    };

    let saved = tracker.save_experiment(&experiment);

    assert!(saved.is_ok(), "Should save experiment");
    assert_eq!(experiment.id, "exp-001");
    assert_eq!(experiment.baseline_pass_rate, 0.31);
}

/// WP17: Test checkpoint tracking
#[tokio::test]
async fn test_add_checkpoint() {
    use caro_evaluation::training_tracker::{Checkpoint, Experiment, TrainingTracker};

    let tracker = TrainingTracker::new();

    let mut experiment = Experiment {
        id: "exp-001".to_string(),
        model: "smollm-135m".to_string(),
        baseline_pass_rate: 0.31,
        checkpoints: vec![],
        created_at: "2026-01-18".to_string(),
    };

    let checkpoint = Checkpoint {
        step: 1000,
        training_loss: 0.45,
        eval_pass_rate: 0.38,
        timestamp: "2026-01-18T10:00:00Z".to_string(),
    };

    experiment.checkpoints.push(checkpoint.clone());

    assert_eq!(experiment.checkpoints.len(), 1);
    assert_eq!(experiment.checkpoints[0].step, 1000);
    assert_eq!(experiment.checkpoints[0].eval_pass_rate, 0.38);

    let improvement = experiment.checkpoints[0].eval_pass_rate - experiment.baseline_pass_rate;
    assert!(
        (improvement - 0.07).abs() < 0.01,
        "Should show 7% improvement"
    );
}

/// WP17: Test overfitting detection
#[tokio::test]
async fn test_detect_overfitting() {
    use caro_evaluation::training_tracker::{Checkpoint, OverfittingDetector};

    let detector = OverfittingDetector::new();

    let checkpoints = vec![
        Checkpoint {
            step: 1000,
            training_loss: 0.45,
            eval_pass_rate: 0.38,
            timestamp: "2026-01-18T10:00:00Z".to_string(),
        },
        Checkpoint {
            step: 2000,
            training_loss: 0.32,
            eval_pass_rate: 0.42,
            timestamp: "2026-01-18T11:00:00Z".to_string(),
        },
        Checkpoint {
            step: 3000,
            training_loss: 0.28,
            eval_pass_rate: 0.41, // Eval plateaued while training loss decreased
            timestamp: "2026-01-18T12:00:00Z".to_string(),
        },
    ];

    let is_overfitting = detector.detect(&checkpoints);

    assert!(
        is_overfitting,
        "Should detect overfitting when eval plateaus"
    );
}

/// WP17: Test correlation analysis
#[tokio::test]
async fn test_correlation_analysis() {
    use caro_evaluation::training_tracker::{Checkpoint, CorrelationAnalyzer};

    let analyzer = CorrelationAnalyzer::new();

    let checkpoints = vec![
        Checkpoint {
            step: 1000,
            training_loss: 0.45,
            eval_pass_rate: 0.38,
            timestamp: "2026-01-18T10:00:00Z".to_string(),
        },
        Checkpoint {
            step: 2000,
            training_loss: 0.32,
            eval_pass_rate: 0.42,
            timestamp: "2026-01-18T11:00:00Z".to_string(),
        },
        Checkpoint {
            step: 3000,
            training_loss: 0.28,
            eval_pass_rate: 0.45,
            timestamp: "2026-01-18T12:00:00Z".to_string(),
        },
    ];

    let correlation = analyzer.calculate_correlation(&checkpoints);

    // Negative correlation expected (lower loss → higher pass rate)
    assert!(
        correlation < 0.0,
        "Should show negative correlation between loss and pass rate"
    );
}

/// WP17: Test training effectiveness report
#[tokio::test]
async fn test_generate_effectiveness_report() {
    use caro_evaluation::training_tracker::{Checkpoint, Experiment, TrainingTracker};

    let tracker = TrainingTracker::new();

    let experiment = Experiment {
        id: "exp-001".to_string(),
        model: "smollm-135m".to_string(),
        baseline_pass_rate: 0.31,
        checkpoints: vec![
            Checkpoint {
                step: 1000,
                training_loss: 0.45,
                eval_pass_rate: 0.38,
                timestamp: "2026-01-18T10:00:00Z".to_string(),
            },
            Checkpoint {
                step: 2000,
                training_loss: 0.32,
                eval_pass_rate: 0.42,
                timestamp: "2026-01-18T11:00:00Z".to_string(),
            },
        ],
        created_at: "2026-01-18".to_string(),
    };

    let report = tracker.generate_report(&experiment);

    assert!(report.contains("exp-001"));
    assert!(report.contains("smollm-135m"));
    assert!(report.contains("31")); // Baseline 31%
    assert!(report.contains("42")); // Final 42%
    assert!(report.contains("Improvement") || report.contains("+"));
}
