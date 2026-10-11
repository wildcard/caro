# Caro as the "Jev" of execution safety — strategy plan

**Date:** 2026-09-27
**Purpose:** Answer "how does caro become the Jev of its own vertical, what is that vertical, and which ongoing components, docs, skills and rules a Jev move touches." Companion to `jev-system-one-gap-analysis.md` and ADR-017.
**Status:** Proposal for review; phases become epic children under #1460.

## Context

Jev (TypeSafe AI) is a **general-purpose calibrated decision model**: three
typed outputs (Noul yes/no, Choice one-of-N, Score), a probability the model
is trained to make honest (RLCD), 70–500 ms latency, "never makes type
errors", evaluated on real workflows decomposed into narrow questions and
scored against frontier consensus on accuracy × cost × latency. It is
horizontal: it sells the *shape* of a decision to any workflow.

**Caro's vertical is execution safety**: the last-mile decision *"should
this shell command run, as drafted, on this machine, right now?"*
(`docs/GUARDIAN_AGENT.md:3`, `COMPANY.md:14-23`: "guardian-agent execution
layer … the execution-time enforcement point"). Gartner formalised the
guardian-agent category in March 2026; identity (Orchid) and policy/observability
(Microsoft AGAT, LangSmith) layers exist; nobody yet owns the calibrated
execution-time decision. That is the seat.

**What "the Jev of execution safety" means concretely**: a local-first,
sub-second decision oracle that any agent, IDE or orchestrator can call and
get back typed, calibrated answers about a command — not free text:

| Decision | Type | Exists today? |
|---|---|---|
| `risk_level` | `Choice<RiskLevel>` | yes, `--approval smart` judge (`src/prompts/risk_judge.rs`) |
| `should_run` / `needs_confirmation` | `Noul` | as enum `blend_smart_decision` (`src/safety/mod.rs:252`), not yet a probability |
| `needs_clarification` | `Noul` | yes (#1462, `src/decision/mod.rs:169`) |
| `intent_category` | `Choice<IntentCategory>` | yes (#1463), not wired to prompt |
| `command_confidence` (is this command correct?) | `Score` | constant per LLM backend — #1464 |
| `platform_fix_needed` | `Noul` | string heuristics `should_refine` (`src/agent/mod.rs:831-850`) |
| `is_prompt_injection` | `Noul` | absent; Jev advertises it; the guardian-agent doc cites CVE-2026-25592 as the motivating attack |

Caro's structural advantage over Jev in this vertical: a **deterministic
regex + CVE floor** that a probability can never relax (Critical always wins,
`blend_smart_decision`). Jev's advantage caro lacks: **honest probabilities
and a published Pareto eval**. The strategy is to keep the floor and add the
calibration, not to replace one with the other.

## What history says (learning from the project)

| Release | Step toward a decision layer | Ref |
|---|---|---|
| 1.0.0 (2025-12-24) → 1.0.3 | free-text NL→command, static safety patterns | `CHANGELOG.md:894` |
| 1.1.0 GA (2026-01-12) | telemetry opt-in; `--safety` wired into backend validators (1.1.3) | `:527`, `:449` |
| 1.3.0 (2026-04-20) | conversational `caro ai`, shell-init: caro becomes a loop, not a one-shot | `:348` |
| 1.4.0 (2026-05-09) | CaroML: intent-tracked tasks with a regen *decision* (`regen_evaluator.rs:16`) | `:152` |
| 1.5.0 (2026-07-12) | custom TOML patterns capped at High; floor hardened over 5 adversarial rounds | `:53` |
| Unreleased (#1459) | `caro::decision`, Brier/ECE in eval, typed clarification + intent, measured static confidence | `:8` |

Pattern: every release since 1.1 has moved a *decision* from prose into
code, and every safety change has kept the floor non-negotiable. The
validation-discipline rule (2026-05-25) then blocked the whole v2.0 feature
set (Karo, Dogma, voice, self-healing, local context) at 0/20 transcripts
(`docs/discovery/v2.0-validation-audit.md:193`). The 2026-07-12 decision
record shipped 1.5.0 honestly instead of a hollow 2.0. Lesson: the project
grows by hardening the core loop, not by adding product lines — and the
"Jev move" is exactly that kind of hardening (extension of `caro-core`, so
Gate 1 does not apply to the internal gates; see the one exception below).

## Ongoing components that benefit from a Jev move

Inventory with the signal each uses today (from the exploration):

1. **Agent refine/advisor gate** `src/agent/mod.rs:349` (`confidence_threshold` 0.8) — fires on backend identity because confidence is constant. Becomes real with #1464.
2. **`should_refine`** `src/agent/mod.rs:831-850` — string heuristics → `platform_fix_needed: Noul` with the heuristics as prior.
3. **Candidate-ranking pipeline** (`candidate-ranking` feature, `src/agent/pipeline/*`): `LinearScorer` with hand-set weights (llm 0.35, safety 0.25, platform 0.20, knowledge 0.15, latency 0.05; `weights.rs:19-25`); `llm_confidence` feature is the constant. Planned to become default "once the eval harness shows wins" — it cannot until confidence is measured. Becomes a calibrated argmax over `Score`s.
4. **Risk judge + `blend_smart_decision`** — already the template; only missing constrained decoding (#1465) and the `should_run` Noul export.
5. **Hybrid privacy gateway** (`src/backends/hybrid/`, ADR-015) — sanitise-or-send is a boolean config; a `Noul` "contains PII" lets the sanitiser report p and be evaluated.
6. **CaroML regen** (`src/caroml/regen_evaluator.rs`): UseCache / HardRegen / SoftExplore is a `Choice` by construction; typing it gives it eval coverage.
7. **Knowledge layer** (`src/knowledge/`, `should_index`, `record_correction`): the correction log is the DPO/RLCD training source (`docs/ml/sft-data-pipeline.md:37-48`); today it stores no probability, so it cannot teach calibration.
8. **Telemetry** (`src/telemetry/events.rs:49-122`): records backend, duration, success, risk_level, advisor accepted — but **no confidence and no outcome label**, so live calibration is impossible today.
9. **MCP server / OpenAI-compatible endpoint** ("in progress", `docs/GUARDIAN_AGENT.md:63-76`) — the natural delivery surface for a decision API; today they would return free text plus a risk string.
10. **Eval**: three harnesses, CSR 94.8% headline recorded on the 58-case beta suite in January 2026 (ROADMAP.md) and re-asserted as `csr >= 0.948` over the 55-case `tests/evaluation/test_cases.toml` by `tests/evaluation/harness.rs`; the 101-case `dataset.yaml` feeds `src/evaluation/` and carries no CSR headline (three suites, as `jev-system-one-gap-analysis.md` describes); Brier/ECE just added; no consensus labels, no Pareto view; `ml_fine_tune_loop` routine runs nightly with nothing calibrated to train on.
11. **Hermes** monitors PRs, market, integration health — not model quality or calibration.
12. **Dogma rule engine / enterprise dashboard** (research-only, `hypothesis-ledger.md:36,40`): custom rules are user-authored decisions; the dashboard is where decisions with probabilities get audited (ADR-003). Both become cheaper once decisions are typed, but both stay behind Gate 1.

## The plan — five phases, each one PR-sized epic child or ADR

### Phase 0 — Land the foundation (this week)
- Merge #1459. Flip ADR-017 to **Accepted** in the PR that closes #1464.

### Phase 1 — Honest confidence everywhere (#1464, next branch)
Findings that change the issue's plan: the embedded **CPU path is a stub**
(`src/backends/embedded/cpu.rs:41-81` sleeps and returns canned JSON; candle
is a dependency but unused), so "CPU log-probs first" is not possible. MLX
runs through the `llama_cpp` crate's `StandardSampler` and exposes only
strings (`mlx.rs:165-180`); log-probs need a custom `Sampler` or the
crate's token API. No OpenAI-compatible request sends `logprobs`.
- `ConfidenceSource { Measured, SelfReported, None }` on `GeneratedCommand` (`src/models/mod.rs:70-96`, serde default `None`); 17 construction sites, list in exploration.
- vLLM / OpenRouter: add `logprobs: true` to the request, parse `choices[].logprobs.content[].logprob`, mean over command tokens → `exp()`. Ollama `/api/generate` has no logprobs → `None`.
- MLX: custom `Sampler` capturing the chosen token's log-prob; if the crate blocks it, `None` and a tracked issue. CPU stub: `None`.
- Claude / advisor: self-report `{"cmd","confidence"}` through `Choice::discrete`; source `SelfReported`.
- AI-Horde, Exo, Mesh: `None`.
- Agent gate fires only on `Measured | SelfReported`; `CalibrationRollup` filters on source; eval table gains a `source` column.
- **Telemetry**: add `confidence`, `confidence_source`, and an outcome field (executed / edited / rejected) to `CommandGeneration` so live ECE is computable; consent unchanged.
- Guard: `calibration::tests::excludes_unsourced_confidence`; per-backend contract tests.

### Phase 2 — The decision API surface (shipped under ADR-017; no separate ADR, so 018 went to Phase 4)
- `caro decide "<request or command>"` (and `--output json`) returning one typed record: `{risk: Choice<RiskLevel>, should_run: Noul, needs_clarification: Noul, intent: Choice, confidence: Score, source, floor_applied: bool, latency_ms}`. Every field is a `caro::decision` type; the JSON Schema is published.
- Same record from MCP `validate_command` / `explain_safety` and the OpenAI-compatible endpoint (the two "in progress" integrations in `GUARDIAN_AGENT.md`).
- `should_run` = calibrated composition of static floor + judge (`blend_smart_decision`) with the invariant: `p_run == 0.0` whenever the floor says Critical.
- `is_prompt_injection: Noul` as a new gate (prior: the existing injection-shaped safety patterns; LLM refinement optional). This is the one decision Jev markets that caro's vertical needs most.
- Constrained decoding for decision prompts only (#1465): Ollama `format: "json"`, vLLM `guided_json`; corrective retry as fallback.
- Latency budget per gate recorded as p50/p95, target the 70–500 ms band on the static path, documented in `docs/PERFORMANCE.md`.
- **Gate check**: a decision API consumed by third-party agents is arguably a new user (orchestrator authors), so it must clear validation-discipline Gate 1. Fold into the already-planned `enterprise-dashboard` interviews (`docs/discovery/interview-enterprise-dashboard.md`): add 3 questions on "how does your agent decide a command is safe today". Until then the API ships as an *extension* used by caro's own agent loop.

### Phase 3 — Eval as a Pareto chart (#1466)
- Consensus labels: use the frontier advisor path (`AgentLoop::try_advisor`) plus one second model to label risk / clarification / intent for the 101-case `dataset.yaml`; store disagreements.
- Report per backend and per gate: accuracy vs `p95_execution_time_ms` vs `cost_per_passed_task`; ECE regression fails the same way CSR regression does once a measured-confidence baseline exists.
- Converge the three harnesses on `src/evaluation/` (the one with baselines and cost); keep `tests/evaluation/` as the CI gate.
- Publish a `caro.sh/evals` page (the website-claims suite already verifies published numbers, ADR-009) mirroring evals.typesafe.ai's "up and to the left" charts. Fix the found drift first: README says 93.1% while ROADMAP says 94.8%; `gtm-use-cases.ts:16` says "Zero telemetry" while telemetry is opt-in.

### Phase 4 — Train the gate model (caro.ml, `ml_fine_tune_loop`)
Gated on Phase 3 data, per the research doc's "not on enthusiasm".
- Dataset: consensus-labelled gate decisions + correction-log triples with recorded p (Phase 1 telemetry) through `src/ai/privacy.rs` redaction.
- Target: a small local classifier (or LoRA on the smoke model) for the Noul gates — risk, injection, needs_clarification — with a calibration objective (Brier/ECE), i.e. caro's RLCD-lite. Command *generation* stays free text.
- Owner: `ml-ds-engineer` agent; deliverable an ADR with before/after ECE, not a model drop. Landed as ADR-018 (Phase 2 shipped without its own ADR, and ADR numbers have no gaps); tracking issue #1510; the eval binary's `CARO_EVAL_EXPORT_LABELS` writes the corpus.

### Phase 5 — Product surfaces (after Gate 1)
- Enterprise dashboard (ADR-003) audits decisions with probabilities; Dogma becomes "custom decision rules" layered under the same floor. Both wait on the 20 transcripts already scheduled in the ledger.

## Documentation milestones
- ADR-017 → Accepted (Phase 1); no separate decision-API ADR (Phase 2 shipped under ADR-017); ADR-018 gate classifier (Phase 4); add rows to `docs/adr/README.md`; reconcile the two untracked legacy ADR files.
- `ROADMAP.md`: new `### v1.6.0 — Calibrated decisions` milestone above v1.5.0; update "Last Updated"; remove Karo/voice from v2.0 success criteria (contradicts the Research section).
- `COMPANY.md:14-23` positioning gains one sentence: "the calibrated, deterministic-floored decision layer for execution safety".
- `playbook/STAGE_MAP.md` Stage 2 evidence: calibration metrics as anti-demoware discipline; Stage 3: `caro.sh/evals` as proactive-recall surface.
- `docs/research/jev-system-one-gap-analysis.md`: append "Status" table per phase; `docs/PERFORMANCE.md`: per-gate latency band.
- `docs/ml/sft-data-pipeline.md`: Source 3 = consensus-labelled decisions; `docs/strategy/TELEMETRY_STRATEGY.md`: confidence + outcome fields.
- `CLAUDE.md` architecture tree: add `src/decision/` and `src/evaluation/calibration.rs`.

## Skills, rules and agent context to add or change
- **New rule** `.claude/rules/decision-gates.md` (Tier 2, after feature-evidence): any new yes/no or one-of-N gate uses `caro::decision`, has a confidence floor, records p and source, is advisory above the safety floor, and ships with a Brier/ECE before/after. Checklist-as-grep like the release rule. Update `constitution.md`.
- **feature-evidence.md**: for gate PRs the "evidence" artifact is the calibration table, not only CSR.
- **New skill** `calibration-audit`: run the eval, print ECE/Brier/coverage per backend and per gate, diff against baseline, open an issue on regression. Wire into `qa_automation_loop` (daily 09:00) and the Monday demo report.
- **prompt-tuner skill / agent**: optimise for ECE alongside CSR; a prompt that raises CSR but worsens calibration is a regression.
- **ml-ds-engineer agent**: owns Phase 4; `caro.ml` skill gains a "gate classifier" experiment template.
- **devils-advocate**: run on this plan's positioning claim ("Jev of execution safety") before ADR-018; record objections in the ADR.
- **Hermes** (`.hermes/AGENT.md`): add calibration to the daily digest and TypeSafe AI to the weekly competitive scan.
- **caro-frustrated-beta**: add "did caro ask when it should have, and stay quiet when it should not" (clarification calibration) to its symptom list.

## Sequence and sizing
| Phase | Size | Depends on |
|---|---|---|
| 1 #1464 | 1 PR (~600 LOC) | #1459 merged |
| 2 `caro decide` + #1465 (under ADR-017) | 2 PRs | Phase 1 |
| 3 #1466 + evals page | 2 PRs | Phase 1 |
| 4 gate classifier (ADR-018, #1510) | ADR + experiment | Phase 3 data |
| 5 product | interviews first | Gate 1 |

Phases 2 and 3 can run in parallel sessions on separate branches.

## Verification
- Phase 1: `cargo test --lib -- calibration decision`, `cargo test --test evaluation` shows a `source` column and ECE only over sourced rows; vLLM contract test asserts `Measured`.
- Phase 2: `caro decide --output json "rm -rf /"` returns `should_run.p_yes == 0.0`, `floor_applied == true`; JSON validates against the published schema; p95 of the static path < 100 ms in `benches/`.
- Phase 3: baseline JSON carries ECE per gate; a deliberately mis-calibrated constant fails the regression gate; website-claims suite passes on the new evals page.
- Phase 4: ADR-018 reports ECE before/after on held-out consensus labels; no safety-pattern change.
- Docs: `grep -n "Last Updated" ROADMAP.md`, ADR README rows sequential, README/ROADMAP CSR figures agree.
