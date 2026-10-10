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

/// `(confidence, outcome)` pairs for results that reported a confidence,
/// scored on pass/fail (the generation metric).
fn generation_pairs<'a>(
    results: impl IntoIterator<Item = &'a EvaluationResult>,
) -> Vec<(f64, bool)> {
    results
        .into_iter()
        .filter_map(|r| usable_confidence(r).map(|c| (c, r.passed)))
        .collect()
}

/// Brier score over `(confidence, outcome)` pairs; `None` for an empty set.
fn brier_of(pairs: &[(f64, bool)]) -> Option<f64> {
    if pairs.is_empty() {
        return None;
    }
    let sum: f64 = pairs
        .iter()
        .map(|&(c, hit)| (c - if hit { 1.0 } else { 0.0 }).powi(2))
        .sum();
    Some(sum / pairs.len() as f64)
}

/// Expected Calibration Error over `(confidence, outcome)` pairs with `bins`
/// equal-width buckets on `[0, 1]`; `None` for an empty set.
fn ece_of(pairs: &[(f64, bool)], bins: usize) -> Option<f64> {
    let bins = bins.max(1);
    let mut conf_sum = vec![0.0_f64; bins];
    let mut hit_sum = vec![0.0_f64; bins];
    let mut count = vec![0_usize; bins];
    for &(c, hit) in pairs {
        let idx = ((c * bins as f64) as usize).min(bins - 1);
        conf_sum[idx] += c;
        hit_sum[idx] += if hit { 1.0 } else { 0.0 };
        count[idx] += 1;
    }
    let n = pairs.len();
    if n == 0 {
        return None;
    }
    let mut total = 0.0_f64;
    for i in 0..bins {
        if count[i] == 0 {
            continue;
        }
        let k = count[i] as f64;
        total += (k / n as f64) * (conf_sum[i] / k - hit_sum[i] / k).abs();
    }
    Some(total)
}

/// Brier score over results that reported a confidence.
///
/// Returns `None` when no result carried a confidence, so a backend that
/// never reports one shows up as "unmeasured" rather than "perfect".
pub fn brier<'a>(results: impl IntoIterator<Item = &'a EvaluationResult>) -> Option<f64> {
    brier_of(&generation_pairs(results))
}

/// Expected Calibration Error over results that reported a confidence.
///
/// Uses `bins` equal-width buckets on `[0, 1]`; the top edge is inclusive so
/// a confidence of exactly `1.0` lands in the last bucket.
pub fn ece<'a>(
    results: impl IntoIterator<Item = &'a EvaluationResult>,
    bins: usize,
) -> Option<f64> {
    ece_of(&generation_pairs(results), bins)
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

/// Count results per confidence provenance (#1464). Results that never
/// generated (`None`) are not counted.
pub fn source_counts<'a>(
    results: impl IntoIterator<Item = &'a EvaluationResult>,
) -> std::collections::BTreeMap<crate::models::ConfidenceSource, u32> {
    let mut counts = std::collections::BTreeMap::new();
    for r in results {
        if let Some(src) = r.confidence_source {
            *counts.entry(src).or_insert(0) += 1;
        }
    }
    counts
}

/// Count results whose risk-judge decision failed (#1465). Results where
/// the judge did not run (`None`) are not counted.
pub fn decision_failure_count<'a>(results: impl IntoIterator<Item = &'a EvaluationResult>) -> u32 {
    results
        .into_iter()
        .filter(|r| r.decision_failed == Some(true))
        .count() as u32
}

/// Local-vs-reference risk agreement (#1466): `(fraction_agreeing,
/// disagreements)` over results carrying both labels; the fraction is `None`
/// when no result has both.
pub fn risk_agreement<'a>(
    results: impl IntoIterator<Item = &'a EvaluationResult>,
) -> (Option<f32>, u32) {
    let (mut both, mut agree) = (0u32, 0u32);
    for r in results {
        if let Some(a) = r.risk_agreement() {
            both += 1;
            agree += u32::from(a);
        }
    }
    let fraction = (both > 0).then(|| agree as f32 / both as f32);
    (fraction, both - agree)
}

/// Bootstrap resamples behind every confidence interval here.
pub const BOOTSTRAP_RESAMPLES: usize = 1000;

/// Fixed seed so two reports over the same results print the same
/// intervals; the resampling noise is not a property of the backend.
const BOOTSTRAP_SEED: u64 = 0x1510_C0DE_5EED;

