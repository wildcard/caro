//! End-to-end proof that CVE rules compiled from `data/cve_rules/*.yaml`
//! actually block commands through the runtime validator.
//!
//! This test is the acceptance bar for spec 010 Phase 2: an edit to any
//! CVE YAML on disk must translate into validator behavior without any
//! other code touch.

use caro::models::{RiskLevel, SafetyLevel, ShellType};
use caro::safety::{SafetyConfig, SafetyValidator, CVE_COMPILED};

/// The canonical xz-utils backdoor trigger — the pattern that motivated
/// this whole pipeline. Covered by `data/cve_rules/CVE-2024-3094.yaml`.
#[tokio::test]
async fn cve_2024_3094_blocks_xz_lzma1_preset_9() {
    let validator = SafetyValidator::new(SafetyConfig::moderate())
        .expect("validator constructs with default moderate safety");

    let result = validator
        .validate_command("xz --lzma1=preset=9 file.txt", ShellType::Bash)
        .await
        .expect("validator runs without error");

    assert!(
        !result.allowed,
        "xz backdoor trigger must be blocked, got {:?}",
        result
    );
    assert_eq!(result.risk_level, RiskLevel::Critical);
    assert!(
        result
            .matched_patterns
            .iter()
            .any(|m| m.contains("cve-2024-3094")),
        "expected matched_patterns to cite CVE-2024-3094 for provenance, got {:?}",
        result.matched_patterns
    );
}

/// Benign xz usage must stay allowed — proves the pattern is specific
/// enough not to regress normal compression workflows.
#[tokio::test]
async fn cve_2024_3094_allows_plain_xz_compression() {
    let validator = SafetyValidator::new(SafetyConfig::moderate()).unwrap();

    let result = validator
        .validate_command("xz -z file.txt", ShellType::Bash)
        .await
        .unwrap();

    assert!(
        result.allowed,
        "plain xz compression must not be blocked by CVE-2024-3094 rule, got {:?}",
        result
    );
    assert_eq!(result.risk_level, RiskLevel::Safe);
}

/// Sanity check: the compiled blob actually ships non-zero rules when the
/// `cve-rules` feature is active (default). If this fires we've silently
/// lost CVE coverage somewhere in the build pipeline.
#[cfg(feature = "cve-rules")]
#[test]
fn cve_compiled_blob_is_non_empty() {
    assert!(
        !CVE_COMPILED.patterns.is_empty(),
        "cve-rules feature is on but compiled ruleset is empty — build.rs didn't pick up data/cve_rules/*.yaml"
    );
    assert_eq!(
        CVE_COMPILED.patterns.len(),
        CVE_COMPILED.metadata.rule_count,
        "ruleset and metadata rule_count disagree"
    );
}

/// Probe the existing built-in pattern path still works — this is a
/// regression guard for the validator wiring change (adding a third
/// pattern source must not break the first two).
#[tokio::test]
async fn builtin_rm_rf_root_still_blocks() {
    let validator = SafetyValidator::new(SafetyConfig::from_level(SafetyLevel::Moderate)).unwrap();

    let result = validator
        .validate_command("rm -rf /", ShellType::Bash)
        .await
        .unwrap();

    assert!(
        !result.allowed,
        "built-in rm -rf / pattern must still block after CVE wiring, got {:?}",
        result
    );
    assert_eq!(result.risk_level, RiskLevel::Critical);
}
