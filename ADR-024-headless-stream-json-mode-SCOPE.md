# Scoping: Caro Headless Mode — Deterministic `stream-json` Event Contract + Warm-Init Sessions

> **Status of this document.** Research + scope deliverable produced by the
> `caro-research--scoping-process` scheduled task on 2026-06-15. The task
> template shipped with an unfilled `[FEATURE NAME]` placeholder; the feature
> selected for analysis is **Claude Code / Claude Agent SDK "headless mode"
> (`--output-format stream-json` + `--resume` session resumption)**, chosen
> because every clue in the template — *structured output contract (payloads,
> exit codes, events)*, *session/context lifecycle (avoid redundant
> initialization)*, *pure subprocess (no daemon, no state)*, *experimental /
> not-yet-GA* — maps directly onto that feature. This is a scope, not an
> implementation: no branch, no PR, no code committed. The embedded ADR draft
> is numbered **ADR-024** (next sequential per `.claude/rules/adr-numbering.md`)
> and is ready to be lifted into `docs/adr/` by a feature session.

---

## Phase 1 — Feature Research

### What it is, what problem it solves, for whom

Claude Code's **headless mode** turns an interactive agent into a scriptable
subprocess: `claude -p "<prompt>" --output-format <text|json|stream-json>`. The
Agent SDK exposes the same surface programmatically. It exists for **CI/CD and
automation consumers** — people who need a machine-readable answer, an exit
code they can branch on, and (for long tasks) a streaming feed they can render
in a dashboard or pipe to `jq`. Roughly 85% of CI integrations use the plain
`json` format and parse `.result` with `jq`.

Three output formats:

| Format | Shape | Consumer |
|---|---|---|
| `text` | Human-readable final answer | A person reading a terminal |
| `json` | One object: `{ result, model, usage, cost_usd, session_id, … }` | Scripts that want the final answer + metadata |
| `stream-json` | **NDJSON** — one self-contained JSON event per line | Real-time dashboards, progress UIs, long-running tasks |

The `stream-json` stream **opens with a `system` event whose `subtype` is
`init`** reporting run metadata: `session_id`, model, available tools, MCP
servers, loaded plugins. Subsequent events carry `type` of `assistant`,
`tool_use`, and a terminal `result`.

### Session / context lifecycle

The `init` event surfaces a `session_id`. Capture it, and a later call with
`--resume <session_id>` (or `--continue` for "the last one") replays the prior
conversation so context carries forward. Each session retains up to ~200k
tokens.

### Why it is experimental / limited, and the failure modes

1. **Silent success (the headline failure mode).** Headless mode follows Unix
   exit conventions — `0` success, `1` generic error, `2` argument error — but
   a documented failure is **exit code `0` with an empty, truncated, or
   irrelevant result**. A pipeline that branches only on the exit code gets a
   false green. ~85% of headless errors trace to three causes (bad API key,
   timeout, unauthorized tool), several of which can present as a silent zero.

2. **Resume reloads the world (the redundant-initialization failure mode).**
   `--resume` is conceptually "replay the conversation," so each subprocess
   call re-pays initialization: re-resolving config, re-enumerating tools/MCP
   servers, and re-hydrating up to 200k tokens of context. There is no
   daemon-free way in the upstream design to say "the environment is unchanged,
   skip re-init." For a **local-LLM** tool this is far worse than for a hosted
   API: re-init can mean **reloading model weights from disk on every call**.

3. **`stream-json` *input* is the experimental edge.** Streaming JSON *output*
   is stable enough for CI; bidirectional streaming JSON *input* is the part
   still flagged experimental and is where contract drift bites integrators.

4. **Partial-payload ambiguity.** Each NDJSON line is "a partial payload."
   Without a strict, versioned per-event schema and a guaranteed terminal
   event, consumers cannot reliably tell "still working" from "done" from
   "died mid-stream."

---

## Phase 2 — Competitive Differentiation

### What they get right (replicate)

- **NDJSON, one self-contained object per line.** Trivially `jq`-able, no
  partial-line reassembly. Keep this exactly.
