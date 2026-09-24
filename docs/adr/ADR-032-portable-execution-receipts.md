# ADR-032: Portable Execution Receipts — Hash-Chained, Verdict-Bound Assessment Payloads

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-07
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: Fetch.ai **AEVS** (Agent Execution Verification
  System, SDK v0.2.2, beta — PH launch 2026-07) and the OpenTelemetry
  **GenAI agent semantic conventions** (`gen_ai.agent.id`, `execute_tool`
  spans; spec status: Development)
- **Depends on**: ADR-024 (headless JSON contract), ADR-021 (execution
  attribution), ADR-026 (multi-turn agentic session)
- **Relates to**: ADR-003 (enterprise audit trail — superset, deferred),
  ADR-031 (session circuit breaker — `session_halted` event shares the
  receipt's identity fields), Hermes memo 2026-07-07 (opportunity 1:
  "publish the assessment-payload + lifecycle-event schema as a spec")
  and 2026-07-03 item D (`agent_id` on lifecycle events)

---

## Context

### The market problem

The market is moving from "logs exist" to "proof is portable and
structured." In the week of 2026-06-30 alone: Fetch.ai launched AEVS
(signed receipts per tool call), Palo Alto launched agent activity
monitoring, and Onpilot sells approvals + audit to SMBs. Buyers are being
trained to expect **structured, independently verifiable evidence** of
what an agent actually did — not a chat transcript claiming it.

Caro produces exactly the two artifacts that matter — a safety verdict
(`ValidationResult`) and a real execution outcome (`ExecutionResult`) —
but emits them only transiently on stdout (ADR-024 envelope). There is no
durable, tamper-evident, schema-versioned record an auditor, SIEM, or
AEVS-style explorer can consume after the process exits. Hermes 07-07:
"Caro's verdicts become joinable evidence in any audit pipeline — the
portable proof buyers are being trained to expect."

### Phase 1 — how AEVS works (and where it breaks)

AEVS is a Python SDK that monkey-patches framework tool dispatch
(`BaseTool.invoke` for LangChain, `ClientSession.call_tool` for MCP).
Every tool call produces a receipt: `tool`, `inputs`, `output`, `status`,
`duration`, `seq`, `prev_hash`, `framework`, `invocation_id`, session
UUID, client-generated `reference_id`, backend-assigned `receipt_id`.
Receipts are HMAC-signed client-side, hash-chained per session,
ECDSA-P-256 (KMS) anchored server-side, buffered in local SQLite, and
drained by a background thread to `api.aevs.fetch.ai`. Verification is a
public API/explorer returning `chain_status ∈ {anchor, linked, mismatch,
gap, broken, unverified}`. Three visibility levels: `public`, `private`
(payloads `[REDACTED]` publicly), `proof_only` (only SHA-256
`input_hash`/`output_hash` ever leave the host).

Why it is beta / its failure modes:

1. **Fail-open evidence loss.** Invalid credentials → silent "no-op
   mode": the agent keeps running and no receipts are recorded (a logged
   warning is the only trace). Buffer overflow (>10k pending) evicts the
   oldest receipt with a "gap marker". An evidence system that silently
   stops collecting evidence is the exact failure auditors care about.
2. **Attestation gap, self-admitted.** From their own docs: verification
   "proves the SDK recorded the call and its reported result — **not that
   an external system actually performed the action**." AEVS observes the
   framework layer; it cannot bind the receipt to the OS-level outcome.
3. **Cloud-tethered.** Signing keys are KMS-hosted; verification requires
   `api.aevs.fetch.ai`. No offline story, no air-gapped audit.
4. **No judgment layer.** Receipts record *what happened*, never *whether
   it should have* — no risk tier, no policy verdict, no matched-pattern
   evidence. The receipt is half the audit story.
5. **Fragile capture point.** Monkey-patching `BaseTool.invoke` breaks
   when frameworks refactor dispatch; hence the LangChain-0.2+/MCP-1.20+
   version pinning and per-framework adapters.
6. **Daemon-shaped.** Background drain thread + SQLite buffer + flush
   discipline (`aevs.flush()` before exit or receipts are lost) — the
   antithesis of a pure subprocess contract.

Session lifecycle: `configure()` → `enable()` patches dispatch once per
process; a session UUID scopes the hash chain; LangGraph runs share an
`invocation_id`. Redundant initialization is avoided by process-global
patching — which is exactly what caro cannot and need not do.

### Phase 2 — what caro already has

