# ADR-026: Headless Multi-Turn Agentic Session — A Versioned Bidirectional NDJSON Conversation Contract

- **Status**: Proposed
- **Date**: 2026-06-26
- **Authors**: caro-research scoping process (automated scheduled run)
- **Target**: Hybrid (Community CLI + Enterprise CI / agent wrappers)
- **Builds on / relates to**: ADR-024 (Headless JSON/NDJSON output contract),
  ADR-025 (Headless init snapshot cache), ADR-003 (Structured output envelope &
  stateless session contract), ADR-015 (MCP safety server — the daemon
  complement), ADR-020 (Tiered Approval Protocol)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound. The two prior runs already scoped the headless
> **output contract** (ADR-024, 2026-06-23) and the **redundant-init cold-start**
> dimension (ADR-025, 2026-06-24); ADR-003 restated the envelope decision.
> Both ADR-024 and ADR-025 explicitly **defer the multi-turn agentic loop
> exposed headlessly to "a separate ADR."** To add value without duplicating,
> this run targets exactly that deferred dimension. The competitor analog
> remains Claude Code's headless mode — specifically its **`--input-format
> stream-json` bidirectional multi-turn input**, the one part of that feature
> that is still experimental and undocumented. Treat the analog choice as a
> reviewable assumption, not a settled decision.

---

## Context

### Phase 1 — what the analog does, and why it is still experimental

Claude Code's headless mode (`claude -p`) is single-shot by default: one prompt
in, one result out, exit. Its **only** programmatic mechanism for a *multi-turn*
conversation in print mode is `--input-format stream-json`: the caller pipes a
stream of newline-delimited JSON messages into stdin, simulating a continuous
conversation, and reads a `stream-json` event stream back. This is what agent
wrappers use to chain phases and pass an existing conversation to a fresh
instance.

The Phase-1 research surfaced that this is the **least-finished corner** of an
otherwise GA feature:

1. **Undocumented input contract.** `--input-format stream-json` "is the only
   CLI mechanism for programmatic bidirectional communication … However, there
   is no documentation explaining how to use it." The message shapes, how
   `max_turns` interacts with a piped stream, and the exact framing are
   community-reverse-engineered, not specified
   ([anthropics/claude-code#24594](https://github.com/anthropics/claude-code/issues/24594)).
2. **Version-coupled, unversioned shapes.** As with the output contract
   (ADR-024 §Context), the event/message shapes carry no schema version and
   drift across binary releases.
3. **Fuzzy process lifecycle.** A background Bash task spawned during a
   `claude -p` run "is terminated about five seconds after Claude has returned
   its final result and stdin has closed." The end of a conversation is not a
   clean, immediate process exit — there is a lingering-task window, and the
   stdin-close / final-result ordering is subtle
   ([Claude Code headless docs](https://code.claude.com/docs/en/headless)).
4. **No bound on turns by the input path.** A piped stream can drive an
   unbounded number of turns; `--max-turns` exists for the agent loop but its
   interaction with `stream-json` input is part of the undocumented surface.

### Who needs this in caro

ADR-024 made caro a machine-callable subprocess for **one** NL→command turn.
But the same three audiences it named routinely need **a short conversation**,
not a single turn:

1. **Agent frameworks** wrapping caro as a tool: "generate a command → that's
   not quite right, refine with this constraint → now adapt it for `zsh`."
   Today each of those is a separate `caro` process that re-inits from scratch
   and (without ADR-024's `--session`) forgets the prior turns.
2. **Multi-phase CI / scripted flows**: a generated remediation that is
   reviewed, then narrowed, then re-validated — a 2–3 turn exchange piped from a
   script.
3. **Interactive front-ends** (editor plugins, the planned shell widget) that
   want one long-lived caro invocation to handle a back-and-forth instead of
   re-spawning per keystroke-batch.

### What caro already has (do not rebuild)

The Phase-2 inventory found that caro's conversational primitive **already
exists**; what is missing is a *headless surface* for it.

| Capability | Exists today | File |
| --- | --- | --- |
| A single AI turn (gen + validate + persist) | ✅ `run_once(AiInvocation) -> Result<AiOutcome>` | `src/ai/runner.rs` |
| Session reconciliation modes | ✅ `SessionMode::{ResumeOrNew, New, ResumeStrict}` | `src/ai/runner.rs` |
| File-backed, serde-persisted session | ✅ `AiSession`, `Turn`, `Role` (all `Serialize`) | `src/ai/session.rs` |
| Session store with TTL resume | ✅ `SessionStore::{open, create, resume_recent, upsert}` | `src/ai/store.rs` |
| Iterative refinement loop | ✅ `AgentLoop` (`_max_iterations: 2`, confidence-gated refine) | `src/agent/mod.rs` |
| Per-turn result | ⚠️ `AiOutcome` — **`#[derive(Debug, Clone)]` only, NOT serializable** | `src/ai/runner.rs:54` |
| Headless envelope + exit codes | ✅ `HeadlessEnvelope`, `ExitCode`, `HeadlessEvent`, `HeadlessStatus` | `src/cli/mod.rs` (ADR-024) |
| Init amortization across processes | ✅ `InitSnapshot` content-addressed cache | `src/cache/init_snapshot.rs` (ADR-025) |
| serde / serde_json / schemars | ✅ | `Cargo.toml` |

