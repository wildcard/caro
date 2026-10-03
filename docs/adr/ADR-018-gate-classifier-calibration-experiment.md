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
matcher reports keyword coverage. TypeSafe AI's System One result, the one
this strategy is built on, is that a *small model trained on the decision
itself with a proper-scoring objective* is better calibrated and faster
than a general model asked to self-report. Phase 4 is the experiment that
tests whether that holds for caro's gates. It is a technical exploration,
not a feature: nothing user-facing changes until a follow-up ADR proposes
it, and that follow-up is what the validation-discipline gates apply to in
full. This ADR adopts Gate 3 (demoware trap) and Gate 4 (devil's-advocate
review) voluntarily because the experiment's outcome is meant to inform a
shipping decision.

The strategy document numbered this ADR 019 and reserved 018 for a
decision-API ADR that Phase 2 shipped without. ADR numbers are sequential
with no gaps (`.claude/rules/adr-numbering.md`), so the gate classifier is
ADR-018.

What the repository actually has, as of this ADR:

- A consensus-label exporter (`src/evaluation/sft_export.rs::
  decision_label_pairs`) that pairs the local backend's risk verdict with
  a reference judge's. Until this change nothing wrote its output to disk,
  so there are zero records. It used to drop Safety-category cases, which
  censored exactly the high-risk region a risk gate most needs; this change
  keeps them and adds the `test_id` each record came from. It still drops
  agreements below confidence 0.7, which biases the corpus toward confident
  cases; the experiment has to account for that, not inherit it silently.
- Harness calibration metrics (`src/evaluation/calibration.rs`) that score
  a backend's *generation* confidence against whether the generated command
  passed. They do not score the risk gate. There is no risk-gate ECE
  anywhere yet, for any backend, and the static matcher's published ECE of
  0.153 is a generation number, not a gate number.
- Three case counts in circulation that are different suites, not a
  discrepancy: the 101-case `tests/evaluation/dataset.yaml` the harness
  runs in CI; the 58-case beta suite from January 2026 behind the 94.8%
  Command Success Rate; and the 55 TOML cases in
  `tests/evaluation/test_cases.toml` the strategy document counted. This
  experiment uses the 101-case harness dataset for training data and must
  not hold out on it alone (see Decision, item 5).
- A static safety floor (`blend_smart_decision`) that protects the
  `Critical` tier only. A learned gate acting between `Safe`, `Moderate`
  and `High` can relax approval friction without ever touching that floor.
- A training population that is maintainer-authored prompts run against a
  maintainer-chosen judge. No user traffic is involved, and the telemetry
  fields that would describe real traffic are not yet collected.

## Decision

Run a bounded, pre-registered experiment tracked in #1510, and record its
outcome in this ADR before any model is wired into the product.

1. **Data path.** The evaluation binary gains
   `CARO_EVAL_EXPORT_LABELS=<path>.jsonl`, which appends
   `decision_label_pairs` for a run that had both a local and a reference
   risk verdict; a run that produced none leaves the file untouched, so the
   corpus accumulates across judged runs. Each record carries its
   `test_id`. These are the only code changes in this ADR.
2. **Pilot before thresholds.** The first judged run over the 101-case
   dataset (one backend, one reference judge) is committed under
   `tests/evaluation/results/` with the backend, judge, prompt template
   and commit recorded, and its `corrected` share and per-tier counts are
   reported in #1510. The data gate below is then set from those counts,
   not guessed in advance. Until the pilot exists the gate reads
   "unknown".
3. **Risk-gate baseline, not generation ECE.** The before column is the
   existing prompt-based judge's calibration *as a risk gate*: its
   confidence scored against whether its verdict matched the reference,
   and, on the gold subset below, against the human label. Computing this
   is a harness change (a `risk_gate` calibration keyed on
   `local_risk`/`reference_risk`, bootstrap confidence intervals included)
   and is a gating item in #1510. No training run is approved until this
   baseline exists for every backend in the corpus.
4. **A human gold subset.** Between 100 and 150 commands, stratified
   across risk tiers and including Safety-category cases the exporter
   drops, labelled by a maintainer with the label rules written down.
   The reference judge's own ECE and disagreement against this subset are
   reported first; if the judge is far from gold, the experiment's target
   becomes gold agreement, not judge agreement. This is the direct answer
   to "a student of an uncalibrated teacher".
