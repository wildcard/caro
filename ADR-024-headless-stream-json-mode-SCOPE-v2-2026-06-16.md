# Scoping (v2): Caro Headless Mode — Deterministic `stream-json` Event Contract + Warm-Init Sessions

> **Status of this document.** Second pass of the `caro-research--scoping-process`
> scheduled task, run **2026-06-16**. It supersedes the
> `2026-06-15` scope (`ADR-024-headless-stream-json-mode-SCOPE.md`) by
> **verifying every code citation against `src/`** and correcting two
> schema-level mistakes that "design the schema first" is precisely meant to
> catch. The selected feature is unchanged: **Claude Code / Claude Agent SDK
> "headless mode" (`-p --output-format stream-json` + `--resume` session
> resumption)** — the template's `[FEATURE NAME]` shipped unfilled, and every
> clue (*structured output contract: payloads, exit codes, events*;
> *session/context lifecycle that avoids redundant init*; *pure subprocess, no
> daemon, no state*; *experimental / not-yet-GA*) maps onto that feature.
>
> This is a scope, not an implementation: **no branch, no PR, no code
> committed** (per `.claude/rules/git-workflow.md`, any actual code lands on a
> feature branch). The embedded ADR is numbered **ADR-024** — verified as the
> next sequential number (`docs/adr/` tops out at ADR-023) per
> `.claude/rules/adr-numbering.md`.

---

## What changed since the 2026-06-15 pass (read this first)

The prior scope was substantially correct, but two citations were wrong in
ways that would have produced a broken schema if implemented verbatim:

1. **Session id is `u64`, not `String`.** `AiSession.id: u64`
   (`src/ai/session.rs:62`) and `SessionStore::get(id: u64)`
   (`src/ai/store.rs:102`). The v1 scope typed the wire field as
   `session_id: String` and the CLI flags as `--resume <String>`. **Correction:**
   the canonical id is `u64`; the wire field is serialized as a `u64`, and the
   CLI accepts `--session-id <u64>`. (A string alias is fine as an *input
   convenience* but the stored/serialized identity is `u64`, so the schema
   must say `u64` to round-trip against `SessionStore`.)

2. **There are two `BackendInfo` types.** One in `src/backends/mod.rs:36`
   (`backend_type, model_name, supports_streaming, max_tokens,
   typical_latency_ms, memory_usage_mb, version`) and one in
   `src/models/mod.rs:426` (`backend_type, model_name, supports_streaming,
   max_tokens, …`). The v1 scope wrote `crate::models::BackendInfo` in one
   place and `BackendInfo` from `src/backends/mod.rs` in the citation table —
   ambiguous. **Correction:** `SystemInit` embeds **`crate::models::BackendInfo`**
   (the richer, already-serde type used elsewhere for metadata); the backends
   variant is an internal detail and must not appear on the wire. This is
   itself a finding: the duplicate type is a latent footgun and the headless
   schema should pin exactly one.

Everything else verified accurate: `OutputFormat { Json, Yaml, Plain }`
(`src/cli/mod.rs:101`); `GeneratedCommand { command, explanation,
safety_level: RiskLevel, estimated_impact, alternatives, backend_used,
generation_time_ms, confidence_score }` (`src/models/mod.rs:72`);
`SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }`
(`src/models/mod.rs:189`); `SessionStore::{open, resume_recent(minutes),
create(shell), upsert, get(u64)}` (`src/ai/store.rs`); `sha2 = "0.10"` already
a dependency (`Cargo.toml:90`) — **no new crate needed**. A full
citation-verification table is in the appendix.

---

## Phase 1 — Feature Research

### What it is, what problem it solves, for whom

Claude Code's **headless mode** turns an interactive agent into a scriptable
subprocess: `claude -p "<prompt>" --output-format <text|json|stream-json>`. The
Agent SDK exposes the same surface programmatically. It targets **CI/CD and
automation consumers** who need a machine-readable answer, an exit code to
branch on, and (for long tasks) a streaming feed to render or pipe to `jq`.

