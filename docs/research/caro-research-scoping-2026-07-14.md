# caro-research--scoping-process — Run Report

**Date**: 2026-07-14
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-036 produced

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02 → 07-13) and the ADR directory. Latest ADR
   on disk is ADR-035 (external policy hooks, 07-13). No newer ADRs landed
   from other sessions.
2. Selection signal was unambiguous: Hermes 07-10 **and** 07-13 both name
   the same "Build/test next" item — a **PreToolUse hook adapter emitting
   caro assessment JSON** ("the smallest artifact that proves 'Caro as the
   reviewer engine behind any agent'"), with 07-13 adding the
   OTel-attribute + OWASP-Agentic-mapping requirement. ADR-035 covered the
   *inverse* (caro hosting hooks); caro *as the guard binary inside other
   agents' hook systems* had zero coverage — no ADR, nothing in `src/`.
3. Mapped `[FEATURE NAME]` to the live host contracts, both fetched today:
   **Claude Code `PreToolUse`** (code.claude.com/docs/en/hooks) and
   **Gemini CLI `BeforeTool`** (geminicli.com/docs/hooks/reference, updated
   2026-04-10). Confirmed OpenAI Codex CLI still has no command-hook
   system — a documented differentiation point, not an adapter target.
4. Verified ADR-036 as next sequential number.

## New ADR Produced

**ADR-036** — `docs/adr/ADR-036-agent-guard-hook-adapter.md`

- One subcommand: `caro guard --host <claude-code|gemini-cli|generic>` —
  pure stateless subprocess, stdin host payload → static-validator verdict
  → host-native output. No daemon, no model load, no network.
- Verdict mapping reuses ADR-020 `SuggestedRouting` verbatim: Block→deny,
  HumanGate→ask (claude-code) / prefixed deny (gemini-cli has no ask
  tier), safe tiers→**silent pass** so caro can tighten but never widen
  host policy (`--emit-allow` opt-in; cannot weaken a deny).
- **Fail-closed by emission**: every internal error path (bad stdin,
  parse failure, validator panic via `catch_unwind`) converges to the
  host's strongest deny signal (exit 2 + stderr). Guard mode never exits
  1 — the code both hosts treat as "proceed". `--on-error allow` opt-out.
- `--host generic` publishes the Hermes-requested **AssessmentEnvelope**
  (schema_version 1, decision_id, sha256 command_hash, risk_level,
  matched_patterns, `owasp_agentic` category IDs, flat `otel_attributes`,
  host-correlation echo), schema emitted via existing `generate-schema`
  bin to `docs/spec/`.
- Exit codes: adapter modes speak the host's contract (0 verdict/pass,
  2 fail-closed); generic mode 0 allow/ask, 3 deny (aligns ADR-024),
  64 usage-error pre-stdin. Nothing else; never 1.
- 6 files changed, no new top-level module, no new dependencies
  (`uuid`/`sha2`/`chrono`/`schemars` already in Cargo.toml); 11
  integration tests with fixture payloads per host.

## Key Research Findings

1. **Both hosts' hook layers fail open by their own documentation.**
   Claude Code: exit 1 "proceeds with the action"; JSON parsed only on
   exit 0; HTTP-hook timeouts/non-2xx "allow execution to continue."
   Gemini CLI: any exit code other than 0/2 is a "Warning… the CLI
   continues," and stray stdout text breaks JSON parsing into that same
   warning path. A crashed guard therefore silently approves the command
   it was judging. ADR-036's core design answer is fail-closed *emission*
   (all errors become the host's deny signal), not a workaround wrapper.
2. **Gemini CLI has no "ask" tier** on BeforeTool (only allow/deny +
   `continue:false`), so caro's HumanGate degrades to deny-with-reason
   there — documented, conservative-by-default.
3. **Allow-signaling is policy-widening.** Emitting
   `permissionDecision: "allow"` on Claude Code bypasses the host's own
   permission prompts. Hence silent-pass default — same "tighten, never
   loosen" principle ADR-035 established in the other direction.
4. **Per-call spawn economics**: hosts spawn hook processes per tool call
   (60s default timeouts). Guard's lifecycle answer is zero redundant
   init — pattern validator only (<2ms), no backends, no state — rather
   than a daemon/warm cache, per the task's pure-subprocess constraint.
5. Both hosts' own documented example guard hooks are single-glob bash
   scripts (`if [[ "$command" == rm* ]]`) — the competitive gap caro's
   52-pattern, quote-context-aware engine fills is real and visible in
   their docs.

## Suggested Next Targets

- `caro guard install <host>` UX (writes settings.json) — deferred to
  v1.1 in ADR-036's out-of-scope list
- MCP RC statefulness audit before the **Jul 28** final spec (carried
  since 07-06; freshness pass on ADR-015)
- PostToolUse/AfterTool receipt emission joining ADR-032 (guard v2)

## Housekeeping Notes (not fixed this run)

- `docs/adr/README.md` index still stale (stops at ADR-015; duplicate
  ADR-004/ADR-015 filename pairs still present — flagged since 07-03;
  violates adr-numbering.md).
- SKILL.md's `[FEATURE NAME]` placeholder remains unfilled; runs keep
  selecting autonomously.
- Files written but **not committed**: per git-workflow.md a feature
  branch + PR is required, and prior runs found sandbox git access
  degraded. A human/interactive session should branch, commit ADR-036 +
  this report, and open a PR.

## Sources

- https://code.claude.com/docs/en/hooks (fetched 2026-07-14)
- https://geminicli.com/docs/hooks/reference/ (fetched 2026-07-14)
- https://developers.googleblog.com/tailor-gemini-cli-to-your-workflow-with-hooks/
- `.hermes/digests/2026-07-10-weekly-agent-market-scan.md`, `2026-07-13-agent-market-scan.md`
- `src/safety/mod.rs` (`SafetyDecision`), `src/models/mod.rs` (`SuggestedRouting`), `src/main.rs` (CLI surface)
- docs/adr/ADR-020, ADR-022, ADR-023, ADR-024, ADR-032, ADR-035
