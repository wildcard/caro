# ADR-017: Typed Decisions and Calibrated Confidence for Pipeline Gates

**Status**: Proposed

**Date**: 2026-09-22

**Authors**: Caro maintainers

**Target**: Community

## Context

caro's generation pipeline is a sequence of *gates* around one free-text
step. The static matcher decides whether it can answer; the agent loop
decides whether to refine or escalate to a frontier advisor; the safety layer
decides whether to auto-run, ask, or block; the prompt decides whether to ask
a clarifying question instead of answering. Each gate is a bounded decision,
yet only one of them — the `--approval smart` risk judge — is built as one:
a strict JSON verdict over an enum, a confidence floor of 0.7, and a hard
floor that a static `Critical` can never be relaxed
(`src/safety/mod.rs::blend_smart_decision`).

Every other gate is either free text (`QUESTION:` prefix, `echo 'Please
clarify'`) or keyed on `GeneratedCommand.confidence_score`, which today is a
constant per backend (static 1.0, embedded 0.85, claude 0.95, ollama 0.8,
AI-Horde 0.75). The refinement gate at `src/agent/mod.rs:338`
(`confidence_score < 0.8`) therefore cannot fire for most backends and is
effectively a backend switch. The evaluation harness could not detect this
because it never recorded confidence.

TypeSafe AI's "System One" work (see
`docs/research/jev-system-one-gap-analysis.md`) names this pattern:
decisions, not text; a probability the model is trained to make honest;
narrow questions instead of one-shot prompts; and evals that score
calibration and latency, not just accuracy. Their MIT adapter shows the same
contract can be enforced on an ordinary LLM with structured outputs and
corrective retries.

Constraints: caro is local-first and AGPL; the deterministic safety floor is
non-negotiable; the shipped embedded model is 135M parameters and cannot be
retrained for calibration in the near term.

## Decision

1. **Introduce `caro::decision`** with three primitives — `Noul` (yes/no with
   `p_yes`), `Choice<T>` (normalised distribution over a bounded label set),
   `Score` (value + confidence) — and one parser, `parse_choice_json`, that
   accepts both the "discrete" and "probabilities" answer modes and returns
   `None` on any type error. The risk judge is the first consumer; new gates
   must use these types rather than ad-hoc JSON.

2. **Two invariants for every decision gate**, inherited from the risk judge:
   - a decision is advisory *above* the deterministic safety floor and can
     never relax a static `Critical`;
   - a decision that fails to parse, or whose confidence is below the gate's
     floor, is `None` and the gate takes its default path.

3. **Confidence is measured or absent.** `confidence_score` must come from
   evidence (keyword coverage, token log-probs, provider `logprobs`, or a
   parsed self-report through `Choice`) or be reported as `None`. A constant
   is a defect. Migration of the existing constants is tracked as follow-up
   work and does not block this ADR.

4. **Eval reports calibration and tail latency.** `EvaluationResult` carries
   `confidence`; `BackendResult` carries `brier`, `ece`,
   `p50_execution_time_ms`, `p95_execution_time_ms`
   (`src/evaluation/calibration.rs`). Once a baseline exists, an ECE
   regression is treated like a CSR regression.

5. **Migration order for gates** (one PR each, each carrying its own
   regression guard): `needs_clarification: Noul` → `intent_category:
   Choice<_>` → `platform_fix_needed: Noul` → replace `confidence_score`
   constants → constrained decoding for decision prompts on backends that
   support it.

## Rationale

- The risk judge already proves the shape works in caro: it improved
  coverage without touching the safety floor. Generalising it is cheaper
  than inventing a second pattern.
- A typed parser makes an invalid label impossible locally, which is the
  no-training equivalent of Jev's "never makes type errors".
- Calibration metrics are pure functions over data the harness already
  collects; they cost nothing at runtime and turn a silent design flaw into
  a number in a report.
- Keeping free-text generation free-text avoids the trap of forcing an
  open-ended task into a classifier.

## Consequences

### Benefits

- The refinement/advisor gate becomes a real decision once confidence is
  measured; until then the eval report shows exactly how fake it is.
- New gates share one vocabulary, one parser, one set of tests.
- Latency tails become visible per backend, which is what users feel.
- Consensus-labelled decisions become clean training pairs for the SFT/DPO
  pipeline.

### Trade-offs

- One more module and two more report fields to maintain.
- Until constants are replaced, Brier/ECE describe the constants, not the
  models; readers must know that.
- Constrained decoding is backend-specific and adds per-backend code.

### Risks

- Risk: a measured confidence is *worse* calibrated than the constant for
  some backend → Mitigation: the eval reports it; the floor keeps the gate
  fail-safe either way.
- Risk: gates proliferate and add latency → Mitigation: p95 per gate is in
  the report; Jev's 70–500 ms band is the budget reference.
- Risk: teams read a typed decision as a safety guarantee → Mitigation:
  invariant 2.1 is documented in the module and enforced by
  `blend_smart_decision` tests.

## Alternatives Considered

### Alternative 1: Adopt TypeSafe's hosted API for the gates

Fast and calibrated out of the box, but every decision would leave the
machine. Conflicts with caro's local-first promise and the off-host warning
already attached to `--approval smart`. **Rejected.**

### Alternative 2: Turn command generation itself into a classification

Would inherit Jev's guarantees end to end, but shell commands are open-ended
text; a bounded label set cannot express them. **Rejected.**

### Alternative 3: Do nothing

The gates keep switching on backend identity while presenting as confidence
thresholds, and the eval keeps being unable to say so. **Rejected.**

### Alternative 4: Only add the eval metrics, no decision module

Cheaper, but the next gate would be written as ad-hoc JSON again. The module
is ~200 lines and the risk judge refactor is behaviour-preserving. **Rejected
in favour of shipping both.**

## References

- `docs/research/jev-system-one-gap-analysis.md`
- <https://typesafe.ai/blog/introducing-system-one-models-and-jev>
- <https://evals.typesafe.ai/>
- <https://github.com/typesafe-ai/system-one-adapter-python> (MIT)
- ADR-015 — frontier advisor path used as the reference labeller
- `src/prompts/risk_judge.rs`, `src/safety/mod.rs` (`blend_smart_decision`)
- `docs/ml/sft-data-pipeline.md`

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-09-22 | Caro maintainers | Initial draft, Proposed |
