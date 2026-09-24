# ADR-021 — Execution Attribution: Tagging Caro-Generated Commands in Shell History

**Status**: Proposed

**Date**: 2026-06-10

**Authors**: `caro-research--scoping-process` (automated research/scoping agent)

**Target**: All users (power users + agent-pipeline integrators)

**Hypothesis ID**: Extension of `caro-core` (Gate 1 exempt — extends already-validated
core loop)

**Relates to**: ADR-017 (local context indexing), ADR-020 (tiered approval protocol),
`src/ai/session.rs`, `src/execution/executor.rs`, `src/knowledge/schema.rs`

---

## Context

### The Gap: Caro Is Invisible in Shell History

When Caro generates and executes a command such as `tar -czf dist.tar.gz dist/`,
that command appears in shell history with no indication it was AI-generated.
Three consequences follow:

1. **No feedback loop.** Caro cannot learn from execution outcomes after the
   session ends. A command that exits with code 127 ("command not found") was
   wrong — but Caro's knowledge index never finds out unless the user explicitly
   corrects it in the same session.

2. **No agent-ecosystem citizenship.** Tools like Atuin (v18.13, March 2026)
   expose an `$all-agent` history filter that shows commands run by AI agents
   (Claude Code, Codex, pi). Caro-generated commands don't appear there —
   they're indistinguishable from user-typed ones.

3. **`Turn` is incomplete.** `src/ai/session.rs::Turn` records the generated
   command and the safety risk level, but not whether the command was actually
   executed, nor the exit code or wall-clock duration. Correction learning in
   `AgentLoop::record_correction` runs in the same async context, but only for
   in-session refinements — post-execution failures are invisible.

### Competitive Landscape: How Atuin Solves Adjacent Problems

**Atuin AI Agent Hooks** (v18.13, May 2026 docs) solves the agent-citizenship
problem for Claude Code and Codex by writing hooks into their settings files
(`~/.claude/settings.json`, `~/.codex/hooks.json`). When the agent runs a Bash
command, the hook calls `atuin history start` before and `atuin history end`
after, tagging the entry with `author: claude-code`.

**Why Caro cannot use Atuin's approach as-is:**

- Atuin's hook model requires the *agent* to have a hook system (Claude Code's
  `PreToolUse` / `PostToolUse`, Codex's `hooks.json`). Caro is a simple CLI
  process; it has no hook surface that external tools can register against.
- Caro is the *generator and executor* — it doesn't need an external hook
  because it already controls the moment of execution.
- Atuin's hooks are one-directional: they capture history but don't feed
  execution outcomes back to the AI. Caro can close this loop natively.

**Atuin's failure modes to avoid:**

1. **Hook fragility**: hooks written to `~/.claude/settings.json` can be
   clobbered by Claude Code restarts. Caro writes attribution at the source
   (the executor), not via a fragile config file.

2. **No outcome feedback**: Atuin records start/end times and exit codes in
   history, but this data never reaches the AI session that generated the
   command. Caro's `AiSession` + `SessionStore` can capture this natively.

3. **Cloud-only history AI**: Atuin's `enable_history_search` requires a
   cloud sync. Caro's attribution stays local-first and offline.

### Existing Infrastructure

| File | What it does | Gap |
|---|---|---|
| `src/execution/executor.rs` | `ExecutionResult { exit_code, stdout, stderr, execution_time_ms, success }` | Never written to session or knowledge index post-run |
| `src/ai/session.rs` | `Turn { role, content, command, confidence, risk, ts }` | No `exit_code`, no `duration_ms`, no `executed_at` |
| `src/knowledge/schema.rs` | `EntryType::Success` / `Correction`; no `author` field | Can't distinguish user-typed from Caro-generated |
| `src/models/` `AiCapabilities` | `enable_history_search` toggle | No `write_atuin_history` toggle |
| `src/ai/runner.rs` | `AiOutcome` returned from `run_once` | No path to write Atuin history |

---

## Decision

Add execution attribution in three layers — all additive, no breaking changes to
existing types:

### Layer 1: Enrich `Turn` with execution outcome

Add three optional fields to `src/ai/session.rs::Turn` that are populated only
when the turn was actually executed (not just generated):

```rust
/// Exit code from execution (None = not yet executed / dry-run).
#[serde(default, skip_serializing_if = "Option::is_none")]
pub exit_code: Option<i32>,

/// Wall-clock execution duration in milliseconds.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub duration_ms: Option<u64>,

/// When the command was sent to the shell (distinct from generation ts).
#[serde(default, skip_serializing_if = "Option::is_none")]
pub executed_at: Option<DateTime<Utc>>,
```

`SessionStore::update_last_turn` gains a method that patches these fields on
the most recent assistant turn after execution completes. The store is already
append-friendly (flat JSON); this adds an in-place patch before the final
flush.

### Layer 2: Add `author` to `KnowledgeEntry`

Add one optional field to `src/knowledge/schema.rs`:

```rust
Field::new("author", DataType::Utf8, true)
```

And to `KnowledgeEntry`:

```rust
/// Who generated this command: "caro", "user", or an agent name.
pub author: Option<String>,
```

