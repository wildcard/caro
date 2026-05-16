//! Candidate scorer.
//!
//! Phase 1 ships a [`LinearScorer`] that combines the hydrated features
//! into a single `[0.0, 1.0]` score using the weights in
//! [`super::weights::ScorerWeights`]. Phase 4 will add a `LearnedScorer`
//! implementing the same [`Scorer`] trait so callers don't change.

use super::weights::ScorerWeights;
use super::Candidate;

/// Combine a candidate's features into a single ranking score.
pub trait Scorer: Send + Sync {
    fn score(&self, candidate: &Candidate) -> f32;
}

/// Latency budget the scorer normalizes against. Anything below this gets
/// `latency_score = 1.0`; anything above gets a linear penalty down to 0.
/// 1500 ms matches the current p95 of the embedded backend on M-series.
const LATENCY_BUDGET_MS: f32 = 1500.0;

/// Linear-combination scorer. Hand-tuned weights for Phase 2; swappable
/// with a learned scorer in Phase 4.
#[derive(Debug, Clone, Default)]
pub struct LinearScorer {
    pub weights: ScorerWeights,
}

impl LinearScorer {
    pub fn with_weights(weights: ScorerWeights) -> Self {
        Self { weights }
    }

    fn latency_score(latency_ms: u64) -> f32 {
        let ratio = (latency_ms as f32) / LATENCY_BUDGET_MS;
        (1.0 - ratio).clamp(0.0, 1.0)
    }
}

impl Scorer for LinearScorer {
    fn score(&self, c: &Candidate) -> f32 {
        let f = &c.features;
        let knowledge = f.knowledge_similarity.unwrap_or(0.0);
        let latency = Self::latency_score(f.latency_ms);
        let raw = self.weights.llm_confidence * f.llm_confidence
            + self.weights.safety_confidence * f.safety_confidence
            + self.weights.platform_fit * f.platform_fit
            + self.weights.knowledge_similarity * knowledge
            + self.weights.latency * latency;
        raw.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::super::CandidateFeatures;
    use super::*;

    fn cand(features: CandidateFeatures) -> Candidate {
        Candidate {
            command: "x".into(),
            source: "test".into(),
            features,
            score: None,
            rejection_reason: None,
        }
    }

    #[test]
    fn perfect_signals_score_one() {
        let s = LinearScorer::default();
        let c = cand(CandidateFeatures {
            llm_confidence: 1.0,
            safety_confidence: 1.0,
            platform_fit: 1.0,
            knowledge_similarity: Some(1.0),
            latency_ms: 0,
            validation_passed: true,
        });
        assert!((s.score(&c) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn zero_signals_score_zero() {
        let s = LinearScorer::default();
        let c = cand(CandidateFeatures::default());
        // latency_ms=0 gives latency_score=1.0, multiplied by weight=0.05
        assert!((s.score(&c) - 0.05).abs() < 1e-5);
    }

    #[test]
    fn safety_dominates_over_knowledge() {
        // High safety + zero knowledge should beat zero safety + high knowledge,
        // because safety has weight 0.25 vs knowledge 0.15.
        let s = LinearScorer::default();
        let safe = cand(CandidateFeatures {
            safety_confidence: 1.0,
            ..Default::default()
        });
        let known = cand(CandidateFeatures {
            knowledge_similarity: Some(1.0),
            ..Default::default()
        });
        assert!(s.score(&safe) > s.score(&known));
    }

    #[test]
    fn latency_penalty_kicks_in_above_budget() {
        let s = LinearScorer::default();
        let fast = cand(CandidateFeatures {
            llm_confidence: 0.8,
            latency_ms: 100,
            ..Default::default()
        });
        let slow = cand(CandidateFeatures {
            llm_confidence: 0.8,
            latency_ms: 5_000, // well above budget
            ..Default::default()
        });
        assert!(s.score(&fast) > s.score(&slow));
    }

    #[test]
    fn missing_knowledge_treated_as_zero() {
        let s = LinearScorer::default();
        let c1 = cand(CandidateFeatures {
            llm_confidence: 0.5,
            knowledge_similarity: None,
            ..Default::default()
        });
        let c2 = cand(CandidateFeatures {
            llm_confidence: 0.5,
            knowledge_similarity: Some(0.0),
            ..Default::default()
        });
        assert!((s.score(&c1) - s.score(&c2)).abs() < 1e-6);
    }
}
