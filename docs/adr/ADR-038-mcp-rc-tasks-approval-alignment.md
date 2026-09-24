# ADR-038: MCP 2026-07-28 RC Alignment — Stateless Validator Server with MRTR Approval Gates and Optional Tasks Compatibility

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-21
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: MCP **2026-07-28 release candidate** (stateless
  core, SEP-2575/2567/2322/2243) and the **MCP Tasks extension**
  (`io.modelcontextprotocol/tasks`, SEP-2663 — experimental in core
  2025-11-25, redesigned and graduated to an official extension in the RC)
- **Depends on**: ADR-015 (MCP safety server — this ADR is the RC-alignment
  audit Hermes has carried since 07-06), ADR-020 (tiered approval — HumanGate
  semantics), ADR-037 (lifecycle event schema — events emitted at each
  approval transition)
- **Relates to**: ADR-024 (headless JSON contract — exit codes reused),
  ADR-027 (headless permission resolution — the subprocess complement of this
  ADR's protocol-native approval), ADR-032 (portable execution receipts),
  ADR-036 (agent-guard hook adapter — same verdict mapping discipline)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Target selection
> rationale: the 2026-07-21 Hermes scan names the MCP validator built
> against the RC as #1 "Build/test next" for the third consecutive week,
> now with a hard external date (spec finalizes **Jul 28**); the "MCP RC
> statefulness audit before the Jul 28 final spec (freshness pass on
> ADR-015)" item has been carried since 07-06 and never executed. This ADR
> is that audit, plus the scope for the one genuinely new design surface
> the RC opened: protocol-native human-approval gates. Treat the
> MRTR-over-Tasks preference (Decision D3) as a reviewable assumption.

---

## Context

### Phase 1 — What the RC and the Tasks extension actually are

**The problem they solve, and for whom.** MCP servers previously required a
protocol-level session: an `initialize`/`initialized` handshake, an
`Mcp-Session-Id` header on every subsequent request, sticky routing, and a
shared session store for horizontal deployments. Server operators paid that
cost even when their server was a pure function. Separately, tool calls
blocked until work finished — impractical for CI pipelines, external job
systems, and (the case caro cares about) **human-in-the-loop approval gates
that block until a person responds**.

**Core architecture of the RC (locked 2026-05-21, final 2026-07-28):**

- **Stateless core** (SEP-2575, SEP-2567): the `initialize` handshake and
  `Mcp-Session-Id` are *removed*. Protocol version, client info, and client
  capabilities travel in `_meta` on every request
  (`io.modelcontextprotocol/clientInfo`, `…/clientCapabilities`); a new
  `server/discover` method returns server capabilities on demand. Any
  request can land on any server instance.
- **Multi Round-Trip Requests (MRTR)** (SEP-2322): a tool call that needs
  user input mid-flight returns an `InputRequiredResult`
  (`resultType: "inputRequired"`) carrying an `inputRequests` map (elicitation
  payloads) and an opaque `requestState` string. The client gathers answers
  and **re-issues the original call** with `inputResponses` plus the echoed
  `requestState`. No long-lived stream; any instance can pick up the retry.
- **Extensions framework** (SEP-2133): reverse-DNS-identified extensions
  negotiated via an `extensions` capability map, versioned independently,
  with their own repositories.
- **Tasks extension** (`io.modelcontextprotocol/tasks`, SEP-2663): a server
  may answer `tools/call` with a `CreateTaskResult`
  (`resultType: "task"` — taskId, initial status, `ttlMs`,
  `pollIntervalMs`). The client drives the lifecycle with `tasks/get`
  (poll), `tasks/update` (fulfill `inputRequests`), and `tasks/cancel`
  (cooperative). Statuses: `working`, `input_required`, `completed`,
  `failed`, `cancelled` (last three terminal). Task creation is
  server-directed; clients that did not advertise the extension get either
  a blocking result or a `-32003` Missing Required Client Capability error.
- Supporting changes that touch caro's contract surface: full JSON Schema
  2020-12 for tool schemas (SEP-2106); missing-resource error moves from
  custom `-32002` to standard `-32602` (SEP-2164); **logging is deprecated
  in favor of stderr for stdio transports** (SEP-2577); W3C Trace Context
  key names fixed in `_meta` (SEP-414).

**Session/context lifecycle — how redundant initialization is avoided.**
The RC's answer is elimination, not caching: there is no handshake to
redo. Capabilities are fetched once via `server/discover` when needed and
cached under server-supplied `ttlMs`/`cacheScope` (SEP-2549); per-request
`_meta` carries the rest. State that applications still need is carried in
**explicit handles** the model threads between calls (the RC blog's
`basket_id` pattern) — visible to the model instead of hidden in transport
metadata.

**Why Tasks is experimental / what its failure modes are.** Tasks shipped
in core 2025-11-25 as experimental; production use forced a redesign and a
demotion-to-extension, which is the strongest available signal about where
the bodies are buried:

1. **The Task Store is a durability obligation.** The spec requires task
   state be "reachable by `tasks/get` even if the worker or connection has
   died," and a `CreateTaskResult` may not be returned until the task is
   findable in the store. For lightweight single-binary servers this
   quietly reintroduces exactly the state the stateless core just removed.
2. **`tasks/list` was removed because it cannot be scoped safely without
   sessions** — an unfixable-by-design hole in the experimental version;
   task IDs are now unguessable bearer tokens and enumeration is forbidden.
3. **The blocking `tasks/result` was replaced by polling** (`tasks/get` +
   `pollIntervalMs`) — the original design held connections open, defeating
   the purpose.
4. **Cancellation is cooperative** — `tasks/cancel` is an intent signal; the
   task may still reach a non-`cancelled` terminal state. Callers that
   treat the ack as a guarantee have a TOCTOU bug.
5. **Migration break**: everyone who shipped against 2025-11-25
   experimental Tasks must migrate. Early adopters of experimental surfaces
   paid the price; adopters of the *extension* start on the redesigned
   lifecycle.
6. **SDK lag**: the Tier-1 beta wave (2026-06-29) covered Python v2,
   TypeScript v2, Go, and C#. **Rust/rmcp is not in the Tier-1 beta set.**
   rmcp reports `ProtocolVersion::V_2026_07_28` and a `Discover` client
   lifecycle mode on main, but its Tasks-extension and MRTR coverage must
   be treated as unverified until the build spike checks it.

### Phase 2 — What this means for caro

**The statefulness audit ADR-015 has been waiting for resolves in caro's
favor.** ADR-015 (2026-05-29, still Proposed; no `src/mcp/`, no `rmcp` in
`Cargo.toml` — the spike never ran) was drafted against the
initialize-handshake world. The RC deleted that world, and everything it
deleted is something caro's validator never wanted: caro's
`SafetyValidator` is a pure function from `(command, shell, safety_level)`
to `SafetyDecision`. The RC's stateless core is not a migration burden for
caro — it is the protocol converging on caro's architecture. What *is*
stale in ADR-015:

| ADR-015 assumption | RC reality | Disposition |
|---|---|---|
| initialize handshake / session lifecycle | Removed; `server/discover` + per-request `_meta` | Update handler surface |
| rmcp 1.x supports "v2025-03-26 / v2025-06-18" | Target `V_2026_07_28`; rmcp not in Tier-1 beta wave | Spike must verify; pin exact version |
| MCP `logging` available for diagnostics | Deprecated; stderr is the documented stdio path | caro already logs to stderr — no change, now spec-blessed |
| Tool schemas draft-07-ish | Full JSON Schema 2020-12 | `schemars` output acceptable; no external `$ref` |
| Custom error codes possible | `-32602` for invalid params; `-32003` reserved for missing capability | Adopt standard codes |
| `SafetyAssessmentOutput`, tool set, exit codes | Unaffected by the RC | **Carry forward unchanged** |

**What the RC newly makes possible — and what this ADR actually scopes.**
ADR-020 wired `HumanGate` to runtime behavior for interactive caro.
ADR-027 defined approval resolution for headless subprocess callers. What
has never existed: a **protocol-native** approval gate, where an MCP client
(any agent host) calls `validate_command`, receives "a human must approve
this," collects the approval through the host's own elicitation UI, and
caro returns the final assessment — all inside the MCP conversation. The
RC provides two mechanisms for exactly this shape, and Hermes 07-21
explicitly flags the Tasks extension as mapping "naturally onto
long-running validation/approval flows."

### The specific failure mode to solve by design

Phase 1's most important finding is failure mode #1: **Tasks reintroduces a
durable state store**, and the task's own constraint list (and ADR-015's
architecture) demands a pure subprocess — no daemon, no state. A naive
"implement Tasks because Hermes said Tasks" scope would violate the
constraint the moment it wrote the task store. The design below solves this
structurally, not with a workaround: the approval gate defaults to **MRTR,
which is stateless by construction**, and caro's determinism makes even the
`requestState` echo tamper-proof — because caro can simply **re-validate
the command on the retry leg** and compare. An LLM-judge safety tool cannot
do this (re-running the judge is expensive and non-deterministic); a
52-pattern regex validator re-derives its verdict in <2ms. Determinism is
the moat, again.