- **A leading `init` event carrying run metadata.** This is the right place to
  publish `session_id`, model, safety level, shell, and resolved capabilities.
- **Three-tier format ladder** (`text` for humans, `json` for one-shot scripts,
  `stream-json` for live feeds). Caro already has `text/plain` + `json`; we add
  `stream-json` as the third rung rather than inventing a parallel mechanism.

### Their gaps we avoid by designing the schema first

- **No silent-success.** Caro's terminal event is **mandatory** and carries an
  explicit `subtype` (`success` | `blocked` | `refused` | `error`). The exit
  code is *derived from* that subtype, never set independently. A consumer that
  reads only the exit code and a consumer that reads only the terminal event
  reach the *same* conclusion — by construction.
- **Versioned envelope.** Every line carries `"schema": "caro.headless.v1"`.
  Breaking the contract requires bumping the string; consumers can hard-fail on
  an unknown schema instead of mis-parsing.
- **Guaranteed terminal sentinel.** Exactly one `result` event is emitted, and
  it is always last. Mid-stream death is detectable as "stream ended without a
  `result`."
- **Resume that does not reload the world** (see Phase 1 failure mode #2),
  solved by design in Phase 3 via a content-addressed init cache.

### Our unique positioning (what they cannot do)

- **Fully offline, no API key, no token cost.** The `cost_usd`/`api_key` failure
  classes simply do not exist for Caro's embedded backend. `usage`/`cost`
  fields are *optional* and present only for remote backends.
- **Safety verdict is a first-class field, not prose.** Every command event
  carries the structured `RiskLevel` + `SuggestedRouting` from the existing
  validator. A CI gate can branch on `risk == "critical"` without NLP.
- **Pure subprocess, genuinely stateless.** No daemon, no background server,
  no socket. State lives in an on-disk JSON session file the user owns.
- **Deterministic for the static backend.** With `--backend static`, identical
  input ⇒ byte-identical NDJSON (modulo monotonic timestamps, which a
  `--deterministic` flag pins). This is what makes integration tests possible
  (Phase 3) — upstream cannot promise this.

### Existing Caro infrastructure that already covers part of this

| Need | Already exists | File |
|---|---|---|
| Output-format selection | `OutputFormat { Json, Yaml, Plain }` + `FromStr` | `src/cli/mod.rs` |
| Serializable command result | `GeneratedCommand { command, explanation, safety_level: RiskLevel, estimated_impact, alternatives, backend_used, generation_time_ms, confidence_score }` | `src/models/mod.rs` |
| Safety verdict types | `RiskLevel`, `SuggestedRouting`, `SafetyLevel` (all serde) | `src/models/mod.rs` |
| Backend metadata for `init` | `BackendInfo { backend_type, model_name, supports_streaming, max_tokens, … }` (serde) | `src/models/mod.rs`, `src/backends/mod.rs` |
| Session model | `AiSession`, `Turn`, `Role` (all serde) | `src/ai/session.rs` |
| Session persistence + resume | `SessionStore::{open, create, upsert, get, resume_recent(minutes)}` (JSON-backed) | `src/ai/store.rs` |
| Session config | `AiConfig { session_continue_minutes, db_path, … }` | `src/models/mod.rs` |
| Generation contract | `CommandGenerator::{generate_command, is_available, backend_info, shutdown}` | `src/backends/mod.rs` |

**Conclusion:** the session lifecycle and the serializable result types already
exist. This feature is **not** "build sessions" — it is "(a) add a third output
format that re-serializes existing types into a versioned NDJSON event stream,
(b) bind `--resume`/`--session-id` to the existing `SessionStore`, and (c) add a
content-addressed init cache so resume skips redundant initialization." No new
module is required.

---

## Phase 3 — Scope Definition

### ADR-024 (draft, ready for `docs/adr/`)

```markdown
# ADR-024: Headless `stream-json` Mode and Warm-Init Sessions

## Status
Proposed — 2026-06-15

## Context
Caro is consumed both interactively and from scripts/CI. Today scripts get
`--output -o json`, a single final object. There is no streaming, no
event-level progress, and no contract for resuming context across invocations
without re-paying initialization cost. Claude Code's headless mode
(`--output-format stream-json` + `--resume`) is the reference design, but it
has two failure modes we must not inherit: (1) **silent success** — exit code 0
with an empty/irrelevant result; (2) **redundant initialization** — resume
replays the full environment, which for a local-LLM tool can mean reloading
model weights every call.

## Decision
1. Add a third `OutputFormat::StreamJson` that emits **NDJSON**, one versioned
   event object per line (`"schema":"caro.headless.v1"`), opening with a
   `system/init` event and closing with **exactly one** terminal `result`
   event.
2. The process **exit code is derived solely from the terminal event's
   `subtype`** (`success`→0, `blocked`→3, `refused`→4, `error`→1, arg error→2).
   Emitting a `result` and setting the exit code are the same code path, so
   silent success is structurally impossible.
3. Bind `--session-id <id>` / `--resume <id>` / `--continue` to the **existing**
   `SessionStore`. Reuse `AiSession`/`Turn`; do not add a parallel store.
4. Solve redundant initialization **without a daemon** via a
   content-addressed **`InitDigest`**: a hash over `{resolved config, safety
   level, shell, backend+model id, capability profile, context-index
   fingerprint}`. The `system/init` event publishes the digest. On resume, if
   the recomputed digest equals the stored one, Caro skips re-running directory
   indexing / capability profiling / model-capability assessment and reuses the
   cached warm state; the model is loaded lazily only when a generation is
   actually requested.

## Consequences
+ CI gets a stable, versioned, `jq`-able event stream with a safety verdict as
  a first-class field and an exit code that can never disagree with the stream.
+ Resume is cheap: unchanged environment ⇒ no re-indexing, no redundant
  profiling.
+ Pure subprocess; no daemon, no socket, no background state.
− One more output format to keep in sync as result types evolve (mitigated:
  the event types *wrap* existing serde types rather than redefining fields).
− `InitDigest` correctness depends on hashing the *right* inputs; a missed
  input means a stale warm-init. Mitigated by a `--no-warm-init` escape hatch
  and by treating the digest inputs as a reviewed, enumerated list.

## Alternatives considered
- **Daemon / persistent server keyed by session.** Rejected: violates the
  "pure subprocess, no daemon, no state" constraint and Caro's offline ethos.
- **Reuse `OutputFormat::Json` and just print multiple objects.** Rejected:
  breaks existing single-object `json` consumers; no init/terminal framing.
- **A brand-new `src/headless/` module.** Rejected: the result and session
  types already exist; a new module would duplicate them. Lands as a thin
  serializer + CLI wiring instead.
- **Inherit upstream "resume replays everything."** Rejected: it is the
  documented failure mode we are explicitly chartered to design out.
```

### New types (added to existing modules — no new module)

Added to **`src/cli/mod.rs`** (next to `OutputFormat`):

```rust
// extend the existing enum
pub enum OutputFormat { Json, Yaml, Plain, StreamJson } // + "stream-json" in FromStr
```

Added to a new file **`src/cli/headless.rs`** (a submodule of the existing `cli`
module, *not* a new top-level module — registered with `mod headless;` in
`src/cli/mod.rs`). All types `#[derive(Serialize, Deserialize)]` from day one:

```rust
/// Wire envelope; one per NDJSON line. `schema` is a hard version gate.
pub struct HeadlessEvent {
    pub schema: &'static str,      // always "caro.headless.v1"
    pub session_id: String,        // ties every line to a SessionStore AiSession
    pub seq: u64,                  // monotonic per stream; first event seq=0
    #[serde(flatten)]
    pub body: HeadlessBody,        // tagged union below
}

#[serde(tag = "type", rename_all = "snake_case")]
pub enum HeadlessBody {
    /// Always first. Publishes run metadata + the warm-init digest.
    System(SystemInit),
    /// A generated command + its safety verdict (wraps existing types).
    Command(CommandEvent),
    /// Optional progress ticks for long generations (heartbeat).
    Progress(ProgressEvent),
    /// Always last; exactly one. Exit code is derived from `subtype`.
    Result(ResultEvent),
}

pub struct SystemInit {
    pub subtype: String,           // "init"
    pub backend: crate::models::BackendInfo,  // reuse
    pub shell: crate::models::ShellType,      // reuse
    pub safety_level: crate::models::SafetyLevel, // reuse
    pub init_digest: String,       // content-addressed warm-init key
    pub warm_init_reused: bool,    // true when resume skipped re-init
    pub resumed_from: Option<String>, // prior session_id if --resume
}

pub struct CommandEvent {
    #[serde(flatten)]
    pub command: crate::models::GeneratedCommand, // reuse (command, explanation,
                                                  // safety_level, alternatives, …)
    pub routing: crate::models::SuggestedRouting, // reuse
}

pub struct ProgressEvent { pub stage: String, pub elapsed_ms: u64 }

#[serde(rename_all = "snake_case")]
pub enum ResultSubtype { Success, Blocked, Refused, Error }

pub struct ResultEvent {
    pub subtype: ResultSubtype,
    pub exit_code: i32,            // == HeadlessExit::from(subtype) as i32
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,  // human note on non-success
    pub elapsed_ms: u64,
}
```

Added to **`src/ai/session.rs`** (alongside `AiSession`):

```rust
/// Content-addressed warm-init key. Serializable; stored on the AiSession.
pub struct InitDigest(pub String);   // hex sha256 of the canonical input tuple

impl InitDigest {
    /// Deterministic over the enumerated, reviewed input set.
    pub fn compute(
        cfg_fingerprint: &str, safety: SafetyLevel, shell: ShellType,
        backend_model: &str, capability_profile: &str, context_index_fp: &str,
    ) -> Self { /* sha256 of a canonical, field-ordered string */ }
}
// AiSession gains: `pub init_digest: Option<InitDigest>` (#[serde(default)] for
// backward-compatible deserialization of existing session files).
```

**Method contracts:**
- `HeadlessEvent::write_line(&self, w: &mut impl Write) -> io::Result<()>` —
  serialize + `\n`, flush. The only emission path.
- `HeadlessExit::from(ResultSubtype) -> i32` — the single source of truth for
  exit codes; the CLI calls `process::exit` with exactly this value.
- `SessionStore` reused as-is via `get(id)` / `create` / `upsert` /
  `resume_recent`. New thin helper `resume_or_create(&mut self, id: Option<&str>,
  shell) -> AiSession` lives in `headless.rs`, not in the store.

### Minimal set of files that change

1. `src/cli/mod.rs` — add `StreamJson` variant + `"stream-json"` arm in
   `FromStr`; add `--session-id` / `--resume` / `--continue` / `--no-warm-init`
   / `--deterministic` flags; route `OutputFormat::StreamJson` to the new
   serializer.
2. `src/cli/headless.rs` — **new submodule file** (not a new top-level module):
   the event types above + `write_line`, `HeadlessExit`, `resume_or_create`,
   and the `run_headless()` orchestration that emits `init → command(s) →
   result`.
3. `src/ai/session.rs` — add `InitDigest` + `AiSession.init_digest` field.
4. `src/main.rs` — when format is `StreamJson`, hand off to
   `cli::headless::run_headless(...)` and exit with `HeadlessExit::from(...)`.
5. `docs/adr/ADR-024-headless-stream-json-mode.md` — the ADR above.
6. `CHANGELOG.md` — `Added` entry under the next release.
7. `tests/headless_contract.rs` — **new integration test file** (Phase below).

No new top-level module. No new dependency (serde, sha2/`sha256` already in the
cache/checksum layer, chrono already present).

### Exit-code / output contract (what machines depend on)

| Exit | `ResultSubtype` | Meaning |
|---|---|---|
| `0` | `success` | A command was generated and is safe to surface |
| `1` | `error` | Generation/runtime error (backend unavailable, model load failed) |
| `2` | — | Argument error (bad flag, unknown format, missing prompt) |
| `3` | `blocked` | Command generated but **blocked by safety validator** (e.g. CRITICAL) |
| `4` | `refused` | Request refused before generation (policy/governance) |

Stream guarantees consumers may rely on under `schema == "caro.headless.v1"`:
- The **first** line is always `type == "system"`, `subtype == "init"`.
- The **last** line is always `type == "result"`, and there is **exactly one**.
- `result.exit_code` **always equals** the process exit code.
- Every line is independently valid JSON with `schema`, `session_id`, `seq`.
- Stream ending without a `result` ⇒ the process died; treat as failure.

### Integration tests (deterministic input → fixed JSON + exit code)

Run with `--backend static --deterministic` (pins timestamps to `0`, `seq`
monotonic) so output is byte-stable.

1. **Happy path.** `caro --backend static --deterministic
   --output-format stream-json -p "list files"` ⇒ stdout is exactly three
   lines (`init`, `command`, `result`); `result.subtype=="success"`,
   `result.exit_code==0`; process exits `0`. Assert full NDJSON equality.
2. **Safety block.** Prompt resolving to a CRITICAL pattern (e.g.
   `rm -rf /`) ⇒ a `command` event with `safety_level=="critical"`, then
   `result.subtype=="blocked"`, `exit_code==3`; process exits `3`.
3. **Arg error.** `--output-format stream-jsom` (typo) ⇒ no NDJSON on stdout,
   error on stderr, exit `2`.
4. **No silent success (regression guard for Phase-1 failure mode #1).**
   Force an empty generation; assert the stream still ends with a `result`
   whose `subtype` is `error` and `exit_code==1` — never a `0` with no
   `result`.
5. **Resume skips re-init (regression guard for failure mode #2).** First call
   captures `session_id` + `init_digest` from the `init` event. Second call
   with `--resume <id>` in an unchanged environment ⇒ `init.warm_init_reused
   == true`, `init.resumed_from == <id>`, and the recomputed `init_digest`
   equals the first. With `--no-warm-init`, `warm_init_reused == false`.
6. **Schema gate.** Every emitted line has `schema == "caro.headless.v1"`;
   a snapshot test fails loudly if any field name changes (forces a version
   bump conversation).

### Explicitly out of scope (next version)

- **`stream-json` as *input*** (bidirectional streaming / multi-turn piped
  conversation). Output-only in v1 — input streaming is exactly the part
  upstream still flags experimental.
- **Tool-use / `tool_use` events.** Caro generates commands; it does not run an
  agentic tool loop in headless mode in v1.
- **Token `usage` / `cost_usd` for remote backends.** Fields are reserved in
  the envelope (optional) but not populated until a later release.
- **Per-token streaming of the command text.** v1 streams *events*, not tokens;
  the `command` event is atomic.
- **`--resume` across machines / session export-import.** Session files are
  local; portability is a later concern.
- **MCP-server / plugin enumeration in `init`.** Reserved field; populated when
  the MCP safety server (ADR-015) integration lands in headless mode.

---

## Constraint compliance check

| Constraint | How this scope satisfies it |
|---|---|
| Reuse existing validator/safety/config infra; don't duplicate | `CommandEvent` flattens `GeneratedCommand` + `SuggestedRouting`; safety verdict comes from the existing validator; format selection extends the existing `OutputFormat`; sessions reuse `SessionStore`. |
| All new types serializable from day one | Every new type derives `Serialize, Deserialize`; `AiSession.init_digest` uses `#[serde(default)]` for backward compat. |
| Pure subprocess, no daemon, no state | No server/socket. State is the user-owned on-disk JSON session file. Warm-init is a content-addressed cache, not a running process. |
| Solve the Phase-1 failure mode by design, not workaround | (a) Silent success eliminated: exit code is *derived from* the mandatory terminal `result` subtype — one code path. (b) Redundant init eliminated: `InitDigest` lets resume skip re-indexing/profiling and defer model load, with no daemon. |
