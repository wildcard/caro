// Integration tests for the Feedback System
// Tests validate end-to-end functionality across all feedback components

use std::path::PathBuf;
use tempfile::TempDir;

use caro::feedback::{
    redaction::{redact_context, redact_sensitive_data},
    storage::FeedbackDatabase,
    tui::create_feedback_non_interactive,
    types::{
        CommandInfo, EnvironmentInfo, ErrorInfo, Feedback, FeedbackContext, FeedbackId,
        FeedbackStatus, GitContext, HistoryEntry, SystemState,
    },
};

/// Helper to create a test feedback context
fn create_test_context() -> FeedbackContext {
    FeedbackContext {
        timestamp: chrono::Utc::now(),
        cmdai_version: "1.0.0-test".to_string(),
        environment: EnvironmentInfo {
            os: "linux".to_string(),
            os_version: "Ubuntu 22.04".to_string(),
            arch: "x86_64".to_string(),
            shell: "bash".to_string(),
            terminal: "xterm-256color".to_string(),
            rust_version: Some("1.83.0".to_string()),
        },
        command_info: CommandInfo {
            user_prompt: "list files in /Users/testuser/documents".to_string(),
            generated_command: "ls -la /Users/testuser/documents".to_string(),
            backend: "embedded".to_string(),
            model: Some("smollm-135m".to_string()),
            command_history: vec![],
        },
        error_info: Some(ErrorInfo {
            exit_code: Some(1),
            stderr: "No such file or directory: /Users/testuser/documents".to_string(),
            stdout: "".to_string(),
            error_message: "Command execution failed".to_string(),
            error_type: Some("IOError".to_string()),
        }),
        system_state: SystemState {
            available_backends: vec!["static".to_string(), "embedded".to_string()],
            cache_dir: PathBuf::from("/home/testuser/.cache/caro"),
            config_file: Some(PathBuf::from("/home/testuser/.config/caro/config.toml")),
            is_ci: false,
            is_interactive: true,
        },
        git_context: Some(GitContext {
            repo_url: Some("https://github.com/user/repo.git".to_string()),
            current_branch: "feature/feedback".to_string(),
            has_uncommitted_changes: true,
            last_commit_hash: Some("abc1234".to_string()),
        }),
    }
}

// =============================================================================
// End-to-End Workflow Tests
// =============================================================================

#[test]
fn test_end_to_end_feedback_submission() {
    // INTEGRATION: Complete flow from context capture to database storage

    // Step 1: Create context (simulates what happens during an error)
    let mut context = create_test_context();

    // Step 2: Redact sensitive data from the context
    redact_context(&mut context);

    // Step 3: Create feedback (non-interactive mode)
    let feedback = create_feedback_non_interactive(
        context,
        "The command failed to list files".to_string(),
        Some("1. Ran caro with 'list files'\n2. Got error".to_string()),
    )
    .expect("Should create feedback");

    // Step 4: Store in database
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");
    db.save(&feedback).expect("Failed to save feedback");

    // Step 5: Retrieve and verify
    let retrieved = db.get(&feedback.id).expect("Failed to get feedback");
    assert!(retrieved.is_some(), "Should retrieve saved feedback");

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, feedback.id);
    assert_eq!(
        retrieved.user_description,
        "The command failed to list files"
    );
    assert_eq!(retrieved.status, FeedbackStatus::Submitted);

    // Verify sensitive data was redacted
    assert!(
        !retrieved
            .context
            .command_info
            .user_prompt
            .contains("/Users/testuser"),
        "Should have redacted home directory in user prompt"
    );
}

#[test]
fn test_end_to_end_multiple_feedback() {
    // INTEGRATION: Multiple feedback submissions work correctly

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    // Submit multiple feedback items
    for i in 0..5 {
        let mut context = create_test_context();
        redact_context(&mut context);

        let feedback = create_feedback_non_interactive(
            context,
            format!("Issue number {}", i),
            None,
        )
        .expect("Should create feedback");

        db.save(&feedback).expect("Failed to save feedback");
    }

    // Verify all were saved
    let all = db.list_all().expect("Failed to list all");
    assert_eq!(all.len(), 5, "Should have 5 feedback entries");
}

