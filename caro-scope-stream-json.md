# Implementation Scope — Headless `stream-json` NDJSON Event Contract

**Feature under analysis:** Claude Code *headless mode* streaming output
(`claude -p --output-format stream-json`, the NDJSON event stream: `system/init`,
`stream_event`, `system/api_retry`, terminal `result`).
**Equivalent we are scoping for Caro:** `caro -o stream-json` — a versioned NDJSON
event stream (`init` → `progress`* → `result`) whose terminal event reuses the
ADR 003 `CommandEnvelope` and whose exit codes are identical to ADR 003.

**Date:** 2026-07-01 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/004-headless-stream-json-event-contract.md`
**Builds on:** `docs/adr/003-structured-output-session-contract.md` +
`caro-scope-structured-output.md`

> The task template left `[FEATURE NAME]` unfilled and ran unattended. I selected
> Claude Code `stream-json` because it is precisely the streaming layer the prior
> run (ADR 003) named as its deferred next step, it reuses the `CommandEnvelope`
> ADR 003 already defines, and — with `caro fix` (ADR 016) and the agentic
> pipeline (`src/agent/`) now in the tree — there is finally a streaming consumer,
> which was the exact condition ADR 003 set for un-deferring it.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom
`--output-format stream-json` lets a non-interactive caller watch a headless run
unfold in real time instead of blocking until exit. A wrapper, TUI, CI log, or
orchestrator consumes one JSON event per line (NDJSON) as work happens: session
metadata up front, incremental output in the middle, and a single machine-readable
summary at the end. The audience is **automation authors driving long or
multi-step runs** who need progress and structure, not a frozen pipe.

### Core architecture (data flow, key types, separation of concerns)
- `claude -p "<prompt>" --output-format stream-json` runs the same agent loop as
  the interactive product but writes NDJSON to stdout.
- Each line is one typed event. The documented families:
  - `system` with `subtype: init` — first event; reports `model`, `tools`, MCP
    servers, loaded plugins, `session_id`. (Unless `CLAUDE_CODE_SYNC_PLUGIN_INSTALL`
    is set, in which case `plugin_install` events precede it — already an ordering
    caveat.)
  - `system` with `subtype: api_retry` — emitted before a retryable API error is
    retried.
  - `stream_event` — token/tool deltas, whose sub-subtypes mirror the Anthropic
    Messages streaming protocol (`message_start`, `content_block_start`,
    `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`).
  - `result` — terminal summary: `type`, `subtype`, `is_error`, `duration_ms`,
    `duration_api_ms`, `num_turns`, `total_cost_usd`, `result` (text), `session_id`.
- Separation of concerns is *thin*: the events are a serialization of the agent
  loop's internal messages, so the wire shape tracks internal refactors.

### Why it is experimental / limited — the failure modes
1. **Event types are under-specified and version-coupled.** The community has open
   issues asking Anthropic to simply *document the emitted message types*
   ([#24596](https://github.com/anthropics/claude-code/issues/24596),
   [#24612](https://github.com/anthropics/claude-code/issues/24612)) and to
   document `--input-format stream-json` ([#24594](https://github.com/anthropics/claude-code/issues/24594)).
   No `schema_version` on any event, so a parser written against one release can
   break on the next.
2. **No explicit ordering or sequence guarantee.** `system/init` is "first…
   unless a plugin-install env var is set." There is no monotonic sequence number
   and no contractual guarantee that exactly one terminal `result` is emitted, so
   a consumer cannot detect a truncated stream (process killed mid-run) versus a
   clean end without heuristics.
3. **Lifecycle foot-guns from the headless process model.** Piped stdin was
   unbounded until a 10 MB cap (v2.1.128); background Bash tasks could hold the
   `-p` process — and therefore the stream — open indefinitely until a grace
   period + ceiling were added (v2.1.163 / v2.1.182). A stream that can hang or
   stay half-open is a contract hazard.
4. **Human/machine interleaving risk.** Because the stream is the agent's own
   message log, non-event chatter and partial writes can interleave if flushing
   isn't disciplined.
5. **Billing-shaped fields bake in a cloud assumption.** `total_cost_usd` and
   `duration_api_ms` are meaningless for a local model and would be dead fields
   in an offline contract.

### Structured output contract (payloads, exit codes, events)
- **Events:** `system/init`, `system/api_retry`, `system/plugin_install`,
  `stream_event` (+ Messages-protocol sub-subtypes), terminal `result`.
- **Exit codes:** Unix convention (0 success / non-zero failure). No published,
  disambiguated table separating usage / auth / blocked / backend errors — the
  same gap ADR 003 already closed for the single-envelope case.

### Session / context lifecycle (avoiding redundant init)
- Each `-p` call re-initializes (re-discovers hooks/skills/plugins/MCP/`CLAUDE.md`)
  unless `--bare` is used; `--bare` is slated to become the default — an admission
  the default path pays a redundant per-call init cost and is non-deterministic
  across machines. The stream surfaces this only implicitly via `duration_ms`.
- Continuity is persisted on disk per project and reloaded on `--resume`; there is
  no daemon holding conversation state across `-p` calls.

---

## Phase 2 — Competitive Differentiation

### What they get right (we should replicate)
- **NDJSON, one event per line** — trivially consumable with `jq -c`, streamable,
  greppable.
- **A first `init` event carrying session metadata** before any work — lets a
  wrapper render context immediately.
- **A single terminal summary event** distinct from the incremental stream.
- **Typed event families** rather than free-form log lines.

### Their gaps we avoid by designing the schema first
- **Put `schema_version` on every event**, matching the envelope's, so the whole
  contract versions atomically.
- **Add a monotonic `seq` and a hard ordering guarantee**: exactly one `init`
  first, exactly one `result` last, `progress` only in between. A consumer detects
  truncation by "last line wasn't a `result`."
- **Make the terminal event *be* the ADR 003 `CommandEnvelope`**, so `-o json` and
  the last line of `-o stream-json` never diverge.
- **Be stateless and flush per line** so hang/interleave lifecycle bugs are
  structurally impossible.
- **No billing fields** — reuse `Timings` (already local-first) instead of
  `total_cost_usd`.

### Our unique positioning (what we can do that they cannot)
- **Offline / local-first.** No API key, no network, no `$` cost — the event
  contract has zero billing fields and works air-gapped.
- **Pure subprocess, daemonless.** NDJSON over stdout, one process, exit with
  `envelope.exit_code`. "Bare" is our default, so replaying identical input yields
  an identical event stream (modulo `seq`/timings) across machines.
- **Safety is a streamed, first-class outcome.** A *blocked* command is a
  guaranteed terminal `Result` event carrying `allowed=false` + `risk` and exit
  code 3 — a machine-detectable event, not prose in a log.
- **Universal / standalone.** No SDK or plugin marketplace to load; two lines of
  `jq` consume the stream.

### Existing Caro infrastructure that already covers part of this
| Need | Already present |
|---|---|
| Terminal result payload | `models::CommandEnvelope` + `EnvelopeStatus/Error/Timings` (ADR 003) |
| Output-format flag + enum | `cli::OutputFormat::{Json,Yaml,Plain}` (add `StreamJson`) |
| Serializable risk / shell / command | `models::{RiskLevel, ShellType, GeneratedCommand}` (serde) |
| One-shot AI turn + safety + persist | `ai::runner::run_once` → `AiOutcome` (+ `into_envelope`, ADR 003) |
| Session lifecycle / resume / TTL | `ai::store::SessionStore`, `ai::session::{AiSession,Turn}` |
| Safety verdicts | `safety::SafetyValidator::validate_command` |
| Backend / shell selection | `cli::CliApp::with_overrides`, `backend_arc` |
| Exit-code table | ADR 003 constants beside `EXIT_CODE_EDIT` in `main.rs` |
| JSON serialization precedent | `assessment/formatters/json.rs` |

**Gaps to close:** no `StreamJson` output variant, no `StreamEvent` type, no
event-emit path threading through the one-shot / `ai --once` code, no per-line
flush discipline.

---

## Phase 3 — Scope Definition

### New types (added to `src/models/mod.rs`, beside `CommandEnvelope`)

All derive `Debug, Clone, PartialEq, Serialize, Deserialize` — serializable from
day one (constraint satisfied). The enum is `#[serde(tag = "type", rename_all =
"snake_case")]` so each line self-describes.

```rust
/// One NDJSON line of `-o stream-json`. Exactly one Init first, exactly one
/// Result last; zero or more Progress in between. Ordering is contractual.
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Init(InitEvent),
    Progress(ProgressEvent),
    Result(ResultEvent),
}