Three output formats:

| Format | Shape | Consumer |
|---|---|---|
| `text` | Human-readable final answer | A person at a terminal |
| `json` | One object: `{ result, model, usage, cost_usd, session_id, … }` | Scripts wanting the final answer + metadata |
| `stream-json` | **NDJSON** — one self-contained JSON event per line | Real-time dashboards, progress UIs, long tasks |

The `stream-json` stream **opens with a `system` event, `subtype: "init"`**
reporting run metadata (session id, model, available tools, MCP servers,
loaded plugins); subsequent events are `assistant`, `tool_use`, and a terminal
`result`. `stream-json` is read in real time so consumers can process messages
as they arrive.

### Session / context lifecycle

The `init` event surfaces a session identifier. Capture it, then a later call
with `--resume <id>` (or `--continue` for "the last one") replays the prior
conversation so context carries forward. Resume must be combined with `-p` for
non-interactive multi-turn. Sessions retain a large context window
(~200k tokens upstream).

### Why it is experimental / limited — the failure modes

1. **Silent success (the headline failure mode).** Headless mode follows Unix
   exit conventions — `0` success, non-zero error — but a documented failure is
   **exit code `0` with an empty/truncated/irrelevant result**. A pipeline
   branching only on the exit code gets a false green. Common error causes (bad
   key, timeout, rate-limit/tool failure) can present as silent zeros if the
   final payload isn't inspected.

2. **Resume reloads the world (the redundant-initialization failure mode).**
   `--resume` is "replay the conversation," so each subprocess re-pays
   initialization: re-resolving config, re-enumerating tools/MCP servers,
   re-hydrating context. For a **local-LLM** tool this is far worse than for a
   hosted API — re-init can mean **reloading model weights from disk on every
   call**. There is no daemon-free upstream way to say "environment unchanged,
   skip re-init."

3. **`stream-json` *input* is the experimental edge.** Streaming JSON *output*
   is stable enough for CI; bidirectional streaming JSON *input* is the part
   still flagged experimental and where contract drift bites integrators.

4. **Partial-payload ambiguity.** Each NDJSON line is a "partial payload."
   Without a strict, versioned per-event schema and a guaranteed terminal
   event, consumers cannot reliably distinguish "still working" from "done"
   from "died mid-stream."

---

## Phase 2 — Competitive Differentiation

### What they get right (replicate)

- **NDJSON, one self-contained object per line.** Trivially `jq`-able, no
  partial-line reassembly. Keep exactly.
- **A leading `init` event carrying run metadata** — the right home for session
  id, model, safety level, shell, resolved capabilities.
- **Three-tier format ladder** (`text`/`json`/`stream-json`). Caro already has
  `Plain`/`Json` (+`Yaml`); we add `StreamJson` as the third rung rather than a
  parallel mechanism.

### Their gaps we avoid by designing the schema first

- **No silent-success.** Caro's terminal event is **mandatory** and carries an
  explicit `subtype` (`success` | `blocked` | `refused` | `error`). The exit
  code is *derived from* that subtype, never set independently — one code path,
  so a consumer reading only the exit code and one reading only the terminal
  event reach the same conclusion by construction.
- **Versioned envelope.** Every line carries `"schema": "caro.headless.v1"`.
  Breaking the contract requires bumping the string; consumers hard-fail on an
  unknown schema instead of mis-parsing.
- **Guaranteed terminal sentinel.** Exactly one `result` event, always last.
  Mid-stream death is detectable as "stream ended without a `result`."
- **Resume that does not reload the world** — solved by design in Phase 3 via a
  content-addressed init digest, no daemon.

### Our unique positioning (what they cannot do)

- **Fully offline, no API key, no token cost.** The `cost_usd`/`api_key`
  failure classes don't exist for Caro's embedded backend; `usage`/`cost` are
  *optional*, present only for remote backends.
