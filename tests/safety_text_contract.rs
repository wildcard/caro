//! Safety-text contract: what the user reads about a risky command is
//! stable and unambiguous.
//!
//! Background: docs/research/2026-10-03-legible-output-ste100.md (STE
//! started as a safety tool: one text, one meaning).

use std::collections::{HashMap, HashSet};

use caro::models::{RiskLevel, ShellType};
use caro::safety::{get_patterns_by_risk, SafetyConfig, SafetyValidator};

async fn explanation(command: &str) -> String {
    SafetyValidator::new(SafetyConfig::moderate())
        .expect("validator constructs")
        .validate_command(command, ShellType::Bash)
        .await
        .expect("validator runs")
        .explanation
}

#[tokio::test]
async fn explanation_text_is_the_same_on_every_run() {
    // Regression: risk keywords came from a HashSet, so their order (and
    // the printed text) changed between runs for the same command.
    let command = "sudo rm -rf /";
    let first = explanation(command).await;
    assert!(
        first.contains(", "),
        "need 2+ keywords to test order: {first}"
    );

    let mut seen = HashSet::from([first]);
    for _ in 0..25 {
        seen.insert(explanation(command).await);
    }
    assert_eq!(seen.len(), 1, "explanation changed between runs: {seen:?}");
}

#[cfg(feature = "cve-rules")]
#[tokio::test]
async fn cve_id_is_shown_once() {
    // Regression: the loader prefixed the ID to a description that already
    // contained it: "CVE-2024-3094: xz-utils backdoor trigger (CVE-2024-3094) — ...".
    let result = SafetyValidator::new(SafetyConfig::moderate())
        .expect("validator constructs")
        .validate_command("xz --lzma1=preset=9 file.txt", ShellType::Bash)
        .await
        .expect("validator runs");
    let warning = result
        .warnings
        .iter()
        .find(|w| w.contains("CVE-2024-3094"))
        .expect("a warning cites CVE-2024-3094");
    assert_eq!(warning.matches("CVE-2024-3094").count(), 1, "{warning}");
}

#[test]
fn built_in_pattern_descriptions_are_unique() {
    // Two patterns with the same text give the user no way to tell which
    // one matched.
    let mut by_description: HashMap<&str, usize> = HashMap::new();
    for p in get_patterns_by_risk(RiskLevel::Safe) {
        *by_description.entry(p.description.as_str()).or_default() += 1;
    }
    let duplicates: Vec<_> = by_description.iter().filter(|(_, n)| **n > 1).collect();
    assert!(
        duplicates.is_empty(),
        "duplicate descriptions: {duplicates:?}"
    );
}

#[tokio::test]
async fn tool_annotation_does_not_add_a_risk_type() {
    // Review finding: "(Remove-Item)" in a description matched "remov" and
    // added "removal" next to "deletion" for the same risk.
    let result = SafetyValidator::new(SafetyConfig::moderate())
        .expect("validator constructs")
        .validate_command(r"Remove-Item -Recurse -Force C:\", ShellType::PowerShell)
        .await
        .expect("validator runs");
    assert!(
        !result.explanation.contains("removal"),
        "{}",
        result.explanation
    );
}
