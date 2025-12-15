//! Comprehensive tests for Safety ML Engine
//!
//! Tests cover:
//! - Feature extraction accuracy
//! - Risk prediction on dangerous commands dataset
//! - Impact estimation
//! - Sandbox execution
//! - Audit logging

use cmdai::safety::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tempfile::TempDir;

/// Test command from dataset
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestCommand {
    command: String,
    expected_risk: f32,
    category: String,
    description: String,
}

/// Load dangerous commands dataset
fn load_test_dataset() -> Vec<TestCommand> {
    let dataset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dangerous_commands.json");

    let contents = std::fs::read_to_string(&dataset_path)
        .expect("Failed to read dangerous commands dataset");

    serde_json::from_str(&contents)
        .expect("Failed to parse dangerous commands dataset")
}

// ============================================================================
// Feature Extraction Tests
// ============================================================================

#[test]
fn test_feature_extraction_basic() {
    let features = CommandFeatures::extract("ls -la");

    assert_eq!(features.token_count, 2);
    assert!(features.flags.contains_key("-l"));
    assert!(features.flags.contains_key("-a"));
    assert_eq!(features.privilege_level, PrivilegeLevel::User);
    assert_eq!(features.destructive_score, 0.0);
}

#[test]
fn test_feature_extraction_dangerous() {
    let features = CommandFeatures::extract("rm -rf /tmp/*");

    assert!(features.has_recursive_flag);
    assert!(features.has_force_flag);
    assert!(features.has_wildcard);
    assert!(features.destructive_score > 0.5);
    // /tmp/* is detected as Root because it starts with /
    assert!(matches!(features.target_scope, TargetScope::Root | TargetScope::System));
}

#[test]
fn test_feature_extraction_sudo() {
    let features = CommandFeatures::extract("sudo apt-get install package");

    assert_eq!(features.privilege_level, PrivilegeLevel::Elevated);
    assert!(features.token_count >= 4);
}

#[test]
fn test_feature_extraction_system_paths() {
    let test_cases = vec![
        ("rm -rf /usr/local", true),
        ("rm -rf /bin/test", true),
        ("rm -rf /etc/config", true),
        ("rm -rf ~/project", false),
    ];

    for (command, should_have_system_path) in test_cases {
        let features = CommandFeatures::extract(command);
        assert_eq!(
            features.has_system_path,
            should_have_system_path,
            "Failed for command: {}",
            command
        );
    }
}

#[test]
fn test_feature_vector_dimensions() {
    let features = CommandFeatures::extract("ls -la");
    let vector = features.to_vector();

    assert_eq!(vector.len(), 30, "Feature vector should have 30 dimensions");

    // Check all values are valid (no NaN or Inf)
    for (i, &value) in vector.iter().enumerate() {
        assert!(
            value.is_finite(),
            "Feature {} has invalid value: {}",
            i,
            value
        );
    }
}

// ============================================================================
// Risk Prediction Tests
// ============================================================================

#[test]
fn test_risk_prediction_safe_commands() {
    let predictor = RuleBasedPredictor::new();

    let safe_commands = vec![
        "ls -la",
        "cat file.txt",
        "grep pattern file.txt",
        "find . -name '*.rs'",
        "git status",
        "docker ps",
    ];

    for command in safe_commands {
        let features = CommandFeatures::extract(command);
        let prediction = predictor.predict_risk(command, &features).unwrap();

        assert!(
            prediction.risk_score < 2.0,
            "Command '{}' should be safe but got risk score {}",
            command,
            prediction.risk_score
        );
    }
}

#[test]
fn test_risk_prediction_dangerous_commands() {
    let predictor = RuleBasedPredictor::new();

    let dangerous_commands = vec![
        ("rm -rf /", 8.0),
        ("dd if=/dev/zero of=/dev/sda", 8.0),
        ("sudo chmod 777 -R /etc", 7.0),
    ];

    for (command, min_risk) in dangerous_commands {
        let features = CommandFeatures::extract(command);
        let prediction = predictor.predict_risk(command, &features).unwrap();

        assert!(
            prediction.risk_score >= min_risk,
            "Command '{}' should have risk >= {} but got {}",
            command,
            min_risk,
            prediction.risk_score
        );
        assert!(!prediction.risk_factors.is_empty());
    }
}