// =============================================================================
// Database Integration Tests
// =============================================================================

#[test]
fn test_database_full_lifecycle() {
    // INTEGRATION: Full database lifecycle (create, read, update, delete)

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    // Create multiple feedback entries
    let mut context1 = create_test_context();
    redact_context(&mut context1);

    let feedback1 = create_feedback_non_interactive(context1, "First issue".to_string(), None)
        .expect("Should create feedback 1");

    let mut context2 = create_test_context();
    redact_context(&mut context2);

    let feedback2 = create_feedback_non_interactive(
        context2,
        "Second issue".to_string(),
        Some("Steps to reproduce".to_string()),
    )
    .expect("Should create feedback 2");

    // Save both
    db.save(&feedback1).expect("Failed to save feedback 1");
    db.save(&feedback2).expect("Failed to save feedback 2");

    // List all
    let all = db.list_all().expect("Failed to list all");
    assert_eq!(all.len(), 2, "Should have 2 feedback entries");

    // Update status
    db.update_status(&feedback1.id, FeedbackStatus::Triaged, None)
        .expect("Failed to update status");

    // Verify update
    let updated = db
        .get(&feedback1.id)
        .expect("Failed to get updated feedback");
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().status, FeedbackStatus::Triaged);

    // Filter by status
    let triaged = db
        .list_by_status(FeedbackStatus::Triaged)
        .expect("Failed to list by status");
    assert_eq!(triaged.len(), 1);
    assert_eq!(triaged[0].id, feedback1.id);

    let submitted = db
        .list_by_status(FeedbackStatus::Submitted)
        .expect("Failed to list submitted");
    assert_eq!(submitted.len(), 1);
    assert_eq!(submitted[0].id, feedback2.id);
}

#[test]
fn test_database_persistence_across_instances() {
    // INTEGRATION: Database persists data across instances

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let feedback_id: FeedbackId;

    // First instance: create and save
    {
        let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");
        let mut context = create_test_context();
        redact_context(&mut context);

        let feedback =
            create_feedback_non_interactive(context, "Persistent feedback".to_string(), None)
                .expect("Should create feedback");

        feedback_id = feedback.id.clone();
        db.save(&feedback).expect("Failed to save feedback");
    }

    // Second instance: verify data persists
    {
        let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");
        let retrieved = db.get(&feedback_id).expect("Failed to get feedback");

        assert!(
            retrieved.is_some(),
            "Should retrieve feedback from persistent storage"
        );
        assert_eq!(retrieved.unwrap().user_description, "Persistent feedback");
    }
}

#[test]
fn test_database_delete() {
    // INTEGRATION: Database delete functionality

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    let mut context = create_test_context();
    redact_context(&mut context);

    let feedback =
        create_feedback_non_interactive(context, "To be deleted".to_string(), None)
            .expect("Should create feedback");

    db.save(&feedback).expect("Failed to save");

    // Verify it exists
    assert!(db.get(&feedback.id).expect("Get failed").is_some());

    // Delete it
    let deleted = db.delete(&feedback.id).expect("Delete failed");
    assert!(deleted, "Should return true for successful delete");

    // Verify it's gone
    assert!(db.get(&feedback.id).expect("Get failed").is_none());

    // Delete again should return false
    let deleted_again = db.delete(&feedback.id).expect("Delete failed");
    assert!(!deleted_again, "Should return false for non-existent");
}

// =============================================================================
// Redaction Pipeline Tests
// =============================================================================

