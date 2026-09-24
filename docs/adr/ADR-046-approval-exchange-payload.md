# ADR-046: Approval Exchange Payload (AEP v1) — A Transport-Independent, Content-Bound Request/Response Contract for Remote Approval Surfaces

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-06
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: Zed **Agent Client Protocol (ACP)** `session/request_permission`
  (agent-client-protocol-schema crate v1.6.0, Apache-2.0; wire protocol v1 stable,
  v2 draft behind `unstable_protocol_v2`)
- **Depends on**: ADR-020 (`SuggestedRouting` tier vocabulary), ADR-040 (tiered
  authorization policy file — option derivation), ADR-045 (`ApprovalProof` — the
  CRITICAL-tier proof carried in the response)
- **Relates to**: ADR-024/027 (headless envelope + exit-code registry this ADR
  extends), ADR-026 (session stream — one possible transport), ADR-032 (receipts
  reference `request_id`), ADR-035/036 (hooks — another transport), ADR-037/041
  (lifecycle events reference `request_id`), ADR-043 (MCP gateway — another transport)
- **Numbering note**: highest existing is ADR-045; per `.claude/rules/adr-numbering.md`,
  renumber on merge if another 046 lands first.

> **Provenance note (autonomous run).** Produced with no user present; the task
> template's `[FEATURE NAME]` was unbound. Target selection: the 2026-08-06 Hermes
> scan names "the versioned assessment/approval JSON schema" its top build item
> (opportunity a, plus opportunity b's `actor` attribution). No covering ADR exists:
> ADR-027 couples approval to the headless caller's stdin stream, ADR-037/041 are
> one-way event streams with no response leg, ADR-045 is the local ceremony with no
> remote-surface payload. The analog choice (ACP) is a reviewable assumption.

---

## Context

Approval surfaces are detaching from the terminal. This week's launches put agent
supervision on phones (Port22), menu bars (AgentMicro), shared canvases (Murmell),
and multiplayer sessions (mpai) — none with a validation layer, and none answering
*who* approved *what content*. Caro's approval flow today assumes the terminal
(dialoguer prompt) or the headless caller's stdin stream (ADR-027). There is no
payload a third-party surface can render and answer.

The researched analog is ACP's permission exchange, the only open, versioned,
schema-published contract for exactly this problem. Its shape (v1):

- `session/request_permission` request: `{ sessionId, toolCall: ToolCallUpdate,
  options: Vec<PermissionOption { id, label, kind }>, _meta }` — sent agent→client
  over JSON-RPC 2.0 on stdio inside a live session.
- Response: `RequestPermissionOutcome = Cancelled | Selected { optionId }`.
- On `session/cancel`, the client MUST answer all pending permission requests
  with `Cancelled`.
- All types are `#[non_exhaustive]`, serde + schemars, with a v2 draft and lossy
  `TryFrom` conversions between versions.

### Failure modes in the analog (each solved by design below)

1. **No assessment vocabulary.** The request carries the agent's *self-description*
   of the tool call (`ToolCallUpdate`: title, kind, rawInput). There is no
   independent risk assessment, no matched patterns, no rationale. The surface
   renders what the supervised party claims it will do.
2. **The supervised party authors its own ballot.** The `options` list (including
   `allow_always`) is composed by the agent requesting permission. A misaligned or
   compromised agent can offer only favorable options.
3. **Approval is not bound to content (TOCTOU).** The outcome carries only
   `optionId`. `ToolCallUpdate` is a *mutable* update type — `rawInput` can change
   after the request is displayed; nothing in the response proves what content was
   approved. Same family as the `updatedInput` gap found in ADR-045's Phase 1.
4. **No actor attribution.** `Selected { optionId }` says which button, not which
   human, on which surface, authenticated how. Multiplayer sessions (mpai) make
   "approved" without identity an audit-trail failure under EU-AI-Act-grade
   oversight documentation.
5. **Session-coupled lifecycle.** The request exists only inside a live JSON-RPC
   connection between an editor and a spawned agent subprocess. It cannot be
   carried to a phone, queued, or verified offline; cancellation semantics assume
   both ends stay up. Version churn (`non_exhaustive` everywhere, v1↔v2 lossy
   conversions) shifts breakage to consumers.

## Decision

Define the **Approval Exchange Payload (AEP v1)**: a versioned, self-contained,
transport-independent JSON request/response pair produced and verified by pure
subprocess calls. Caro emits the request; any surface renders it and returns the
response; caro verifies the response offline. No daemon, no session, no state.

### New types (in `src/safety/exchange.rs`, re-exported from `crate::safety`)

All types derive `Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq`
(serde + schemars are already direct deps of `src/safety`).

```rust
pub struct ApprovalRequest {
    pub schema_version: String,        // "aep/1" — literal, checked on parse
    pub request_id: String,            // UUIDv4 (test-overridable, see below)
    pub created_at: String,            // RFC 3339 UTC
    pub expires_at: String,            // RFC 3339 UTC; policy default 300s
    pub command: String,               // exact command text assessed
    pub command_sha256: String,        // hex SHA-256 of `command` (sha2, existing dep)
    pub shell: ShellType,              // existing type
    pub assessment: SafetyDecision,    // existing type — risk_level, reason,
                                       //   suggested_routing, matched_patterns, confidence
    pub options: Vec<ApprovalOption>,  // derived from ADR-040 policy, never caller-supplied
    pub agent_id: Option<String>,      // ADR-021 attribution of the *requesting* agent
    pub meta: Option<serde_json::Map<String, serde_json::Value>>,
}

pub struct ApprovalOption {
    pub id: String,                    // "approve-once" | "deny-once" | "block"
    pub label: String,                 // human label, surface may localize
    pub kind: ApprovalOptionKind,
}

#[serde(rename_all = "snake_case")]
pub enum ApprovalOptionKind { ApproveOnce, DenyOnce, Block }

#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApprovalOutcome {
    Selected { option_id: String },
    Cancelled,
    Expired,
}

pub struct ApprovalActor {
    pub id: String,                    // user identity as known to the surface
    pub surface: String,               // e.g. "terminal", "port22-ios", "zed"
    pub auth: ActorAuthKind,           // how the surface authenticated the human
}

#[serde(rename_all = "snake_case")]
pub enum ActorAuthKind { None, SurfaceSession, StrongAuth } // StrongAuth ⇒ proof required

pub struct ApprovalResponse {
    pub schema_version: String,        // "aep/1"
    pub request_id: String,            // MUST echo request
    pub command_sha256: String,        // MUST echo request — content binding
    pub outcome: ApprovalOutcome,
    pub actor: ApprovalActor,
    pub decided_at: String,            // RFC 3339 UTC
    pub proof: Option<ApprovalProof>,  // ADR-045; REQUIRED when policy tier is CRITICAL
    pub meta: Option<serde_json::Map<String, serde_json::Value>>,
}
```

**Method contracts** (unit-testable, no I/O):

- `ApprovalRequest::from_decision(cmd, shell, decision, policy, now) -> Option<Self>` —
  `None` when routing is `AutoApprove` or `Block` (no exchange needed); options come
  from the ADR-040 policy mapping only.
- `ApprovalResponse::verify(&self, request: &ApprovalRequest, now) -> Result<Verified, AepError>` —
  fails closed on: schema_version mismatch, request_id mismatch, hash mismatch,
  `now > expires_at`, missing proof when required, malformed proof. `AepError` is a
  `thiserror` enum, serializable, one variant per failure.

### How each analog failure mode is solved by design

| ACP gap | AEP design answer |
|---|---|
| No assessment in request | `assessment: SafetyDecision` — deterministic 52-pattern verdict, matched patterns, rationale, tier |
| Agent authors its own options | Options derived exclusively from the ADR-040 policy file; the assessed command's source can never add or remove options |
| Approval not bound to content | Response must echo `request_id` + `command_sha256`; verification recomputes the hash from the request's `command`; any drift → `AepError::ContentMismatch`, exit 8 |
| No actor attribution | Mandatory `actor { id, surface, auth }` (scan opportunity b); spoofing threat model documented honestly — surface-asserted identity is trust-on-surface unless `auth = strong_auth` + ADR-045 proof |
| Session-coupled, unverifiable offline | Self-contained payloads with explicit `expires_at`; verification is a pure function; `Expired` is a first-class outcome; no connection required |
| `non_exhaustive` version churn | Literal `schema_version: "aep/1"`; additive-only within v1; unknown fields ignored on parse, unknown `kind` values rejected |

### CLI surface (pure subprocess; no new binary)

- `caro assess "<query-or-command>" -o json [--command-only]` — runs the existing
  generation+validation path, prints exactly one JSON document to stdout:
  the ADR-024 envelope extended with an optional `approval_request` field.
- `caro approve verify --request <file|-> --response <file|->` — offline
  verification, prints a one-line JSON verdict `{ "valid": bool, "reason": ... }`.

### Exit-code contract (extends the ADR-024/027 registry; no reuse conflicts)

| Code | Meaning |
|---|---|
| 0 | `assess`: auto-approve (no exchange needed) · `verify`: response valid and outcome is approval |
| 3 | `assess`: hard Block (existing semantics) · `verify`: valid response, outcome is deny/cancel/block |
| 7 | `assess`: approval required — `ApprovalRequest` emitted (existing `needs_approval_no_channel` code, now meaning "request handed to caller") |
| **8** | `verify`: response invalid — content/id mismatch, expired, missing/invalid proof, malformed (**new code; registered here**) |
| 1/2/4/5/6, 201 | unchanged (ADR-024/027) |

## Consequences

**Positive.** Every remote-supervision surface becomes a potential caro consumer
without caro shipping any UI ("emit the payload; let surface owners render it" —
the scan's explicit avoid-list). ADR-043's MCP gateway, ADR-035/036 hooks, and
ADR-026/027 headless streams all carry the same two payloads instead of inventing
three approval dialects. Lifecycle events (ADR-037/041) and receipts (ADR-032)
gain a `request_id` join key, closing the assessment→decision→approver→outcome
chain that transcript-grade audit products (Inventory, Basedash) do not have.

**Negative / risks.** Actor identity below `strong_auth` is asserted by the
surface, not proven — the ADR documents this rather than hiding it; spoofed-actor
hardening is ADR-045's proof path. Two new public serialized types become
compatibility surface; the additive-only rule must be enforced by schema tests.
Exit code 8 is new registry surface for wrapper scripts.

**Neutral.** ACP interop is *conversion*, not adoption: an adapter mapping
`ApprovalRequest` → `session/request_permission` (and back) is possible later and
would follow `.claude/rules/external-sdk-integration.md` (build-spike first) if the
`agent-client-protocol-schema` crate is taken as a dep.

## Alternatives considered

1. **Adopt ACP's types wholesale** (Apache-2.0, Rust crate exists). Rejected:
   architecture mismatch — ACP assumes the *agent* requests permission inside a
   live editor session; caro is the out-of-band reviewer, and the five failure
   modes above are structural, not cosmetic. License and MSRV were not the blocker.
2. **Extend ADR-027's inline stream only.** Rejected: transport-coupled to the
   spawning caller's stdin; cannot reach detached surfaces, which is the market
   shift this ADR answers.
3. **`caro serve --assess` daemon** (the scan's literal phrasing). Rejected for
   v1: violates the pure-subprocess constraint; a serve mode can wrap these same
   payloads later without changing them.
4. **Sign the request as well as the response.** Deferred: request integrity is
   protected by the response echoing the hash; full request signing needs key
   management that ADR-045 scopes for CRITICAL only.

## Out of scope (next version)

`allow_always` / session-blanket options (deliberately absent from
`ApprovalOptionKind`); transport adapters (ACP, MCP gateway wiring — ADR-043;
phone/menu-bar anything); request signing and key distribution; delegation
chains / multi-approver quorum; batch requests; localization of option labels;
persistence or replay of pending requests.