So caro can already run a turn, persist it, and resume it. It cannot yet (a)
consume a **stream of user turns** in one subprocess, (b) emit a **versioned
multi-turn event contract**, or (c) serialize `AiOutcome` into that contract.

### The failure mode we must design around

> **An experimental, undocumented, unbounded, lingering bidirectional channel.**
> The analog's multi-turn input is the one part of headless mode that ships
> without a documented schema, without a turn bound on the input path, and with
> a non-deterministic end-of-conversation window (background tasks killed ~5 s
> after the fact). A naïve port would reproduce all four: scrape an
> ad-hoc message shape, loop forever on stdin, and leave the exit timing fuzzy.

ADR-024 solved *output* determinism; ADR-025 solved *init* cost. ADR-026 must
make the **multi-turn input channel** a fully-specified, versioned, bounded, and
cleanly-terminating contract **by construction** — and do it while staying a
pure subprocess with no resident state.

---

## Decision

Introduce a **Headless Conversation Mode**: when `--input-format ndjson` is
passed alongside `--output {json,ndjson}`, caro reads a stream of
newline-delimited **user-turn** events from stdin, runs each through the
existing `ai::runner::run_once` against a single in-process session, emits a
versioned event per turn, and — on stdin EOF, a stop event, a blocked turn, or a
hard `--max-turns` bound — emits exactly one terminal `result` event carrying a
`ConversationEnvelope`, then exits with an ADR-024 exit code. No daemon, no
resident state, no lingering tasks.

Five design commitments, each closing a Phase-1 failure mode or a constraint by
construction:

### 1. The input contract is a versioned, documented type — not a scraped shape

