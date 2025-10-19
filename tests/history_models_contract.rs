//! Contract tests for command history models
//!
//! Tests T001-T010: History models and database schema validation

use chrono::Utc;
use cmdai::history::models::{
    CommandHistoryEntry, ExecutionMetadata, HistoryQueryFilter, SafetyMetadata,
};
use cmdai::models::{RiskLevel, SafetyLevel, ShellType};
use serde_json;
use std::time::Duration;

#[test]
fn test_command_history_entry_creation() {
    let entry = CommandHistoryEntry::new(
        "ls -la",
        "list all files in directory",
        ShellType::Bash,
        "/home/user",
    );

    assert_eq!(entry.command, "ls -la");
    assert_eq!(entry.explanation, "list all files in directory");
    assert_eq!(entry.shell_type, ShellType::Bash);
    assert_eq!(entry.working_directory, "/home/user");
    assert!(entry.id.len() > 0);
    assert!(entry.timestamp <= Utc::now());
    assert_eq!(entry.user_input, None);
    assert_eq!(entry.execution_metadata, None);
    assert_eq!(entry.safety_metadata, None);
}

#[test]
fn test_command_history_entry_with_metadata() {
    let safety_metadata = SafetyMetadata {
        risk_level: RiskLevel::Moderate,
        patterns_matched: vec!["sudo".to_string()],
        user_confirmed: true,
        safety_score: 0.6,
        safety_level: SafetyLevel::Strict,
        validation_time_ms: 15,
    };

    let execution_metadata = ExecutionMetadata {
        exit_code: Some(0),
        execution_time: Some(Duration::from_millis(1250)),
        output_size: None,
        backend_used: "embedded".to_string(),
        generation_time: Duration::from_millis(450),
        validation_time: Duration::from_millis(75),
        model_name: "Qwen2.5-Coder-3B".to_string(),
        confidence_score: 0.92,
    };

    let mut entry = CommandHistoryEntry::new(
        "sudo systemctl restart nginx",
        "restart nginx service with sudo",
        ShellType::Bash,
        "/etc/nginx",
    );

    entry = entry
        .with_user_input("restart web server")
        .with_safety_metadata(safety_metadata.clone())
        .with_execution_metadata(execution_metadata.clone());

    assert_eq!(entry.user_input, Some("restart web server".to_string()));
    assert_eq!(entry.safety_metadata, Some(safety_metadata));
    assert_eq!(entry.execution_metadata, Some(execution_metadata));
}

#[test]
fn test_command_history_entry_serialization() {
    let entry = CommandHistoryEntry::new(
        "find . -name '*.rs'",
        "find all Rust files in current directory",
        ShellType::Zsh,
        "/workspace/cmdai",
    );

    // Should serialize to JSON without errors
    let json = serde_json::to_string(&entry).expect("Failed to serialize CommandHistoryEntry");
    assert!(json.contains("find . -name '*.rs'"));
    assert!(json.contains("zsh"));

    // Should deserialize from JSON
    let deserialized: CommandHistoryEntry =
        serde_json::from_str(&json).expect("Failed to deserialize CommandHistoryEntry");
    assert_eq!(deserialized.command, entry.command);
    assert_eq!(deserialized.shell_type, entry.shell_type);
}

#[test]
fn test_history_query_filter_creation() {
    let filter = HistoryQueryFilter::new()
        .with_command_pattern("git")
        .with_shell_type(ShellType::Bash)
        .with_working_directory("/workspace")
        .with_time_range(Utc::now() - chrono::Duration::days(7), Utc::now())
        .with_risk_level_max(RiskLevel::Moderate)
        .with_limit(50);

    assert_eq!(filter.command_pattern, Some("git".to_string()));
    assert_eq!(filter.shell_type, Some(ShellType::Bash));
    assert_eq!(filter.working_directory, Some("/workspace".to_string()));
    assert_eq!(filter.limit, 50);
    assert!(filter.start_time.is_some());
    assert!(filter.end_time.is_some());
    assert_eq!(filter.max_risk_level, Some(RiskLevel::Moderate));
}

#[test]
fn test_safety_metadata_validation() {
    #[allow(deprecated)]
    let metadata = SafetyMetadata {
        risk_level: RiskLevel::High,
        patterns_matched: vec!["rm".to_string(), "recursive".to_string()],
        user_confirmed: false,
        safety_score: 0.9,
        safety_level: SafetyLevel::Permissive,
        validation_time_ms: 25,
    };

    assert_eq!(metadata.risk_level, RiskLevel::High);
    assert_eq!(metadata.patterns_matched.len(), 2);
    assert!(!metadata.user_confirmed);
    #[allow(deprecated)]
    {
        assert!(metadata.validation_time_ms > 0);
    }
}

#[test]
fn test_execution_metadata_validation() {
    let metadata = ExecutionMetadata {
        exit_code: Some(1),
        execution_time: Some(Duration::from_secs(2)),
        output_size: Some(1024),
        backend_used: "mlx".to_string(),
        generation_time: Duration::from_millis(1800),
        validation_time: Duration::from_millis(120),
        model_name: "Qwen2.5-Coder-7B".to_string(),
        confidence_score: 0.87,
    };

    assert_eq!(metadata.exit_code, Some(1));
    assert_eq!(metadata.execution_time, Some(Duration::from_secs(2)));
    assert_eq!(metadata.backend_used, "mlx");
    assert!(metadata.confidence_score > 0.0 && metadata.confidence_score <= 1.0);
}

#[test]
fn test_command_history_entry_privacy_filtering() {
    let mut entry = CommandHistoryEntry::new(
        "aws s3 cp --key=SECRET123 file.txt s3://bucket",
        "upload file with credentials",
        ShellType::Bash,
        "/workspace",
    );

    // Should filter out sensitive information
    entry = entry.filter_sensitive_data();

    // Command should be sanitized
    assert!(!entry.command.contains("SECRET123"));
    assert!(entry.command.contains("aws s3 cp"));
}

#[test]
fn test_command_history_entry_search_relevance() {
    let entry = CommandHistoryEntry::new(
        "docker build -t myapp .",
        "build Docker image for application",
        ShellType::Bash,
        "/workspace/myapp",
    );

    // Should calculate search relevance score
    let relevance = entry.calculate_relevance("docker build");
    assert!(relevance > 0.8);

    let relevance = entry.calculate_relevance("kubernetes");
    assert!(relevance < 0.2);
}
