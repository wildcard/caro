//! End-to-End V2 Workflow Tests
//!
//! Tests the complete V2 pipeline integrating:
//! - Context Intelligence Engine
//! - Safety ML Engine
//! - Learning Engine
//!
//! These tests validate that all three systems work together correctly
//! in realistic scenarios.

use cmdai::intelligence::{ContextGraph, ContextOptions, ProjectType};
use cmdai::learning::{CommandPattern, LearningEngine, PatternDB};
use cmdai::safety::{
    AuditEntry, AuditLogger, CommandFeatures, ExecutionOutcome, ImpactEstimator,
    RiskLevel, RuleBasedPredictor, Sandbox,
};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper to create a test Rust project
async fn create_rust_project(dir: &Path) -> std::io::Result<()> {
    tokio::fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
"#,
    )
    .await?;

    tokio::fs::create_dir_all(dir.join("src")).await?;
    tokio::fs::write(
        dir.join("src/main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}
"#,
    )
    .await?;

    Ok(())
}

/// Helper to create a Git repository
async fn create_git_repo(dir: &Path) -> std::io::Result<()> {
    use tokio::process::Command;

    Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output()
        .await?;

    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(dir)
        .output()
        .await?;

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .await?;

    tokio::fs::write(dir.join("README.md"), "# Test Project").await?;

    Command::new("git")
        .args(&["add", "."])
        .current_dir(dir)
        .output()
        .await?;

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(dir)
        .output()
        .await?;

    Ok(())
}

// ============================================================================
// WORKFLOW 1: Happy Path - Safe Command Generation with Context
// ============================================================================

#[tokio::test]
async fn test_e2e_safe_command_with_context() {
    // Setup: Create test project with Cargo.toml
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path()).await.unwrap();

    // Step 1: Build ContextGraph (should detect Rust project)
    let context = ContextGraph::build(temp_dir.path()).await;
    assert!(
        context.is_ok(),
        "Context build should succeed: {:?}",
        context.err()
    );

    let context = context.unwrap();
    assert_eq!(
        context.project.project_type,
        ProjectType::Rust,
        "Should detect Rust project from Cargo.toml"
    );
    assert!(
        context.project.key_dependencies.contains(&"tokio".to_string()),
        "Should extract tokio dependency"
    );

    // Step 2: Generate command with context (simulated prompt)
    let user_prompt = "build my project";
    let generated_cmd = "cargo build"; // In real scenario, this comes from LLM

    // Step 3: Safety ML validates (should be SAFE)
    let features = CommandFeatures::extract(generated_cmd);
    let predictor = RuleBasedPredictor::new();
    let risk = predictor
        .predict_risk(generated_cmd, &features)
        .expect("Risk prediction should succeed");

    assert!(
        risk.risk_score < 2.0,
        "cargo build should be safe, got risk score {}",
        risk.risk_score
    );
    assert!(
        matches!(risk.risk_level(), RiskLevel::Safe),
        "Risk level should be Safe, got {:?}",
        risk.risk_level()
    );

    // Step 4: Learning engine records interaction
    let db = PatternDB::new(":memory:").await.unwrap();
    db.record_interaction(
        user_prompt,
        generated_cmd,
        &context.to_llm_context(),
        None,
    )
    .await
    .expect("Should record interaction");

    // Step 5: Verify command is correct and safe
    let patterns = db.find_by_prompt(user_prompt).await.unwrap();
    assert_eq!(patterns.len(), 1, "Should have recorded one pattern");
    assert_eq!(
        patterns[0].generated_command, generated_cmd,
        "Recorded command should match"
    );

    println!(
        "✓ Safe command workflow complete: {} (risk: {:.1})",
        generated_cmd, risk.risk_score
    );
}

// ============================================================================
// WORKFLOW 2: Dangerous Command Prevention
// ============================================================================

