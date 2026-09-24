# ADR-016 — `caro fix`: Post-Execution Self-Healing via Structured Failure Context

**Status**: Proposed

**Date**: 2026-06-01

**Authors**: `caro-research--scoping-process` (automated research/scoping agent)

**Target**: Community

**Hypothesis ID**: `self-healing`

**Relates to**: ROADMAP.md v2.0.0 `Self-Healing Features`, ADR-015 (MCP Safety Server), `src/caroml/validators/` v0.2 roadmap item (`idempotency`, `reversibility`)

---

## Context

Caro generates POSIX shell commands from natural language. When a generated
command **fails at runtime** (non-zero exit code), the user currently has no
automated path to recovery: they re-describe the task or manually diagnose the
error.

Three competitors address this gap in different ways, each with a distinct
failure mode:

### Competitive Analysis

**Warp Active AI Recommendations** (cloud, closed-source):
Uses exit code + recent stdout/stderr + git branch metadata to proactively
surface fix suggestions in the terminal UI. Correct signal, but:
- Cloud-dependent: no suggestion when offline
- Terminal-native: cannot be called as a subprocess or piped to CI
- Privacy risk: stdout/stderr of failed commands is sent to Warp's servers
- Cannot distinguish side-effectful commands from safe ones before suggesting retry

