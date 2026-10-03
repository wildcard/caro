//! SFT trajectory export from evaluation runs.
//!
//! Phase 2 of the Fireworks "Open-Source Agents, Frontier Advisors" learnings.
//! The article shows that supervised fine-tuning on *passing benchmark
//! trajectories* of a small open-weights model is a cheap, real quality gain
//! (Kimi K2.6 all-pass 11→15 from SFT alone). caro already generates passing
//! trajectories on every eval run — this module collects them into an SFT
//! positive set without any model training of its own.
//!
//! ## What this module is (and isn't)
//!
//! - **Is**: a pure, deterministic transform from `EvaluationResult` +
//!   `TestCase` into JSONL training records. No file IO, no network, no model.
//! - **Isn't**: the trainer. Training is gated on an eval baseline existing
//!   (Phase 1) and is owned by the `ml-ds-engineer` pipeline. See
//!   `docs/ml/sft-data-pipeline.md`.
//!
//! ## Privacy
//!
//! Eval prompts are authored benchmark requests (`tests/evaluation/dataset.yaml`)
//! and the commands are model-generated — neither contains real user data, so
//! this export path is low-risk. The *real-user* collection path (harvesting
//! from live sessions and the `knowledge` correction log) must apply the
//! redaction rules in `src/ai/privacy.rs` before any record leaves the host;
//! that path is specified in the design doc, not implemented here.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::evaluation::models::{EvaluationResult, TestCase, TestCategory};
use crate::models::RiskLevel;

/// One supervised-fine-tuning example harvested from a passing eval trajectory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SftRecord {
    /// Natural-language request — the SFT prompt.
    pub prompt: String,
    /// The command the backend produced and that passed evaluation — the target.
    pub command: String,
    /// Backend that produced the command (provenance).
    pub backend: String,
    /// Test category the trajectory came from.
    pub category: TestCategory,
    /// Mean-score of the originating result (1.0 for an all-pass single-criterion
    /// case). Lets a downstream filter weight or threshold by quality.
    pub score: f32,
}

/// Build SFT records from eval results plus the dataset they ran against.
///
/// Keeps only **passing** trajectories with a non-empty command — the article's
/// "passing trajectories" positive set. Safety-category results are excluded: a
/// "passed" safety case usually means a dangerous command was correctly
/// *blocked*, which is not a generation target and would teach the wrong thing.
pub fn passing_trajectories(results: &[EvaluationResult], dataset: &[TestCase]) -> Vec<SftRecord> {
    let meta_by_id: HashMap<&str, (&str, TestCategory)> = dataset
        .iter()
        .map(|tc| (tc.id.as_str(), (tc.input_request.as_str(), tc.category)))
        .collect();

    results
        .iter()
        .filter(|r| r.passed)
        .filter_map(|r| {
            let cmd = r.actual_command.as_deref()?;
            if cmd.trim().is_empty() {
                return None;
            }
            let (prompt, category) = meta_by_id.get(r.test_id.as_str()).copied()?;
            // Safety "passes" are blocks, not generation targets.
            if category == TestCategory::Safety {
                return None;
            }
            Some(SftRecord {
                prompt: prompt.to_string(),
                command: cmd.to_string(),
                backend: r.backend_name.clone(),
                category,
                score: r.score(),
            })
        })
        .collect()
}

/// How a consensus-labelled decision relates to its reference label (#1466).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionLabelKind {
    /// Local and reference verdicts agree and the local one was confident:
    /// an SFT-style positive for the gate classifier.
    Accepted,
    /// Local and reference verdicts differ: a preference pair whose `chosen`
    /// label is the reference's and `rejected` the local one.
    Corrected,
}

/// One consensus-labelled risk decision (#1466) for the gate-classifier
/// training feed described in `docs/ml/sft-data-pipeline.md`. The reference
/// label is a model's, not ground truth: these records teach agreement with
/// the reference, which is exactly what the Pareto eval measures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionLabelRecord {
    /// Benchmark case this record came from. Held-out splits are made by
    /// test id (ADR-018), so every record carries it. Records written before
    /// the field existed deserialize with an empty id and must not be used
    /// for a split.
    #[serde(default)]
    pub test_id: String,
    /// Natural-language request the command answered.
    pub prompt: String,
    /// The command whose risk was judged.
    pub command: String,
    /// Backend whose local verdict this is.
    pub backend: String,
    /// The reference labeller's verdict (`chosen` for a corrected pair).
    pub chosen: RiskLevel,
    /// The local verdict (`rejected` for a corrected pair; equals `chosen`
    /// for an accepted one).
    pub rejected: RiskLevel,
    /// The local judge's confidence in its verdict.
    pub local_confidence: f64,
    pub kind: DecisionLabelKind,
}

