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

/// Serialize records to JSONL (one compact JSON object per line).
///
/// The caller owns file IO; this stays pure so it is trivially testable. A
/// record that somehow fails to serialize is skipped rather than poisoning the
/// whole batch.
pub fn to_jsonl(records: &[SftRecord]) -> String {
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
            execution: None,
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
    fn empty_input_yields_empty_output() {
        assert!(passing_trajectories(&[], &[]).is_empty());
        assert_eq!(to_jsonl(&[]), "");
    }
}