`AgentLoop::record_success` and `record_correction` pass `author: Some("caro")`
when called from an AI-generated path. Direct user corrections (future) would
pass `"user"`. This is purely additive — existing entries read back as `None`
(unknown author) without a migration.

Schema version bumps from 1 → 2 in `migration.rs`; the migrator adds the
nullable column and back-fills `None`.

### Layer 3: `AtuinHistoryBridge`

A new zero-dep struct in `src/ai/atuin_bridge.rs`:

```rust
/// Writes Caro-generated command history to Atuin if it is installed.
///
/// Design: pure subprocess calls (`atuin history start/end`). No daemon.
/// No Atuin SDK dep. Degrades gracefully when Atuin is not installed.
pub struct AtuinHistoryBridge {
    enabled: bool,
}

impl AtuinHistoryBridge {
    /// Probe PATH for `atuin` once at session start.
    pub fn probe() -> Self { ... }

    /// Call `atuin history start --command <cmd> --exit 0` before execution.
    /// Returns an opaque token used to close the record.
    pub fn record_start(&self, command: &str, cwd: &Path) -> Option<AtuinToken> { ... }

    /// Call `atuin history end <token> --exit <code> --duration <ms>`.
    pub fn record_end(&self, token: AtuinToken, exit_code: i32, duration_ms: u64) { ... }
}
```

