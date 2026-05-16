//! Hydrators — populate [`CandidateFeatures`] from candidate text.
//!
//! Phase 2 ships two:
//!
//! - [`PlatformFitHydrator`] reuses the BSD/GNU heuristics from
//!   [`crate::agent::AgentLoop::should_refine`] (`src/agent/mod.rs:720`) and
//!   maps a hit to a [0, 1] `platform_fit` score. The logic is intentionally
//!   identical so behavior is preserved when the pipeline is wired in.
//! - [`SafetyHydrator`] calls [`crate::safety::SafetyValidator::validate_command`]
//!   and writes `safety_confidence` + `risk_level` onto the candidate. The
//!   [`super::SafetyFilter`] later consumes `risk_level` to drop Critical
//!   candidates — keeping the validator call here means the network of
//!   filters never duplicates a (potentially expensive) regex sweep.

use async_trait::async_trait;
use std::sync::Arc;

use super::{Candidate, Hydrator};
use crate::models::ShellType;
use crate::safety::SafetyValidator;

/// Platform-fit heuristic — penalizes BSD/GNU flag mismatches. Mirrors the
/// existing `should_refine` regexes so wiring this in is behavior-preserving.
pub struct PlatformFitHydrator {
    os: String,
}

impl PlatformFitHydrator {
    pub fn new(os: impl Into<String>) -> Self {
        Self { os: os.into() }
    }

    fn score(&self, cmd: &str) -> f32 {
        match self.os.as_str() {
            // macOS uses BSD tools — GNU long flags and Linux-only commands
            // are penalized exactly the same way `should_refine` flags them.
            "macos" => {
                let has_gnu_long_flag = cmd.contains("--sort") || cmd.contains("--max-depth");
                let uses_linux_only = cmd.contains("ss ") || cmd.starts_with("find /");
                if has_gnu_long_flag || uses_linux_only {
                    0.3
                } else {
                    1.0
                }
            }
            // Linux happily takes both BSD and GNU — full fit.
            _ => 1.0,
        }
    }
}

#[async_trait]
impl Hydrator for PlatformFitHydrator {
    async fn hydrate(&self, c: &mut Candidate) {
        c.features.platform_fit = self.score(&c.command);
    }
    fn name(&self) -> &str {
        "platform-fit"
    }
}

/// Runs [`SafetyValidator::validate_command`] and writes the result onto
/// the candidate's features. A separate [`super::SafetyFilter`] consumes
/// the `risk_level` to reject Critical hits.
pub struct SafetyHydrator {
    validator: Arc<SafetyValidator>,
    shell: ShellType,
}

impl SafetyHydrator {
    pub fn new(validator: Arc<SafetyValidator>) -> Self {
        Self {
            validator,
            shell: ShellType::Bash,
        }
    }

    pub fn with_shell(mut self, shell: ShellType) -> Self {
        self.shell = shell;
        self
    }
}

#[async_trait]
impl Hydrator for SafetyHydrator {
    async fn hydrate(&self, c: &mut Candidate) {
        match self.validator.validate_command(&c.command, self.shell).await {
            Ok(result) => {
                c.features.safety_confidence = result.confidence_score;
                c.features.risk_level = Some(result.risk_level);
            }
            Err(_) => {
                // Validator errored; treat as unknown — leave defaults but
                // mark risk as Critical so the SafetyFilter rejects.
                c.features.risk_level = Some(crate::models::RiskLevel::Critical);
                c.features.safety_confidence = 0.0;
            }
        }
    }

    fn name(&self) -> &str {
        "safety"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RiskLevel;
    use crate::safety::SafetyConfig;

    fn cand(cmd: &str) -> Candidate {
        Candidate::new(cmd, "test")
    }

    #[tokio::test]
    async fn macos_gnu_long_flag_penalized() {
        let h = PlatformFitHydrator::new("macos");
        let mut c = cand("ps aux --sort=-pcpu | head");
        h.hydrate(&mut c).await;
        assert!(c.features.platform_fit < 0.5);
    }

    #[tokio::test]
    async fn macos_bsd_command_full_fit() {
        let h = PlatformFitHydrator::new("macos");
        let mut c = cand("ps aux | sort -k3 -rn | head -5");
        h.hydrate(&mut c).await;
        assert!((c.features.platform_fit - 1.0).abs() < 1e-5);
    }

    #[tokio::test]
    async fn linux_gnu_flag_full_fit() {
        let h = PlatformFitHydrator::new("linux");
        let mut c = cand("ps aux --sort=-pcpu | head");
        h.hydrate(&mut c).await;
        assert!((c.features.platform_fit - 1.0).abs() < 1e-5);
    }

    #[tokio::test]
    async fn macos_linux_only_command_penalized() {
        let h = PlatformFitHydrator::new("macos");
        let mut c = cand("ss -tuln");
        h.hydrate(&mut c).await;
        assert!(c.features.platform_fit < 0.5);
    }

    #[tokio::test]
    async fn safety_hydrator_marks_dangerous_critical() {
        let validator = Arc::new(SafetyValidator::new(SafetyConfig::moderate()).unwrap());
        let h = SafetyHydrator::new(validator);
        let mut c = cand("rm -rf /");
        h.hydrate(&mut c).await;
        assert_eq!(c.features.risk_level, Some(RiskLevel::Critical));
    }

    #[tokio::test]
    async fn safety_hydrator_marks_safe_commands_safe() {
        let validator = Arc::new(SafetyValidator::new(SafetyConfig::moderate()).unwrap());
        let h = SafetyHydrator::new(validator);
        let mut c = cand("ls -la");
        h.hydrate(&mut c).await;
        assert!(matches!(
            c.features.risk_level,
            Some(RiskLevel::Safe) | Some(RiskLevel::Moderate)
        ));
    }
}
