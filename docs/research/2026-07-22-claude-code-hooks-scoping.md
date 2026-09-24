# Scoping: `caro hook` — Universal Agent Guardrail (Claude Code PreToolUse adapter)

**Date**: 2026-07-22 (automated research/scoping run)
**Status**: Research report — NOT a spec PR, NOT committed. Untracked file per git-workflow rules.
**Feature researched**: Claude Code hooks (PreToolUse command-validation hooks)

> Note on autonomy: the scheduled task template's `[FEATURE NAME]` placeholder was
> unfilled. Target chosen autonomously: Claude Code's hooks system, because (a) caro's
> own repo already consumes it (`.claude/hooks/block-main-commits.sh`), (b) its contract
> maps 1:1 onto caro's safety validator, and (c) it satisfies every constraint in the
> task (pure subprocess, exit codes, structured output, reuse of safety infra).

---

## Phase 1 — Feature Research: Claude Code hooks

Source: https://code.claude.com/docs/en/hooks (fetched 2026-07-22; key sections on
input/output contract, exit codes, and PreToolUse decision control read in full).

### Problem it solves, and for whom

Deterministic policy enforcement at agent tool-call boundaries. Prompting an LLM to
"never run dangerous commands" is probabilistic; a hook is a guarantee. Users: teams
running Claude Code (interactive or headless/CI) who need guardrails, audit logging,
input rewriting, or custom permission flows.

### Core architecture

- Event-driven: events fire at session lifecycle points (`PreToolUse`, `PostToolUse`,
  `UserPromptSubmit`, `SessionStart`, `PermissionRequest`, …). A `matcher` (regex over
  `tool_name` for tool events) selects which hooks run.
- Three handler types: `command` (subprocess, JSON on stdin), `http` (POST body),
  `mcp_tool`. Command hooks are the GA-shaped core; per-event JSON in, exit code +
  stdout JSON out.
- **PreToolUse stdin payload** (command hooks):
  `session_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`,
  plus event-specific `tool_name`, `tool_input` (for Bash: `{command, description}`),
  `tool_use_id`.
- **Decision channel**: PreToolUse returns decisions inside `hookSpecificOutput`:
  `permissionDecision`: `allow | deny | ask | defer`, with `permissionDecisionReason`,
  optional `updatedInput` (rewrites tool args pre-execution). Multi-hook precedence:
  `deny > defer > ask > allow`.
- Separation of concerns: Claude Code owns event dispatch, matching, timeout,
  precedence merging; the hook owns judgment. Hooks are stateless subprocesses — the
  only session continuity handle is the `session_id` field in the payload.

### Structured output contract

| Mechanism | Meaning |
|---|---|
| Exit 0 + stdout JSON | Success; decision read from `hookSpecificOutput` |
| Exit 2 | Blocking error; **stdout/JSON ignored**, stderr fed to Claude as the error |
| Any other exit code | **Non-blocking error — execution continues** (fail-open) |
| Timeout (default 600s command hooks; 30s UserPromptSubmit; 10s MessageDisplay) | Hook canceled, output discarded — fail-open |

### Why it's limited / failure modes (Phase 1 findings to solve by design)

1. **F1 — Fail-open on error.** A hook that crashes (any exit code other than 0/2)
   or times out is a *non-blocking* error: the tool call proceeds. A buggy or slow
   safety hook silently stops protecting. This is the primary failure mode to solve
   by design.
2. **F2 — Every author reinvents danger detection, badly.** The docs' own canonical
   example greps for the substring `rm -rf`. It misses `rm -fr`, `rm -r -f`, fork
   bombs, `dd of=/dev/sda`, `curl | sudo bash`, encoded variants — the exact corpus
   caro's 52+ context-aware compiled patterns (+ CVE patterns) already cover with a
   zero-false-positive test suite. The hook system ships the *transport*, not the
   *judgment*.