`AtuinToken` is a newtype around the `u64` ID that `atuin history start`
prints to stdout (Atuin v18's stdout contract).

`AtuinHistoryBridge` is gated by a new config toggle:

```toml
[ai.attribution]
write_atuin_history = true   # default: true if `atuin` found in PATH, else false
author_tag = "caro"          # written as --author to atuin history start
```

The bridge is instantiated once in `run_once` and lives for the duration of the
invocation (stateless subprocess model — no daemon).

---

## New Types

### `src/ai/atuin_bridge.rs` (new file, ~80 LOC)

```rust
pub struct AtuinHistoryBridge { enabled: bool }
pub struct AtuinToken(u64);

impl AtuinHistoryBridge {
    pub fn probe() -> Self
    pub fn record_start(&self, command: &str, cwd: &Path) -> Option<AtuinToken>
    pub fn record_end(&self, token: AtuinToken, exit_code: i32, duration_ms: u64)
    pub fn is_enabled(&self) -> bool
}
```

All methods are synchronous and infallible: errors are logged at `DEBUG` level
and silently swallowed, never propagating to the caller. This matches Atuin's
own hook design (hooks must not block the agent).

### Additions to existing types

`src/ai/session.rs::Turn` — three new `Option` fields (see Layer 1).

`src/knowledge/schema.rs::knowledge_schema()` — one new nullable `author` column.

`src/knowledge/index.rs::KnowledgeEntry` — one new `author: Option<String>` field.

`src/models/mod.rs::AiConfig` — `attribution: AttributionConfig` sub-struct.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AttributionConfig {
    /// Write caro-generated commands to Atuin history if atuin is in PATH.
    pub write_atuin_history: bool,
    /// Author tag written to Atuin and the knowledge index.
    pub author_tag: String,
}

impl Default for AttributionConfig {
    fn default() -> Self {
        Self {
            write_atuin_history: true,  // auto-detected per probe()
            author_tag: "caro".into(),
        }
    }
}
```

---

## Files That Need to Change

Minimal set — no new modules, only new files inside existing modules:

| File | Change |
|---|---|
| `src/ai/session.rs` | +3 optional fields on `Turn`; +`update_with_outcome()` constructor |
| `src/ai/store.rs` | +`patch_last_turn_outcome(exit_code, duration_ms)` method |
| `src/ai/runner.rs` | Instantiate `AtuinHistoryBridge`; call `record_start` before execute; call `record_end` after; patch turn |
| `src/ai/mod.rs` | `pub mod atuin_bridge;` |
| `src/ai/atuin_bridge.rs` | **New file** (~80 LOC) |
| `src/knowledge/schema.rs` | Add `author` column to schema; bump `SCHEMA_VERSION` |
| `src/knowledge/index.rs` | `author: Option<String>` on `KnowledgeEntry`; pass through in `record_success` / `record_correction` |
| `src/agent/mod.rs` | Pass `Some("caro")` as author to `record_success` / `record_correction` |
| `src/models/mod.rs` | `AttributionConfig` struct; add to `AiConfig` |
| `src/knowledge/migration.rs` | Version 1→2 migration: add nullable `author` column, back-fill `NULL` |

---

## Exit Code / Output Contract

Attribution is transparent to the caller. Caro's existing exit codes are unchanged:

| Exit code | Meaning |
|---|---|
| 0 | Command generated (or executed) successfully |
| 1 | Error during generation or validation |
| 201 | Edit-mode: command pushed to clipboard / shell variable |

The Atuin bridge failure (atuin not found, atuin history start fails) never
produces a non-zero exit from Caro. Attribution is best-effort.

`--dry-run --json` output gains one new field on the `result` object:

```json
{
  "command": "tar -czf dist.tar.gz dist/",
  "attribution": {
    "author": "caro",
    "atuin_enabled": false,
    "atuin_token": null
  }
}
```

This is additive and does not break existing JSON consumers.

---

## Integration Tests

### Test 1 — `Turn` outcome patch round-trip

```
Input: AiSession with one assistant Turn, no exit_code
Action: store.patch_last_turn_outcome(0, 142)
Assert: deserialized Turn.exit_code == Some(0), Turn.duration_ms == Some(142)
Assert: other Turn fields unchanged
```

### Test 2 — `KnowledgeEntry` author round-trip

```
Input: record_success("list files", "ls -la", author=Some("caro"))
Assert: find_similar("list files") returns entry with author == Some("caro")
```

### Test 3 — `AtuinHistoryBridge::probe` when atuin not in PATH

```
Env: PATH does not contain atuin
Assert: bridge.is_enabled() == false
Assert: record_start(...) returns None (no-op)
Assert: record_end(...) is a no-op
```

### Test 4 — `AtuinHistoryBridge` subprocess mock

```
Env: CARO_TEST_ATUIN_BIN=path/to/mock-atuin (mock prints "42" then exits 0)
Action: token = bridge.record_start("ls -la", Path::new("/tmp"))
Assert: token == Some(AtuinToken(42))
Action: bridge.record_end(token.unwrap(), 0, 55)
Assert: mock-atuin received correct args
```

### Test 5 — Migration v1 → v2

```
Input: LanceDB table with no `author` column (v1 fixture)
Action: migration::run()
Assert: table now has nullable `author` column
Assert: all existing rows have author == None
Assert: new rows can be written with author == Some("caro")
```

---

## Explicit Out-of-Scope

The following belong in subsequent versions:

1. **Reading Atuin history as context** — querying `atuin search --author '$all-agent'`
   to seed the knowledge index belongs in ADR-017 (local context indexing). This ADR
   only *writes* to Atuin; it does not read from it.

2. **`caro hook install`** — a user-facing CLI subcommand that writes Caro as a hook
   handler into Claude Code's `settings.json` (mirroring `atuin hook install claude-code`)
   is a future ergonomics feature. Attribution via the bridge works without it.

3. **Author filter in `caro ai` history search** — surfacing `--author caro` vs
   `--author user` in `caro ai` interactive search belongs in the conversational UI
   iteration after v1.5.

4. **Stdout/stderr capture in `Turn`** — recording command output in the session store
   raises storage and privacy concerns. Scoped for a separate ADR with explicit
   opt-in (`[ai.attribution] capture_output = false`).

5. **Windows `atuin history` path handling** — Atuin on Windows uses different path
   conventions. Defer to after `ExecutionContext::os == "windows"` CI is green.

---

## Consequences

**Positive:**
- Caro-generated commands become first-class citizens in Atuin's `$all-agent`
  history ecosystem, providing `atuin search --author caro` out of the box.
- Post-execution outcomes close the feedback loop: a command that fails with
  exit 127 can trigger a follow-up correction prompt in a future session.
- Knowledge index entries gain authorship provenance, enabling analytics (how
  often do user corrections override Caro suggestions?).

**Negative / Risks:**
- The Atuin bridge adds a subprocess call per execution (≤5ms when atuin is not
  found via PATH probe; ≤50ms when calling `atuin history start`). This is
  acceptable for an interactive CLI; log a `WARN` if it exceeds 200ms.
- Schema version bump requires all users to run the migration on first upgrade.
  Migration is tested in Test 5 and is backward-compatible (nullable column,
  no existing data deleted).
- `session_continue_minutes` (default 60 in Atuin; 60 in Caro's `store.rs`)
  means a long-paused session resumes stale context. Unrelated to attribution
  but worth noting as a known issue.

---

## Alternatives Considered

**A — External hook model (Atuin-style):** register Caro as a hook in the user's
shell (`.zshrc`/`.bashrc`). Rejected: requires shell reload, fragile across
shell upgrades, and doesn't help with the knowledge index feedback loop.

**B — Telemetry events only:** expand `EventType::CommandGeneration` to include
execution outcome. Rejected: telemetry is one-way and opt-out; it doesn't update
the knowledge index or Atuin history.

**C — Single unified `AiHistoryEntry` type:** replace `Turn` with a new type that
collapses generation + execution into one record. Rejected: `Turn` is part of the
stable session serialization format; replacing it would break all existing
`ai_sessions.json` files. Additive fields preserve backward compatibility.

---

## See Also

- [Atuin AI Agent Hooks docs](https://docs.atuin.sh/cli/guide/agent-hooks/) —
  the competitive feature this ADR responds to
- [Atuin v18.13 release blog](https://blog.atuin.sh/atuin-v18-13/) — context for
  the `$all-agent` history filter
- `ADR-017-local-context-indexing.md` — reading Atuin history as context (the
  other direction of the bridge)
- `ADR-020-tiered-approval-protocol.md` — `HumanGate` wiring; attribution interacts
  with approval decisions (approved commands should record author at approval time)
- `src/ai/runner.rs` — the integration point for Layer 3
- `src/knowledge/migration.rs` — schema migration infrastructure this ADR extends
