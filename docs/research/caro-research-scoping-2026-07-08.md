# caro-research--scoping-process — Run Report

**Date**: 2026-07-08
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-033 produced

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02, 06-05, 07-03) and the full ADR index.
   Since 07-03, other sessions landed ADR-031 (session circuit breaker,
   07-06) and ADR-032 (portable execution receipts, 07-07).
2. Surveyed hot experimental agent features with no caro ADR coverage.
   Sandboxing (ADR-010), approval (ADR-020/027), headless (ADR-024–029),
   voice (ADR-030), receipts (ADR-032) are all taken. **Checkpoint/undo has
   zero coverage** — no ADR, no ROADMAP mention, nothing in `src/`.
3. Selected **Claude Code Checkpointing / `/rewind`** (+ the Agent SDK's
   file-checkpointing API) as the researched feature. Fetched both official
   doc pages live (2026-07-08) and confirmed demand-side signal via
   openai/codex#12558 requesting the same capability.

## New ADR Produced

**ADR-033** — `docs/adr/ADR-033-command-undo-snapshots.md`

- `caro undo` + automatic pre-execution snapshots for `--execute` at
  `RiskLevel::Medium`+ — pure subprocess, no daemon, no session
- Content-addressed checkpoint IDs (not session UUIDs); versioned
  `UndoCheckpoint` / `RestoreOutcome` JSON (schema_version 1) inside the
  ADR-024 envelope
- New exit codes 9 (capture failed → execution aborted, fail-safe) and
  40/41/42 (undo domain), non-colliding with 0–8 and 30–32
- Typed `CaptureFidelity` (full/partial/none) + `gaps[]` — honesty about
  unpredictable write sets instead of silent non-capture
- `post_hash` conflict detection at restore (refuses with exit 41 if a file
  changed externally; `--force` overrides)
- 7 files changed, no new top-level module; storage via existing
  `CacheManager`; path prediction upgrades transparently when ADR-007's AST
  parser lands

## Key Research Findings

1. **Claude Code's checkpoint system excludes bash side effects by
   design** — their docs lead with it: `rm`/`mv`/`cp`/`sed -i` cannot be
   rewound; only Write/Edit/NotebookEdit tool edits are captured. For caro,
   whose entire output surface is shell commands, replicating their
   architecture would protect nothing. ADR-033 inverts it: capture at the
   executor choke point, *because* caro knows the command before it runs.
2. **Their programmatic contract is hidden-flag experimental**: checkpoint
   UUIDs require an undocumented `replay-user-messages` extra arg; CLI rewind
   needs the `CLAUDE_CODE_ENABLE_SDK_FILE_CHECKPOINTING` env var plus a
   `--rewind-files` flag absent from `--help`; post-stream rewind requires
   resuming the session with an empty prompt to reopen a transport. This is
   the session-coupling failure mode the task constraints target — ADR-033
   answers with a session-free, content-addressed, documented contract.
3. **Directory operations and external concurrent edits** are additional
   documented gaps: dirs aren't undone at all, and rewind can silently
   clobber manual edits. ADR-033 covers dirs via recursive bounded capture
   and external edits via post-execution hash conflict detection.
4. Their 30-day retention default was adopted as caro's
   `undo.retention_days` default — a familiar number for migrating users.

## Housekeeping Notes (not fixed this run)

- `docs/adr/README.md` index remains stale (still ends around ADR-015 per the
  07-03 report; now 18 ADRs behind). A dedicated docs-sync session should run
  `/caro.sync`.
- There are two ADR-004s and two ADR-015s in `docs/adr/` — violates
  `.claude/rules/adr-numbering.md` (sequential, no duplicates). Needs a
  renumbering pass.
- SKILL.md's `[FEATURE NAME]` placeholder is still unfilled; each run keeps
  choosing autonomously. Consider either filling it per-run or codifying the
  autonomous-selection behavior in the task file.
- Files written but **not committed**: sandbox git access is degraded
  (worktree metadata unresolvable, `index.lock` unlinkable) and
  `.claude/rules/git-workflow.md` requires a feature branch + PR anyway.
  Next interactive session should branch, commit, and open the PR.

## Sources

- https://code.claude.com/docs/en/checkpointing
- https://code.claude.com/docs/en/agent-sdk/file-checkpointing
- https://github.com/openai/codex/issues/12558