**GitHub Copilot CLI** (`gh copilot suggest`, closed-source):
Stateless NL-to-command generator. Offers no failure context at all:
`gh copilot suggest` has no `--stderr` or `--exit-code` input. Automation
is actively hostile: piping stdin causes `EOF` errors (tracked in
gh community #148439). Missing the correction loop entirely.

**thefuck** (open-source, Python):
Rule-based pattern matching on `(script, stdout, stderr)`. Each rule
implements `match(command) -> bool` + `get_new_command(command) -> str`.
200+ hand-authored rules. Fast and offline, but:
- **Core failure mode: pattern drift.** When a CLI tool changes its error
  output format, the matching rule breaks silently. Rules encode
  output strings from specific tool versions.
- **Core failure mode: no side-effect guard.** thefuck auto-executes
  corrections. `rm -rf /tmp/project` fails → thefuck suggests `sudo rm -rf
  /tmp/project` → executes it. The original command already deleted files;
  the "fix" adds privilege escalation. This is the exact scenario the
  v2.0 validation audit flagged as "high demoware-trap risk."
- Not extensible to new tools without authoring a new rule; no LLM inference.

**Atuin AI Agent Hooks** (open-source):
Captures `PreToolUse` / `PostToolUseFailure` lifecycle events with exit
codes and duration, tagged by agent author. Pure history recording — no
correction loop. Architecture is relevant: the `PostToolUseFailure` event
carries exactly the structured context `caro fix` needs as input.

### What Caro already has

The building blocks for self-healing exist but are not wired together:

- `src/execution/executor.rs` → `ExecutionResult { exit_code, stdout, stderr, execution_time_ms, success }` — structured failure data
- `src/agent/mod.rs` → `AgentLoop::repair_command()` — repairs pre-execution validation failures; not wired to runtime failures
- `src/caroml/validators/side_effects.rs` → `SideEffectsAngle` — detects sudo/network/destructive operations. The v0.2 roadmap note in the module doc explicitly lists `idempotency` and `reversibility` as planned angles.
- `src/knowledge/` → `KnowledgeIndex::record_correction()` — records successful corrections for future retrieval
- `src/safety/` → 52-pattern safety validator with `SuggestedRouting`

### The design gap

The gap is a `caro fix` verb that:
1. Accepts structured failure context as input (failed command + exit code + stderr)
2. **Classifies the original command's idempotency** before offering a correction
3. Generates a corrected command via the existing `AgentLoop` backend
4. Safety-validates the correction before returning it
5. Returns deterministic JSON — no auto-execution, no interactive prompt

---

## Decision

Add a new CLI verb `caro fix` that takes structured failure context and returns
a JSON `FixSuggestion`. The feature works as a pure subprocess call, offline,
with no state.

### Input contract

```
caro fix --command <cmd> --exit-code <n> --stderr <text> [--prompt <original-intent>]
```

or via stdin JSON (for pipe-friendly CI use):

```
echo '{"command":"ls /nope","exit_code":2,"stderr":"No such file","stdout":"","original_prompt":"list files in nope"}' \
  | caro fix
```

### Output contract (stdout)

Always valid JSON. One of two shapes:

**Success (exit 0):**
```json
{
  "fixed": true,
  "command": "ls /tmp",
  "rationale": "Directory /nope does not exist; corrected to /tmp based on cwd context",
  "idempotency": "Safe",
  "confidence": 0.87,
  "original_command": "ls /nope"
}
```

**Cannot fix (exit 2 or 3):**
```json
{
  "fixed": false,
  "command": null,
  "rationale": "Command has irreversible side effects; manual review required before retry",
  "idempotency": "Dangerous",
  "confidence": 0.0,
  "original_command": "dd if=/dev/zero of=/dev/sda"
}
```

### Process exit codes

| Code | Meaning |
|------|---------|
| `0`  | Correction found; `FixSuggestion.fixed == true` |
| `1`  | Internal error (backend unreachable, JSON parse failure) |
| `2`  | Blocked: safety validator rejected the correction, or original command is `Dangerous` |
| `3`  | Unfixable: failure type not recognized (no LLM fix could be generated) |

---

## Rationale

**Solving the demoware-trap by design, not workaround.**
The v2.0 validation audit flagged self-healing's "what breaks at 100 real
users" as: silent data corruption, runaway retries, and double-execution of
side-effectful commands. This design solves all three structurally:

1. `caro fix` never auto-executes. It is strictly a generator.
2. Idempotency classification gates the correction: `Dangerous` commands return
   `fixed: false` with a rationale — the caller (human or shell wrapper) must
   explicitly opt in.
3. The safety validator runs on the *correction* before it is returned. A fix
   that introduces a safety violation (e.g., adding `sudo` to a dangerous `rm`)
   is rejected at `exit 2`, not returned.

**Caro's unique positioning vs. competitors:**
- Offline: no cloud required; embedded backend works with no network
- Subprocess-friendly: pure JSON in / JSON out; no interactive prompt
- Side-effect-aware: uses `SideEffectsAngle` (already in-tree) to classify
  the original command before generating a correction
- Safety-gated: the correction is validated before it leaves `caro fix`
- Community-extensible: the idempotency classification rules live in the open
  `src/caroml/validators/` framework, not a proprietary rule database

---

## Consequences

### Benefits

- Closes the runtime failure recovery gap without introducing auto-execution risk
- Reuses all existing infrastructure: no new crates, no new backends
- JSON output makes `caro fix` composable with shell wrappers, CI pipelines,
  and eventually the MCP Safety Server (ADR-015)
- Records successful corrections to `KnowledgeIndex`, improving future
  suggestion quality organically
- Provides a clean test surface: fixed inputs produce deterministic JSON

### Trade-offs

- LLM-backed correction (embedded/Ollama/vLLM) is slower than thefuck's
  rule-based match (≤50ms). Acceptable: the user already experienced a failed
  command; 2–5s for a correct fix is better than instant wrong suggestion.
- `Dangerous` classification may over-block in edge cases (e.g., `git push`
  is classified as having network side effects even for non-destructive pushes)
  — initial version errs conservative; future tuning via `SideEffectsAngle`

### Risks

- **Risk: LLM generates plausible but wrong fix.** Mitigation: `confidence`
  field is always returned; shell wrappers should show the rationale before
  executing. Safety validator catches structural violations.
- **Risk: `SideEffectsAngle` classification produces false Dangerous on common
  read-heavy commands.** Mitigation: `SideEffectsAngle` is warn-only today;
  the IdempotencyClass mapping (see Implementation Notes) starts conservative
  and can be tuned against the eval suite before shipping.

---

## Alternatives Considered

### Alternative 1: Integrate thefuck's rule database

Adopt thefuck's 200+ rules as the correction engine (pattern match first, LLM
fallback).
- Pro: instant offline matches for common cases; known good patterns
- Con: pattern drift — rules encode tool-specific output strings that become
  stale. Maintaining a Rust port of a Python rule database adds ongoing
  maintenance cost. Doesn't solve the side-effect guard problem (thefuck
  auto-executes). **Rejected.**

