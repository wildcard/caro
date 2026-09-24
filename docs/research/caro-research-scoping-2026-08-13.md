# caro-research--scoping-process — Run Report

**Date**: 2026-08-13
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-050 produced

---

## What Happened This Run

The task's SKILL.md still carries the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously:

1. **Coverage map of today's Hermes scan** (2026-08-13 weekly memo):
   - A (Reviewer Endpoint Mode) → ADR-036 + ADR-048 (scoped 08-11)
   - B (Audit-ready lifecycle log) → ADR-037 / ADR-041 / ADR-049
   - C ("Human Approval Isn't Enough" benchmark) → positioning content,
     outside this task's charter
   - E (Pre-flight Agent Safety Score) → parked by the memo itself behind
     `validation-discipline.md` Gate 1
   - **D (Execution Budgets for Destructive Operation Classes) → no ADR
     coverage.** Memo's next step is literally "design note only" — this
     task's exact deliverable shape. Selected.
2. **Feature analogs researched** (Phase 1):
   - **Cloudflare Wallets / cloudflare.pay** (announced 2026-08-04, pre-GA):
     Account Wallet → per-agent Virtual Wallets with allowance, merchant
     allowlist, max transaction size; delegation is tighten-only. Limits:
     platform-bound, currency-only, centralized, spend still future-tense.
   - **Claude Code hooks** (re-verified via docs + community issue trackers):
     no cross-invocation session state, parallel hook execution (counter
     races), exit-1/timeout fail-open, config snapshot per session. This is
     the enforcement point ADR-036 occupies and where naive budget counters
     break — the Phase-1 failure mode ADR-050 solves by design (locked
     append-only local ledger, tighten-only, fail-closed).
   - Context: AWS AgentCore 14-day sessions motivate session-scoped (not
     time-windowed) budgets and the ledger `ttl_days = 14` default.
3. **Phase 2/3 output**: `docs/adr/ADR-050-execution-budget-ledger.md` —
   scope-only ADR: `OperationClass` enum + pattern classification,
   `[budgets]` policy section (ADR-040 file), per-`(host, session)` flock'd
   NDJSON ledger, tighten-only verdict escalation, add-only
   `AssessmentEnvelope.budget` field, no new exit codes, 11 integration
   tests, 7 out-of-scope items, 5 alternatives considered.

## Key Decisions (reviewable)

- **Constraint relaxation**: the task's "no daemon, no state" was read as
  "no daemon, no *resident* state" (ADR-031 precedent — breaker state in the
  session file). A budget is definitionally cumulative; durable file state
  is the honest minimum. Flagged in the ADR's provenance note.
- **Timing**: memo marks D "Later · revisit after A ships". A's *scope*
  (ADR-048) shipped 08-11; implementation has not. Producing the design note
  now front-loads the schema decision without claiming implementation
  priority. ADR-050 is explicitly marked **blocked by ADR-036
  implementation**.
- **Distinct from ADR-031**: breaker = risk-level counters, caro-owned
  session, halt semantics; budgets = operation-class counters, host-owned
  session, escalate semantics. Documented in Alternatives §1; a future ADR
  may merge the counters.

## Codebase Facts Verified This Run

- `DANGEROUS_PATTERNS` actual count is **67**, not the "52+" quoted in
  module docs, CLAUDE.md, and several ADRs (marketing "52+" remains true;
  drift noted, no main-branch edit made per git-workflow rule).
- `SafetyValidator` is a struct (not trait); `validate_command` is async
  with zero I/O (ADR-022 de-async still pending) — guard paths need a
  runtime until that lands.
- `CARO_CONFIG_DIR` env var is set by `tests/e2e_cli_tests.rs` /
  `cli_flag_tests.rs` but **read nowhere in `src/`** — test config isolation
  is currently illusory. ADR-050 therefore specifies explicit `--state-dir`
  (and relies on ADR-036's `--config`) instead of the env var.
- Next ADR number: 050 (highest on disk: ADR-049). `docs/adr/README.md`
  index is stale (lists through ADR-015 only). Two historical duplicate
  numbers exist (004, 015) — candidates for an adr-numbering cleanup PR.

## Suggested Follow-ups (not done — require human/PR review)

1. Open a feature branch + PR landing ADR-050 (and optionally flipping
   ADR-036 to Accepted if implementation is being scheduled).
2. `docs/adr/README.md` index refresh + duplicate-number renumbering per
   `.claude/rules/adr-numbering.md`.
3. Hermes memo recommendation C (benchmark blog post) is cheap and
   time-sensitive (news window); belongs to a content session, not this task.
4. Fill the SKILL.md `[FEATURE NAME]` placeholder or bless the current
   "pick from latest Hermes scan" convention in the task file.

## Artifacts

- `docs/adr/ADR-050-execution-budget-ledger.md` (new, uncommitted)
- This report (new, uncommitted)

*Neither file was committed — git-workflow.md forbids main-branch commits;
landing them requires a feature branch + PR.*
