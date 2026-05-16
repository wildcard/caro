//! Filters — drop candidates that should not be considered.
//!
//! Phase 2 ships two:
//!
//! - [`SafetyFilter`] reads the `risk_level` written by
//!   [`super::SafetyHydrator`] and rejects Critical candidates. Decoupling
//!   "compute safety" (hydrator) from "act on safety" (filter) means the
//!   validator runs once per candidate even if many filters care about it.
//! - [`ValidationFilter`] wraps the existing [`crate::prompts::CommandValidator`]
//!   so structurally-broken commands (bad quoting, banned tools, etc.) are
//!   dropped before scoring. It also sets `features.validation_passed`.

use async_trait::async_trait;

use super::{Candidate, Filter};
use crate::models::RiskLevel;
use crate::prompts::CommandValidator;

/// Rejects candidates whose [`super::SafetyHydrator`] marked them Critical.
/// Moderate / High candidates pass through here — they get routed to
/// human approval elsewhere in caro, not blocked at the pipeline stage.
#[derive(Default)]
pub struct SafetyFilter;

#[async_trait]
impl Filter for SafetyFilter {
    async fn filter(&self, c: &Candidate) -> Option<String> {
        match c.features.risk_level {
            Some(RiskLevel::Critical) => Some("Critical risk level".to_string()),
            None => Some("safety not hydrated".to_string()),
            _ => None,
        }
    }
    fn name(&self) -> &str {
        "safety-filter"
    }
}

/// Runs the existing structural [`CommandValidator`] and rejects on hard
/// errors. Also stamps `validation_passed` onto the candidate so downstream
/// observers can see why a candidate scored as it did.
pub struct ValidationFilter {
    validator: CommandValidator,
}

impl ValidationFilter {
    pub fn new(validator: CommandValidator) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl Filter for ValidationFilter {
    async fn filter(&self, c: &Candidate) -> Option<String> {
        let result = self.validator.validate(&c.command);
        if result.is_valid() {
            None
        } else {
            Some(result.error_message())
        }
    }
    fn name(&self) -> &str {
        "validation-filter"
    }
}

#[cfg(test)]
mod tests {
    use super::super::CandidateFeatures;
    use super::*;

    fn cand_with_risk(risk: Option<RiskLevel>) -> Candidate {
        Candidate {
            command: "x".into(),
            source: "t".into(),
            features: CandidateFeatures {
                risk_level: risk,
                ..Default::default()
            },
            score: None,
            rejection_reason: None,
        }
    }

    #[tokio::test]
    async fn safety_filter_rejects_critical() {
        let f = SafetyFilter;
        assert!(f
            .filter(&cand_with_risk(Some(RiskLevel::Critical)))
            .await
            .is_some());
    }

    #[tokio::test]
    async fn safety_filter_passes_safe() {
        let f = SafetyFilter;
        assert!(f.filter(&cand_with_risk(Some(RiskLevel::Safe))).await.is_none());
    }

    #[tokio::test]
    async fn safety_filter_passes_moderate() {
        let f = SafetyFilter;
        assert!(f
            .filter(&cand_with_risk(Some(RiskLevel::Moderate)))
            .await
            .is_none());
    }

    #[tokio::test]
    async fn safety_filter_rejects_unhydrated_candidate() {
        let f = SafetyFilter;
        let reason = f.filter(&cand_with_risk(None)).await;
        assert!(reason.is_some(), "should reject when safety not hydrated");
    }
}
