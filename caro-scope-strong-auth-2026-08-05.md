# Implementation Scope — Strong-Authorization Approval Gate (CRITICAL Tier)

**Feature under analysis:** MoonPay **PayBox** single-action passkey approvals for
agent-initiated actions (launched 2026-07-29) + Claude Code **`--permission-prompt-tool`**
delegated-approval surface (undocumented, headless-only).
**Equivalent we are scoping for Caro:** a `StrongGate` routing tier — CRITICAL commands
execute only after a local, out-of-band, content-bound human ceremony (Touch ID / FIDO2),
producing a serializable `ApprovalProof` for receipts and lifecycle events.

**Date:** 2026-08-05 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-045-strong-auth-approval-gate.md`

> The task template left `[FEATURE NAME]` unfilled and ran unattended. Target selection:
> the 2026-08-05 Hermes scan's opportunity (c) — strong-authorization tier — is the only
> "Now/Next" item with no covering ADR: (a) MCP gateway → ADR-038/043; (b) audit
> trail → ADR-032/037/041; (d) sandbox handshake → ADR-039; (e) is content. Provider
> ordering and crate choices below are reviewable assumptions, not settled decisions.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom
A keystroke approval proves a byte arrived on stdin, not that a human decided. Users
supervising agents on consequential actions (disk ops, credential use, prod targets)
need approval an agent cannot forge and auditors can verify. PayBox's answer: each
action gets a fresh passkey ceremony, scoped to that action, expiring after use, with
key material split (MPC/TEE) so the agent can never sign alone.

### Core architecture observed
- **PayBox**: agent prepares → structured review surface → human passkey → scoped
  credential issued → action executes. Two modes: Always-Ask (per-action ceremony)
  vs Autonomous-within-limits (policy downgrade) — the same shape as ADR-040's
  declarative risk→action mapping.
- **`--permission-prompt-tool`**: Claude Code forwards "allow this tool call?" to an
  MCP tool that returns allow/deny plus an optional `updatedInput` that rewrites the
  tool input before execution. Works only with `--print`.

### Why experimental / failure modes found
1. **Self-approval.** The supervised agent can drive a pty/tmux and answer its own
   prompt; `--dangerously-skip-permissions` auto-approves even the sandbox-bypass
   question (antigravity-cli #36). Approval fatigue pushes users into exactly this.
2. **Approval ≠ content.** `updatedInput` lets the input mutate after the decision —
   review/execute TOCTOU, same family as the plan-mode gap (lowercase ADR-008).
3. **No proof artifact.** Granted approvals leave no verifiable record of who
   approved what, when — now a compliance gap (EU AI Act enforcement since Aug 2).
4. **Undocumented contract.** `--permission-prompt-tool` has no working example
   (anthropics/claude-code #1175); shapes are version-coupled.

### Structured output / lifecycle
PayBox approvals are single-use and expiring (good contract, closed source). Claude
Code's surface has no exit-code distinction for "denied by permission tool" vs other
failures and no schema versioning. Neither emits an offline-verifiable proof.

## Phase 2 — Competitive Differentiation

### What they get right (replicate)
- Single-action, expiring approvals — never session-blanket.
- Out-of-band factor (passkey) the agent's I/O channel cannot synthesize.
- Policy-declared autonomy tiers (PayBox limits ≙ ADR-040 mapping).

### Their gaps (avoid by design)
| Their gap | Our design answer |
|---|---|
| Self-approval via pty / yolo flags | Factor is biometric/FIDO2 hardware — `-y` explicitly does **not** satisfy `StrongGate` |
| `updatedInput` TOCTOU | Challenge = SHA-256 of canonical `ApprovalRequest`; re-hash immediately before spawn; mismatch → exit 11, no spawn |
| No proof artifact | `ApprovalProof` (serde) embedded in ADR-032 receipts + ADR-037/041 events; FIDO2 path is offline-verifiable |
| Network/vendor dependency (PayBox MPC service) | Local-only ceremony; no daemon, no network |
| Undocumented, unversioned contract | Envelope + exit-code registry (ADR-024 lineage), `schema_version` on all payloads |

### Our unique positioning
Universal, offline, standalone: any agent that shells out to caro gets a spoof-proof
human gate with a deterministic 52-pattern validator behind it — no cloud vault, no
platform lock-in. The "reviewer that isn't the agent" (scan opportunity e), now with
an approval the agent can't counterfeit.

### Existing infrastructure reused
`SuggestedRouting` + `SafetyAssessment.suggested_routing` (ADR-020, `src/models/mod.rs:189`,
`src/safety/mod.rs:192`); ADR-040 policy file for activation; `sha2` (already a direct
dep) for binding; `dialoguer` prompt site (`src/main.rs:3679`) as the insertion point;
receipts/events (ADR-032/037/041) as proof carriers; feature-flag spike pattern from the
`governance` flag (PR #1103 precedent).

## Phase 3 — Scope Definition

### New ADR
`docs/adr/ADR-045-strong-auth-approval-gate.md` (written this run) — decisions D1–D5:
content binding, out-of-band factor, fail-closed fallback, proof artifact, policy-file
activation. Renumber per `.claude/rules/adr-numbering.md` if another 045 lands first.

### New types (existing modules; all serde from day one)

| Where | Type | Fields / contract |
|---|---|---|
| `src/models/mod.rs` | `SuggestedRouting::StrongGate` | new variant between `HumanGate` and `Block`; additive, serde-compatible |
| `src/models/mod.rs` | `ApprovalRequest` | `command`, `risk`, `pattern_ids`, `policy_digest`, `platform`, `nonce`, `issued_at`; canonical serialization defined (sorted keys, no floats) |
| `src/models/mod.rs` | `ApprovalProof` | `method: StrongAuthMethod`, `challenge_hash`, `issued_at`, `duration_ms`, `provider_data: Option<Fido2Assertion>` |
| `src/models/mod.rs` | `StrongAuthMethod` | `Biometric \| Fido2 \| Downgraded` |
| `src/safety/strong_auth.rs` (new file, existing module) | `trait StrongAuthProvider` | `fn authorize(&self, req: &ApprovalRequest) -> Result<ApprovalProof, StrongAuthError>`; impls behind features; `MockStrongAuth` under `mock-backend` |

### Minimal file set

| File | Change |
|---|---|
| `Cargo.toml` | optional deps `robius-authentication` (biometric), `ctap-hid-fido2` (fido2); features `strong-auth-biometric`, `strong-auth-fido2`; **not in default** |
| `src/models/mod.rs` | types above + `Display`/serde |
| `src/safety/strong_auth.rs` | provider trait, hash binding, resolution order, smoke fn (Phase 0) |
| `src/safety/mod.rs` | `pub mod strong_auth;` + routing resolution honors `StrongGate` |
| `src/main.rs` | branch at the `requires_confirmation` site: `StrongGate` → ceremony instead of `Confirm`; `-y` immunity; exit 11 wiring |

Phase 0 is a build spike per `.claude/rules/external-sdk-integration.md`: ≤100 LOC,
license check (both crates MIT/Apache-2.0 — verify transitives via `cargo deny`),
MSRV ≤ 1.85, `pub fn smoke()`, two verification builds. Net new code after Phase 0
≈ 400–500 LOC + tests.

### Exit code / output contract
- Ceremony success → normal flow (exit 0/…); envelope gains `approval_proof`.
- Exit **11** (new): `error.kind` ∈ `strong_auth_unavailable` (no provider / headless),
  `strong_auth_denied` (human rejected or ceremony failed), `approval_binding_mismatch`
  (command changed between approval and spawn). Nothing executes on 11.
- `--confirm/-y`, `--approval auto` never satisfy `StrongGate` (tested).
- Determinism: same request + policy ⇒ same `challenge_hash` modulo `nonce`/`issued_at`
  (both excluded from the canonical-content digest, included in the proof).

### Integration tests (known input → deterministic output)
1. Policy `critical = "strong-gate"` + `MockStrongAuth::approve` → exit 0; receipt and
   `approval_granted` event contain matching `challenge_hash`.
2. `MockStrongAuth::deny` → exit 11, `strong_auth_denied`, zero spawns.
3. Non-interactive (stdin not a tty) + strong-gate → exit 11, `strong_auth_unavailable`.
4. Harness mutates command post-approval → exit 11, `approval_binding_mismatch`.
5. `-y` with strong-gate policy → ceremony still required (mock asserts invocation).
6. `ApprovalRequest`/`ApprovalProof` serde round-trip property tests; canonical-hash
   golden vector.
7. FIDO2 assertion verification against a fixture credential (software authenticator).

### Out of scope (next version)
- Remote/asynchronous approval (Slack, webhook) — composes via ADR-035 later.
- Approval caching / "approve for 15 minutes" sessions.
- Windows Hello provider; Linux platform-biometric (fprintd) provider.
- Delegation bridge to Claude Code `--permission-prompt-tool` / MCP hosts (ADR-038/043 turf).
- ed25519 machine-attestation signatures for CI.
- Enrollment UX for FIDO2 credentials (`caro auth enroll`) beyond a minimal stub.

### Constraint check
- Reuses validator/safety/config/policy/receipt infrastructure — no duplication. ✔
- All new types serde from day one. ✔
- Pure subprocess: ceremony is synchronous in-process; no daemon, no state beyond the
  policy file and enrolled FIDO2 credential. ✔
- Phase-1 failure modes solved by design: out-of-band factor (self-approval impossible),
  content-bound hash re-checked at spawn (TOCTOU impossible), proof artifact emitted
  (audit gap closed), versioned envelope + exit 11 (contract documented). ✔

---
**Sources:** [MoonPay PayBox FAQs](https://support.moonpay.com/en/articles/669843-paybox-faqs) ·
[PayBox launch coverage](https://blog.thirdweb.com/ai-agents-can-now-pay-on-your-behalf-moonpay-paybox-explained/) ·
[Claude Code permissions docs](https://code.claude.com/docs/en/permissions) ·
[claude-code #1175 (permission-prompt-tool docs gap)](https://github.com/anthropics/claude-code/issues/1175) ·
[antigravity-cli #36 (skip-permissions bypasses sandbox prompt)](https://github.com/google-antigravity/antigravity-cli/issues/36) ·
[robius-authentication](https://github.com/project-robius/robius-authentication) ·
[localauthentication-rs](https://crates.io/crates/localauthentication-rs) ·
Hermes scan `.hermes/digests/2026-08-05-agent-market-scan.md`
