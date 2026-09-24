# Headless Structured-Output Contract — Consolidation & Ratification Scope

- **Date**: 2026-07-24
- **Author**: `caro-research--scoping-process` scheduled task (automated run)
- **Type**: Research report + implementation scope (NOT a new ADR — see "Why no new ADR" below)

> **Provenance.** The task template's `[FEATURE NAME]` placeholder was unbound.
> Following prior-run precedent, this run selected **Claude Code's headless
> mode** (`claude -p --output-format json|stream-json`) as the competitor
> analog, matching the template's emphasis on output contracts, exit codes,
> and pure-subprocess lifecycle. The Phase-2 codebase audit then revealed that
> **five prior runs of this same task already drafted overlapping ADRs for
> this exact feature** (ADR-024 2026-06-23, lowercase 003 2026-06-25,
> ADR-028 2026-06-30, lowercase 004 2026-07-01, plus ADR-025/026/027/029/034),
> all still `Proposed`, none implemented (`HeadlessEnvelope`, `ExitCode`,
> `Ndjson`, `--session` — zero matches in `src/`). Producing a sixth draft
> would worsen the duplication. This run therefore pivots to what the corpus
> actually needs: **fresh competitive verification, a duplication map, and a
> single ratification-ready implementation scope.**

---

## Why no new ADR

`.claude/rules/adr-numbering.md` requires sequential, gap-free numbering.
The corpus currently violates it twice over: two parallel series (uppercase
`ADR-0NN` up to ADR-039, lowercase `00N` up to 006), two files both named
ADR-004 and two named ADR-015, and a `docs/adr/README.md` index that stops at
ADR-015. Adding ADR-040 for content that already exists in ADR-024 would be
pure duplication. The correct move is **ratify ADR-024 as canonical, amend it
with the July-2026 findings below, and mark the duplicates Superseded** — a
housekeeping PR, not a new decision record.

## Duplication map (for the housekeeping PR)

| File | Drafted | Overlap | Recommended disposition |
| --- | --- | --- | --- |
| `ADR-024-headless-json-contract.md` | 06-23 | — | **Canonical.** Amend per §Phase 3, move to Accepted when the implementation PR merges |
| `003-structured-output-session-contract.md` | 06-25 | ≈100% of ADR-024 (envelope + stateless session) | Mark `Superseded by ADR-024` |
| `004-headless-stream-json-event-contract.md` | 07-01 | NDJSON layer of ADR-024 + ADR-028 | Mark `Superseded by ADR-024/ADR-028` |
| `ADR-028-headless-token-streaming.md` | 06-30 | token-delta layer (ADR-024 defers it to v2) | Keep as the v2 streaming ADR; cross-link |
| `ADR-025/026/027/029/034` | 06-2x | adjacent (init cache, multi-turn, permissions, schema-enforced output, degraded results) | Keep as sequenced follow-ons; they depend on ADR-024 landing first |
| `docs/adr/README.md` | — | index stale (stops at ADR-015) | Regenerate full table in the same PR |

---

## Phase 1 — Fresh competitor findings (July 2026)

Re-verified against current docs; these are the deltas since the June drafts.

### Claude Code headless (`claude -p`)

1. **`--bare` is now the documented recommendation and slated to become the
   default for `-p`.** This *validates ADR-024's core bet* (deterministic
   bare-by-default, `--use-config` opt-in). The competitor is converging on
   the design ADR-024 chose from day one — ship it before they finish.
2. **stdin cap is 10 MB and fails loud** with a clear error + non-zero exit
   (v2.1.128+). ADR-024's 1 MiB cap with `error.kind:"stdin_too_large"` is the
   same fail-loud pattern; 1 MiB is defensible for a command-generation tool
   (prompts, not build logs). Keep.
3. **New failure mode discovered — output truncation on slow consumers.**
   Before v2.1.208, piping a large response could truncate the final line and
   omit the `result` message entirely; before v2.1.214 the exit drain wait was
   ~2 s and could cut off large responses. They patched by scaling the drain
   wait with queued output (capped 30 s). **This is the ADR-024-style
   "solve by design" candidate for our run**: caro must flush and drain stdout
   before `process::exit` — non-negotiable in the contract, tested, not
   patched later. Note caro's release profile uses `panic = "abort"`, which
   skips atexit flushing: the JSON/NDJSON writer must explicitly flush after
   the final byte and before computing the exit path.
