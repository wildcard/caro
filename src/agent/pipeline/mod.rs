//! Multi-stage candidate pipeline.
//!
//! Inspired by [`xai-org/x-algorithm`][1]'s feed pipeline pattern:
//! `sources -> hydrators -> filters -> scorer -> selector`. The existing
//! caro primitives map onto these stages without re-invention:
//!
//! - **Sources** wrap [`crate::backends::CommandGenerator`] impls (a
//!   knowledge-retrieval source lands in a follow-up).
//! - **Hydrators** populate [`CandidateFeatures`] — platform-fit, safety
//!   confidence, structural validation.
//! - **Filters** consume hydrated features to drop unsafe / invalid
//!   candidates.
//! - **Scorer** computes a weighted multi-signal score (see [`weights`]).
//! - **Selector** picks the top candidate.
//!
//! Note: this module is currently *additive*. The pipeline is not wired
//! into [`crate::agent::AgentLoop`] yet — that change ships in a separate
//! commit so reviewers can vet the abstraction in isolation.
//!
//! [1]: https://github.com/xai-org/x-algorithm

pub mod filters;
pub mod hydrators;
pub mod scorer;
pub mod selector;
pub mod sources;
pub mod weights;

pub use filters::{SafetyFilter, ValidationFilter};
pub use hydrators::{PlatformFitHydrator, SafetyHydrator, ValidationHydrator};
pub use scorer::{LinearScorer, Scorer};
pub use selector::{ArgmaxSelector, Selector};
pub use sources::BackendSource;

use async_trait::async_trait;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::backends::CommandGenerator;
use crate::models::{RiskLevel, ShellType};
use crate::prompts::CommandValidator;
use crate::safety::SafetyValidator;

/// Default wall-clock budget for the whole source fan-out. A source that
/// hasn't produced a candidate by this point is dropped so a slow or hung
/// backend can't blow the interactive latency budget.
const DEFAULT_LATENCY_BUDGET: Duration = Duration::from_secs(8);

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
    /// Whether the structural / syntactic validator passed. `false` is the
    /// safe default — [`ValidationFilter`] rejects unhydrated candidates the
    /// same way [`SafetyFilter`] does, so a misconfigured pipeline fails
    /// loudly instead of silently passing everything.
    pub validation_passed: bool,
    /// Error message from the structural validator, when it rejected.
    /// Populated by [`ValidationHydrator`]; read by [`ValidationFilter`].
    pub validation_error: Option<String>,
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

/// Outcome of a ranked pipeline run: the winning candidate plus the full
/// scored set (sorted best-first) so callers can surface the trade-off each
/// source produced — e.g. in `--dry-run` / `--explain`.
#[derive(Debug, Clone)]
pub struct RankedRun {
    /// The selected best candidate. Identical to `run()`'s return value.
    pub winner: Candidate,
    /// Every candidate that was produced, sorted by score descending.
    /// Rejected candidates keep their `rejection_reason` and sort last.
    pub ranked: Vec<Candidate>,
}

/// Orchestrates the five stages. Sources fan out concurrently under a shared
/// [`Pipeline::latency_budget`]; the remaining stages run over the candidates
/// that came back in time.
pub struct Pipeline {
    pub sources: Vec<Arc<dyn CandidateSource>>,
    pub hydrators: Vec<Arc<dyn Hydrator>>,
    pub filters: Vec<Arc<dyn Filter>>,
    pub scorer: Arc<dyn Scorer>,
    pub selector: Arc<dyn Selector>,
    /// Wall-clock budget for the source fan-out (see [`DEFAULT_LATENCY_BUDGET`]).
    pub latency_budget: Duration,
}

impl Pipeline {
    /// Assemble the canonical caro best-of-N pipeline from a set of backends.
    ///
    /// Wraps each `(backend, label)` in a [`BackendSource`], wires the standard
    /// hydrators ([`PlatformFitHydrator`], [`SafetyHydrator`],
    /// [`ValidationHydrator`]), filters ([`SafetyFilter`], [`ValidationFilter`]),
    /// a [`LinearScorer`], and an [`ArgmaxSelector`]. Reuses the existing
    /// primitives rather than re-implementing scoring or selection.
    pub fn standard(
        sources: Vec<(Arc<dyn CommandGenerator>, String)>,
        safety: Arc<SafetyValidator>,
        validator: Arc<CommandValidator>,
        os: impl Into<String>,
        shell: ShellType,
    ) -> Self {
        let os = os.into();
        let sources: Vec<Arc<dyn CandidateSource>> = sources
            .into_iter()
            .map(|(backend, label)| {
                Arc::new(BackendSource::new(backend, label).with_shell(shell))
                    as Arc<dyn CandidateSource>
            })
            .collect();
        let hydrators: Vec<Arc<dyn Hydrator>> = vec![
            Arc::new(PlatformFitHydrator::new(os)),
            Arc::new(SafetyHydrator::new(safety).with_shell(shell)),
            Arc::new(ValidationHydrator::new(validator)),
        ];
        let filters: Vec<Arc<dyn Filter>> =
            vec![Arc::new(SafetyFilter), Arc::new(ValidationFilter)];
        Self {
            sources,
            hydrators,
            filters,
            scorer: Arc::new(LinearScorer::default()),
            selector: Arc::new(ArgmaxSelector),
            latency_budget: DEFAULT_LATENCY_BUDGET,
        }
    }

