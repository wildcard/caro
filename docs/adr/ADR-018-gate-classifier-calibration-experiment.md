# ADR-018: Gate-Classifier Experiment with a Calibration Objective

**Status**: Proposed

**Date**: 2026-10-03

**Authors**: Caro maintainers

**Target**: Community

## Context

ADR-017 turned caro's pipeline gates into typed decisions that carry a
probability and a source, and Phases 1 to 3 of the Jev strategy
(`docs/research/jev-of-execution-safety-strategy.md`, epic #1460) gave
those probabilities an honest origin and a public scoreboard: measured
confidence per backend, Brier and Expected Calibration Error (ECE) in the
evaluation harness, an ECE regression gate, consensus risk labels from a
reference judge, and the published `caro.sh/evals` page.

Today every gate probability still comes from a prompt. The `--approval
smart` risk judge asks an LLM for a verdict and a confidence; the static
matcher reports keyword coverage. On the committed snapshot
(main@9debe6e) the static matcher sits at ECE 0.153 over 63% coverage, and
no LLM judge has a published ECE yet. TypeSafe AI's System One result, the
one this strategy is built on, is that a *small model trained on the
decision itself with a proper-scoring objective* is better calibrated and
faster than a general model asked to self-report. Phase 4 is the
experiment that tests whether that holds for caro's gates.

The strategy document numbered this ADR 019 and reserved 018 for a
decision-API ADR that Phase 2 shipped without. ADR numbers are sequential
with no gaps (`.claude/rules/adr-numbering.md`), so the gate classifier is
ADR-018.

Constraints:

- Training is gated on data, not enthusiasm. The consensus-label exporter
  (`src/evaluation/sft_export.rs::decision_label_pairs`) exists, but until
  this change nothing wrote its output to disk, so there are zero records.
- The static safety floor is not negotiable: a learned gate is advisory
  above `blend_smart_decision`'s floor and can never relax a static
  `Critical`.
- The target hardware is the `ml-ds-engineer` role's M4 Max with 48 GB of
  unified memory; the shipped artifact, if any, must fit the embedded
  backend's envelope.
- Command *generation* stays free text. Only the bounded decisions (risk,
  injection, needs_clarification) are candidates for a classifier.

## Decision

Run a bounded experiment, tracked in #1510, and record its outcome in this
ADR before any model is wired into the product.

1. **Data gate first.** The evaluation binary gains
   `CARO_EVAL_EXPORT_LABELS=<path>.jsonl`, which writes
   `decision_label_pairs` for a run that had both a local and a reference
   risk verdict. The experiment does not start until at least one judged
   run over the 101-case dataset is committed under
   `tests/evaluation/results/` with the backend, the judge and the commit
   recorded, and the corpus holds at least 500 `accepted`/`corrected`
   records from at least two judged backends. If fewer than 10% of records
   are `corrected`, the reference judge is adding no signal and the
   experiment stops there with that finding.
2. **Baseline before training.** The harness's ECE, Brier, agreement and
   p95 latency for the existing prompt-based risk gate, per backend, are
   the before column. They come from the same run that produced the data.
3. **Two candidates, one objective.** Candidate A is a small classifier
   (logistic or a two-layer MLP) over tokenizer features of
   `(prompt, command)`. Candidate B is a LoRA adapter on the smoke model
   (SmolLM-135M, falling back to Qwen2.5-Coder-1.5B) restricted to the
   risk gate. Both train with a proper-scoring loss (negative
   log-likelihood or Brier), not accuracy, because the deliverable is a
   calibrated probability.
4. **Held-out by test id**, never by record, so the same prompt cannot
   appear on both sides.
5. **Report the after column** in the same table: ECE, Brier, agreement
   with the reference, p95 decision latency. A candidate that raises
   agreement but worsens ECE is not a win. A candidate that wins on both is
   proposed, in a follow-up ADR, for the advisory slot above the safety
   floor. Either way this ADR moves to Accepted with the numbers in it.

## Rationale

- **Calibration is the product claim.** `caro.sh/evals` publishes ECE
  because the strategy's positioning is "the calibrated, deterministic-
  floored decision layer for execution safety". A gate trained for
  calibration is the direct test of that claim; a gate tuned for accuracy
  alone would undercut it.
- **The eval already measures what the experiment needs.** Brier, ECE,
  agreement and latency are harness columns since #1466, and the ECE
  regression gate already fails a run that gets worse. The experiment
  reuses that machinery rather than inventing a training-only metric.
- **Data gating protects against demoware.** The project's validation
  discipline warns against mistaking a built thing for a validated one.
  Requiring a committed judged run and a minimum corrected share forces
  the question "does the reference judge disagree with us often enough to
  teach anything?" before any GPU time is spent.
- **Two cheap candidates bound the cost.** Candidate A trains in seconds
  on a laptop; Candidate B is a LoRA on a 135M-parameter model. If neither
  beats the prompt-based gate on ECE, the honest outcome is "prompts are
  good enough for now", and that is recorded.

## Consequences

### Benefits

- A before/after ECE table for the risk gate, whichever way it falls.
- Consensus-label data starts accumulating from every judged eval run.
- The decision about shipping a learned gate is made on numbers already
  published at `caro.sh/evals`, not on a demo.

### Trade-offs

- Judged runs cost a model call per generated command, so the data gate
  needs a maintainer to run the eval against a live Ollama or vLLM; CI
  cannot produce the data.
- The reference label is a model's verdict, not ground truth. The
  experiment measures agreement with the stronger judge, which is exactly
  what the Pareto view measures, and says so wherever the numbers appear.
- Two candidates and a held-out split are a small experiment by design;
  they will not settle whether a larger model would do better.

### Risks

- Corpus too small or too agreeable: mitigated by the explicit 500-record
  and 10%-corrected thresholds, which turn "not enough data" into a
  recorded finding rather than a stalled branch.
- Learned gate over-trusted: mitigated by keeping it advisory above the
  static floor, and by the ECE regression gate blocking any release where
  it worsens calibration.
- Benchmark leakage: mitigated by the test-id split and by excluding
  Safety-category cases from the training feed, as `decision_label_pairs`
  already does.

## Alternatives Considered

### Alternative 1: Keep prompting, tune the prompt for ECE

The `prompt-tuner` skill already optimises for the success rate and is
being extended to watch ECE. Cheaper, but it cannot change the fact that a
general model's self-reported confidence is weakly tied to its error rate.
It stays the control arm: the before column is exactly this.

### Alternative 2: Fine-tune the whole embedded model on decisions

Teaches the generator to classify, mixing two jobs in one set of weights
and risking the generation quality the embedded path is judged on. The
strategy keeps generation free text on purpose.

### Alternative 3: Adopt a hosted decision API

TypeSafe's hosted gates would give calibrated probabilities tomorrow, at
the price of sending every command off-host. It contradicts caro's
local-first privacy boundary (`src/ai/privacy.rs`) and was rejected in
ADR-017 for the same reason.

### Alternative 4: Do nothing until users ask

The gates work today. But the published ECE is the project's own
scoreboard, and "we never tried to improve it" is a worse answer than a
recorded negative result.

## References

- `docs/research/jev-of-execution-safety-strategy.md`, Phase 4
- `docs/research/jev-system-one-gap-analysis.md`
- ADR-017: Typed Decisions and Calibrated Confidence for Pipeline Gates
- `docs/ml/sft-data-pipeline.md`, Source 3
- `src/evaluation/sft_export.rs`, `src/evaluation/calibration.rs`
- Tracking issue #1510; epic #1460 (Phases 1 to 3)

## Revision History

| Date | Change |
| --- | --- |
| 2026-10-03 | Proposed. Data gate and experiment design; export hook landed with this ADR. Before/after table pending the first judged run. |