pub struct InitEvent {
    pub schema_version: u32,     // same value as CommandEnvelope.schema_version
    pub seq: u64,                // monotonic, starts at 0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<u64>, // first-class, never scraped
    pub backend: String,
    pub shell: ShellType,        // reuse existing enum
    pub model_warm: bool,        // mirrors Timings.model_warm (false in v1)
}

pub struct ProgressEvent {
    pub schema_version: u32,
    pub seq: u64,
    /// Coarse machine-readable stage, not free text.
    pub stage: ProgressStage,    // Initializing | Generating | Validating
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
}

#[serde(rename_all = "snake_case")]
pub enum ProgressStage { Initializing, Generating, Validating }

pub struct ResultEvent {
    pub schema_version: u32,
    pub seq: u64,
    /// The ADR 003 envelope, verbatim — single source of truth for the outcome.
    pub envelope: CommandEnvelope,
}
```

**Method contracts**
- `StreamEvent::init(seq, session_id, backend, shell, model_warm) -> Self`.
- `StreamEvent::progress(seq, stage, elapsed_ms) -> Self`.
- `StreamEvent::result(seq, envelope) -> Self` — `schema_version` copied from the
  envelope so the two can never disagree.
- `StreamEvent::to_ndjson_line(&self) -> serde_json::Result<String>` — compact
  (`serde_json::to_string`, **not** pretty) + trailing `\n`; one object per line.
- `StreamEvent::exit_code(&self) -> Option<i32>` — `Some` only for `Result`
  (returns `envelope.exit_code`); lets the driver find the terminating code.

A tiny sink type keeps the emit loop stateless and testable:

```rust
/// Writes one flushed NDJSON line per event. No buffering across events, so a
/// killed process leaves a clean prefix of whole lines (never a half line).
pub struct StreamEmitter<W: Write> { writer: W, seq: u64 }
impl<W: Write> StreamEmitter<W> {
    pub fn new(writer: W) -> Self;
    pub fn emit(&mut self, make: impl FnOnce(u64) -> StreamEvent) -> io::Result<()>;
    // make() receives the next seq; emit() writes the line and flushes.
}
```

### Exit-code / output contract (what machines depend on)

Identical to ADR 003 — the stream reframes, it does not change semantics:

| Code | terminal `envelope.status` / `error.kind` | Trigger |
|---|---|---|
| 0 | `ok` | command generated and allowed |
| 1 | `error` / `internal` | unexpected internal failure |
| 2 | `error` / `usage` | no/empty prompt, invalid flag combination |
| 3 | `blocked` | safety validator rejected the command |
| 4 | `error` / `backend_unavailable` | model missing or remote backend unreachable |
| 5 | `error` / `config` | invalid/unreadable configuration |
| 6 | `error` / `auth` | remote backend credential/auth failure |
| 201 | (edit mode) | existing `EXIT_CODE_EDIT`, unchanged |

**Stream guarantees (the contract machines depend on):**
1. Line 1 is exactly one `init`. The last line is exactly one `result`. `progress`
   only in between.
2. `seq` is strictly increasing from 0, no gaps.
3. Every line is a complete JSON object terminated by `\n`, flushed before the
   next begins — a killed process yields a clean whole-line prefix.
4. The process exits with the `result` event's `envelope.exit_code`. A consumer
   reading only the last line + exit code is fully served (parity with `-o json`).
5. Every event carries the same `schema_version`; a bump is a deliberate,
   documented breaking change (locked by the snapshot test below).

This fixes the Phase-1 failure modes **by design**: version-coupled/undocumented
events → `schema_version` + a typed enum; no ordering/terminal guarantee → `seq`
+ mandated first/last events; hang/interleave → stateless, per-line flush;
billing assumption → `Timings`, no `$` fields.

### Minimal set of files to change (no new src modules)

1. **`src/models/mod.rs`** — add `StreamEvent` (+ `InitEvent`, `ProgressEvent`,
   `ProgressStage`, `ResultEvent`, `StreamEmitter`) and their constructors /
   `to_ndjson_line` / `exit_code`. Reuses `CommandEnvelope`, `ShellType`.
2. **`src/cli/mod.rs`** — add `OutputFormat::StreamJson` + its `from_str`
   (`"stream-json" | "stream_json"`), update the `-o` help text, and add a shared
   `emit_stream(...)` helper (mirrors the ADR 003 `emit_envelope` helper).
3. **`src/ai/runner.rs`** — accept an optional `&mut StreamEmitter` (or an
   `Option<&mut dyn FnMut(ProgressStage, u64)>`) so `run_once` can emit
   `Initializing`/`Generating`/`Validating` around the existing backend and
   validator calls it already times for `Timings`. No change to `AiOutcome`'s
   shape beyond ADR 003's `into_envelope`.
4. **`src/main.rs`** — in `run_ai_once` and the one-shot `generate` path, when
   `output_format == StreamJson`: construct a `StreamEmitter` over stdout, emit
   `init`, thread it as the progress sink, build the envelope exactly as under
   `-o json`, emit the terminal `result`, then `process::exit(envelope.exit_code)`.
   Human chatter stays on stderr.
5. **`tests/stream_json_contract.rs`** — new integration test binary (a `tests/`
   file is a separate crate, not a `src` module → respects "no new modules").

### Integration tests — known inputs → deterministic NDJSON + exit code

Use the deterministic `mock`/static backend so events are reproducible. Assertions
parse each line as JSON and check order, `seq`, and the terminal envelope.

| # | Invocation | Asserted stream | Exit |
|---|---|---|---|
| 1 | `caro -o stream-json -b mock "list files"` | line0 `type=="init"`; last line `type=="result"`, `envelope.status=="ok"`, `envelope.command` non-empty; `seq` strictly increasing from 0 | 0 |
| 2 | `caro -o stream-json -b mock "delete everything"` (mock → `rm -rf /`) | last line `type=="result"`, `envelope.status=="blocked"`, `envelope.allowed==false`, `risk` ∈ {High,Critical} | 3 |
| 3 | `printf '' \| caro -o stream-json` (empty prompt) | exactly one `init` then one `result` with `envelope.error.kind=="usage"` | 2 |
| 4 | `caro -o stream-json -b ollama --endpoint http://127.0.0.1:1` | terminal `result`, `envelope.error.kind=="backend_unavailable"` | 4 |
| 5 | `caro ai --once -o stream-json -b mock "list files"` ×2 | 2nd run `init.session_id` == 1st run's; terminal `result.envelope.resumed==true` | 0 |
| 6 | every line of test 1 | each carries the same `schema_version`; `init` first, `result` last, `progress` only between; no gaps in `seq` | n/a |
| 7 | serde round-trip / snapshot of a fixed 3-event stream | byte-stable field order, `type` tag present on every line, `result.envelope` equals the `-o json` output for the same input | n/a |