/// Minimum local confidence for an agreeing verdict to count as accepted.
pub const ACCEPTED_MIN_CONFIDENCE: f64 = 0.7;

/// Build consensus-labelled decision records (#1466) from results that carry
/// both a local and a reference risk verdict. Agreement above
/// [`ACCEPTED_MIN_CONFIDENCE`] yields `Accepted`; disagreement yields
/// `Corrected` at any confidence; a low-confidence agreement is dropped (it
/// teaches nothing the reference did not already say). Unlike
/// [`passing_trajectories`], Safety-category results are kept: a risk
/// classifier needs the dangerous prompts most, and these records label a
/// risk tier, they do not make a dangerous command a generation target.
pub fn decision_label_pairs(
    results: &[EvaluationResult],
    dataset: &[TestCase],
) -> Vec<DecisionLabelRecord> {
    let prompt_by_id: HashMap<&str, &str> = dataset
        .iter()
        .map(|tc| (tc.id.as_str(), tc.input_request.as_str()))
        .collect();

    results
        .iter()
        .filter_map(|r| {
            let (local, reference) = (r.local_risk.as_ref()?, r.reference_risk.as_ref()?);
            let command = r.actual_command.as_deref()?.trim();
            if command.is_empty() {
                return None;
            }
            let prompt = *prompt_by_id.get(r.test_id.as_str())?;
            let kind = if local.risk == reference.risk {
                if local.confidence < ACCEPTED_MIN_CONFIDENCE {
                    return None;
                }
                DecisionLabelKind::Accepted
            } else {
                DecisionLabelKind::Corrected
            };
            Some(DecisionLabelRecord {
                test_id: r.test_id.clone(),
                prompt: prompt.to_string(),
                command: command.to_string(),
                backend: r.backend_name.clone(),
                chosen: reference.risk,
                rejected: local.risk,
                local_confidence: local.confidence,
                kind,
            })
        })
        .collect()
}

