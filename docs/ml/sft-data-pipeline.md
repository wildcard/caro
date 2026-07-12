# SFT Data Pipeline — Design

**Status:** Phase 2 of the Fireworks "Open-Source Agents, Frontier Advisors"
learnings (see `.claude/plans/what-we-can-learn-partitioned-knuth.md`).
**Owner:** `ml-ds-engineer` role.
**Deliverable of this phase:** a dataset *builder* + this doc. **Not** a trained
model — training is gated on an eval baseline (Phase 1) and a separate go/no-go.

## Why

The article's most actionable supporting finding: supervised fine-tuning on
*passing benchmark trajectories* of a small open-weights model is a cheap, real
quality gain (Kimi K2.6 all-pass 11→15 from SFT alone, ~$84). caro already
produces passing trajectories on every eval run and records user corrections via
the `knowledge` feature. The data exists; this pipeline collects it.

## Sources

| # | Source | Record shape | Status |
|---|--------|--------------|--------|
| 1 | Passing eval trajectories | `{prompt, command, backend, category, score}` | **Implemented** — `src/evaluation/sft_export.rs::passing_trajectories` |
| 2 | `knowledge` correction log | `{prompt, rejected_command, accepted_command}` → preference pair | Specified below (not yet implemented) |
| 3 | Live-session accepted commands | same as #1 but from real use | Future; strict privacy gate required |

### Source 1 — passing eval trajectories (done)

`passing_trajectories(results, dataset)` keeps only `passed` results with a
non-empty command and **excludes the Safety category** (a "passed" safety case
is usually a *block*, not a generation target — training on it would teach the
model to emit dangerous commands). Output is JSONL via `to_jsonl`. Pure, no IO;
the caller (an eval-run script or the `caro test` path) writes the file.

Each record carries `score` (mean-score of the originating result) so a
downstream filter can threshold quality once multi-criterion cases exist
(Phase 1b).

### Source 2 — correction log → preference pairs (next)

`AgentLoop::record_correction(prompt, original, refined, feedback)`
(`src/agent/mod.rs`) already writes `{prompt, rejected, accepted}` triples into
the `knowledge` vector DB when a refinement changed the command. Harvesting
these as DPO-style preference pairs is the natural next increment:

- Add a read-side export to the `knowledge` module (feature = `knowledge`) that
  streams stored corrections as `{prompt, chosen: accepted, rejected: original}`.
- These are **preference pairs** (DPO), complementary to Source 1's SFT targets.
- Implementation is feature-gated and touches the DB layer, so it ships
  separately from the pure exporter to keep this phase mergeable.

## Privacy boundary

- **Sources 1** is benchmark data (authored prompts in
  `tests/evaluation/dataset.yaml`, model-generated commands) — no real user
  data, low risk.
- **Sources 2 and 3** may contain real user prompts/commands. Before any record
  leaves the host it MUST pass the redaction rules in `src/ai/privacy.rs`
  (the same boundary that gates off-host context: no raw paths, no secrets,
  honor the opt-in toggles). A record that cannot be safely redacted is dropped,
  not emitted.

## Training target (not in this phase)

- **Hardware envelope:** M4 Max, 48 GB unified memory (the `ml-ds-engineer`
  role's documented target).
- **Method:** LoRA fine-tune of the embedded base model (Qwen 2.5 Coder 1.5B)
  on Source 1 (+ Source 2 once available).
- **Gate:** do **not** spend GPU time until (a) Phase 1's eval baseline exists
  so before/after is measurable, and (b) a LoRA-config + eval-baseline PR lands
  per the role's "config before GPU" rule.
- **Success metric:** all-pass rate on the eval suite must improve vs the base
  model baseline, with no Safety-category regression and no cost/latency
  regression on the embedded path.

## What ships in Phase 2

1. `src/evaluation/sft_export.rs` — pure exporter + tests (done).
2. This design doc (done).
3. Follow-ups filed: Source 2 correction-log export; an eval-run wiring that
   writes the JSONL artifact; the LoRA-config + baseline PR.