### Alternative 2: Shell widget (interactive, inline correction)

Display the fix suggestion inline in the prompt after a non-zero exit, like
Warp Active AI.
- Pro: lowest friction UX
- Con: requires PTY/terminal control; cannot work as a subprocess; scope is
  2–3x larger. This is the right v2 surface once `caro fix` exists as the
  generator backend. **Deferred to follow-on feature (shell integration).**

### Alternative 3: Extend `caro ai` with `--fix` flag

Add `caro ai --fix --last-exit-code $?` to the existing `caro ai once-mode`.
- Pro: reuses existing session infrastructure
- Con: conflates session management with one-shot correction; `caro ai` manages
  conversational context which is irrelevant for a single-command fix; exit code
  contract for `caro ai` is not currently machine-readable JSON. **Rejected.**

### Alternative 4: MCP tool only (no CLI verb)

Skip `caro fix` as a CLI verb; expose self-healing only as an MCP tool via
ADR-015's `caro-mcp` server.
- Pro: aligns with agent-first architecture
- Con: makes `caro fix` unavailable to non-MCP consumers (CI scripts, shell
  wrappers, humans). The CLI verb is the primitive; the MCP tool wraps it in
  a follow-on. **Rejected.**

---

## New Types

All three types go in **`src/models/mod.rs`** (no new module). All derive
`Serialize, Deserialize, JsonSchema`.

```rust
/// Structured context about a command that failed at runtime.
///
/// Used as input to `AgentLoop::fix_command()` and the `caro fix` CLI verb.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailureContext {
    /// The shell command that was executed and failed.
    pub command: String,
    /// Exit code returned by the shell (non-zero indicates failure).
    pub exit_code: i32,
    /// Captured stderr from the failed execution.
    pub stderr: String,
    /// Captured stdout from the failed execution (may be empty).
    pub stdout: String,
    /// Original natural-language intent that produced the command, if known.
    pub original_prompt: Option<String>,
    /// Working directory at time of failure (for context-aware correction).
    pub cwd: Option<String>,
}

/// A suggested correction for a failed command.
///
/// Always returned by `caro fix` as JSON regardless of whether a fix
/// was found. `fixed == false` is a valid, informative response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FixSuggestion {
    /// True iff a safe correction was generated.
    pub fixed: bool,
    /// The corrected command (None if `fixed == false`).
    pub command: Option<String>,
    /// Human-readable explanation of what was changed or why no fix is possible.
    pub rationale: String,
    /// Idempotency classification of the *original* command.
    /// Callers should surface this to users before auto-executing any fix.
    pub idempotency: IdempotencyClass,
    /// Model confidence in the correction (0.0–1.0). 0.0 when `fixed == false`.
    pub confidence: f32,
    /// Echo of the original command for correlation in pipelines.
    pub original_command: String,
}

/// Classification of whether a command's effects can be safely re-applied.
///
/// Derived from `SideEffectsAngle` output on the original command.
/// Used to gate auto-execution in shell wrappers and CI.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum IdempotencyClass {
    /// Read-only or inherently idempotent. Safe to retry the fix.
    /// Examples: `ls`, `cat`, `git status`, `grep`, `curl -s` (GET only)
    Safe,
    /// Partial state change possible; retry may cause duplicate effects.
    /// Examples: `mkdir -p`, `touch`, `pip install`, `git commit`
    Caution,
    /// Irreversible or has network/destructive side effects.
    /// `caro fix` returns `fixed: false` when original command is Dangerous.
    /// Examples: `rm`, `dd`, `git push --force`, `curl | bash`, `sudo *`
    Dangerous,
}
```

