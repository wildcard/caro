# ADR-034: Headless Degraded-Result Contract — Typed Fallback Provenance, Non-Fatal Error Taxonomy, and Stdout Hygiene

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-10
- **Authors**: caro-research scoping process (automated scheduled run)
- **Builds on / relates to**: ADR-024 (Headless JSON/NDJSON output contract),
  ADR-025 (init snapshot cache), ADR-026 (multi-turn agentic session),
  ADR-027 (permission resolution), ADR-028 (token streaming),
  ADR-029 (schema-enforced structured output), ADR-031 (session circuit breaker)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Prior runs scoped the
> headless envelope (024), warm init (025), multi-turn (026), permissions
> (027), streaming (028), and caller schemas (029). This run switches analog:
> instead of Claude Code, it analyzes **Gemini CLI's headless mode**
> (`gemini --output-format json`, google-gemini/gemini-cli — fully OSS, so
> its issue tracker exposes real failure modes in production). The dimension
> selected is the one none of the six prior ADRs cover and the analog
> demonstrably gets wrong: **what the contract says when the run partially
> fails** — backend fallback, non-fatal errors, and keeping the machine
> stream clean while degrading. Treat the analog choice as a reviewable
> assumption, not a settled decision.

---

## Context

### Phase 1 — what the analog does, and where it breaks

Gemini CLI headless mode (`-p` + `--output-format json`) emits a single JSON
object `{ response, stats, error }` and documents "consistent exit codes for
error handling" ([headless docs](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/headless.md)).
It is the direct OSS peer of the surface caro standardized in ADR-024. Four
failure modes are live in their tracker:

