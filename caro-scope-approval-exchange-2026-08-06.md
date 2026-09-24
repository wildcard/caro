# Implementation Scope — Approval Exchange Payload (AEP v1)

**Feature under analysis:** Zed **Agent Client Protocol** `session/request_permission`
(agent-client-protocol-schema v1.6.0, Apache-2.0; protocol v1 stable, v2 draft behind
`unstable_protocol_v2` with lossy cross-version conversions).
**Equivalent we are scoping for Caro:** a transport-independent, content-bound
`ApprovalRequest`/`ApprovalResponse` JSON pair ("approve anywhere") emitted and
verified by pure subprocess calls — the wire format every remote supervision
surface (phone, menu bar, canvas, editor) can render without caro shipping UI.

**Date:** 2026-08-06 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-046-approval-exchange-payload.md`

> The task template left `[FEATURE NAME]` unfilled and ran unattended. Target
> selection: the 2026-08-06 Hermes scan's top build item is the versioned
> assessment/approval schema (opportunities a + b). Coverage check: ADR-027 is
> stdin-stream-coupled, ADR-037/041 are one-way events, ADR-045 is the local
> ceremony. The request/response payload itself has no covering ADR. The ACP
> analog choice is a reviewable assumption.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom
Editors integrating N coding agents needed N bespoke approval dialogs. ACP
standardizes one: the agent sends `session/request_permission` with a tool-call
description and options; the editor renders it and returns the user's choice.
Users get human-in-the-loop on sensitive operations; agent and editor vendors
get interop (Zed, JetBrains partnership, Gemini CLI, Goose, Copilot CLI).

### Core architecture observed
JSON-RPC 2.0 over stdio; client (editor) spawns the agent subprocess. Lifecycle:
`initialize` (version + capability negotiation) → `session/new` → `session/prompt`
→ streamed `session/update` notifications (tool calls with kinds
read/edit/delete/move/search/execute/think/fetch) → agent→client
`session/request_permission { sessionId, toolCall: ToolCallUpdate, options:
[{id, label, kind ∈ allow_once|allow_always|reject_once|reject_always}], _meta }`
→ response `outcome: Cancelled | Selected { optionId }`. Redundant initialization
is avoided by long-lived sessions (`session/load` replays history); the schema
crate ships serde + schemars types, `#[non_exhaustive]`, versioned modules
(`v1`, draft `v2`) with `TryFrom` conversions.

### Why experimental / failure modes found
1. **No independent assessment** — the request carries the supervised agent's
   self-description; no risk verdict, no matched patterns, no rationale.
2. **Agent authors its own ballot** — the options list (incl. `allow_always`)
   is composed by the party requesting permission.
3. **Approval ≠ content (TOCTOU)** — outcome carries only `optionId`;
   `ToolCallUpdate` is mutable (`rawInput` can change post-display); nothing
   binds the decision to the bytes that execute. Same family as
   `--permission-prompt-tool`'s `updatedInput` (ADR-045 Phase 1).
4. **No actor attribution** — `Selected { optionId }` has no who/where/how;
   multiplayer sessions (mpai) make this an oversight-documentation failure.
5. **Session-coupled lifecycle** — the exchange exists only inside a live
   JSON-RPC connection; on `session/cancel` the client MUST answer pending
   requests `Cancelled`. Nothing is queueable, portable, or offline-verifiable;
   `non_exhaustive` + v1↔v2 lossy conversion shifts churn onto consumers.

### Structured output contract
Typed request/response with JSON Schema generation (schemars); JSON-RPC error
codes (-32602, -32002 permission denied, etc.); no exit codes (not a CLI
contract); no proof artifact; `_meta` reserved for extensions.

## Phase 2 — Competitive Differentiation

### What they get right (replicate)
Typed options with machine-readable `kind` (not free text); one schema shared by
all surfaces; explicit versioned schema modules with JSON Schema artifacts;
cancellation as a first-class outcome, not an error; `_meta` extensibility.

### Their gaps we avoid by designing the schema first
Assessment embedded in the request; options derived from policy, never from the
assessed party; SHA-256 content binding echoed in the response; mandatory actor
block; explicit `expires_at` + `Expired` outcome so payloads are self-contained;
literal `schema_version: "aep/1"` with an additive-only rule enforced by tests.

### Our unique positioning
Caro is the reviewer, not the agent — the assessment in the payload comes from a
deterministic, inspectable 52-pattern validator, offline, agent-agnostic, and
standalone ("trust as mechanism, not adjective" — 08-06 scan, opportunity d).
ACP can carry *a* permission prompt; only caro can carry a *verdict* any surface
can trust without trusting the agent.

### Existing infrastructure reused (no duplication)
`SafetyDecision` + `SuggestedRouting` (`src/safety/mod.rs:189`, ADR-020) — the
assessment field verbatim; ADR-040 policy file — option derivation + expiry
default; `ApprovalProof` (ADR-045) — CRITICAL-tier response proof; `sha2`
(existing dep) — content binding; serde + schemars (already derive targets in
`src/safety`); ADR-024/027 envelope + exit-code registry — extended, not
replaced; `request_id` join key consumed by ADR-032 receipts and ADR-037/041
events; `--agent-id` (ADR-021) attribution of the requesting agent.

