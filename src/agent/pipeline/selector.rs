//! Candidate selector — picks the winner from a scored set.
//!
//! Phase 1 ships a single [`ArgmaxSelector`]. Phase 4 may add an
//! `EpsilonGreedySelector` for online exploration once telemetry is rich
//! enough to support it.

use super::Candidate;

pub trait Selector: Send + Sync {
    /// Return the best non-rejected candidate, or `None` if none qualify.
    fn select<'a>(&self, candidates: &'a [Candidate]) -> Option<&'a Candidate>;
}

/// Picks the candidate with the highest `score`. Stable tiebreak: keep the
/// first one seen, matching `Vec` insertion order from the pipeline run.
#[derive(Debug, Clone, Default)]
pub struct ArgmaxSelector;

impl Selector for ArgmaxSelector {
    fn select<'a>(&self, candidates: &'a [Candidate]) -> Option<&'a Candidate> {
        // Manual loop with strict `>` so equal scores keep the first-seen
        // candidate — `Iterator::max_by` would return the last on ties.
        let mut best: Option<&'a Candidate> = None;
        let mut best_score = f32::NEG_INFINITY;
        for c in candidates.iter().filter(|c| !c.is_rejected()) {
            let s = c.score.unwrap_or(f32::NEG_INFINITY);
            if s > best_score {
                best_score = s;
                best = Some(c);
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::super::CandidateFeatures;
    use super::*;

    fn scored(cmd: &str, score: f32) -> Candidate {
        Candidate {
            command: cmd.into(),
            source: "t".into(),
            features: CandidateFeatures::default(),
            score: Some(score),
            rejection_reason: None,
        }
    }

    #[test]
    fn picks_highest_score() {
        let cs = vec![scored("a", 0.1), scored("b", 0.9), scored("c", 0.5)];
        let s = ArgmaxSelector;
        assert_eq!(s.select(&cs).unwrap().command, "b");
    }

    #[test]
    fn skips_rejected_even_if_scored_higher() {
        let mut rejected = scored("a", 0.99);
        rejected.rejection_reason = Some("unsafe".into());
        let cs = vec![rejected, scored("b", 0.2)];
        let s = ArgmaxSelector;
        assert_eq!(s.select(&cs).unwrap().command, "b");
    }

    #[test]
    fn returns_none_when_all_rejected() {
        let mut a = scored("a", 0.99);
        a.rejection_reason = Some("x".into());
        let mut b = scored("b", 0.2);
        b.rejection_reason = Some("y".into());
        let s = ArgmaxSelector;
        assert!(s.select(&[a, b]).is_none());
    }

    #[test]
    fn stable_tiebreak_keeps_first() {
        let cs = vec![scored("a", 0.5), scored("b", 0.5), scored("c", 0.5)];
        let s = ArgmaxSelector;
        assert_eq!(s.select(&cs).unwrap().command, "a");
    }
}
