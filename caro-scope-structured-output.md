# Implementation Scope — Structured Output Envelope & Stateless Session Contract

**Feature under analysis:** Claude Code *headless mode* (the Agent SDK CLI: `claude -p`
with `--output-format json|stream-json`, `--continue`/`--resume`, `--bare`).
**Equivalent we are scoping for Caro:** a versioned, machine-readable result
envelope (`caro -o json`) plus a documented exit-code contract and stateless session
reuse.

**Date:** 2026-06-25 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/003-structured-output-session-contract.md`

> The task template left `[FEATURE NAME]` unfilled and ran unattended. I selected
> Claude Code headless mode because it maps cleanly onto every constraint in the
> task (structured output contract, exit codes, session/context lifecycle, pure
> subprocess) and because Caro already has the scaffolding (`-o/--output`,
> `ai::SessionStore`, `AiOutcome`) that makes this a high-leverage, low-risk scope.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom
Headless mode lets non-interactive callers — CI pipelines, build scripts, GitHub
Actions, other programs — drive Claude Code as a subprocess and consume its result
programmatically instead of reading a TTY. `--output-format json` returns a
structured document with the text result, a `session_id`, and metadata; `--resume
<id>`/`--continue` chain turns; `--bare` strips ambient context for reproducibility.
The audience is **automation authors**, not interactive users.

### Core architecture (data flow, key types, separation of concerns)
- `claude -p "<prompt>"` is the entry point; it is the **Agent SDK** exposed as a
  CLI (the same agent loop and context manager as the interactive product).
- Output is selected by `--output-format`:
  - `text` (default) — plain prose.
  - `json` — one object: `{ result, session_id, total_cost_usd, per-model cost, … }`;
    with `--json-schema`, constrained output lands in a `structured_output` field.
  - `stream-json` — newline-delimited JSON (NDJSON), one event object per line.
- Stream events are typed: `system/init` (session metadata; first event),
  `system/api_retry`, `system/plugin_install`, `stream_event` (token deltas),
  and a terminal `result`. Each event carries `uuid` and `session_id`.
- Session continuity: `session_id` is returned in the JSON result; the caller
  scrapes it (`jq -r '.session_id'`) and passes it to `--resume`. Resume lookup is
  **scoped to the current project directory + git worktrees**.

### Why it is experimental / limited — the failure modes
1. **Version-coupled schema, no `schema_version`.** The docs are full of
   `min-version:` and "as of v2.1.x" qualifiers. A script written against one
   release can break on the next because the event/result shape shifted.
2. **Redundant initialization by default.** Plain `claude -p` re-discovers hooks,
   skills, plugins, MCP servers, auto-memory, and `CLAUDE.md` **on every
   invocation**. `--bare` was added specifically to skip this for CI determinism and
   startup speed, and is slated to *become the default* — an admission that the
   default per-call init is both slow and non-deterministic across machines.
3. **`session_id` is scraped, not handed back as a stable handle**, so multi-turn
   automation is string-parsing glue.
4. **Ambiguous failure signaling.** Without a typed status + distinct exit codes,
   callers must parse text to tell *why* a run failed (bad args vs auth vs blocked).
5. **Lifecycle foot-guns shipped then patched.** Piped stdin was unbounded until a
   10 MB cap landed in v2.1.128; background Bash tasks could hold the `-p` process
   open indefinitely until a 5-second grace + 10-minute ceiling were added
   (v2.1.163 / v2.1.182). These are state-lifecycle bugs that a stateless,
   single-shot design never has.

### Structured output contract (payloads, exit codes, events)
- **JSON result:** `result` (text), `session_id`, `total_cost_usd`, per-model cost;
  `structured_output` when `--json-schema` is used.
- **NDJSON events:** `system/init`, `system/api_retry`, `system/plugin_install`,
  `stream_event`, `result` — each with `uuid`, `session_id`.
- **Exit codes:** Unix convention (0 success, non-zero failure); the docs do **not**
  publish a stable, disambiguated table separating usage/auth/blocked/backend
  errors — the gap we close.

### Session / context lifecycle (avoiding redundant init)
- Interactive sessions keep the model + context warm; headless `-p` does not — each
  call re-initializes. `--bare` reduces *what* is initialized (skips discovery) but
  does not keep weights warm between subprocesses.
- Continuity is **persisted on disk per project** and re-loaded on `--resume`; there
  is no daemon keeping conversation state in memory across `-p` calls.

---

## Phase 2 — Competitive Differentiation

### What they get right (we should replicate)
- **Three explicit output formats** with JSON carrying metadata, not just the result.
- **`session_id` returned in the structured payload** so turns can be chained.
- **A "bare"/deterministic posture for CI** — reproducible output independent of
  ambient machine config.
- **Typed event metadata** (`system/init` reporting model/tools) — good for
  surfacing what actually ran.

### Their gaps we avoid by designing the schema first
- **Add `schema_version` from day one.** Decouple the contract from the binary
  version; bumping it becomes a deliberate, documented event.
- **Model `session_id`/`resumed` as first-class typed fields**, never scraped.
- **Publish a disambiguated exit-code table** (blocked ≠ backend-unavailable ≠ auth
  ≠ usage), each mirrored by `error.kind` in the envelope.
- **Be stateless by construction** so the stdin-cap and background-task lifecycle
  bugs are structurally impossible.

### Our unique positioning (what we can do that they cannot)
- **Offline / local-first.** Caro runs a local model with no API key, no network,
  no per-call `$` cost — so the contract needs no billing fields and works in
  air-gapped CI.
- **Pure subprocess, daemonless, no ambient state.** "Bare" is our *default*, not a
  flag — identical input ⇒ identical envelope across machines.
- **Safety is a first-class contract field.** Every result carries a `risk` level
  and `allowed` verdict from the 52-pattern validator; a *blocked* command is a
  distinct, machine-detectable outcome (exit 3) rather than just text.
- **Universal / standalone.** No SDK, plugin marketplace, or login flow to load;
  the envelope is consumable by `jq` in two lines.

### Existing Caro infrastructure that already covers part of this
| Need | Already present |
|---|---|
| Output-format flag + enum | `cli::OutputFormat::{Json,Yaml,Plain}`, parsed from `-o/--output` |
| Serializable risk taxonomy | `models::RiskLevel` (serde) |
| Generation result | `models::GeneratedCommand` (serde) |
| One-shot AI turn + safety + persist | `ai::runner::run_once` → `AiOutcome` |
| Session lifecycle / resume / TTL | `ai::store::SessionStore` (`resume_recent`, `upsert`, atomic flush) + `ai::session::{AiSession,Turn}` (serde) |
| Safety verdicts | `safety::SafetyValidator` (`validate_command`) |
| Backend selection | `cli::CliApp::with_overrides` / `backend_arc` |
| Exit-code precedent | `EXIT_CODE_EDIT = 201` in `main.rs` |
| JSON formatting precedent | `assessment/formatters/json.rs` (`serde_json::to_string_pretty`) |

**Gaps to close:** `AiOutcome` is not `Serialize`; there is no result envelope, no
`schema_version`, and no exit-code table beyond 201.

---

## Phase 3 — Scope Definition

### New types (added to `src/models/mod.rs`, the existing serde-types home)

All derive `Debug, Clone, PartialEq, Serialize, Deserialize` — serializable from
day one (constraint satisfied).

```rust
/// Stable, versioned machine-readable result emitted on stdout by `-o json`.
pub struct CommandEnvelope {
    /// Contract version. Bumped only on breaking changes. v1 ships as 1.
    pub schema_version: u32,
    pub status: EnvelopeStatus,
    /// Generated command; None only when we errored before generating.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    pub explanation: String,
    pub risk: RiskLevel,          // reuse existing enum
    pub confidence: f64,
    pub allowed: bool,
    pub warnings: Vec<String>,
    /// First-class session handle (ai path only). Never scraped from prose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumed: Option<bool>,
    pub backend: String,
    pub shell: ShellType,         // reuse existing enum
    /// Mirrors the process exit code this run terminates with.
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EnvelopeError>,
    pub timings: Timings,
}

