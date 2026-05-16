//! Hand-tuned weights for [`super::LinearScorer`].
//!
//! Phase 2 will load these from `~/.config/caro/scorer.toml`; Phase 4 will
//! replace the linear combination with a learned two-tower scorer that
//! implements the same [`super::Scorer`] trait. Keeping the weights in one
//! `const` block makes that swap a one-file change.

/// Default weights. Must sum to 1.0 (asserted in unit tests).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScorerWeights {
    pub llm_confidence: f32,
    pub safety_confidence: f32,
    pub platform_fit: f32,
    pub knowledge_similarity: f32,
    pub latency: f32,
}

impl Default for ScorerWeights {
    fn default() -> Self {
        Self {
            llm_confidence: 0.35,
            safety_confidence: 0.25,
            platform_fit: 0.20,
            knowledge_similarity: 0.15,
            latency: 0.05,
        }
    }
}

impl ScorerWeights {
    pub fn sum(&self) -> f32 {
        self.llm_confidence
            + self.safety_confidence
            + self.platform_fit
            + self.knowledge_similarity
            + self.latency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights_sum_to_one() {
        let w = ScorerWeights::default();
        let s = w.sum();
        assert!((s - 1.0).abs() < 1e-6, "weights must sum to 1.0, got {}", s);
    }
}
