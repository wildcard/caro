# Scoping: Headless JSON Output + Session Resume Contract

**Status:** DRAFT scope (not yet an accepted ADR — proposed number **ADR-024**)
**Author:** caro-research--scoping-process (automated run)
**Date:** 2026-06-18
**Feature analyzed:** Claude Code headless mode (`claude -p` / `--print`) with
`--output-format json|stream-json` and session resume (`--continue` / `--resume`)

> **Autonomous-run note.** This document was produced by the scheduled
> `caro-research--scoping-process` task. The task template shipped with an
> unfilled `[FEATURE NAME]` placeholder, so the agent selected the highest-fit
> target: **Claude Code's headless structured-output + session-resume contract.**
> Rationale: it exercises every Phase 1 question (structured payloads, exit
> codes, events, session lifecycle, "pure subprocess" constraint) and maps onto
> a feature Caro half-built but never finished wiring (the `--output json` flag
> exists but does not emit a stable machine contract for the main NL path).
> No source files were modified; this is a scope/report deliverable only.

---

## Phase 1 — Feature Research

### What problem it solves and for whom

Claude Code's headless mode (`-p`/`--print`) lets the agent run
non-interactively so scripts, CI jobs, and parent processes can call it like any
other Unix tool: pipe stdin in, get a result out, branch on the exit code.
`--output-format json` wraps the result with machine-readable metadata
(`session_id`, `result`, `is_error`, `total_cost_usd`, `num_turns`,
`duration_ms`); `--output-format stream-json` emits newline-delimited JSON
(NDJSON) where each line is one self-contained event. The audience is
**automation authors** — CI pipelines, git hooks, `package.json` linters, and
SDK wrappers — who need deterministic, parseable output rather than prose.

### Core architecture: data flow, key types, separation of concerns

```
stdin/prompt ──► agent loop ──► [system/init event] ──► turn events ──► [result event] ──► exit
                                     │                                        │
                                session_id                              is_error, subtype
                                model, tools,                           total_cost_usd,
                                mcp_servers, plugins                    num_turns, duration_ms
```

- The **`system/init`** event is always first in the stream (unless
  `CLAUDE_CODE_SYNC_PLUGIN_INSTALL` injects `plugin_install` events ahead of
  it). It reports run metadata: `session_id`, `model`, available tools, MCP
  servers, and `plugins` / `plugin_errors`.
- Mid-stream **`system/api_retry`** events surface retryable API failures
  (`attempt`, `max_retries`, `retry_delay_ms`, `error_status`, `error`
  category).
- The terminal **`result`** event carries `type:"result"`,
  `subtype:"success"|"error_max_turns"|…`, `is_error`, `total_cost_usd`,
  `duration_ms`, `num_turns`, `result`, `session_id`.
- Session state is keyed by **session ID, scoped to the current project
  directory** and its git worktrees. `--continue` resumes the most recent;
  `--resume <id>` resumes a specific one.

Separation of concerns: the transport (text / json / stream-json) is decoupled
from the agent loop; the same loop drives all three formats.

### Why it is experimental / limited — failure modes

