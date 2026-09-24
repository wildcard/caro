//! Calibration and latency-tail metrics for evaluation results.
//!
//! Borrowed from TypeSafe AI's "System One" framing (see
//! `docs/research/jev-system-one-gap-analysis.md`): a decision is only useful
//! if the probability attached to it is *honest*. A backend that says "0.85"
//! on every command is not 85 % confident — it is reporting a constant, and
//! every gate downstream (agent refinement, advisor escalation, candidate
//! ranking) is silently keyed on backend identity rather than on evidence.
//!
//! These metrics make that visible in the eval report:
//!
//! - **Brier score** — mean squared error between reported confidence and
//!   the 0/1 outcome. `0.0` is perfect; `0.25` is what a coin-flip at 0.5
//!   scores. Lower is better.
//! - **ECE (Expected Calibration Error)** — bucket results by confidence,
//!   compare mean confidence to observed accuracy in each bucket, weight by
//!   bucket size. A constant-confidence backend has `ECE == |c − pass_rate|`.
//! - **p50 / p95 latency** — the tail is what a user feels; the mean hides it.
//!   Timed-out results are excluded: their `execution_time_ms` is the
//!   harness's timeout cap, not a generation time.
//! - **Coverage** — the fraction of results that carried a confidence at
//!   all. Brier/ECE are computed over that subset, so a backend with low
//!   coverage (e.g. failures that report no confidence) is flagged rather
//!   than flattered.
//!
//! Non-finite confidences are treated as absent rather than poisoning the
//! rollups with NaN.
//!
//! Everything here is pure over `&EvaluationResult` so it can be unit-tested
//! with hand-computed numbers and reused by any report printer.

use crate::evaluation::{ErrorType, EvaluationResult};

/// A usable confidence: present, finite, clamped to `0.0..=1.0`.
fn usable_confidence(r: &EvaluationResult) -> Option<f64> {
    r.confidence
        .filter(|c| c.is_finite())
        .map(|c| c.clamp(0.0, 1.0))
}

/// Number of equal-width confidence buckets used for ECE.
pub const ECE_BINS: usize = 10;

/// Brier score over results that reported a confidence.
///
/// Returns `None` when no result carried a confidence, so a backend that
/// never reports one shows up as "unmeasured" rather than "perfect".
pub fn brier<'a>(results: impl IntoIterator<Item = &'a EvaluationResult>) -> Option<f64> {
    let mut sum = 0.0_f64;
    let mut n = 0_usize;
    for r in results {
        if let Some(c) = usable_confidence(r) {
            let outcome = if r.passed { 1.0 } else { 0.0 };
            sum += (c - outcome).powi(2);
            n += 1;
        }
    }
    (n > 0).then(|| sum / n as f64)
}

/// Expected Calibration Error over results that reported a confidence.
///
/// Uses `bins` equal-width buckets on `[0, 1]`; the top edge is inclusive so
/// a confidence of exactly `1.0` lands in the last bucket.
pub fn ece<'a>(
    results: impl IntoIterator<Item = &'a EvaluationResult>,
    bins: usize,
) -> Option<f64> {
    let bins = bins.max(1);
    let mut conf_sum = vec![0.0_f64; bins];
    let mut hit_sum = vec![0.0_f64; bins];
    let mut count = vec![0_usize; bins];
    let mut n = 0_usize;

    for r in results {
        if let Some(c) = usable_confidence(r) {
            let idx = ((c * bins as f64) as usize).min(bins - 1);
            conf_sum[idx] += c;
            hit_sum[idx] += if r.passed { 1.0 } else { 0.0 };
            count[idx] += 1;
            n += 1;
        }
    }
    if n == 0 {
        return None;
    }

    let mut total = 0.0_f64;
    for i in 0..bins {
        if count[i] == 0 {
            continue;
        }
        let k = count[i] as f64;
        let mean_conf = conf_sum[i] / k;
        let accuracy = hit_sum[i] / k;
        total += (k / n as f64) * (mean_conf - accuracy).abs();
    }
    Some(total)
}

