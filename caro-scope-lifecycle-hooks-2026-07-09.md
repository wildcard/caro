# Scope: Lifecycle Hooks for Caro (ADR-034 draft)

> Scheduled research run 2026-07-09 (`caro-research--scoping-process`).
> Feature chosen autonomously: the task template's `[FEATURE NAME]` was unset.
> Selected **Claude Code hooks** (PreToolUse/PostToolUse lifecycle hooks) because
> prior runs already covered headless/stream-json (ADR-024–029), receipts
> (ADR-032), undo snapshots (ADR-033), circuit breaker (ADR-031), MCP safety
> server (ADR-015-mcp) and skills extension (#664) — and `grep -ril hook src/`
> confirms caro has no lifecycle-hook system today.
>
> **Not committed** (git-workflow rule: no work on main). To land: `bin/sk-new-feature`,
> move ADR section to `docs/adr/ADR-034-lifecycle-hooks.md`, open a PR.

---

## Phase 1 — Feature research: Claude Code hooks

Source: official hooks reference (code.claude.com/docs/en/hooks, fetched 2026-07-09).

**Problem and audience.** Deterministic, user-owned control points around an
agent's lifecycle: block dangerous tool calls, enforce org policy, inject
context, log/audit — guarantees that don't depend on the model choosing to
comply. Audience: individual power users and platform/security teams.

**Architecture.**
- Config lives in settings JSON (user/project/local), snapshotted at session
  start to prevent mid-session tampering.
- Event fires → `matcher` filters (regex/exact hybrid on e.g. `tool_name`) →
  all matching handlers run **in parallel**, deduplicated by command string.
- Five handler types: `command`, `http`, `mcp_tool`, `prompt`, `agent`. The
  primitive is the **command hook**: event JSON on stdin (common fields
  `session_id`, `transcript_path`, `cwd`, `hook_event_name` + per-event
  fields), result via exit code + stdout.
- Decision control: exit 0 + stdout JSON → `hookSpecificOutput` with
  `permissionDecision: allow|deny|ask`, `permissionDecisionReason`,
  `updatedInput` (rewrite tool args), `additionalContext`; universal fields
  `continue`, `stopReason`, `systemMessage`. Output strings capped at 10k chars.
- Timeouts: 600s default; 30s for UserPromptSubmit; 10s for MessageDisplay.
- Env contract: `CLAUDE_PROJECT_DIR`; `CLAUDE_ENV_FILE` for persisting env
  vars across the session (SessionStart/Setup/CwdChanged/FileChanged).

**Structured output contract (exit codes).**
| Exit | Meaning |
|---|---|
| 0 | Success; stdout parsed as JSON decision if valid, else shown as info |
| 2 | Blocking error; stdout ignored, stderr fed to the model; blocks the action *for events that can block* |
| other | **Non-blocking error; execution continues** (stderr → transcript notice) |

HTTP hooks: 2xx = success; non-2xx / connection failure / timeout = **non-blocking, continues**.

**Why it's limited / failure modes (the gaps to solve by design):**
1. **Fail-open policy enforcement.** A crashed hook (exit ≠ 0/2), a timed-out
   hook, or an unreachable HTTP endpoint is a *non-blocking* error — the tool
   call proceeds. A broken security hook silently stops protecting.
2. **Inconsistent exit-2 semantics per event.** Exit 2 blocks PreToolUse but
   is ignored (notice only) for SessionStart/Setup/SubagentStart; each event
   has its own table. Users must memorize a matrix.
3. **Matcher footguns.** Hybrid regex/exact evaluation: `mcp__memory` is
   exact-match and matches nothing (needs `mcp__memory__.*`); `Edit.*` also
   matches `NotebookEdit`; FileChanged/StopFailure use a narrower charset.
4. **Silent timeout data loss.** A timed-out UserPromptSubmit hook's
   `additionalContext` is discarded; before v2.1.196 with no notice at all.
5. **Unversioned payload.** No `schema_version` in stdin JSON; scripts break
   silently across releases.
6. **Prompt-injection surface.** `additionalContext` flows into model context;
   docs advise phrasing workarounds rather than enforcing provenance.

**Session/context lifecycle.** Redundant initialization is avoided via the
startup config snapshot (hooks read once per session), handler dedup, and
`CLAUDE_ENV_FILE` for cross-command env persistence. There is no per-hook
warm state — every firing is a fresh subprocess (which is what keeps the
contract simple and language-agnostic).

## Phase 2 — Competitive differentiation

**Replicate (they got this right):**
- Pure-subprocess contract: JSON on stdin, exit code + stdout JSON back.
  Language-agnostic, no SDK, trivially testable. Matches caro's own
  "pure subprocess call" constraint.
- Structured decision object instead of exit-code-only control.
- Config snapshot at process start; per-hook `timeout`; handler dedup.
- `ask` as a first-class decision (maps directly onto caro's tiered
  approval, ADR-020).

**Avoid (design our schema first):**
- Fail-open default → caro hooks get an explicit per-hook `on_failure`
  policy, default **fail-closed** for pre-execution events.
- Per-event exit-code matrix → one uniform exit contract for all events.
- Regex/exact hybrid matchers → v1 has **no matchers at all**; hooks fire per
  event and filter themselves from the payload (they already get full JSON).
  Matchers are v2 if profiling shows spawn cost matters.
- Unversioned payload → `schema_version: 1` in every `HookInput`.
- Arbitrary context injection → hook output can annotate the *user-facing*
  result, never the LLM prompt (out of scope v1), eliminating the
  injection surface.

**Caro's unique positioning.**
- **The safety floor.** Claude Code hooks are the *only* policy layer; caro
  hooks compose with the deterministic 52-pattern `SafetyValidator`. Design
  rule: hooks can tighten, never loosen — a hook `allow` cannot override a
  CRITICAL pattern block. Competitors have no equivalent deterministic floor.
- **Offline & auditable.** Local LLM + local hooks; no HTTP handler type in
  v1 means no fail-open network dependency in the policy path.
- **One-shot CLI.** No session daemon; hooks integrate into a single
  generate→validate→confirm→execute pass, so the whole contract stays
  scriptable (`caro --headless` per ADR-024).

**Existing infra that already covers part of this:**
- `SafetyValidator`, `SafetyDecision`, `ValidationResult` (`src/safety/mod.rs`)
  — the decision model hooks merge into; `blend_smart_decision` is precedent
  for merging an external judgment into a `SafetyDecision`.
- `ExecutionContext::capture()` (`src/execution/mod.rs`) — the cwd/shell/
  platform/user payload `HookInput` needs already exists and is serializable.
- `ConfigManager` + schema validation (`src/config/`) — hooks config slots in
  as a new TOML section with existing merge/validate machinery.
- ADR-024 `ExitCode` + `HeadlessEnvelope` — hook outcomes surface as fields in
  the existing headless envelope, not a second contract.
- `CommandExecutor` (`src/execution/executor.rs`) — reused to spawn hook
  subprocesses with timeout (`with_timeout`).

## Phase 3 — Scope definition

### ADR-034: Lifecycle hooks (context / decision / consequences / alternatives)

- **Context.** Teams adopting caro in CI (caro-scan, ADR-023) and org
  settings need policy extension without forking `patterns.rs`. Claude Code
  normalized a hooks contract; its fail-open flaw is documented above.
- **Decision.** Add four lifecycle events — `pre_generate`, `post_generate`,
  `pre_execute`, `post_execute` — that run user-configured executables as
  pure subprocesses: versioned JSON on stdin, uniform exit-code + stdout-JSON
  contract, explicit `on_failure` policy (default `block` for `pre_*`,
  `warn` for `post_*`), and a hooks-cannot-loosen safety floor. Hooks run
  sequentially in config order (deterministic; parallelism is v2).
- **Consequences.** (+) org policy without recompiling; CI-composable;
  fail-closed by default. (−) added latency per hook (bounded by
  `timeout_ms`, default 5000); a misconfigured fail-closed hook can block
  all executions (mitigated by `caro hooks doctor` in v2 and clear stderr).
- **Alternatives considered.** (a) WASM plugin ABI — rejected v1: heavy,
  language-restrictive, duplicates subprocess semantics. (b) Lua/Rhai
  embedded scripting — rejected: new interpreter surface inside the safety
  path. (c) HTTP webhooks — rejected v1: reintroduces the fail-open network
  failure mode found in Phase 1. (d) Only more TOML `DangerPattern` entries —
  already exists (`add_custom_pattern`) but can't express contextual policy
  (time, dir, user, external state).

### New types (existing modules only; all `serde::{Serialize, Deserialize}` from day one)

In `src/config/schema.rs` (config section, validated by `ConfigManager::validate_schema`):

```rust
pub struct HooksSection { pub enabled: bool, pub hooks: Vec<HookEntry> }
pub struct HookEntry {
    pub event: HookEvent,          // pre_generate | post_generate | pre_execute | post_execute
    pub run: String,               // executable path (no shell interpolation; argv-style `args: Vec<String>`)
    pub args: Vec<String>,
    pub timeout_ms: u64,           // default 5000
    pub on_failure: HookFailurePolicy, // block | warn | allow; default block for pre_*, warn for post_*
}
```

In `src/execution/hooks.rs` (one new file inside the existing `execution` module — unavoidable minimum):

```rust
pub enum HookEvent { PreGenerate, PostGenerate, PreExecute, PostExecute }
pub struct HookInput {           // stdin payload
    pub schema_version: u32,     // 1
    pub event: HookEvent,
    pub prompt: String,
    pub command: Option<String>,         // populated post_generate onward
    pub risk_level: Option<RiskLevel>,   // populated after validation
    pub validation: Option<ValidationResult>,
    pub execution: Option<ExecutionResult>, // post_execute only
    pub context: ExecutionContextModel, // cwd, shell, platform, user (existing type)
}
pub struct HookOutput {          // parsed from stdout on exit 0/3
    pub decision: Option<HookDecision>,  // allow | deny | ask
    pub reason: Option<String>,
    pub updated_command: Option<String>, // post_generate only; re-validated afterwards
}
pub struct HookOutcome {         // recorded per hook, surfaced in HeadlessEnvelope.hooks[]
    pub entry_index: usize, pub event: HookEvent, pub exit_code: i32,
    pub decision: HookDecision, pub reason: Option<String>,
    pub duration_ms: u64, pub failure_applied: Option<HookFailurePolicy>,
}
pub struct HookRunner { /* Vec<HookEntry>, spawn via CommandExecutor::with_timeout */ }
impl HookRunner {
    pub fn run(&self, event: HookEvent, input: &HookInput) -> Vec<HookOutcome>;
    pub fn merge(outcomes: &[HookOutcome], base: SafetyDecision) -> SafetyDecision;
    // merge precedence: deny > ask > allow; validator CRITICAL block is a floor
    // (mirrors blend_smart_decision in src/safety/mod.rs)
}
```

### Hook exit-code contract (uniform across all four events — fixes Phase-1 gap #2)

| Exit | Meaning | stdout |
|---|---|---|
| 0 | allow (or decision in stdout JSON) | optional `HookOutput` JSON |
| 2 | deny | ignored; stderr = reason shown to user |
| 3 | escalate to ask (tiered approval, ADR-020) | optional `HookOutput` JSON |
| other / timeout / spawn error | apply `on_failure` policy | ignored |

Caro process exit codes are unchanged (ADR-024 `ExitCode`); one addition:
`HookDenied = 7`. `HeadlessEnvelope` gains `hooks: Vec<HookOutcome>`.

### Files that change (minimal set)

1. `src/config/schema.rs` — `HooksSection`, `HookEntry`, schema validation.
2. `src/config/mod.rs` — merge `[hooks]` TOML section (existing merge paths).
3. `src/execution/hooks.rs` — **new file**: types + `HookRunner` (~250 LOC).
4. `src/execution/mod.rs` — `pub mod hooks;`.
5. `src/safety/mod.rs` — `SafetyDecision::apply_hook_outcomes(...)` (floor rule).
6. `src/main.rs` — four call sites in the existing generate→validate→execute flow.
7. `src/cli/mod.rs` (headless envelope per ADR-024) — `hooks` field + `HookDenied` exit code.
8. `docs/adr/ADR-034-lifecycle-hooks.md` + README index row (ADR numbering rule: next sequential, no gaps).

No new top-level modules; no daemon; no state files.

### Integration tests (`tests/hooks_integration.rs`, fixture scripts in `tests/fixtures/hooks/`)

Deterministic known-input → JSON + exit-code assertions:
1. `allow.sh` (exit 0, no stdout) → command proceeds; envelope `hooks[0].decision == "allow"`, process exit 0.
2. `deny.sh` (exit 2, stderr "policy: prod window") → no execution; envelope exit_code 7; reason string surfaced verbatim.
3. `ask.sh` (exit 3) on a LOW-risk command → decision escalates to ask (headless: `NeedsApproval` per ADR-024/027 path).
4. `rewrite.sh` (exit 0, stdout `{"updated_command": "ls -la"}`) at `post_generate` → re-validation runs on the rewritten command (assert a rewrite to `rm -rf /` is still blocked by the validator — the floor test).
5. `hang.sh` (sleeps past `timeout_ms=100`, `on_failure=block`) → deny with `failure_applied: "block"` — **the fail-closed test that Claude Code fails**.
6. `crash.sh` (exit 17, `on_failure=warn`) → proceeds, warning outcome recorded.
7. Hook `allow` + validator CRITICAL pattern → still blocked (floor).
8. Malformed stdout JSON on exit 0 → treated as plain allow + `reason: "unparseable output"` recorded (never a crash).

### Out of scope (next version)

- `http` / `mcp_tool` / `prompt` handler types (v1 is command-only, offline).
- Matchers/filtering DSL; parallel hook execution; hook dedup.
- Context injection into the LLM prompt (`additional_context`) — injection-surface review first.
- Session-level events (session_start/stop) — caro is one-shot; revisit with ADR-026 multiturn.
- `caro hooks doctor` / `caro hooks list` UX commands.
- Env-var persistence (`CLAUDE_ENV_FILE` equivalent); hook marketplace/community registry.

### Constraint compliance

- Reuses `SafetyValidator`/`SafetyDecision`, `ConfigManager`, `CommandExecutor`,
  `ExecutionContext`, ADR-024 envelope — no duplication.
- All new types serde-serializable from day one.
- Pure subprocess: fresh process per firing, JSON stdin, exit code out; no daemon, no state.
- Phase-1 failure mode #1 (fail-open) solved **by design** via `on_failure`
  defaulting to `block` on pre-execution events; #2 solved by the uniform
  exit table; #3 by omitting matchers; #5 by `schema_version`; #6 by keeping
  hook output out of the model prompt.