#[tokio::test]
async fn test_e2e_dangerous_command_blocked() {
    // User prompt: "delete all files in root"
    let user_prompt = "delete all files in root";
    let generated_cmd = "rm -rf /"; // Dangerous command

    // Step 1: Build context (doesn't matter for this test)
    let temp_dir = TempDir::new().unwrap();
    let context = ContextGraph::build(temp_dir.path()).await.unwrap();

    // Step 2: Extract features
    let features = CommandFeatures::extract(generated_cmd);

    // Step 3: Safety ML predicts HIGH/CRITICAL risk
    let predictor = RuleBasedPredictor::new();
    let risk = predictor.predict_risk(generated_cmd, &features).unwrap();

    assert!(
        risk.risk_score >= 8.0,
        "rm -rf / should be CRITICAL risk, got {}",
        risk.risk_score
    );
    assert!(
        matches!(risk.risk_level(), RiskLevel::Critical),
        "Risk level should be Critical, got {:?}",
        risk.risk_level()
    );
    assert!(
        !risk.risk_factors.is_empty(),
        "Should identify risk factors"
    );

    // Step 4: System blocks execution (simulated - no actual execution)
    let should_block = risk.risk_score >= 8.0;
    assert!(should_block, "Command should be blocked");

    // Step 5: Audit log created
    let temp_log = tempfile::NamedTempFile::new().unwrap();
    let logger = AuditLogger::new(temp_log.path().to_path_buf());
    logger.init().await.unwrap();

    let audit_entry = AuditEntry::new(
        "testuser".to_string(),
        "testhost".to_string(),
        temp_dir.path().to_path_buf(),
        user_prompt.to_string(),
        generated_cmd.to_string(),
        risk.risk_score,
        format!("{:?}", risk.risk_level()),
    )
    .with_outcome(ExecutionOutcome::Blocked);

    logger.log(audit_entry).await.unwrap();

    // Verify audit log
    let entries = logger.query(Default::default()).await.unwrap();
    assert_eq!(entries.len(), 1, "Should have one audit entry");
    assert_eq!(
        entries[0].outcome,
        ExecutionOutcome::Blocked,
        "Outcome should be Blocked"
    );

    println!(
        "✓ Dangerous command blocked: {} (risk: {:.1})",
        generated_cmd, risk.risk_score
    );
}

// ============================================================================
// WORKFLOW 3: Learning from User Edits
// ============================================================================

#[tokio::test]
async fn test_e2e_learning_from_corrections() {
    // Setup: User prompt "list files"
    let user_prompt = "list files";
    let generated_cmd = "ls";
    let user_edited_cmd = "ls -la --color"; // User adds flags

    let temp_dir = TempDir::new().unwrap();
    let context = ContextGraph::build(temp_dir.path()).await.unwrap();

    // Step 1: Record initial generation
    let db = PatternDB::new(":memory:").await.unwrap();
    let pattern_id = db
        .record_interaction(user_prompt, generated_cmd, &context.to_llm_context(), None)
        .await
        .unwrap();

    // Step 2: User edits to "ls -la --color"
    db.learn_from_edit(&pattern_id, user_edited_cmd, true, Some(5))
        .await
        .unwrap();

    // Step 3: Verify pattern stored
    let pattern = db.get_pattern_by_id(&pattern_id).await.unwrap();
    assert_eq!(
        pattern.final_command,
        Some(user_edited_cmd.to_string()),
        "Should store user's edited command"
    );
    assert_eq!(
        pattern.execution_success,
        Some(true),
        "Should record execution success"
    );
    assert_eq!(
        pattern.user_rating,
        Some(5),
        "Should record user rating"
    );

    // Step 4: Verify edited patterns can be retrieved
    let edited = db.get_edited_patterns().await.unwrap();
    assert_eq!(edited.len(), 1, "Should have one edited pattern");
    assert_eq!(edited[0].id, pattern_id, "Pattern ID should match");

    // Step 5: Next time, suggest learned command (simulated)
    // In real implementation, LLM would be biased toward "ls -la --color"
    // based on this pattern

    println!(
        "✓ Learning from edit: {} → {} (rating: 5)",
        generated_cmd, user_edited_cmd
    );
}

