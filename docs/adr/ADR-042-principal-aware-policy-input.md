# ADR-042: Principal-Aware Policy Input — Restrict-Only Identity Selectors in the Authorization Policy File

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-29
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: **Prefactor agent authentication** (PH launch
  2026-07-29 — agent login, delegated access, "auth as code", MCP-compliant
  per-agent identity; closed platform), read against two OSS references:
  the **MCP 2026-07-28 `clientInfo` self-assertion model** (already audited
  in ADR-038) and the **SPIFFE ID naming standard** (`spiffe://trust-domain/path`,
  CNCF, spiffe/spiffe `standards/SPIFFE-ID.md`)
- **Amends**: ADR-040 (tiered authorization policy file — this ADR adds one
  optional selector block to that schema before it is implemented, so the
  change is additive-on-paper, not breaking-in-code)
- **Relates to**: ADR-021 (execution attribution — owns `--agent-id`),
  ADR-024 (headless envelope + frozen exit-code registry), ADR-032
  (receipts carry policy digest), ADR-037/041 (event envelope `agent_id`
  field, populated from `CARO_AGENT_ID`), ADR-038 (MCP validator —
  `clientInfo` passthrough), Hermes market scan 2026-07-29 (opportunity 4)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Target selection
> rationale: of the 2026-07-29 Hermes scan's opportunities, (1)
> `--emit-events` is scoped by ADR-037/041, (2) the MCP validator by
> ADR-015/038, (3) "governed execution" positioning is content, not
> architecture. Opportunity 4 — "reserve an optional identity/principal
> field in the policy schema now to avoid a breaking change later" — is
> the only engineering item with no ADR behind it (ADR-021 covers
> *retrospective* attribution; ADR-040 carries policy identity only into
> audit artifacts; nothing makes caller identity a *prospective* policy
> input). The scan's directive is explicitly schema headroom, **not** an
> identity integration; this ADR obeys that boundary. Treat the
> restrict-only invariant (Decision D2) as the load-bearing reviewable
> assumption.
>
> **Validation-discipline note.** Architecture scope, no PMF claim. The
> scan records zero discovery transcripts for identity features; this ADR
> therefore reserves schema, defines invariants, and stops. Gate
> obligations attach to any future spec that builds actual identity
> integration.

---

## Context

### Phase 1 — What the researched feature is

**The problem it solves, and for whom.** Prefactor (launched today) sells
authentication *for agents*: each agent gets a unique identity instead of
borrowed human credentials or static API keys; users grant agents scoped,
delegated permissions after SSO ("delegator consent"); every action links
back to both the agent and the authorizing user; and the whole grant
surface is defined as versioned code deployed through CI/CD. Buyers are
platform teams facing ~45:1 non-human-to-human identity ratios with no
auditable trail of *which agent* did *what* under *whose* authority.

**Core architecture (from public docs; platform is closed-source):**
identity issuance and verification live in a cloud control plane federated
with existing IdPs (Auth0, Okta, Firebase, Clerk); the MCP layer assigns
per-agent identities and threads them through tool calls; policy is
versioned artifact ("auth as code") with audit trails.

**Why it is limited, and the failure modes that matter to caro:**

1. **No enforcement point at execution.** Prefactor answers *who is
   asking*; it has no opinion on *whether this shell command is safe to
   run*. Identity and execution-safety are complementary layers — the
   scan's core observation — and the execution side is caro's.
2. **Cloud-tethered.** Verification requires the control plane and an
   IdP. The local runtimes multiplying on caro's home turf (Osaurus, Sim,
   llama.cpp-MCP stacks) run offline on endpoints where no such verifier
   exists.
3. **The self-assertion gap at the boundary.** The identity signal that
   actually reaches a local validator today is *self-asserted*. MCP's
   2026-07-28 RC carries `io.modelcontextprotocol/clientInfo` in `_meta`
   on every request — a client can claim any name (audited in ADR-038).
   Likewise `CARO_AGENT_ID` (ADR-041) and Atuin-style `author:` tags
   (ADR-021) are whatever the caller says they are. **Any design that
   lets an unverified identity *unlock* privileges converts a spoofed
   string into privilege escalation.** This is the specific failure mode
   this ADR must solve by design, per the task constraints.

