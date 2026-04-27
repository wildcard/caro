//! Multi-angle validator framework.
//!
//! Per-step generation in the CaroML interpreter (PR 4) runs the generated
//! command through a stack of [`Validator`]s, each focused on a different
//! "angle" — safety, platform compatibility, secrets exposure, side-effect
//! surfacing, and so on. Validators run in parallel for a single step
//! (they're independent and side-effect-free); their outcomes drive both
//! the lock's per-step `validations` field and the repair-hint feedback
//! that's fed into the next LLM iteration if any validator fails.
//!
//! # v0.1 angles (this PR)
//!
//! - [`safety::SafetyAngle`] — wraps the existing 52+ pattern + CVE matcher
//! - [`platform::PlatformAngle`] — BSD-vs-GNU flag heuristics from `CapabilityProfile`
//! - [`secrets::SecretsAngle`] — regex scan for hard-coded credentials
//! - [`side_effects::SideEffectsAngle`] — warn-only stub flagging sudo/network/destructive ops
//!
//! v0.2 will add `idempotency`, `reversibility`, `resource_impact`, and the
//! external-validator hook (path to a binary that reads JSON on stdin / writes JSON on stdout).

use async_trait::async_trait;

pub mod platform;
pub mod safety;
pub mod secrets;
pub mod side_effects;

pub use platform::PlatformAngle;
pub use safety::SafetyAngle;
pub use secrets::SecretsAngle;
pub use side_effects::SideEffectsAngle;

// ---------------------------------------------------------------------------
// Trait + types
// ---------------------------------------------------------------------------

/// Inputs needed by a validator. Borrowed for cheap parallel dispatch.
pub struct ValidatorContext<'a> {
    /// The shell command being validated.
    pub command: &'a str,
    /// The natural-language intent that produced this command (for repair hints).
    pub intent: &'a str,
    /// Title of the parent task (for context in error messages).
    pub task_title: &'a str,
    /// Target platform: `"macos"` / `"linux"` / `"windows"` / `"posix"`.
    pub platform: &'a str,
    /// True iff the task explicitly declared `NEED sudo` (lets `side_effects`
    /// not flag sudo as a finding when it was opted into).
    pub sudo_declared: bool,
    /// Optional capability profile (drives the `platform` validator's heuristics).
    pub capability_profile: Option<&'a crate::prompts::CapabilityProfile>,
}

/// One validator's verdict for a single command.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationOutcome {
    pub angle: String,
    pub result: Verdict,
    /// Free-form note for human display (`caro why` / `caro check` output).
    pub note: Option<String>,
    /// Hint fed back into the next LLM iteration if `result` is `Fail` or `Warn`.
    pub repair_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
}

impl ValidationOutcome {
    pub fn pass(angle: &'static str) -> Self {
        Self {
            angle: angle.to_string(),
            result: Verdict::Pass,
            note: None,
            repair_hint: None,
        }
    }

    pub fn warn(angle: &'static str, note: impl Into<String>) -> Self {
        Self {
            angle: angle.to_string(),
            result: Verdict::Warn,
            note: Some(note.into()),
            repair_hint: None,
        }
    }

    pub fn fail(angle: &'static str, note: impl Into<String>, repair: impl Into<String>) -> Self {
        Self {
            angle: angle.to_string(),
            result: Verdict::Fail,
            note: Some(note.into()),
            repair_hint: Some(repair.into()),
        }
    }
}

/// A validator focuses on one "angle" of correctness.
#[async_trait]
pub trait Validator: Send + Sync {
    /// Stable string identifier (`"safety"` / `"platform"` / …).
    fn angle(&self) -> &'static str;

    /// Whether a `Fail` from this validator should block the loop entirely
    /// (rather than just trigger a repair iteration). Default `false`.
    /// `safety` is the only `must_pass = true` validator in v0.1.
    fn must_pass(&self) -> bool {
        false
    }