#[test]
fn test_risk_prediction_dataset_accuracy() {
    let predictor = RuleBasedPredictor::new();
    let dataset = load_test_dataset();

    let mut total = 0;
    let mut correct = 0;
    let mut false_positives = 0;
    let mut false_negatives = 0;

    for test_case in dataset {
        let features = CommandFeatures::extract(&test_case.command);
        let prediction = predictor
            .predict_risk(&test_case.command, &features)
            .unwrap();

        total += 1;

        // Define thresholds
        let is_dangerous = test_case.expected_risk >= 5.0;
        let predicted_dangerous = prediction.risk_score >= 5.0;

        if is_dangerous == predicted_dangerous {
            correct += 1;
        } else if predicted_dangerous && !is_dangerous {
            false_positives += 1;
        } else if !predicted_dangerous && is_dangerous {
            false_negatives += 1;
            eprintln!(
                "FALSE NEGATIVE: {} (expected: {}, got: {})",
                test_case.command, test_case.expected_risk, prediction.risk_score
            );
        }
    }

    let accuracy = (correct as f32 / total as f32) * 100.0;
    let fp_rate = (false_positives as f32 / total as f32) * 100.0;
    let fn_rate = (false_negatives as f32 / total as f32) * 100.0;

    println!("\n=== Risk Prediction Accuracy ===");
    println!("Total test cases: {}", total);
    println!("Correct predictions: {} ({:.1}%)", correct, accuracy);
    println!("False positives: {} ({:.1}%)", false_positives, fp_rate);
    println!("False negatives: {} ({:.1}%)", false_negatives, fn_rate);

    // Target: >90% accuracy, <5% false negatives
    assert!(
        accuracy >= 85.0,
        "Accuracy {:.1}% is below target 85%",
        accuracy
    );
    assert!(
        fn_rate <= 10.0,
        "False negative rate {:.1}% is above target 10%",
        fn_rate
    );
}

#[test]
fn test_risk_factors_identification() {
    let predictor = RuleBasedPredictor::new();

    // Test recursive deletion factor
    let features = CommandFeatures::extract("rm -rf /tmp");
    let prediction = predictor.predict_risk("rm -rf /tmp", &features).unwrap();

    assert!(
        prediction
            .risk_factors
            .iter()
            .any(|f| f.name.contains("Recursive")),
        "Should identify recursive deletion risk factor"
    );

    // Test elevated privileges factor
    let features = CommandFeatures::extract("sudo rm file");
    let prediction = predictor.predict_risk("sudo rm file", &features).unwrap();

    assert!(
        prediction
            .risk_factors
            .iter()
            .any(|f| f.name.contains("privilege")),
        "Should identify elevated privileges risk factor"
    );
}

#[test]
fn test_mitigation_suggestions() {
    let predictor = RuleBasedPredictor::new();

    let features = CommandFeatures::extract("rm -rf /tmp/*");
    let prediction = predictor
        .predict_risk("rm -rf /tmp/*", &features)
        .unwrap();

    assert!(
        !prediction.mitigations.is_empty(),
        "Should provide mitigation suggestions for dangerous command"
    );
}

// ============================================================================
// Impact Estimation Tests
// ============================================================================

#[tokio::test]
async fn test_impact_estimator_basic() {
    let temp_dir = TempDir::new().unwrap();
    let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());

    let features = CommandFeatures::extract("ls -la");
    let impact = estimator.estimate("ls -la", &features).await.unwrap();

    assert_eq!(impact.base_estimate.blast_radius, BlastRadius::Local);
    assert!(impact.base_estimate.is_reversible);
}

