//! Multi-stage candidate pipeline (Phase 1 scaffolding).
//!
//! Inspired by [`xai-org/x-algorithm`][1]'s feed pipeline pattern:
//! `sources -> hydrators -> filters -> scorer -> selector`. The existing caro
//! primitives map onto these stages without re-invention:
//!
//! - **Sources** wrap [`crate::backends::CommandGenerator`] impls plus the
//!   knowledge index k-NN retrieval (Phase 2).
//! - **Hydrators** add features such as platform-fit and safety-confidence.
//! - **Filters** drop unsafe / invalid candidates using the existing
//!   [`crate::safety::SafetyValidator`] and `CommandValidator`.
//! - **Scorer** computes a weighted multi-signal score (see [`weights`]).
//! - **Selector** picks the top candidate.
//!
//! Phase 1 (this module) provides the types, traits, a [`LinearScorer`], an
//! [`ArgmaxSelector`], and a [`Pipeline`] orchestrator with unit tests.
//! Phase 2 will plumb real sources / hydrators / filters and wire the
//! pipeline into [`crate::agent::AgentLoop`] behind this feature flag.
//!
//! [1]: https://github.com/xai-org/x-algorithm

pub mod filters;
pub mod hydrators;
pub mod scorer;
pub mod selector;
pub mod sources;
pub mod weights;

pub use filters::{SafetyFilter, ValidationFilter};
pub use hydrators::{PlatformFitHydrator, SafetyHydrator};
pub use scorer::{LinearScorer, Scorer};
pub use selector::{ArgmaxSelector, Selector};
pub use sources::BackendSource;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::RiskLevel;

/// Feature vector hydrated onto each candidate before scoring.
///
/// Each field is a normalized signal in `[0.0, 1.0]` (or, for `latency_ms`,
/// a raw measurement transformed to `[0, 1]` inside the scorer). The
/// [`Scorer`] is the only place that knows how to combine these.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CandidateFeatures {
    /// LLM self-reported confidence (or 1.0 for static-matcher hits).
    pub llm_confidence: f32,
    /// Confidence from [`crate::safety::SafetyValidator`] — higher = safer.
    pub safety_confidence: f32,
    /// Risk level from the safety validator. `None` until [`SafetyHydrator`] runs.
    pub risk_level: Option<RiskLevel>,
    /// 1.0 if no platform-specific issues detected, else lower.
    pub platform_fit: f32,
    /// Cosine similarity to the closest known successful command, if any.
    pub knowledge_similarity: Option<f32>,
    /// Wall-clock time the source took to produce this candidate.
    pub latency_ms: u64,
    /// Whether the structural / syntactic validator passed.
    pub validation_passed: bool,
}

/// A single candidate command produced by a source.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub command: String,
    /// Human-readable identifier of the source (e.g. `"static"`, `"embedded@T=0.4"`).
    pub source: String,
    pub features: CandidateFeatures,
    /// Set by [`Scorer`] after filters have run.
    pub score: Option<f32>,
    /// Populated by the first [`Filter`] that rejects this candidate.
    pub rejection_reason: Option<String>,
}

impl Candidate {
    pub fn new(command: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            source: source.into(),
            features: CandidateFeatures::default(),
            score: None,
            rejection_reason: None,
        }
    }

    pub fn is_rejected(&self) -> bool {
        self.rejection_reason.is_some()
    }
}

/// A producer of candidate commands. Sources should be cheap to run in
/// parallel — they must not share mutable state with each other.
#[async_trait]
pub trait CandidateSource: Send + Sync {
    async fn produce(&self, prompt: &str) -> Result<Candidate, PipelineError>;
    fn name(&self) -> &str;
}

/// Adds features to a candidate. Hydrators are pure: they never reject. The
/// trait is async so impls can call out to async machinery (e.g.
/// [`crate::safety::SafetyValidator::validate_command`]).
#[async_trait]
pub trait Hydrator: Send + Sync {
    async fn hydrate(&self, candidate: &mut Candidate);
    fn name(&self) -> &str;
}

/// Inspects a candidate; returns `Some(reason)` to reject, `None` to keep.
/// Async-trait friendly even when individual impls don't await — keeps the
/// trait composable with async hydration upstream.
#[async_trait]
pub trait Filter: Send + Sync {
    async fn filter(&self, candidate: &Candidate) -> Option<String>;
    fn name(&self) -> &str;
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("source `{name}` failed: {message}")]
    SourceFailed { name: String, message: String },
    #[error("all candidates were filtered out or no sources produced output")]
    NoCandidates,
}

