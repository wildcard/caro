//! Integration tests for runtime-loadable safety patterns via TOML config.
//!
//! Verifies the wiring documented in
//! `~/.claude/plans/strip-novel-ideas-from-expressive-waffle.md`:
//!
//! - User-defined `[[safety.custom_patterns]]` in `config.toml` flows through
//!   to `SafetyValidator` and blocks matching commands.
//! - Optional sibling `patterns.toml` file is merged in.
//! - Hardening invariants are enforced:
//!   - User patterns capped at `High` severity (Critical reserved for built-ins).
//!   - Pattern source ≤ 512 chars (ReDoS bound).
//!   - Malformed regex surfaces a `ValidationError`, not silent skip.
//!   - User patterns cannot weaken/disable a built-in (additive only).

use caro::config::ConfigManager;
use caro::models::{RiskLevel, ShellType};
use caro::safety::{validate_user_pattern, DangerPattern, SafetyConfig, SafetyValidator};
use tempfile::TempDir;

/// Baseline TOML covering `UserConfiguration`'s mandatory fields. Injected
/// in front of every test's safety-specific block so each test only has
/// to spell out what it cares about.
const BASELINE: &str = "safety_level = \"moderate\"\n\
                        log_level = \"info\"\n\
                        cache_max_size_gb = 10\n\
                        log_rotation_days = 7\n\n";

/// Helper: build a `SafetyValidator` from a TOML config string written to a
/// temp dir as `config.toml`. The mandatory `UserConfiguration` baseline
/// is auto-prepended. Optional sibling `patterns.toml` content is written
/// when provided.
fn validator_from_toml(
    config_toml: &str,
    patterns_toml: Option<&str>,
) -> Result<SafetyValidator, String> {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    let full = format!("{BASELINE}{config_toml}");
    std::fs::write(&config_path, full).unwrap();

    if let Some(pt) = patterns_toml {
        std::fs::write(tmp.path().join("patterns.toml"), pt).unwrap();
    }

    let manager =
        ConfigManager::with_config_path(config_path).map_err(|e| format!("manager: {e}"))?;
    let user_config = manager.load().map_err(|e| format!("load: {e}"))?;
    let safety_config = SafetyConfig::from_user_config(&user_config, manager.config_path());
    SafetyValidator::new(safety_config).map_err(|e| format!("validator: {e}"))
}

#[tokio::test]
async fn custom_pattern_from_inline_config_blocks_command() {
    let toml = r#"
[[safety.custom_patterns]]
pattern = 'kubectl\s+delete\s+(-n\s+prod|--namespace\s+prod)'
risk_level = "high"
description = "Delete in prod namespace"
"#;
    let validator = validator_from_toml(toml, None).expect("validator builds");
    let result = validator
        .validate_command("kubectl delete -n prod pod/api", ShellType::Bash)
        .await
        .expect("validate");

    assert!(!result.allowed, "user pattern should block: {:?}", result);
    assert!(
        result
            .matched_patterns
            .iter()
            .any(|d| d.contains("delete in prod namespace")),
        "expected description match, got: {:?}",
        result.matched_patterns
    );
}

#[tokio::test]
async fn custom_pattern_from_sibling_patterns_toml_blocks_command() {
    let patterns = r#"
[[pattern]]
pattern = 'aws\s+s3\s+rb\s+s3://prod-'
risk_level = "high"
description = "Remove prod S3 bucket"
"#;
    let validator = validator_from_toml("", Some(patterns))
        .expect("validator builds from sibling patterns.toml");
    let result = validator
        .validate_command("aws s3 rb s3://prod-customer-data --force", ShellType::Bash)
        .await
        .expect("validate");

    assert!(
        !result.allowed,
        "sibling pattern should block: {:?}",
        result
    );
}

#[tokio::test]
async fn safe_command_passes_when_no_pattern_matches() {
    let toml = r#"
[[safety.custom_patterns]]
pattern = 'kubectl\s+delete\s+-n\s+prod'
risk_level = "high"
description = "Delete in prod"
"#;
    let validator = validator_from_toml(toml, None).expect("validator builds");
    let result = validator
        .validate_command("ls -la", ShellType::Bash)
        .await
        .expect("validate");

    assert!(result.allowed, "safe command should pass: {:?}", result);
}

#[tokio::test]
async fn user_pattern_severity_capped_at_high() {
    let pat = DangerPattern {
        pattern: "wipe-customer-data".to_string(),
        risk_level: RiskLevel::Critical,
        description: "User pattern claiming Critical".to_string(),
        shell_specific: None,
    };
    let err =
        validate_user_pattern(&pat).expect_err("Critical should be rejected for user pattern");
    assert!(
        err.to_lowercase().contains("critical")
            || err.to_lowercase().contains("severity")
            || err.to_lowercase().contains("risk_level"),
        "error should mention severity cap: {err}"
    );
}

#[tokio::test]
async fn user_pattern_length_capped_at_512_chars() {
    let long_pat = "a".repeat(513);
    let pat = DangerPattern {
        pattern: long_pat,
        risk_level: RiskLevel::High,
        description: "Too long".to_string(),
        shell_specific: None,
    };
    let err = validate_user_pattern(&pat).expect_err("over-512-char pattern should be rejected");
    assert!(
        err.to_lowercase().contains("length") || err.to_lowercase().contains("512"),
        "error should mention length cap: {err}"
    );
}

#[tokio::test]
async fn user_pattern_description_required() {
    let pat = DangerPattern {
        pattern: "echo hi".to_string(),
        risk_level: RiskLevel::High,
        description: String::new(),
        shell_specific: None,
    };
    let err = validate_user_pattern(&pat).expect_err("empty description should be rejected");
    assert!(
        err.to_lowercase().contains("description"),
        "error should mention description: {err}"
    );
}

#[tokio::test]
async fn malformed_regex_surfaces_loudly() {
    let toml = r#"
[[safety.custom_patterns]]
pattern = '['
risk_level = "high"
description = "Bad regex"
"#;
    let result = validator_from_toml(toml, None);
    assert!(
        result.is_err(),
        "malformed regex must surface as an error, not silent skip"
    );
}

#[tokio::test]
async fn user_pattern_cannot_disable_critical_builtin() {
    // Even if the user allowlists `rm -rf /` via a wide regex, the Critical
    // built-in must still block it.
    let toml = r#"
[safety]
allowlist_patterns = ["^rm\\s+"]

[[safety.custom_patterns]]
pattern = 'never-match-anything-XYZZY'
risk_level = "high"
description = "placeholder so the [safety] section is non-trivial"
"#;
    let validator = validator_from_toml(toml, None).expect("validator builds");
    let result = validator
        .validate_command("rm -rf /", ShellType::Bash)
        .await
        .expect("validate");

    assert!(
        !result.allowed,
        "user allowlist must NOT override Critical built-in `rm -rf /`: {:?}",
        result
    );
    assert_eq!(result.risk_level, RiskLevel::Critical);
}

#[tokio::test]
async fn no_user_patterns_means_no_behaviour_change() {
    let validator = validator_from_toml("", None).expect("empty config builds");
    let result = validator
        .validate_command("rm -rf /", ShellType::Bash)
        .await
        .expect("validate");
    assert!(!result.allowed);
    assert_eq!(result.risk_level, RiskLevel::Critical);

    let result = validator
        .validate_command("echo hello", ShellType::Bash)
        .await
        .expect("validate");
    assert!(result.allowed);
}