- **Safety verdict is a first-class field, not prose.** Every command event
  carries the structured `RiskLevel` + `SuggestedRouting` from the existing
  validator. A CI gate branches on `safety_level == "critical"` with no NLP.
- **Pure subprocess, genuinely stateless.** No daemon, no socket. State is the
  user-owned on-disk JSON session file (`SessionStore`).
- **Deterministic for the static backend.** With `--backend static
  --deterministic`, identical input ⇒ byte-identical NDJSON (timestamps pinned,
  `seq` monotonic). This is what makes the Phase-3 integration tests possible —
  upstream cannot promise byte-stability.

### Existing Caro infrastructure that already covers part of this (verified)

| Need | Already exists | Location (verified) |
|---|---|---|
| Output-format selection | `OutputFormat { Json, Yaml, Plain }` + `FromStr` | `src/cli/mod.rs:101` |
| Serializable command result | `GeneratedCommand { command, explanation, safety_level, estimated_impact, alternatives, backend_used, generation_time_ms, confidence_score }` | `src/models/mod.rs:72` |
| Safety routing verdict | `SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }` | `src/models/mod.rs:189` |
| Backend metadata for `init` | `BackendInfo` (serde) — **use `crate::models::BackendInfo`** | `src/models/mod.rs:426` (NB: a second `BackendInfo` exists at `src/backends/mod.rs:36`) |
| Session model | `AiSession { id: u64, created_at, last_at, shell, cwd, turns }`, `Turn`, `Role` (serde) | `src/ai/session.rs:60` |
| Session persistence + resume | `SessionStore::{open, resume_recent(minutes), create(shell), upsert, get(u64)}` (JSON-backed) | `src/ai/store.rs` |
| Hashing for the init digest | `sha2 = "0.10"` already a dependency | `Cargo.toml:90` |

**Conclusion:** session lifecycle and the serializable result types already
exist. This feature is **not** "build sessions." It is: (a) add a third output
format that re-serializes existing types into a versioned NDJSON event stream;
(b) bind `--session-id`/`--resume`/`--continue` to the existing `SessionStore`
(keyed by `u64`); (c) add a content-addressed init digest so resume skips
redundant initialization. **No new top-level module.**

---

## Phase 3 — Scope Definition

### ADR-024 (draft, ready for `docs/adr/`)

```markdown
# ADR-024: Headless `stream-json` Mode and Warm-Init Sessions

## Status
Proposed — 2026-06-16

## Context
Caro is consumed both interactively and from scripts/CI. Today scripts get
`-o json`, a single final object. There is no streaming, no event-level
progress, and no contract for resuming context across invocations without
re-paying initialization cost. Claude Code's headless mode
(`--output-format stream-json` + `--resume`) is the reference design, but it has
two failure modes we must not inherit: (1) **silent success** — exit code 0
with an empty/irrelevant result; (2) **redundant initialization** — resume
replays the full environment, which for a local-LLM tool can mean reloading
model weights every call.

## Decision
1. Add `OutputFormat::StreamJson` that emits **NDJSON**, one versioned event
   object per line (`"schema":"caro.headless.v1"`), opening with a
   `system`/`init` event and closing with **exactly one** terminal `result`.
2. The process **exit code is derived solely from the terminal event's
   `subtype`** (`success`→0, `error`→1, arg error→2, `blocked`→3, `refused`→4).
   Emitting the `result` and setting the exit code are the same code path, so
   silent success is structurally impossible.
3. Bind `--session-id <u64>` / `--resume <u64>` / `--continue` to the
   **existing** `SessionStore` (sessions are keyed by `u64`). Reuse
   `AiSession`/`Turn`; add no parallel store.
4. Solve redundant initialization **without a daemon** via a content-addressed
   **`InitDigest`**: a sha256 over `{resolved-config fingerprint, safety level,
   shell, backend+model id, capability profile, context-index fingerprint}`.
   The `init` event publishes the digest. On resume, if the recomputed digest
   equals the stored one, Caro skips directory indexing / capability profiling /
   model-capability assessment and reuses cached warm state; the model is loaded
   lazily only when a generation is actually requested.

## Consequences
+ CI gets a stable, versioned, `jq`-able event stream with a safety verdict as a
  first-class field and an exit code that can never disagree with the stream.
+ Resume is cheap: unchanged environment ⇒ no re-indexing, no redundant
  profiling, no eager weight reload.
+ Pure subprocess; no daemon, no socket, no background state.
− One more output format to keep in sync as result types evolve — mitigated:
  event types *wrap* existing serde types rather than redefining fields.
− `InitDigest` correctness depends on hashing the *right* inputs; a missed input
  means a stale warm-init. Mitigated by a `--no-warm-init` escape hatch and by
  treating the digest inputs as a reviewed, enumerated list.

## Alternatives considered
- **Daemon / persistent server keyed by session.** Rejected: violates "pure
  subprocess, no daemon, no state" and Caro's offline ethos.
- **Reuse `OutputFormat::Json`, print multiple objects.** Rejected: breaks
  existing single-object `json` consumers; no init/terminal framing.
- **A new `src/headless/` top-level module.** Rejected: result and session
  types already exist; lands as a `cli` submodule + serializer instead.
- **String session ids.** Rejected: `SessionStore`/`AiSession` are keyed by
  `u64`; a String identity would not round-trip without an extra mapping table.
- **Inherit upstream "resume replays everything."** Rejected: it is the
  documented failure mode we are chartered to design out.
```