    async fn validate(&self, ctx: &ValidatorContext<'_>) -> ValidationOutcome;
}

/// Run every validator in `chain` against `ctx` concurrently and collect
/// the outcomes in chain order.
pub async fn run_all(
    chain: &[Box<dyn Validator>],
    ctx: &ValidatorContext<'_>,
) -> Vec<ValidationOutcome> {
    use futures::future::join_all;
    let futures = chain.iter().map(|v| v.validate(ctx));
    join_all(futures).await
}

/// True iff every outcome in `outcomes` is `Pass` (warnings count as not-passing for the loop).
pub fn all_pass(outcomes: &[ValidationOutcome]) -> bool {
    outcomes.iter().all(|o| o.result == Verdict::Pass)
}

/// Whether the loop should give up entirely — i.e. any `must_pass` validator
/// returned `Fail`. Returns the first such outcome's angle for diagnostics.
pub fn fatal_angle(chain: &[Box<dyn Validator>], outcomes: &[ValidationOutcome]) -> Option<String> {
    chain
        .iter()
        .zip(outcomes.iter())
        .find(|(v, o)| v.must_pass() && o.result == Verdict::Fail)
        .map(|(_, o)| o.angle.clone())
}

// ---------------------------------------------------------------------------
// Default v0.1 validator chain
// ---------------------------------------------------------------------------

/// Build the default v0.1 chain: `safety` (must-pass) → `platform` → `secrets` → `side_effects`.
pub fn default_chain() -> Vec<Box<dyn Validator>> {
    vec![
        Box::new(SafetyAngle::default()),
        Box::new(PlatformAngle),
        Box::new(SecretsAngle),
        Box::new(SideEffectsAngle),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_for<'a>(command: &'a str) -> ValidatorContext<'a> {
        ValidatorContext {
            command,
            intent: "test intent",
            task_title: "test task",
            platform: "macos",
            sudo_declared: false,
            capability_profile: None,
        }
    }

    #[tokio::test]
    async fn default_chain_runs_all_four_angles() {
        let chain = default_chain();
        assert_eq!(chain.len(), 4);
        let angles: Vec<&str> = chain.iter().map(|v| v.angle()).collect();
        assert_eq!(
            angles,
            vec!["safety", "platform", "secrets", "side_effects"]
        );
    }

    #[tokio::test]
    async fn run_all_returns_outcomes_in_chain_order() {
        let chain = default_chain();
        let ctx = ctx_for("ls -la");
        let outcomes = run_all(&chain, &ctx).await;
        assert_eq!(outcomes.len(), 4);
        assert_eq!(outcomes[0].angle, "safety");
        assert_eq!(outcomes[1].angle, "platform");
        assert_eq!(outcomes[2].angle, "secrets");
        assert_eq!(outcomes[3].angle, "side_effects");
    }

    #[tokio::test]
    async fn all_pass_is_true_when_outcomes_all_pass() {
        let outcomes = vec![
            ValidationOutcome::pass("safety"),
            ValidationOutcome::pass("platform"),
        ];
        assert!(all_pass(&outcomes));
    }

    #[tokio::test]
    async fn all_pass_is_false_when_any_warn() {
        let outcomes = vec![
            ValidationOutcome::pass("safety"),
            ValidationOutcome::warn("platform", "BSD detected, GNU flag used"),
        ];
        assert!(!all_pass(&outcomes));
    }

    #[tokio::test]
    async fn fatal_angle_identifies_must_pass_failure() {
        let chain: Vec<Box<dyn Validator>> = vec![
            Box::new(SafetyAngle::default()), // must_pass = true
            Box::new(SideEffectsAngle),
        ];
        let outcomes = vec![
            ValidationOutcome::fail("safety", "blocked", "use a less destructive approach"),
            ValidationOutcome::pass("side_effects"),
        ];
        assert_eq!(fatal_angle(&chain, &outcomes), Some("safety".to_string()));
    }

    #[tokio::test]
    async fn fatal_angle_ignores_non_must_pass_failure() {
        let chain: Vec<Box<dyn Validator>> =
            vec![Box::new(SafetyAngle::default()), Box::new(SideEffectsAngle)];
        let outcomes = vec![
            ValidationOutcome::pass("safety"),
            ValidationOutcome::fail("side_effects", "writes /etc", "scope to /tmp"),
        ];
        assert_eq!(fatal_angle(&chain, &outcomes), None);
    }
}