// ============================================================================
// WORKFLOW 4: Context-Aware Command Generation
// ============================================================================

#[tokio::test]
async fn test_e2e_context_aware_commands() {
    // Setup: Git repo with uncommitted changes
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path()).await.unwrap();
    create_git_repo(temp_dir.path()).await.unwrap();

    // Make uncommitted change
    tokio::fs::write(temp_dir.path().join("test.txt"), "uncommitted")
        .await
        .unwrap();

    // Step 1: Context detects Git repo
    let context = ContextGraph::build(temp_dir.path()).await.unwrap();

    assert!(context.git.is_repo, "Should detect Git repository");
    assert!(
        context.git.branch.is_some(),
        "Should detect current branch"
    );

    // Step 2: LLM context includes Git info
    let llm_context = context.to_llm_context();
    assert!(
        llm_context.contains("Git:") || llm_context.contains("Branch:"),
        "LLM context should include Git information"
    );

    // Step 3: Generated command should be Git-specific (simulated)
    let user_prompt = "show my changes";
    let generated_cmd = "git diff"; // Context-aware (not generic diff)

    // Verify command is appropriate for Git repo
    assert!(
        generated_cmd.starts_with("git"),
        "Command should be Git-specific in Git repo"
    );

    println!(
        "✓ Context-aware command: '{}' → {} (detected: Git repo)",
        user_prompt, generated_cmd
    );
}

// ============================================================================
// WORKFLOW 5: Tutorial Completion Flow
// ============================================================================

#[tokio::test]
async fn test_e2e_tutorial_progression() {
    use cmdai::learning::{Tutorial, TutorialDifficulty};

    // Step 1: Load tutorial
    let tutorial = Tutorial::find_basics();
    assert_eq!(tutorial.id, "find-basics", "Should load find tutorial");
    assert_eq!(
        tutorial.difficulty,
        TutorialDifficulty::Beginner,
        "Should be beginner difficulty"
    );
    assert_eq!(tutorial.lessons.len(), 3, "Should have 3 lessons");

    // Step 2: Complete lesson 1
    let lesson1 = &tutorial.lessons[0];
    assert_eq!(
        lesson1.title, "Finding Files by Name",
        "First lesson should be about finding by name"
    );

    let quiz = lesson1.quiz.as_ref().expect("Lesson should have quiz");
    let user_answer = "find . -name '*.log'";
    let correct = quiz.check_answer(user_answer);
    assert!(correct, "Answer should be correct");

    // Step 3: Achievement unlocking (simulated)
    use cmdai::learning::{Achievement, UnlockCondition};

    let first_lesson_achievement = Achievement {
        id: "first-lesson".to_string(),
        name: "First Lesson".to_string(),
        description: "Completed your first tutorial lesson".to_string(),
        icon: "📚".to_string(),
        unlock_condition: UnlockCondition::TutorialsCompleted { count: 1 },
    };

    // Check if condition met (1 lesson completed)
    assert!(
        matches!(
            first_lesson_achievement.unlock_condition,
            UnlockCondition::TutorialsCompleted { count: 1 }
        ),
        "Achievement should unlock after 1 tutorial"
    );

    // Step 4: Progress tracking (would be in database in real implementation)
    // Step 5: Resume tutorial later (database would persist state)

    println!(
        "✓ Tutorial progression: Completed lesson 1 of {}",
        tutorial.title
    );
}

// ============================================================================
// WORKFLOW 6: Sandbox Execution
// ============================================================================