#[test]
fn test_redaction_in_full_pipeline() {
    // INTEGRATION: Sensitive data is properly redacted throughout the pipeline

    // Create context with sensitive data
    let mut context = create_test_context();
    context.command_info.user_prompt =
        "list files in /Users/johndoe/secret/project".to_string();
    context.command_info.generated_command = "ls /Users/johndoe/secret/project".to_string();

    // Add sensitive command history
    context.command_info.command_history = vec![
        HistoryEntry {
            prompt: "set API_KEY=sk-1234567890abcdefghij".to_string(),
            command: "export API_KEY=sk-1234567890abcdefghij".to_string(),
            timestamp: chrono::Utc::now(),
            success: true,
        },
    ];

    // Add error with sensitive data
    context.error_info = Some(ErrorInfo {
        exit_code: Some(1),
        stderr: "Error in /home/johndoe/work: token invalid".to_string(),
        stdout: "".to_string(),
        error_message: "Auth failed with OPENAI_API_KEY=sk-secretkey123456789".to_string(),
        error_type: Some("AuthError".to_string()),
    });

    // Redact
    redact_context(&mut context);

    // Verify redaction
    assert!(
        !context.command_info.user_prompt.contains("johndoe"),
        "User prompt should not contain username"
    );
    assert!(
        context.command_info.user_prompt.contains("$HOME"),
        "User prompt should contain $HOME placeholder"
    );

    assert!(
        !context.command_info.generated_command.contains("johndoe"),
        "Generated command should not contain username"
    );

    // Check error info is redacted
    if let Some(error) = &context.error_info {
        assert!(
            !error.stderr.contains("johndoe"),
            "stderr should not contain username"
        );
        assert!(
            error.error_message.contains("[REDACTED]") || !error.error_message.contains("sk-secret"),
            "Error message should have secrets redacted"
        );
    }

    // Store in database and retrieve
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    let feedback =
        create_feedback_non_interactive(context, "Test description".to_string(), None)
            .expect("Should create feedback");

    db.save(&feedback).expect("Failed to save");

    let retrieved = db
        .get(&feedback.id)
        .expect("Failed to retrieve")
        .expect("Should have feedback");

    // Verify redaction persists through storage
    assert!(
        !retrieved
            .context
            .command_info
            .user_prompt
            .contains("johndoe"),
        "Retrieved feedback should maintain redaction"
    );
}

#[test]
fn test_redaction_handles_edge_cases() {
    // INTEGRATION: Redaction handles various edge cases

    let edge_cases = vec![
        // Empty strings
        "",
        // No sensitive data
        "just a normal string",
        // Multiple home paths
        "/Users/user1/file and /home/user2/other",
        // Windows paths (should not be redacted on Unix focus)
        "C:\\Users\\username\\Documents",
        // API keys in various formats
        "key=sk-abcdef123456789012345 and token=ghp_ABCDEFGHIJKLMNOP1234567890123456",
        // Mixed content
        "User /home/myuser ran cmd with key=secret123",
    ];

    for case in edge_cases {
        let result = redact_sensitive_data(case);
        // Should not panic and should return a string
        assert!(
            result.len() <= case.len() + 100,
            "Redaction shouldn't drastically increase size"
        );
    }
}

// =============================================================================
// Concurrent Operations Tests
// =============================================================================

