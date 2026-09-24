# ADR-028: Headless Token Streaming — A Versioned, Backend-Agnostic, Advisory-Only Partial-Output Event Contract

- **Status**: Proposed
- **Date**: 2026-06-30
- **Authors**: caro-research scoping process (automated scheduled run)
- **Target**: Hybrid (Community CLI + Enterprise CI / agent wrappers / editor front-ends)
- **Builds on / relates to**: ADR-024 (Headless JSON/NDJSON output contract),
  ADR-025 (Headless init snapshot cache), ADR-026 (Headless multi-turn agentic
  NDJSON session), ADR-027 (Headless permission resolution contract), ADR-015
  (MCP safety server — the daemon complement)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. The four prior runs
> scoped the headless **output contract** (ADR-024, 2026-06-23), the
> **redundant-init cold-start** dimension (ADR-025, 2026-06-24), the
> **multi-turn bidirectional NDJSON conversation** (ADR-026, 2026-06-26), and
> the **permission-resolution** surface (ADR-027, 2026-06-29). Both ADR-024
> and ADR-026 explicitly defer **token-level / partial-message streaming**
> (Claude Code's `--include-partial-messages`) to "a separate ADR / v2" — it is
> the single most-repeatedly-punted dimension of the analog and remains its
> least-finished corner. To add value without duplicating, this run targets
> exactly that deferred dimension. The competitor analog remains Claude Code's
> headless mode — specifically `--output-format stream-json --verbose
> --include-partial-messages`, the token-delta path. Treat the analog choice as
> a reviewable assumption, not a settled decision.

---

## Context

### Phase 1 — what the analog does, and why it is still rough

Claude Code's headless mode can stream model output token-by-token. The path is
`claude -p "…" --output-format stream-json --verbose --include-partial-messages`:
every line is a JSON event, and with partial messages enabled the stream carries
`stream_event` objects whose `event.delta.type == "text_delta"` deliver tokens
as they are generated. A consumer filters those deltas (the docs show a `jq`
one-liner) to render text as it streams
([Claude Code headless docs](https://code.claude.com/docs/en/headless)).

Phase-1 research surfaced four things that make this the **least-finished
corner** of an otherwise-GA feature:

1. **Three-flag friction.** Token streaming requires `--output-format
   stream-json` **and** `--verbose` **and** `--include-partial-messages`
   together. Two of the three are about *verbosity plumbing*, not intent; miss
   one and you silently get no deltas. The activation contract is awkward and
   easy to get wrong.
2. **Provider-coupled, unversioned delta shapes.** The streamed objects are raw
   Anthropic Messages-API `stream_event` payloads (`event.delta.text_delta`,
   `content_block_delta`, …). The wire shape is the *provider's* streaming
   schema, carries no Claude-Code-level `schema_version`, and drifts when the
   underlying API event taxonomy changes — the same version-coupling root cause
   ADR-024 §Context identified for the result payload.
3. **Deltas are not the source of truth, but nothing says so.** The authoritative
   answer is the final `result` payload; the partial deltas are advisory and may
   not concatenate to the final text byte-for-byte (retries, redaction, post-hoc
   formatting). A consumer that reconstructs the answer from deltas can diverge
   from the official result, and the contract does not state this invariant.
4. **Capability is implicit.** Whether tokens will actually stream depends on the
   model/provider; there is no single up-front field telling a machine "expect
   fine-grained deltas" vs "expect one coarse chunk." The consumer discovers it
   empirically.

### Who needs this in caro

ADR-024 made caro a machine-callable subprocess emitting a versioned envelope;
ADR-026 added a multi-turn NDJSON stream. Both emit **turn-granular** events: a
command appears whole, only once generation finishes. Three audiences want
**intra-turn** progress:

1. **Editor / IDE front-ends and the planned shell widget** (named as a target
   in ADR-026): render the generated command *as it is produced*, so a
   multi-hundred-millisecond local-model generation feels live instead of
   frozen.
2. **Interactive wrappers** that show a spinner today and want a "typing"
   effect, or that want to start syntax-highlighting / previewing the command
   before generation completes.
3. **Long-generation observability**: a CI or agent caller that wants a
   heartbeat proving the local model is making progress on a slow box, rather
   than waiting blind for the final envelope.

### What caro already has (do not rebuild)

The Phase-2 inventory found the plumbing is partly present; what is missing is a
**streaming path through the trait and a typed delta event**.

| Capability | State today | Location |
| --- | --- | --- |
| Backend trait | ✅ `CommandGenerator::{generate_command, is_available, backend_info, shutdown}` — **no streaming method** | `src/backends/mod.rs` |
| Streaming-capability flag | ✅ `BackendInfo.supports_streaming: bool` — but `false` for embedded + static + most remote; `true` only for `exo` | `src/backends/mod.rs:39`, `embedded_backend.rs:532`, `remote/exo.rs:439` |
| Whole-command result | ✅ `GeneratedCommand { command, explanation, … }` (serde) | `src/models/mod.rs` |
| NDJSON event enum + `Init`/`Result` | ✅ (proposed) `HeadlessEvent`, `Ndjson` output variant | ADR-024, `src/cli/mod.rs` |
| Streaming HTTP + futures | ✅ `reqwest` `["stream"]` feature; `futures = "0.3"` | `Cargo.toml:49,102` |
| Remote backends already speak SSE | ⚠️ have a `stream: bool` request field, hardcoded `stream: false` | `remote/{ollama,vllm,exo,openrouter,claude}.rs` |
| serde / serde_json / schemars | ✅ | `Cargo.toml` |

So caro can express *whether* a backend streams (`supports_streaming`) but has
**no method to actually emit tokens** and **no event to carry them**. No backend
streams tokens to the user today (the embedded backend reports
`supports_streaming: false`). This is the gap.

### The failure mode we must design around

> **An awkward-to-activate, provider-coupled, unversioned, source-of-truth-ambiguous
> token stream that only some backends can produce.** A naïve port reproduces all
> four Phase-1 problems: a three-flag activation, raw provider delta shapes on
> the wire, no statement that deltas are advisory, and per-backend behavior the
> consumer must reverse-engineer.

ADR-028 must make partial output a **single-flag, typed, versioned,
advisory-only, capability-declared** stream that behaves **identically to a
consumer regardless of backend** — and do it as a pure subprocess with no new
exit codes and a final envelope byte-identical to non-streaming mode.

---

## Decision

Introduce **Headless Token Streaming**: when `--include-partial-output` is passed
alongside `--output ndjson`, caro emits additive `output_delta` events carrying
the command text as it is generated, then the existing terminal `result` event
(ADR-024). The deltas are **advisory display-only**; the `result` envelope
remains the single source of truth and the only thing the exit code derives from.

Five design commitments, each closing a Phase-1 failure mode or a constraint by
construction:

### 1. One flag, gated on the stream — no verbosity plumbing

Activation is exactly one intent flag, `--include-partial-output`, valid **only**
with `--output ndjson` (deltas need a line stream). There is no `--verbose`
coupling: caro's NDJSON is already structured, so partial output is just "emit
more events on the stream you already asked for." Passing
`--include-partial-output` with `--output json` (single object, no stream) or
with `--output plain` is a **usage error** (exit `2`, `error.kind = "usage"`) —
fail loud, never silently drop deltas (the exact silent-no-deltas trap of the
analog's three-flag requirement).

### 2. A typed, versioned delta — not a raw provider shape

The wire carries caro's own `HeadlessEvent::OutputDelta { index, text }`, a
stable typed variant under ADR-024's `schema_version`, **not** the backend's
native streaming objects (Anthropic `text_delta`, Ollama NDJSON chunks, vLLM
SSE, …). Each backend's provider-specific stream is adapted *inside the backend*
into caro's neutral `&str` deltas. The consumer parses one shape forever; a
provider changing its SSE taxonomy is absorbed in the backend adapter, never on
caro's contract. This is the direct fix for the analog's provider-coupling.

### 3. Advisory-only ⇒ the envelope stays authoritative (the load-bearing property)

`output_delta` events are **display-only**. The authoritative command, safety
verdict, permission outcome (ADR-027), and exit code come **solely** from the
final `result` envelope. The contract states explicitly: *the concatenation of
`output_delta.text` is not guaranteed to equal `result.payload.command`* — a
post-generation safety rewrite, trimming, or backend canonicalization may differ.
This mirrors ADR-025's "optimization-only, never on the correctness path": just
as the init cache can never change output, partial deltas can never change the
contract. A consumer that ignores deltas entirely gets the byte-identical
ADR-024 result. (Enforced by tests 3 and 5.)

### 4. Capability declared up front in the `init` event

The NDJSON `Init` event gains a `streaming: bool` field, populated from the
active backend's `BackendInfo.supports_streaming`. A consumer reads it on the
first line and knows whether to expect fine-grained deltas (`true`) or exactly
one coarse whole-command delta (`false`). No empirical discovery. This is the
fix for the analog's implicit capability.

### 5. Graceful degradation by default ⇒ identical consumer code on every backend

`CommandGenerator` gains **one** new method, `generate_command_streaming`, with a
**default implementation** that calls the existing `generate_command` and emits
the whole command as a single terminal delta. Non-streaming backends (static,
embedded-today, ollama, vllm, openrouter) need **zero** changes and still produce
a well-formed `output_delta` stream — exactly one delta whose `text` is the whole
command. Streaming-capable backends (embedded once its decode loop is wired; exo)
override the method to emit per-token deltas. Either way the consumer parses an
identical event sequence: `Init` → one-or-more `OutputDelta` → `Result`. Backend
heterogeneity is invisible to the wire.

---

## New types (added to existing modules — no new `src` module)

All contract-facing types live in **`src/cli/mod.rs`**, beside ADR-024's
`HeadlessEvent` / `HeadlessEnvelope` / `ExitCode`. The one new trait method lives
on the existing `CommandGenerator` in `src/backends/mod.rs`. Every new/edited
type derives `#[derive(Debug, Clone, Serialize, Deserialize)]` from day one;
contract-facing ones additionally derive `schemars::JsonSchema`. **No new
dependency** (`futures` and `reqwest[stream]` already present).

### Extend ADR-024's `HeadlessEvent` (additive variant + additive `Init` field)

```rust
// New variant on the existing HeadlessEvent enum (src/cli/mod.rs).
// Emitted only when --include-partial-output is set with --output ndjson.
OutputDelta {
    /// Monotonic 0-based index of this delta within the current turn.
    index: u32,
    /// A neutral text fragment of the command-being-generated. ADVISORY:
    /// concatenation is NOT guaranteed to equal the final result.command.
    text: String,
},

// The existing Init variant gains one additive field:
//   streaming: bool   // == active backend BackendInfo.supports_streaming
```

No existing variant changes shape; `OutputDelta` is purely additive, and an
`Init` gaining an optional-by-convention bool is permitted under ADR-024's
additive-only rule within `schema_version: "1"`.

### New trait method on `CommandGenerator` (`src/backends/mod.rs`)

```rust
/// Generate a command while emitting command-text deltas through `on_delta`.
///
/// Default implementation degrades gracefully: it produces the whole command
/// via `generate_command` and emits it as a single delta. Streaming backends
/// override to call `on_delta` per decoded token.
///
/// `on_delta` is display-only; the returned GeneratedCommand is authoritative.
async fn generate_command_streaming(
    &self,
    request: &CommandRequest,
    on_delta: &mut (dyn FnMut(&str) + Send),
) -> Result<GeneratedCommand, GeneratorError> {
    let cmd = self.generate_command(request).await?;
    on_delta(&cmd.command);
    Ok(cmd)
}
```

The `&mut dyn FnMut(&str) + Send` sink keeps the trait object-safe and avoids a
new associated `Stream` type or generic — the driver owns the closure that turns
each `&str` into a numbered `OutputDelta` line. This is the minimal additive
surface: callers that never stream are entirely unaffected (default method,
never invoked unless `--include-partial-output`).

### Method contracts

In `src/cli/mod.rs`:

- `HeadlessEvent::output_delta(index: u32, text: impl Into<String>) -> Self` —
  the only constructor for a delta line; bounds nothing else.
- `HeadlessEvent::render_line(&self) -> String` — already exists per ADR-024
  (one `serde_json` line); reused unchanged for the new variant.

In `src/backends/embedded/embedded_backend.rs`:

- Override `generate_command_streaming` to invoke `on_delta` from inside the
  existing decode loop (the token loop behind `generate_command`, ~L446), and
  flip `BackendInfo.supports_streaming` to `true` **only** once the override is
  wired. Until wired, the default method + `supports_streaming: false` apply —
  honest capability reporting, no contract change.

In `src/backends/remote/exo.rs` (already `supports_streaming: true`):

- Override to consume the provider SSE (`reqwest` `stream` feature, already a
  dep; flip the request's existing `stream: bool` to `true`) and forward each
  chunk's text through `on_delta`. Other remote backends keep the default until
  individually wired.

In `src/main.rs` (driver, not a type):

- `async fn run_streaming_turn(... on_delta closure ...)` — when
  `--include-partial-output` && `--output ndjson`, build a closure
  `|frag| emit(HeadlessEvent::output_delta(next_index, frag))` and call
  `backend.generate_command_streaming(req, &mut closure)`; then emit the ADR-024
  `Result`. Otherwise call the non-streaming `generate_command` path unchanged.

---

## Exit-code / output contract (what machines depend on)

**No new exit codes.** Streaming is display-only; the process exits with the
exact ADR-024 / ADR-027 code derived from the final envelope. The only contract
additions are events and one flag:

| Surface | Value | Notes |
| --- | --- | --- |
| `--include-partial-output` | flag | Valid only with `--output ndjson`. With `--output json`/`plain` → exit `2`, `error.kind = "usage"`. |
| `init.streaming` | bool | `true` ⇒ expect token-granular deltas; `false` ⇒ expect exactly one whole-command delta. |
| `output_delta` event | `{type, index, text}` | Zero-or-more per turn, strictly increasing `index`, between `init` and `result`. Advisory/display-only. |

Stability promises (documented in `docs/headless-contract.md`, additive to
ADR-024/025/026):

- `schema_version` stays `"1"`; `output_delta` and `init.streaming` are *added*
  surface under the additive-only rule. Existing single-shot and conversation
  consumers are unaffected (they never set the flag).
- **Authoritative-source invariant:** the final `result.payload` is the only
  source of truth. `concat(output_delta.text)` MAY differ from
  `result.payload.command`; consumers MUST take the command/verdict/exit from the
  envelope, never from reconstructed deltas.
- Deltas are command-text only in v1 (no explanation/reasoning deltas).
- All deltas go to **stdout** on the NDJSON stream; human/diagnostic text stays
  on stderr (unchanged from ADR-024).
- A non-streaming backend still emits a well-formed stream: exactly one
  `output_delta` (whole command) then `result`.

---

## Minimal set of files to change (no new `src` modules)

No new modules. The trait method, one event variant, one `Init` field, one flag,
one driver path, plus doc + test:

1. **`src/backends/mod.rs`** — add `generate_command_streaming` (default impl) to
   the `CommandGenerator` trait. The only trait change; default keeps every
   existing backend compiling untouched.
2. **`src/backends/embedded/embedded_backend.rs`** — override the new method from
   the existing decode loop; flip `supports_streaming` to `true` when wired.
3. **`src/backends/remote/exo.rs`** — override the new method over the existing
   SSE path (`stream: true`); other remote backends keep the default for v1.
4. **`src/cli/mod.rs`** — add the `OutputDelta` variant and the additive
   `streaming` field on `HeadlessEvent::Init`; add the `output_delta` constructor.
5. **`src/main.rs`** — add the `--include-partial-output` flag to `Cli`; add the
   `run_streaming_turn` driver path; validate flag/`--output` combination
   (usage-error on non-ndjson); populate `init.streaming` from `backend_info`.
6. **`docs/headless-contract.md`** *(extends the ADR-024 doc)* — document the
   flag, the `output_delta` event, `init.streaming`, the one-flag activation, and
   the authoritative-source (advisory-only) invariant.
7. **`docs/adr/README.md`** — add the ADR-028 index row.
8. **`tests/headless_streaming.rs`** *(new integration test binary — a `tests/`
   file is a separate crate, not a `src` module)* — below.

Reused **without modification**: `generate_command`, `GeneratedCommand`,
`BackendInfo`, the `SafetyValidator`, ADR-024's `HeadlessEnvelope` / `ExitCode` /
`render_line`, ADR-026's conversation driver (streaming composes with it: the
active turn streams, prior turns are unchanged), ADR-027's permission record
(decided on the final envelope, unaffected by deltas), and the `static`/`mock`
backends for deterministic offline tests.

---

## Integration tests (known inputs → deterministic JSON + exit code)

In `tests/headless_streaming.rs`, offline and deterministic. A small in-test
`mock` backend that overrides `generate_command_streaming` to emit fixed deltas
(`"ls"`, `" -la"`) gives a deterministic streaming case; the `static` backend
gives the degradation case. `$XDG_CACHE_HOME` → `tempfile::TempDir`.

| # | Invocation | Asserted events / exit |
| --- | --- | --- |
| 1 | mock backend, `--output ndjson --include-partial-output "list files"` | `init.streaming == true`; ≥2 `output_delta` events with strictly increasing `index`; then one `result`; `concat(text) == "ls -la"`; **exit 0**. |
| 2 | static backend (non-streaming), `--output ndjson --include-partial-output "list files"` | `init.streaming == false`; **exactly one** `output_delta` whose `text == result.payload.command` (default-impl degradation); **exit 0**. |
| 3 | **Advisory invariance** — same prompt with and without `--include-partial-output` | the terminal `result.payload` is **byte-identical** across both runs (deltas change nothing on the contract). |
| 4 | **Usage gate** — `--output json --include-partial-output "x"` | **exit 2**, `status == "error"`, `error.kind == "usage"` (deltas need a stream). |
| 5 | **Blocked, display-only** — CRITICAL prompt, mock streams the command text, `--output ndjson --include-partial-output` | deltas may appear, but the final `result` has `status == "blocked"`, **exit 3** (ADR-024); assert no delta carries an executable/approval signal and the safety verdict appears **only** in the envelope. Proves deltas are not an approval channel. |
| 6 | **No `--verbose` needed** — case 1 without any verbose flag | deltas still stream (one-flag activation; no three-flag friction). |
| 7 | **serde round-trip** — `to_string`→`from_str` on `OutputDelta` and on an `Init` with `streaming` | byte-stable; proves serializable-from-day-one. |

Every test asserts both the parsed events/envelope **and** the process exit code,
so the two can never drift. Test 3 locks the advisory-only invariant; test 5
locks that streaming never becomes a safety side channel.

---

## Out of scope (explicitly deferred to a later ADR)

- **Explanation / reasoning-token streaming.** v1 streams *command text* only.
  Local models may emit scratchpad/`<think>` tokens; surfacing those as a
  separate `reasoning_delta` is a clean additive follow-on.
- **`--json-schema` enforced structured output** (constraining output to a
  caller schema). Still deferred from ADR-024; unaffected here.
- **Cost / usage accounting** (`total_cost_usd` analog). Still deferred from
  ADR-024; caro is offline, a `tokens`/`latency` block is additive later.
- **Backpressure / flow control** on a slow consumer. v1 is best-effort: the
  driver writes deltas as produced and relies on the OS pipe buffer; a consumer
  must drain stdout. Bounded buffering / a `--max-delta-rate` is a v2 concern.
- **Streaming intermediate turns of an auto-executing agent loop.** The
  auto-execution loop itself remains deferred (ADR-026 out-of-scope); when it
  lands, each turn streams via this same contract additively.
- **Wiring every remote backend's SSE.** v1 wires embedded (the primary local
  path) and exo (already capability-`true`); ollama/vllm/openrouter/claude keep
  the graceful default until individually wired — honest `supports_streaming`
  reporting means no contract lie in the interim.

---

## Consequences

### Positive

- Delivers the one dimension ADR-024 and ADR-026 both punted on, completing the
  headless event taxonomy: `init` → `output_delta`* → (`user_turn`/`assistant_turn`)
  → `result`.
- Fixes all four Phase-1 problems by design: one-flag activation (vs three),
  typed/versioned deltas (vs raw provider shapes), an explicit advisory-only
  invariant (vs ambiguous source-of-truth), and up-front `init.streaming`
  capability (vs empirical discovery).
- Graceful default means **zero** churn for non-streaming backends and a single
  consumer code path across all backends — backend heterogeneity never reaches
  the wire.
- Additive and opt-in: default behavior and the ADR-024 envelope are unchanged;
  no existing wrapper breaks; no new exit code.
- Net new code is tiny: one default trait method, one event variant, one `Init`
  field, one flag, one driver path, plus two backend overrides — everything else
  is composition over existing types.

### Negative / risks

- A second trait method enlarges the backend contract; mitigated by the default
  impl (overriding is optional) and by `generate_command` remaining the single
  authoritative generator the streaming method must agree with.
- **Risk:** a consumer treats concatenated deltas as the answer and diverges from
  the envelope. → **Mitigation:** the advisory-only invariant is documented and
  asserted (test 3); the final command/verdict/exit derive solely from the
  envelope.
- **Risk:** streaming a command's text before the safety verdict tempts a naïve
  consumer to act on partial output. → **Mitigation:** deltas carry no
  executable/approval signal; the safety verdict and permission outcome (ADR-027)
  appear only in `result`; test 5 locks this. Deltas are display-only by
  contract.
- **Risk:** a slow/blocked stdout consumer stalls the producer. → **Mitigation:**
  best-effort writes + documented "consumer must drain"; bounded buffering is
  explicitly v2.

### Neutral

- No new dependency, no new module, no daemon, no persisted state, no new exit
  code — a pure subprocess contract, consistent with ADR-024/025/026/027.

---

## Alternatives considered

1. **Copy Claude Code's `stream-json` + `--verbose` + `--include-partial-messages`
   verbatim, forwarding raw provider deltas.** Rejected: reproduces the
   three-flag friction and provider-coupling — the exact bug-then-patch path
   ADR-024 set out to avoid. "Solve the failure mode by design, not workaround."
2. **A real `Stream`/`futures::Stream` associated type on the trait.** Rejected
   for v1: makes `CommandGenerator` not object-safe without boxing gymnastics,
   forces every backend to restructure, and complicates the default. A
   `&mut dyn FnMut(&str)` sink is the minimal object-safe additive surface; a
   richer `Stream` can come later without breaking the wire (the wire is events,
   not the Rust type).
3. **Make `output_delta` authoritative (reconstruct the command from deltas).**
   Rejected: couples correctness to a display channel and forbids
   post-generation safety rewrites. Keeping deltas advisory is what preserves the
   ADR-024 envelope as the single source of truth — the same discipline ADR-025
   used to keep the init cache off the correctness path.
4. **Gate streaming behind `--verbose` like the analog.** Rejected: caro's NDJSON
   is already structured; coupling intent to a verbosity flag is the friction we
   are removing. One intent flag, validated against the output mode.
5. **A new `src/streaming/` module.** Rejected: unnecessary. The event type
   belongs beside ADR-024's in `cli`; the trait method on the existing trait; the
   driver is a thin branch in `main.rs`. "No new modules unless unavoidable."

---

## Constraint-compliance check

- *Reuse existing validator/safety/config infrastructure; do not duplicate* — no
  new safety/validation logic; deltas are display-only, the verdict is the
  existing `SafetyValidator` result carried in the ADR-024 envelope. ✅
- *All new types serializable from day one* — `OutputDelta`, the `Init.streaming`
  field, and the trait sink all derive/serialize from the start; the trait method
  returns the already-serde `GeneratedCommand`. ✅
- *Pure subprocess call (no daemon, no state)* — one process in, an NDJSON stream
  out ending in one `result`, exit. No resident process, no persisted streaming
  state. ✅
- *Solve the Phase-1 failure mode by design, not workaround* — one-flag
  activation, typed/versioned deltas, explicit advisory-only invariant, and
  declared `init.streaming` capability replace the analog's three-flag,
  provider-coupled, source-ambiguous, implicit-capability stream. ✅

---

## Suggested landing sequence (per repo rules)

1. Feature branch via `bin/sk-new-feature "headless token streaming contract"`
   (`.claude/rules/git-workflow.md` — never commit to `main`).
2. Land the trait default method + the `OutputDelta`/`Init.streaming` additions +
   the driver branch behind the new flag (additive; default behavior unchanged).
3. Wire the embedded override (flip `supports_streaming` to `true` only when the
   decode-loop hook is in place) and the exo SSE override; leave other remotes on
   the default.
4. Update `docs/adr/README.md` with ADR-028; renumber only if a later ADR already
   claimed 028 (`.claude/rules/adr-numbering.md`).
5. This refines an already-GA path (headless mode), not a new user-facing product
   line, so `validation-discipline.md`'s 20-transcript gate does not apply (per
   that rule's "what this rule does NOT do" clause); standard `dev-process.md` CI
   governs (`cargo test`, `cargo clippy -- -D warnings`, `cargo test safety`).
   Test 5 (streaming-is-not-a-safety-side-channel) is safety-adjacent — develop
   it TDD-first.

## Sources

- [Run Claude Code programmatically (headless) — Claude Code Docs](https://code.claude.com/docs/en/headless)
  — `--output-format stream-json`, `--verbose`, `--include-partial-messages`,
  `text_delta` events, `system/init` capability reporting.
- Caro codebase: `src/backends/mod.rs` (`CommandGenerator`, `BackendInfo.supports_streaming`),
  `src/backends/embedded/embedded_backend.rs` (decode loop, `supports_streaming: false`),
  `src/backends/remote/exo.rs` (`supports_streaming: true`, SSE `stream` field),
  `src/cli/mod.rs` (`OutputFormat`, ADR-024 `HeadlessEvent`), `src/models/mod.rs`
  (`GeneratedCommand`), `Cargo.toml` (`reqwest["stream"]`, `futures`).
- Prior runs of this scoping process: ADR-024 (headless JSON/NDJSON contract —
  defers partial-message streaming), ADR-025 (init snapshot cache — the
  "optimization/advisory, never on the correctness path" discipline reused here),
  ADR-026 (multi-turn NDJSON session — also defers partial-message streaming;
  this contract composes with its per-turn events), ADR-027 (permission
  resolution — the verdict that deltas must never short-circuit).
- `.claude/rules/adr-numbering.md` — sequential numbering (this is ADR-028,
  following ADR-027).
- `.claude/rules/validation-discipline.md` — refinement of a GA path; discovery
  gates do not apply.

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-30 | caro-research--scoping-process (automated) | Initial draft (ADR-028) |