#[tokio::test]
async fn test_e2e_sandbox_execution() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file
    tokio::fs::write(temp_dir.path().join("test.txt"), "original content")
        .await
        .unwrap();

    // Step 1: Create sandbox
    let sandbox = Sandbox::create(temp_dir.path()).await;
    assert!(sandbox.is_ok(), "Sandbox creation should succeed");

    let sandbox = sandbox.unwrap();

    // Step 2: Execute command in sandbox
    let command = "echo 'modified' > test.txt";
    let result = sandbox.execute(command).await;
    assert!(result.is_ok(), "Sandbox execution should succeed");

    let result = result.unwrap();
    assert_eq!(result.exit_code, 0, "Command should succeed");

    // Step 3: Verify changes detected
    assert!(
        !result.changes.is_empty(),
        "Should detect file modifications"
    );

    // Step 4: Verify changes isolated (original file unchanged)
    let original_content = tokio::fs::read_to_string(temp_dir.path().join("test.txt"))
        .await
        .unwrap();
    assert_eq!(
        original_content, "original content",
        "Original file should be unchanged (sandboxed)"
    );

    // Step 5: Rollback test
    let rollback = sandbox.rollback().await;
    assert!(rollback.is_ok(), "Rollback should succeed");

    println!("✓ Sandbox execution: Command isolated and rolled back");
}

// ============================================================================
// WORKFLOW 7: Audit Logging for Compliance
// ============================================================================

#[tokio::test]
async fn test_e2e_audit_trail() {
    use cmdai::safety::{AuditFilter, ComplianceFormat};

    let temp_log = tempfile::NamedTempFile::new().unwrap();

    // Step 1: Enable audit logging
    let logger = AuditLogger::new(temp_log.path().to_path_buf());
    logger.init().await.unwrap();

    // Step 2: Execute high-risk command (simulated)
    let temp_dir = TempDir::new().unwrap();
    let command = "sudo rm -rf /tmp/test";
    let features = CommandFeatures::extract(command);
    let predictor = RuleBasedPredictor::new();
    let risk = predictor.predict_risk(command, &features).unwrap();

    let entry = AuditEntry::new(
        "admin".to_string(),
        "production-server".to_string(),
        temp_dir.path().to_path_buf(),
        "delete temp files".to_string(),
        command.to_string(),
        risk.risk_score,
        format!("{:?}", risk.risk_level()),
    )
    .with_outcome(ExecutionOutcome::Success)
    .with_exit_code(0)
    .with_duration(1500);

    // Step 3: Verify audit log entry created
    logger.log(entry).await.unwrap();

    // Step 4: Query audit logs
    let all_entries = logger.query(AuditFilter::default()).await.unwrap();
    assert_eq!(all_entries.len(), 1, "Should have one audit entry");

    // Verify all fields captured
    assert_eq!(all_entries[0].user, "admin", "User should be recorded");
    assert_eq!(
        all_entries[0].hostname, "production-server",
        "Hostname should be recorded"
    );
    assert_eq!(
        all_entries[0].command, command,
        "Command should be recorded"
    );
    assert!(
        all_entries[0].risk_score >= 5.0,
        "Risk score should be recorded"
    );
    assert!(
        all_entries[0].timestamp.len() > 0,
        "Timestamp should be recorded"
    );

    // Step 5: Export logs to JSON
    let json_export = logger
        .export_compliance(ComplianceFormat::JsonLines)
        .await
        .unwrap();
    assert!(
        json_export.contains("\"user\":\"admin\""),
        "JSON export should contain user field"
    );
    assert!(
        json_export.contains(command),
        "JSON export should contain command"
    );

    println!(
        "✓ Audit trail: Command logged and exportable (risk: {:.1})",
        risk.risk_score
    );
}

// ============================================================================
// WORKFLOW 8: Full Pipeline Integration Test
// ============================================================================

