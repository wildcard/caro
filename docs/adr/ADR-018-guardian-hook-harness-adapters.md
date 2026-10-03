# ADR-018: Guardian Hook and Harness Adapters

**Status**: Proposed

**Date**: 2026-10-03

**Authors**: Caro maintainers

**Target**: Community

## Context

Caro's positioning is the execution-safety layer of a guardian-agent stack
(`docs/GUARDIAN_AGENT.md`). The jev strategy
(`docs/research/jev-of-execution-safety-strategy.md`, Phase 2) reserved this
ADR number for "the decision API surface".

Four agent harnesses matter now: Claude Code, xAI Grok Build, OpenAI Codex CLI
and OpenCode. All four expose a *pre-tool-use* interception point. The
research in `docs/research/2026-10-03-grok-build-lessons.md` §3 found:

- Claude Code, Grok Build and Codex share one hook shape: the PreToolUse
  payload arrives on stdin, and the decision goes out as
  `hookSpecificOutput.permissionDecision`. Grok Build reads `.claude/settings.json`
  unchanged and sends camelCase keys. OpenCode uses a JS plugin that blocks by
  throwing.
- Hooks **fail open** on timeout, crash or malformed output.
- An `allow` decision means different things. In Claude Code it **skips the
  user's permission prompt**. In Grok Build it means only "not blocked".
- Codex treats `additionalContext` on PreToolUse as an error, and the hook
  then fails open.
- Grok Build's own auto-mode keeps reasons as typed, text-free findings, so a
  verdict cannot echo attacker-controlled text back to the model.

Caro already has the decision core:

- `SafetyValidator::validate_command`, with a deterministic catastrophic floor
  that no allowlist or judge can relax (ADR-017 invariant).
- `RiskLevel`, and a `SuggestedRouting` enum that has an unused `AsyncLog`
  tier.

What Caro lacks is a way to apply that core to *other agents'* commands.

## Decision

1. **Add `caro guard`**, a stdin-JSON hook adapter, as the first surface of
   the decision API. It is a CLI subcommand, not a daemon. Each harness spawns
   it per tool call, the same way it spawns any hook.

2. **One policy for every harness:**
   - Critical → `deny`.
   - High → `ask`.
   - Anything else → no opinion.

   **Caro never emits `allow`.** A guardian can only add friction. It never
   removes the harness's own checks.

3. **Shadow is the default mode.** In shadow mode Caro decides, logs the
   decision, notes it on stderr, and emits nothing the harness acts on.
   Enforce mode is opt-in through `--mode enforce` or `CARO_GUARD_MODE=enforce`.

4. **The hook path uses static rules only.** There is no model load and no
   network on the hot path. A slow guardian under a fail-open contract is
   worse than none. An LLM judge may be added later as an opt-in. It must
   escalate to `ask` on timeout or parse failure, and it stays advisory above
   the floor (ADR-017).

5. **Fail toward the human.** In enforce mode, any internal error on a shell
   tool emits `ask` rather than silence or `deny`. In shadow mode errors exit 0.

6. **Reasons are Caro-authored text only.** Pattern descriptions and the risk
   level go into `permissionDecisionReason`, never the command. No
   `additionalContext` is emitted on PreToolUse.

7. **Adapters:**
   - **`claude`/`grok`/`codex`:** one parser using serde aliases for both key
     styles. The tool names `Bash`, `run_terminal_command` and `shell` count as
     shell tools.
   - **`generic`:** `{"command"}` in, decision record out, exit codes
     0/2/3.
   - **OpenCode:** a ~30-line TS plugin calls `generic` and throws.

8. **Decision record.** Every shell decision is appended as one redacted JSON
   line to `data_dir/caro/guard/decisions.jsonl`. The field names are a
   forward-compatible subset of the Phase 2 `caro decide` record, and include
   `source` (`static` now) for provenance. `caro guard report` summarizes the
   log.

## Consequences

**Positive**

- One binary serves four harnesses, and the docs now describe a command that
  exists. Previously they advertised a nonexistent `caro --validate`.
- Shadow mode generates the evidence the validation-discipline Gate 1 asks for
  before any enforcement claim.
- The never-allow rule makes installing the guard monotonic: it cannot reduce
  safety relative to the harness alone.

**Negative / risks**

- Whole-string regex validation misses segment-level semantics that Grok
  Build's tree-sitter splitter captures. This is tracked as a follow-up,
  requiring TDD and a human safety owner.
- Every shell call pays one process spawn plus config load plus a validator
  scan. The latency is recorded per decision so the cost stays visible.
- A project-scope install can be edited by the guarded agent. The docs
  recommend user scope.
- The hook formats belong to third parties and can drift. Adapter tests pin
  today's shapes, and the research doc records the sources and the date
  checked.

## Alternatives considered

- **An MCP `validate_command` tool first (#928).** Rejected as the first step:
  MCP tools are called by the *model* when it chooses to call them. A hook is
  enforced by the *harness*, so a guardian needs the hook. MCP remains a
  follow-up on the same core.
- **Emitting `allow` for Safe commands, to reduce prompts.** Rejected. It
  turns a guardian into an auto-approver and bypasses user policy in Claude
  Code.
- **Defaulting to enforce.** Rejected. It has no evidence base, and false
  positives would train users to bypass the guard.

## References

- `docs/research/2026-10-03-grok-build-lessons.md`
- `specs/011-harness-guardian/spec.md`
- ADR-017 (typed decisions, catastrophic floor), ADR-010 (sandbox), ADR-003 (audit trail)
