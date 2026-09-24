# ADR-041: caro-events Phase 0 — Standalone Lifecycle-Event Emitter (De-Risked Cut of ADR-037)

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-28
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: Claude Code headless `--output-format stream-json`
  (NDJSON lifecycle events; undocumented/experimental — see Phase 1) and the
  OTel GenAI semconv findings inherited from ADR-037
- **Revises**: ADR-037 (lifecycle-event-schema) — same schema, smaller v1 cut
- **Relates to**: ADR-024/026/031/032 (all Proposed, **none implemented** as
  of this date — that fact is this ADR's reason to exist), lowercase ADR-004
  (stream-json research), ADR-020 (tier vocabulary), ADR-021 (`--agent-id`)
- **Numbering note**: highest existing is ADR-040; per
  `.claude/rules/adr-numbering.md`, renumber on merge if another 041 lands
  first.

> **Provenance note (autonomous run).** Produced with no user present; the
> task template's `[FEATURE NAME]` was unbound. Target selection: the
> 2026-07-27 and 2026-07-28 Hermes scans both name the structured
> assessment/lifecycle event stream as the single top build item ("two
> consecutive scans surfacing the same top item is a convergence signal").
> ADR-037 already defines the schema, so this run scoped the missing piece:
> an implementable first increment. Treat the de-scoping decisions below as
> reviewable, not settled.

---

## Context

### Phase 1 — the analog, re-verified

Claude Code's headless `--output-format stream-json` emits NDJSON lifecycle
events (`system/init` → message events → terminal `result` with stats). Its
documented and community-reported failure modes, current as of July 2026:

1. **Undocumented, unversioned event shapes.** Event types are coupled to the
   binary version; no `schema_version` on any line; open issues ask Anthropic
   to document the types at all (anthropics/claude-code #24596, #24612).
2. **Flag coupling**: `stream-json` silently requires `--verbose` (and
   `--include-partial-messages` for token deltas) — the contract is activated
   by an unrelated diagnostic flag.
3. **Lifecycle hangs**: unbounded stdin and background tasks can hold the
   process open; the stream can interleave or stall.
4. **No assessment vocabulary**: like OTel GenAI semconv (`execute_tool` but
   no "assess"/"decide" span — ADR-037 failure mode 3), the stream describes
   *doing*, never *deciding*. A blocked action has no first-class
   representation.

ADR-037 already answers 1–4 at the schema level (version-per-line, dedicated
flags, closed 8-variant taxonomy with `assessment_completed` /
`decision_made`). What it does not answer is a fifth failure mode, ours not
theirs:

5. **The dependency-chain failure mode.** ADR-037 specs emission "on the
   existing ADR-024/026 NDJSON stream", bound to the ADR-032 receipt chain,
   including ADR-031's `session_halted`. A grep of `src/` on 2026-07-28 finds
   **zero** of those: no `HeadlessEnvelope`, no `ExitCode` enum, no session
   stream, no `ExecutionReceipt`, no `--agent-id`, no `docs/schemas/`. Five
   scoping runs have produced five Proposed ADRs and no shipped contract,
   while two more vendors (Dynatrace, Microsoft, Jul 28) made per-decision
   audit logs a procurement criterion and EU AI Act Phase 2 lands in August.
   A schema that requires four unbuilt ADRs is itself demoware — impressive
   on paper, unable to handle real-world usage, because there is no code
   path that can emit it.

**Decision principle for this ADR: solve failure mode 5 by design — every
item in scope must be implementable against code that exists in `src/` today.**

### Phase 2 — what exists today (verified against source, not ADRs)

| Need | Exists now | Where |
|---|---|---|
| Event envelope idiom (`tag = "type"`, flatten, uuid, `DateTime<Utc>`) | yes | `src/telemetry/events.rs` |
| Append-only JSONL writer precedent | yes | `src/caroml/history.rs` (`append_to`) |
| Advisory file locking | dep present (`fd-lock` 4.0), unused for JSONL | `Cargo.toml` |
| Assessment payload | `ValidationResult` (Serialize; lacks `JsonSchema`) | `src/safety/mod.rs:167` |
| Decision payload | `SafetyDecision` + `SuggestedRouting` (`auto_approve\|async_log\|human_gate\|block`) | `src/safety/mod.rs:180`, `src/models/mod.rs:189` |
| Generation metadata | `GeneratedCommand` (`backend_used`, `generation_time_ms`, `confidence_score`) | `src/models/mod.rs:71` |
| Hash-not-content precedent | `stderr_digest` (SHA-256); `sha2` 0.10 in-tree | `src/caroml/history.rs` |
| Secret detection | `Redaction::redact` / `contains_sensitive` | `src/logging/redaction.rs` |
| Non-breaking flag insertion | `IntoCliArgs` defaulted methods | `src/cli/mod.rs:145` |
| Schema generation | `schema_for!` pattern (schemars 0.8) | `src/bin/generate-schema.rs` |
| No-op-safe global emitter pattern | `OnceLock` + `emit_event` | `src/telemetry/mod.rs` |
| Safety validation call site | `SafetyValidator` on `CliApp` (post-generation) | `src/cli/mod.rs:33-41` |
| Execution call sites | executor/shell | `src/execution/` |

Also verified: exit codes today are ad-hoc `{0, 1, 201}` (`EXIT_CODE_EDIT =
201` for the shell-wrapper edit flow); there is no `ExitCode` enum anywhere.
The telemetry `EventType` must **not** be reused for audit events: telemetry
is metadata-only, anonymous, opt-out, uploaded; the audit stream is
per-invocation, identified, local-first. Shared idiom, separate enum.

### Differentiation (unchanged from ADR-037, restated)

Claude Code streams execution telemetry for *its own* agent, unversioned.
Caro Phase 0 ships a **versioned, schema-published, offline, agent-agnostic**
record of assess → decide → execute, including commands that never ran — as
a pure subprocess, no daemon, no collector. That is the artifact Dynatrace
and Microsoft told their customers to go build.

---

## Decision

Ship **`caro-events` schema v1, Phase 0**: the ADR-037 envelope and taxonomy
**minus every variant whose producer does not exist**, emitted to a single
sink (`--events-file <path>`, append-only JSONL, advisory-locked), teed from
the existing CLI flow. ADR-037 remains the schema authority; this ADR is the
implementation cut.

### Event taxonomy (Phase 0 — 5 of ADR-037's 8 variants)

| # | `type` | Emitted from | Payload core |
|---|--------|--------------|--------------|
| 1 | `run_started` | `run_cli` entry | `caro_version`, `backend`, `model_name?`, `shell`, `mode` (`"cli"` literal in Phase 0) |
| 2 | `assessment_completed` | after `SafetyValidator::validate_command` in `CliApp` | `command_hash`, embedded `ValidationResult` verbatim |
| 3 | `decision_made` | where `SafetyDecision`/blocked-reason is resolved | `decision` (`allow`\|`deny`\|`ask`), `actor` (`policy`\|`human`\|`auto`), `reason`, `suggested_routing` |
| 4 | `execution_started` | `src/execution/` dispatch | `command_hash` |
| 5 | `execution_completed` | wait() return | `exit_code`, `duration_ms`, `stdout_hash`, `stderr_hash` |

Deferred to their owning ADRs (see Out of scope): `session_halted` (031),
`receipt_written` (032), `result` (024/026).

Invariant, stated as in ADR-037: a blocked command emits `1,2,3(deny)` and
**no** execution events — the pre-execution audit trail that after-the-fact
recorders structurally cannot produce.

### Envelope

Exactly ADR-037's `EventEnvelope` (`schema: "caro-events/1"`, monotonic
`seq`, `ts`, `invocation_id`, optional `session_id`/`agent_id`, flattened
`CaroEvent`), with two Phase-0 notes:

- `agent_id` is populated from `CARO_AGENT_ID` env only (the `--agent-id`
  flag is ADR-021's to add); field is present in the schema from day one.
- `session_id` is always `None` in Phase 0 (no session stream exists);
  field present, reserved.

Emitting 5 of 8 variants is legal under ADR-037's additive-only rule:
consumers must ignore unknown variants, so Phase 1+ adds the remaining three
without a version bump.

### Redaction: hash-only by design (Phase 0)

Phase 0 events carry **no raw command, prompt, stdout, or stderr — only
SHA-256 hashes** (`command_hash`, `stdout_hash`, `stderr_hash`), following
the `caroml/history.rs` digest precedent. This eliminates ADR-037's failure
mode 5 (PII in log pipelines) by construction instead of by sanitizer:
there is nothing to redact. The one free-text field that crosses the
boundary — `ValidationResult.explanation` / `warnings` / pattern
descriptions — is static program text, not user input; as belt-and-braces
the emitter passes the serialized line through
`logging::redaction::contains_sensitive` and drops to `redacted:true` +
`Redaction::redact` output if it trips. Content-by-consent
(`--receipt-visibility`-style knob) is ADR-032's decision and is out of
scope here; when it lands, the same knob governs events per ADR-037.

### Sink and flags

- `--events-file <path>`: append-only JSONL, `OpenOptions::append` +
  `fd-lock` advisory lock per write, one `EventEnvelope` per line. No stdout
  emission in Phase 0 — stdout stays owned by the existing `-o
  json|yaml|plain` contract, so Phase 0 cannot break any current consumer.
  (stdout NDJSON arrives with ADR-024/026, where it belongs.)
- `--require-events`: opt-in fail-closed. If the sink cannot be opened or a
  write fails **before execution**, the command is not executed.
- Both flags enter via `IntoCliArgs` defaulted methods (non-breaking) plus
  the clap `Cli` struct in `src/main.rs`.
- Config: `#[serde(default)] pub events: EventsConfig` on
  `UserConfiguration` (`events_file: Option<PathBuf>`, `require_events:
  bool`), deriving the full house incl. `JsonSchema`, mirrored in the
  builder's three sites per the established pattern.

### Emitter mechanics

A small `EventSink` struct owned by `CliApp` (not a global): holds the
locked file handle, `invocation_id` (UUID v4), and an `AtomicU64` seq.
`fn emit(&self, event: CaroEvent) -> Result<(), EventSinkError>`; callers
ignore the error unless `require_events` is set. No daemon, no thread, no
buffering — a failed single-line append is visible immediately, which is
what fail-closed requires. Sub-100-line impl.

### Exit codes / output contract (what machines depend on)

- Exit codes **unchanged**: `{0, 1, 201}` as today. `--require-events` sink
  failure exits `1` with a single stderr line
  `caro: events sink unavailable: <detail>` — greppable, and documented as
  such. A dedicated code is **reserved, not assigned**: it moves into
  ADR-024's `ExitCode` enum when that enum exists (this avoids inventing a
  code that 024 would immediately renumber).
- `--events-file` content: v1 envelopes only, UTF-8, one JSON object per
  line, `seq` monotonic from 0 per invocation, gap = dropped event.
- Stability: within `caro-events/1`, fields are never removed or renamed;
  unknown fields and unknown `type` values must be ignored by consumers.
- Machine-readable schema: `docs/schemas/caro-events-1.schema.json`,
  generated via the existing `generate-schema` binary pattern (schemars
  0.8 — do not upgrade schemars for this; ADR-037's compatibility note
  applies), committed and diffed in CI.

## Minimal file set (no new modules)

| # | File | Change |
|---|------|--------|
| 1 | `src/models/mod.rs` | `EventEnvelope`, `CaroEvent` (5 variants), `EventsConfig`; serde + `JsonSchema` from day one; `UserConfiguration` + Default + builder ×3 |
| 2 | `src/main.rs` | two clap flags; `IntoCliArgs` impl additions |
| 3 | `src/cli/mod.rs` | `EventSink` (construction, emit, fail-closed gate); emit calls at assessment/decision sites; defaulted `IntoCliArgs` methods |
| 4 | `src/execution/mod.rs` (or `executor.rs`) | emit hooks at dispatch/complete behind existing call sites, no logic change |
| 5 | `src/safety/mod.rs` | one-line `JsonSchema` derive on `ValidationResult` |
| 6 | `src/bin/generate-schema.rs` + `docs/schemas/caro-events-1.schema.json` | emit the events schema alongside the config schema |
| 7 | `tests/lifecycle_events.rs` | integration tests below |

(7 files; item 6 is generator + committed artifact.)

## Integration tests (known input → deterministic JSON + exit code)

1. **Blocked chain**: `caro --events-file <tmp> "delete everything under /"`
   (matches built-in Critical pattern, deterministic via static patterns) →
   events exactly `run_started, assessment_completed, decision_made(deny)`;
   no `execution_*`; process exit `1`; golden-file with
   `ts`/`invocation_id`/hashes normalized.
2. **Allowed dry-run chain**: benign command + `--dry-run` →
   `run_started, assessment_completed, decision_made(allow)`, no execution
   events, exit `0`.
3. **Executed chain**: benign command + `-x -y` → all 5 events in order;
   `execution_completed.exit_code == 0`; `seq` strictly `0..=4`.
4. **Schema conformance**: every line from tests 1–3 validates against
   `caro-events-1.schema.json` (`jsonschema` crate, dev-dependency — the
   one new dep, dev-only).
5. **Fail-closed**: `--require-events --events-file /nonexistent-dir/x` →
   command not executed, exit `1`, stderr contains
   `events sink unavailable`.
6. **Fail-open default**: same unwritable path without `--require-events` →
   command proceeds normally, warning on stderr, exit `0`.
7. **Hash-only guarantee**: raw command string never appears in any emitted
   line for a command containing a sentinel secret
   (`API_KEY=sk-sentinel...`); `command_hash` matches SHA-256 of the
   command.

All use `assert_cmd` + `tempfile` (already dev-dependencies); static-matcher
paths keep them deterministic with no LLM backend required.

## Out of scope (owned by later phases / other ADRs)

- stdout NDJSON stream, `result` terminal event, `HeadlessEnvelope`,
  `ExitCode` enum → ADR-024/026 (Phase 1: emitter tees to that stream when
  it exists).
- `session_halted` → ADR-031. `receipt_written` + content-visibility knob +
  hash-chain binding → ADR-032. `--agent-id` flag → ADR-021.
- Agent-loop generation-stage events (static-matcher hit, repair attempts)
  — telemetry covers aggregate counts today; per-run generation events are
  a v2 taxonomy addition.
- OTLP/OTel exporter (mapping table in docs only), per-event signing,
  SIEM-specific adapters, `caro events tail` tooling, MCP notification
  bridging — all per ADR-037's list.
- Windows lock semantics beyond `fd-lock` defaults.

## Alternatives considered

1. **Implement ADR-037 as written** (stream + receipts + sessions) —
   rejected: requires ADR-024/026/031/032 first; measured cost of that
   ordering is five Proposed ADRs and zero shipped contract since 2026-06-25.
2. **Sequence ADR-024 first, then 037** — viable but slower; 024's envelope
   redesigns stdout for all consumers, a strictly larger blast radius than
   an additive opt-in file sink. Phase 0 ships value in one PR and 024 can
   land on top unchanged.
3. **Reuse `telemetry::EventType`** — rejected: privacy postures are
   opposite (anonymous/aggregate/uploaded vs identified/per-run/local);
   conflating them either leaks content into the uploader or cripples the
   audit stream.
4. **Emit via `tracing` JSON layer** — rejected: subscriber-formatted,
   unversioned, no additive-only guarantee — exactly the Claude Code
   stream-json failure mode we're differentiating against.
5. **Wait for the MCP server (ADR-015)** — rejected: hook hosts and CLI
   consumers need the contract now; these types feed MCP later unchanged.

## Consequences

**Positive**: a shippable one-PR increment of the two-scans-running top
priority; the assess→decide→execute chain becomes exportable this quarter;
schema v1 published and CI-diffed; ADR-024/026/031/032 inherit a live
contract to extend instead of a paper one; zero change to any existing
stdout/exit-code behavior (pure additive opt-in).

**Negative / risks**: three taxonomy variants ship later than the schema
that names them (mitigated: additive-only rule makes that the designed
path); golden-file tests add CI maintenance; hash-only payloads mean
Phase-0 events alone can't show *what* was blocked without the operator's
own correlation (mitigated: that is the correct privacy default, and
content-by-consent arrives with ADR-032's knob).

**Neutral**: `EXIT_CODE_EDIT = 201` and the `{0,1}` codes are untouched;
when ADR-024's enum lands, the reserved sink-failure code is assigned there
and this ADR's stderr contract remains valid.
