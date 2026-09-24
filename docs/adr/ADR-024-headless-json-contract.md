# ADR-024: Headless Mode — A Versioned, Stable JSON/NDJSON Output Contract

- **Status**: Proposed
- **Date**: 2026-06-23
- **Authors**: caro-research scoping process (automated scheduled run)
- **Supersedes / relates to**: ADR-016 (`caro fix` structured failure context),
  ADR-020 (Tiered Approval Protocol), ADR-022 (caro safety library API)

> **Provenance note (autonomous run).** This ADR was produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound. With no user present to disambiguate, the run
> selected the feature that best fits the task's stated emphasis on *output
> contracts, exit codes, session/context lifecycle, and "pure subprocess
> call (no daemon, no state)"*: **Claude Code's non-interactive "print /
> headless" mode** (`claude -p --output-format json|stream-json`). The
> scope below adapts that feature to caro's offline, standalone positioning.
> Treat the choice of analog as a reviewable assumption, not a settled
> decision.

---

## Context

### The problem and who has it

caro today is built for a human at a terminal: it generates a command, renders
a colored block, asks for confirmation, and executes. But three growing
audiences want caro as a **machine-callable subprocess**, not a TUI:

1. **CI / pre-commit pipelines** that want to call `caro` to lint or assess a
   shell command and branch on a *stable* exit code and a *stable* JSON shape.
2. **Agent frameworks and wrappers** (the same way people wrap `claude -p`)
   that want to embed caro's NL→command + safety verdict as a tool, parsing a
   documented payload rather than scraping human-formatted text.
3. **Scripts** (`jq` pipelines, Makefiles, git hooks) that need
   `caro --output json "..." | jq -r '.command'` to never break across
   patch releases.

