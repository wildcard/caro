# Jev / System One vs Caro (caro): Gap Analysis

**Date:** 2026-09-22
**Purpose:** What caro can learn from TypeSafe AI's "System One" models (Jev),
and how to build on their public work without copying their product shape.
**Sources:**
- Announcement: <https://typesafe.ai/blog/introducing-system-one-models-and-jev>
- Workflow evals: <https://evals.typesafe.ai/>
- MIT adapter: <https://github.com/typesafe-ai/system-one-adapter-python>

---

## Executive Summary

TypeSafe AI's thesis is that most steps inside an automated workflow are
**decisions, not text**: yes/no, one-of-N, or a rating. They ship a model
(Jev) whose only output is a typed decision with a calibrated probability,
trained with what they call *Reinforcement Learning for Calibrated Decisions*
(RLCD) instead of RLHF, and evaluated by decomposing real workflows into
narrow questions rather than one-shot prompts.

**Key Finding:** caro already has one gate built exactly this way — the
`--approval smart` risk judge — and every other gate is either free text or
keyed on a *fake* number. `confidence_score` is a per-backend constant
(static 1.0, embedded 0.85, claude 0.95, ollama 0.8, AI-Horde 0.75), so the
agent loop's `confidence < 0.8` refinement gate, the advisor escalation and
the candidate-ranking scorer's `llm_confidence` feature are all silently
switching on *which backend answered*, not on *how sure it was*. Nothing in
the eval harness could have caught this because it never recorded confidence.

The cheapest, highest-leverage borrowing is therefore not a new model: it is
**measuring calibration** so a constant confidence shows up as a defect, and
**naming caro's gates as typed decisions** so the next ones are built like
the risk judge rather than like the `QUESTION:` prefix.

---

## Project Profile: TypeSafe AI / Jev

| | |
|---|---|
| Product | Jev — first public "System One Model"; hosted API |
| Output types | **Noul** (yes/no → probability), **Choice** (one of ≤255 options → distribution), **Score** (rating on a scale → score + distribution + confidence) |
| Training | "Reinforcement Learning for Calibrated Decisions (RLCD)" — optimises for "answers with epistemically honest probabilities", not for a ground-truth classification |
| Inference | parallel sampler: all outputs in one query, no sequential token generation |
| Latency claim | "70ms–500ms" end-to-end vs "3 to 329 seconds" for frontier LLMs on the same workflows |
| Cost claim | input "$0.042 / MTok", output "FREE (too cheap to meter)"; production-workflow Pareto claim "≈193.6× faster and 444.6× cheaper" |
| Guarantees claimed | "never makes type errors"; every answer carries a calibrated probability; used to "detect jailbreaks of LLM prompts" |
| Model size / data | not disclosed ("Where does our training data come from?" is left unanswered in their FAQ) |
| Open source | `system-one-adapter-python` (MIT): drop-in that constrains OpenAI/Anthropic models to the same `state → {questions}` API via structured outputs, corrective retries on schema violations, and probability normalisation. Two answer modes: `"probabilities"` (per-label distributions) and `"discrete"` (one label + confidence). |
| Eval method | four real workflows (security-incident triage, invoice processing, agent-trace review, customer-service routing) decomposed into Noul/Choice/Score questions; accuracy measured against consensus labels from frontier models; plotted as accuracy vs cost and accuracy vs latency ("up and to the left is better"). Self-declared caveat: "workflows were made by individuals on our model capabilities team, so some bias could exist." |
| Design principle (quoted) | "Rather than ask a model to solve the entire problem in one shot… we ask independent narrow questions and defer to code where possible." |

Shell or command generation is not addressed anywhere in their material.
Jev is not a competitor to caro's core loop; it is a pattern for the *gates
around* that loop.

---

## Gap Analysis

### 1. Calibrated confidence

| Jev | caro (as of this doc) |
|---|---|
| Every answer carries a probability the model was trained to make honest. | `GeneratedCommand.confidence_score` is validated to `0.0..=1.0` (`src/models/mod.rs:107`) but populated by constants: `src/backends/static_matcher.rs` (1.0 "deterministic match"), `src/backends/embedded/embedded_backend.rs` (0.85), `src/backends/remote/claude.rs` (0.95), `ollama.rs` (0.8), `ai_horde.rs` (0.75). |
| Confidence is the product. | The agent refinement prompt asks for `{"cmd","confidence","changes"}` (`src/agent/mod.rs:~680`) but the parsed value is overwritten by the backend constant before anyone reads it. |
| — | Consumers of the constant: `src/agent/mod.rs:338` (`low_confidence` → refine/advisor), `src/agent/pipeline/sources.rs:61` (`llm_confidence` feature), `src/safety/mod.rs` `blend_smart_decision` (the one place a *real* confidence is honoured, with a 0.7 floor). |

**Consequence:** the 0.8 refinement threshold can never fire for the static
matcher (1.0), always fires for Ollama (0.8 is not `< 0.8`, so actually never),
and never fires for embedded (0.85). The gate is dead code dressed as a
decision.

### 2. Decisions, not text