5. **Data gate**, finalised after the pilot and recorded in #1510, in
   terms that bound variance rather than raw counts: a minimum number of
   records per populated confidence bin on the held-out set (the default
   proposal is 30), a minimum number of unique held-out test ids, and a
   held-out set that includes commands which have never been part of the
   published eval so the experiment does not tune on the scoreboard. The
   `corrected` share is reported as a finding; a low share is evidence the
   local judge is already close to the reference, not a reason to stop.
6. **Candidates, cheapest first.**
   - Candidate 0: post-hoc recalibration of the existing judge's
     confidence (temperature scaling, Platt scaling or isotonic
     regression) on the same labels. No new model, no GPU. If this closes
     most of the gap, the learned gate has to beat it, not the raw judge.
   - Candidate A: a small classifier (logistic or a two-layer MLP) over
     tokenizer features of `(prompt, command)`.
   - Candidate B: a LoRA adapter on the smoke model (SmolLM-135M, falling
     back to Qwen2.5-Coder-1.5B) restricted to the risk gate.
   All train with a proper-scoring loss. Candidate B is run only if A
   beats Candidate 0.
7. **Pre-registered success and stop rules.** Success: held-out risk-gate
   ECE improves over Candidate 0 with a bootstrap 95% interval excluding
   zero, agreement with gold does not fall, and no risk tier gets worse.
   Stop: the pilot shows the data gate cannot be met with the resources
   available, or the judge's tier disagreement with gold exceeds the
   maximum fixed in #1510 before any gold label is reviewed (the default
   proposal is 15 percentage points). Either outcome moves this ADR to Accepted
   with the numbers in it; "Accepted" records the experiment's result,
   not approval to ship.
8. **One-sided by construction.** Any learned gate that is later proposed
   may only raise a risk tier relative to the static patterns, never
   lower one, and stays advisory above the `Critical` floor. Results are
   reported per tier so a `High` to `Moderate` relaxation cannot hide in
   an aggregate.

## Rationale

- **The scoreboard is public, so the gate's calibration has to be real.**
  `caro.sh/evals` publishes Brier and ECE. Whether a trained gate is
  better calibrated than a prompted one is the internal hypothesis behind
  the strategy's positioning; this experiment tests that hypothesis. It is
  not evidence of demand, and the ADR makes no claim about users.
- **Judge agreement is a proxy and is treated as one.** Training against
  the reference teaches agreement with a model, not correctness. The gold
  subset exists so the proxy's error is a measured number and so the
  experiment can switch targets if the proxy is poor.
- **Cheapest candidate first bounds the conclusion.** Without Candidate 0
  the experiment could not tell "a learned gate wins" from "any
  calibration wins". With it, a learned model has to earn its complexity.
- **Pre-registration protects against noise.** With a held-out set drawn
  from a 101-case dataset, bootstrap intervals on ECE will be wide. A
  stated success rule with an interval requirement turns "0.153 versus
  0.12" from a headline into a question with an answer.
- **Data gating protects against demoware.** Requiring a committed pilot,
  a gold subset and a real risk-gate baseline before any training run
  forces the question "do we have the signal to learn anything?" before
  any GPU time is spent.

## What breaks at 100 real users

This is the Gate 3 section the validation discipline asks for. The
experiment itself is local and single-user, so nothing breaks while it
runs. The path it opens, a learned gate in the approval flow, has these
failure modes, and a follow-up ADR that proposes shipping it must address
each:

- **Distribution shift.** The classifier is trained on about a hundred
  maintainer-authored prompts. Real users issue long-tail commands (pipes,
  `find -exec`, `kubectl`, cloud CLIs). A small classifier over tokenizer
  features will not extrapolate, and its confidence out of distribution is
  unconstrained. In-distribution ECE does not predict production ECE.
  Instrumentation: the opt-in telemetry fields for gate confidence and
  outcome from `docs/strategy/TELEMETRY_STRATEGY.md`, which do not exist
  yet and are a prerequisite for shipping. Fallback: the prompt-based
  judge, which stays in the binary.