1. **Stream hang after the final event** (
   [anthropics/claude-code#25629](https://github.com/anthropics/claude-code/issues/25629)
   ): the CLI emits `{"type":"result","subtype":"success"}` and then **does not
   exit** — stdout stays open, the process lingers, and a naive
   `output=$(claude -p … )` caller blocks forever until `SIGINT`/`SIGKILL`. The
   terminal event is not a reliable end-of-stream signal.
2. **Background tasks holding the process open.** Before v2.1.163, a
   background Bash task (dev server, watcher) started during a `-p` run kept the
   invocation alive indefinitely. The fix was a 5-second grace-kill — a
   *workaround*, not a structural guarantee.
3. **No officially enumerated exit-code table.** Community docs converge on
   `0` success / `1` generic / `2` auth-or-arg, but there is no published,
   stable mapping from failure class → exit code. Scripts guess.
4. **Stdin cap (10MB, as of v2.1.128).** Exceeding it exits non-zero with an
   error — a scale boundary the contract did not originally express.
5. **Local context bleed.** Without `--bare`, `-p` loads whatever hooks,
   skills, MCP servers, and `CLAUDE.md` happen to sit in the working dir or
   `~/.claude`, so "the same prompt" yields different results per machine.
   `--bare` was added precisely to make scripted runs reproducible — evidence
   that *implicit context initialization* was the original design's central
   reproducibility bug.

### Structured output contract (payloads, exit codes, events)

- `--output-format json`: one object with `result`, `session_id`, `is_error`,
  `total_cost_usd`, `num_turns`, `duration_ms`; optional `structured_output`
  when `--json-schema` is supplied.
- `--output-format stream-json`: NDJSON; event `type` ∈ {`system`,
  `assistant`, `stream_event`, `result`}, `system` events further keyed by
  `subtype` (`init`, `api_retry`, `plugin_install`).
- Exit codes: loosely `0/1/2` (not formally specified).

### Session / context lifecycle (avoiding redundant initialization)

The session is the unit that avoids re-initialization: capture `session_id`
from the first run, pass it to `--resume` so the second run reuses the prior
conversation rather than rebuilding context. `--continue` is the
"most-recent-in-this-dir" shortcut. `--bare` is the opt-out that skips
auto-discovery entirely for fast, reproducible cold starts.

---

## Phase 2 — Competitive Differentiation

### What they get right that we should replicate

- **First-class machine envelope** with `session_id` + status metadata, not
  just the payload. Callers branch on structure, not string-scraping.
- **Session ID as the resume handle**, scoped to a directory. Caro already has
  the spine for this (`AiSession`, `SessionStore`, `SessionMode`).
- **A `--bare`-style "reproducible cold start"** — pass only explicit flags,
  ignore ambient config. This is Caro's natural home turf (it is already a
  single-shot CLI), so we get it nearly for free.
- **An explicit terminal event** so consumers know the run is done.

### Their bugs / design gaps we avoid by designing the schema first

