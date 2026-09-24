# caro-research--scoping-process — Run Report

**Date**: 2026-07-13
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-035 produced

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02 → 07-08) and the ADR index. Latest ADR on
   disk is ADR-034 (headless degraded-result contract, 07-10). No new ADRs
   landed from other sessions since the 07-08 run besides 034.
2. Surveyed uncovered surfaces. Sandboxing (010), approval (020/027),
   headless (024–029, 034), voice (030), circuit breaker (031), receipts
   (032), undo (033) are all taken. **User-extensible hooks have zero
   coverage** — and `src/caroml/validators/mod.rs` explicitly promises an
   "external-validator hook (binary, JSON stdin/stdout)" for v0.2 with no
   ADR behind it.
3. Selected **Claude Code Hooks** as the analog. Fetched the official
   reference live (code.claude.com/docs/en/hooks, 2026-07-13): now 30
   lifecycle events, five handler types (command/HTTP/MCP/prompt/agent),
   matchers, exec/shell forms, async hooks, per-event timeouts.

## New ADR Produced

**ADR-035** — `docs/adr/ADR-035-external-policy-hooks.md`

- One hook event (`pre_execute`) at the executor choke point instead of the
  analog's 30-event lattice — caro's whole output surface is one action
- Exec-form-only config in `config.toml` (`[[hooks.pre_execute]]`), no
  shell-form strings; user-level config only (no repo-local hooks in v1)
- Single-channel wire contract: exit 0 + versioned JSON (`HookRequest` /
  `HookResponse`, schema_version 1) is the only success path; **fail-closed
  by default** (`on_failure = "closed"`)
- Hooks can tighten, never loosen — `allow` cannot override the built-in
  safety validator's Block
- Implemented as `ExternalHookAngle` on the existing `Validator` trait, so
  the CaroML interpreter chain and the CLI single-command path share one
  mechanism; outcomes serialize into the ADR-024 envelope
- Exit codes: reuse 3 (Blocked, with `blocked_by: "hook:<name>"`); new 50
  (hook infra failure under fail-closed) and 51 (hook misconfiguration) —
  non-colliding with 0–9, 11, 30–32, 40–42
- 7 files changed, no new top-level module; 10 integration tests with
  fixture hook scripts

## Key Research Findings

1. **The analog's policy layer fails open by its own documentation.** Exit
   code 1 (the conventional Unix failure code) proceeds with the action;
   only the magic value 2 blocks. HTTP hook timeouts and non-2xx responses
   "allow execution to continue". The `if` pre-filter "fails open … when
   the Bash command can't be parsed". Their docs literally recommend using
   the permission system, not hooks, for hard deny. For a safety-first
   tool, this is the failure mode to solve by design: ADR-035 makes exit
   0 + parseable JSON the only success path and defaults every hook to
   fail-closed.
2. **Dual signaling channels lose information.** Exit codes XOR JSON-on-
   exit-0; JSON is ignored on exit 2, so authors who combine them silently
   lose their structured deny reason. ADR-035 has one channel.
3. **Coverage gap in PreToolUse**: `@`-referenced files bypass hooks
   entirely. Caro's single choke point (every command passes the executor)
   has no equivalent second path.
4. **Session coupling**: hook input includes `session_id` and
   `transcript_path`. ADR-035's `HookRequest` is session-free and
   byte-replayable, per the task's pure-subprocess constraint.
5. Useful ideas adopted from the analog: exec form (`command` + `args`)
   as the safe spawn shape, parallel dispatch with dedup, per-hook
   timeouts, and regex matchers (load-time-validated in caro's version).

## Housekeeping Notes (not fixed this run)

- `docs/adr/README.md` index still stale; duplicate ADR-004/ADR-015 pairs
  still present (flagged since 07-03; violates adr-numbering.md).
- SKILL.md's `[FEATURE NAME]` placeholder remains unfilled; runs keep
  selecting autonomously.
- Files written but **not committed**: per git-workflow.md a feature branch
  + PR is required, and prior runs found sandbox git access degraded. A
  human/interactive session should branch, commit ADR-035 + this report,
  and open a PR.

## Sources

- https://code.claude.com/docs/en/hooks (fetched 2026-07-13)
- src/caroml/validators/mod.rs (v0.2 external-hook promise)
- docs/adr/ADR-020, ADR-022, ADR-024, ADR-033, ADR-034 (registry + style)