/// Calibration rollup over one backend's results.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CalibrationRollup {
    /// See [`brier`].
    pub brier: Option<f32>,
    /// See [`ece`] (with [`ECE_BINS`] buckets).
    pub ece: Option<f32>,
    /// Fraction of results (0.0..=1.0) that carried a usable confidence and
    /// therefore entered `brier`/`ece`. `0.0` for an empty set.
    pub coverage: f32,
}

impl CalibrationRollup {
    /// Compute both metrics in one place so the per-backend and full-run
    /// aggregation paths in the harness agree by construction.
    pub fn of<'a>(results: impl IntoIterator<Item = &'a EvaluationResult> + Clone) -> Self {
        let (total, with_conf) = results
            .clone()
            .into_iter()
            .fold((0usize, 0usize), |(t, w), r| {
                (t + 1, w + usize::from(usable_confidence(r).is_some()))
            });
        Self {
            brier: brier(results.clone()).map(|v| v as f32),
            ece: ece(results, ECE_BINS).map(|v| v as f32),
            coverage: if total > 0 {
                with_conf as f32 / total as f32
            } else {
                0.0
            },
        }
    }
}

/// Latency percentiles (milliseconds) over a set of results.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LatencyPercentiles {
    /// Median per-test generation time.
    pub p50_ms: u64,
    /// 95th percentile per-test generation time (nearest-rank).
    pub p95_ms: u64,
    /// Slowest single test.
    pub max_ms: u64,
}

impl LatencyPercentiles {
    /// Nearest-rank percentiles over results that actually generated (timed
    /// out results carry the timeout cap, not a latency, and are excluded);
    /// all zeros for an empty set.
    pub fn of<'a>(results: impl IntoIterator<Item = &'a EvaluationResult>) -> Self {
        let mut times: Vec<u64> = results
            .into_iter()
            .filter(|r| r.error_type != Some(ErrorType::Timeout))
            .map(|r| r.execution_time_ms)
            .collect();
        if times.is_empty() {
            return Self::default();
        }
        times.sort_unstable();
        Self {
            p50_ms: nearest_rank(&times, 0.50),
            p95_ms: nearest_rank(&times, 0.95),
            max_ms: *times.last().unwrap_or(&0),
        }
    }
}