The bidirectional channel is defined by a single serde enum, `HeadlessInputEvent`
(below), published in `docs/headless-contract.md` with its JSON Schema (via the
existing `schemars` derive). This is the direct answer to
[claude-code#24594](https://github.com/anthropics/claude-code/issues/24594): the
*input* side is specified from day one, not reverse-engineered. Malformed input
fails loud with exit `2` / `error.kind = "usage"` — never silently skipped.

### 2. One init, N turns ⇒ redundant initialization avoided *within* a conversation

This is ADR-026's distinct lifecycle contribution, complementary to ADR-024 and
ADR-025:

- **ADR-024 `--session <file>`** carries continuity across *separate* subprocess
  invocations — but pays one init **per call**.
- **ADR-025 init cache** amortizes that per-call init across *processes* via a
  content-addressed snapshot.
- **ADR-026 conversation mode** runs N turns inside **one** subprocess: platform
  detection, safety-pattern compilation, backend availability, and context
  gathering happen **once for the whole conversation**, and every subsequent
  turn reuses the in-memory `ExecutionContext`, `Arc<SafetyValidator>`, and
  `Arc<dyn CommandGenerator>`. This is the literal answer to the task's
  "how does it avoid redundant initialization?" — amortize across turns in one
  process, while keeping zero resident state *between* conversations.

### 3. Hard turn bound ⇒ no unbounded loop

`--max-turns <n>` (default = `AgentLoop`'s existing iteration budget surfaced as
a constant, currently `2`; configurable, `0` ≡ unlimited only when explicitly
set) bounds the input path. Reaching it ends the conversation cleanly with
`stop_reason = "max_turns_reached"` and exit `0` (every executed turn
succeeded). The bound is in the contract, not an undocumented flag interaction.

### 4. Deterministic, immediate exit ⇒ no lingering-task window

caro's headless conversation **generates**; it does not execute intermediate
turns. The only execution is the final turn under explicit `--execute`, which is
bounded and fully reported in the envelope (ADR-024 `ExecutionReport`). There is
no agent-spawned background task to outlive the run: the subprocess reads until
EOF/stop/bound, emits the terminal `result` event, flushes, and exits. The
"5-second background-task linger" foot-gun is structurally absent.

### 5. Reuse, do not rebuild

Per-turn generation + validation + persistence is `ai::runner::run_once`,
unchanged. Session state is the existing file-backed `AiSession` via
`SessionStore`. Safety verdicts are the existing `SafetyValidator`. The output
envelope, event enum, and exit codes are ADR-024's types, *extended additively*
under the same `schema_version: "1"`. The only new behavior is the driver loop
and the input-event type; everything else is composition.

---

## New types

All contract-facing types live in **`src/cli/mod.rs`**, next to ADR-024's
`HeadlessEnvelope` / `HeadlessEvent` / `ExitCode`. No new module. Every type
derives `#[derive(Debug, Clone, Serialize, Deserialize)]` from day one;
contract-facing types additionally derive `schemars::JsonSchema`.

### `HeadlessInputEvent` (the stdin contract — new)

```rust
/// One newline-delimited input line in conversation mode (`--input-format ndjson`).
/// The contract is published with its JSON Schema in docs/headless-contract.md.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HeadlessInputEvent {
    /// A user turn. `prompt` is the natural-language request.
    User { prompt: String },
    /// Optional graceful end-of-conversation marker (EOF has the same effect).
    Stop,
}
```

### `ConversationEnvelope` (the terminal `result` payload — new)

```rust
/// The single, versioned object that ends a headless conversation. Reuses
/// ADR-024's schema_version line and exit_code mirroring; reuses RiskLevel.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConversationEnvelope {
    /// Contract version. Shares ADR-024's "1"; additive-only within a version.
    pub schema_version: String,
    /// Stable session handle (reused from AiSession.id) — first-class, never scraped.
    pub session_id: u64,
    /// One summary per turn actually run, in order.
    pub turns: Vec<TurnSummary>,
    /// Number of assistant turns produced (== turns.len() of role==assistant).
    pub num_turns: u32,
    /// Why the conversation ended.
    pub stop_reason: StopReason,
    /// Mirrors the process exit code so payload and exit can never disagree.
    pub exit_code: i32,
    /// Additive ADR-025 init-cache observability; omitted when disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<InitCacheStatus>,
    /// Present iff stop_reason == Error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<HeadlessError>,
}
```

### `TurnSummary` (new)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TurnSummary {
    pub index: u32,
    /// "user" | "assistant" — reuses ai::session::Role's serde repr.
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,
    /// False ⇒ this turn was blocked by the safety validator.
    pub allowed: bool,
    pub warnings: Vec<String>,
}
```

### `StopReason` (new)

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    Completed,        // stdin EOF or Stop event after >=1 successful turn
    MaxTurnsReached,  // hit --max-turns
    Blocked,          // a turn was blocked by safety; conversation halts
    Error,            // backend/internal failure mid-conversation
}
```

### Output events — extend ADR-024's `HeadlessEvent` (additive)

Two new variants are added to the existing `HeadlessEvent` enum (ADR-024); the
`Init` and `Result` variants gain additive fields. No existing variant changes.

```rust
// added variants:
UserTurn { index: u32, prompt: String },
AssistantTurn {
    index: u32, command: String, confidence: Option<f64>,
    risk_level: RiskLevel, allowed: bool,
},
// Init gains: max_turns: u32, session_id: u64
// Result.payload becomes an untagged union of HeadlessEnvelope (single-shot,
//   ADR-024) | ConversationEnvelope (conversation mode), distinguished by the
//   presence of `turns` — both share schema_version + exit_code.
```