/// Risk-gate calibration (#1510): the backend's confidence in its own risk
/// verdict, scored against *agreement with the reference labeller* on the
/// same command. This is the baseline ADR-018's classifier has to beat; the
/// generation [`CalibrationRollup`] measures a different decision.
///
/// Computed over results carrying both verdicts. Intervals are 95 %
/// percentile bootstrap CIs over [`BOOTSTRAP_RESAMPLES`] resamples. At small
/// `n`, or when the labelled rows repeat the same (confidence, outcome),
/// the percentile bootstrap under-covers and can collapse to the point
/// estimate (one row, or two identical rows, print `[x, x]`), so `n` and
/// `coverage` are reported alongside and the intervals are only read as a
/// 95 % range once `n` is in the tens.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RiskGateCalibration {
    /// Rows with a local verdict, a reference verdict and a finite confidence.
    pub n: u32,
    /// `n` as a fraction (0.0..=1.0) of all results for the backend, the
    /// same denominator as [`CalibrationRollup::coverage`].
    pub coverage: f32,
    /// Brier score of the local verdict's confidence against agreement.
    pub brier: f32,
    /// 95 % bootstrap interval for `brier` as `(low, high)`.
    pub brier_ci: (f32, f32),
    /// ECE (with [`ECE_BINS`] buckets) of the same.
    pub ece: f32,
    /// 95 % bootstrap interval for `ece` as `(low, high)`.
    pub ece_ci: (f32, f32),
}

/// Risk-gate calibration over results carrying both risk verdicts; `None`
/// when no result does (the judge or the reference labeller did not run).
pub fn risk_gate_calibration<'a>(
    results: impl IntoIterator<Item = &'a EvaluationResult>,
) -> Option<RiskGateCalibration> {
    let mut total = 0usize;
    let pairs: Vec<(f64, bool)> = results
        .into_iter()
        .inspect(|_| total += 1)
        .filter_map(|r| {
            let agree = r.risk_agreement()?;
            let c = r.local_risk.as_ref()?.confidence;
            c.is_finite().then(|| (c.clamp(0.0, 1.0), agree))
        })
        .collect();
    let brier = brier_of(&pairs)?;
    let ece = ece_of(&pairs, ECE_BINS)?;
    let (b_lo, b_hi) = bootstrap_ci(&pairs, |p| brier_of(p).unwrap_or(0.0));
    let (e_lo, e_hi) = bootstrap_ci(&pairs, |p| ece_of(p, ECE_BINS).unwrap_or(0.0));
    Some(RiskGateCalibration {
        n: pairs.len() as u32,
        coverage: pairs.len() as f32 / total as f32,
        brier: brier as f32,
        brier_ci: (b_lo as f32, b_hi as f32),
        ece: ece as f32,
        ece_ci: (e_lo as f32, e_hi as f32),
    })
}

