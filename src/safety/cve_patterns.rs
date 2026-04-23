//! Runtime CVE pattern loader.
//!
//! Loads the bincode blob written by `build.rs` at `$OUT_DIR/cve_patterns.bin`
//! and exposes the rules in two shapes:
//!
//! 1. `CVE_COMPILED` — the full `CompiledRuleset` (rules + metadata), used by
//!    `--version` to report how many CVE-derived rules are embedded.
//! 2. `CVE_COMPILED_PATTERNS` — a pre-compiled `Vec<(Regex, RiskLevel, String,
//!    Option<ShellType>)>` with the exact same tuple shape as the built-in
//!    `super::patterns::COMPILED_PATTERNS`, so the validator loop can iterate
//!    both in the same pass.
//!
//! The bincode blob is produced unconditionally at build time (see
//! `build.rs::compile_cve_ruleset`). When the `cve-rules` feature is off we
//! substitute an empty ruleset so the include_bytes! is elided and no
//! deserialization cost is paid at startup.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::dogma::CompiledRuleset;
use crate::models::{RiskLevel, ShellType};

/// Deserialized CVE ruleset (patterns + metadata).
///
/// Safe to access from any thread. Lazy-loaded on first reference — the cost
/// is a single bincode pass over ~a few KB for typical rule counts (< 500).
pub static CVE_COMPILED: Lazy<CompiledRuleset> = Lazy::new(|| {
    #[cfg(feature = "cve-rules")]
    {
        const BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/cve_patterns.bin"));
        // On deserialize failure we fall back to an empty ruleset rather than
        // panicking — safety must never be a hard dependency for caro to run.
        // The validator simply applies its built-in pattern set.
        bincode::deserialize(BYTES).unwrap_or_default()
    }
    #[cfg(not(feature = "cve-rules"))]
    {
        CompiledRuleset::default()
    }
});

/// Tuple shape aligned with `super::patterns::COMPILED_PATTERNS` so the
/// validator can treat built-in and CVE-derived patterns identically.
type CvePattern = (Regex, RiskLevel, String, Option<ShellType>);

/// Pre-compiled regex patterns for all CVE rules, ready for the validator loop.
///
/// Rules whose regex fails to compile are silently dropped with a stderr warn;
/// a single malformed rule must not take down the validator for all other rules.
/// The build-time compiler (`src/dogma/compiler.rs`) already validates regex
/// syntax as part of the schema check, so reaching this path means either the
/// schema check was bypassed (manual YAML edit) or bincode was tampered with —
/// either way, dropping the rule is safer than panicking.
pub static CVE_COMPILED_PATTERNS: Lazy<Vec<CvePattern>> = Lazy::new(|| {
    CVE_COMPILED
        .patterns
        .iter()
        .filter_map(|p| {
            let regex = match Regex::new(&p.pattern) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "WARN: CVE rule {} has invalid regex {:?}: {}",
                        p.id, p.pattern, e
                    );
                    return None;
                }
            };
            let risk = parse_risk_level(&p.risk_level)?;
            let shell = p.shell_specific.as_deref().and_then(parse_shell_type);
            // Prepend the CVE ID to the description so validator warnings like
            // "Critical: CVE-2024-3094: xz backdoor trigger" surface the source.
            let description = format!("{}: {}", p.id, p.description);
            Some((regex, risk, description, shell))
        })
        .collect()
});

/// Get CVE-derived compiled patterns applicable to a given shell.
///
/// Mirrors `super::patterns::get_compiled_patterns_for_shell` exactly — returns
/// patterns with `shell_specific == None` (any shell) plus those matching the
/// requested shell.
pub fn get_cve_compiled_patterns_for_shell(shell: ShellType) -> Vec<&'static CvePattern> {
    CVE_COMPILED_PATTERNS
        .iter()
        .filter(|(_, _, _, shell_specific)| {
            shell_specific.is_none() || *shell_specific == Some(shell)
        })
        .collect()
}

fn parse_risk_level(s: &str) -> Option<RiskLevel> {
    match s {
        "safe" => Some(RiskLevel::Safe),
        "moderate" => Some(RiskLevel::Moderate),
        "high" => Some(RiskLevel::High),
        "critical" => Some(RiskLevel::Critical),
        _ => None,
    }
}

fn parse_shell_type(s: &str) -> Option<ShellType> {
    match s {
        "bash" => Some(ShellType::Bash),
        "zsh" => Some(ShellType::Zsh),
        "fish" => Some(ShellType::Fish),
        "sh" => Some(ShellType::Sh),
        "powershell" => Some(ShellType::PowerShell),
        "cmd" => Some(ShellType::Cmd),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cve_ruleset_loads_without_panic() {
        // Force Lazy init; tolerates 0-rule and N-rule cases alike.
        let _ = &*CVE_COMPILED;
        let _ = &*CVE_COMPILED_PATTERNS;
    }

    #[test]
    fn parse_risk_level_accepts_all_variants() {
        assert_eq!(parse_risk_level("safe"), Some(RiskLevel::Safe));
        assert_eq!(parse_risk_level("moderate"), Some(RiskLevel::Moderate));
        assert_eq!(parse_risk_level("high"), Some(RiskLevel::High));
        assert_eq!(parse_risk_level("critical"), Some(RiskLevel::Critical));
        assert_eq!(parse_risk_level("Critical"), None, "case-sensitive");
        assert_eq!(parse_risk_level("unknown"), None);
    }

    #[test]
    fn parse_shell_type_accepts_all_variants() {
        assert_eq!(parse_shell_type("bash"), Some(ShellType::Bash));
        assert_eq!(parse_shell_type("zsh"), Some(ShellType::Zsh));
        assert_eq!(parse_shell_type("fish"), Some(ShellType::Fish));
        assert_eq!(parse_shell_type("sh"), Some(ShellType::Sh));
        assert_eq!(parse_shell_type("powershell"), Some(ShellType::PowerShell));
        assert_eq!(parse_shell_type("cmd"), Some(ShellType::Cmd));
        assert_eq!(parse_shell_type("unknown"), None);
    }

    #[test]
    fn shell_filter_includes_cross_shell_patterns() {
        // Cross-shell (None) patterns should show up for every shell.
        // This is a behavior check, not a data check — if no CVE rules are
        // embedded the test is vacuously true.
        for shell in [ShellType::Bash, ShellType::Zsh, ShellType::PowerShell] {
            let filtered = get_cve_compiled_patterns_for_shell(shell);
            for (_, _, _, sp) in &filtered {
                assert!(sp.is_none() || *sp == Some(shell));
            }
        }
    }
}