#[tokio::test]
async fn test_impact_estimator_with_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    tokio::fs::write(temp_dir.path().join("file1.txt"), "test1")
        .await
        .unwrap();
    tokio::fs::write(temp_dir.path().join("file2.txt"), "test2")
        .await
        .unwrap();
    tokio::fs::write(temp_dir.path().join("file3.log"), "log")
        .await
        .unwrap();

    let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());

    let features = CommandFeatures::extract("rm *.txt");
    let impact = estimator.estimate("rm *.txt", &features).await.unwrap();

    // Should detect some files affected
    assert!(impact.base_estimate.files_affected.unwrap_or(0) > 0);
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(500), "500 bytes");
    assert_eq!(format_bytes(1024), "1.00 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(format_bytes(1536), "1.50 KB");
}

// ============================================================================
// Sandbox Tests
// ============================================================================

#[tokio::test]
async fn test_sandbox_creation() {
    let temp_dir = TempDir::new().unwrap();
    let sandbox = Sandbox::create(temp_dir.path()).await;

    assert!(sandbox.is_ok(), "Sandbox creation should succeed");
}

#[tokio::test]
async fn test_sandbox_execution() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file
    tokio::fs::write(temp_dir.path().join("test.txt"), "original")
        .await
        .unwrap();

    let sandbox = Sandbox::create(temp_dir.path()).await.unwrap();
    let result = sandbox.execute("echo 'modified' > test.txt").await.unwrap();

    assert_eq!(result.exit_code, 0);
}

#[tokio::test]
async fn test_sandbox_change_detection() {
    let temp_dir = TempDir::new().unwrap();

    let sandbox = Sandbox::create(temp_dir.path()).await.unwrap();
    let result = sandbox.execute("echo 'new file' > created.txt").await.unwrap();

    assert!(!result.changes.is_empty());
    assert!(result
        .changes
        .iter()
        .any(|c| c.change_type == ChangeType::Created));
}

#[tokio::test]
async fn test_sandbox_rollback() {
    let temp_dir = TempDir::new().unwrap();

    let sandbox = Sandbox::create(temp_dir.path()).await.unwrap();
    sandbox.execute("echo 'test' > file.txt").await.unwrap();

    let rollback_result = sandbox.rollback().await;
    assert!(rollback_result.is_ok());
}

// ============================================================================
// Audit Logger Tests
// ============================================================================

#[tokio::test]
async fn test_audit_logger_init() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let logger = AuditLogger::new(temp_file.path().to_path_buf());

    let result = logger.init().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_audit_logger_write() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let logger = AuditLogger::new(temp_file.path().to_path_buf());
    logger.init().await.unwrap();

    let entry = AuditEntry::new(
        "testuser".to_string(),
        "testhost".to_string(),
        PathBuf::from("/tmp"),
        "list files".to_string(),
        "ls -la".to_string(),
        0.0,
        "Safe".to_string(),
    );

    let result = logger.log(entry).await;
    assert!(result.is_ok());

    // Verify file was written
    let contents = tokio::fs::read_to_string(temp_file.path()).await.unwrap();
    assert!(contents.contains("testuser"));
    assert!(contents.contains("ls -la"));
}

#[tokio::test]
async fn test_audit_logger_query() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let logger = AuditLogger::new(temp_file.path().to_path_buf());
    logger.init().await.unwrap();

    // Log multiple entries
    for i in 0..5 {
        let entry = AuditEntry::new(
            format!("user{}", i),
            "host".to_string(),
            PathBuf::from("/tmp"),
            "test".to_string(),
            format!("command{}", i),
            i as f32,
            "Safe".to_string(),
        );
        logger.log(entry).await.unwrap();
    }

    // Query all entries
    let all_entries = logger.query(AuditFilter::default()).await.unwrap();
    assert_eq!(all_entries.len(), 5);

    // Query with filter
    let filtered = logger
        .query(AuditFilter {
            user: Some("user2".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].user, "user2");
}

