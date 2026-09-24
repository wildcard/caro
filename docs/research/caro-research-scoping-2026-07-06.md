# caro-research--scoping-process — Run Report

**Date**: 2026-07-06
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-031 produced

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02 → 07-03) — ADR-015…030 already scoped; the
   07-03 report's suggested next targets and the Hermes 07-02/07-03 memos
   both converge on one item: the **session circuit breaker** (Hermes
   07-03 recommendation 1, "the smallest gap between Caro and the
   kill-switch checklist").
2. Verified no ADR overlap: ADR-026 halts the *conversation loop* on a
   single `Blocked` turn (exit 3) but has no cumulative counters, no
   persisted halted state, no trip policy. `grep` for circuit
   breaker/kill-switch/tripwire across `docs/adr/` and `src/` came back
   empty. ADR-031 confirmed as next sequential number.
3. Mapped `[FEATURE NAME]` to the live OSS reference implementations:
   **OpenAI Agents SDK guardrail tripwires** and **Claude Agent SDK hooks**
   (plus Trust3's platform-bound kill switch as the commercial benchmark).

## New ADR Produced

**ADR-031** — `docs/adr/ADR-031-session-circuit-breaker.md`

- Declarative `[circuit_breaker]` policy in `config.toml`
  (`trip_on_critical`, `max_high`, `max_blocked`) + cumulative
  `BreakerCounts` and persisted `BreakerTrip` on `AiSession`
- Synchronous pre-dispatch evaluation in `AgentLoop`; tripped sessions
  refuse instantly (no backend init) until explicit `--acknowledge-halt`
- Wire contract: exit code **8 = SessionHalted** (0–6 ADR-024, 7 ADR-027,
  30–32 ADR-030), `HeadlessStatus::"halted"`, `session_halted` NDJSON event
- 7 files changed (1 new: `tests/session_breaker.rs`); no new modules, no
  new crates; all types serde from day one
- 7 deterministic integration tests incl. the anti-race test (executor
  never invoked on the tripping turn)

## Key Research Findings

1. **OpenAI's tripwire halt is advisory, not enforced.** Issue #991 (+#889):
   the SDK's optimistic concurrency means tools still execute and handoffs
   still fire *after* `InputGuardrailTripwireTriggered` is raised. ADR-031
   solves this by construction — breaker evaluation is a blocking step
   *before* dispatch in caro's single-threaded pipeline, and the trip is
   persisted before process exit.
2. **Neither SDK gives subprocess callers a structured halt contract.**
   OpenAI surfaces a Python exception; Claude SDK surfaces hook callbacks in
   the host process. No exit code, no JSON, no event for foreign
   orchestrators. Caro's exit-8 + envelope + NDJSON event is the gap.
3. **Neither SDK has cumulative session memory.** "3 High verdicts this
   session" requires hand-rolled state in both. Caro already owns durable
   session state (`SessionStore`), so declarative counters are cheap.
4. **Claude SDK's one property worth copying:** a `PreToolUse` deny survives
   `bypassPermissions`. Mirrored: `SafetyLevel::Permissive` cannot bypass a
   trip; only explicit acknowledgment clears it.
5. **Codebase footgun logged:** `Turn.risk` is a stringly-typed
   `Option<String>` with labels (LOW/MEDIUM) that don't match `RiskLevel`
   variants (Safe/Moderate). The breaker counts from the typed verdict;
   a follow-up should migrate the field.

## Housekeeping Notes

- Files written to the **working tree only, not committed** (git-workflow
  rule; current checkout is on a dispatch branch). Suggested branch:
  `feat/adr-031-session-circuit-breaker-scope`.
- `docs/adr/README.md` index still stale (ends at ADR-015); unchanged to
  avoid parallel-session collisions.
- P0 from 2026-06-02 remains open: fill `[FEATURE NAME]` in the scheduled
  task, or formally accept autonomous target selection.

## Suggested Next Targets

- MCP RC statefulness audit before the **Jul 28** final spec (Hermes 07-03
  item B) — freshness pass on ADR-015 (`caro mcp serve`)
- `agent_id` field on lifecycle events/audit rows (Hermes item D; ADR-031's
  event schema left room)
- Security hardening (#6) gap analysis vs ADR-010/012
