# ADR-037: Lifecycle Event Schema — One Versioned, SIEM-Ingestible Event Contract from Assessment to Receipt

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-20
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: OpenTelemetry **GenAI agent semantic conventions**
  (`create_agent` / `invoke_agent` / `execute_tool` spans, `gen_ai.agent.id`;
  spec status: **Development** as of Semantic Conventions 1.40.0, Apr 2026,
  with no published stabilization timeline) and Fetch.ai **AEVS** lifecycle
  receipts (per ADR-032's Phase-1 research; PH launch 2026-07)
- **Depends on**: ADR-024 (headless JSON contract), ADR-026 (multi-turn NDJSON
  session), ADR-031 (session circuit breaker — `session_halted`), ADR-032
  (portable execution receipts)
- **Relates to**: ADR-015 (MCP safety server — same payloads over a different
  transport), ADR-020 (tiered approval), ADR-021 (execution attribution),
  ADR-036 (agent-guard hook adapter — hosts will forward these events),
  Hermes memos 2026-07-17 (opportunity A: "spec the event schema … as an ADR")
  and 2026-07-20 (recommendation 2: "one ADR covers both last week's
  lifecycle-events recommendation and this week's proof-of-execution shift")

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Target selection
> rationale: the 2026-07-20 Hermes scan's top-3 are (1) the MCP validator
> spike — already fully scoped in ADR-015, blocked on execution, not scoping;
> (2) signed assessment receipts folded into a lifecycle-event schema ADR —
> the receipt half exists (ADR-032) but the **event-schema ADR requested on
> 2026-07-17 was never written** (ADRs stop at 036, 2026-07-14); (3) a
> CISO-rubric blog post — content, not engineering scope. This run therefore
> scopes item (2)'s missing half. Treat the analog choice (OTel GenAI
> semconv + AEVS) as a reviewable assumption, not a settled decision.

---

## Context

### The problem and who has it

Teams adopting agents have no audit-ready record of what was proposed, what
caro assessed, what was blocked or approved, by which actor, and what then
actually ran. Caro computes every link of that chain — and emits them as
**four uncoordinated fragments**:

1. ADR-024's headless envelope: one terminal JSON blob, coarse NDJSON
   lifecycle events "small in v1", no schema version on individual events.
2. ADR-026's session stream: `#[serde(tag = "type", rename_all =
   "snake_case")]` input/output events, terminal `result` — taxonomy private
   to the headless session, undocumented for external consumers.
3. ADR-031's `session_halted`: a single bespoke event bolted onto the ADR-026
   stream.
4. ADR-032's `ExecutionReceipt`: a durable hash-chained record — but a
   *separate artifact* with no defined correspondence to the transient events
   a SIEM saw during the run. Events and receipts can silently disagree.

Consumers who need this: SIEMs ingesting agent telemetry (Exabeam added agent
telemetry in July 2026), compliance teams answering the Anthropic CISO
rubric's observability question, agent hosts embedding caro via ADR-036 hooks
or ADR-015 MCP, and auditors replaying "what did the safety layer decide and
why" after the fact.

### Phase 1 — the analogs, and why both are still rough

**OTel GenAI semantic conventions** are the industry's schema for agent
lifecycle telemetry: `create_agent`, `invoke_agent`, `execute_tool` spans,
`gen_ai.agent.id` et al. Status as of Semantic Conventions 1.40.0 (April
2026): **Development**, explicitly experimental, attribute names still
churning behind an `OTEL_SEMCONV_STABILITY_OPT_IN=gen_ai_latest_experimental`
opt-in flag, and "no public timeline for stabilization." Structure: spans in
a trace tree, emitted by an SDK, shipped to a collector.

Failure modes to solve by design, not workaround:

1. **Schema churn without version detection.** OTel's own migration story is
   an environment variable that flips attribute dialects process-wide. A
   consumer cannot look at one event and know which dialect it speaks. Any
   schema we publish must carry its version *in every line*.
2. **Collector-shaped, not subprocess-shaped.** OTel spans presuppose an SDK,
   batching exporter, and collector endpoint. Caro is a short-lived
   subprocess on an offline-capable machine; requiring a collector would
   break the "pure subprocess call, no daemon, no state" invariant that
   ADR-024 established and hosts depend on.
3. **Execution-only vocabulary.** `execute_tool` describes *doing*. There is
   no span for *assessing* or *deciding* — the pre-execution half of the
   chain (verdict, tier, approval actor) has no standard home. AEVS shares
   this gap from the other side: it records tool calls after the fact and
   structurally cannot represent "this command was blocked and never ran."
4. **Events and evidence disagree.** Neither analog binds its live telemetry
   to a durable evidence record. AEVS receipts live behind their API; OTel
   spans live in whatever backend received them. A blocked-command event
   with no corresponding receipt (or vice versa) is undetectable.
5. **PII by default.** OTel GenAI guidance itself warns about prompt/content
   capture. Events that embed raw prompts or full commands leak into log
   pipelines with broader access than the machine that ran the command.

### Phase 2 — what caro already has (do not rebuild)

- `ValidationResult` and `SafetyDecision` (`src/safety/mod.rs`) — already
  `Serialize`/`Deserialize`, already carry `risk_level`,
  `matched_patterns`, `confidence`, `SuggestedRouting`. schemars 0.8 is
  in-tree and imported in the module, but `ValidationResult` does **not**
  yet derive `JsonSchema` — a one-line derive addition, listed below.
- The serde event pattern (`tag = "type", rename_all = "snake_case"`) from
  ADR-026, and the `HeadlessStatus` / exit-code enum from ADR-024.
- `ExecutionReceipt` scope (ADR-032): `schema_version`, `receipt_id`, `seq`,
  `prev_hash`, `agent_id`, `prompt_hash`/`command_hash` redaction levels,
  fail-closed append. The receipt is the durable artifact; this ADR defines
  the *stream* and binds the two.
- Attribution fields: `--agent-id` / `CARO_AGENT_ID` (ADR-021/032, aligned
  with `gen_ai.agent.id`).
- Tiered approval vocabulary (ADR-020): tier, actor, decision source.

What is missing is exactly one thing: **a single versioned event enum, with
full-chain coverage and a published schema, that every surface (CLI headless,
MCP server, guard hook) emits identically.**

### Our unique positioning

Nobody this cycle ships the full chain as one artifact: AEVS records after
execution; Draco blocks in-flight; OTel describes execution telemetry. Caro
can emit **assessment → decision → execution → receipt** as one versioned
stream, offline, deterministic, no collector, receipts included for commands
that *never ran* — the artifact after-the-fact recorders structurally cannot
produce.

---

## Decision

Define **`caro-events` schema v1**: a newline-delimited JSON event contract,
emitted on the existing ADR-024/026 NDJSON stream and optionally teed to a
file sink, with every event carrying `schema` + `seq` + identity fields, and
with the terminal events cross-bound to the ADR-032 receipt chain by
`receipt_id`. Publish the machine-readable JSON Schema as a build artifact.

### Event taxonomy (v1 — closed set, additive-only evolution)

| # | `type` | Emitted when | Payload core |
|---|--------|--------------|--------------|
| 1 | `run_started` | process accepts a task | `caro_version`, `backend`, `shell`, `mode` (`headless` \| `mcp` \| `guard` \| `interactive`) |
| 2 | `assessment_completed` | validator returns | `command_hash`, `command?` (visibility-gated), embedded `ValidationResult` verbatim |
| 3 | `decision_made` | tier resolution (ADR-020) | `decision` (`allow` \| `deny` \| `ask`), `tier`, `actor` (`policy` \| `human` \| `auto`), `reason` |
| 4 | `execution_started` | command dispatched | `command_hash` |
| 5 | `execution_completed` | wait() returns | `exit_code`, `duration_ms`, `stdout_hash`, `stderr_hash` |
| 6 | `session_halted` | ADR-031 breaker trips | ADR-031 payload, unchanged, now a first-class member |
| 7 | `receipt_written` | ADR-032 chain append succeeds | `receipt_id`, `seq`, `chain_path`, `line_hash` |
| 8 | `result` | terminal (ADR-024/026) | existing envelope, plus `receipt_id?` |

Blocked command ⇒ events 1,2,3(`deny`),7,8 — a complete, provable trail with
no execution events. This is the pre-execution receipt story stated as a
stream invariant.

### Envelope (every line, no exceptions)

```rust
// src/models/mod.rs — new; serde + JsonSchema from day one
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct EventEnvelope {
    /// "caro-events/1". Solves analog failure mode 1: dialect is
    /// self-identifying per line, never per process.
    pub schema: String,
    /// Monotonic within the process, from 0. Gap = dropped event.
    pub seq: u64,
    /// ISO-8601 UTC.
    pub ts: DateTime<Utc>,
    /// UUID v4 per process invocation; joins events ↔ receipt.
    pub invocation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<u64>,
    /// Caller identity (--agent-id / CARO_AGENT_ID); OTel gen_ai.agent.id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// The tagged event (taxonomy above).
    #[serde(flatten)]
    pub event: CaroEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CaroEvent { /* 8 variants per table */ }
```

### Design principles (each solves a named Phase-1 failure mode)

- **Version-per-line** (`schema: "caro-events/1"`) — failure mode 1. Schema
  evolution is additive-only within a major; consumers reject majors they
  don't know.
- **No collector, no exporter** — failure mode 2. Sinks are stdout (existing
  stream) and `--events-file <path>` (append, advisory-locked, JSONL). OTel
  interop is a *mapping table in docs* (our field ↔ `gen_ai.*` attribute),
  not a dependency. An OTLP bridge is somebody's 20-line sidecar.
- **Assessment and decision are first-class events** — failure mode 3. This
  vocabulary (verdict + tier + actor) is the moat; it maps directly onto the
  Anthropic CISO rubric's allowed-actions / blast-radius / observability
  questions.
- **Receipt binding** — failure mode 4. `receipt_written.receipt_id` appears
  in the terminal `result`; an auditor joins stream ↔ chain file on
  `invocation_id`/`receipt_id` and detects disagreement mechanically.
- **Hashes by default, content by consent** — failure mode 5. Events carry
  `command_hash`/`stdout_hash`; raw `command` appears only under the ADR-032
  visibility setting (`--receipt-visibility` governs both artifacts —
  one knob, two surfaces).
- **Emission never changes semantics.** Exit codes remain ADR-024's enum
  untouched. Default: sink failure warns on stderr. With
  `--require-events`, sink failure is fail-closed (block execution, distinct
  exit code) — same posture ADR-032 chose for the chain, opt-in here.

---

## Minimal file set (no new modules)

| # | File | Change |
|---|------|--------|
| 1 | `src/models/mod.rs` | `EventEnvelope`, `CaroEvent` + variants (types only) |
| 2 | `src/cli/mod.rs` | emitter (wraps existing NDJSON writer), `--events-file`, `--require-events` flags |
| 3 | `src/execution/mod.rs` | emit hooks at dispatch/complete (behind the existing call sites, no logic change) |
| 4 | `src/safety/mod.rs` | add `JsonSchema` derive to `ValidationResult` (no logic change) |
| 5 | `docs/schemas/caro-events-1.schema.json` | generated from schemars in `build.rs` or a `cargo xtask`; committed, diffed in CI |
| 6 | `tests/lifecycle_events.rs` | integration tests below |

## Output contract (what machines will depend on)

- stdout NDJSON: every line is either a v1 `EventEnvelope` or (during
  migration) a legacy ADR-026 line; consumers filter on `schema` prefix.
- `--events-file`: v1 envelopes only, append-only JSONL.
- Exit codes: unchanged (ADR-024 authority). One addition, reserved not
  assigned here: `--require-events` sink-failure exit (next free code in the
  ADR-024 enum; assign at implementation PR against the then-current enum).
- Stability promise: within `caro-events/1`, fields are never removed or
  renamed; unknown fields must be ignored by consumers.

## Integration tests (deterministic input → fixed events + exit code)

1. **Blocked chain**: `caro --headless "delete everything under /"` (matches
   built-in Critical pattern) → assert event sequence `run_started,
   assessment_completed, decision_made(deny), receipt_written, result`, no
   `execution_*`, exit code = ADR-024 blocked code; golden-file the stream
   with `ts`/`invocation_id` normalized.
2. **Allowed chain**: benign command under `--dry-run` → sequence without
   execution events; with execution → full 8-event ordering asserted.
3. **Schema conformance**: every emitted line validates against
   `caro-events-1.schema.json` (jsonschema crate, dev-dependency).
4. **Receipt binding**: `receipt_written.receipt_id` equals chain-file last
   line's `receipt_id`; tamper with chain file → join detects mismatch.
5. **Fail-closed**: `--require-events --events-file /proc/none/x` → command
   not executed, designated exit code.
6. **Seq monotonicity** across a 3-turn ADR-026 session.

## Out of scope (v2 candidates)

- OTLP/collector exporter or any OTel SDK dependency (mapping doc only).
- Cryptographic signing of events (the ADR-032 chain is the tamper-evidence
  layer; do not duplicate it per-line).
- Observra/Exabeam/OWASP Agentic Top 10 *adapters* — v1 ships the mapping
  table in docs; field-name validation against OWASP/Observra is a
  documentation review task on the implementation PR, not code.
- MCP `notifications/*` bridging of these events (belongs to ADR-015's
  post-spike phases).
- Event replay/query tooling (`caro events tail` etc.).
- Windows advisory-lock semantics beyond what ADR-032 already decides.

## Alternatives considered

1. **Adopt OTel GenAI semconv wholesale** — rejected: Development-status
   schema churn (failure mode 1), collector-shaped (2), and no vocabulary
   for assessment/decision (3). We keep a one-page mapping instead.
2. **Extend ADR-032 receipts to be the only artifact** — rejected: receipts
   are durable state per gated execution; SIEMs want a live stream, and
   sessions need mid-run events (`session_halted`) that aren't receipts.
3. **Per-event HMAC signing (AEVS-style)** — rejected for v1: duplicates the
   chain's tamper evidence, adds key management; revisit with enterprise
   demand (ADR-003 territory).
4. **Do nothing until MCP server ships** — rejected: ADR-036 hook hosts and
   ADR-024 headless consumers need the contract now; the same types feed the
   MCP server later, in line with "reuse, don't duplicate."

## Consequences

**Positive**: one contract across CLI/MCP/hook surfaces; the assess → decide
→ execute → prove chain becomes a single exportable artifact no
after-the-fact recorder can copy; CISO-rubric positioning gets a concrete
mechanism to point at; ADR-015/032/036 all gain a shared vocabulary instead
of three private ones.

**Negative / risks**: schemars 0.8 is pinned in-tree while ADR-015's rmcp
line expects schemars 1.x — the generated-schema build step must not force
an upgrade (two schemars majors can coexist; the events schema generates
from 0.8). Golden-file tests add CI maintenance. The additive-only promise
constrains future renames; that is the point.

**Neutral**: legacy ADR-026 lines remain valid during migration; `schema`
filtering makes the transition mechanical.