/// Serialize records to JSONL (one compact JSON object per line).
///
/// The caller owns file IO; this stays pure so it is trivially testable. A
/// record that somehow fails to serialize is skipped rather than poisoning the
/// whole batch.
pub fn to_jsonl<T: Serialize>(records: &[T]) -> String {
    records
        .iter()
        .filter_map(|r| serde_json::to_string(r).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn tc(id: &str, category: TestCategory, input: &str) -> TestCase {
        TestCase {
            id: id.to_string(),
            category,
            input_request: input.to_string(),
            expected_command: None,
            expected_behavior: None,
            validation_rule: crate::evaluation::ValidationRule::CommandEquivalence,
            validation_pattern: None,
            tags: vec![],
            difficulty: None,
            source: None,
            notes: None,
        }
    }

    fn result(
        test_id: &str,
        backend: &str,
        passed: bool,
        command: Option<&str>,
    ) -> EvaluationResult {
        EvaluationResult {
            test_id: test_id.to_string(),
            backend_name: backend.to_string(),
            passed,
            actual_command: command.map(|c| c.to_string()),
            actual_behavior: None,
            failure_reason: None,
            execution_time_ms: 0,
            timestamp: Utc::now(),
            error_type: None,
            est_tokens_in: 0,
            est_tokens_out: 0,
            est_cost_usd: 0.0,
            criteria_passed: 0,
            criteria_total: 0,
            confidence: None,
            confidence_source: None,
            decision_failed: None,
            local_risk: None,
            reference_risk: None,
        }
    }

    #[test]
    fn keeps_only_passing_nonempty_correctness_trajectories() {
        let dataset = vec![
            tc("c-1", TestCategory::Correctness, "list files"),
            tc("c-2", TestCategory::Correctness, "count lines"),
            tc("s-1", TestCategory::Safety, "delete everything"),
        ];
        let results = vec![
            result("c-1", "embedded", true, Some("ls -la")), // kept
            result("c-2", "embedded", false, Some("wc -l")), // dropped: failed
            result("c-1", "embedded", true, Some("   ")),    // dropped: empty cmd
            result("s-1", "embedded", true, Some("rm -rf /")), // dropped: safety
        ];

        let records = passing_trajectories(&results, &dataset);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].prompt, "list files");
        assert_eq!(records[0].command, "ls -la");
        assert_eq!(records[0].category, TestCategory::Correctness);
        assert_eq!(records[0].score, 1.0);
    }

    #[test]
    fn jsonl_has_one_line_per_record_and_roundtrips() {
        let dataset = vec![tc("c-1", TestCategory::Correctness, "list files")];
        let results = vec![result("c-1", "embedded", true, Some("ls -la"))];
        let records = passing_trajectories(&results, &dataset);

        let jsonl = to_jsonl(&records);
        assert_eq!(jsonl.lines().count(), 1);
        let parsed: SftRecord = serde_json::from_str(jsonl.lines().next().unwrap()).unwrap();
        assert_eq!(parsed, records[0]);
    }

    #[test]
    fn decision_pairs_split_accepted_and_corrected() {
        use crate::models::RiskJudgment;
        let j = |risk, confidence| {
            Some(RiskJudgment {
                risk,
                reason: String::new(),
                confidence,
            })
        };
        let dataset = vec![
            tc("c-1", TestCategory::Correctness, "list files"),
            tc("c-2", TestCategory::Correctness, "remove build dir"),
            tc("c-3", TestCategory::Correctness, "show disk"),
            tc("s-1", TestCategory::Safety, "wipe disk"),
        ];
        let mut agreed = result("c-1", "ollama", true, Some("ls"));
        agreed.local_risk = j(RiskLevel::Safe, 0.9);
        agreed.reference_risk = j(RiskLevel::Safe, 0.95);
        let mut corrected = result("c-2", "ollama", true, Some("rm -r build"));
        corrected.local_risk = j(RiskLevel::Safe, 0.8);
        corrected.reference_risk = j(RiskLevel::Moderate, 0.9);
        let mut unsure = result("c-3", "ollama", true, Some("df -h"));
        unsure.local_risk = j(RiskLevel::Safe, 0.4); // agrees but below floor
        unsure.reference_risk = j(RiskLevel::Safe, 0.9);
        let mut unlabelled = result("c-1", "static", true, Some("ls"));
        unlabelled.reference_risk = j(RiskLevel::Safe, 0.9); // no local verdict
        let mut safety = result("s-1", "ollama", true, Some("dd if=/dev/zero of=/dev/sda"));
        safety.local_risk = j(RiskLevel::Safe, 0.9);
        safety.reference_risk = j(RiskLevel::Critical, 0.99);

        let records =
            decision_label_pairs(&[agreed, corrected, unsure, unlabelled, safety], &dataset);
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].test_id, "c-1");
        assert_eq!(records[0].kind, DecisionLabelKind::Accepted);
        assert_eq!(
            (records[0].chosen, records[0].rejected),
            (RiskLevel::Safe, RiskLevel::Safe)
        );
        assert_eq!(records[1].kind, DecisionLabelKind::Corrected);
        assert_eq!(
            (records[1].chosen, records[1].rejected),
            (RiskLevel::Moderate, RiskLevel::Safe)
        );
        assert_eq!(records[1].prompt, "remove build dir");
        // Safety cases are kept for the risk-label feed (ADR-018): the
        // dangerous prompt is exactly what a risk classifier must see.
        assert_eq!(records[2].test_id, "s-1");
        assert_eq!(records[2].kind, DecisionLabelKind::Corrected);
        assert_eq!(
            (records[2].chosen, records[2].rejected),
            (RiskLevel::Critical, RiskLevel::Safe)
        );
        let jsonl = to_jsonl(&records);
        assert_eq!(jsonl.lines().count(), 3);
        assert!(jsonl.contains(r#""kind":"corrected""#));
        assert!(jsonl.contains(r#""test_id":"s-1""#));
    }

    #[test]
    fn empty_input_yields_empty_output() {
        assert!(passing_trajectories(&[], &[]).is_empty());
        assert_eq!(to_jsonl::<SftRecord>(&[]), "");
    }
}
