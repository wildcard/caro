# Competitive Analysis — faramesh-core: What Caro Can Learn

**Date**: 2026-06-23
**Author**: Caro Maintainers
**Subject**: [`faramesh/faramesh-core`](https://github.com/faramesh/faramesh-core)
**Type**: Competitive analysis / design learnings (gap-focused)

---

## TL;DR

[faramesh-core](https://github.com/faramesh/faramesh-core) is an open-source
**"governance-as-code"** control plane for AI-agent tool calls (Go, MPL-2.0;
also published as [arXiv 2601.17744](https://arxiv.org/abs/2601.17744)). Its
thesis: *"Every agent tool call is a policy decision,"* enforced by a
**non-bypassable, local, deterministic** boundary that returns
**PERMIT / DEFER / DENY** with **no LLM in the decision path**, and writes a
**tamper-evident audit trail**.

The headline finding is **convergence, not a gap**: Caro independently
arrived at nearly the same enforcement architecture, and most of faramesh's
remaining deltas are already captured in Caro's open ADRs/PRDs. This memo is
therefore deliberately narrow — it documents the **three places faramesh
genuinely adds something Caro has not already designed**, and explicitly lists
what is already covered so future readers don't redo this analysis.

---

## What faramesh is

| Primitive | Meaning |
|---|---|
| **AAB** — Action Authorization Boundary | A non-bypassable checkpoint every tool call must pass before execution. |
| **CAR** — Canonical Action Representation | Agent intent normalized into a canonical form so evaluation is semantic, not textual. |
| **Decision artifact** | A deterministic `PERMIT / DEFER / DENY` verdict that executors must validate *before* acting. |
| **FPL** — Faramesh Policy Language | A readable DSL (`.fms`) with `permit`/`defer`/`deny` rules, conditionals on payload, rate limits, and budgets; first-match-wins. |
| **Decision Provenance Records** | An append-only, hash-chained write-ahead log keyed by canonical action hash, enabling tamper detection and deterministic replay. |

Core design principle, stated bluntly in their launch essay
([*"Your agent's guardrails are suggestions, not enforcement"*](https://dev.to/brianrhall/your-agents-guardrails-are-suggestions-not-enforcement-2c8k)):
prompt-based guardrails are *guidance* (the model may comply); real safety is
*enforcement* (deterministic code intercepts the call regardless of prompt
state or injection).

---

## Convergence: Caro already shares faramesh's DNA

This is a positioning asset. Caro reached the same conclusions independently:

| faramesh concept | Caro equivalent (already shipped) |
|---|---|
| PERMIT / DEFER / DENY | `SuggestedRouting::{AutoApprove, AsyncLog, HumanGate, Block}` — `src/models/mod.rs:189` |
| "No LLM in the decision path" | Smart-mode hard-floor: a static `Critical` is never relaxed by the LLM judge — `src/cli/mod.rs` (`blend_smart_decision`) |
| "Guardrails are suggestions, not enforcement" | `docs/SAFETY_PHILOSOPHY.md`: *"a safety rule that depends on the LLM remembering it is not a safety rule."* |
| Deterministic, pure-function evaluation | 62 pre-compiled regex patterns + CVE rules, symmetric across all backends — `src/safety/patterns.rs`, `src/safety/cve_patterns.rs` |
| CAR — evaluate semantics, not text | `docs/PRD-ast-parser-shell-validation.md` (AST validation, Draft) |
| Enterprise audit/provenance | `docs/adr/ADR-003-monitoring-audit-trail.md` (Proposed) |
| Microsoft Agent Governance Toolkit lineage | already Phase-0 build-spiked: `src/governance/mod.rs`, `--features governance` |

The practical takeaway: when Caro talks about its safety model publicly, it can
cite an independent, peer-reviewed control plane (faramesh) and Microsoft's AGT
as external validation that *deterministic, out-of-model enforcement* is the
correct architecture — not a Caro idiosyncrasy.

---

## The three real learnings

### Gap A — A local, hash-chained decision-provenance log

**faramesh has it; Caro does not.** Caro's `SuggestedRouting::AsyncLog`
variant exists but currently routes **nowhere** — there is no durable record
of what was decided. Caro's only audit design, `ADR-003`, is a heavyweight
**enterprise SaaS** edifice (Kafka, Elasticsearch, CISO dashboard, org-wide
terminal injection, opt-out monitoring). faramesh's *Decision Provenance
Records* are the **local-first, single-user** version ADR-003 skips: an
append-only, hash-chained log keyed by a canonical command hash —
tamper-evident with **zero cloud infrastructure**, and enabling deterministic
**replay** for evaluation/regression.

**Why it matters for Caro specifically:** Caro is a local CLI. A local
decision log gives users a "what did Caro decide and why" history, gives the
eval harness a replay corpus, and finally gives `AsyncLog` a destination —
without inheriting any of ADR-003's enterprise weight.

**Touches:** `src/models/mod.rs` (`AsyncLog`), `src/safety/mod.rs`,
`src/cli/mod.rs`, new `src/safety/audit.rs`.

### Gap B — The decision as a first-class, serializable, hashed artifact

faramesh emits a **decision artifact** that executors must validate *before*
execution — decoupling the *decision* from whoever runs the command. Caro
today couples decision + execution in-process: `ValidationResult` is produced
and consumed inline in `src/cli/mod.rs`. Promoting
`ValidationResult` + `SuggestedRouting` + a canonical command hash into a
single serializable `DecisionRecord` is the **enabler** for both Gap A (it is
*what you log*) and Gap C (it is *what every executor must check*).

**Touches:** `src/models/mod.rs`, `src/safety/mod.rs`, `src/cli/mod.rs`.

### Gap C — Non-bypassability of the *agent-facing* surface (the AAB lens)

Caro's safety philosophy already guarantees enforcement symmetry across
*backends* (embedded, static, ollama, vllm). faramesh's **Action
Authorization Boundary** points at a threat Caro has not *explicitly* audited:
when Caro is **driven by another agent** — the `caro-shell` skill, any
MCP/library entry point, a future Karo — can a caller ever obtain a **raw,
unvalidated** command and execute it itself, bypassing the validator?

The `caro-shell` skill is designed correctly (it "presents the suggestion for
explicit approval" and "refuses to execute"), but no contract test currently
*proves* that every generation entry point emits a validated decision. The
learning is to apply the AAB framing as an **audit + contract test** of the
agent-facing API, and to document that boundary.

**Touches:** public library API (`src/lib.rs`), `.claude/skills/caro-shell`,
`tests/safety_validator_contract.rs`, `docs/SAFETY_PHILOSOPHY.md`.

---

## Explicitly NOT new (already covered — do not re-file)

| faramesh idea | Already in Caro |
|---|---|
| Canonicalize intent / evaluate semantics (CAR) | `docs/PRD-ast-parser-shell-validation.md` + `docs/adr/ADR-007-ast-parser-shell-validation.md` |
| Enterprise audit trail / provenance / compliance | `docs/adr/ADR-003-monitoring-audit-trail.md` |
| Sandboxed execution of risky commands | `docs/adr/ADR-010-bubblewrap-sandbox-execution.md` |
| "Enforcement, not suggestion" philosophy | `docs/SAFETY_PHILOSOPHY.md` |
| Governance/provisioning policy layer | `docs/adr/ADR-002-governance-provisioning-system.md` |

## Where FPL's extra primitives mostly don't apply

FPL's `if`-on-payload conditionals, **rate limits**, **budgets**, and
**agent-scoped default-deny** are aimed at multi-agent, multi-tenant server
deployments. For Caro's local, single-user CLI most are N/A. The one idea
worth tracking for the *enterprise* direction (ADR-002) is a **readable policy
expression above raw regex** — today power users hand-write regex in
`config.toml`; a higher-level matcher would lower that bar. Low priority, and
already adjacent to ADR-002.

---

## Strategic note (GATED — do not turn into a build issue)

faramesh + Microsoft AGT validate a tempting idea: **extract Caro's safety
engine as a reusable "shell-command authorization boundary"** that other
agents call. That would be a **new product line** and must clear all five
gates in [`.claude/rules/validation-discipline.md`](../.claude/rules/validation-discipline.md)
— 20 first-hand transcripts, the demoware-trap section, devil's-advocate
review, and a Sean-Ellis defended cohort — **before** any spec is written.
This memo records the thread as research only; it is not a green light.

---

## Recommended follow-ups

Three GitHub issues, scoped tightly:

- **A.** `safety: local hash-chained decision-provenance log (wire up AsyncLog)`
- **B.** `safety: promote validation result to a serializable, hashed DecisionRecord`
- **C.** `safety: audit non-bypassability of the agent-facing surface (AAB)`

**Validation-discipline status:** Gaps A and B are **safety hardening /
internal tooling**, and Gap C is an **investigation** — none is a new
user-facing product line, so per `validation-discipline.md` ("does not require
interviews for bug fixes, refactors, security patches, or internal tooling")
**none is blocked by the 20-transcript gate.** Only the strategic note above
is gated.

---

## Sources

- faramesh-core repository — <https://github.com/faramesh/faramesh-core>
- faramesh paper — <https://arxiv.org/abs/2601.17744>
- "Your agent's guardrails are suggestions, not enforcement" — <https://dev.to/brianrhall/your-agents-guardrails-are-suggestions-not-enforcement-2c8k>
- Microsoft Agent Governance Toolkit (lineage Caro already spiked) — <https://github.com/microsoft/agent-governance-toolkit>