#[serde(rename_all = "snake_case")]
pub enum EnvelopeStatus { Ok, Blocked, Error }

pub struct EnvelopeError {
    /// Matches the exit-code table: usage|backend_unavailable|config|auth|internal.
    pub kind: String,
    pub message: String,
}

pub struct Timings {
    pub total_ms: u64,
    pub model_init_ms: u64,   // cold-start cost, surfaced not hidden
    pub generate_ms: u64,
    pub validate_ms: u64,
    pub model_warm: bool,     // true once a warm-cache lands (always false in v1)
}
```

**Method contracts**
- `CommandEnvelope::ok(...) -> Self` — status `Ok`, `exit_code = 0`.
- `CommandEnvelope::blocked(risk, warnings, …) -> Self` — status `Blocked`,
  `allowed = false`, `exit_code = 3`.
- `CommandEnvelope::error(kind, message, exit_code) -> Self` — status `Error`.
- `CommandEnvelope::to_json(&self) -> serde_json::Result<String>` — pretty JSON,
  mirroring `assessment/formatters/json.rs`.
- `AiOutcome::into_envelope(self, backend, shell, timings) -> CommandEnvelope`
  (added in `ai/runner.rs`; `AiOutcome` also gains `Serialize`).

### Exit-code / output contract (what machines depend on)

| Code | `status` / `error.kind` | Trigger |
|---|---|---|
| 0 | `ok` | command generated and allowed |
| 1 | `error` / `internal` | unexpected internal failure |
| 2 | `error` / `usage` | no/empty prompt, invalid flag combination |
| 3 | `blocked` | safety validator rejected the command |
| 4 | `error` / `backend_unavailable` | model missing or remote backend unreachable |
| 5 | `error` / `config` | invalid/unreadable configuration |
| 6 | `error` / `auth` | remote backend credential/auth failure |
| 201 | (edit mode) | existing `EXIT_CODE_EDIT`, unchanged |

Guarantee: on `-o json`, exactly one `CommandEnvelope` is written to stdout, the
process exits with `envelope.exit_code`, and human chatter stays on stderr. This
directly fixes the Phase-1 failure mode (Claude Code's unversioned schema +
arg/auth conflation) **by design**: the contract is versioned and the failure
classes are distinct codes, not parsed text.

### Minimal set of files to change (no new src modules)

1. **`src/models/mod.rs`** — add the four types above + constructors/`to_json`.
2. **`src/ai/runner.rs`** — `#[derive(Serialize)]` on `AiOutcome`; add
   `into_envelope`; capture `model_init_ms`/`generate_ms`/`validate_ms` via
   `Instant` around the existing backend + validator calls.
