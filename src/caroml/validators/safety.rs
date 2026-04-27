//! Safety angle — wraps the existing 52+ pattern + CVE matcher
//! ([`crate::safety::SafetyValidator`]).
//!
//! `must_pass = true`: a `Fail` from this validator stops the generation loop
//! (no further repair iterations). The other v0.1 angles are advisory and
//! merely trigger another iteration with their `repair_hint` fed back.

use async_trait::async_trait;
use std::sync::Arc;

use crate::caroml::validators::{ValidationOutcome, Validator, ValidatorContext, Verdict};
use crate::models::{RiskLevel, ShellType};
use crate::safety::{SafetyConfig, SafetyValidator};

pub struct SafetyAngle {
    inner: Arc<SafetyValidator>,
    /// Map a CaroML `platform` field to a `ShellType` for the wrapped validator.
    /// macOS / Linux / POSIX → Bash; Windows → PowerShell.
    fallback_shell: ShellType,
}

impl Default for SafetyAngle {
    fn default() -> Self {
        let inner = SafetyValidator::new(SafetyConfig::moderate())
            .expect("default SafetyConfig::moderate must produce a valid SafetyValidator");
        Self {
            inner: Arc::new(inner),
            fallback_shell: ShellType::Bash,
        }
    }
}

impl SafetyAngle {
    pub fn new(inner: Arc<SafetyValidator>) -> Self {
        Self {
            inner,
            fallback_shell: ShellType::Bash,
        }
    }

    fn shell_for(&self, platform: &str) -> ShellType {
        match platform {
            "windows" => ShellType::PowerShell,
            _ => self.fallback_shell,
        }
    }
}

#[async_trait]
impl Validator for SafetyAngle {
    fn angle(&self) -> &'static str {
        "safety"
    }

    fn must_pass(&self) -> bool {
        true
    }

    async fn validate(&self, ctx: &ValidatorContext<'_>) -> ValidationOutcome {
        let shell = self.shell_for(ctx.platform);
        let result = match self.inner.validate_command(ctx.command, shell).await {
            Ok(r) => r,
            Err(e) => {
                return ValidationOutcome::fail(
                    "safety",
                    format!("validator error: {}", e),
                    "the safety validator could not run; this should not happen",
                )
            }
        };

        if !result.allowed {
            return ValidationOutcome::fail(
                "safety",
                format!(
                    "{} (risk: {:?}, matched: {})",
                    result.explanation,
                    result.risk_level,
                    result.matched_patterns.join(", "),
                ),
                "rewrite the command without the dangerous pattern; \
                 prefer a least-privilege approach (read-only, scoped paths, \
                 explicit confirmations)",
            );
        }

        match result.risk_level {
            RiskLevel::Safe => ValidationOutcome::pass("safety"),
            RiskLevel::Moderate => {
                let warnings = if result.warnings.is_empty() {
                    "moderate risk".to_string()
                } else {
                    result.warnings.join("; ")
                };
                ValidationOutcome {
                    angle: "safety".to_string(),
                    result: Verdict::Pass, // moderate is allowed, surface as note
                    note: Some(warnings),
                    repair_hint: None,
                }
            }
            RiskLevel::High | RiskLevel::Critical => ValidationOutcome::fail(
                "safety",
                format!("risk={:?}: {}", result.risk_level, result.explanation),
                "downgrade the action — narrower scope, dry-run mode, or a safer tool",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(command: &'a str) -> ValidatorContext<'a> {
        ValidatorContext {
            command,
            intent: "test",
            task_title: "test",
            platform: "macos",
            sudo_declared: false,
            capability_profile: None,
        }
    }

    #[tokio::test]
    async fn safe_command_passes() {
        let v = SafetyAngle::default();
        let outcome = v.validate(&ctx("ls -la")).await;
        assert_eq!(outcome.angle, "safety");
        assert_eq!(outcome.result, Verdict::Pass);
    }

    #[tokio::test]
    async fn dangerous_command_fails_with_repair_hint() {
        let v = SafetyAngle::default();
        let outcome = v.validate(&ctx("rm -rf /")).await;
        assert_eq!(outcome.result, Verdict::Fail);
        assert!(outcome.repair_hint.is_some());
        assert!(outcome.note.is_some());
    }

    #[tokio::test]
    async fn safety_angle_must_pass() {
        assert!(SafetyAngle::default().must_pass());
    }
}