**Naming reference.** SPIFFE (CNCF) standardizes workload identity naming
as a URI: `spiffe://<trust-domain>/<path>`, e.g.
`spiffe://acme.internal/ci/deploy-agent`. The name is separable from
verification (SVIDs/SPIRE); adopting the *format* costs nothing and keeps
the field forward-compatible with real verification later.

### Phase 2 — Differentiation and existing caro infrastructure

**What Prefactor gets right, worth replicating:** identity as versioned,
digested artifact (caro already does this for policy — ADR-040's
`digest`); every decision linkable to a principal in the audit trail
(caro's envelope/receipts/events are the natural carriers); per-principal
*constraint* of what an agent may do (maps exactly onto ADR-040's tier
overrides).

**What caro can do that they cannot:** enforce at the execution boundary,
offline, agent-agnostically, with no control plane — and remain *safe
under self-asserted identity* by construction (Decision D2). Identity
platforms must verify to be useful; a restrict-only policy input is
useful even unverified.

**Existing infrastructure already covering part of this:**

| Piece | Where | Status |
|---|---|---|
| Identity assertion channel | `CARO_AGENT_ID` env (ADR-041), `--agent-id` flag (ADR-021) | Scoped |
| Identity in audit artifacts | `EventEnvelope.agent_id` (037/041), receipts (032) | Scoped |
| Declarative tier policy + layered resolution + digest | ADR-040 | Scoped |
| Verdict vocabulary | `SuggestedRouting` (`AutoApprove`/`AsyncLog`/`HumanGate`/`Block`), `src/models/` | Shipped |
| MCP `clientInfo` passthrough | ADR-038 | Scoped |

The gap is one edge: nothing connects the assertion channel to the policy
evaluation. ADR-040's schema, if implemented as written, would need a
breaking `schema_version` bump to add it later. Amending it now, while it
is still paper, is the entire point of this ADR.

---

## Decision

### D1 — One additive selector block in the ADR-040 schema

`[principal.'<pattern>']` mirrors `[env.<name>]` exactly: a sparse
tier→action override map, selected at resolution time by matching the
asserted principal id against `<pattern>` (glob semantics, case-sensitive
path per SPIFFE; first-match-wins in file order, lint on overlapping
patterns).

```toml
[policy]
schema_version = 1            # unchanged — additive field, absent block is legal
name = "acme-default"

[tiers]
safe     = "auto"
moderate = "log"
high     = "approve"
critical = "block"

[principal.'spiffe://acme.internal/ci/*']
moderate = "approve"          # tighter than baseline "log" — legal
high     = "block"            # tighter than baseline "approve" — legal

[principal.'agent:untrusted-*']
safe     = "approve"          # even safe commands gated for this caller
```

Principal ids are opaque strings; SPIFFE-format URIs are the documented
recommendation, `agent:<name>` the lightweight convention (matching
ADR-021's attribution tags). Empty/absent assertion ⇒ no principal ⇒
block never matches ⇒ baseline policy. Resolution order within ADR-040's
layered model: principal override applies *after* env override within the
same layer; it is a selector inside a policy file, **not** a new
`PolicySource` layer.

### D2 — Restrict-only invariant (the failure mode, solved by design)

A principal match may only move a tier's action toward *more* restrictive
(`AutoApprove < AsyncLog < HumanGate < Block`); resolution computes
`max(baseline_action, principal_action)` per tier. Consequences:

- **Spoofing is pointless.** Claiming someone else's id can only get the
  caller a stricter tier or the unchanged baseline. Self-asserted
  identity (env, flag, MCP `clientInfo`) is therefore *safe to consume
  unverified* — the property neither Prefactor-at-a-distance nor raw
  `clientInfo` gives a local validator.
- **Load-time lint, resolution-time guarantee.** A `[principal.*]` entry
  looser than the same file's `[tiers]` baseline is rejected at load
  (exit 11, `error.kind: "principal_grant"`, reusing ADR-040 D5's
  `PolicyInvalid` — no new exit code). Because the effective baseline can
  come from another layer, the runtime clamp is the actual invariant; the
  resolved output records `clamped: true` when it fires.
- **Grants require verification.** Loosening for a principal (the
  Prefactor-style use case) is deliberately impossible until a future ADR
  introduces *verified* principals (SVID/OIDC). The schema reserves
  nothing that would have to break to add it: a verified principal is the
  same id with a different `source`.

### D3 — Types (additive to ADR-040's module; serializable day one)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Principal {
    pub id: String,                     // opaque; SPIFFE URI recommended
    pub source: PrincipalSource,        // how it was asserted
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalSource { Flag, Env, McpClientInfo }
// Verified variants (e.g. SpiffeSvid, OidcJwt) are future additive arms.

// AuthorizationPolicy gains:
//   pub principals: Vec<PrincipalOverride>,        // ordered; empty = absent
// pub struct PrincipalOverride {
//     pub pattern: String,                          // glob over Principal.id
//     pub tiers: BTreeMap<RiskLevel, SuggestedRouting>,
// }
// ResolvedPolicy gains:
//   pub principal: Option<PrincipalResolution>,
// pub struct PrincipalResolution {
//     pub id: String,
//     pub source: PrincipalSource,
//     pub matched: Option<String>,     // pattern, None if no match
//     pub clamped: bool,               // true if D2 clamp fired
// }
```

`SuggestedRouting` gains `Ord` with the D2 ordering (also needed by
ADR-040's own monotonicity check). ADR-040's `digest` extends to include
the matched pattern and the four *post-clamp* actions (additive input to
the hash; absent principal hashes identically to today's definition, so
existing fixtures survive).

Precedence of assertion sources when several are present:
`--agent-id` flag > `CARO_AGENT_ID` env > MCP `clientInfo` (wiring for
the last is ADR-038's, out of scope here beyond the enum arm existing).

### D4 — Output contract (machines depend on this)

- **Headless envelope (ADR-024):** the `policy` block gains optional
  `principal: {id, source, matched, clamped}`. Absent when no assertion.
- **Events (ADR-037/041):** no schema change — `agent_id` already carries
  the raw assertion; the policy digest in `assessment` events now
  reflects principal-derived actions automatically.
- **Exit codes:** none added, none changed. Registry stays 0–11 frozen
  (0–6 ADR-024, 7/8/9/10/11 per 027/029/034/039/040). A
  principal-triggered block exits exactly as any other block; a
  `principal_grant` policy error is exit 11.
- **Pure subprocess:** matching is deterministic string/glob work at
  policy resolution; no daemon, no state, no network, no new
  initialization cost when the block is absent.

### D5 — Minimal file set (all changes ride ADR-040's implementation PR
or a small follow-up; no new modules)

| File | Change |
|---|---|
| ADR-040's policy module (`src/config/policy.rs` as designated there) | `PrincipalOverride` parse + lint + clamp in resolution |
| `src/models/mod.rs` | `Principal`, `PrincipalSource`, `Ord` for `SuggestedRouting` |
| `src/cli/` | read `--agent-id`/`CARO_AGENT_ID` into `Option<Principal>` (flag itself is ADR-021's) |
| `src/main.rs` | thread principal into resolution; envelope `policy.principal` |
| `docs/headless-contract.md` | `policy.principal` block; `principal_grant` error kind |

### Integration tests (known input → deterministic JSON + exit code)

1. **Headroom is free:** no assertion, policy without `[principal.*]` ⇒
   resolution JSON byte-identical to ADR-040's fixtures; digest
   unchanged. Exit 0.
2. **Restrict works:** `CARO_AGENT_ID=spiffe://acme.internal/ci/deploy`
   \+ the example policy, moderate-risk command ⇒ action `approve`,
   `policy.principal.matched: "spiffe://acme.internal/ci/*"`,
   `clamped: false`. Deterministic envelope; block-path exit codes as
   per ADR-024.
3. **Spoof yields nothing:** arbitrary `CARO_AGENT_ID` matching no
   pattern ⇒ baseline actions, `matched: null`. Same verdict JSON as
   test 1 modulo the `principal` echo.
4. **Grant rejected:** policy with `[principal.'agent:x']
   high = "auto"` under baseline `high = "approve"` ⇒ exit 11,
   `error.kind: "principal_grant"`, JSON error report on stdout with
   `--json`.
5. **Cross-layer clamp:** looser principal action injected via a
   lower-precedence file that lints clean in isolation ⇒ runtime clamp,
   `clamped: true`, effective action = baseline. (This is the test that
   proves D2 is a resolution invariant, not just a lint.)

## Consequences

**Benefits:** the breaking change the scan warned about is pre-empted at
paper cost; caro gains the "identity layer + execution layer" joint story
(consume Prefactor-style ids; never depend on them) while staying
offline-safe; audit artifacts, digest, and envelope tell one coherent
who/what story across ADR-021/032/037/040.

**Trade-offs / risks:** restrict-only means the popular "trusted CI agent
auto-approves more" use case is impossible until verified identity lands
— deliberate, and reviewable; glob matching semantics (first-match,
overlap lint) add a small parser surface to ADR-040's PR; if ADR-040's
implementation has already started when this merges, the amendment must
land in the same release to avoid shipping `schema_version = 1` twice
with different shapes.

## Alternatives considered

1. **Full identity integration (OIDC/Prefactor/SPIRE) now** — rejected:
   zero discovery transcripts (validation-discipline), scan explicitly
   lists it as the thing to avoid; the layer is being claimed by
   dedicated players.
2. **Principal as a new resolution layer (`PolicySource::Principal`)** —
   rejected: layers encode file provenance/precedence; principal is a
   runtime *selector*, symmetric with `env`. Keeping it a selector
   preserves ADR-040's ceiling semantics untouched.
3. **Allow grants behind a `--trust-asserted-identity` flag** — rejected:
   a footgun that converts env-var spoofing into privilege escalation;
   exactly the Phase-1 failure mode. Verified-identity grants get their
   own ADR with real verification.
4. **Do nothing until verification exists** — rejected: ADR-040
   implements `schema_version = 1` soon; retrofitting the selector later
   is a breaking schema bump, the precise cost this week's scan flagged.

## Out of scope (next version / other ADRs)

- Identity **verification** of any kind (SPIFFE SVIDs, OIDC/JWT,
  Prefactor federation) and the verified-`PrincipalSource` arms' wiring.
- Privilege **grants** for verified principals; delegated-consent chains
  (user→agent authorization à la Prefactor).
- MCP `clientInfo` → `Principal` wiring (ADR-038's server; the enum arm
  exists so it lands additively).
- Per-principal allowlists/denylists of command patterns, rate limits,
  or audit queries (`caro events --principal …`).
- Identity issuance — never caro's business.

## References

- Hermes scan 2026-07-29, opportunity 4 (`.hermes/digests/`)
- Prefactor — [product](https://prefactor.tech/), [agent identity & MCP auth](https://prefactor.tech/blog/prefactor-mcp-auth-agent-identity-and-the-future-of-authentication/), [access-control practices](https://prefactor.tech/blog/5-best-practices-for-ai-agent-access-control)
- SPIFFE ID standard — [spiffe/spiffe SPIFFE-ID.md](https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE-ID.md), [concepts](https://spiffe.io/docs/latest/spiffe-about/spiffe-concepts/)
- ADR-021, ADR-024, ADR-032, ADR-037, ADR-038 (§ `clientInfo` self-assertion), ADR-040, ADR-041

**Numbering note:** highest existing is ADR-041; per
`.claude/rules/adr-numbering.md`, renumber on merge if another 042 lands
first.