---

## Minimal File Changeset

No new modules. Changes are additions to existing files only.

| File | Change |
|------|--------|
| `src/models/mod.rs` | Add `FailureContext`, `FixSuggestion`, `IdempotencyClass` |
| `src/agent/mod.rs` | Add `AgentLoop::fix_command(&self, ctx: &FailureContext) -> Result<FixSuggestion, GeneratorError>` |
| `src/prompts/command_templates.rs` (or equivalent) | Add `build_fix_prompt(ctx: &FailureContext) -> String` |
| `src/cli/mod.rs` (verb dispatch) | Add `fix` subcommand; parse `--command`, `--exit-code`, `--stderr`, `--prompt`; accept stdin JSON |
| `src/caroml/validators/side_effects.rs` | Add `classify_idempotency(command: &str) -> IdempotencyClass` helper (reuses existing detection patterns) |
| `Cargo.toml` | No new dependencies |

**`AgentLoop::fix_command` pseudocode:**

```
fix_command(ctx):
  1. idempotency = SideEffectsAngle::classify_idempotency(ctx.command)
  2. if idempotency == Dangerous:
       return FixSuggestion { fixed: false, idempotency: Dangerous, rationale: "…" }
  3. prompt = build_fix_prompt(ctx)
  4. corrected = backend.generate(prompt).await?
  5. safety_result = SafetyValidator.validate(corrected.command)
  6. if safety_result.blocked:
       return FixSuggestion { fixed: false, idempotency, rationale: safety_result.reason }
  7. record_correction(ctx.command, corrected.command, feedback=ctx.stderr)  // knowledge index
  8. return FixSuggestion { fixed: true, command: corrected.command, idempotency, confidence: corrected.confidence }
```

---

## Integration Tests

Tests in `tests/fix_integration.rs` (or `src/cli/fix_tests.rs`).
All tests use deterministic static-matcher responses (no LLM required).

| Input | Expected output | Exit code |
|-------|----------------|-----------|
| `--command "ls /nonexistent" --exit-code 2 --stderr "No such file or directory"` | `fixed: true`, command corrects path | `0` |
| `--command "git push" --exit-code 128 --stderr "Permission denied (publickey)"` | `fixed: false`, rationale mentions SSH key, `idempotency: Caution` | `3` |
| `--command "rm -rf /var/log" --exit-code 1 --stderr "Permission denied"` | `fixed: false`, `idempotency: Dangerous`, exit `2` | `2` |
| `--command "curl http://bad-url" --exit-code 6 --stderr "Could not resolve host"` | `fixed: true`, corrected URL format | `0` |
| stdin JSON `{"command":"dd if=/dev/zero of=/dev/sda","exit_code":1,...}` | `fixed: false`, `idempotency: Dangerous`, exit `2` | `2` |
| `--command "python3 script.py" --exit-code 1 --stderr "ModuleNotFoundError: No module named 'requests'"` | `fixed: true`, suggests `pip install requests && python3 script.py` | `0` |

---

## Explicit Out-of-Scope (v1)

The following belong in subsequent iterations:

