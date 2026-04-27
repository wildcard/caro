//! Variant lifecycle helpers — generation IDs, sibling-platform consistency
//! hints, and lookups across the lock's per-step variants. The
//! adopt/retire/promote logic lives in PR 6 (`#899`); this module is the
//! reusable substrate.

use crate::caroml::lock::{Lock, Variant};
use chrono::{DateTime, Utc};

/// Build a stable, sortable, human-readable generation ID.
///
/// Format: `gen_<YYYY-MM-DD>_<platform>_<suffix>` where `<suffix>` is the
/// existing-count's letter (`a` for the first variant on this platform/day,
/// `b` for the second, etc.). After 26 variants on the same platform-day,
/// rolls over to two letters (`aa`, `ab`, ...).
pub fn generation_id(now: DateTime<Utc>, platform: &str, day_index: usize) -> String {
    let date = now.format("%Y-%m-%d");
    format!("gen_{}_{}_{}", date, platform, alpha_suffix(day_index))
}

fn alpha_suffix(mut n: usize) -> String {
    let mut buf = Vec::new();
    loop {
        let digit = (n % 26) as u8;
        buf.push((b'a' + digit) as char);
        n /= 26;
        if n == 0 {
            break;
        }
        n -= 1;
    }
    buf.iter().rev().collect()
}

/// Build a sibling-platform consistency hint for the LLM prompt.
///
/// Returns a string like:
///
/// ```text
/// On macos for the same intent, the active variant chose:
///   `find /var/log -type f -name '*.log'`
/// Prefer the same shape unless platform-specific reason to differ.
/// ```
///
/// Returns `None` if no sibling platform has an active variant for this step.
pub fn sibling_consistency_hint(
    lock: &Lock,
    step_idx: usize,
    target_platform: &str,
) -> Option<String> {
    let step = lock.steps.get(step_idx)?;
    for variant in &step.variants {
        if variant.platform != target_platform && variant.active {
            return Some(format!(
                "On {} for the same intent, the active variant chose:\n  `{}`\n\
                 Prefer the same shape unless platform-specific reason to differ.",
                variant.platform, variant.command,
            ));
        }
    }
    None
}

/// All challenger variants across all steps for a given platform.
pub fn all_challengers_for<'a>(
    lock: &'a Lock,
    platform: &'a str,
) -> impl Iterator<Item = (usize, &'a Variant)> + 'a {
    lock.steps
        .iter()
        .enumerate()
        .flat_map(move |(idx, s)| s.challengers(platform).map(move |v| (idx, v)))
}

/// Count of active variants for `platform` across all steps.
pub fn active_count_for(lock: &Lock, platform: &str) -> usize {
    lock.steps
        .iter()
        .filter(|s| s.active_variant(platform).is_some())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caroml::lock::{Lock, Step as LockStep, Variant};
    use chrono::TimeZone;
    use std::collections::BTreeMap;

    fn variant(platform: &str, active: bool, command: &str, gen_id: &str) -> Variant {
        Variant {
            platform: platform.into(),
            active,
            generation_id: gen_id.into(),
            command: command.into(),
            reasoning: "".into(),
            exports: vec![],
            imports: vec![],
            risk_level: "safe".into(),
            matched_patterns: vec![],
            warnings: vec![],
            confidence: 1.0,
            iterations: 1,
            validations: vec![],
            generated_at: Utc::now(),
            model: "m".into(),
            backend: "b".into(),
            tool_versions: BTreeMap::new(),
            track_record: Default::default(),
            retired_at: None,
            runbook_hash: String::new(),
        }
    }

    #[test]
    fn alpha_suffix_basic() {
        assert_eq!(alpha_suffix(0), "a");
        assert_eq!(alpha_suffix(1), "b");
        assert_eq!(alpha_suffix(25), "z");
        assert_eq!(alpha_suffix(26), "aa");
        assert_eq!(alpha_suffix(27), "ab");
        assert_eq!(alpha_suffix(51), "az");
        assert_eq!(alpha_suffix(52), "ba");
    }

    #[test]
    fn generation_id_is_well_formed() {
        let when = Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap();
        assert_eq!(generation_id(when, "macos", 0), "gen_2026-04-26_macos_a");
        assert_eq!(generation_id(when, "linux", 1), "gen_2026-04-26_linux_b");
    }

    #[test]
    fn sibling_hint_returns_none_when_no_sibling() {
        let mut lock = Lock::default();
        lock.steps = vec![LockStep {
            line: 1,
            intent: "x".into(),
            intent_hash: "h".into(),
            notes: vec![],
            variants: vec![variant("macos", true, "ls", "gen_a")],
        }];
        assert!(sibling_consistency_hint(&lock, 0, "macos").is_none());
    }

    #[test]
    fn sibling_hint_returns_other_platform_command() {
        let mut lock = Lock::default();
        lock.steps = vec![LockStep {
            line: 1,
            intent: "x".into(),
            intent_hash: "h".into(),
            notes: vec![],
            variants: vec![
                variant("macos", true, "ls -G", "gen_a"),
                variant("linux", true, "ls --color", "gen_b"),
            ],
        }];
        let hint = sibling_consistency_hint(&lock, 0, "linux").unwrap();
        assert!(hint.contains("On macos"));
        assert!(hint.contains("ls -G"));
    }

    #[test]
    fn all_challengers_for_filters_correctly() {
        let mut lock = Lock::default();
        lock.steps = vec![LockStep {
            line: 1,
            intent: "x".into(),
            intent_hash: "h".into(),
            notes: vec![],
            variants: vec![
                variant("macos", true, "active", "gen_a"),
                variant("macos", false, "challenger", "gen_b"),
                variant("linux", true, "linux-active", "gen_c"),
            ],
        }];
        let challengers: Vec<_> = all_challengers_for(&lock, "macos").collect();
        assert_eq!(challengers.len(), 1);
        assert_eq!(challengers[0].1.command, "challenger");
    }

    #[test]
    fn active_count_matches_per_platform() {
        let mut lock = Lock::default();
        lock.steps = vec![
            LockStep {
                line: 1,
                intent: "a".into(),
                intent_hash: "h1".into(),
                notes: vec![],
                variants: vec![variant("macos", true, "x", "gen_a")],
            },
            LockStep {
                line: 2,
                intent: "b".into(),
                intent_hash: "h2".into(),
                notes: vec![],
                variants: vec![variant("macos", true, "y", "gen_b")],
            },
            LockStep {
                line: 3,
                intent: "c".into(),
                intent_hash: "h3".into(),
                notes: vec![],
                variants: vec![],
            },
        ];
        assert_eq!(active_count_for(&lock, "macos"), 2);
        assert_eq!(active_count_for(&lock, "linux"), 0);
    }
}
