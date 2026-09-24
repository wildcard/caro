# ADR-045: Strong-Authorization Approval Gate — Content-Bound Local Second Factor for CRITICAL-Tier Commands

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-05
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: MoonPay **PayBox** (launched 2026-07-29;
  single-action, expiring passkey approvals gating agent-initiated
  transactions) and Claude Code **`--permission-prompt-tool`**
  (undocumented delegated-approval surface, headless `--print` only;
  anthropics/claude-code #1175 asks for a working example) — read
  against the documented failure mode that keystroke approvals are
  spoofable by the supervised agent itself (pty/tmux self-approval;
  `--dangerously-skip-permissions` auto-approving even the
  sandbox-bypass prompt, google-antigravity/antigravity-cli #36)
- **Depends on**: ADR-020 (tiered approval — `SuggestedRouting`; this ADR
  adds the tier above `HumanGate`), ADR-040 (tiered authorization policy
  file — where `strong-gate` is declared)
- **Relates to**: ADR-024 (envelope + exit-code registry 0–6), ADR-027
  (exit 7), ADR-029 (exit 8), ADR-032 (execution receipts — carry
  `ApprovalProof`), ADR-034 (exit 9), ADR-037/041 (lifecycle events —
  `approval_granted`/`approval_denied` carry `ApprovalProof`), ADR-039
  (exit 10), ADR-044 (fail-closed effects resolution — same fail-closed
  posture), Hermes market scan 2026-08-05 (opportunity c)
- **Numbering note**: highest existing is ADR-044; per
  `.claude/rules/adr-numbering.md`, renumber on merge if another 045
  lands first. Exit code 11 claimed here; registry precedent 0–10.

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Target selection
> rationale: of the 2026-08-05 Hermes scan's opportunities, (a) the MCP
> gateway is scoped by ADR-038/043; (b) audit trail/export is
> substantially scoped by ADR-037/041 plus ADR-032 receipts; (d) the
> sandbox handshake extends ADR-039; (e) is content, not architecture.
> Opportunity (c) — "strong-authorization tier for CRITICAL commands …
> local WebAuthn/TouchID gate in the approval path; no network
> dependency" — has no covering ADR. Treat provider ordering (Decision
> D3) as a reviewable assumption.
>
> **Validation-discipline note.** This is architecture scope with no PMF
> claim. Gate obligations under `.claude/rules/validation-discipline.md`
> attach to the feature-spec/implementation PR, not to this document.

---

## Context

### The gap

Caro's approval today is a keystroke: `dialoguer::Confirm` at the
`requires_confirmation` site (`src/main.rs:3679`), defaulting to No, with
`--confirm/-y` as a blanket bypass. ADR-020 wires `SuggestedRouting`
(`AutoApprove | AsyncLog | HumanGate | Block`) to that prompt; ADR-040
makes the risk→action mapping declarative. But every action short of
`Block` is ultimately satisfiable by a single byte on stdin — and the
market's reference designs just moved past that.

### Phase 1 — what the analogs do, and where they fail

**PayBox** (2026-07-29): agent prepares a transaction; a human approves
via passkey; MPC/TEE key-splitting means neither the agent nor the vendor
can sign alone. Two properties matter for caro: approvals are **scoped to
a single action and expire after use**, and the approval factor is
**out-of-band from the agent's I/O channel** — an agent cannot fake a
passkey ceremony. Its "Autonomous mode within spending limits" is the
same tier-downgrade idea as ADR-040's policy mapping.

**Claude Code `--permission-prompt-tool`**: delegates "should we allow
this tool call?" to an MCP tool. Failure modes, current as of Aug 2026:

1. **Undocumented and example-free** (#1175); headless-only coupling to
   `--print`.
2. **`updatedInput` mutation**: the permission tool may rewrite the tool
   input *after* the human-visible decision — approval is not bound to
   the content that executes (a review/execute TOCTOU gap, same family
   as the plan-mode gap catalogued in lowercase ADR-008).
3. **Self-approval**: the supervised agent can drive a pty/tmux and
   answer its own keystroke prompts; and
   `--dangerously-skip-permissions` auto-approves even the
   "may the agent bypass the sandbox?" prompt (antigravity-cli #36).
   Approval fatigue makes users opt into exactly this.
4. **No proof artifact**: a granted approval leaves no verifiable record
   binding approver, content, and time — nothing to hand an auditor
   (EU AI Act Art. 50 enforcement began 2026-08-02).

## Decision

Add a fifth routing action, **`StrongGate`**, between `HumanGate` and
`Block`: the command executes only after a **local, out-of-band,
content-bound** human authorization ceremony. No network dependency, no
daemon, pure subprocess.

### D1 — Content binding (what-you-approve-is-what-runs)

The approval challenge is `SHA-256` over the canonical serialization of
an `ApprovalRequest` = { command string, risk level, matched pattern ids,
ADR-040 policy digest, platform target, nonce, issued_at }. The prompt
displays the command and the short hash; the resulting `ApprovalProof`
records the full hash. Immediately before spawn, caro recomputes the
hash from the command it is about to execute; any mismatch → exit 11,
nothing runs. This kills the `updatedInput`-class TOCTOU by design.

### D2 — Out-of-band factor (self-approval immunity)

The ceremony must not be satisfiable through stdin/tty:

- **Biometric provider** (macOS): LocalAuthentication evaluation
  (Touch ID / Watch), via `robius-authentication` (MIT/Apache-2.0,
  multi-platform abstraction) — subject to the ADR-045 Phase-0 build
  spike below.
- **FIDO2 provider** (Linux + cross-platform): a user-verification
  assertion from a security key, challenge = the approval hash — the
  returned signature *is* the proof, verifiable offline against the
  enrolled credential's public key.

A pty-driving agent can type `y`; it cannot synthesize a fingerprint or
a key touch. `--confirm/-y` and any future yolo flag **do not** satisfy
`StrongGate` — that immunity is the tier's definition and gets its own
integration test.

### D3 — Provider order and fallback (reviewable)

Resolution order: platform biometric → FIDO2 → **fail closed**
(exit 11, `error.kind = "strong_auth_unavailable"`). Non-interactive /
headless invocations fail closed unconditionally: a strong gate that can
be satisfied without a human present is not a strong gate (ADR-044
posture). No silent downgrade to keystroke; a policy-file
`downgrade = "human-gate"` escape hatch is explicit, logged, and off by
default.

### D4 — Proof artifact

`ApprovalProof { method, challenge_hash, issued_at, duration_ms,
provider_data }` (serde from day one). `provider_data` carries the FIDO2
signature + credential id when cryptographic, and is empty for
LocalAuthentication (whose OS attestation is not exportable — recorded
as method-level provenance only). The proof embeds into ADR-032 receipts
and ADR-037/041 `approval_granted` / `approval_denied` events — the
machine-readable human-override record compliance now asks for.

### D5 — Activation

Off unless declared: ADR-040 policy file maps a tier to `strong-gate`
(e.g. `critical = "strong-gate"`). No new CLI mode; the existing
approval site branches on the resolved action. Features
`strong-auth-biometric` / `strong-auth-fido2` are optional, not in
`default`, per `.claude/rules/external-sdk-integration.md` — first PR is
a ≤100-LOC build spike (license ✓ MIT/Apache-2.0, MSRV ≤ 1.85 to
verify, optional dep + flag, `pub fn smoke()`, two verification builds).

## Consequences

**Positive**: self-approval and approval-mutation become impossible by
construction, not by policy; caro gains the propose → structured review →
human cryptographic sign-off shape the market converged on (PayBox,
Gemini Spark) while staying local-only; auditors get a proof artifact.

**Negative / costs**: two new optional native deps (spike-gated); a
fifth enum variant ripples through ADR-020/040 mapping tables; UX
friction on CRITICAL-tier commands is the point but needs careful copy;
LocalAuthentication proofs are attestation-free (documented limitation).

**Neutral**: headless callers needing CRITICAL commands must either
pre-approve via policy or run interactively — consistent with ADR-027's
resolution contract.

## Alternatives considered

1. **Delegate to an external approval service (Slack/webhook)** —
   rejected for v1: network dependency, availability coupling, and it
   re-opens the relay-spoofing hole locally. Composes later via ADR-035.
2. **OS keychain unlock as the factor** — rejected: unlock state is
   session-cached; not single-action-scoped.
3. **`sudo`-style PAM integration (`pam_tid`)** — rejected: macOS-only,
   silently falls back to password over the same tty (spoofable), and
   couples caro to PAM stack configuration.
4. **Signed keystroke (ed25519 local key)** — rejected as primary: the
   key lives where the agent runs; it proves *a machine*, not *a
   human*. Kept as possible CI attestation in a future ADR.
5. **Do nothing / rely on ADR-039 sandbox placement** — rejected:
   sandbox placement changes blast radius, not authorization; the
   Aug-2026 escape disclosures show containment alone fails.

## Exit-code registry delta

| Code | Meaning | Introduced |
|---|---|---|
| 0–6 | per ADR-024 | ADR-024 |
| 7–10 | permission / schema / degraded / sandbox | ADR-027/029/034/039 |
| **11** | strong auth unavailable, denied, or approval-binding mismatch (`error.kind` disambiguates) | **this ADR** |