1. **Non-fatal errors are fatal in JSON mode.**
   [#9281](https://github.com/google-gemini/gemini-cli/issues/9281): with
   `--output-format json` the CLI **exits on any tool error, even non-fatal
   ones** (e.g. the model passing invalid args to a tool once and then
   self-correcting). Interactive mode recovers; machine mode aborts. The
   contract has only two terminal states — perfect success or hard error —
   so any mid-run hiccup is forced into "hard error", discarding work that
   interactive users would have kept.
2. **Plain text corrupts the machine stream.**
   [#22647](https://github.com/google-gemini/gemini-cli/issues/22647): in ACP
   mode, internal log lines and ephemeral system messages are printed to
   stdout, corrupting the JSON-RPC stream and crashing parent clients'
   parsers. Stream purity is promised in docs but not enforced anywhere in
   code — any `console.log` added by any contributor can break every
   downstream consumer.
3. **Provenance is missing from the payload.**
   [#14435](https://github.com/google-gemini/gemini-cli/issues/14435): the
   JSON output does not carry the session ID needed to resume, so multi-step
   workflows cannot chain runs;
   [#22604](https://github.com/google-gemini/gemini-cli/issues/22604): on
   resume, logs are tagged with a freshly generated ephemeral session ID
   instead of the resumed one. The payload tells you *what* was answered but
   not reliably *how/under which identity* the run executed.
4. **Docs and implementation drift.**
   [#9009](https://github.com/google-gemini/gemini-cli/issues/9009): for a
   period, documented `--output-format json` did not exist in the shipped
   binary — the flag was documentation-first, implementation-later, with no
   mechanical check tying the two together.

The common shape: **the JSON contract only models the happy path and the
fatal path.** Everything in between — a fallback that succeeded on the second
try, a warning that didn't stop the run, an internal log line — either kills
the run (1), corrupts the stream (2), or is silently absent from the payload
(3). Session lifecycle: per-invocation, nothing cached; resume exists but its
identity bookkeeping is buggy (3), which is precisely why ADR-024 chose
caller-owned session files.

### Phase 2 — what caro already has, and what only caro can do

Codebase inventory (2026-07-10):

| Capability | Exists today | File |
| --- | --- | --- |
| Versioned envelope + exit-code enum (0–8 allocated) | ✅ ADR-024/027/029 | `src/cli/mod.rs` |
| `-o/--output {json,yaml,plain}` render path | ✅ | `src/main.rs` (~L3627), `src/cli/mod.rs` `OutputFormat` |
| Multi-backend fallback chain | ✅ `HybridBackend` (sanitize→enhance→restore→**fallback**) | `src/backends/hybrid/mod.rs` |
| Deterministic always-available floor | ✅ `StaticMatcher` template backend | `src/backends/static_matcher.rs` |
| Typed backend errors | ✅ `GeneratorError` (thiserror, `Serialize`) incl. `Unsafe { risk_level }` | `src/backends/mod.rs` |
| Backend self-description | ✅ `BackendInfo` (`Serialize`) | `src/backends/mod.rs` |
| Warnings channel in result | ✅ `CliResult.warnings: Vec<String>` (untyped strings) | `src/cli/mod.rs` |
| Diagnostics-to-stderr convention | ⚠️ partial — `CARO_WRAPPER` path only; `colored` prints elsewhere | `src/main.rs` `print_plain_output` |

**What the analog gets right, to replicate:** a `stats` block distinct from
the answer (usage/latency separated from payload); a top-level `error` object
rather than bare exit codes; recognizing (in interactive mode) that tool
errors are recoverable events, not run-enders.

**Their gaps we avoid by designing the schema first:** (1) no vocabulary for
"succeeded, with caveats" — we add one; (2) stream purity as convention — we
make it a tested invariant; (3) provenance as an afterthought — we make the
backend chain a first-class envelope field; (4) docs-first drift — the
contract doc is generated from the same `schemars` types the binary uses.

**What only caro can do:** caro ships a **deterministic offline floor** — the
static template backend — beneath every probabilistic backend. A remote-API
wrapper degrades to *nothing* when its API misbehaves; caro can degrade to a
*worse-but-valid, still safety-gated* answer and say so honestly. "Never
return exit 1 for a recoverable generation problem" is a promise only an
offline-capable, multi-backend tool can make. Combined with the hybrid
backend's PII sanitizer, provenance also answers a question no competitor
payload answers: *did my prompt leave the machine, and redacted how?*

### The failure mode we must design around

Gemini #9281 generalized: **a contract with no degraded state forces every
consumer to treat all failures as total.** CI callers retry runs that already
produced usable output; wrappers can't distinguish "remote was down, local
answered" from "everything is fine" — so they can't alert on quietly rotting
infrastructure. And #22647 generalized: **any print statement anywhere is a
latent contract breach** unless stream purity is mechanically enforced. We
solve both by design: a typed degradation taxonomy that keeps runs alive, and
a single-emitter stdout rule locked by tests.

---

## Decision

Introduce the **degraded-result contract** for headless mode: runs that
recover from non-fatal failures exit `0`, carry `degraded: true`, a typed
event list, and a full backend-attempt provenance report. Four commitments:

### 1. Non-fatal never means fatal

Inside headless generation, a backend failure is terminal **only if no
backend in the resolved chain can answer**. Each failed attempt becomes a
`DegradationEvent`; the run proceeds down the chain (hybrid → embedded →
static, per existing resolution order). Exit `5` (`BackendUnavailable`)
is reserved for "the entire chain failed" — its ADR-024 meaning is unchanged.
This is the designed-in fix for #9281: recoverable errors are *data*, not
process death.

### 2. Provenance is a first-class field

The envelope gains a `provenance` report: which backend was requested, which
answered, every attempt in order with outcome and latency, and whether the
hybrid sanitizer redacted the prompt. This is the designed-in fix for
#14435/#22604: a machine consumer never has to guess how the answer was
produced. (Session identity itself is already caller-owned per ADR-024
`--session`; provenance records the path, not the identity.)

### 3. Degradation reporting is honest but non-gating by default

`degraded: true` never changes the exit code on its own — a degraded success
is a success (exit `0`). Callers that want to gate (e.g. CI that must not
silently fall back to templates) pass **`--fail-on-degraded`**, which turns
`degraded: true` into exit **`9`** with the envelope otherwise identical.
Same opt-in philosophy as ADR-029's advisory findings: the payload informs,
the caller decides policy.

### 4. Stdout carries contract bytes only — enforced, not promised

In headless mode (`--output json|ndjson`) there is exactly **one** code path
that writes to stdout: the envelope/event emitter. All diagnostics, verbose
traces, `colored` output, and progress text go to stderr unconditionally
(today this is gated on `CARO_WRAPPER`; the gate becomes `output_format !=
Plain`). The invariant is locked by an integration test that runs with
`--verbose` and asserts stdout parses as exactly one JSON value. This is the
designed-in fix for #22647 — purity as a tested invariant, not a convention.

---

## New types (all `Serialize + Deserialize + JsonSchema` from day one)

In `src/cli/mod.rs`, beside the ADR-024 envelope types. No new modules.

```rust
/// Outcome of one backend attempt in the resolution chain.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome { Succeeded, Failed, Skipped }

/// One entry in the ordered backend-resolution chain.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BackendAttempt {
    pub backend: String,               // e.g. "ollama", "embedded", "static"
    pub outcome: AttemptOutcome,
    pub latency_ms: Option<u64>,       // None for Skipped
    /// Stable machine kind, present iff outcome == Failed.
    /// One of: "unreachable" | "timeout" | "parse_error" | "unsafe" |
    ///         "model_load" | "internal"
    pub error_kind: Option<String>,
}

/// Where in the pipeline a non-fatal failure occurred.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DegradationStage { Sanitize, Generate, Enhance, Restore, Cache }

/// A non-fatal failure the run recovered from.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DegradationEvent {
    pub stage: DegradationStage,
    pub kind: String,                  // same stable vocabulary as error_kind
    pub message: String,               // human detail; NOT stable
    /// What recovered: the backend or mechanism that absorbed the failure,
    /// e.g. "fallback:static", "retry", "cache_bypass".
    pub recovered_by: String,
}

/// How the answer was produced. First-class fix for provenance-free payloads.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProvenanceReport {
    pub requested_backend: String,     // what the caller asked for ("auto" if unset)
    pub used_backend: String,          // what actually answered
    pub attempts: Vec<BackendAttempt>, // full ordered chain
    pub sanitizer_applied: bool,       // hybrid PII redaction ran on the prompt
    pub degraded: bool,                // mirrors envelope.degraded
}
```

Envelope additions (additive under `schema_version: "1"` per ADR-024 — fields
only added, never renamed; absent fields keep old consumers byte-compatible):

```rust
#[serde(default, skip_serializing_if = "std::ops::Not::not")]
pub degraded: bool,
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub degradations: Vec<DegradationEvent>,
#[serde(skip_serializing_if = "Option::is_none")]
pub provenance: Option<ProvenanceReport>,   // always Some in headless mode
```

`ExitCode` (ADR-024 enum) gains one variant: `DegradedGated = 9` — emitted
**only** under `--fail-on-degraded`.

Method contracts:

- `fn stable_error_kind(&GeneratorError) -> &'static str` in
  `src/backends/mod.rs` — the single mapping from the internal `thiserror`
  enum to the stable string vocabulary. The crate's error type never leaks
  into the envelope (same stabilization-boundary pattern as ADR-029's
  `SchemaViolation::from`).
- `ProvenanceReport::single(backend: &str, latency_ms: u64) -> Self` — the
  non-degraded common case (one attempt, succeeded), so the happy path stays
  one line.
- `HeadlessEnvelope::push_degradation(&mut self, DegradationEvent)` — sets
  `degraded = true` and keeps `provenance.degraded` in sync; the only way to
  mark a run degraded, so flag and events can never disagree.
- NDJSON: `HeadlessEvent` gains a `Degradation { payload: DegradationEvent }`
  variant emitted at the moment of recovery (additive tag per ADR-024's
  event taxonomy rules).

---

## Minimal set of files to change (no new `src` modules)

1. **`src/cli/mod.rs`** — the five types above; three envelope fields;
   `ExitCode::DegradedGated = 9`; `--fail-on-degraded` flag plumbing.
2. **`src/main.rs`** — (a) flag on `Cli`; (b) build `ProvenanceReport` around
   the existing backend-resolution loop; (c) flip the diagnostics-to-stderr
   gate from `CARO_WRAPPER` to `output_format != Plain`; (d) map
   `degraded && fail_on_degraded` → exit 9 at the single envelope-emit site.
3. **`src/backends/hybrid/mod.rs`** — surface the fallback decision instead
   of swallowing it: the sanitize→enhance→restore→fallback path returns
   `(GeneratedCommand, Vec<DegradationEvent>)` internally (or pushes into a
   collector passed by the runner). No behavior change — the same fallbacks
   fire; they just become visible.
4. **`src/backends/mod.rs`** — `stable_error_kind()`; no trait change
   (attempt recording lives in the runner's resolution loop, not in each
   backend).
5. **`docs/headless-contract.md`** — exit-9 row, the three new fields, the
   stable `error_kind` vocabulary table, and the stdout-purity invariant.
6. **`docs/adr/README.md`** — ADR-034 table row.
7. **`tests/degraded_contract.rs`** — new integration-test binary (a `tests/`
   file is a separate crate, not a `src` module).

Reused unchanged: `SafetyValidator` and all 52+ patterns (degraded answers
pass the identical gate — the static fallback's output is validated exactly
like a model's), `StaticMatcher`, `GeneratorError`, `BackendInfo`,
`ConfigManager`, ADR-024 envelope machinery, ADR-025 snapshot, ADR-029
schema pipeline.

---

## Exit-code / output contract (what machines depend on)

Extends the ADR-024/027/029 table (0–8) under `schema_version: "1"`:

| Exit | Meaning | Envelope signal |
| --- | --- | --- |
| 0 | success — possibly degraded | `degraded: false` (clean) or `degraded: true` + non-empty `degradations` (recovered) |
| 5 | entire backend chain failed (meaning unchanged from ADR-024) | `status: "error"`, `provenance.attempts` all `failed` — now diagnosable |
| 9 | degraded AND caller passed `--fail-on-degraded` | identical envelope to the exit-0 degraded case except `exit_code: 9` |

Promises:

- `degraded`, `degradations`, and `provenance` are additive; ADR-024
  consumers that ignore them are byte-compatible (fields are skipped when
  empty/false in the clean case… `provenance` is always present in headless
  mode but new, hence additive).
- The `error_kind` / `DegradationEvent.kind` vocabulary
  (`unreachable | timeout | parse_error | unsafe | model_load | internal`) is
  frozen; new kinds may be **added**, existing ones never renamed. `message`
  is explicitly NOT stable — machines branch on `kind`, humans read `message`.
- Exit 9 fires **only** under `--fail-on-degraded`; without the flag a
  degraded run is exit 0, forever. No ambient config can flip this (ADR-024
  deterministic-bare rule applies).
- stdout in `--output json` mode parses as exactly one JSON value under any
  combination of `--verbose`, warnings, and degradation — enforced by test 6.

---

## Integration tests (known inputs → deterministic JSON + exit code)

`tests/degraded_contract.rs`, offline, driven by the static backend and a
mock remote (an `ollama` backend pointed at a closed local port — fails fast
and deterministically, no network):

1. **Clean run** — `caro --backend static --output json "list files"` ⇒
   exit 0, `degraded: false`, `degradations: []`,
   `provenance == {requested: "static", used: "static", attempts: [succeeded]}`.
2. **Recovered fallback** — hybrid with unreachable remote enhancer ⇒ exit 0,
   `degraded: true`, one `DegradationEvent {stage: "enhance", kind:
   "unreachable", recovered_by: "fallback:static"}`, `used_backend: "static"`,
   `command` non-empty and safety-gated. (The #9281 case: proven alive.)
3. **Gated fallback** — same as 2 plus `--fail-on-degraded` ⇒ exit 9;
   envelope deep-equals case 2's except `exit_code`.
4. **Total failure** — chain restricted to the unreachable remote only ⇒
   exit 5 (ADR-024 semantics preserved), `provenance.attempts == [failed]`,
   `error.kind: "backend_unavailable"`.
5. **Kind stability golden test** — every `GeneratorError` variant mapped
   through `stable_error_kind()` compared against a checked-in golden list;
   adding a variant without extending the golden file fails CI (the #9009
   docs-drift fix, applied to our own vocabulary).
6. **Stdout hygiene** — case 2 with `--verbose` ⇒ stdout parses as exactly
   one `serde_json::Value`; stderr is non-empty. (The #22647 fix, as a test.)
7. **Determinism** — case 2 twice ⇒ byte-identical stdout (mock latency
   fields normalized via a `--timing-zero` test hook or field-masked compare,
   whichever the ADR-024 determinism tests already chose).
8. **NDJSON ordering** — case 2 with `--output ndjson` ⇒ a `degradation`
   event appears after `init` and before `result`; `result.payload`
   deep-equals the `--output json` envelope.

Each test asserts the parsed envelope **and** the process exit code.

---

## Consequences

### Positive

- Completes the headless surface's missing dimension: 024 said *what* the
  payload is, 034 says *what happened on the way* — the difference between a
  contract that models reality and one that models the demo.
- Turns caro's existing silent hybrid fallback into a monitorable signal:
  fleets can alert on `degraded: true` rates and catch rotting remote
  backends *without* failing builds (or fail them deliberately via exit 9).
- Fixes, by design, all four live analog failure modes (#9281, #22647,
  #14435/#22604, #9009) — each mapped to a specific mechanism and test.
- Zero behavior change for interactive users and existing headless consumers;
  everything is additive under `schema_version: "1"`.

### Trade-offs / risks

- `provenance` grows every payload (~100–300 bytes typical). Accepted: it is
  the feature. Consumers that don't care skip the field.
- Surfacing hybrid-fallback events requires touching `HybridBackend`'s
  internal flow; risk of behavior drift is bounded by test 2 pinning the
  fallback path end-to-end.
- A frozen `kind` vocabulary is a maintenance commitment (same discipline as
  ADR-024's envelope); the golden test makes violations mechanical to catch.
- Latency fields threaten byte-determinism in tests; resolved by the same
  normalization the ADR-024 suite already uses (test 7 notes both options).

## Out of scope (explicitly deferred)

- **Retry policies** (backoff, per-backend retry counts, retry budgets).
  v1 records what happened; it does not add new retry behavior.
- **Circuit-breaker integration** (ADR-031): halting a *session* on repeated
  degradation is session policy, not per-invocation contract; ADR-031 can
  consume `degradations` as an input signal later.
- **Health-check subcommand** (`caro doctor --json` contract): related but
  separate surface; provenance here covers only actual runs.
- **Degradation of execution** (`--execute` retry/fallback semantics):
  execution failure remains exit 6 per ADR-024; this ADR covers generation
  only.
- **Streaming partial-degradation recovery UX** (ADR-028 deltas during a
  fallback): the NDJSON `degradation` event ships; richer interleaving waits.
- **Cost/usage accounting** — still deferred from ADR-024; unaffected.

## Alternatives considered

1. **Copy the analog: abort on any mid-run error in JSON mode.** Rejected:
   that is the bug (#9281), not a design. caro's static floor makes
   "always return a gated best-effort answer" achievable; wasting it to
   match a competitor's failure mode is strictly worse.
2. **Fold degradation into the existing `warnings: Vec<String>`.** Rejected:
   untyped prose is exactly the "machines can't branch on it" trap; the
   analog's `stats`-vs-`error` split shows consumers need structure.
   `warnings` stays for human-facing notes.
3. **Make degraded runs exit non-zero by default (e.g. always 9).** Rejected:
   breaks the ADR-024 promise that exit 0 means "command generated"; would
   force every `jq` pipeline to special-case 9. Opt-in gating preserves
   compatibility and lets policy live with the caller.
4. **A separate `--provenance` flag to include the report.** Rejected:
   optional presence recreates the analog's silent-absence trap (ADR-029's
   rough edge 2 — miss a flag, silently get nothing). Provenance is always
   present in headless mode; it is cheap and additive.
5. **Enforce stdout purity via a global writer abstraction (newtype over
   stdout injected everywhere).** Rejected for v1: a large mechanical
   refactor across dozens of print sites; the single-emit-site rule plus the
   `--verbose` purity test delivers the same guarantee at a fraction of the
   diff. The newtype remains open as a v2 hardening step.

## References

- Gemini CLI headless mode docs: https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/headless.md
- Non-fatal tool errors kill JSON mode — gemini-cli#9281: https://github.com/google-gemini/gemini-cli/issues/9281
- Plain text corrupts the machine stream — gemini-cli#22647: https://github.com/google-gemini/gemini-cli/issues/22647
- Session ID missing from JSON output — gemini-cli#14435: https://github.com/google-gemini/gemini-cli/issues/14435
- Inconsistent session ID on resume — gemini-cli#22604: https://github.com/google-gemini/gemini-cli/issues/22604
- Documented flag absent from binary — gemini-cli#9009: https://github.com/google-gemini/gemini-cli/issues/9009
- ADR-024 §exit codes and stability promises — the base contract extended here
- `.claude/rules/adr-numbering.md` — sequential numbering (033 is the highest merged; this is 034)
