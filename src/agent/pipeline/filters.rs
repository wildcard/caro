//! Filters — drop candidates that should not be considered.
//!
//! Filters are paired with hydrators: the hydrator does the work (runs the
//! validator, computes the risk level), the filter reads the resulting
//! feature and decides. This keeps the validator sweep to one call per
//! candidate even if many filters consume the result.
//!
//! Shipped filters:
//!
//! - [`SafetyFilter`] reads `risk_level` written by [`super::SafetyHydrator`]
//!   and rejects Critical candidates. Moderate / High pass through — the
//!   caller is responsible for routing those to human approval.
//! - [`ValidationFilter`] reads `validation_passed` written by
//!   [`super::ValidationHydrator`] and rejects on hard structural errors.

use async_trait::async_trait;

use super::{Candidate, Filter};
use crate::models::RiskLevel;

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

/// Rejects candidates whose [`super::ValidationHydrator`] flagged as invalid.
/// Like [`SafetyFilter`], rejects unhydrated candidates by default — easier
/// to spot a misconfigured pipeline than to silently let bad commands win.
#[derive(Default)]
pub struct ValidationFilter;

#[async_trait]
impl Filter for ValidationFilter {
    async fn filter(&self, c: &Candidate) -> Option<String> {
        if c.features.validation_passed {
            None
        } else {
            // Echo the hydrator's reason when available; otherwise treat
            // missing hydration as a failure.
            Some(
                c.features
                    .validation_error
                    .clone()
                    .unwrap_or_else(|| "validation not hydrated".to_string()),
            )
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
        assert!(f
            .filter(&cand_with_risk(Some(RiskLevel::Safe)))
            .await
            .is_none());
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

    fn cand_with_validation(passed: bool, err: Option<&str>) -> Candidate {
        Candidate {
            command: "x".into(),
            source: "t".into(),
            features: CandidateFeatures {
                validation_passed: passed,
                validation_error: err.map(String::from),
                ..Default::default()
            },
            score: None,
            rejection_reason: None,
        }
    }

    #[tokio::test]
    async fn validation_filter_passes_valid_command() {
        let f = ValidationFilter;
        assert!(f.filter(&cand_with_validation(true, None)).await.is_none());
    }

    #[tokio::test]
    async fn validation_filter_rejects_invalid_with_echoed_reason() {
        let f = ValidationFilter;
        let reason = f
            .filter(&cand_with_validation(false, Some("banned tool: curl")))
            .await
            .expect("should reject");
        assert!(reason.contains("banned tool"));
    }

    #[tokio::test]
    async fn validation_filter_rejects_unhydrated_candidate() {
        let f = ValidationFilter;
        // Default: validation_passed = false, validation_error = None
        let reason = f.filter(&Candidate::new("x", "t")).await;
        assert!(
            reason.is_some(),
            "should reject when validation not hydrated"
        );
        assert!(reason.unwrap().contains("not hydrated"));
    }
}