4. **Signals**: SIGTERM aborts the turn, runs teardown hooks, exits **143**.
   ADR-024's table doesn't address signals; the amendment below documents
   128+n semantics as *outside* the contract table (reserved, never reused).
5. **`system/init` now carries a `capabilities` string array** so consumers
   feature-detect instead of comparing versions. Directly applicable to
   caro's NDJSON `Init` event.
6. **`--json-schema` validation is now loud** (invalid schema → immediate
   error; previously silently ignored). Confirms ADR-029's direction for v2;
   nothing to change in v1.
7. Session continuity remains file/ID-based (`--continue`, `--resume <id>`,
   `session_id` in JSON output), scoped to the working directory. Matches
   ADR-024's `--session <path>` file-snapshot design (caro's is more explicit:
   the caller owns the file, no directory-scoped magic — better for CI).

### OpenAI Codex CLI (`codex exec --json`) — second reference point

- Emits typed NDJSON events: `thread.started`, `turn.started/completed`,
  `item.started/completed` with `item.type` (`agent_message`,
  `command_execution` with `exit_code`/`status`, `file_change`, `web_search`).
- Supports `--output-schema <file>` (schema-enforced output) and
  `-o <file>` (write final message to a file, bypassing stdout races).
- **Design gap to avoid**: transient stream reconnects are emitted as
  `type:"error"` events that consumers must know are non-fatal. Caro's
  taxonomy must never overload `error` — advisory conditions go in
  `warnings`/dedicated event types; an `error` event is always terminal.
- Their `item.exit_code` is null/omitted until completion — confirms
  ADR-024's choice of `Option<i32>` in `ExecutionReport`.

## Phase 2 — What caro already has (audit summary)

- **Plumbing exists, contract doesn't.** `--output {json,yaml,plain}`,
  `--dry-run`, `--quiet` flags already exist; `CliResult` (20 fields,
  Serialize), `TimingInfo`, `GeneratedCommand`, `ValidationResult`,
  `SafetyDecision`, `RiskLevel`, `SuggestedRouting`, file-persisted
  `AiSession`/`Turn`, `serde_json`, `schemars` + a `generate-schema` bin —
  all present.
- **What's missing** (net-new, unchanged from ADR-024): versioned
  `HeadlessEnvelope` decoupled from `CliResult`; one `#[repr(i32)] ExitCode`
  enum replacing ~60 scattered `process::exit(0|1)` literals (+
  `EXIT_CODE_EDIT = 201`); `Ndjson` variant in `OutputFormat`;
  bare-by-default config handling; stdin cap; `tests/headless_contract.rs`.
- **Structural hazard found by the audit**: blocked/confirmation/dry-run
  handling lives *inside* the Plain render branch (`print_plain_output`,
  `src/main.rs` ~L3648+) and calls `process::exit` mid-render. JSON mode
  currently inherits `exit(1)` for blocked results with no machine-readable
  reason. The implementation must lift status determination *above* the
  format branch: compute `(HeadlessStatus, ExitCode)` once, then render.
- CLI-level test precedent exists (`tests/e2e_cli_tests.rs` already pipes
  `--output json` and asserts exit codes with `assert_cmd`).

## Phase 3 — Consolidated implementation scope

**Canonical spec: ADR-024 as written** (types `HeadlessEnvelope`,
`HeadlessStatus`, `ExecutionReport`, `HeadlessError`, `HeadlessEvent`,
`ExitCode` 0–6; six-file change list; seven integration tests; out-of-scope
list). Not restated here. **Six amendments from this run's research:**

- **A1 — Drain-before-exit guarantee** (from finding 3). Contract text:
  "caro writes the complete envelope/final event, flushes stdout, and only
  then exits. Under `panic = "abort"` this flush is explicit, not atexit."
  New integration test 8: pipe NDJSON output through a slow reader
  (e.g. `while read; do sleep 0.05; done`) and assert the last line still
  parses as `Result` with the correct envelope.
- **A2 — Signal semantics documented as reserved.** Exit codes 128+n
  (e.g. 130 SIGINT, 143 SIGTERM) are outside the contract table and will
  never be assigned contract meanings. On SIGTERM caro makes no guarantee an
  envelope was emitted; consumers treat missing `Result` + 143 as aborted.
- **A3 — `capabilities` array in the NDJSON `Init` event** (from finding 5):
  `capabilities: Vec<String>`, v1 value `["envelope_v1"]`. Consumers
  feature-detect; unknown values are ignored. Cheaper than version parsing
  and additive-safe.