- **The regression gate cannot see production.** `compare_with_ece` runs
  on the same 101 cases, so calibration drift in the field is invisible
  to CI. The same telemetry is the only fix.
- **Retraining has no owner.** A model artifact goes stale when the judge,
  the prompt template or the dataset changes. The shipping ADR must name
  the retraining trigger and who runs it, or the gate is not shippable.
- **Switch-off.** Any shipped gate needs a configuration flag that
  restores the prompt-based judge without a release, and the per-tier
  one-sided rule above so a bad model can only add friction, never remove
  it.

## Consequences

### Benefits

- A measured risk-gate calibration baseline for the existing judge, which
  does not exist today and is useful whatever the experiment finds.
- A human gold subset for risk labels, reusable by the eval and by the
  prompt-tuner.
- Consensus-label data starts accumulating from every judged eval run.
- A recorded, falsifiable answer to "is a trained gate better calibrated
  than a prompted one on caro's data?"

### Trade-offs

- Judged runs cost a model call per generated command, so the data gate
  needs a maintainer to run the eval against a live Ollama or vLLM; CI
  cannot produce the data.
- The gold subset costs a maintainer an afternoon of labelling and a
  written label guide.
- The experiment is small by design and will not settle whether a larger
  model would do better.

### Risks

- Corpus too small or too agreeable: mitigated by the pilot-first rule
  and variance-based gate, which turn "not enough data" into a recorded
  finding rather than a stalled branch.
- Learned gate over-trusted: mitigated by the one-sided rule, the
  `Critical` floor, per-tier reporting and the ECE regression gate.
- Benchmark leakage: mitigated by the test-id split, by a held-out set
  that includes commands outside the published eval, and by reporting
  unique-id counts with every number.
- Judge and student share blind spots: mitigated by the gold subset, which
  is the only label source independent of both.

## Alternatives Considered

### Alternative 1: Keep prompting, tune the prompt for ECE

The `prompt-tuner` skill already optimises for the success rate and is
being extended to watch ECE. It stays the control arm: the before column
is exactly this, measured for the first time as a risk gate.

### Alternative 2: Post-hoc recalibration only

Temperature, Platt or isotonic scaling of the existing judge's confidence
needs the same labels and no new model. Rather than dismiss it, this ADR
makes it Candidate 0: a learned gate is justified only if it beats it.

### Alternative 3: Fine-tune the whole embedded model on decisions

Teaches the generator to classify, mixing two jobs in one set of weights
and risking the generation quality the embedded path is judged on. The
strategy keeps generation free text on purpose.

### Alternative 4: Adopt a hosted decision API

TypeSafe's hosted gates would give calibrated probabilities tomorrow, at
the price of sending every command off-host. It contradicts caro's
local-first privacy boundary (`src/ai/privacy.rs`) and was rejected in
ADR-017 for the same reason.

### Alternative 5: Do nothing

A legitimate outcome, and the one this experiment produces if Candidate 0
or the raw judge is already well calibrated against gold. No user has
asked for a learned risk gate; the only reason to run the experiment is
that the published ECE is caro's own scoreboard and the maintainers want
to know whether it can be moved cheaply. If the pilot or baseline shows
it cannot, this ADR records that and stops.

## References

- `docs/research/jev-of-execution-safety-strategy.md`, Phase 4
- `docs/research/jev-system-one-gap-analysis.md`
- ADR-017: Typed Decisions and Calibrated Confidence for Pipeline Gates
- `docs/ml/sft-data-pipeline.md`, Source 4
- `src/evaluation/sft_export.rs`, `src/evaluation/calibration.rs`
- `.claude/rules/validation-discipline.md`, Gates 3 and 4
- Tracking issue #1510; epic #1460 (Phases 1 to 3)
- Devil's Advocate Review on PR #1511 (Gate 4), whose objections shaped
  items 2 to 8 of the Decision

## Revision History

| Date | Change |
| --- | --- |
| 2026-10-03 | Proposed. Export hook landed with this ADR. Revised after the Gate 4 devil's-advocate review: pilot-first data gate, risk-gate baseline instead of generation ECE, human gold subset, Candidate 0 (post-hoc recalibration), pre-registered success and stop rules, one-sided per-tier rule, Gate 3 section, positioning reframed as an internal hypothesis. Before/after table pending the pilot. |
