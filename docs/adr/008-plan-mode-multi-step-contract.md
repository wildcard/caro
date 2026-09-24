# ADR 008 — Plan Mode: Multi-Step Command Plans as a Stateless Contract

- **Status:** Proposed (drafted 2026-08-04 by the `caro-research--scoping-process` scheduled task)
- **Deciders:** Caro maintainers
- **Relates to:** ADR 003 (structured output envelope), ADR 020 (tiered approval),
  ADR 033 (command undo snapshots — execution-side complement)
- **Numbering note:** placed in the lowercase architecture series after 007.
  Per `.claude/rules/adr-numbering.md`, renumber on merge if another `008-` lands first.

> Scope note: this ADR is the decision record. The full Phase 1–3 analysis,
> type definitions, file-change list, and test matrix live in
> `caro-scope-plan-mode-2026-08-04.md` at the repo root.

---

## Context

Caro generates exactly one command (or one pipeline) per invocation — the agent
prompt explicitly instructs "Use single command or pipeline (no multiple
commands)" (`src/agent/mod.rs:678`). Real user tasks ("set up a Python venv and
install deps", "rotate these logs then restart the service") are multi-step, and
today the user must decompose them manually and invoke caro N times with no
shared review of the whole sequence.

The reference implementation studied is **Claude Code's plan mode**
(`--permission-mode plan`, the `ExitPlanMode` tool, `~/.claude/plans/*.md`).
It separates propose from execute: a read-only exploration state produces a plan,
the user approves it, then execution begins. It gets the *separation* right, but
its mechanics have failure modes we can avoid by designing the contract first:

1. **Mode drift.** Plan mode is enforced by prompt/permission state inside a
   long-lived session; Claude sometimes exits plan mode mid-session and write
   tools start firing unexpectedly. Read-only is a *policy*, not a *structure*.
2. **Modal state-machine bugs.** Toggling plan mode after
   `--dangerously-skip-permissions` leaves the session stuck — approval says
   "yes" but the transition never happens (anthropics/claude-code#32934).
   Stateful mode transitions inside one process are inherently fragile.
3. **Plans are prose, not data.** Plans land in `~/.claude/plans/` as markdown
   with random names (`dreamy-orbiting-quokka.md`) — no schema, no version, not
   machine-consumable, not diffable against what actually executes.
4. **Approval is not bound to plan content.** The approval prompt echoes the plan
   text, but nothing cryptographically ties the approved plan to the executed
   actions — a TOCTOU gap between review and execution.

## Decision

Introduce plan mode as **two pure subprocess calls over a versioned, serializable
plan document** — never a modal state:

1. **`caro plan "<task>"`** generates a `models::CommandPlan`: an ordered list of
   `PlanStep`s, each carrying its own `GeneratedCommand` and a full
   `ValidationResult` from the existing `SafetyValidator`. With `-o json` the
   plan is emitted on stdout inside the ADR-003 envelope. This path has **no
   executor wired in at the type level** — read-only by construction, not by
   policy (solves failure mode 1 and 2: there is no mode to drift out of and no
   state machine to wedge).
2. **`caro plan run <plan.json>`** executes an approved plan. Before running
   anything it (a) verifies `plan_hash` (SHA-256 over the canonical serialized
   steps) so the executed plan is bit-identical to the reviewed one, and
   (b) **re-validates every step** through the safety validator at execution
   time — validation at generation AND execution (solves failure modes 3 and 4).
3. Steps execute sequentially via the existing `CommandExecutor`; a non-zero
   step exit halts the plan (`--keep-going` deferred). Per-step
   `ExecutionResult`s are appended to the envelope's `steps[]` on completion.
4. New serializable types live in `src/models/mod.rs`: `CommandPlan`,
   `PlanStep`, `StepOutcome` — all `Serialize + Deserialize` from day one,
   under the envelope's existing `schema_version`.
5. Exit codes extend the ADR-003 table with two codes:

   | Code | Meaning | Envelope `status` / `error.kind` |
   |---|---|---|
   | 3 | Any step blocked by safety validator (gen or run) | `blocked` |
   | 7 | Plan integrity failure (hash mismatch, bad schema_version) | `error` / `plan_invalid` |

   All other codes (0, 1, 2, 4, 5, 6, 201) carry over unchanged.

Reuse, do not duplicate: step commands come from the existing `AgentLoop`
generation path (prompt variant that permits an ordered list); safety verdicts
from `safety::SafetyValidator`; execution from `execution::CommandExecutor`;
risk taxonomy from `models::RiskLevel`; approval semantics from the existing
`ApprovalMode` (`prompt` asks once per plan, per-step for High risk; `auto`
refuses plans containing High/Critical steps — consistent with ADR-020 tiers).

## Consequences

**Positive**
- Propose/approve/execute are separate processes; there is no session state to
  corrupt, matching caro's "pure subprocess, no daemon" constraint.
- `plan_hash` makes approval tamper-evident — the reviewed plan is provably the
  executed plan. Claude Code has no equivalent.
- Plans are typed JSON: diffable, storable in CI artifacts, replayable, and
  covered by `schema_version` — machine consumers get a contract, not markdown.
- Double validation (generation + execution) means a plan file edited by hand or
  by another tool cannot smuggle a dangerous command past review.

**Negative / costs**
- Multi-step generation needs a new prompt template and list-shaped parsing from
  the LLM — a new failure surface (mitigated: static-matcher steps stay
  deterministic; parse failure is a clean `error/parse` envelope, exit 1).
- The plan file is a user-held artifact; stale plans (environment changed since
  generation) can fail at run time. Re-validation catches the safety dimension;
  semantic staleness is documented as the caller's responsibility.
- One more public type surface in `models` under the schema_version discipline.

**Neutral**
- Single-command generation remains the default; `plan` is a subcommand and
  changes nothing for existing callers.

## Alternatives considered

1. **A modal `--plan` flag on the existing generate path** (mirror Claude Code).
   Rejected: reintroduces the mode state machine and its bug class; two
   subcommands over a document are simpler and testable in isolation.
2. **Store plans in a caro-managed directory** (`~/.caro/plans/`). Rejected for
   v1: violates "no state"; the plan travels as a file the caller owns. A plan
   registry can layer on later without changing the document contract.
3. **Execute steps inside the agent loop with replanning on failure.** Rejected
   for v1: agentic replanning is complexity without a validated consumer, and it
   blurs the propose/approve boundary that is the whole point. Deferred.
4. **Markdown plans for human readability.** Rejected as the primary artifact:
   prose is not verifiable. `caro plan` prints a human summary to stderr in
   plain mode; the JSON document is the contract.