#[test]
fn test_concurrent_database_operations() {
    // INTEGRATION: Database handles concurrent operations safely

    use std::thread;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().to_path_buf();

    // Create multiple threads that write to the database
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let path = db_path.clone();
            thread::spawn(move || {
                let db = FeedbackDatabase::new(&path).expect("Failed to create database");

                let mut context = create_test_context();
                redact_context(&mut context);

                let feedback = create_feedback_non_interactive(
                    context,
                    format!("Concurrent feedback {}", i),
                    None,
                )
                .expect("Should create feedback");

                db.save(&feedback).expect("Failed to save feedback");
                feedback.id
            })
        })
        .collect();

    // Collect all feedback IDs
    let ids: Vec<FeedbackId> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread should complete"))
        .collect();

    // Verify all were saved
    let db = FeedbackDatabase::new(&db_path).expect("Failed to create database");
    let all = db.list_all().expect("Failed to list all");

    assert_eq!(all.len(), 5, "Should have 5 concurrent feedback entries");

    // Verify each ID exists
    for id in ids {
        let found = db.get(&id).expect("Failed to get feedback");
        assert!(found.is_some(), "Should find feedback {}", id);
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_error_propagation_across_components() {
    // INTEGRATION: Errors propagate correctly across component boundaries

    // Test invalid feedback ID parsing
    let invalid_ids = vec!["", "invalid", "fb", "fb-", "fb-toolongid"];

    for invalid_id in invalid_ids {
        let result = invalid_id.parse::<FeedbackId>();
        assert!(
            result.is_err(),
            "Should fail to parse invalid ID: {}",
            invalid_id
        );
    }

    // Test invalid ID parsing error messages
    let result = "".parse::<FeedbackId>();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(!err.to_string().is_empty(), "Error should have a message");

    // Test invalid prefix
    let result = "id-abc123".parse::<FeedbackId>();
    assert!(matches!(
        result,
        Err(caro::FeedbackIdError::InvalidPrefix)
    ));

    // Test invalid length
    let result = "fb-abc".parse::<FeedbackId>();
    assert!(result.is_err());

    // Test invalid characters
    let result = "fb-ABC123".parse::<FeedbackId>();
    assert!(result.is_err());
}

#[test]
fn test_database_handles_nonexistent_records() {
    // INTEGRATION: Database handles queries for non-existent records gracefully

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    // Query for non-existent feedback
    let fake_id = FeedbackId::generate();
    let result = db.get(&fake_id).expect("Query should not fail");
    assert!(
        result.is_none(),
        "Should return None for non-existent feedback"
    );

    // Update non-existent feedback should fail
    let update_result = db.update_status(&fake_id, FeedbackStatus::Triaged, None);
    assert!(
        update_result.is_err(),
        "Should fail to update non-existent feedback"
    );
}

// =============================================================================
// Feedback ID Tests
// =============================================================================

#[test]
fn test_feedback_id_uniqueness() {
    // INTEGRATION: Generated feedback IDs are unique

    let ids: Vec<FeedbackId> = (0..1000).map(|_| FeedbackId::generate()).collect();

    // Check all unique
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        assert!(seen.insert(id.to_string()), "IDs should be unique");
    }

    // Verify format
    for id in &ids {
        let s = id.to_string();
        assert!(s.starts_with("fb-"), "ID should start with fb-");
        assert_eq!(s.len(), 9, "ID should be 9 characters (fb- + 6 chars)");
    }
}

#[test]
fn test_feedback_id_roundtrip() {
    // INTEGRATION: Feedback ID can be serialized and parsed back

    for _ in 0..100 {
        let original = FeedbackId::generate();
        let string = original.to_string();
        let parsed: FeedbackId = string.parse().expect("Should parse valid ID");
        assert_eq!(original, parsed, "Roundtrip should preserve ID");
    }
}

// =============================================================================
// Status Workflow Tests
// =============================================================================

#[test]
fn test_feedback_status_workflow() {
    // INTEGRATION: Feedback status transitions work correctly

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    let mut context = create_test_context();
    redact_context(&mut context);

    let feedback =
        create_feedback_non_interactive(context, "Workflow test".to_string(), None)
            .expect("Should create feedback");

    db.save(&feedback).expect("Failed to save");

    // Initial status
    let f = db
        .get(&feedback.id)
        .expect("Get failed")
        .expect("Should exist");
    assert_eq!(f.status, FeedbackStatus::Submitted);

    // Transition through states
    let states = vec![
        FeedbackStatus::Triaged,
        FeedbackStatus::InProgress,
        FeedbackStatus::FixAvailable,
        FeedbackStatus::Resolved,
    ];

    for status in states {
        db.update_status(&feedback.id, status, None)
            .expect("Update should succeed");

        let f = db
            .get(&feedback.id)
            .expect("Get failed")
            .expect("Should exist");
        assert_eq!(f.status, status);
    }
}

#[test]
fn test_feedback_status_with_github_url() {
    // INTEGRATION: Status update with GitHub issue URL

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    let mut context = create_test_context();
    redact_context(&mut context);

    let feedback =
        create_feedback_non_interactive(context, "GitHub test".to_string(), None)
            .expect("Should create feedback");

    db.save(&feedback).expect("Failed to save");

    // Update with GitHub URL
    let github_url = "https://github.com/user/repo/issues/123";
    db.update_status(
        &feedback.id,
        FeedbackStatus::Triaged,
        Some(github_url),
    )
    .expect("Update should succeed");

    // Verify
    let f = db
        .get(&feedback.id)
        .expect("Get failed")
        .expect("Should exist");
    assert_eq!(f.status, FeedbackStatus::Triaged);
    assert_eq!(f.github_issue_url.as_deref(), Some(github_url));
}