/// Nearest-rank percentile on an already-sorted, non-empty slice.
fn nearest_rank(sorted: &[u64], p: f64) -> u64 {
    let n = sorted.len();
    let rank = ((p * n as f64).ceil() as usize).clamp(1, n);
    sorted[rank - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn result(passed: bool, confidence: Option<f64>, ms: u64) -> EvaluationResult {
        EvaluationResult {
            test_id: "t-1".into(),
            backend_name: "b".into(),
            passed,
            actual_command: None,
            actual_behavior: None,
            failure_reason: None,
            execution_time_ms: ms,
            timestamp: Utc::now(),
            error_type: None,
            est_tokens_in: 0,
            est_tokens_out: 0,
            est_cost_usd: 0.0,
            criteria_passed: 0,
            criteria_total: 0,
            confidence,
        }
    }

    #[test]
    fn brier_none_without_confidence() {
        let rs = vec![result(true, None, 1), result(false, None, 1)];
        assert_eq!(brier(&rs), None);
        assert_eq!(ece(&rs, ECE_BINS), None);
        assert_eq!(CalibrationRollup::of(&rs), CalibrationRollup::default());
    }

    #[test]
    fn constant_confidence_all_correct() {
        // Four passes at 0.85: Brier = (0.15)^2 = 0.0225, ECE = |0.85 - 1.0| = 0.15.
        let rs: Vec<_> = (0..4).map(|_| result(true, Some(0.85), 1)).collect();
        assert!((brier(&rs).unwrap() - 0.0225).abs() < 1e-9);
        assert!((ece(&rs, ECE_BINS).unwrap() - 0.15).abs() < 1e-9);
    }

    #[test]
    fn constant_confidence_ece_equals_gap_to_pass_rate() {
        // 3 of 4 pass at constant 1.0 → pass_rate 0.75, ECE = 0.25, Brier = 0.25.
        let rs = vec![
            result(true, Some(1.0), 1),
            result(true, Some(1.0), 1),
            result(true, Some(1.0), 1),
            result(false, Some(1.0), 1),
        ];
        assert!((ece(&rs, ECE_BINS).unwrap() - 0.25).abs() < 1e-9);
        assert!((brier(&rs).unwrap() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn ece_weights_bins_by_count() {
        // Bin [0.9,1.0]: two at 0.9, both pass → |0.9-1.0| = 0.1 weighted 2/4.
        // Bin [0.2,0.3): two at 0.2, none pass → |0.2-0.0| = 0.2 weighted 2/4.
        let rs = vec![
            result(true, Some(0.9), 1),
            result(true, Some(0.9), 1),
            result(false, Some(0.2), 1),
            result(false, Some(0.2), 1),
        ];
        assert!((ece(&rs, ECE_BINS).unwrap() - 0.15).abs() < 1e-9);
    }

    #[test]
    fn confidence_is_clamped_and_nan_is_absent() {
        let rs = vec![result(true, Some(7.0), 1), result(false, Some(-3.0), 1)];
        assert!((brier(&rs).unwrap() - 0.0).abs() < 1e-9);

        let nan = vec![
            result(true, Some(f64::NAN), 1),
            result(true, Some(f64::INFINITY), 1),
        ];
        assert_eq!(brier(&nan), None);
        assert_eq!(ece(&nan, ECE_BINS), None);
        assert_eq!(CalibrationRollup::of(&nan).coverage, 0.0);
    }

    #[test]
    fn coverage_counts_results_without_confidence() {
        // Two failures with no confidence + two passes at 0.9: Brier/ECE see
        // only the passes (look calibrated-ish) but coverage exposes the gap.
        let rs = vec![
            result(false, None, 1),
            result(false, None, 1),
            result(true, Some(0.9), 1),
            result(true, Some(0.9), 1),
        ];
        let c = CalibrationRollup::of(&rs);
        assert!((c.coverage - 0.5).abs() < 1e-6);
        assert!((c.ece.unwrap() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn latency_excludes_timeouts() {
        let mut timed_out = result(false, None, 30_000);
        timed_out.error_type = Some(ErrorType::Timeout);
        let rs = vec![result(true, None, 10), result(true, None, 20), timed_out];
        let p = LatencyPercentiles::of(&rs);
        assert_eq!(p.p95_ms, 20);
        assert_eq!(p.max_ms, 20);
    }

    #[test]
    fn latency_percentiles_nearest_rank() {
        let rs: Vec<_> = [50u64, 10, 30, 20, 40, 1000]
            .iter()
            .map(|&ms| result(true, None, ms))
            .collect();
        let p = LatencyPercentiles::of(&rs);
        // sorted: 10 20 30 40 50 1000 → p50 rank ceil(3)=3 → 30; p95 rank ceil(5.7)=6 → 1000
        assert_eq!(p.p50_ms, 30);
        assert_eq!(p.p95_ms, 1000);
        assert_eq!(p.max_ms, 1000);
    }

    #[test]
    fn pre_calibration_json_still_loads() {
        // A BackendResult / EvaluationResult serialised before these fields
        // existed must still deserialise (serde(default)), so stored
        // baselines survive the upgrade.
        let backend: crate::evaluation::BackendResult = serde_json::from_str(
            r#"{"backend_name":"b","pass_rate":0.5,"total_tests":2,"passed":1,
                "failed":1,"timeouts":0,"avg_execution_time_ms":3,
                "category_breakdown":{}}"#,
        )
        .unwrap();
        assert_eq!(backend.brier, None);
        assert_eq!(backend.ece, None);
        assert_eq!(backend.p95_execution_time_ms, 0);

        let r: EvaluationResult = serde_json::from_str(
            r#"{"test_id":"t","backend_name":"b","passed":true,
                "execution_time_ms":1,"timestamp":"2026-01-01T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(r.confidence, None);
    }

    #[test]
    fn latency_percentiles_empty() {
        let rs: Vec<EvaluationResult> = vec![];
        assert_eq!(LatencyPercentiles::of(&rs), LatencyPercentiles::default());
    }
}