### Make `AiOutcome` serializable (the "serializable from day one" fix)

`AiOutcome` (`src/ai/runner.rs:54`) is currently `#[derive(Debug, Clone)]`. Add
`Serialize, Deserialize` — the same move ADR-025 made for `PlatformContext` and
ADR-003 proposed for `AiOutcome`. Its fields (`session_id`, `command`,
`explanation`, `confidence`, `risk: RiskLevel`, `warnings`, `allowed`,
`resumed`, `warns_offhost`) are all already serializable.

### Method contracts

In `src/ai/runner.rs`:

- `AiOutcome::to_turn_summary(&self, index: u32) -> TurnSummary` — pure mapping;
  the *only* place the per-turn projection lives, so `AiOutcome` can evolve
  internally without breaking the wire shape.

In `src/cli/mod.rs`:

- `HeadlessInputEvent::parse_line(line: &str) -> Result<HeadlessInputEvent, HeadlessError>`
  — `serde_json::from_str`; on parse error returns `HeadlessError { kind:
  "usage", .. }`. The only input parser; bounded per-line (see contract).
- `ConversationEnvelope::from_turns(session_id: u64, turns: Vec<TurnSummary>, stop: StopReason, code: ExitCode) -> Self`
  — assembles the terminal payload; sets `num_turns` and `exit_code` from inputs
  so they cannot drift.
- `ConversationEnvelope::error(session_id, kind, msg, code: ExitCode) -> Self`.
- `ConversationEnvelope::render(&self, ndjson: bool) -> String` — `serde_json`;
  for NDJSON wraps in the `HeadlessEvent::Result` line.

In `src/main.rs` (driver, not a type):

- `async fn run_headless_conversation(cli, ctx, validator, backend, max_turns) -> ExitCode`
  — opens **one** `SessionStore`, creates the session once, then loops:
  read line → `HeadlessInputEvent::parse_line` → on `User`, call
  `ai::runner::run_once` with `SessionMode::ResumeStrict` (after the first turn
  uses `New`/`ResumeOrNew`) reusing the in-memory ctx/validator/backend → emit
  `UserTurn` + `AssistantTurn` events → break on blocked turn / `Stop` / EOF /
  `max_turns`. Emits one `Result` event, returns the `ExitCode`.

---

## Minimal set of files that change

No new modules. Six existing files + one new test file:

1. **`src/ai/runner.rs`** — derive `Serialize, Deserialize` on `AiOutcome`; add
   `to_turn_summary`. (No change to `run_once`'s logic.)
2. **`src/cli/mod.rs`** — add `HeadlessInputEvent`, `ConversationEnvelope`,
   `TurnSummary`, `StopReason`; add the two `HeadlessEvent` variants and the two
   additive `Init`/`Result` fields; re-export `Role` for `TurnSummary`.
3. **`src/main.rs`** — add `--input-format <ndjson>` and `--max-turns <n>` flags
   to `Cli`; add `run_headless_conversation`; route to it when `--input-format
   ndjson` is set with `--output {json,ndjson}`. Reuse ADR-024 `ExitCode`,
   ADR-024's bare-by-default config skip, and ADR-025's `warm_or_cold` init.
4. **`src/ai/store.rs`** — only if not already present: confirm
   `SessionStore::open`/`create`/`upsert` are usable with an explicit store path
   (they are, per `AiInvocation.store_path`). No change expected.
5. **`docs/headless-contract.md`** *(extends the ADR-024 doc)* — publish the
   `HeadlessInputEvent` schema, the conversation event taxonomy, `--max-turns`,
   `StopReason` values, the per-line and total stdin caps, and the
   single-init / clean-exit lifecycle guarantee.
6. **`docs/adr/README.md`** — add the ADR-026 table row.
7. **`tests/headless_conversation.rs`** *(new integration test file)* — below.

Reused **without modification**: `ai::runner::run_once`, `AiInvocation`,
`SessionMode`, `AiSession`/`Turn`/`Role`, `SessionStore`, `SafetyValidator`,
`RiskLevel`, ADR-024's `HeadlessEnvelope`/`ExitCode`/`HeadlessStatus`/
`HeadlessError`/`ExecutionReport`, ADR-025's `InitSnapshot`/`InitCacheStatus`,
and the `StaticMatcher` backend (for deterministic, offline tests).

---

## Exit-code / output contract (what machines depend on)

**No new exit codes** — the conversation reuses ADR-024's frozen 0–6 table, so
existing consumers' branching is unchanged:

| Exit | Conversation meaning | `stop_reason` |
| --- | --- | --- |
| 0 | all turns generated OK; ended on EOF/Stop/max-turns | `completed` / `max_turns_reached` |
| 1 | internal/unexpected error mid-conversation | `error` |
| 2 | usage error: malformed input line, bad flag combo, stdin over cap | `error` |
| 3 | a turn was blocked by the safety validator; conversation halted | `blocked` |
| 5 | no backend reachable | `error` |
| 6 | final turn ran under `--execute` and exited non-zero | `completed` + `execution.exit_code != 0` |

Stability promises (documented in `docs/headless-contract.md`, additive to
ADR-024):

- `--output json` prints exactly one `ConversationEnvelope` to stdout at end of
  conversation; `--output ndjson` prints the event stream with a single terminal
  `Result` line whose `payload` is that same envelope.
- `schema_version` stays `"1"`; `turns`, `num_turns`, `stop_reason`, and the new
  events are *added* fields/variants — within ADR-024's additive-only rule.
- All human/diagnostic text goes to stderr; stdout is pure contract.
- **Per-line input cap 64 KiB; total stdin cap 1 MiB** (reuses ADR-024's cap
  lesson). Exceeding either exits `2` with `error.kind = "stdin_too_large"` —
  fail loud, never truncate.
- The process exits within a bounded time of the terminal `Result` event with no
  retained tasks; stdin EOF is honored immediately (no linger window).

---

## Integration tests (deterministic input → fixed JSON + exit code)

In `tests/headless_conversation.rs`, driven through the `static` backend
(offline, deterministic; no model download), `$XDG_CACHE_HOME` and the session
store pointed at a `tempfile::TempDir`. Input is piped as NDJSON to stdin.

1. **Two-turn happy path.** Pipe `{"type":"user","prompt":"list files"}` then
   `{"type":"user","prompt":"now sorted by size"}`, EOF.
   → exit `0`, `stop_reason:"completed"`, `num_turns:2`, two `AssistantTurn`
   events, both `allowed:true`, stable `session_id`. Assert field-by-field via
   `serde_json::from_str` (additive-safe).
2. **Max-turns bound.** Pipe 5 user events with `--max-turns 2`.
   → exit `0`, `num_turns:2`, `stop_reason:"max_turns_reached"`, exactly two
   `AssistantTurn` events; turns 3–5 are not run.
3. **Blocked mid-conversation.** Turn 1 safe, turn 2 a CRITICAL prompt.
   → exit `3`, `stop_reason:"blocked"`, `turns[1].allowed:false`,
   `turns[1].risk_level:"critical"`; no turn 3 even if piped.
4. **Malformed input line.** Pipe `{ not json`.
   → exit `2`, `stop_reason:"error"`, `error.kind:"usage"`.
5. **Single-init guarantee (the lifecycle test).** Run a 3-turn conversation
   and assert the ADR-025 `cache.source` is observed exactly once as `cold`
   (init ran once for the whole conversation), not once per turn — proving
   redundant init is amortized across turns in one process.
6. **Determinism.** Pipe identical NDJSON twice; assert the two event streams
   are byte-identical except timestamps/timings, and the two
   `ConversationEnvelope`s deep-equal modulo the `timing`/`cache` fields. With a
   populated fake `$XDG_CONFIG_HOME/caro/config.toml` present, output is
   unchanged (proves ADR-024 bare-by-default still holds in conversation mode).
7. **Clean exit / no hang (anti-#24594, anti-linger).** Close stdin after the
   final user event; assert the process exits within a bounded timeout and emits
   exactly one terminal `Result` event (no second `Result`, no hang waiting on
   stdin).
8. **Round-trip.** `serde_json::to_string` → `from_str` on a `ConversationEnvelope`
   and on each `HeadlessInputEvent` variant — proves serializable-from-day-one.

Every test asserts both the parsed payload **and** the process exit code, so the
two can never drift.

---

## Out of scope (next version / other ADRs)

- **Tool-use / auto-execution agentic loop** — where caro executes an
  intermediate command and feeds its stdout/stderr back as the next turn's
  context, looping until a goal is met. v1 *generates* per turn; the operator
  (or the calling agent) decides what to execute. Auto-execution is a much
  larger safety surface (re-validation per executed step, sandboxing per
  ADR-010, attribution per ADR-021) and warrants its own ADR.
- **Token-level / partial-message streaming within a turn**
  (`--include-partial-messages`). Already deferred by ADR-024; unchanged here.
  v1 conversation events are turn-granular.
- **`--json-schema` enforced output** (constraining a turn's command to a
  caller-supplied schema). Already deferred by ADR-024.
- **A conversation that outlives a single subprocess with an in-memory warm
  model** across turns. That is a daemon = forbidden. Cross-*process* continuity
  is already covered by ADR-024's `--session` file; cross-process init cost by
  ADR-025. Keeping the *model weights* warm between processes remains MCP-server
  (ADR-015) territory.
- **Concurrent / interleaved turns, branching conversations, cost accounting.**
  v1 is strictly sequential, single-branch; `num_turns` is the only counter.

---

## Rationale

A multi-turn surface is the natural next step after ADR-024's single-turn
contract, and it is the dimension both prior ADRs explicitly punted on. Doing it
as a **bidirectional NDJSON channel over one subprocess** is what lets caro
match the *capability* of Claude Code's `stream-json` input while fixing the
*four things* that make that input experimental: it ships documented, versioned,
bounded, and cleanly-terminating. It also turns the task's "avoid redundant
initialization" requirement into a concrete, testable property — one init per
conversation — that neither ADR-024 nor ADR-025 delivers on its own.

## Consequences

### Benefits

- Agent wrappers get true multi-turn caro from a single subprocess spawn —
  fewer process launches, one init, in-memory context reuse across turns.
- The input channel is specified and schema-published from day one, the exact
  gap ([#24594](https://github.com/anthropics/claude-code/issues/24594)) that
  makes the analog's multi-turn input hard to consume.
- Zero new exit codes and additive-only output fields: existing ADR-024
  consumers keep working untouched.
- Net new code is small: ~4 contract types + 2 event variants + one driver loop
  + 1 derive change; everything substantive is composed from `run_once`,
  `SessionStore`, and the safety validator.
- `--max-turns` and the immediate-exit guarantee remove the unbounded-loop and
  lingering-task foot-guns by construction.

### Trade-offs

- One subprocess now has a longer lifetime (the whole conversation) than a
  single-shot call; the driver loop must handle partial stdin and per-line caps
  carefully (mitigated by the 64 KiB/1 MiB caps and bounded reads).
- `AiOutcome` gaining `Serialize` couples its field names to a wire projection;
  isolated by `to_turn_summary` so the coupling is one function, not the API.
- `HeadlessEvent::Result.payload` becoming a union of `HeadlessEnvelope` |
  `ConversationEnvelope` adds a (documented) discriminator (`turns` present);
  consumers of single-shot output are unaffected.

### Risks

- **Risk:** a slow or stalled stdin producer holds the subprocess open. →
  **Mitigation:** honor stdin EOF immediately; no read blocks past the declared
  caps; document that the caller owns closing stdin (test 7 asserts the
  no-hang behavior).
- **Risk:** state accumulation in a very long conversation (memory growth in
  `AiSession.turns`). → **Mitigation:** `--max-turns` bound; the session is the
  same bounded file structure already used interactively.
- **Risk:** contract drift between single-shot and conversation envelopes. →
  **Mitigation:** both share `schema_version` + `exit_code`; a `schemars`
  golden-schema CI check (recommended in ADR-024) covers both.

## Alternatives Considered

### Alternative 1: One subprocess per turn + ADR-024 `--session` file
- Description: Skip conversation mode; tell callers to loop `caro --session f.json`.
- Pros: Zero new code; ADR-024 already ships it.
- Cons: Pays one init **per turn** (ADR-025 softens but does not eliminate it),
  re-spawns a process per turn, and never amortizes context in memory. Fails the
  task's "avoid redundant initialization" emphasis for the multi-turn case.

### Alternative 2: Copy Claude Code's `stream-json` input verbatim, document later
- Description: Accept the same undocumented message shapes for compatibility.
- Pros: Drop-in for existing `claude -p` wrappers.
- Cons: Reproduces the exact bug-then-patch path — unversioned, undocumented,
  unbounded. Violates "solve the failure mode by design, not by workaround."

### Alternative 3: A resident `caro serve` conversation daemon
- Description: Keep a process warm holding the session and model in memory.
- Pros: Cheapest possible multi-turn; true warm context.
- Cons: Violates "pure subprocess, no daemon, no state." Daemon territory belongs
  to the MCP safety server (ADR-015). Rejected, consistent with ADR-024/025.

### Alternative 4: A new `src/conversation/` module
- Description: House the loop and types in a dedicated module.
- Cons: Unnecessary. Types belong next to ADR-024's in `cli`; the loop is a thin
  driver in `main.rs`. "No new modules unless unavoidable."

## Implementation Notes

- Land behind the existing headless surface; conversation mode activates only on
  `--input-format ndjson`, so default behavior is untouched.
- Reuse ADR-025's `warm_or_cold` for the single init at conversation start.
- First turn uses `SessionMode::New` (fresh conversation) unless `--session` is
  also passed (then `ResumeOrNew`); subsequent turns reuse the in-memory session
  and `SessionMode::ResumeStrict` semantics against the same store path.
- Enforce the per-line and total stdin caps in the read loop before parsing.
- Follow `.claude/rules/git-workflow.md`: land on a feature branch via a PR; do
  not commit to `main`.

## Success Metrics

- A 3-turn conversation completes in **one** subprocess with **one** init
  (test 5 green).
- `tests/headless_conversation.rs` (8 cases) green on macOS + Linux CI with the
  static backend, fully offline.
- Published `HeadlessInputEvent` JSON Schema validates against the schemars
  golden check.
- An external agent wrapper can drive a multi-turn caro exchange parsing only
  the documented events — zero stdout text scraping.

## Business Implications

- **Differentiation**: caro offers an offline, deterministic, *documented*
  multi-turn agent channel — the part of the analog's headless mode that is
  still experimental. Strong story for agent-framework and CI integrators.
- **Customer pain addressed**: per-turn process re-spawn cost and the inability
  to hold a short caro conversation from a wrapper without text-scraping.
- **Sales enablement**: "multi-turn, single-init, no daemon, versioned contract"
  is a concrete, demoable claim distinct from wrapping `claude -p`.

## References

- [Claude Code — Run Claude Code programmatically (headless)](https://code.claude.com/docs/en/headless)
  — `--input-format stream-json`, `--output-format stream-json`, `--max-turns`,
  background-task termination behavior.
- [anthropics/claude-code#24594 — `--input-format stream-json` is undocumented](https://github.com/anthropics/claude-code/issues/24594)
  — the Phase-1 failure mode this ADR designs around.
- ADR-024 (Headless JSON/NDJSON output contract) — the envelope, events, and
  exit codes extended here.
- ADR-025 (Headless init snapshot cache) — cross-process init amortization, used
  for the single conversation-start init.
- ADR-003 (Structured output envelope & stateless session contract) — prior
  framing; proposed deriving `Serialize` on `AiOutcome`.
- ADR-015 (MCP safety server) — the resident-process complement; daemon concerns
  live there.
- `src/ai/runner.rs` (`run_once`, `AiInvocation`, `SessionMode`, `AiOutcome`),
  `src/ai/session.rs` (`AiSession`, `Turn`, `Role`), `src/ai/store.rs`
  (`SessionStore`), `src/agent/mod.rs` (`AgentLoop`) — the reused primitives.
- `.claude/rules/adr-numbering.md` — sequential numbering (this is ADR-026,
  following ADR-025).
- `.claude/rules/validation-discipline.md` — this ADR refines an already-GA path
  (headless mode) rather than launching a new user-facing product line, so the
  five discovery gates do not gate it (per that rule's "what this rule does NOT
  do" clause).

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-26 | caro-research--scoping-process (automated) | Initial draft (ADR-026) |