The competitor reference point is Claude Code's headless mode: `-p` +
`--output-format {text,json,stream-json}`, session resume via `--resume <id>`,
documented exit codes (0 success / 1 general error / 2 auth error), an
`--include-partial-messages` token stream, and a `--bare` mode that skips
auto-discovery for deterministic CI results. ([Claude Code headless
docs](https://code.claude.com/docs/en/headless))

### What caro already has (do not rebuild)

The Phase-2 codebase inventory found that the *plumbing* largely exists; what
is missing is a **contract**:

| Capability | Exists today | File |
| --- | --- | --- |
| `--output {json,yaml,plain}` flag | ✅ | `src/main.rs` (`Cli`), `src/cli/mod.rs` `OutputFormat` |
| A result envelope struct | ✅ `CliResult` (20 fields, `Serialize`) | `src/cli/mod.rs` |
| Per-run timing | ✅ `TimingInfo` | `src/cli/mod.rs` |
| Backend output | ✅ `GeneratedCommand` (`Serialize`) | `src/models/mod.rs` |
| Safety verdict | ✅ `ValidationResult`, `SafetyDecision`, `RiskLevel`, `SuggestedRouting` (all `Serialize`) | `src/safety/mod.rs`, `src/models/mod.rs` |
| Session state | ✅ `AiSession` / `Turn` (`Serialize`, file-persisted) | `src/ai/session.rs`, `src/ai/runner.rs` |
| serde / serde_json / schemars | ✅ | `Cargo.toml` |
| pretty JSON render path | ✅ wired in `main.rs` (~L3610) | `src/main.rs` |

So caro can already emit *a* JSON blob. What it cannot do is promise that the
blob's shape, field names, or exit codes are stable — there is no
`schema_version`, no documented exit-code table (only an ad-hoc
`EXIT_CODE_EDIT = 201`), no newline-delimited event stream for agents, and no
guarantee that picking up `~/.config/caro` config won't change the output on a
different machine.

### The failure mode we must design around

Claude Code's headless mode ships with a documented determinism hazard: a
plain `claude -p` "loads the same context an interactive session would,
including anything configured in the working directory or `~/.claude`," so a
teammate's hook or a project `.mcp.json` silently changes results. Their fix
was to **add `--bare` after the fact** to skip auto-discovery — a workaround
bolted onto an already-shipped default.

**We solve this by design, not by workaround:** caro's headless contract is
deterministic *by default*. Headless mode reads only explicitly-passed flags
and a pinned config snapshot; ambient state (user config file, env-derived
overrides, telemetry, session auto-resume) is excluded from the contract
unless a flag opts in. There is no "add a bare flag later" — the machine-facing
path starts bare.

---

## Decision

Introduce a **Headless Mode** governed by a single, versioned output contract.

### 1. Surface

`caro --output json "<prompt>"` and `caro --output ndjson "<prompt>"` enter
headless mode. Headless mode implies:

- **Non-interactive**: never prompts; `requires_confirmation` is reported in
  the payload instead of blocking on a TTY.
- **Deterministic by default**: ignores the user config file and env
  overrides; uses built-in defaults plus only the flags passed. Opt back in
  with `--use-config` (loads `ConfigManager`) — explicit, not ambient.
- **Pure subprocess**: reads optional stdin (capped, see contract), writes the
  envelope to stdout, diagnostics to stderr, exits with a contract code. No
  daemon, no lockfile, no background process retained after the final byte.

`--output json` emits exactly one envelope object. `--output ndjson` emits a
stream of newline-delimited events ending in a `result` event whose `payload`
is the same envelope. (NDJSON is caro's equivalent of Claude Code's
`stream-json`; we keep `yaml`/`plain` untouched for humans.)

### 2. Schema-first envelope (new type: `HeadlessEnvelope`)

A new struct in `src/cli/mod.rs` (next to `CliResult`), serializable from day
one, with an explicit version field as field #1. It **wraps and reuses** the
existing serializable types rather than duplicating them.

### 3. Stable exit codes (new type: `ExitCode`)

A single `#[repr(i32)]` enum in `src/cli/mod.rs` is the only place process exit
codes are defined for headless mode, replacing scattered literals.

### 4. Stateless session resume

Session "lifecycle" is satisfied without a daemon by reusing the existing
file-backed `AiSession`: `--session <path>` reads a session JSON, appends one
`Turn`, writes it back, and exits. Redundant initialization is avoided not by
keeping a process warm but by (a) the deterministic-bare default skipping all
auto-discovery, and (b) a `--session` snapshot that carries prior turns in a
file the caller owns. This matches "no daemon, no state" while still giving
agents multi-turn continuity.

---

## New types

All live in **`src/cli/mod.rs`** (no new module). All derive
`#[derive(Debug, Clone, Serialize, Deserialize)]`; the envelope additionally
derives `schemars::JsonSchema` so we can publish the schema and run
`--json-schema`-style validation later.

### `HeadlessEnvelope`

```rust
/// The single, versioned contract object emitted in headless mode.
/// `schema_version` is field #1 and MUST be bumped on any breaking change.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HeadlessEnvelope {
    /// Contract version. "1" for the first release. Breaking changes bump this.
    pub schema_version: String,
    /// "ok" | "blocked" | "needs_confirmation" | "error"
    pub status: HeadlessStatus,
    /// Present unless status == "error" before generation.
    pub command: Option<String>,
    pub explanation: Option<String>,
    /// Reuses existing safety types — NOT a new safety struct.
    pub safety: Option<ValidationResult>,
    pub risk_level: Option<RiskLevel>,
    pub suggested_routing: Option<SuggestedRouting>,
    pub alternatives: Vec<String>,
    pub shell: ShellType,
    pub backend: String,
    pub confidence: Option<f64>,
    /// Reuses existing TimingInfo.
    pub timing: TimingInfo,
    pub warnings: Vec<String>,
    /// Present only when --execute and execution occurred.
    pub execution: Option<ExecutionReport>,
    /// Present only when --session was passed.
    pub session_id: Option<u64>,
    /// Structured error; present iff status == "error".
    pub error: Option<HeadlessError>,
    /// The exit code the process will return, mirrored for log capture.
    pub exit_code: i32,
}
```

### `HeadlessStatus`

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeadlessStatus { Ok, Blocked, NeedsConfirmation, Error }
```

### `ExecutionReport`

```rust
/// Carved from CliResult's execution fields so the contract is stable
/// independent of CliResult's internal churn.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExecutionReport {
    pub executed: bool,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub error: Option<String>,
}
```

### `HeadlessError`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HeadlessError {
    /// Stable machine code: "usage" | "stdin_too_large" | "backend_unavailable"
    /// | "generation_failed" | "session_io" | "internal"
    pub kind: String,
    pub message: String,
}
```

### `HeadlessEvent` (NDJSON only)

```rust
/// One line of the --output ndjson stream. The final event is
/// { "type": "result", "payload": <HeadlessEnvelope> }.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HeadlessEvent {
    /// First line: schema_version, caro version, shell, backend, deterministic flag.
    Init { schema_version: String, caro_version: String, shell: ShellType, backend: String, deterministic: bool },
    Generation { command: String, confidence: Option<f64> },
    Safety { risk_level: RiskLevel, allowed: bool },
    Execution { stage: String },              // "start" | "complete"
    Result { payload: HeadlessEnvelope },     // always last
}
```

### `ExitCode`

```rust
/// The ONLY definition of headless exit codes. `as i32` gives the process code.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    Ok = 0,                 // command generated (and, if --execute, ran successfully)
    GenericError = 1,       // unexpected/internal failure
    UsageError = 2,         // bad flags / bad input combination
    Blocked = 3,            // safety validator blocked the command
    NeedsConfirmation = 4,  // would require confirmation; not executed (no TTY)
    BackendUnavailable = 5, // no inference backend reachable
    ExecutionFailed = 6,    // command ran under --execute but exited non-zero
}
```

Method contracts (all in `impl` blocks in `src/cli/mod.rs`):

- `HeadlessEnvelope::from_cli_result(&CliResult, ExitCode) -> Self` — the single
  adapter from the existing internal result to the public contract. This is the
  *only* place the mapping lives, so `CliResult` can keep evolving internally.
- `HeadlessEnvelope::error(kind: &str, msg: impl Into<String>, code: ExitCode) -> Self`
  — constructs an error envelope with sensible empty defaults.
- `HeadlessEnvelope::render(&self, ndjson: bool) -> String` — serializes via
  `serde_json`; for NDJSON, wraps in the `Result` event.
- `ExitCode::as_process_code(self) -> i32` — `self as i32`.
- `impl From<&HeadlessEnvelope> for ExitCode` is **not** provided; the code is
  decided at the call site and stored in `envelope.exit_code` so payload and
  process code can never disagree.

---

## Minimal set of files that change

No new modules. Six existing files:

1. **`src/cli/mod.rs`** — add the six types above + their impls. Add
   `Ndjson` variant to the existing `OutputFormat` enum.
2. **`src/main.rs`** — (a) add `--use-config` and `--session <path>` flags to
   `Cli`; (b) in the existing render branch (~L3610) route `Json`/`Ndjson`
   through `HeadlessEnvelope::from_cli_result` and exit via
   `ExitCode::as_process_code`; (c) make headless mode skip the
   config/env load unless `--use-config`. Remove the ad-hoc
   `EXIT_CODE_EDIT = 201` literal in favor of the enum (or document it as a
   non-headless special case).
3. **`src/ai/session.rs`** — add `AiSession::load_or_new(path) ->
   io::Result<Self>` and `save(&self, path) -> io::Result<()>` if not already
   present; reused by `--session`.
4. **`docs/adr/README.md`** — add the ADR-024 table row.
5. **`docs/headless-contract.md`** (new doc, not a module) — publishes the
   exit-code table, the envelope JSON Schema (generated via `schemars`), and
   the stability promise. This is the user-facing contract.
6. **`tests/headless_contract.rs`** (new integration test file) — see below.

Reused without modification: `ValidationResult`, `SafetyDecision`,
`RiskLevel`, `SuggestedRouting`, `GeneratedCommand`, `TimingInfo`,
`ShellType`, the `SafetyValidator`, and `ConfigManager`.

---

## Exit-code / output contract (what machines depend on)

The published, frozen contract for v1 (`schema_version: "1"`):

| Exit | Meaning | `status` field |
| --- | --- | --- |
| 0 | command generated (and executed OK if `--execute`) | `ok` |
| 1 | internal/unexpected error | `error` |
| 2 | usage error (bad flags/input) | `error` |
| 3 | safety validator blocked the command | `blocked` |
| 4 | needs confirmation; not executed (no TTY) | `needs_confirmation` |
| 5 | no backend reachable | `error` |
| 6 | `--execute` ran but command exited non-zero | `ok` (generation) + `execution.exit_code != 0` |

Stability promises (documented in `docs/headless-contract.md`):

- Within a `schema_version`, fields are only **added**, never renamed/removed,
  and exit-code meanings never change. Breaking either bumps `schema_version`.
- `caro --output json` always prints exactly one JSON object to stdout; all
  human/diagnostic text goes to stderr.
- stdin is accepted as extra context and **capped at 1 MiB**; exceeding the cap
  exits `2` with `error.kind = "stdin_too_large"` (mirrors Claude Code's 10 MB
  cap lesson — fail loud, don't truncate silently).

---

## Integration tests (deterministic input → fixed JSON + exit code)

In `tests/headless_contract.rs`, driven through the `StaticBackend` (template
backend) so output is deterministic and offline — no model download, ideal for
CI:

1. **Safe command** — `caro --backend static --output json "list files"`
   → exit `0`, `status:"ok"`, `command` non-empty, `safety.allowed:true`,
   `schema_version:"1"`. Assert exact JSON via `serde_json::from_str` →
   field-by-field (not string match, to allow additive fields).
2. **Blocked command** — a prompt the validator flags CRITICAL
   → exit `3`, `status:"blocked"`, `risk_level:"critical"`,
   `safety.allowed:false`, `command` still present (we report what was blocked).
3. **Usage error** — `--output json --execute --session bad/\0/path`
   → exit `2`, `status:"error"`, `error.kind:"usage"`.
4. **stdin cap** — pipe 2 MiB → exit `2`, `error.kind:"stdin_too_large"`.
5. **Determinism** — run case 1 twice, once with a populated fake
   `$XDG_CONFIG_HOME/caro/config.toml` present, once without; assert byte-identical
   stdout (proves the bare-by-default design — config does not leak in).
6. **NDJSON shape** — `--output ndjson "list files"` → first line parses as
   `Init`, last line parses as `Result`, and `Result.payload` deep-equals the
   `--output json` envelope for the same input.
7. **Session continuity** — run with `--session $TMP/s.json` twice; assert the
   file gains a second `Turn` and `session_id` is stable across the two runs.

Each test asserts both the parsed envelope **and** the process exit code, so
the two can never drift.

---

## Out of scope (next version)

- **Token-level / partial-message streaming** (Claude Code's
  `--include-partial-messages`). v1 NDJSON emits coarse lifecycle events only;
  embedded-backend token deltas wait for v2.
- **`--json-schema` constrained output** (forcing the model to fit a
  caller-supplied schema). v1 *publishes* a schema; it does not *enforce* a
  caller's. Hooks are left (the `schemars` derive) so v2 can add it.
- **Cost/usage accounting** (`total_cost_usd`). caro is offline/local; a
  `tokens`/`latency` block can be added additively under the same
  `schema_version` later.
- **Multi-turn agentic loop in one process** (caro's `src/agent/` loop exposed
  headlessly). v1's `--session` file snapshot covers continuity without a
  resident loop; exposing the full agent loop is a separate ADR.
- **Daemon / server mode.** Explicitly excluded — violates the "pure
  subprocess, no daemon" constraint. The MCP safety server (ADR-015) remains
  the long-lived-process story; headless mode is its stateless complement.

---

## Consequences

**Benefits**

- A single adapter (`from_cli_result`) decouples the public contract from
  `CliResult`'s internal churn — internal refactors no longer risk breaking
  `jq` pipelines.
- Deterministic-by-default eliminates the exact "ambient config changes output"
  failure mode Claude Code had to retrofit `--bare` for.
- Reuses every existing serializable type and the safety validator; net new
  code is ~6 small types + one adapter + tests.
- Versioned schema from day one means we can evolve without a flag-day break.

**Trade-offs / risks**

- A frozen contract is a maintenance commitment: additive-only discipline must
  be enforced (a `schemars`-generated golden schema check in CI is recommended).
- Deterministic-by-default surprises users who *expect* their config to apply;
  mitigated by the explicit `--use-config` opt-in and clear docs.
- NDJSON event taxonomy is small in v1; agents wanting fine-grained progress
  must wait for v2 (documented as out-of-scope, not a silent gap).

## Alternatives considered

1. **Extend `CliResult` and just stabilize it directly.** Rejected: it has 20
   fields tied to interactive concerns (`confirmation_prompt`, `explain_mode`,
   `debug_info`); freezing it as the public contract couples the machine API to
   TUI internals — exactly the coupling that bit Claude Code.
2. **Ship `--output json` as-is and call it done.** Rejected: no version, no
   exit-code contract, config leaks in → the determinism failure mode survives.
3. **Add a long-lived `caro serve` daemon for session reuse.** Rejected:
   violates the "no daemon, no state" constraint; the file-backed `--session`
   snapshot gives continuity without a resident process. (Daemon territory
   already belongs to the MCP server, ADR-015.)
4. **Copy Claude Code's `stream-json` + add `--bare` later.** Rejected: that
   reproduces the bug-then-patch sequence. Designing bare-by-default is the
   whole point.

## References

- [Claude Code — Run Claude Code programmatically (headless)](https://code.claude.com/docs/en/headless)
- ADR-015 (MCP safety server) — the long-lived-process complement to this stateless mode
- ADR-016 (`caro fix` structured failure context) — precedent for structured, serializable output
- ADR-020 (Tiered Approval Protocol) — source of `SuggestedRouting` reused here
- ADR-022 (caro safety library API) — `ValidationResult` reuse
- `.claude/rules/adr-numbering.md` — sequential numbering (this is ADR-024)