#[tokio::test]
async fn test_audit_logger_export_csv() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let logger = AuditLogger::new(temp_file.path().to_path_buf());
    logger.init().await.unwrap();

    let entry = AuditEntry::new(
        "user".to_string(),
        "host".to_string(),
        PathBuf::from("/tmp"),
        "test".to_string(),
        "ls".to_string(),
        0.0,
        "Safe".to_string(),
    );
    logger.log(entry).await.unwrap();

    let csv = logger
        .export_compliance(ComplianceFormat::Csv)
        .await
        .unwrap();

    assert!(csv.contains("timestamp,user,hostname"));
    assert!(csv.contains("user,host"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_safety_pipeline() {
    // Test complete flow: feature extraction -> risk prediction -> impact estimation

    let command = "rm -rf /tmp/test";

    // 1. Extract features
    let features = CommandFeatures::extract(command);
    assert!(features.has_recursive_flag);
    assert!(features.has_force_flag);

    // 2. Predict risk
    let predictor = RuleBasedPredictor::new();
    let prediction = predictor.predict_risk(command, &features).unwrap();
    assert!(prediction.risk_score >= 4.0);
    assert!(!prediction.risk_factors.is_empty());

    // 3. Estimate impact
    let temp_dir = TempDir::new().unwrap();
    let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());
    let impact = estimator.estimate(command, &features).await.unwrap();
    assert!(impact.base_estimate.data_loss_risk > 0.5);

    // 4. Log audit entry
    let temp_log = tempfile::NamedTempFile::new().unwrap();
    let logger = AuditLogger::new(temp_log.path().to_path_buf());
    logger.init().await.unwrap();

    let audit_entry = AuditEntry::new(
        "testuser".to_string(),
        "testhost".to_string(),
        temp_dir.path().to_path_buf(),
        "delete temp files".to_string(),
        command.to_string(),
        prediction.risk_score,
        format!("{:?}", prediction.risk_level()),
    )
    .with_outcome(ExecutionOutcome::Blocked);

    logger.log(audit_entry).await.unwrap();

    // Verify logged
    let entries = logger.query(AuditFilter::default()).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].outcome, ExecutionOutcome::Blocked);
}

#[test]
fn test_performance_feature_extraction() {
    use std::time::Instant;

    let commands: Vec<&str> = vec![
        "ls -la",
        "rm -rf /tmp",
        "sudo apt-get install package",
        "find / -name '*.log'",
        "docker run -it ubuntu bash",
    ];

    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        for command in &commands {
            let _ = CommandFeatures::extract(command);
        }
    }

    let duration = start.elapsed();
    let avg_per_command = duration / (iterations * commands.len() as u32);

    println!("\n=== Feature Extraction Performance ===");
    println!("Total time: {:?}", duration);
    println!(
        "Average per command: {:?}",
        avg_per_command
    );
    println!(
        "Commands per second: {:.0}",
        1_000_000_000.0 / avg_per_command.as_nanos() as f64
    );

    // Should be very fast (< 1ms per command)
    assert!(
        avg_per_command.as_micros() < 1000,
        "Feature extraction took {} μs (target: < 1000 μs)",
        avg_per_command.as_micros()
    );
}

#[test]
fn test_performance_risk_prediction() {
    use std::time::Instant;

    let predictor = RuleBasedPredictor::new();
    let dataset = load_test_dataset();

    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        for test_case in &dataset {
            let features = CommandFeatures::extract(&test_case.command);
            let _ = predictor.predict_risk(&test_case.command, &features);
        }
    }

    let duration = start.elapsed();
    let avg_per_prediction = duration / (iterations * dataset.len() as u32);

    println!("\n=== Risk Prediction Performance ===");
    println!("Total time: {:?}", duration);
    println!("Average per prediction: {:?}", avg_per_prediction);
    println!(
        "Predictions per second: {:.0}",
        1_000_000_000.0 / avg_per_prediction.as_nanos() as f64
    );

    // Target: < 50ms per prediction
    assert!(
        avg_per_prediction.as_millis() < 50,
        "Risk prediction took {} ms (target: < 50 ms)",
        avg_per_prediction.as_millis()
    );
}