/// Orchestrates the five stages. Phase 1 runs sources sequentially; Phase 2
/// will fan them out under a shared latency budget.
pub struct Pipeline {
    pub sources: Vec<Arc<dyn CandidateSource>>,
    pub hydrators: Vec<Arc<dyn Hydrator>>,
    pub filters: Vec<Arc<dyn Filter>>,
    pub scorer: Arc<dyn Scorer>,
    pub selector: Arc<dyn Selector>,
}

impl Pipeline {
    pub async fn run(&self, prompt: &str) -> Result<Candidate, PipelineError> {
        let mut candidates: Vec<Candidate> = Vec::with_capacity(self.sources.len());
        for src in &self.sources {
            if let Ok(c) = src.produce(prompt).await {
                candidates.push(c);
            }
        }

        for c in candidates.iter_mut() {
            for h in &self.hydrators {
                h.hydrate(c).await;
            }
        }

        for c in candidates.iter_mut() {
            for f in &self.filters {
                if let Some(reason) = f.filter(c).await {
                    c.rejection_reason = Some(format!("{}: {}", f.name(), reason));
                    break;
                }
            }
        }

        for c in candidates.iter_mut() {
            if !c.is_rejected() {
                c.score = Some(self.scorer.score(c));
            }
        }

        self.selector
            .select(&candidates)
            .cloned()
            .ok_or(PipelineError::NoCandidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeSource(&'static str, &'static str);

    #[async_trait]
    impl CandidateSource for FakeSource {
        async fn produce(&self, _prompt: &str) -> Result<Candidate, PipelineError> {
            let mut c = Candidate::new(self.1.to_string(), self.0.to_string());
            c.features.llm_confidence = 0.5;
            Ok(c)
        }
        fn name(&self) -> &str {
            self.0
        }
    }

    struct FlatHydrator;
    #[async_trait]
    impl Hydrator for FlatHydrator {
        async fn hydrate(&self, c: &mut Candidate) {
            c.features.safety_confidence = 1.0;
            c.features.platform_fit = 1.0;
            c.features.validation_passed = true;
        }
        fn name(&self) -> &str {
            "flat"
        }
    }

    struct RejectByName(&'static str);
    #[async_trait]
    impl Filter for RejectByName {
        async fn filter(&self, c: &Candidate) -> Option<String> {
            (c.source == self.0).then(|| format!("rejected source {}", self.0))
        }
        fn name(&self) -> &str {
            "reject-by-name"
        }
    }

    #[tokio::test]
    async fn pipeline_picks_best_scored_candidate() {
        let p = Pipeline {
            sources: vec![
                Arc::new(FakeSource("a", "echo a")),
                Arc::new(FakeSource("b", "echo b")),
                Arc::new(FakeSource("c", "echo c")),
            ],
            hydrators: vec![Arc::new(FlatHydrator)],
            filters: vec![],
            scorer: Arc::new(LinearScorer::default()),
            selector: Arc::new(ArgmaxSelector),
        };
        let winner = p.run("anything").await.unwrap();
        // All candidates have identical features → first wins (deterministic).
        assert_eq!(winner.command, "echo a");
        assert!(winner.score.is_some());
    }

    #[tokio::test]
    async fn pipeline_drops_filtered_candidates() {
        let p = Pipeline {
            sources: vec![
                Arc::new(FakeSource("a", "echo a")),
                Arc::new(FakeSource("b", "echo b")),
            ],
            hydrators: vec![Arc::new(FlatHydrator)],
            filters: vec![Arc::new(RejectByName("a"))],
            scorer: Arc::new(LinearScorer::default()),
            selector: Arc::new(ArgmaxSelector),
        };
        let winner = p.run("anything").await.unwrap();
        assert_eq!(winner.command, "echo b");
    }

    #[tokio::test]
    async fn pipeline_errors_when_all_filtered() {
        let p = Pipeline {
            sources: vec![Arc::new(FakeSource("a", "echo a"))],
            hydrators: vec![Arc::new(FlatHydrator)],
            filters: vec![Arc::new(RejectByName("a"))],
            scorer: Arc::new(LinearScorer::default()),
            selector: Arc::new(ArgmaxSelector),
        };
        let err = p.run("anything").await.unwrap_err();
        assert!(matches!(err, PipelineError::NoCandidates));
    }
}