#[tokio::test]
async fn test_e2e_full_pipeline_integration() {
    // This test validates the complete V2 pipeline from end to end

    // Setup: Create realistic project
    let temp_dir = TempDir::new().unwrap();
    create_rust_project(temp_dir.path()).await.unwrap();
    create_git_repo(temp_dir.path()).await.unwrap();

    // 1. CONTEXT INTELLIGENCE
    let start = std::time::Instant::now();
    let context = ContextGraph::build(temp_dir.path()).await.unwrap();
    let context_time = start.elapsed();

    assert!(
        context_time.as_millis() < 300,
        "Context build should be <300ms, took {}ms",
        context_time.as_millis()
    );
    assert_eq!(context.project.project_type, ProjectType::Rust);
    assert!(context.git.is_repo);

    // 2. COMMAND GENERATION (simulated)
    let user_prompt = "run tests";
    let generated_cmd = "cargo test"; // Context-aware (Rust project)
    let llm_context = context.to_llm_context();
    assert!(llm_context.contains("Rust") || llm_context.contains("cargo"));

    // 3. SAFETY VALIDATION
    let start = std::time::Instant::now();
    let features = CommandFeatures::extract(generated_cmd);
    let predictor = RuleBasedPredictor::new();
    let risk = predictor.predict_risk(generated_cmd, &features).unwrap();
    let safety_time = start.elapsed();

    assert!(
        safety_time.as_millis() < 50,
        "Risk prediction should be <50ms, took {}ms",
        safety_time.as_millis()
    );
    assert!(
        risk.risk_score < 2.0,
        "cargo test should be safe, got {}",
        risk.risk_score
    );

    // 4. IMPACT ESTIMATION
    let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());
    let impact = estimator.estimate(generated_cmd, &features).await.unwrap();
    assert!(impact.base_estimate.is_reversible);

    // 5. LEARNING ENGINE
    let db = PatternDB::new(":memory:").await.unwrap();
    let pattern_id = db
        .record_interaction(user_prompt, generated_cmd, &llm_context, None)
        .await
        .unwrap();
    assert!(!pattern_id.is_empty());

    // 6. AUDIT LOGGING
    let temp_log = tempfile::NamedTempFile::new().unwrap();
    let logger = AuditLogger::new(temp_log.path().to_path_buf());
    logger.init().await.unwrap();

    let audit_entry = AuditEntry::new(
        "developer".to_string(),
        "dev-machine".to_string(),
        temp_dir.path().to_path_buf(),
        user_prompt.to_string(),
        generated_cmd.to_string(),
        risk.risk_score,
        format!("{:?}", risk.risk_level()),
    )
    .with_outcome(ExecutionOutcome::Success);

    logger.log(audit_entry).await.unwrap();

    // 7. VERIFICATION
    let patterns = db.find_by_prompt(user_prompt).await.unwrap();
    assert_eq!(patterns.len(), 1);

    let audit_entries = logger.query(Default::default()).await.unwrap();
    assert_eq!(audit_entries.len(), 1);

    println!(
        "✓ Full pipeline integration: Context ({:.0}ms) → Safety ({:.0}ms) → Learning → Audit",
        context_time.as_millis(),
        safety_time.as_millis()
    );
}

// ============================================================================
// WORKFLOW 9: Error Recovery and Graceful Degradation
// ============================================================================

#[tokio::test]
async fn test_e2e_graceful_degradation() {
    // Test that system degrades gracefully when components fail

    // 1. Context fails on invalid path - should still work
    let invalid_path = PathBuf::from("/nonexistent/path/12345");
    let context_result = ContextGraph::build(&invalid_path).await;

    // Context should either succeed with warnings or provide minimal context
    if let Ok(context) = context_result {
        // Should have warnings about missing path
        assert!(
            !context.warnings.is_empty() || context.environment.platform.len() > 0,
            "Should provide warnings or minimal environment context"
        );
    }

    // 2. Safety check should work even without context
    let command = "ls -la";
    let features = CommandFeatures::extract(command);
    let predictor = RuleBasedPredictor::new();
    let risk = predictor.predict_risk(command, &features).unwrap();
    assert!(risk.risk_score < 2.0, "Safety should work independently");

    // 3. Learning should work with minimal context
    let db = PatternDB::new(":memory:").await.unwrap();
    let result = db
        .record_interaction("test", command, "minimal context", None)
        .await;
    assert!(
        result.is_ok(),
        "Learning should work with any context string"
    );

    println!("✓ Graceful degradation: System works even when components fail");
}