Test 7 locks the wire contract *and* the `-o json` ⇔ last-line-of-`stream-json`
equivalence: any drift between the two modes, or any field rename/reorder, fails
CI and forces a deliberate `schema_version` bump.

### Explicit out-of-scope (next version)

- **Token-delta streaming (`StreamEvent::Token`).** Per-token advisory output is a
  separate, backend-agnostic best-effort concern; the enum is designed to accept a
  `Token` variant later without a breaking change. This ADR ships the *lifecycle*
  events (init → progress → result) that have an immediate consumer.
- **`--input-format stream-json` (bidirectional / multi-turn over stdin).** A
  streamed *conversation* is a follow-up; v1 is one prompt → one event stream.
- **Persistent warm model cache / `caro serve` daemon.** Still the real fix for
  cold-start; still deferred (violates "no daemon"). The stream only *surfaces*
  the cost via `init.model_warm` and `result.envelope.timings.model_init_ms`.
- **`api_retry`-style backend retry events.** Add a `progress` stage or a variant
  when remote-backend retry telemetry is wired; not in v1.
- **YAML/other stream framing.** NDJSON only; `OutputFormat::Yaml` stays
  single-document.

### Constraint compliance check
- *Reuse validator/safety/config:* terminal event embeds the ADR 003 envelope,
  which already wraps `SafetyValidator`, `CliApp`, `SessionStore`, `RiskLevel`.
  Zero duplication. ✅