## Phase 3 — Scope Definition

### ADR
`docs/adr/ADR-046-approval-exchange-payload.md` (written alongside this scope):
context, decision, consequences, four alternatives considered (wholesale ACP
adoption, ADR-027-stream-only, `caro serve` daemon, request signing).

### New types (all serializable from day one)
In **`src/safety/exchange.rs`** (new file in existing module; re-exported from
`crate::safety`): `ApprovalRequest`, `ApprovalOption`, `ApprovalOptionKind`
(`ApproveOnce | DenyOnce | Block` — deliberately no `AllowAlways`),
`ApprovalOutcome` (`Selected | Cancelled | Expired`), `ApprovalActor`,
`ActorAuthKind`, `ApprovalResponse`, `AepError` (thiserror). Field lists,
serde attributes, and the two method contracts
(`ApprovalRequest::from_decision(..) -> Option<Self>`,
`ApprovalResponse::verify(&self, &ApprovalRequest, now) -> Result<_, AepError>`,
fail-closed) are specified in ADR-046. Determinism hooks for tests:
`request_id`/timestamps injectable (`CARO_AEP_TEST_NOW`, `CARO_AEP_TEST_ID` or
builder params) — never randomness inside `verify`.

### Minimal file set
1. `src/safety/exchange.rs` — types + verification (new file, existing module)
2. `src/safety/mod.rs` — `pub mod exchange;` + re-exports
3. `src/cli/mod.rs` — `assess` / `approve verify` arg parsing
4. `src/main.rs` — wiring + exit codes
5. `tests/approval_exchange.rs` — integration tests below
6. `docs/adr/ADR-046-approval-exchange-payload.md`

No new crates. No new top-level modules. No daemon: both subcommands are pure
subprocess calls that read args/stdin, write one JSON document, and exit.

### Exit code / output contract (machines depend on this)
`caro assess -o json`: exactly one JSON envelope (ADR-024) on stdout, optionally
carrying `approval_request`. `caro approve verify`: one JSON verdict line.

| Code | `assess` | `approve verify` |
|---|---|---|
| 0 | auto-approve, no exchange | response valid, outcome approves |
| 3 | hard Block | response valid, outcome denies/cancels |
| 7 | approval required, request emitted | — |
| **8** (new) | — | response invalid: hash/id mismatch, expired, missing proof, malformed |
| 1/2/4/5/6, 201 | unchanged (ADR-024/027 registry) | unchanged |

### Integration tests (known input → deterministic JSON + exit code)
Fixed clock + request id via test env; `-b mock` backend where generation is
involved.

| # | Invocation | Expect |
|---|---|---|
| 1 | `caro assess -o json --command-only "ls -la"` | no `approval_request`, exit 0 |
| 2 | `caro assess -o json --command-only "rm -rf /"` | `assessment.risk_level=="Critical"`, routing Block, exit 3 |
| 3 | `caro assess -o json --command-only "sudo rm -rf /var/log"` (High) | `approval_request` present: `schema_version=="aep/1"`, fixed `request_id`, correct `command_sha256`, 3 policy options, exit 7 |
| 4 | verify: matching response `Selected{approve-once}` | `{"valid":true}`, exit 0 |
| 5 | verify: response for tampered command (hash mismatch) | `AepError::ContentMismatch`, exit 8 |
| 6 | verify: response after `expires_at` | `Expired`, exit 8 |
| 7 | verify: CRITICAL-tier response with `proof: null` | `ProofRequired`, exit 8 |
| 8 | verify: outcome `Cancelled` (well-formed) | `{"valid":true}`, exit 3 |
| 9 | schema snapshot: `schemars` output for both types unchanged vs committed fixture | additive-only rule enforced |

### Explicit out-of-scope (next version)
`allow_always`/blanket options; transport adapters (ACP conversion layer, MCP
gateway wiring per ADR-043, any phone/menu-bar/canvas UI); request signing +
key distribution; multi-approver quorum/delegation; batch requests; pending-
request persistence/replay; label localization; `caro serve` wrapper.

### Constraint compliance
Reuses validator/safety/config throughout (no parallel assessment types); all
new types serde+schemars from day one; pure subprocess, no daemon, no state;
the analog's core failure mode (approval unbound from content, unattributed,
session-locked) is solved structurally by hash binding + actor block +
self-contained expiry — not by workaround.

---
*Sources: [ACP overview](https://agentic-ai.readthedocs.io/en/latest/Standards/agent-client-protocol/) ·
[ACPex protocol overview](https://hexdocs.pm/acpex/protocol_overview.html) ·
[agent-client-protocol-schema v1.6.0 docs](https://docs.rs/agent-client-protocol-schema) —
[RequestPermissionRequest](https://docs.rs/agent-client-protocol-schema/latest/agent_client_protocol_schema/v1/struct.RequestPermissionRequest.html),
[RequestPermissionOutcome](https://docs.rs/agent-client-protocol-schema/latest/agent_client_protocol_schema/v1/enum.RequestPermissionOutcome.html) ·
Hermes scan `.hermes/digests/2026-08-06-weekly-agent-market-scan.md`.*