- `ValidationResult { allowed, risk_level, explanation, warnings,
  matched_patterns, confidence_score }` — serde, implemented
  (`src/safety/mod.rs:175`). The judgment payload AEVS lacks.
- `ExecutionResult { exit_code, stdout, stderr, execution_time_ms,
  success }` — implemented (`src/execution/executor.rs:9`). The OS-level
  ground truth AEVS cannot see: **caro is the executor**, so binding
  verdict to outcome is free.
- `AiSession` / `SessionStore` (`src/ai/session.rs`, `src/ai/store.rs`) —
  durable local session state with stable `id: u64`.
- ADR-024 `HeadlessEnvelope` + ADR-004 NDJSON events — the transient wire
  contract the receipt persists; `schema_version` discipline established.
- `schemars 0.8` + `src/bin/generate-schema.rs` — JSON Schema generation
  already in the build; `RiskLevel`, `DangerPattern`, `RiskJudgment`
  already derive `JsonSchema`.
- ADR-021 scoped `exit_code` / `duration_ms` / `executed_at` on `Turn`
  (not yet implemented) — the receipt supersedes the per-turn fields as
  the canonical outcome record; `Turn` additions become derived data.

Gap inventory: no durable receipt record, no hash chain, no `agent_id`
anywhere in `src/` (grep confirms: only ADR-031 mentions it), no
published schema file others can validate against, no verify command.

### Our unique positioning

1. **Verdict-bound receipts.** Only caro can put `risk_level`,
   `matched_patterns`, and the routing decision *inside* the receipt,
   because only caro is both judge and executor. AEVS/Palo Alto record
   activity; caro records activity **plus the policy evaluation that
   allowed it**.
2. **Executor-level attestation.** Caro spawns the shell process, so the
   receipt binds to the real `exit_code` and output hashes — closing
   AEVS's self-admitted gap by construction.
3. **Offline and air-gapped.** Chain verification is a local recompute;
   no backend, no account, no network. `caro receipt verify` works on a
   USB stick.
4. **Pure subprocess.** One invocation appends one receipt (read last
   line → link → append). No patch, no thread, no flush discipline, no
   buffer to overflow.
5. **Agent-agnostic capture point.** Any agent that shells out through
   caro is covered — no per-framework adapter matrix to maintain.

## Decision

Add a **portable execution receipt**: an append-only, hash-chained,
schema-versioned NDJSON record emitted per headless invocation, with a
published JSON Schema and a local `caro receipt verify` subcommand.

### Design principles (solve AEVS failure modes by construction)

- **Fail-closed evidence** (vs failure mode 1): when `--receipt` is
  requested and the chain file cannot be appended, the run exits with an
  error *before* execution — evidence is a precondition, not
  best-effort. No silent no-op mode, no eviction, no gap markers.
- **Bind, don't observe** (vs failure mode 2): the receipt is written by
  the process that ran the command, containing the verdict that gated it
  and the exit code that resulted. There is no framework layer to lie
  through.
- **Local verification** (vs failure mode 3): the chain is a file;
  verification is `SHA-256` recomputation. Signing is explicitly v2
  (see out-of-scope) — an unsigned local chain already beats a signed
  chain behind someone else's API for the offline audit use case.
- **Stateless per invocation** (vs failure mode 6): linking requires only
  reading the final line of the chain file. No daemon, no lock beyond an
  advisory file lock during append (same pattern as `SessionStore`).

### New types (existing modules only; serde + `JsonSchema` from day one)