3. **F3 — Split-brain output channels.** Exit 2 ignores stdout JSON; exit 0 requires
   JSON for a decision. Authors who mix models (exit 2 + JSON) get silently wrong
   behavior. caro's own `block-main-commits.sh` exits `1` on block — which per the
   contract is a **non-blocking** error (see F1); it works today only because stderr
   messaging persuades the model, not because the block is enforced. Dogfooding bug.
4. **F4 — Per-call cold start.** One subprocess per tool call. Anything heavy (LLM
   judge, model load) is unusable in the hot path; the design rewards sub-millisecond
   deterministic validators — precisely caro's static path (`once_cell` precompiled
   patterns, no model load).

### Session/context lifecycle

No daemon, no persistent state; redundant initialization is avoided only by keeping
hooks cheap, not by caching. `session_id` enables *optional* external state (e.g.
per-session audit logs) but nothing in the contract requires it. This validates the
task constraint: a pure subprocess with fast static init is the winning shape.

---

## Phase 2 — Competitive Differentiation

### What they get right (replicate)

- Machine-first JSON contract with a *reason* field alongside every decision.
- Four-outcome decision space (`allow/deny/ask/defer`) — richer than binary
  block/allow; `ask` routes to a human, mapping exactly onto caro's `HumanGate`.
- Matcher-scoped invocation and strict precedence ordering (`deny > defer > ask > allow`).
- Statelessness as the contract; timeouts bounded by the caller.

### Their gaps we avoid by designing our schema first

- **Fail-closed by construction** (vs F1/F3): caro's hook mode always emits valid
  decision JSON and exits 0 on any evaluable input. Internal errors emit
  `permissionDecision: "ask"` with the error as reason — never a bare non-zero exit
  that fails open. Exit codes ≠ decision channel; JSON is the single source of truth.
- **Judgment included** (vs F2): decision quality comes from the maintained,
  TDD-tested pattern database, not from per-user bash/jq.
- **Graded risk, not binary**: `RiskLevel` × `SafetyLevel` → `SuggestedRouting`
  (`AutoApprove/AsyncLog/HumanGate/Block`) gives policy-tunable outcomes; Claude Code
  hooks have no notion of a risk gradient or a user policy knob.

### Our unique positioning

- **Offline & deterministic**: no network, no model in the hook path; sub-ms latency.
- **Universal**: the same subprocess can back Claude Code today and Gemini CLI /
  Codex / opencode adapters later — one guardrail brain, N agent frontends. Claude
  Code hooks only guard Claude Code.