---

## Decision

Ship `caro-mcp` (the ADR-015 binary) targeted at **protocol version
2026-07-28**, with the approval gate built on MRTR and Tasks as an
optional, in-memory, stdio-scoped compatibility layer.

### D1 — Target the RC, not 2025-11-25

`caro-mcp` speaks `2026-07-28` natively: implements `server/discover`,
reads client capabilities from per-request `_meta`, emits standard error
codes (`-32602`, `-32003`), declares tool schemas as JSON Schema 2020-12
(no external `$ref`), and uses stderr for diagnostics. Building against
2025-11-25 in the week the spec finalizes would be instant legacy. Backward
interop with older clients is delegated to rmcp's negotiation (verified at
spike time), not hand-rolled.

### D2 — Tool surface: carry ADR-015 forward, add nothing

Two tools, unchanged from ADR-015: `validate_command`, `validate_batch`.
Payload: `SafetyAssessmentOutput` (schema_version 1) exactly as ADR-015
defines it. This ADR adds **no new tools** — the approval gate changes how
`validate_command` *completes*, not what exists.

### D3 — Approval gate default: MRTR (stateless by construction)

When `validate_command` yields `SuggestedRouting::HumanGate` **and** the
caller passed `approval_mode: "gate"` (new optional request field,
default `"report"` preserves ADR-015's pure-report behavior):

1. caro-mcp returns an `InputRequiredResult` whose `inputRequests` map
   carries one elicitation (`key: "approval"`, boolean schema, message
   naming the command, risk level, and matched patterns), and whose
   `requestState` is a serialized `ApprovalGateState` (below).
2. The client's host shows its own approval UI, then re-issues the call
   with `inputResponses` and the echoed `requestState`.
3. On the retry leg caro-mcp **re-validates the command from scratch**,
   requires the fresh `SafetyDecision` to match the `command_hash` and
   `risk_level` recorded in `ApprovalGateState` (tamper/downgrade check),
   then returns the final `SafetyAssessmentOutput` with a new field
   `approval: {granted, actor: "mcp-client", at}`. Denial returns the
   assessment with `allowed: false`, `suggested_routing: "block"` — a
   *result*, not an error (the assessment succeeded; the command didn't).

No task store, no timer, no state between the two legs. Kill caro-mcp
between leg 1 and leg 2 and the retry still works — the state is in the
payload, and the verdict cannot be forged because it is recomputed.

### D4 — Tasks: optional compatibility layer, in-memory, stdio-scoped

If (and only if) the client advertises `io.modelcontextprotocol/tasks` in
its per-request capabilities and passes `approval_mode: "gate"`, caro-mcp
MAY answer `tools/call` with a `CreateTaskResult` instead of the MRTR
result, then serve `tasks/get` / `tasks/update` / `tasks/cancel` for that
task. Design rules, each pinned to a Phase-1 failure mode:

- **Task store = one in-process `HashMap<TaskId, McpTaskRecord>`.** Over
  stdio the transport *is* the conversation: the pipe dying means the
  client is gone, and the spec's "survives disconnects" durability serves
  HTTP deployments caro-mcp does not have (HTTP transport is out of scope,
  as in ADR-015). This is the RC's own "stateless protocol, stateful
  applications" pattern with state scoped to process lifetime. Documented
  loudly in `server/discover` docs and README: **tasks do not survive the
  subprocess**. (Failure mode 1, solved by scoping rather than persisting.)
- Task IDs from the existing `uuid` dependency (v4, 122 bits — satisfies
  the unguessability requirement); no `tasks/list` (spec removed it);
  unknown/expired taskId → `-32602`. (Failure mode 2.)
- Polling contract: `pollIntervalMs: 100` (approval latency is
  human-scale), `ttlMs: 600_000` (10 min) after which the record is
  dropped and the task is gone — TTL expiry is deny-by-default: an
  unapproved HumanGate command never becomes approvable by waiting.
- `tasks/cancel` → record marked `cancelled` (terminal), ack empty result;
  since caro's "worker" is instantaneous validation, cancellation is
  always honorable — one of the few servers where cooperative cancel has
  no TOCTOU window. (Failure mode 4.)
- `tasks/update` approval fulfillment runs the **same revalidation check
  as D3** before emitting the terminal `completed` result. The in-memory
  record is a UX convenience, never the source of truth for the verdict.
- Every status transition emits an ADR-037 lifecycle event on stderr
  (assessment → gate_opened → approval_granted/denied/expired → result),
  so SIEM consumers see the same chain regardless of transport.

Clients without the tasks capability never see a task (progressive
enhancement per spec); caro-mcp never returns `-32003` for the approval
flow because MRTR is always available as the fallback.

### D5 — `caro --dry-run --output json` exit codes are unchanged

The subprocess path keeps ADR-015's exit-code table verbatim (0 safe /
1 blocked / 2 human-gate / 3 async-log / 10 input error / 11 internal
error). ADR-036's guard-mode codes are likewise untouched. Machines
already scripted against those contracts see no change from this ADR.

---

## New Types

All in `src/mcp/` per ADR-015's layout; all Serde-serializable from day
one; no new validation logic anywhere — projection and bookkeeping only.

### `ApprovalGateState` — the MRTR `requestState` payload (`src/mcp/mod.rs`)

```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalGateState {
    pub schema_version: u8,        // 1
    /// SHA-256 of command bytes — retry leg must re-derive identically.
    pub command_hash: String,
    /// Risk level at gate time — retry leg revalidation must not be lower.
    pub risk_level: RiskLevel,
    pub safety_level: SafetyLevel,
    pub shell: ShellType,
    /// RFC 3339; retry leg rejects if older than gate_ttl (10 min default).
    pub gated_at: String,
    /// The single inputRequests key this state corresponds to.
    pub elicitation_key: String,   // "approval"
}
```

Serialized as base64(JSON) into `requestState`. **Integrity model:** no
signature needed in v1 — the retry leg recomputes the verdict from the
command itself, so a tampered `ApprovalGateState` can only cause a
*mismatch error*, never a weaker verdict. (A signed/HMAC variant is listed
out of scope; it becomes necessary only if a future version lets
`requestState` carry decisions that are not re-derivable.)

### `McpTaskRecord` + `McpTaskStatus` (`src/mcp/tasks.rs`, new file, `#[cfg(feature = "mcp-server")]`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTaskRecord {
    pub task_id: String,               // uuid v4
    pub status: McpTaskStatus,
    pub created_at: String,            // RFC 3339
    pub last_updated_at: String,
    pub ttl_ms: u64,                   // 600_000
    pub poll_interval_ms: u64,         // 100
    pub gate: ApprovalGateState,       // same struct as MRTR path
    /// Present when terminal-completed.
    pub result: Option<SafetyAssessmentOutput>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpTaskStatus {
    Working,
    InputRequired,
    Completed,
    Failed,
    Cancelled,
}
```

Method contracts: `McpTaskStore::insert(record) -> TaskId` (record findable
before `CreateTaskResult` is sent, per spec), `get(&id) -> Option<&record>`
(prunes expired entries on access), `fulfill(&id, approved: bool) ->
Result<SafetyAssessmentOutput, GateMismatch>` (runs revalidation),
`cancel(&id)`. Plain struct + `Mutex<HashMap>`; no trait, no async store
abstraction — YAGNI until an HTTP transport exists.

### `SafetyAssessmentOutput.approval` — additive field (ADR-015 type)

```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRecord {
    pub granted: bool,
    pub actor: String,     // "mcp-client" in v1; ADR-021 attribution later
    pub at: String,        // RFC 3339
}
// on SafetyAssessmentOutput:
//   #[serde(skip_serializing_if = "Option::is_none")]
//   pub approval: Option<ApprovalRecord>,
```

`Option` + skip-if-none keeps every existing schema_version-1 consumer
byte-compatible; schema_version stays 1 (additive change).

---

## Files That Change

| File | Change |
|------|--------|
| `Cargo.toml` | `rmcp = { version = "=X.Y.Z", optional = true }` (exact pin at spike time; RC-supporting release), `mcp-server = ["dep:rmcp"]` feature, `[[bin]] caro-mcp` — all as ADR-015 specified |
| `Cargo.lock` | Regenerated, committed |
| `src/mcp/mod.rs` | **NEW** (ADR-015's file) — handler, `SafetyAssessmentOutput` + `ApprovalRecord`, `ApprovalGateState`, MRTR two-leg flow, `server/discover` |
| `src/mcp/tasks.rs` | **NEW** — `McpTaskRecord`, `McpTaskStatus`, `McpTaskStore`, the three `tasks/*` handlers |
| `src/bin/caro-mcp.rs` | **NEW** (ADR-015's file) — entry point, stdio transport |
| `src/lib.rs` | `#[cfg(feature = "mcp-server")] pub mod mcp;` |
| `tests/mcp_integration.rs` | **NEW** — ADR-015's 7 tests + the 8 below |

Seven files total; the only file this ADR adds beyond ADR-015's list is
`src/mcp/tasks.rs`. `src/safety/`, `src/models/`, `src/cli/` untouched
except ADR-015's already-scoped `--output json` flag.

---

## Output Contract

Machines and scripts will depend on:

1. **`SafetyAssessmentOutput` schema_version 1** (ADR-015) with optional
   `approval` — published to `docs/schema/safety-assessment-v1.json`.
2. **MRTR leg-1**: `InputRequiredResult` with exactly one `inputRequests`
   entry keyed `"approval"`, boolean elicitation schema, opaque
   `requestState`. Leg-2 mismatch (hash, risk downgrade, TTL) → `-32602`
   with a machine-readable `data.reason` ∈
   {`hash_mismatch`, `risk_mismatch`, `gate_expired`, `state_malformed`}.
3. **Tasks**: statuses exactly per extension spec; TTL expiry ⇒ task
   disappears (subsequent `tasks/get` → `-32602`); `completed.result`
   always a `SafetyAssessmentOutput`.
4. **Process exit codes for `caro-mcp` itself**: 0 clean shutdown (stdin
   EOF), 64 usage error, 11 internal fatal. All *assessment* outcomes are
   protocol payloads, never process exit codes — the binary exiting
   non-zero always means the server broke, not that a command was risky.
5. ADR-037 lifecycle events on stderr, one JSON object per line.

---

## Integration Tests (`tests/mcp_integration.rs`)

Deterministic — no LLM, no network; drives the binary over stdio. ADR-015's
7 tests carry forward; added:

| Test | Input | Expected |
|------|-------|----------|
| `test_discover_advertises_tasks` | `server/discover` | tools + `extensions["io.modelcontextprotocol/tasks"]` present |
| `test_gate_mrtr_flow` | `sudo rm -rf /var`, `approval_mode: gate`, no tasks cap | `InputRequiredResult`, then approve retry → `completed`, `approval.granted: true`, exit-free |
| `test_gate_mrtr_deny` | same, deny on retry | `allowed: false`, `suggested_routing: "block"`, `approval.granted: false` |
| `test_gate_state_tamper` | retry with `command_hash` bytes flipped | `-32602`, `data.reason: "hash_mismatch"` |
| `test_gate_task_flow` | same command, tasks cap advertised | `CreateTaskResult` → `tasks/get` = `input_required` → `tasks/update` approve → `tasks/get` = `completed` with assessment |
| `test_task_cancel` | create then `tasks/cancel` | ack, then `tasks/get` = `cancelled` |
| `test_task_unknown_id` | `tasks/get` random uuid | `-32602` |
| `test_report_mode_never_gates` | dangerous cmd, default `approval_mode` | plain `SafetyAssessmentOutput`, no gate, no task |

Plus one **conformance step** (not a Rust test): run the official
`modelcontextprotocol/conformance` suite's stateless-core scenarios against
`caro-mcp` in CI once the suite ships spec-final scenarios — SEP-2484 makes
this the ecosystem's definition of "compliant."

---

## Competitive Differentiation

**What their design gets right (replicated here):** server-directed task
creation with client capability gating (progressive enhancement — no
bifurcated tool catalog); reads/writes split (`tasks/get` idempotent and
cacheable, `tasks/update` mutating); explicit-handle state visible to the
model rather than hidden in transport metadata; unguessable IDs replacing
enumerable lists.

**Their gaps caro designs around:** the Task Store durability obligation
(solved by MRTR-default + process-scoped store, D3/D4); cooperative-cancel
TOCTOU (caro's instantaneous worker makes cancel always honorable);
experimental-surface migration risk (caro ships against the *extension*,
never shipped the 2025-11-25 experimental API, and the RC's deprecation
policy now guarantees 12-month windows going forward).

**What caro can do that nobody in this ecosystem can:** re-derive the
verdict on every leg. Every other approval gate must *trust its own stored
state* about what was pending; caro's determinism means stored/echoed state
is only ever a correlation hint, and the verdict is recomputed from the
command in <2ms, offline, on every transition. Tampering downgrades to a
visible error by construction. Combined with ADR-037 events and ADR-032
receipts, this is the pre-assessment → tiered decision → approval →
receipt chain Hermes notes "nobody has shipped" — with the approval step
now protocol-native.

---

## Explicit Out-of-Scope (next version)

| Not in scope | Rationale |
|---|---|
| HTTP/streamable transport (and with it *real* durable task stores) | ADR-015 position unchanged; stdio covers all known callers; HTTP reopens auth + persistence |
| `notifications/tasks` + `subscriptions/listen` push updates | Polling at 100ms is fine for human-latency approvals; push needs long-lived streams stdio callers don't want |
| Signed/HMAC `requestState` | Unnecessary while all state is re-derivable (see integrity model); revisit if non-derivable fields are added |
| MCP Apps approval UI (`ext-apps`) | Host elicitation UIs are sufficient; an Apps-based approval card is a v2 polish item |
| Task persistence across caro-mcp restarts | Deliberately rejected, not deferred — documented process-scoped semantics (D4) |
| `approval_mode: gate` on `validate_batch` | Per-command gating on batches needs partial-input semantics; report-mode batches unaffected |
| ADR-021 actor attribution richer than `"mcp-client"` | Needs host identity plumbing; lands with ADR-021 |
| Tasks for embedded-LLM generation calls (minutes-long `working` state) | Real use case for Tasks' polling, but touches backends; separate ADR after this lands |

---

## Consequences

**Positive:** the Jul 28 first-mover claim Hermes has asked for three weeks
running, on the spec-final protocol rather than the about-to-be-legacy one;
the statefulness audit closes with a documented answer (stateless core =
caro-native; Tasks = optional, process-scoped); approval becomes
protocol-native without violating the pure-subprocess constraint; zero
changes to existing safety logic or existing output contracts.

**Negative / risks:** rmcp is not in the Tier-1 beta wave — its
2026-07-28 + MRTR + extensions support is the single biggest schedule risk;
the build spike (below) exists to surface it in under an hour. If rmcp
lacks MRTR/Tasks types at spike time, fallback is implementing the two
`resultType` payloads as local serde types over rmcp's raw-JSON escape
hatch (they are plain JSON shapes), which the spike must also smoke-test.
The RC could still change before Jul 28 (it is locked, but "RC" means what
it means) — pin exact rmcp versions and re-run conformance at spec-final.

## Alternatives Considered

- **Tasks-only approval gate (no MRTR).** Rejected: hard-couples the
  headline feature to the optional extension *and* to a state store;
  clients without the extension would get `-32003` instead of a working
  gate. MRTR is core, stateless, and universal.
- **MRTR-only (no Tasks at all).** Seriously considered — it is the purest
  fit. Rejected narrowly: Hermes' distribution thesis is that Tasks makes
  "pending human approval" *protocol-native state* that agent frameworks
  (LangGraph 1.0 treats MCP tools as first-class nodes) will build UX
  around; the compatibility layer is ~200 LOC on top of shared gate logic
  and is honestly scoped as process-lifetime state. If review disagrees,
  D4 detaches cleanly — nothing in D3 depends on it.
- **Daemonized caro-mcp with a real task store** (sled/SQLite). Rejected:
  violates the pure-subprocess constraint that every ADR since 015 has
  held; durability for HTTP deployments is a different product decision
  that arrives (if ever) with the HTTP transport.
- **Wait for spec-final + rmcp stable before scoping.** Rejected: the
  ten-week validation window exists precisely so implementers build
  against the RC; waiting forfeits the date-anchored positioning and
  Hermes' third consecutive #1 recommendation.

---

## Build Spike (Phase 0 — per `.claude/rules/external-sdk-integration.md`, must merge before any of the above)

The 5-step checklist, instantiated:

1. **License**: rmcp is MIT (one-way absorption into AGPL-3.0 OK);
   `cargo deny check licenses` for transitives.
2. **MSRV**: caro is 1.85; ADR-015 recorded rmcp 1.x requiring ≥1.85 —
   re-verify on the RC-supporting release, which may have moved.
3. **Optional dep + feature**: `mcp-server = ["dep:rmcp"]`, never default.
4. **Code ref forcing compile**: `pub fn mcp_smoke()` constructing an rmcp
   server type **and** naming `ProtocolVersion::V_2026_07_28` — the spike's
   job this time is specifically to prove RC-surface support, plus a
   compile-time probe for MRTR/Tasks types (presence/absence recorded in
   the commit body either way).
5. **Two builds + smoke**: `cargo check --no-default-features --features
   embedded-cpu` and `…,mcp-server`; `cargo test --features mcp-server
   --lib mcp::` runs the smoke. Results in the commit body.

Spike PR title: `feat(mcp): phase 0 - rmcp RC build spike`. ≤100 LOC.

---

*Generated by `caro-research--scoping-process` scheduled agent · 2026-07-21*