```rust
// src/models/mod.rs

/// Schema-versioned, hash-chained record of one gated execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ExecutionReceipt {
    /// Receipt schema version, independent of caro version. Starts "1".
    pub schema_version: String,
    /// Client-generated UUID v4; stable identity for external joins.
    pub receipt_id: String,
    /// Monotonic position in this chain file (0 = anchor).
    pub seq: u64,
    /// Hex SHA-256 of the previous receipt line; None only when seq == 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_hash: Option<String>,
    /// ISO-8601 UTC.
    pub ts: DateTime<Utc>,
    /// AiSession id when running under --session; None for one-shot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<u64>,
    /// Caller-declared agent identity (--agent-id / CARO_AGENT_ID).
    /// Aligned with OTel `gen_ai.agent.id`. Free-form, max 256 chars.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub caro_version: String,
    pub backend: String,
    pub shell: String,
    /// SHA-256 of the natural-language prompt (prompt text never stored).
    pub prompt_hash: String,
    /// The generated command. Redacted to its SHA-256 under
    /// --receipt-visibility proof_only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    pub command_hash: String,
    /// The judgment layer — embedded verbatim, the field AEVS lacks.
    pub safety: ValidationResult,
    /// Terminal status mirroring HeadlessStatus (ok | blocked |
    /// needs_confirmation | error) plus "halted" (ADR-031).
    pub status: String,
    /// Outcome. None when not executed (blocked / dry-run / gen-only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ReceiptExecution>,
    /// Process exit code caro returned for this invocation (ADR-024 map).
    pub caro_exit_code: i32,
}

/// OS-level outcome, hash-bound. Raw stdout/stderr never stored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ReceiptExecution {
    pub exit_code: i32,
    pub duration_ms: u64,
    pub stdout_hash: String,   // hex SHA-256, "" for empty
    pub stderr_hash: String,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
}

/// Result of local chain verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ChainVerification {
    pub schema_version: String,
    pub receipts: u64,
    /// anchor_ok && every link recomputes && seq is gapless.
    pub chain_intact: bool,
    /// First failure, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_break: Option<ChainBreak>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ChainBreak {
    pub seq: u64,
    pub kind: ChainBreakKind, // hash_mismatch | seq_gap | parse_error
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChainBreakKind {
    HashMismatch,
    SeqGap,
    ParseError,
}
```

Chain I/O (`append`, `last_line_hash`, `verify`) lives in one new file
inside the existing module: `src/ai/receipts.rs` (mirrors
`src/ai/store.rs` — advisory-locked append to
`$XDG_DATA_HOME/caro/receipts.ndjson`, one JSON object per line; the
line's canonical bytes are what the next receipt's `prev_hash` covers).

### Wire contract

- `--receipt` (headless only): append one `ExecutionReceipt` per
  invocation; envelope gains `receipt { receipt_id, seq }`; NDJSON gains
  a `receipt_sealed { receipt_id, seq, prev_hash }` event after `Result`.
- `--agent-id <ID>` / `CARO_AGENT_ID`: stamps `agent_id` on the receipt,
  the ADR-024 envelope, the NDJSON `Init` event, and ADR-031's
  `session_halted` event (Hermes item D closed in one place).
- `--receipt-visibility {full|proof_only}`: `proof_only` nulls `command`
  (hash retained) — AEVS's strongest privacy mode, minus the modes that
  depend on a backend.
- `caro receipt verify [--file <path>] [--json]`: recomputes the chain;
  emits `ChainVerification` JSON on stdout.
- `caro receipt schema`: prints the JSON Schema (via `schemars`), same
  document committed at `docs/spec/execution-receipt.schema.json`.
- **Exit codes**: `verify` exits **0** (intact) or **9 =
  ReceiptChainBroken** (9 is the next free slot: 0–6 ADR-024, 7 ADR-027,
  8 ADR-031, 30–32 ADR-030). A `--receipt` run that cannot append exits
  **1** with `error.kind = "receipt_io"` before any execution.
- Stability promise: within `schema_version: "1"`, fields are only ever
  added, never renamed or removed (same discipline as ADR-024).

### Minimal file set (one new source file, no new modules, no new crates)

| File | Change |
|---|---|
| `src/models/mod.rs` | `ExecutionReceipt`, `ReceiptExecution`, `ChainVerification`, `ChainBreak{,Kind}` |
| `src/ai/receipts.rs` | **new** — chain append / read-last / verify (std + serde + sha2 already in tree) |
| `src/ai/mod.rs` | `pub mod receipts;` |
| `src/main.rs` | `--receipt`, `--agent-id`, `--receipt-visibility` flags; `receipt verify|schema` subcommand; exit 9 |
| `src/cli/mod.rs` (headless path) | seal receipt after Result; `receipt` field on envelope; `receipt_sealed` event |
| `src/bin/generate-schema.rs` | also emit `execution-receipt.schema.json` |
| `docs/spec/execution-receipt.schema.json` | committed generated schema (the publishable spec, Hermes opp. 1) |
| `tests/receipt_chain.rs` | **new** — integration tests below |

### Integration tests (deterministic input → fixed JSON + exit code)

1. Gen-only run with `--receipt --dry-run` on the static backend →
   receipt with `seq 0`, `prev_hash` absent, `execution` absent,
   `status "ok"`; process exit 0.
2. Second run appends `seq 1` whose `prev_hash` equals SHA-256 of line 0
   (recomputed by the test, not by the code under test).