- *All new types serializable from day one:* full serde derives on every event. ✅
- *Pure subprocess, no daemon/state:* one process, NDJSON to stdout, per-line
  flush, exit with the envelope's code; bare by default. ✅
- *Solve the Phase-1 failure mode by design:* `schema_version` + `seq` + mandated
  first/last events + stateless per-line flush replace version-coupled/unordered/
  potentially-hanging streams — structurally, not as a workaround. ✅

---

## Suggested landing sequence (per repo rules)
1. Land ADR 003's envelope first (this ADR depends on `CommandEnvelope`); if 003
   is still unmerged, sequence 004 behind it.
2. Feature branch via `bin/sk-new-feature "stream-json event contract"`
   (`.claude/rules/git-workflow.md` — never commit to `main`).
3. Add `StreamEvent` + `OutputFormat::StreamJson` + tests behind the new flag
   (additive; `Plain` stays default, `-o json` unchanged).
4. Move this ADR into `docs/adr/` (already placed) and add its row to
   `docs/adr/README.md`; renumber if a `004-` already merged
   (`.claude/rules/adr-numbering.md`).
5. Output-format/contract feature, not a new product line → the
   `validation-discipline.md` 20-transcript gate does not apply; standard
   `dev-process.md` CI (`cargo test`, `cargo clippy -- -D warnings`) governs.

## Sources
- [Run Claude Code programmatically — Claude Code Docs](https://code.claude.com/docs/en/headless)
- [Headless Mode and CI/CD — Command Reference, SFEIR Institute](https://institute.sfeir.com/en/claude-code/claude-code-headless-mode-and-ci-cd/command-reference/)
- [Issue #24596 — CLI `--output-format stream-json` lacks event type reference](https://github.com/anthropics/claude-code/issues/24596)
- [Issue #24612 — Document all message types emitted by `claude -p --output-format stream-json`](https://github.com/anthropics/claude-code/issues/24612)
- [Issue #24594 — `--input-format stream-json` usage is undocumented](https://github.com/anthropics/claude-code/issues/24594)
- Caro codebase: `src/main.rs`, `src/cli/mod.rs`, `src/models/mod.rs`,
  `src/ai/{runner,session,store}.rs`, `src/safety/`, `src/assessment/formatters/json.rs`.
- Prior run: `docs/adr/003-structured-output-session-contract.md`,
  `caro-scope-structured-output.md`.