    /// Override the source fan-out latency budget.
    pub fn with_latency_budget(mut self, budget: Duration) -> Self {
        self.latency_budget = budget;
        self
    }

    /// Run the pipeline and return only the winning candidate.
    pub async fn run(&self, prompt: &str) -> Result<Candidate, PipelineError> {
        Ok(self.run_ranked(prompt).await?.winner)
    }

    /// Run the pipeline and return the winner plus the full scored set.
    pub async fn run_ranked(&self, prompt: &str) -> Result<RankedRun, PipelineError> {
        // Stage 1 — sources fan out concurrently. Each source gets the shared
        // latency budget; a failure or timeout drops that source rather than
        // failing the whole run, so one slow backend can't starve the rest.
        let budget = self.latency_budget;
        let produced = self.sources.iter().map(|src| {
            let src = Arc::clone(src);
            async move {
                match timeout(budget, src.produce(prompt)).await {
                    Ok(Ok(candidate)) => Some(candidate),
                    Ok(Err(_)) => None, // source errored
                    Err(_) => None,     // exceeded the latency budget
                }
            }
        });
        let mut candidates: Vec<Candidate> =
            join_all(produced).await.into_iter().flatten().collect();

        // Stage 2 — hydrate features onto each surviving candidate.
        for c in candidates.iter_mut() {
            for h in &self.hydrators {
                h.hydrate(c).await;
            }
        }

        // Stage 3 — filter out unsafe / invalid candidates.
        for c in candidates.iter_mut() {
            for f in &self.filters {
                if let Some(reason) = f.filter(c).await {
                    c.rejection_reason = Some(format!("{}: {}", f.name(), reason));
                    break;
                }
            }
        }

        // Stage 4 — score the survivors.
        for c in candidates.iter_mut() {
            if !c.is_rejected() {
                c.score = Some(self.scorer.score(c));
            }
        }

        // Stage 5 — select the winner (authoritative, stable tiebreak) before
        // reordering the set for display.
        let winner = self
            .selector
            .select(&candidates)
            .cloned()
            .ok_or(PipelineError::NoCandidates)?;

        candidates.sort_by(|a, b| {
            let sa = a.score.unwrap_or(f32::NEG_INFINITY);
            let sb = b.score.unwrap_or(f32::NEG_INFINITY);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(RankedRun {
            winner,
            ranked: candidates,
        })
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
            latency_budget: Duration::from_secs(5),
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
            latency_budget: Duration::from_secs(5),
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
            latency_budget: Duration::from_secs(5),
        };
        let err = p.run("anything").await.unwrap_err();
        assert!(matches!(err, PipelineError::NoCandidates));
    }

    /// Source with a controllable confidence, so scoring is deterministic.
    struct ConfSource(&'static str, &'static str, f32);
    #[async_trait]
    impl CandidateSource for ConfSource {
        async fn produce(&self, _prompt: &str) -> Result<Candidate, PipelineError> {
            let mut c = Candidate::new(self.1.to_string(), self.0.to_string());
            c.features.llm_confidence = self.2;
            Ok(c)
        }
        fn name(&self) -> &str {
            self.0
        }
    }

    /// Source that sleeps before producing, to simulate a slow/hung backend.
    struct SlowSource(&'static str, u64);
    #[async_trait]
    impl CandidateSource for SlowSource {
        async fn produce(&self, _prompt: &str) -> Result<Candidate, PipelineError> {
            tokio::time::sleep(Duration::from_millis(self.1)).await;
            let mut c = Candidate::new("slow-cmd", self.0);
            c.features.llm_confidence = 0.99; // would win on score if it arrived
            Ok(c)
        }
        fn name(&self) -> &str {
            self.0
        }
    }

    #[tokio::test]
    async fn parallel_fanout_selects_highest_scored_and_ranks_all() {
        let p = Pipeline {
            sources: vec![
                Arc::new(ConfSource("low", "echo low", 0.1)),
                Arc::new(ConfSource("high", "echo high", 0.9)),
                Arc::new(ConfSource("mid", "echo mid", 0.5)),
            ],
            hydrators: vec![Arc::new(FlatHydrator)],
            filters: vec![],
            scorer: Arc::new(LinearScorer::default()),
            selector: Arc::new(ArgmaxSelector),
            latency_budget: Duration::from_secs(5),
        };
        let run = p.run_ranked("anything").await.unwrap();
        // Highest llm_confidence wins under the default weights.
        assert_eq!(run.winner.command, "echo high");
        // Every source's candidate survives and the set is sorted best-first.
        assert_eq!(run.ranked.len(), 3);
        assert_eq!(run.ranked[0].command, "echo high");
        assert_eq!(run.ranked[2].command, "echo low");
    }

    #[tokio::test]
    async fn slow_source_dropped_at_latency_budget() {
        let p = Pipeline {
            sources: vec![
                Arc::new(ConfSource("fast", "echo fast", 0.2)),
                Arc::new(SlowSource("slow", 500)),
            ],
            hydrators: vec![Arc::new(FlatHydrator)],
            filters: vec![],
            scorer: Arc::new(LinearScorer::default()),
            selector: Arc::new(ArgmaxSelector),
            latency_budget: Duration::from_millis(50),
        };
        let run = p.run_ranked("anything").await.unwrap();
        // The slow source would have scored higher, but it never arrives in
        // time, so the fast source wins and is the only ranked candidate.
        assert_eq!(run.winner.command, "echo fast");
        assert_eq!(run.ranked.len(), 1);
    }
}