3. Blocked command (`rm -rf /`) with `--receipt` → receipt has
   `status "blocked"`, `safety.allowed = false`, non-empty
   `matched_patterns`, `execution` absent, `caro_exit_code 3`.
4. `caro receipt verify --json` on the 3-line chain → `chain_intact
   true`, exit 0.
5. Flip one byte in line 1 → verify reports `first_break { seq: 2, kind:
   hash_mismatch }`, exit 9.
6. Delete line 1 → `seq_gap` at seq 2, exit 9.
7. Unwritable chain path + `--receipt` → exit 1, `error.kind
   "receipt_io"`, and the executor was never invoked (fail-closed
   assertion, mirrors ADR-031's anti-race test).
8. `--agent-id ci-bot` → `agent_id "ci-bot"` present in receipt, envelope,
   and `Init` event; omitted → field absent everywhere (serde skip).
9. `caro receipt schema` output validates the receipts produced in tests
   1–3 (schema/instance round-trip via `jsonschema` dev-dependency —
   dev-only, no runtime crate added).

### Out of scope (v2+)

- **Cryptographic signing** (Ed25519/minisign of chain segments). Needs
  a key-management story and a new crate → external-sdk build-spike rule
  applies. The chain format reserves nothing; a `sig` field is additive.
- **Receipt export/ingest adapters** (OTel span emission, AEVS-explorer
  upload, SIEM shippers). The committed JSON Schema is the interface.
- **Per-turn receipts inside one agentic session process** (ADR-026
  multi-turn loop sealing N receipts) — v1 seals one receipt per
  invocation; the loop follow-up belongs with ADR-026 implementation.
- **ADR-003 enterprise audit fields** (machine/user/department blocks).
  The receipt's `agent_id` + `session_id` are the join keys; enrichment
  happens downstream.
- **Retention/rotation policy** for `receipts.ndjson`.
- Migrating `Turn.risk: Option<String>` to typed `RiskLevel` (footgun
  logged by ADR-031; receipts embed the typed `ValidationResult` so the
  receipt path is unaffected).

## Consequences

Positive: caro gains the "portable proof" artifact the market is
converging on, with two properties no observer-layer competitor can
match (verdict-binding, executor-level attestation) and one property the
cloud players won't ship (offline verification). The published schema is
the integration surface for AEVS-style explorers, SIEMs, and the OTel
GenAI conventions (`agent_id` aligns with `gen_ai.agent.id`). Hermes
07-07 opportunity 1 and 07-03 item D are both discharged by one scoped
change.

Negative / accepted: an unsigned chain proves integrity of the file, not
authorship — a hostile local user can rewrite the whole chain
consistently. That is the same trust model as shell history and is
explicitly priced in for v1 (signing is v2). `receipts.ndjson` grows
unboundedly until v2 rotation. One more flag on an already-wide headless
surface.

## Alternatives considered

1. **Adopt AEVS's SDK/backend directly** — rejected: Python-only,
   cloud-tethered, fail-open, and caro would be feeding evidence into a
   beta third-party service by default.
2. **Sign from day one** — rejected for v1: pulls a crypto crate (spike
   rule), and a KMS-less local key that lives next to the chain file adds
   ceremony without changing the local-attacker trust model.
3. **SQLite receipt store** — rejected: NDJSON keeps the chain
   greppable, diffable, and verifiable by 20 lines of any language;
   hash-chaining lines is the tamper evidence SQLite wouldn't add.
4. **Extend ADR-003's enterprise audit event instead** — rejected:
   ADR-003 is enterprise-scoped and unimplemented; the receipt is the
   FOSS-core primitive ADR-003 can later enrich.
5. **Emit OTel spans instead of receipts** — rejected as primary format:
   OTel GenAI conventions are still Development-status and span export
   assumes a collector (daemon-shaped). Attribute *names* are borrowed
   (`agent_id`), export is a v2 adapter.

## References

- AEVS: https://aevs.fetch.ai/ · https://aevs.fetch.ai/llms.txt ·
  https://github.com/fetchai/AEVS-sdk (SDK 0.2.2, beta)
- OTel GenAI agent spans:
  https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/ ·
  https://opentelemetry.io/blog/2026/genai-observability/
- Hermes memos: `.hermes/digests/2026-07-07-agent-market-scan.md`
  (opportunity 1), `2026-07-03` (item D)
- ADR-021, ADR-024, ADR-026, ADR-031, ADR-003
