# Scope: Self-Healing Command Repair (`caro fix`)

**Generated**: 2026-05-15 (scheduled research task)
**Roadmap item**: Self-Healing Features — v2.0.0 (#155)
**Proposed ADR**: ADR-009
**Feature name**: `caro fix` — runtime command repair via LLM + safety pipeline

---

## Phase 1 — Feature Research

### What problem it solves and for whom

When a shell command fails (non-zero exit code), the user gets a cryptic
`stderr` message and must debug manually: read the error, understand the
platform nuance, revise the command, and re-run. This is especially painful
for users who used `caro <intent>` to generate the command in the first place
— they trusted caro to get it right, and now they're on their own.

**Target users:**
- DevOps engineers running generated commands on unfamiliar platforms (BSD
  vs GNU flag differences are a dominant failure class)
- CI pipeline authors using `caro run` where failed steps currently just stop
- Shell users learning new tools who got a permissions or syntax error

### Core architecture of competitor implementations

**Atuin v18.13 (`atuin ai`):**
- Architecture: SQLite-backed shell history → PTY proxy captures block I/O →
  aspirational AI repair on block failure (not yet shipped)
- The Atuin blog (2025) explicitly states: "we'd love to explore having AI as
  part of your workflow — for example, suggesting fixes to a failed block
  execution." No code exists yet; it's a roadmap item.
- Key type: `RunOutput { exit_code, stdout, stderr, duration }` stored in
  history per command invocation. Repair would read this and pass to LLM.
- Failure mode: tightly coupled to Atuin's PTY/shell-hook infrastructure.
  Cannot be invoked as a pure subprocess call outside the Atuin shell session.

**Warp Agent Mode (open-sourced April 2026):**
- Architecture: full terminal emulator with PTY blocks. "Active AI" observes
  exit codes and contextual block I/O, then surfaces inline suggestions.
- When a block exits non-zero, the Warp UI highlights the error and offers
  "Ask AI" which sends: block command, stdout (truncated), stderr, exit code,
  current directory, git branch.
- Key weakness: requires Warp as the terminal host. Not callable as a
  subprocess. Not usable in SSH sessions or CI. No `--json` output mode.
  No safety re-validation of the AI's repair suggestion.

**llm-cmd (Simon Willison, OSS):**
- Simplest: runs `llm` CLI with the failed command and output as context.
  No structured output, no safety layer, no repair loop limit.

**RTK (token-reduction kit for LLM commands):**
- Intercepts command output and compresses it before LLM sees it.
  Saves full unfiltered output when command fails. Relevant technique but
  not a repair tool per se.

### Why these are experimental / what the failure modes are

1. **No safety re-validation**: Repair suggestions from Warp and llm-cmd are
   shown directly to users. A repair prompt can be coerced to suggest a
   dangerous command (e.g., `chmod 777 -R /` instead of `chmod 755 ./dir`).
   No competitor routes repair output through a safety validator.

2. **No structured output contract**: All implementations are interactive-TTY
   only. No competitor emits a machine-readable repair event, making them
   unusable in CI or scripting.

3. **Context explosion**: Warp sends full stdout + stderr to the LLM. A
   failing `cargo build` can produce 50KB of output. This explodes token
   budgets and buries the relevant error signal. No competitor implements
   relevance-ranked stderr truncation.

4. **No repair attempt limit**: llm-cmd has no max-iterations guard. In an
   automated context this could loop indefinitely.

5. **Daemon / session coupling**: Atuin and Warp require a running session
   context. Neither works as a pure subprocess call (`caro fix --json ...`).

### Structured output contract (what competitors lack)

No competitor ships a JSON output mode for repair events. Scripts and CI
pipelines cannot reliably parse terminal color-coded suggestions.

### Session/context lifecycle

Competitors re-initialize context (shell detection, CWD, platform) on every
request. The repair call is stateless by design in all implementations —
consistent with caro's "pure subprocess call" constraint. No daemon is needed.

---

## Phase 2 — Competitive Differentiation

### What they get right (replicate)

- **Capture both stdout and stderr** — exit code alone is insufficient for
  repair context. Atuin's `RunOutput` and Warp both capture both streams.
- **Include original intent/query** in the repair prompt, not just the
  command text. The LLM generates better repairs when it knows what the user
  was trying to accomplish.
- **Keep stderr capture bounded**: RTK's core insight — compress/truncate
  output before sending to LLM. Cap at `STDERR_CAPTURE_BYTES = 4096`.

### Their bugs/design gaps we avoid by designing schema first

| Gap | How caro avoids it |
|-----|--------------------|
| No safety re-validation of repair | Every `RepairResult.repaired_command` passes through `SafetyValidator` before being returned to caller |
| No structured output | `RepairResult` is `Serialize` from day one; `--json` flag is first-class |
| Context explosion | Hard cap `STDERR_CAPTURE_BYTES = 4096`; take last N bytes (error tail) |
| No repair attempt limit | `max_repair_attempts: u32` in config, default 2 |
| Session/daemon coupling | Pure fn signature: `repair(req: &RepairRequest) -> RepairResult` — no state |
| Repair of dangerous command | Safety block is a `RepairResult` field, not an error variant — caller can inspect |

### Caro's unique positioning

1. **Safety-first**: No other CLI tool runs repair suggestions through a
   pre-compiled safety pattern database before presenting them to the user.
   The 52-pattern `SafetyValidator` (with CVE patterns) applies to repairs
   exactly as it applies to generation.

2. **Offline-capable**: The embedded backend (SmolLM/CPU) means `caro fix`
   works without an API key or internet access. Warp and Atuin require cloud.

3. **JSON output contract**: `--json` mode makes `caro fix` scriptable in CI
   pipelines. `RepairResult` is the ground truth; the TTY presentation is
   derived from it.

4. **CaroML integration hook**: `caro run` (CaroML step executor) can
   optionally invoke repair on step failure and write a patched variant back
   to the `.caro.lock`. No competitor has a file-backed intent/lock system
   that makes repair auditable across runs.

5. **Platform-aware repair**: `ExecutionContext` already knows BSD vs GNU flag
   differences. The repair prompt is pre-armed with the platform rules that
   already power generation — the LLM sees the same platform context.

### Existing infrastructure that covers part of this

| Existing module | What it provides |
|----------------|-----------------|
| `execution/executor.rs::ExecutionResult` | exit_code, stdout, stderr already captured |
| `safety/validator.rs::SafetyValidator` | re-use directly on repair output |
| `backends::CommandGenerator` trait | repair is a specialized `generate_command` call |
| `context/mod.rs::ExecutionContext` | platform rules for repair prompt |
| `prompts/command_templates.rs` | prompt building pattern to follow |
| `caroml/interpreter.rs` | static repair loop pattern (iterations + prior_failures) to adapt for runtime |
| `models::CommandRequest` | existing request type; repair adds `failure_context` field |

---

## Phase 3 — Scope Definition

### ADR-009: Self-Healing Command Repair

**Context:**
Caro generates shell commands from natural language. When a generated command
fails at runtime, the user must manually debug. Competitor tools (Atuin, Warp)
have planned or partially implemented repair loops, but none route repair
suggestions through a safety validator or emit a machine-readable output
contract. Caro's existing `ExecutionResult`, `SafetyValidator`, and
`CommandGenerator` trait provide all the needed primitives.

**Decision:**
Implement `caro fix` as a new CLI verb. It accepts a failed command + failure
context (exit code + stderr snippet) and returns a repair suggestion that has
been validated by the full safety pipeline. Output in interactive mode is
a human-readable prompt; with `--json` it is a serialized `RepairResult`.
Max repair attempts is configurable (default 2). The feature works as a pure
subprocess call with no daemon or session dependency.

**Consequences:**
- `RepairResult` becomes a stable JSON schema; bumping its fields is a
  semver concern starting from first release.
- The `--json` output mode creates a contract that CI tools may depend on;
  it must be kept backward-compatible.
- Repair prompts consume LLM tokens even for simple flag errors that a
  static lookup could handle; accepted trade-off for v1 scope.

**Alternatives considered:**
- *Static repair rules database* — fast but brittle; can't handle novel
  errors. Deferred to a future optimization layer.
- *Atuin-style PTY integration* — requires shell hooks and a running
  session. Violates the "pure subprocess call" constraint.
- *Re-running the full generation pipeline* — would ignore the failure
  context entirely; the repair prompt must include stderr to be useful.

---

### New types (fields, serialization, method contracts)

All new types live in **existing modules** — no new top-level modules.

#### `src/execution/repair.rs` (new file within `execution/` module)

```rust
use serde::{Deserialize, Serialize};

/// Caller-supplied failure context for a repair request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairRequest {
    /// The command that failed.
    pub original_command: String,
    /// The original natural-language intent that produced the command.
    /// Empty string if caller does not know it (ad-hoc fix mode).
    pub intent: String,
    /// Exit code of the failed command.
    pub exit_code: i32,
    /// Last STDERR_CAPTURE_BYTES bytes of stderr. Caller truncates.
    pub stderr_snippet: String,
    /// Platform identifier (e.g. "macos-arm64", "linux-x86_64").
    pub platform: String,
    /// Shell name (e.g. "bash", "zsh", "fish").
    pub shell: String,
}

/// Maximum bytes captured from stderr. Enforced by the caller before
/// constructing RepairRequest to keep LLM context bounded.
pub const STDERR_CAPTURE_BYTES: usize = 4096;

/// Outcome of one repair attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    /// The original failing command (echoed for machine consumers).
    pub original_command: String,
    /// The repaired command, if one was found and passed safety checks.
    pub repaired_command: Option<String>,
    /// LLM explanation of what was wrong and what was changed.
    pub explanation: String,
    /// Confidence score in [0.0, 1.0] from the backend.
    pub confidence: f32,
    /// Whether the repaired command passed the SafetyValidator.
    /// Always false when repaired_command is None.
    pub safety_passed: bool,
    /// If safety blocked the repair, the human-readable reason.
    pub repair_blocked_reason: Option<String>,
    /// Number of LLM calls made (for telemetry).
    pub attempts: u32,
}

/// Drives one round of repair: prompt → generate → validate → result.
pub struct RepairEngine<'a> {
    backend: &'a dyn crate::backends::CommandGenerator,
    validator: &'a crate::safety::SafetyValidator,
    max_attempts: u32,
}

impl<'a> RepairEngine<'a> {
    pub fn new(
        backend: &'a dyn crate::backends::CommandGenerator,
        validator: &'a crate::safety::SafetyValidator,
        max_attempts: u32,
    ) -> Self { ... }

    /// Run the repair loop. Always returns Ok(RepairResult); failures are
    /// encoded in the result fields, not as Err variants, so callers
    /// (including --json mode) always get a structured output.
    pub async fn repair(&self, req: &RepairRequest) -> RepairResult { ... }
}
```

#### `src/prompts/repair_prompt.rs` (new file within `prompts/` module)

```rust
/// Build the system + user prompt pair for a repair request.
/// Follows the same platform-rules injection as `command_templates.rs`.
pub fn build_repair_prompt(req: &RepairRequest, platform_rules: &str) -> String;
```

#### `src/models/` — extend `CommandRequest` (existing type)

Add an optional field so the repair prompt can re-use the generation pipeline:

```rust
// In models/profile.rs or wherever CommandRequest lives:
#[serde(skip_serializing_if = "Option::is_none")]
pub failure_context: Option<RepairRequest>,
```

---

### Minimal set of files that need to change

| File | Change |
|------|--------|
| `src/execution/mod.rs` | `pub mod repair; pub use repair::{RepairRequest, RepairResult, RepairEngine, STDERR_CAPTURE_BYTES};` |
| `src/execution/repair.rs` | **New file** — `RepairRequest`, `RepairResult`, `RepairEngine` |
| `src/prompts/repair_prompt.rs` | **New file** — `build_repair_prompt()` |
| `src/prompts/mod.rs` | `pub mod repair_prompt; pub use repair_prompt::build_repair_prompt;` |
| `src/main.rs` | Route `caro fix` verb; parse `--json`, `--exit-code`, `--stderr` flags |
| `Cargo.toml` | No new dependencies (tokio, serde, anyhow, thiserror already present) |

**Optional (v2.0 stretch, not blocking):**
| File | Change |
|------|--------|
| `src/caroml/interpreter.rs` | Call `RepairEngine` when a step's `ExecutionResult.success == false` and `[ai] repair.enabled = true` in config |
| `src/config/schema.rs` | Add `[ai.repair] enabled = false, max_attempts = 2` config block |

---

### Exit code / output contract

Machines and scripts depend on the following:

```
caro fix "<command>" [--exit-code <n>] [--stderr "<text>"] [--json]
```

| Condition | Exit code | Description |
|-----------|-----------|-------------|
| Repair found, safety passed | `0` | `repaired_command` is populated |
| No repair found (LLM uncertain) | `2` | `repaired_command` is null |
| Safety blocked the repair | `3` | `repair_blocked_reason` is populated |
| Backend error (LLM unavailable) | `1` | Standard error exit |

**JSON schema (`--json` output):**

```json
{
  "original_command": "df --sort=size",
  "repaired_command": "df -h | sort -k5 -hr",
  "explanation": "macOS df does not support --sort; pipe to sort instead.",
  "confidence": 0.92,
  "safety_passed": true,
  "repair_blocked_reason": null,
  "attempts": 1
}
```

All fields are always present in the JSON output (nulls for absent optionals).
This is the stable contract; adding new optional fields is backward-compatible.

---

### Integration tests (known inputs → deterministic output)

Tests live in `tests/repair_integration.rs` and use the static backend to
avoid LLM nondeterminism.

```rust
// Test 1: Safety blocks a dangerous repair
// Input: command that failed, LLM would suggest rm -rf /
// Expected: safety_passed=false, exit code 3

// Test 2: BSD/GNU flag mismatch (the dominant failure class)
// Input: command="df --sort=size", exit_code=1,
//        stderr="df: illegal option -- -"
//        platform="macos-arm64"
// Expected: repaired_command does not contain "--sort", safety_passed=true

// Test 3: Permission denied → sudo variant
// Input: command="cat /etc/sudoers", exit_code=1,
//        stderr="Permission denied"
// Expected: repaired_command contains "sudo", safety_passed depends on
//           whether sudo is in the NEEDS declaration (test both paths)

// Test 4: Unknown command → command not found
// Input: command="foobar --help", exit_code=127,
//        stderr="command not found: foobar"
// Expected: repaired_command is null OR repaired_command is a real alternative;
//           in either case safety_passed=true and exit code is 0 or 2

// Test 5: JSON output structure
// Input: any valid repair request with --json flag
// Expected: output parses as valid RepairResult; all fields present
```

---

### Explicit out-of-scope (belongs in next version)

| Feature | Why deferred |
|---------|-------------|
| Automatic execution of repair without user confirmation | Safety risk; user must always approve |
| Stdout capture in repair context | stderr is sufficient for error diagnosis; stdout adds noise and token cost |
| CaroML `.caro.lock` patching on repair success | Requires `caro run` integration; separate PR scope |
| Repair history and learning from past repairs | Requires telemetry store integration; v2.1 |
| Multi-step repair chains (repair the repair) | Complexity/risk; `max_attempts=2` is sufficient for v1 |
| Static repair rules lookup (pre-LLM fast path) | Optimization; measure miss rate first |
| `--watch` mode: auto-trigger repair on every failed command in the shell | Requires shell hooks; v2.1 |

---

## Implementation checklist for the executing agent

- [ ] Create feature branch: `bin/sk-new-feature "self-healing command repair"`
- [ ] Add `RepairRequest` + `RepairResult` + `STDERR_CAPTURE_BYTES` to `src/execution/repair.rs`
- [ ] Add `build_repair_prompt()` to `src/prompts/repair_prompt.rs`
- [ ] Implement `RepairEngine::repair()` — calls backend, runs validator, returns `RepairResult`
- [ ] Wire `caro fix` verb in `src/main.rs` with `--json`, `--exit-code`, `--stderr` flags
- [ ] Write integration tests in `tests/repair_integration.rs` covering all 5 scenarios above
- [ ] Verify `cargo test safety` still passes (zero regressions)
- [ ] Verify `cargo clippy -- -D warnings` passes
- [ ] Write ADR-009 to `docs/adr/ADR-009-self-healing-repair.md` (use this document as source)
- [ ] Open PR, update CHANGELOG.md with `## [Unreleased]` entry

---

*Autonomous research run — no user input available. Feature selected based on
ROADMAP.md v2.0.0 "Self-Healing Features" (#155) starting May 1, 2026.*