### New types (added to existing modules — no new top-level module)

**`src/cli/mod.rs`** — extend the existing enum and `FromStr`:

```rust
pub enum OutputFormat { Json, Yaml, Plain, StreamJson } // + "stream-json" arm
```

**`src/cli/headless.rs`** — new *submodule* file (`mod headless;` in
`src/cli/mod.rs`), not a new top-level module. All types derive
`Serialize, Deserialize` from day one:

```rust
/// Wire envelope; one per NDJSON line. `schema` is a hard version gate.
pub struct HeadlessEvent {
    pub schema: &'static str,   // always "caro.headless.v1"
    pub session_id: u64,        // CORRECTED: matches AiSession.id / SessionStore::get(u64)
    pub seq: u64,               // monotonic per stream; first event seq=0
    #[serde(flatten)]
    pub body: HeadlessBody,
}

#[serde(tag = "type", rename_all = "snake_case")]
pub enum HeadlessBody {
    System(SystemInit),     // always first
    Command(CommandEvent),  // a generated command + safety verdict
    Progress(ProgressEvent),// optional heartbeat for long generations
    Result(ResultEvent),    // always last; exactly one
}

pub struct SystemInit {
    pub subtype: String,                          // "init"
    pub backend: crate::models::BackendInfo,      // CORRECTED: models::, not backends::
    pub shell: crate::models::ShellType,          // reuse
    pub safety_level: crate::models::SafetyLevel, // reuse
    pub init_digest: String,                      // content-addressed warm-init key
    pub warm_init_reused: bool,                   // true when resume skipped re-init
    pub resumed_from: Option<u64>,                // CORRECTED: u64 prior session id
}

pub struct CommandEvent {
    #[serde(flatten)]
    pub command: crate::models::GeneratedCommand, // reuse (verified fields)
    pub routing: crate::models::SuggestedRouting, // reuse
}

pub struct ProgressEvent { pub stage: String, pub elapsed_ms: u64 }

#[serde(rename_all = "snake_case")]
pub enum ResultSubtype { Success, Blocked, Refused, Error }

pub struct ResultEvent {
    pub subtype: ResultSubtype,
    pub exit_code: i32,                  // == HeadlessExit::from(subtype) as i32
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub elapsed_ms: u64,
}
```

**`src/ai/session.rs`** — alongside `AiSession`:

```rust
/// Content-addressed warm-init key. Serializable.
pub struct InitDigest(pub String); // hex sha256 of the canonical input tuple

impl InitDigest {
    pub fn compute(
        cfg_fingerprint: &str, safety: SafetyLevel, shell: &str,
        backend_model: &str, capability_profile: &str, context_index_fp: &str,
    ) -> Self { /* sha256 of a canonical, field-ordered string (sha2 already a dep) */ }
}
// AiSession gains: `#[serde(default)] pub init_digest: Option<InitDigest>`
// (serde(default) keeps existing on-disk session files deserializing cleanly).
```

**Method contracts:**
- `HeadlessEvent::write_line(&self, w: &mut impl Write) -> io::Result<()>` —
  serialize + `\n` + flush. The only emission path.
- `HeadlessExit::from(ResultSubtype) -> i32` — single source of truth for exit
  codes; `main` calls `process::exit` with exactly this value.
- `SessionStore` reused as-is (`get(u64)` / `create(shell)` / `upsert` /
  `resume_recent(minutes)`). A thin `resume_or_create(&mut SessionStore,
  id: Option<u64>, shell) -> AiSession` lives in `headless.rs`, not the store.

### Minimal set of files that change

1. `src/cli/mod.rs` — `StreamJson` variant + `"stream-json"` `FromStr` arm;
   `--session-id <u64>` / `--resume <u64>` / `--continue` / `--no-warm-init` /
   `--deterministic` flags; route `StreamJson` to the serializer.
2. `src/cli/headless.rs` — **new submodule**: the event types above +
   `write_line`, `HeadlessExit`, `resume_or_create`, and `run_headless()`
   orchestration (`init → command(s) → result`).
3. `src/ai/session.rs` — `InitDigest` + `AiSession.init_digest`.
4. `src/main.rs` — when format is `StreamJson`, hand off to
   `cli::headless::run_headless(...)` and exit with `HeadlessExit::from(...)`.
5. `docs/adr/ADR-024-headless-stream-json-mode.md` — the ADR above.
6. `CHANGELOG.md` — `Added` entry under the next release.
7. `tests/headless_contract.rs` — new integration test file (below).

No new top-level module. No new dependency (serde, sha2, chrono all present).

### Exit-code / output contract (what machines depend on)

| Exit | `ResultSubtype` | Meaning |
|---|---|---|
| `0` | `success` | A command was generated and is safe to surface |
| `1` | `error`   | Generation/runtime error (backend unavailable, model load failed) |
| `2` | —         | Argument error (bad flag, unknown format, missing prompt) |
| `3` | `blocked` | Command generated but blocked by the safety validator (e.g. CRITICAL) |
| `4` | `refused` | Request refused before generation (policy/governance) |

Guarantees consumers may rely on under `schema == "caro.headless.v1"`:
- The **first** line is always `type=="system"`, `subtype=="init"`.
- The **last** line is always `type=="result"`, and there is **exactly one**.
- `result.exit_code` **always equals** the process exit code.
- Every line is independently valid JSON with `schema`, `session_id` (u64),
  `seq`.
- Stream ending without a `result` ⇒ the process died; treat as failure.

### Integration tests (deterministic input → fixed JSON + exit code)

Run with `--backend static --deterministic` (timestamps pinned to `0`, `seq`
monotonic) so output is byte-stable.

1. **Happy path.** `caro --backend static --deterministic
   --output-format stream-json -p "list files"` ⇒ stdout is exactly three lines
   (`init`, `command`, `result`); `result.subtype=="success"`,
   `result.exit_code==0`; process exits `0`. Assert full NDJSON equality.
2. **Safety block.** A prompt resolving to a CRITICAL pattern (e.g. `rm -rf /`)
   ⇒ a `command` event with `safety_level=="critical"`, then
   `result.subtype=="blocked"`, `exit_code==3`; process exits `3`.
3. **Arg error.** `--output-format stream-jsom` (typo) ⇒ no NDJSON on stdout,
   error on stderr, exit `2`.
4. **No silent success (regression guard for failure mode #1).** Force an empty
   generation; assert the stream still ends with a `result` whose `subtype` is
   `error` and `exit_code==1` — never a `0` with no `result`.
5. **Resume skips re-init (regression guard for failure mode #2).** First call
   captures `session_id` (u64) + `init_digest` from `init`. Second call with
   `--resume <id>` in an unchanged environment ⇒ `init.warm_init_reused==true`,
   `init.resumed_from==<id>`, recomputed `init_digest` equals the first. With
   `--no-warm-init`, `warm_init_reused==false`.
6. **Schema gate.** Every emitted line has `schema=="caro.headless.v1"`; a
   snapshot test fails loudly if any field name changes (forces a version-bump
   conversation).
7. **(New) Session id round-trips as `u64`.** `init.session_id` parsed as a JSON
   number, fed to `--session-id`, resolves the same `AiSession` via
   `SessionStore::get(u64)` — guards the v1→v2 type correction.

### Explicitly out of scope (next version)

- **`stream-json` as *input*** (bidirectional / piped multi-turn). Output-only
  in v1 — input streaming is the part upstream still flags experimental.
- **Tool-use / `tool_use` events.** Caro generates commands; no agentic tool
  loop in headless mode in v1.
- **Token `usage` / `cost_usd` for remote backends.** Reserved (optional) in
  the envelope, not populated in v1.
- **Per-token streaming of command text.** v1 streams *events*, not tokens; the
  `command` event is atomic.
- **`--resume` across machines / session export-import.** Session files are
  local; portability is later.
- **MCP-server / plugin enumeration in `init`.** Reserved field; populated when
  the MCP safety server (ADR-015) integrates with headless mode.
- **Collapsing the duplicate `BackendInfo` types.** v1 simply pins
  `models::BackendInfo` on the wire; deduping the two definitions is its own
  cleanup PR (boy-scout, not headless-scoped).

---

## Constraint compliance check

| Constraint | How this scope satisfies it |
|---|---|
| Reuse existing validator/safety/config infra; don't duplicate | `CommandEvent` flattens `GeneratedCommand` + `SuggestedRouting`; safety verdict from the existing validator; format extends `OutputFormat`; sessions reuse `SessionStore`; hashing reuses `sha2`. |
| All new types serializable from day one | Every new type derives `Serialize, Deserialize`; `AiSession.init_digest` uses `#[serde(default)]` for backward compat. |
| Pure subprocess, no daemon, no state | No server/socket. State is the user-owned on-disk JSON session file. Warm-init is a content-addressed cache, not a process. |
| Solve the Phase-1 failure mode by design | (a) Silent success eliminated: exit code derived from the mandatory terminal `result` subtype — one code path. (b) Redundant init eliminated: `InitDigest` lets resume skip re-indexing/profiling and defer model load, no daemon. |

---

## Appendix — Citation verification (2026-06-16)

| Claim in scope | Verified? | Evidence |
|---|---|---|
| `OutputFormat { Json, Yaml, Plain }` + `FromStr` | ✅ | `src/cli/mod.rs:101` |
| `GeneratedCommand` 8 fields incl. `generation_time_ms`, `confidence_score` | ✅ | `src/models/mod.rs:72-96` |
| `SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }` | ✅ | `src/models/mod.rs:189` |
| `SessionStore::{open, resume_recent, create, upsert, get}` | ✅ | `src/ai/store.rs:55,70,80,89,102` |
| `sha2 = "0.10"` already a dependency | ✅ | `Cargo.toml:90` |
| Next ADR number is 024 | ✅ | `docs/adr/` highest is ADR-023 |
| **Session id type** | ❌→fixed | `AiSession.id: u64` (`src/ai/session.rs:62`), `get(id: u64)` — v1 said `String`; corrected to `u64` |
| **`BackendInfo` identity** | ❌→fixed | two defs: `src/backends/mod.rs:36` and `src/models/mod.rs:426` — pin `models::BackendInfo` on the wire |