| Their failure mode | Our design-time fix |
| --- | --- |
| Stream hangs after final `result` event (#25629) | Single-shot subprocess: write **one** JSON object (or a bounded NDJSON stream that *ends with EOF*), flush stdout, `return`/`exit` deterministically. No agent loop left running, no open stdout. |
| Background tasks hold process open | Caro never spawns background tasks in generate-mode; generation → validation → emit → exit is synchronous and bounded by the existing backend timeout. |
| No enumerated exit codes | Ship a closed `ExitStatus` enum (§ Exit/Output Contract) with a documented integer for every outcome from day one. |
| Implicit context init differs per machine | Caro's JSON mode is pure: inputs are the prompt + flags + persisted session file; no hook/skill/MCP auto-discovery to drift. Reproducibility is the default, not an opt-out. |
| `is_error` boolean conflates failure classes | We separate **transport success** (did we produce a valid response object?) from **safety verdict** (`allowed`) from **process exit class**. A blocked-but-correctly-analyzed command is `exit 3`, not a generic error. |

### Our unique positioning (what they cannot do)

- **Offline / local-first.** No API key, no network, no `total_cost_usd` —
  the embedded backend runs the model locally. JSON mode works air-gapped.
- **Safety verdict is a structured first-class field.** Caro's whole reason to
  exist is `ValidationResult` (risk level + matched patterns). The JSON
  contract can expose *why* a command is dangerous, with machine-readable
  `risk_level` and `matched_patterns` — something a generic agent CLI has no
  concept of.
- **Deterministic by construction.** Same prompt + same session file + same
  backend ⇒ same JSON. No hidden context discovery.
- **Universal shell targeting.** `shell` is already a typed input
  (`ShellType`), so the JSON contract can be parameterized per shell for
  cross-platform CI.

### Existing infrastructure that already covers part of this

The codebase is ~80% there. Confirmed by source inspection:

| Capability | Where it already lives | State |
| --- | --- | --- |
| Resume-or-new session logic | `src/ai/runner.rs` → `run_once()` + `SessionMode::{ResumeOrNew,New,ResumeStrict}` | ✅ done |
| Persisted, serializable session | `src/ai/session.rs` → `AiSession`, `Turn`, `Role` (all `Serialize/Deserialize`), `SessionStore` | ✅ done |
| Structured outcome of one run | `src/ai/runner.rs` → `AiOutcome { session_id, command, explanation, confidence, risk, warnings, allowed, resumed, warns_offhost }` | ⚠️ exists but **not `Serialize`** |
| Safety verdict struct | `src/safety/mod.rs` → `ValidationResult { allowed, risk_level, explanation, warnings, matched_patterns, confidence_score }` | ✅ already `Serialize` |
| Risk taxonomy | `src/safety/mod.rs` → `RiskLevel {Safe,Moderate,High,Critical}` (`#[serde(rename_all="lowercase")]`) | ✅ already `Serialize` |
| Output-format flag | `src/main.rs` → `--output` (`Option<String>`, "json/yaml/plain") | ⚠️ parsed but **not wired** to emit a stable contract on the NL path |
| Per-run timing | `src/cli/mod.rs` → `TimingInfo { generation_time_ms, execution_time_ms, total_time_ms }` | ✅ already `Serialize` |
| Special exit code precedent | `src/main.rs` → `EXIT_CODE_EDIT: i32 = 201` | ⚠️ ad-hoc constant, no enum |

**Implication:** this is mostly a *wiring + serialization-hardening* job, not a
new subsystem. The constraint "reuse existing validator/safety/config
infrastructure; do not duplicate" is satisfiable almost entirely by composition.

---

## Phase 3 — Scope Definition

### Proposed ADR-024 (context / decision / consequences / alternatives)

**Title:** ADR-024 — Headless JSON Output & Session-Resume Contract
**Status:** Proposed
**Date:** 2026-06-18

**Context.** Caro is already a single-shot CLI with a persisted session store,
a serializable safety verdict, and an unfinished `--output json` flag. Automation
callers (CI typo-linters, git hooks, the existing `caro-mcp-server` from
ADR-015, SDK wrappers) currently have to scrape human-formatted stdout. The
competitor reference (Claude Code headless mode) proves demand for this contract
but ships with a structural defect — the stream can hang after its terminal
event because the agent loop and stdout stay open. Caro can offer the same
contract *without* that defect because generation in Caro is synchronous and
bounded.

**Decision.** Add a **headless JSON mode** that emits exactly one
schema-versioned JSON object to stdout and exits with a closed, documented
`ExitStatus` code. It is triggered by the existing `--output json` flag (no new
top-level flag). It composes the existing `run_once()` outcome and
`ValidationResult` into one envelope, threads `--resume <id>` / `--session-id`
into the existing `SessionMode`, and guarantees flush-then-exit with no
lingering work. `stream-json` (NDJSON) is explicitly **out of scope for v1**
(see below) because Caro's bounded single-turn generation has nothing to stream
incrementally that a single object cannot carry.

**Consequences.**
- *Positive:* machines get a stable, offline, reproducible contract; the
  `caro-mcp-server` and any CI wrapper drop their stdout-scraping; the
  competitor's #1 failure mode (stream hang) is impossible by construction.
- *Positive:* `AiOutcome` becoming `Serialize` makes every future structured
  surface (logs, telemetry, MCP) cheaper.
- *Negative / cost:* the JSON envelope becomes a **compatibility surface** —
  once scripts depend on `schema_version: 1`, fields are append-only. This is
  the deliberate price of the contract and is mitigated by the
  `schema_version` field.
- *Negative:* a second output path through `main.rs` must be kept behaviorally
  in sync with the human path (mitigated by routing both through `run_once()`).

**Alternatives considered.**
1. *Reuse `CliResult` directly as the JSON schema.* Rejected: `CliResult`
   (src/cli/mod.rs) carries presentation concerns (`confirmation_prompt`,
   `debug_info`, `output_format`) that are not a stable machine contract and
   would leak internal churn into the public schema.
2. *Ship `stream-json` now to match the competitor 1:1.* Rejected: it imports
   the exact failure mode we want to avoid (open-stdout end-of-stream
   ambiguity) for a streaming benefit Caro's single-turn model does not need.
3. *New `caro json` subcommand.* Rejected: duplicates the NL pipeline; the
   constraint is "no new modules unless unavoidable." A flag on the existing
   path is sufficient.
4. *Emit JSON only for `--dry-run`.* Rejected: automation needs the contract
   for executed runs too; coupling format to execution mode is the kind of
   conflation (Claude Code's `is_error`) we are explicitly avoiding.

### New types (added to existing modules — no new module)

**1. `ExitStatus` — add to `src/main.rs`** (replaces the bare
`EXIT_CODE_EDIT: i32 = 201` constant; that value is preserved as a variant).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(i32)]
pub enum ExitStatus {
    Success         = 0,   // command generated, allowed (and executed if requested)
    GenericError    = 1,   // unclassified failure
    UsageError      = 2,   // bad args / invalid flag combination
    BlockedBySafety = 3,   // generated but validator denied it (allowed == false)
    BackendError    = 4,   // inference backend unavailable / timed out
    SessionError    = 5,   // resume requested but session not found / store unreadable
    EditMode        = 201, // preserves existing EXIT_CODE_EDIT semantics
}

impl ExitStatus {
    pub fn code(self) -> i32 { self as i32 }
}
```
- *Serialization:* `Serialize` so the code is echoed inside the JSON envelope
  (`exit.status` + `exit.code`) as well as returned as the process exit code.
- *Method contract:* `code(self) -> i32` is the single source of truth used
  by `std::process::exit(status.code())`. Every current bare `exit(1)` /
  `exit(201)` call site migrates to a variant.

**2. `CaroJsonResponse` — add to `src/cli/mod.rs`** (the envelope; thin,
append-only, composed from existing serializable types).

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CaroJsonResponse {
    pub schema_version: u32,            // == 1; bump only on breaking change
    pub session_id: u64,               // from AiOutcome.session_id
    pub resumed: bool,                  // from AiOutcome.resumed
    pub input: String,                  // the NL prompt as received
    pub shell: ShellType,               // already Serialize
    pub command: String,                // generated command (empty if blocked pre-gen)
    pub explanation: String,
    pub confidence: f64,
    pub safety: ValidationResult,       // REUSED verbatim from src/safety/mod.rs
    pub warnings: Vec<String>,
    pub executed: bool,                 // false in --dry-run
    pub exit: ExitInfo,
    pub timing: TimingInfo,             // REUSED from src/cli/mod.rs
}

#[derive(Debug, Clone, Serialize)]
pub struct ExitInfo {
    pub status: ExitStatus,             // serializes to "success" | "blocked_by_safety" | …
    pub code: i32,                      // status.code(); duplicated for jq convenience
}
```
- *Contract:* built by one constructor `CaroJsonResponse::from_outcome(input,
  shell, &AiOutcome, &ValidationResult, &TimingInfo, executed, ExitStatus)`,
  so there is exactly one place mapping internal state → public schema.
- *No new safety/validation logic* — `safety` is the existing
  `ValidationResult`, `timing` the existing `TimingInfo`.

**3. Make `AiOutcome` serializable — edit `src/ai/runner.rs`.**
Add `Serialize, Deserialize` to its existing `#[derive(Debug, Clone)]`. Its
fields are already all serializable (`u64`, `String`, `f64`, `RiskLevel`,
`Vec<String>`, `bool`). Satisfies the "all new types serializable from day one"
constraint and the "do not duplicate" constraint (we serialize the canonical
outcome rather than inventing a parallel one).

### Minimal set of files that change (no new modules)

| File | Change |
| --- | --- |
| `src/ai/runner.rs` | Derive `Serialize, Deserialize` on `AiOutcome`. (~1 line) |
| `src/cli/mod.rs` | Add `CaroJsonResponse`, `ExitInfo`, and `from_outcome()`; reuse existing `ValidationResult`/`TimingInfo`. |
| `src/main.rs` | Add `ExitStatus` enum; migrate existing `process::exit` / `EXIT_CODE_EDIT` call sites; when `--output == "json"`, route the NL command through `run_once()`, build `CaroJsonResponse`, `serde_json::to_writer(stdout)`, flush, `exit(status.code())`. Add `--resume <id>` and `--session-id` plumbing into the existing `SessionMode`. |
| `docs/adr/ADR-024-headless-json-mode.md` + `docs/adr/README.md` | New ADR doc + index row (next sequential number is **024**; last on disk is ADR-023). Per `.claude/rules/adr-numbering.md`, renumber on merge if it lands out of order. |
| `tests/headless_json.rs` (new test file, not a new src module) | Integration tests (below). A test file is permitted; "no new modules" refers to `src/`. |
| `CHANGELOG.md` | `Added` entry under the next release. |

No changes to `src/safety/`, `src/inference/`, `src/platform/`, or
`src/config/` — they are consumed, not modified.

### Exit-code / output contract (what machines depend on)

**Stdout:** exactly one JSON object matching `CaroJsonResponse`, terminated by
a single `\n`, then **EOF**. stderr carries human/log lines only (so
`caro --output json … 2>/dev/null | jq` is always clean). This is the
structural fix for the competitor's stream-hang: a single object + closed
stdout + immediate `exit()` makes "is it done?" unambiguous.

**Exit codes (closed set, documented from day one):**

| Code | `ExitStatus` | Meaning |
| --- | --- | --- |
| 0 | `Success` | Command generated and allowed (executed if not `--dry-run`). |
| 1 | `GenericError` | Unclassified internal failure. |
| 2 | `UsageError` | Bad flags / invalid combination (e.g. `--resume` + `--session-mode new`). |
| 3 | `BlockedBySafety` | Generated but validator denied (`safety.allowed == false`). Still emits a full JSON object. |
| 4 | `BackendError` | Inference backend unavailable or timed out. |
| 5 | `SessionError` | `--resume <id>` not found, or session store unreadable. |
| 201 | `EditMode` | Preserves existing edit-mode shell-capture semantics. |

A caller can branch on the exit code alone (no parsing) for the common
success/blocked/backend cases — the property the reference implementation never
formally guaranteed.

### Integration tests (known input → deterministic JSON + exit code)

In `tests/headless_json.rs`, run the built binary against the **mock backend**
(`BackendType::Mock`) so output is deterministic and offline:

1. **Safe command, dry-run.**
   Input: `caro --output json --dry-run --backend mock "list files"`
   Assert: exit `0`; parsed `schema_version == 1`; `safety.allowed == true`;
   `safety.risk_level == "safe"`; `executed == false`; `exit.code == 0`;
   stdout is exactly one JSON object (one `\n`, then EOF).
2. **Dangerous command is blocked.**
   Input: `caro --output json --backend mock "wipe the root filesystem"`
   Assert: exit `3`; `safety.allowed == false`; `safety.risk_level` ∈
   {`high`,`critical`}; `safety.matched_patterns` non-empty; a full JSON object
   is still emitted (blocked ≠ no-output).
3. **Resume a known session.**
   Pre-seed a session file; `caro --output json --resume <id> --backend mock
   "now sort them"`. Assert: exit `0`; `resumed == true`; `session_id ==
   <id>`.
4. **Resume miss.**
   `caro --output json --resume 999999 --backend mock "x"`. Assert: exit `5`
   (`SessionError`); JSON `error`/`exit.status == "session_error"`.
5. **Schema stability / golden file.**
   Snapshot test of case 1's object against `tests/fixtures/headless_v1.json`
   (with timing fields normalized to `0`) to catch accidental
   schema-breaking field renames.
6. **Clean-exit guarantee (the failure-mode test).**
   Run case 1 under a wall-clock timeout in the test harness; assert the
   process exits on its own (no SIGKILL needed) — directly regression-guards
   the competitor's stream-hang defect.

All six are deterministic (mock backend, normalized timing) and assert on both
the parsed JSON and the exit code, per the constraint.

### Explicitly out of scope (next version)

- **`stream-json` / NDJSON event stream** and a `system/init`-style preamble.
  Deferred until a multi-turn agent surface exists that has something to stream;
  v1's single object is sufficient for the bounded generate→validate→emit loop.
- **`--json-schema` arbitrary structured output.** Caro's payload is fixed
  (a command + verdict), so user-supplied output schemas add no value yet.
- **Cost / token accounting fields** (`total_cost_usd`). Meaningless for the
  offline embedded backend; revisit only if remote-backend billing surfaces
  demand it.
- **`--continue` (most-recent-in-dir) shorthand.** v1 ships explicit
  `--resume <id>` only; `--continue` is sugar over the same `SessionMode` and
  can land later.
- **Multi-object / batch mode** (many prompts in one invocation). One prompt,
  one object, one exit — keeps the subprocess contract pure.
- **`--bare`-equivalent context-discovery toggle.** Caro's JSON mode is already
  context-pure, so no opt-out is needed; only revisit if local-context-indexing
  (ADR-017) starts injecting ambient state into generation.

---

## Constraint compliance check

| Constraint | How this scope satisfies it |
| --- | --- |
| Reuse existing validator/safety/config; do not duplicate | `safety: ValidationResult` and `RiskLevel` reused verbatim; generation routed through existing `run_once()`; no new safety logic. |
| All new types serializable from day one | `CaroJsonResponse`, `ExitInfo`, `ExitStatus` derive `Serialize`; `AiOutcome` upgraded to `Serialize`. |
| Pure subprocess (no daemon, no state) | Single invocation: generate → validate → emit one object → flush → `exit`. Session state is a file on disk, read/written within the one run, never a resident process. |
| Solve the Phase-1 failure mode by design | The stream-hang (#25629) is structurally impossible: one object, stdout closed, deterministic `exit(code)`; no background tasks; fully enumerated exit codes replace the competitor's unspecified `0/1/2`. |

---

## Sources

- [Run Claude Code programmatically (headless) — Claude Code Docs](https://code.claude.com/docs/en/headless)
- [Claude Code stream-json output format — Background Claude](https://backgroundclaude.com/blog/stream-json)
- [BUG: Claude Code CLI hangs after final result event in stream-json mode — anthropics/claude-code#25629](https://github.com/anthropics/claude-code/issues/25629)
- [Headless Mode and CI/CD (exit codes, errors) — SFEIR Institute](https://institute.sfeir.com/en/claude-code/claude-code-headless-mode-and-ci-cd/errors/)
- [Claude Code in CI/CD and Headless Automation — hidekazu-konishi.com](https://hidekazu-konishi.com/entry/claude_code_cicd_and_headless_automation.html)

*Caro source references (read-only inspection, unmodified):* `src/main.rs`
(`Cli`, `--output`, `EXIT_CODE_EDIT`), `src/ai/runner.rs` (`AiOutcome`,
`run_once`, `SessionMode`), `src/ai/session.rs` (`AiSession`, `Turn`,
`SessionStore`), `src/safety/mod.rs` (`ValidationResult`, `RiskLevel`),
`src/cli/mod.rs` (`CliResult`, `OutputFormat`, `TimingInfo`),
`docs/adr/` (last sequential ADR on disk: ADR-023).