- **A4 — `error` events are terminal, advisory conditions are not errors**
  (from the Codex gap). NDJSON taxonomy rule: any event of type `error`
  means the run is ending with non-zero exit; retries/degraded fallbacks
  surface as `warnings` in the envelope (ADR-034 territory), never as
  `error` events.
- **A5 — Status computed above the format branch** (from the audit hazard).
  Refactor requirement, not just contract text: `run_cli` returns
  `(CliResult, HeadlessStatus, ExitCode)`; `print_plain_output` and the
  JSON/NDJSON renderers consume it; no `process::exit` inside render code.
  This is the riskiest part of the diff — it touches the interactive path —
  so it lands as commit 1 with the existing e2e tests as the regression net.
- **A6 — ADR housekeeping in the same PR series**: mark lowercase 003/004
  Superseded, regenerate `docs/adr/README.md` index (stale since ADR-015),
  and record the two-series numbering violation for a follow-up renumber per
  `.claude/rules/adr-numbering.md`.

### Exit-code / output contract (unchanged from ADR-024, restated for machines)

| Exit | Meaning | `status` |
| --- | --- | --- |
| 0 | generated (and executed OK if `--execute`) | `ok` |
| 1 | internal/unexpected error | `error` |
| 2 | usage error (bad flags, stdin > 1 MiB) | `error` |
| 3 | safety validator blocked | `blocked` |
| 4 | needs confirmation, no TTY — not executed | `needs_confirmation` |
| 5 | no backend reachable | `error` |
| 6 | `--execute` ran, command exited non-zero | `ok` + `execution.exit_code != 0` |
| 128+n | signal death — reserved, no contract meaning (A2) | — |
| 201 | `EXIT_CODE_EDIT` shell-wrapper special case — non-headless, documented as excluded | — |

Stability: within `schema_version: "1"`, additive-only; one JSON object on
stdout in `json` mode; all diagnostics to stderr; stdout flushed before exit.

### Delivery plan (feature branch per `.claude/rules/git-workflow.md`)

1. **PR 1 — status lift (A5)**: extract `(status, exit_code)` computation
   from `print_plain_output`; introduce `ExitCode` enum; replace exit
   literals on the generation path. No contract exposure yet. Existing
   `tests/e2e_cli_tests.rs` + `tests/cli_interface_contract.rs` guard it.
2. **PR 2 — contract**: `HeadlessEnvelope` + friends in `src/cli/mod.rs`,
   `Ndjson` variant, bare-by-default + `--use-config`, stdin cap,
   `--session <path>`, drain-before-exit (A1), `docs/headless-contract.md`
   with schemars-generated schema, `tests/headless_contract.rs` (ADR-024's
   7 tests + A1's slow-reader test), driven via the static backend for
   offline determinism.
3. **PR 3 — housekeeping (A6)**: ADR status changes + README index. Small,
   mechanical.

### Out of scope (unchanged, consolidated)

Token-delta streaming (ADR-028, v2) · schema-enforced output (ADR-029, v2) ·
init snapshot cache (ADR-025) · in-process multi-turn agent loop (ADR-026) ·
cost accounting · any daemon/server mode (MCP server ADR-015 owns that).

---

## Recommended next actions (for a human session)

1. Review this report; if the consolidation is agreed, run PR 3's
   housekeeping first or alongside PR 1 — the ADR corpus confusion is
   compounding with every scheduled run.
2. **Fix the scheduled task itself**: bind `[FEATURE NAME]` in the
   `caro-research--scoping-process` SKILL.md, or add an instruction to check
   `docs/adr/` for prior coverage before drafting — that single line would
   have prevented four duplicate drafts.
3. Ratify ADR-024 (Proposed → Accepted) with amendments A1–A5 when PR 2 opens.

## Sources

- [Claude Code — Run Claude Code programmatically (headless)](https://code.claude.com/docs/en/headless)
- [OpenAI Codex — Non-interactive mode](https://developers.openai.com/codex/noninteractive)
- [Codex exec --json event cheatsheet](https://takopi.dev/reference/runners/codex/exec-json-cheatsheet/)
- [Codex CLI exec-mode flag experiments (gist)](https://gist.github.com/alexfazio/359c17d84cb6a5af12bac88fa1db9770)
- Repo: `docs/adr/ADR-024-headless-json-contract.md`, `003-…`, `004-…`,
  `ADR-028-…`; `src/main.rs`, `src/cli/mod.rs`, `src/safety/mod.rs`,
  `src/models/mod.rs`, `tests/e2e_cli_tests.rs`