- **Auto-execution**: `caro fix` never runs the corrected command. A `--execute` flag is explicitly out of scope until the idempotency classifier has been validated against the eval suite.
- **Shell widget**: inline inline-prompt fix suggestion after non-zero exit (requires PTY integration; depends on this ADR's foundation).
- **Knowledge-index-assisted suggestions**: using `KnowledgeIndex::search()` to surface past similar corrections as few-shot examples in the fix prompt. Infrastructure exists; plumbing deferred.
- **MCP tool**: exposing `caro fix` as an MCP tool via `caro-mcp` (ADR-015). ADR-015 is still Proposed; this follows after merge.
- **Multi-step correction chains**: re-running after a corrected command also fails. Single-shot only in v1.
- **CI streaming mode**: `--stream` flag for real-time JSON events during fix generation. Deferred.

---

## What Breaks at 100 Real Users

*(Required by `.claude/rules/validation-discipline.md` Gate 3)*

**Data model assumption**: `SideEffectsAngle::classify_idempotency()` classifies
commands by pattern match on the command string, not by observing actual system
state. It will misclassify at the margins.

**Failure mode**: A `git commit` on a repo with a pre-commit hook that sends
a Slack notification is classified `Caution` (partial state change), but is
actually `Dangerous` at that repo's scale (external effect). At 100 users
across 100 different repos, roughly 10–20% of `Caution` classifications will
underestimate actual side-effect scope.

**Instrumentation**: The `FixSuggestion` JSON includes `idempotency` in all
responses. Telemetry (opt-in) records the `(idempotency, command_prefix,
exit_code)` triple. If users report that a `Caution`-classified command caused
unintended effects, the pattern is visible in telemetry.

**Fallback**: Shell wrappers must always show the `idempotency` field and
`rationale` to the user before offering execution. The UX contract is
"suggest, never execute." The idempotency classification informs the warning
level shown to the user; it does not gate output availability.

---

## Success Metrics

- **Fix rate**: `fixed: true` responses / total `caro fix` invocations ≥ 60%
  for the 20 most common Unix error classes (ENOENT, EPERM, ECONNREFUSED, etc.)
- **False Dangerous rate**: `Dangerous` classification on actually-safe commands
  ≤ 5% (measured against curated eval set)
- **Safety regression**: zero safety-validator bypasses introduced by fix
  suggestions (enforced by existing test suite running on correction output)
- **Subprocess composability**: `echo '...' | caro fix | jq .command` works
  in a single pipeline; tested in CI

---

## Validation Gate Status

Per `.claude/rules/validation-discipline.md`:

| Gate | Status | Notes |
|------|--------|-------|
| Gate 1 (20 transcripts) | **0/20** | This is a new feature extension of the core loop — exempt by the "extension" rule if scoped to the existing caro user. Needs team decision. |
| Gate 2 (no surveys only) | n/a | No survey evidence |
| Gate 3 (demoware-trap) | ✅ | "What Breaks at 100 Real Users" section above |
| Gate 4 (devil's advocate) | ⏳ | Not yet run — required before implementation PR |
| Gate 5 (PMF) | n/a | Not claiming PMF |

**Recommended action before opening implementation PR**: run the
`devils-advocate` agent against this ADR, and confirm with the team whether
`caro fix` qualifies as a "core loop extension" (exempt from Gate 1) or a
"new product line" (requires 20 transcripts). The distinction turns on whether
caro's existing users already encounter post-execution failures — high
probability yes, but unconfirmed by first-hand interview.

---

## References

- Warp Active AI documentation: https://docs.warp.dev/agent-platform/local-agents/active-ai/
- thefuck architecture: https://nvbn.github.io/2015/10/08/how-thefuck-works/
- Atuin AI Agent Hooks (PostToolUseFailure pattern): https://docs.atuin.sh/cli/guide/agent-hooks/
- GitHub Copilot CLI automation issue: https://github.com/orgs/community/discussions/148439
- v2.0 validation audit: `docs/discovery/v2.0-validation-audit.md`
- ADR-015 (MCP Safety Server): `docs/adr/ADR-015-mcp-safety-server.md`
- `src/caroml/validators/side_effects.rs` — existing `SideEffectsAngle`
- `src/execution/executor.rs` — existing `ExecutionResult` (failure data source)
- `.claude/rules/validation-discipline.md` — five validation gates

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-01 | `caro-research--scoping-process` | Initial draft — autonomous research/scoping run |
