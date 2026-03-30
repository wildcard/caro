/// Integration tests for `caro validate` subcommand (ADR-015)
///
/// Tests the exit code contract and JSON assessment payload against known
/// dangerous, safe, and borderline commands.
///
/// Exit code contract:
///   0 = allow
///   1 = block
///   2 = warn
///   3 = internal error
use caro::{
    models::ShellType,
    safety::{AssessmentPayload, Decision, ReviewedBy, SafetyConfig, SafetyValidator},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run the validate handler logic directly (mirrors handle_validate_command in main.rs)
async fn validate(command: &str) -> AssessmentPayload {
    let config = SafetyConfig::moderate();
    let validator = SafetyValidator::new(config).expect("validator init");
    let result = validator
        .validate_command(command, ShellType::Bash)
        .await
        .expect("validation should not error");
    AssessmentPayload::from_validation(result, ReviewedBy::PatternsOnly)
}

// ---------------------------------------------------------------------------
// Exit code contract
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_validate_dangerous_exits_block() {
    let payload = validate("rm -rf /").await;
    assert_eq!(
        payload.decision,
        Decision::Block,
        "rm -rf / must be blocked"
    );
    assert_eq!(payload.exit_code(), 1, "block → exit 1");
}

#[tokio::test]
async fn test_validate_safe_command_exits_allow() {
    let payload = validate("ls -la").await;
    assert_eq!(payload.decision, Decision::Allow, "ls -la must be allowed");
    assert_eq!(payload.exit_code(), 0, "allow → exit 0");
}

#[tokio::test]
async fn test_validate_fork_bomb_exits_block() {
    let payload = validate(":(){ :|:& };:").await;
    assert_eq!(payload.decision, Decision::Block, "fork bomb must be blocked");
    assert_eq!(payload.exit_code(), 1);
}

#[tokio::test]
async fn test_validate_disk_wipe_exits_block() {
    let payload = validate("dd if=/dev/random of=/dev/sda").await;
    assert_eq!(payload.decision, Decision::Block, "disk wipe must be blocked");
    assert_eq!(payload.exit_code(), 1);
}

#[tokio::test]
async fn test_validate_safe_commands_all_allow() {
    let safe = ["pwd", "echo hello", "cat README.md", "git status", "cargo build"];
    for cmd in safe {
        let payload = validate(cmd).await;
        assert_eq!(
            payload.decision,
            Decision::Allow,
            "safe command '{cmd}' should be allowed"
        );
        assert_eq!(payload.exit_code(), 0, "safe command '{cmd}' → exit 0");
    }
}

// ---------------------------------------------------------------------------
// AssessmentPayload field contract
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_payload_has_required_fields() {
    let payload = validate("rm -rf /").await;

    // All fields must be populated
    assert!(!payload.rationale.is_empty(), "rationale must not be empty");
    assert!(payload.risk_score > 0, "dangerous command must have risk_score > 0");
    assert!(!payload.caro_version.is_empty(), "caro_version must be set");

    // reviewed_by must indicate patterns-only (no LLM in basic validate)
    assert!(
        matches!(payload.reviewed_by, ReviewedBy::PatternsOnly),
        "basic validate must use PatternsOnly"
    );
}

#[tokio::test]
async fn test_payload_safe_command_risk_score() {
    let payload = validate("ls -la").await;
    // Safe commands should have low risk scores
    assert!(
        payload.risk_score < 50,
        "safe command risk_score should be < 50, got {}",
        payload.risk_score
    );
}

#[tokio::test]
async fn test_payload_dangerous_command_risk_score() {
    let payload = validate("rm -rf /").await;
    // Critical dangerous commands should have high risk scores
    assert!(
        payload.risk_score >= 70,
        "rm -rf / risk_score should be >= 70, got {}",
        payload.risk_score
    );
}

// ---------------------------------------------------------------------------
// JSON serialisation contract
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_payload_serialises_to_valid_json() {
    let payload = validate("rm -rf /").await;
    let json = serde_json::to_string_pretty(&payload).expect("must serialize to JSON");

    // All required top-level fields present
    assert!(json.contains("\"decision\""), "JSON must contain decision");
    assert!(json.contains("\"risk_score\""), "JSON must contain risk_score");
    assert!(json.contains("\"risk_level\""), "JSON must contain risk_level");
    assert!(json.contains("\"rationale\""), "JSON must contain rationale");
    assert!(json.contains("\"confidence_score\""), "JSON must contain confidence_score");
    assert!(json.contains("\"reviewed_by\""), "JSON must contain reviewed_by");
    assert!(json.contains("\"caro_version\""), "JSON must contain caro_version");
}

#[tokio::test]
async fn test_payload_decision_field_is_lowercase() {
    // Agents depend on lowercase decision values: "allow", "block", "warn"
    let block = validate("rm -rf /").await;
    let json = serde_json::to_string(&block).expect("serialize");
    assert!(
        json.contains("\"block\""),
        "decision must be lowercase 'block' in JSON, got: {json}"
    );

    let allow = validate("ls -la").await;
    let json = serde_json::to_string(&allow).expect("serialize");
    assert!(
        json.contains("\"allow\""),
        "decision must be lowercase 'allow' in JSON, got: {json}"
    );
}

#[tokio::test]
async fn test_payload_deserialises_roundtrip() {
    // JSON roundtrip: serialize then deserialize and compare key fields
    let payload = validate("rm -rf /").await;
    let json = serde_json::to_string(&payload).expect("serialize");
    let recovered: AssessmentPayload = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(payload.decision, recovered.decision);
    assert_eq!(payload.risk_score, recovered.risk_score);
    assert_eq!(payload.rationale, recovered.rationale);
    assert_eq!(payload.caro_version, recovered.caro_version);
}

// ---------------------------------------------------------------------------
// Shell-specific validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_validate_respects_shell_type() {
    // Both Bash and Zsh should catch rm -rf /
    for shell in [ShellType::Bash, ShellType::Zsh, ShellType::Sh] {
        let config = SafetyConfig::moderate();
        let validator = SafetyValidator::new(config).expect("validator init");
        let result = validator
            .validate_command("rm -rf /", shell)
            .await
            .expect("validation should not error");
        let payload = AssessmentPayload::from_validation(result, ReviewedBy::PatternsOnly);
        assert_eq!(
            payload.decision,
            Decision::Block,
            "rm -rf / must be blocked for shell {shell:?}"
        );
    }
}