// =============================================================================
// Serialization Tests
// =============================================================================

#[test]
fn test_feedback_serialization_roundtrip() {
    // INTEGRATION: Feedback can be serialized and deserialized

    let mut context = create_test_context();
    redact_context(&mut context);

    let feedback = create_feedback_non_interactive(
        context,
        "Serialization test".to_string(),
        Some("Steps".to_string()),
    )
    .expect("Should create feedback");

    // Serialize to JSON
    let json = serde_json::to_string(&feedback).expect("Should serialize");

    // Deserialize back
    let parsed: Feedback = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(feedback.id, parsed.id);
    assert_eq!(feedback.user_description, parsed.user_description);
    assert_eq!(feedback.reproduction_steps, parsed.reproduction_steps);
    assert_eq!(feedback.status, parsed.status);
}

#[test]
fn test_context_serialization() {
    // INTEGRATION: FeedbackContext can be serialized and deserialized

    let mut context = create_test_context();
    redact_context(&mut context);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&context).expect("Should serialize");

    // Deserialize back
    let parsed: FeedbackContext = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(context.cmdai_version, parsed.cmdai_version);
    assert_eq!(context.environment.os, parsed.environment.os);
    assert_eq!(
        context.command_info.backend,
        parsed.command_info.backend
    );
}

// =============================================================================
// Performance Tests
// =============================================================================

#[test]
fn test_redaction_performance() {
    // INTEGRATION: Redaction completes in reasonable time

    let large_text = "Path /home/user/file repeated ".repeat(1000);

    let start = std::time::Instant::now();
    let _ = redact_sensitive_data(&large_text);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 100,
        "Redaction should complete quickly, took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_database_query_performance() {
    // INTEGRATION: Database queries complete in reasonable time

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    // Insert many entries
    for i in 0..100 {
        let mut context = create_test_context();
        redact_context(&mut context);

        let feedback = create_feedback_non_interactive(
            context,
            format!("Performance test {}", i),
            None,
        )
        .expect("Should create feedback");

        db.save(&feedback).expect("Failed to save");
    }

    // Time listing all
    let start = std::time::Instant::now();
    let all = db.list_all().expect("Failed to list");
    let elapsed = start.elapsed();

    assert_eq!(all.len(), 100);
    assert!(
        elapsed.as_millis() < 500,
        "Listing should complete quickly, took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_database_get_recent() {
    // INTEGRATION: Get recent feedback works correctly

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    // Insert entries
    for i in 0..10 {
        let mut context = create_test_context();
        redact_context(&mut context);

        let feedback = create_feedback_non_interactive(
            context,
            format!("Entry {}", i),
            None,
        )
        .expect("Should create feedback");

        db.save(&feedback).expect("Failed to save");
    }

    // Get recent 5
    let recent = db.get_recent(5).expect("Failed to get recent");
    assert_eq!(recent.len(), 5, "Should return 5 recent entries");
}

#[test]
fn test_database_count_by_status() {
    // INTEGRATION: Count by status works correctly

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = FeedbackDatabase::new(temp_dir.path()).expect("Failed to create database");

    // Create feedback with different statuses
    for i in 0..6 {
        let mut context = create_test_context();
        redact_context(&mut context);

        let feedback = create_feedback_non_interactive(
            context,
            format!("Entry {}", i),
            None,
        )
        .expect("Should create feedback");

        db.save(&feedback).expect("Failed to save");

        // Update some statuses
        if i % 2 == 0 {
            db.update_status(&feedback.id, FeedbackStatus::Triaged, None)
                .expect("Update failed");
        }
    }

    // Count by status
    let counts = db.count_by_status().expect("Failed to count");
    assert_eq!(*counts.get("triaged").unwrap_or(&0), 3);
    assert_eq!(*counts.get("submitted").unwrap_or(&0), 3);
}