| Jev | caro |
|---|---|
| Three typed primitives; a type error is impossible by construction. | One typed gate: the risk judge (`src/prompts/risk_judge.rs`) — strict JSON, enum label, confidence floor, hard `Critical` floor. This is already the Jev shape and is the template to copy. |
| Decompose the workflow into narrow questions. | Clarification is a free-text convention: the model may emit `QUESTION: …` (`src/prompts/smollm_prompt.rs:178`); remote backends instead emit `echo 'Please clarify your request'`. Intent categorisation is a prompt instruction ("STEP 1: CATEGORIZE") plus `contains()` on a template name (`src/prompts/command_templates.rs:129`). "Platform fix needed" is a heuristic (`should_refine`). |
| Constrained outputs. | No constrained decoding on any backend: no Ollama `format: "json"`, no vLLM `guided_json`, no llama.cpp grammar. Recovery is lenient re-parsing plus a correction retry (`embedded_backend.rs:~425`). |

### 3. Latency

| Jev | caro |
|---|---|
| Publishes a 70–500 ms band and plots latency per case. | Eval reports a single mean `avg_execution_time_ms`; no percentiles. There are **no** LLM generation-latency numbers anywhere (`docs/PERFORMANCE.md` lists "Benchmark MLX inference latency" as TODO); `benches/performance.rs` exists but is not registered as a `[[bench]]`. `BackendInfo.typical_latency_ms` is a hand-typed constant. |

### 4. Eval methodology

| Jev | caro |
|---|---|
| Accuracy vs cost vs latency Pareto; consensus labels from frontier models; disagreement views. | Three parallel harnesses (`src/eval/`, `src/evaluation/`, `tests/evaluation/`), all judged by string equality with flag normalisation; `src/evaluation/` already has cost-per-passed-task and a baseline/regression store. **No** Brier, ECE or reliability diagram anywhere; `EvaluationResult` had no confidence field, so the join was impossible. The 94.8 % CSR headline is 55 TOML cases under exact-match. |

---

## What to borrow (ranked by value ÷ cost)

1. **Measure calibration before changing anything.** Thread confidence into
   `EvaluationResult`; add Brier and ECE per backend; add p50/p95 latency.
   *Shipped in the PR that adds this document* — see
   `src/evaluation/calibration.rs`. Expected first result: every backend's
   ECE equals `|constant − pass_rate|`, which is the evidence for item 2.
2. **Name the gates as typed decisions.** Introduce `caro::decision`
   (`Noul`, `Choice<T>`, `Score`) and make the risk judge the first consumer
   (*also shipped*). Then migrate, one per PR, in this order:
   `needs_clarification: Noul` (replaces the `QUESTION:` convention and the
   `echo 'Please clarify'` hack), `intent_category: Choice<TemplateCategory>`
   (replaces `contains()`), `platform_fix_needed: Noul` (replaces
   `should_refine` heuristics, or wraps them as the prior). Generation itself
   stays free text; only the decisions around it change.
3. **Replace the confidence constants with measurements.** Cheapest honest
   signals available today, per backend: static matcher → fraction of
   required+optional keywords matched (its own module docs ask for this at
   150+ patterns); embedded/llama.cpp → mean log-prob of the emitted command
   tokens; remote OpenAI-compatible → `logprobs` where offered, else the
   model's self-reported figure through the `Choice` parser. A backend that
   cannot measure reports `None`, never a constant.
4. **Constrained decoding for decisions only.** Ollama `format: "json"` and
   vLLM `guided_json` for the Noul/Choice prompts, with the adapter's
   corrective-retry loop as the fallback. Keep the command prompt unconstrained.
5. **Consensus labels and the Pareto view.** Use the existing frontier
   advisor path (ADR-015) as the reference labeller for gate decisions, store
   disagreements, and feed accepted pairs into `docs/ml/sft-data-pipeline.md`.
   Plot accuracy vs `p95_execution_time_ms` vs `cost_per_passed_task` per
   backend in the weekly demo report. A tiny local classifier for the Noul
   gates is a later experiment and should be gated on this data, not on
   enthusiasm.

## What *not* to copy

- **The hosted-API shape.** caro is local-first; `--approval smart` already
  prints an off-host warning. Jev's cost/latency numbers are for their
  servers, not for a 135M-parameter model on a laptop — compare on the
  *shape* of the eval, not on the numbers.
- **"Can't hallucinate."** caro's safety floor is the deterministic regex set
  plus the catastrophic list, and decisions sit *above* it as advisory
  signals. A typed decision that cannot produce an invalid label is still a
  decision that can be wrong; the floor is what makes being wrong survivable.
- **RLCD as a training target for the command model.** Command text is
  open-ended; calibration training belongs to the gates. Revisit only once
  items 1–3 show the gates are where the errors are.

## Open questions

- Which honest confidence signal is cheapest for the embedded CPU path —
  token log-probs, or a second Noul call "is this command correct?" against
  the same model?
- Does the `Choice` distribution add anything over the discrete
  label+confidence for a 4-way risk decision, or should the judge prompt
  stay discrete and only the parser accept both?
- The three eval harnesses should converge before calibration becomes a
  release gate; which one survives is out of scope here.

## See also

- `docs/adr/ADR-017-typed-decisions-and-calibrated-confidence.md` — the
  decision record for items 1 and 2.
- `docs/PERFORMANCE.md` — "Decision Latency & Calibration" section.
- `docs/ml/sft-data-pipeline.md` — where consensus-labelled decisions go.
- `.claude/rules/validation-discipline.md` — this is internal tooling and a
  proposal, not a new product line; no transcript gate applies.