/// 95 % percentile bootstrap interval of `stat` over `pairs` (resampled with
/// replacement, [`BOOTSTRAP_RESAMPLES`] times, fixed seed).
fn bootstrap_ci(pairs: &[(f64, bool)], stat: impl Fn(&[(f64, bool)]) -> f64) -> (f64, f64) {
    let n = pairs.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let mut rng = BOOTSTRAP_SEED;
    let mut sample = vec![(0.0, false); n];
    let mut stats = Vec::with_capacity(BOOTSTRAP_RESAMPLES);
    for _ in 0..BOOTSTRAP_RESAMPLES {
        for slot in sample.iter_mut() {
            // xorshift64*: small, dependency-free, more than enough for resampling.
            rng ^= rng >> 12;
            rng ^= rng << 25;
            rng ^= rng >> 27;
            let idx = (rng.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) as usize % n;
            *slot = pairs[idx];
        }
        stats.push(stat(&sample));
    }
    stats.sort_by(|a, b| a.total_cmp(b));
    let at = |q: f64| stats[((q * (stats.len() - 1) as f64).round() as usize).min(stats.len() - 1)];
    (at(0.025), at(0.975))
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
            confidence_source: confidence.map(|_| crate::models::ConfidenceSource::Measured),
            decision_failed: None,
            local_risk: None,
            reference_risk: None,
        }
    }

    #[test]
    fn counts_decision_failures() {
        let mut rows = [
            result(true, None, 1),
            result(true, None, 1),
            result(false, None, 1),
        ];
        rows[0].decision_failed = Some(true);
        rows[1].decision_failed = Some(false);
        // rows[2] never ran the judge
        assert_eq!(decision_failure_count(rows.iter()), 1);
        assert_eq!(decision_failure_count(std::iter::empty()), 0);
    }

    #[test]
    fn risk_gate_calibration_scores_confidence_against_agreement() {
        use crate::models::{RiskJudgment, RiskLevel};
        let j = |risk, confidence| RiskJudgment {
            risk,
            reason: String::new(),
            confidence,
        };
        let mut rows = [
            result(true, None, 1),
            result(true, None, 1),
            result(true, None, 1),
            result(true, None, 1),
        ];
        rows[0].local_risk = Some(j(RiskLevel::Safe, 0.9));
        rows[0].reference_risk = Some(j(RiskLevel::Safe, 1.0));
        rows[1].local_risk = Some(j(RiskLevel::Safe, 0.9));
        rows[1].reference_risk = Some(j(RiskLevel::Safe, 1.0));
        rows[2].local_risk = Some(j(RiskLevel::Safe, 0.8));
        rows[2].reference_risk = Some(j(RiskLevel::High, 1.0)); // confident and wrong
        rows[3].local_risk = Some(j(RiskLevel::High, 0.5)); // no reference label: excluded

        let cal = risk_gate_calibration(rows.iter()).unwrap();
        assert_eq!(cal.n, 3);
        assert!(
            (cal.coverage - 0.75).abs() < 1e-6,
            "3 of 4 rows carry both verdicts"
        );
        // ((0.1)^2 + (0.1)^2 + (0.8)^2) / 3
        assert!((cal.brier - 0.22).abs() < 1e-6, "brier {}", cal.brier);
        // bin 0.9: |0.9 - 1.0| * 2/3; bin 0.8: |0.8 - 0.0| * 1/3
        assert!(
            (cal.ece - (0.1 * 2.0 / 3.0 + 0.8 / 3.0) as f32).abs() < 1e-6,
            "ece {}",
            cal.ece
        );
        assert!(cal.brier_ci.0 <= cal.brier && cal.brier <= cal.brier_ci.1);
        assert!(cal.ece_ci.0 <= cal.ece && cal.ece <= cal.ece_ci.1);
        assert!(
            cal.brier_ci.0 < cal.brier_ci.1,
            "three distinct rows give a real interval"
        );

        // Deterministic: same rows, same intervals.
        assert_eq!(risk_gate_calibration(rows.iter()), Some(cal));
        // Unmeasured when nothing carries both labels.
        assert_eq!(risk_gate_calibration(std::iter::once(&rows[3])), None);
        assert_eq!(risk_gate_calibration(std::iter::empty()), None);
    }

    #[test]
    fn bootstrap_ci_collapses_on_a_single_row() {
        let (lo, hi) = bootstrap_ci(&[(0.7, true)], |p| brier_of(p).unwrap());
        assert!((lo - 0.09).abs() < 1e-9 && (hi - 0.09).abs() < 1e-9);
        assert_eq!(bootstrap_ci(&[], |_| 1.0), (0.0, 0.0));
    }

    #[test]
    fn risk_agreement_counts_only_doubly_labelled_rows() {
        use crate::models::{RiskJudgment, RiskLevel};
        let j = |risk| RiskJudgment {
            risk,
            reason: String::new(),
            confidence: 0.9,
        };
        let mut rows = [
            result(true, None, 1),
            result(true, None, 1),
            result(true, None, 1),
            result(true, None, 1),
        ];
        rows[0].local_risk = Some(j(RiskLevel::Safe));
        rows[0].reference_risk = Some(j(RiskLevel::Safe));
        rows[1].local_risk = Some(j(RiskLevel::Safe));
        rows[1].reference_risk = Some(j(RiskLevel::High));
        rows[2].local_risk = Some(j(RiskLevel::High)); // no reference label
        assert_eq!(risk_agreement(rows.iter()), (Some(0.5), 1));
        assert_eq!(risk_agreement(std::iter::empty()), (None, 0));
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

    #[test]
    fn excludes_unsourced_confidence() {
        // A backend that cannot measure reports `Unknown` and the harness
        // records no confidence for it: it must count toward coverage's
        // denominator only, never toward Brier/ECE.
        let mut unknown = result(true, None, 5);
        unknown.confidence_source = Some(crate::models::ConfidenceSource::Unknown);
        let measured = result(false, Some(0.2), 5);
        let rows = [unknown, measured];
        let roll = CalibrationRollup::of(rows.iter());
        assert!((roll.coverage - 0.5).abs() < 1e-6);
        assert!((roll.brier.unwrap() - 0.04).abs() < 1e-6);
        let counts = source_counts(rows.iter());
        assert_eq!(
            counts.get(&crate::models::ConfidenceSource::Unknown),
            Some(&1)
        );
        assert_eq!(
            counts.get(&crate::models::ConfidenceSource::Measured),
            Some(&1)
        );
    }
}