- **Standalone**: single Rust binary from crates.io/Homebrew; no runtime deps (the
  docs' examples require `jq`).
- **Community layer**: user patterns via `SafetyConfig` (already ReDoS-hardened,
  `Critical` reserved for built-ins) become shareable guardrail packs.

### Existing infrastructure already covering this

| Piece | Where | Status |
|---|---|---|
| `SafetyValidator::validate_command(cmd, shell)` | `src/safety/mod.rs` | ✅ done, async, static path needs no LLM |
| `ValidationResult` (`Serialize`/`Deserialize`) | `src/safety/mod.rs:175` | ✅ serializable today |
| `SafetyDecision` + `SuggestedRouting::from_risk_and_safety` | `src/safety/mod.rs:189`, `src/models/mod.rs:189` | ✅ exact semantic core of `permissionDecision` mapping |
| `OutputFormat::{Json,Yaml,Plain}` | `src/cli/mod.rs:105` | ✅ precedent for machine output + consent skipping (`main.rs:3337`) |
| Special exit-code precedent (`EXIT_CODE_EDIT = 201`) | `src/main.rs:938` | ✅ pattern for stable exit contracts consumed by shell wrappers |
| No-LLM subcommand precedent (`caro check` — "No LLM calls, no execution") | `src/main.rs:512` | ✅ shape to copy |
| Config/SafetyLevel loading | `src/config` | ✅ reuse as-is |

Net: this feature is ~90% wiring. The judgment engine, types, serialization, and
policy mapping all exist.

---

## Phase 3 — Scope Definition

### Feature

`caro hook` — read an agent hook event as JSON on stdin, validate the command with
the existing static safety pipeline, emit a Claude Code-compatible decision on
stdout. Pure subprocess; no daemon; no LLM; no state.

```bash
# .claude/settings.json
{ "hooks": { "PreToolUse": [ { "matcher": "Bash",
    "hooks": [ { "type": "command", "command": "caro hook --format claude-code" } ] } ] } }
```

### ADR draft (to be filed as next sequential ADR — renumber on merge per adr-numbering.md)

- **Title**: ADR-0XX: Agent guardrail hook mode (`caro hook`)
- **Context**: Agent runtimes (Claude Code hooks GA'd the contract; Gemini CLI and
  Codex converging on similar shapes) delegate tool-call safety to user-authored
  subprocess hooks. The reference implementations are naive substring greps and the
  contract fails open on hook error. Caro already owns a hardened command-safety
  engine with serializable verdict types.
- **Decision**: Add a `hook` subcommand that adapts stdin hook events to
  `SafetyValidator` and maps `SuggestedRouting` → `permissionDecision`. JSON-on-stdout
  is the sole decision channel; the process exits 0 whenever it produced a decision,
  including on internal errors (which yield `ask` + reason). Static patterns only —
  the embedded/remote LLM backends are never loaded in this path.
- **Consequences**: (+) caro becomes a drop-in guardrail for any hooks-capable agent;
  fail-closed where the native contract fails open; sub-ms hot path. (−) we track an
  external, evolving schema (mitigate: `--format` versioned adapter enum, unknown
  stdin fields ignored via serde defaults); binary gains a subcommand surface that
  must stay stable.
- **Alternatives considered**: (a) MCP server exposing a `validate` tool — rejected:
  requires daemon/session, violates pure-subprocess constraint, and PreToolUse can't
  call MCP tools portably; (b) documenting a jq recipe around `caro --output json` —
  rejected: keeps F2/F3 alive, jq dependency; (c) generic `--stdin-json` on the main
  generate path — rejected: conflates generation with validation.

### New types (added to existing modules — no new module trees)

In `src/safety/mod.rs` (or `src/cli/hook.rs` as a file within the existing `cli`
module — one new *file*, zero new modules):

```rust
/// Parsed agent hook event (Claude Code PreToolUse wire format).
/// serde(default) everywhere: unknown/missing fields must not fail closed the parse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    #[serde(default)] pub session_id: String,
    #[serde(default)] pub cwd: String,
    #[serde(default)] pub hook_event_name: String,   // expect "PreToolUse"
    #[serde(default)] pub tool_name: String,          // expect "Bash"
    #[serde(default)] pub tool_input: HookToolInput,
    #[serde(default)] pub tool_use_id: Option<String>,
    #[serde(default)] pub permission_mode: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookToolInput {
    #[serde(default)] pub command: String,
    #[serde(default)] pub description: Option<String>,
}

/// Outbound decision, Claude Code wire format (camelCase).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookOutput { pub hook_specific_output: HookDecision }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookDecision {
    pub hook_event_name: String,                     // "PreToolUse"
    pub permission_decision: PermissionDecision,     // allow | deny | ask
    pub permission_decision_reason: String,          // explanation + matched patterns + risk level
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionDecision { Allow, Deny, Ask }

impl From<SuggestedRouting> for PermissionDecision {
    // AutoApprove → Allow; AsyncLog → Allow (reason notes "logged");
    // HumanGate → Ask; Block → Deny
}
```

Method contract: `pub async fn decide_hook(event: &HookEvent, cfg: &SafetyConfig) -> HookOutput`
— total function; never returns `Err` to the caller path (internal failures map to
`Ask` with the error text in the reason). All types serializable from day one;
`ValidationResult`/`SafetyDecision` reused, not duplicated.

### Minimal file changes

1. `src/main.rs` — add `Hook { format: HookFormat, exit_codes: bool }` variant to
   `Commands`; wire dispatch (mirrors `Check`'s no-LLM shape).
2. `src/cli/hook.rs` (new file inside existing `cli` module) — stdin read, types
   above, mapping, stdout emit.
3. `src/safety/mod.rs` — `impl From<SuggestedRouting> for PermissionDecision` (or keep
   in `cli/hook.rs` to avoid touching safety at all — preferred).
4. `tests/hook_integration.rs` — integration tests below.
5. `docs/` + README one-liner — deferred to feature PR per repo docs-sync flow.

### Exit code / output contract (what machines depend on)

Default mode (`--format claude-code`, recommended for Claude Code):

| Condition | stdout | exit |
|---|---|---|
| Command evaluated (allow/ask/deny) | one-line `HookOutput` JSON | **0** |
| Malformed/empty stdin, unknown event, missing command | `HookOutput` with `ask` + reason `"caro hook: <error> — failing closed"` | **0** |
| stdout itself unwritable (EPIPE etc.) | — | **74** (EX_IOERR) |

Opt-in `--exit-codes` mode (for non-JSON consumers / plain CI):
`0` = allow, `2` = deny (stderr carries reason — matches Claude Code's blocking-exit
semantics), `3` = ask, `64` = unusable input (EX_USAGE). Never any other code, so a
crash is distinguishable from a verdict.

Stability promise: `permissionDecision` values and the exit-code table are a public
contract from first release; additive JSON fields only.

### Integration tests (known input → deterministic output)

Fixtures as literal stdin JSON; assert exact `permission_decision`, reason substring,
exit code. No LLM, no network → fully deterministic.

1. `rm -rf /` (Bash, moderate SafetyLevel) → `deny`, reason contains pattern id, exit 0.
2. `ls -la` → `allow`, exit 0.
3. `chmod -R 777 /` at `SafetyLevel::Moderate` → `ask` (HumanGate), exit 0.
4. Same at `SafetyLevel::Permissive` → `allow`-with-logged-reason (AsyncLog mapping).
5. Malformed stdin (`{not json`) → `ask`, reason mentions parse failure, exit 0 (fail-closed proof — the F1 regression test).
6. `tool_name: "Write"` (non-command tool) → `allow` with reason "not a command tool", exit 0.
7. `--exit-codes` mode: `rm -rf /` → exit 2, stderr non-empty, stdout still valid JSON.
8. Fork bomb `:(){ :|:& };:` → `deny` (the case the docs' jq example misses — differentiation test).

### Constraints compliance

- Reuses `SafetyValidator`/`SafetyConfig`/`SuggestedRouting` — zero duplication. ✅
- All new types `Serialize + Deserialize` from day one. ✅
- Pure subprocess: stdin→stdout, no daemon, no state, no model load. ✅
- F1 (fail-open) solved by design: decision JSON + exit 0 is the only success path;
  errors become `ask`, never a non-blocking crash. ✅

### Out of scope (next version)

- Other events (`PostToolUse` audit logging, `UserPromptSubmit` context injection).
- `updatedInput` command rewriting (e.g. auto-adding `--dry-run`) — high value, needs
  its own safety review.
- `defer` support and SDK resume flows.
- `--format gemini-cli` / `--format codex` adapters (the `HookFormat` enum reserves
  the seam).
- LLM-judge escalation for `ask` cases; community pattern-pack distribution;
  HTTP hook server mode (violates no-daemon constraint by definition).

### Process notes / risks

- **Validation-discipline gate check**: build size is small (<1 week), but this *is*
  a new user-facing capability. Recommend an explicit maintainer ruling on whether
  the five gates apply or it qualifies as an integration of the already-validated
  core loop (natural-language→safety-validated command). Flagging rather than assuming.
- **Dogfooding fix rides along**: `.claude/hooks/block-main-commits.sh` exits 1 on
  block — a non-blocking error under the actual contract. Migrating repo hooks to
  `caro hook` (or at least exit 2) closes a real enforcement gap found during this
  research (F3).
- ADR number assigned at PR time per `adr-numbering.md`; release-version rules not
  triggered (no version bump in the feature PR).
