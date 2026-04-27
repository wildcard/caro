//! Adopt / experiment / retire — the variant-lifecycle operations called
//! by `caro adopt`, `caro experiment`, and (later) `caro adopt --auto-best`.
//!
//! - `adopt(lock, variant_id)`: flip a challenger to active, retire the
//!   previously-active variant (sets `retired_at = now`).
//! - `find_variant_by_id(lock, variant_id)`: resolve a generation_id to its
//!   `(step_index, platform, &Variant)` triple.
//! - `aggregated_score`: lightweight score function used by `--auto-best`
//!   in the future; ships now so PR 6 docs can reference its semantics.

use crate::caroml::lock::{Lock, TrackRecord, Variant};
use chrono::Utc;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdoptError {
    #[error("variant `{0}` not found in lock")]
    NotFound(String),
    #[error("variant `{0}` is already active for platform `{1}`")]
    AlreadyActive(String, String),
    #[error("variant `{0}` has been retired and cannot be re-adopted")]
    Retired(String),
}

/// Promote `variant_id` to active for its platform, retiring whichever
/// variant was previously active for that platform.
pub fn adopt(lock: &mut Lock, variant_id: &str) -> Result<(), AdoptError> {
    // First, locate the target.
    let (step_idx, platform) = locate(lock, variant_id)?;

    // Validate state.
    {
        let step = &lock.steps[step_idx];
        let target = step
            .variants
            .iter()
            .find(|v| v.generation_id == variant_id)
            .expect("locate guarantees the variant exists");
        if target.retired_at.is_some() {
            return Err(AdoptError::Retired(variant_id.to_string()));
        }
        if target.active {
            return Err(AdoptError::AlreadyActive(
                variant_id.to_string(),
                platform.clone(),
            ));
        }
    }

    let now = Utc::now();
    let step = &mut lock.steps[step_idx];
    for v in &mut step.variants {
        if v.platform != platform {
            continue;
        }
        if v.active {
            v.active = false;
            v.retired_at = Some(now);
        }
        if v.generation_id == variant_id {
            v.active = true;
            v.retired_at = None;
        }
    }
    Ok(())
}

/// Locate a variant by id; returns `(step_idx, platform_string)`.
pub fn locate(lock: &Lock, variant_id: &str) -> Result<(usize, String), AdoptError> {
    for (idx, step) in lock.steps.iter().enumerate() {
        for v in &step.variants {
            if v.generation_id == variant_id {
                return Ok((idx, v.platform.clone()));
            }
        }
    }
    Err(AdoptError::NotFound(variant_id.to_string()))
}

/// Crude score function for adopt suggestions.
///
/// Returns a number in `[0.0, 1.0]`:
/// - `1.0` if `runs == 0` (treated as "no signal yet, defer to confidence")
///   weighted by `confidence`.
/// - else `succeeded / runs`.
pub fn aggregated_score(variant: &Variant, tr: &TrackRecord) -> f32 {
    if tr.runs == 0 {
        variant.confidence.clamp(0.0, 1.0)
    } else {
        (tr.succeeded as f32) / (tr.runs as f32)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caroml::lock::{Lock, Meta, Step as LockStep, Variant};
    use chrono::Utc;
    use std::collections::BTreeMap;

    fn variant(platform: &str, active: bool, gen_id: &str, command: &str) -> Variant {
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
            confidence: 0.9,
            iterations: 1,
            validations: vec![],
            generated_at: Utc::now(),
            model: "m".into(),
            backend: "b".into(),
            tool_versions: BTreeMap::new(),
            track_record: Default::default(),
            retired_at: None,
        }
    }

    fn lock_with_active_and_challenger() -> Lock {
        let mut lock = Lock::default();
        lock.meta = Meta {
            caro_version: "1.4.0".into(),
            intent_path: "tasks/x.caro".into(),
            intent_hash: "sha256:x".into(),
            supported_platforms: vec!["macos".into()],
            last_full_regen: Some(Utc::now()),
        };
        lock.steps = vec![LockStep {
            line: 1,
            intent: "demo".into(),
            intent_hash: "sha256:s".into(),
            notes: vec![],
            variants: vec![
                variant("macos", true, "gen_a", "ls -la"),
                variant("macos", false, "gen_b", "ls -lA"),
            ],
        }];
        lock
    }

    #[test]
    fn adopt_promotes_challenger_and_retires_active() {
        let mut lock = lock_with_active_and_challenger();
        adopt(&mut lock, "gen_b").unwrap();
        let variants = &lock.steps[0].variants;
        let a = variants
            .iter()
            .find(|v| v.generation_id == "gen_a")
            .unwrap();
        let b = variants
            .iter()
            .find(|v| v.generation_id == "gen_b")
            .unwrap();
        assert!(!a.active);
        assert!(a.retired_at.is_some());
        assert!(b.active);
        assert!(b.retired_at.is_none());
    }

    #[test]
    fn adopt_already_active_is_error() {
        let mut lock = lock_with_active_and_challenger();
        match adopt(&mut lock, "gen_a") {
            Err(AdoptError::AlreadyActive(_, _)) => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn adopt_unknown_variant_is_error() {
        let mut lock = lock_with_active_and_challenger();
        match adopt(&mut lock, "gen_zzz") {
            Err(AdoptError::NotFound(id)) if id == "gen_zzz" => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn adopt_retired_variant_is_error() {
        let mut lock = lock_with_active_and_challenger();
        // retire `gen_b`
        if let Some(v) = lock.steps[0]
            .variants
            .iter_mut()
            .find(|v| v.generation_id == "gen_b")
        {
            v.retired_at = Some(Utc::now());
        }
        match adopt(&mut lock, "gen_b") {
            Err(AdoptError::Retired(_)) => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn aggregated_score_uses_confidence_when_no_runs() {
        let v = variant("macos", false, "gen_x", "ls");
        let tr = TrackRecord::default();
        let score = aggregated_score(&v, &tr);
        assert!((score - 0.9).abs() < 1e-6);
    }

    #[test]
    fn aggregated_score_uses_success_rate_with_runs() {
        let v = variant("macos", false, "gen_x", "ls");
        let tr = TrackRecord {
            runs: 4,
            succeeded: 3,
            failed: 1,
            last_used: None,
        };
        let score = aggregated_score(&v, &tr);
        assert!((score - 0.75).abs() < 1e-6);
    }

    #[test]
    fn locate_returns_step_index_and_platform() {
        let lock = lock_with_active_and_challenger();
        let (idx, plat) = locate(&lock, "gen_b").unwrap();
        assert_eq!(idx, 0);
        assert_eq!(plat, "macos");
    }
}