3. **`src/main.rs`** — add exit-code constants beside `EXIT_CODE_EDIT`; in
   `run_ai_once` and the primary `generate` path, when `output_format == Json`
   build the envelope, print `envelope.to_json()` to stdout, and
   `process::exit(envelope.exit_code)`. Map `GeneratorError` /
   `anyhow` failures to the `kind` + code table.
4. **`src/cli/mod.rs`** — small `emit_envelope(...)` helper so both call sites share
   one serialization path; `OutputFormat` already exists, no enum change.
5. **`tests/json_contract.rs`** — new integration test crate file (a `tests/` file
   is a separate test binary, not a `src` module, so this respects the
   "no new modules" constraint).

### Integration tests — known inputs → deterministic JSON + exit code

Use the deterministic `mock`/static backend so output is reproducible.

| # | Invocation | Asserted JSON | Exit |
|---|---|---|---|
| 1 | `caro -o json -b mock "list files"` | `schema_version==1`, `status=="ok"`, `command` non-empty, `allowed==true` | 0 |
| 2 | `caro -o json -b mock "delete everything" ` (mock returns `rm -rf /`) | `status=="blocked"`, `allowed==false`, `risk` ∈ {High,Critical} | 3 |
| 3 | `printf '' \| caro -o json` (empty prompt) | `status=="error"`, `error.kind=="usage"` | 2 |
| 4 | `caro -o json -b ollama --endpoint http://127.0.0.1:1` | `status=="error"`, `error.kind=="backend_unavailable"` | 4 |
| 5 | `caro ai --once -o json -b mock "list files"` ×2 | 2nd run `resumed==true`, same `session_id` | 0 |
| 6 | serde round-trip / snapshot of a fixed `CommandEnvelope` | byte-stable field order, `schema_version==1` present | n/a |

Test 6 locks the wire contract: any accidental field rename/reorder fails CI,
forcing a deliberate `schema_version` bump.

### Explicit out-of-scope (next version)

- **`stream-json` / NDJSON event stream.** Generation is single-shot today; defer
  until a streaming consumer exists. The versioned envelope is forward-compatible.
- **Persistent warm model cache (mmap) or `caro serve` daemon.** The real fix for
  cold-start; deferred to its own ADR. v1 only *surfaces* the cost via `timings`.
- **`--json-schema` constrained structured output.**
- **YAML envelope body** (enum value exists; v1 emits the envelope for JSON only).
- **Cost/usage/token fields** (local model ⇒ no `$`; add `tokens` later if useful).
- **`--resume <specific id>` targeting.** Today resume = most-recent-within-TTL via
  `SessionStore::resume_recent`; addressing an arbitrary id is a follow-up.

### Constraint compliance check
- *Reuse validator/safety/config:* envelope wraps `SafetyValidator`, `CliApp`,
  `SessionStore`, `RiskLevel` — no duplication. ✅
- *All new types serializable from day one:* yes, full serde derives. ✅
- *Pure subprocess, no daemon/state:* one process in, one envelope out, exit; bare
  by default. ✅
- *Solve the Phase-1 failure mode by design:* `schema_version` + distinct exit codes
  + typed `session_id` replace version-coupled schemas, conflated codes, and scraped
  ids — structurally, not as a workaround. ✅

---

## Suggested landing sequence (per repo rules)
1. Feature branch via `bin/sk-new-feature "structured json output contract"`
   (`.claude/rules/git-workflow.md` — never commit to `main`).
2. Land types + tests behind `-o json` (additive; `Plain` stays default).
3. Move this ADR to `docs/adr/` and update `docs/adr/README.md`; renumber if a
   `003-` already merged (`.claude/rules/adr-numbering.md`).
4. This is an output-format/contract feature, not a new product line, so the
   `validation-discipline.md` 20-transcript gate does not apply; standard
   `dev-process.md` CI (`cargo test`, `cargo clippy -- -D warnings`) governs.

## Sources
- [Run Claude Code programmatically — Claude Code Docs](https://code.claude.com/docs/en/headless)
- [Headless Mode and CI/CD — Command Reference, SFEIR Institute](https://institute.sfeir.com/en/claude-code/claude-code-headless-mode-and-ci-cd/command-reference/)
- Caro codebase: `src/main.rs`, `src/cli/mod.rs`, `src/models/mod.rs`,
  `src/ai/{runner,session,store}.rs`, `src/backends/mod.rs`, `src/model_loader.rs`,
  `src/assessment/formatters/json.rs`.
