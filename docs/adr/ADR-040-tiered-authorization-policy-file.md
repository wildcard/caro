# ADR-040: Tiered Decision-Authorization Policy File — Declarative Risk→Action Mapping with Deterministic Layered Resolution and Auditable Provenance

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-27
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: OpenAI **Codex CLI approval & sandbox policy
  configuration** (`approval_policy`, `sandbox_mode`, named permission
  profiles, six-layer config precedence, `requirements.toml` admin
  ceiling) — openai/codex, OSS, the most mature declarative approval
  policy in any agentic CLI; docs fetched live 2026-07-27
  ([config basics](https://developers.openai.com/codex/config-basic))
- **Depends on**: ADR-020 (tiered approval — `SuggestedRouting` runtime
  wiring; this ADR makes its mapping user-configurable), ADR-024
  (headless JSON contract — envelope + exit-code registry)
- **Relates to**: ADR-027 (exit 7), ADR-029 (exit 8), ADR-032 (execution
  receipts — carry the policy digest), ADR-034 (exit 9), ADR-035
  (external policy hooks — dynamic validators; this ADR is the *static*
  layer they compose with), ADR-037 (lifecycle events — carry the policy
  digest), ADR-038 (MCP validator payloads), ADR-039 (sandbox placement
  axis, exit 10), Hermes market scan 2026-07-27 (opportunity 2)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Target selection
> rationale: of the 2026-07-27 Hermes scan's opportunities, (1) the
> SIEM event stream is scoped by ADR-037; (3) the MCP validator is scoped
> by ADR-015/038; (4) incident-taxonomy pattern grounding is data work
> for the `safety-pattern-developer` skill, not an architecture scope;
> (5) is content. Opportunity 2 — "Caro has risk levels but no
> user-facing autonomy-tier config … draft the policy-file format" —
> is the only Priority-Now engineering item with no ADR behind it
> (ADR-020 wires the *runtime*, ADR-035 scopes *dynamic hooks*; neither
> defines a declarative policy artifact). The analog choice (Codex CLI
> config, not the non-OSS OpenAI Presence) is a reviewable assumption.
>
> **Validation-discipline note.** This is an architecture scope, not a
> feature spec; it makes no PMF claim. Gate obligations under
> `.claude/rules/validation-discipline.md` attach to the implementing
> spec/PR, not to this document.

---

## Context

### The problem and who has it

Caro computes a four-tier routing verdict for every command
(`AutoApprove` / `AsyncLog` / `HumanGate` / `Block`) but the mapping from
risk to action is **hard-coded** in
`SuggestedRouting::from_risk_and_safety()` (`src/models/mod.rs`). Users
can move one global knob (`SafetyLevel`) and one mode knob
(`ApprovalMode`), but cannot express the thing buyers and regulators now
ask for directly:

> *"In CI, high-risk commands are always blocked. On dev laptops they
> need approval. Nothing auto-executes above moderate. Show me the file
> that says so, and prove the run used it."*

Two market events made this a compliance question rather than a
convenience: China's binding tiered agent-authorization framework
(effective 2026-07-26) codifies graduated autonomy tiers, and the x402
Foundation standard (Linux Foundation, 2026-07-22) mandates roles,
limits, and human oversight for agent-initiated actions. Products that
can emit **auditable evidence of tier decisions** have a regulatory
story; products that can't, won't (Hermes 2026-07-27, §2).

Consumers: platform/security teams standardizing agent safety across
environments; agent hosts embedding caro (ADR-036 hooks, ADR-038 MCP)
that need the host's org policy honored identically everywhere; auditors
who must reconstruct *which policy* produced a decision.

### Phase 1 — what the analog does, and where it breaks

Codex CLI's authorization surface is the most complete in the OSS field:

- **Two orthogonal knobs**: `approval_policy`
  (`untrusted` / `on-request` / `never`) — *when to pause for a human* —
  and `sandbox_mode` (`read-only` / `workspace-write` /
  `danger-full-access`) — *what the process may touch*.
- **Named permission profiles**: `[permissions.<name>]` tables bundling
  filesystem/network policy; built-ins `:read-only`, `:workspace`,
  `:danger-full-access`.
- **Profiles as bundles**: `--profile ci` activates a named settings
  bundle; docs recommend naming profiles after environments
  (`dev`, `ci`, `prod`).
- **Six-layer precedence**: CLI flags > project `.codex/config.toml`
  (trusted projects only) > profile file > user config > system
  `/etc/codex/config.toml` > built-ins.
- **Admin ceiling**: managed machines enforce `requirements.toml`
  (e.g. *disallow* `approval_policy = "never"` or
  `sandbox_mode = "danger-full-access"`).

That is the right shape. The tracker shows where it breaks — one failure
class, five symptoms, all live:

1. **Layering silently produces the wrong effective policy.**
   [#11885](https://github.com/openai/codex/issues/11885): starting with
   `workspace-write` + `untrusted` via alias/profile/config override
   yields `workspace-write` + **`on-failure`** — a *more permissive*
   approval policy than declared, with no error.
2. **Declared config not respected, silently.**
   [#3034](https://github.com/openai/codex/issues/3034) (profile sandbox
   settings ignored), [#6667](https://github.com/openai/codex/issues/6667)
   (`workspace_write` in config, effective mode read-only). Divergence in
   *both* directions — sometimes more permissive, sometimes more
   restrictive — and in neither case loud.
3. **Nondeterministic effective policy.**
   [#5038](https://github.com/openai/codex/issues/5038): approval prompts
   fire "randomly" for identical commands in one session despite
   `approval_policy = "never"`.
4. **Decisions invisible to the embedding host.**
   [#21982](https://github.com/openai/codex/issues/21982): an approval
   requirement is not surfaced through the app-server API — the machine
   consumer cannot see the tier decision it must answer.
5. **Path-shape edge cases flip decisions.**
   [#20720](https://github.com/openai/codex/issues/20720): a symlinked
   `.codex` inside the writable root forces approvals that the declared
   policy says are unnecessary.

**The specific failure mode, named:** *silent divergence between declared
and effective policy.* There is no command that prints the resolved
policy with per-field provenance, no digest binding a run's decisions to
the policy that produced them, and unknown/misplaced keys are ignored
rather than rejected. Every symptom above is undiagnosable by design.

### Phase 2 — what caro already has, and the gap

| Piece | Where | Status |
| --- | --- | --- |
| Risk tiers | `RiskLevel` (Safe/Moderate/High/Critical), serde + JsonSchema | Shipped |
| Threshold knob | `SafetyLevel` (strict/moderate/permissive) | Shipped |
| Decision-mode knob | `ApprovalMode` (prompt/auto/smart) | Shipped |
| Canonical risk→action mapping | `SuggestedRouting::from_risk_and_safety()` | Shipped, hard-coded |
| Runtime wiring of `HumanGate` | ADR-020 | Scoped |
| Config load/merge + schema registry | `ConfigManager`, `ConfigSchema` (`src/config/`) | Shipped (validator is a placeholder) |
| Exit-code registry | 0–6 (ADR-024), 7 (027), 8 (029), 9 (034), 10 (039) | Scoped/frozen |
| Audit artifacts to carry policy identity | receipts (032), lifecycle events (037), MCP payload (038) | Scoped |
| Dynamic per-command validators | ADR-035 external policy hooks | Scoped |

The gap is exactly one artifact: a **declarative, versioned,
environment-scoped policy file** that overrides the canonical mapping,
resolves deterministically across layers, and leaves an auditable trace.

**Differentiation available to caro and not to the analog:** the mapping
domain is a *closed, ordered 4×4 space* (four risk tiers × four actions),
not an open command-pattern space. That means caro can validate
**monotonicity** (a higher risk tier may never receive a more permissive
action than a lower one) at load time — a whole class of misconfiguration
Codex cannot even express a check for. Caro is also deterministic and
offline: same inputs, same resolved policy, same verdict, every time.

## Decision

Introduce `caro-policy.toml` — a declarative tiered-authorization policy
— plus a deterministic resolver and an introspection command, designing
out the analog's silent-divergence failure mode via three mechanisms:
**fail-loud parsing**, **an explain surface**, and **a policy digest
bound into every assessment artifact**.

### D1 — File format (`schema_version = 1`)

```toml
[policy]
schema_version = 1
name = "acme-default"          # free-form identifier, lands in audit artifacts

[tiers]                        # risk tier -> action
safe     = "auto"              # auto | log | approve | block
moderate = "log"
high     = "approve"
critical = "block"

[env.ci]                       # sparse override, selected by --env / CARO_ENV
high     = "block"
moderate = "approve"

[env.dev]
moderate = "auto"
```

- Actions parse into the existing `SuggestedRouting`
  (`auto`→`AutoApprove`, `log`→`AsyncLog`, `approve`→`HumanGate`,
  `block`→`Block`). **No new action vocabulary** — the audit trail,
  ADR-020 runtime, and ADR-038 payloads keep one enum.
- `#[serde(deny_unknown_fields)]` everywhere. An unknown key, a
  misspelled tier, an unknown `env` referenced by `--env` — each is a
  hard error, never a silent ignore (kills analog symptom 2 at parse
  time).
- **Monotonicity invariant**: with actions ordered
  `auto < log < approve < block` and risks ordered
  `safe < moderate < high < critical`, the effective action must be
  non-decreasing in risk. Violations are rejected at load (exit 11).
- A ceiling file (system-owned, e.g. `/etc/caro/policy-ceiling.toml`,
  same schema minus `[env]`) sets the **minimum** action per tier. User
  and project layers may only tighten. A layer that attempts to relax
  below the ceiling is a hard error naming the offending file and key —
  the analog's `requirements.toml` idea, made fail-loud.

### D2 — Deterministic layered resolution

Four layers, highest precedence first:

1. CLI flags (`--tier high=block`, `--env ci`)
2. Project `./caro-policy.toml` — **only if** the user config sets
   `policy.allow_project = true` (caro has no project-trust concept;
   default-off is the safe analog of Codex's trusted-project gating)
3. User `~/.config/caro/policy.toml`
4. System `/etc/caro/policy.toml`

Resolution is a pure function: `resolve(layers, ceiling, env) ->
ResolvedPolicy`. Field-level merge (sparse layers override per tier, not
per file). Every resolved field records **which layer and file** supplied
it. No policy file present ⇒ `ResolvedPolicy` is generated from
`SuggestedRouting::from_risk_and_safety(risk, safety_level)` — existing
behavior, bit-identical, provenance `builtin`.

### D3 — New types (all in existing modules; serializable from day one)

`src/config/policy.rs` (new file inside the existing `config` module — no
new top-level module):

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationPolicy {
    pub schema_version: u32,               // must be 1
    pub name: Option<String>,
    pub tiers: BTreeMap<RiskLevel, SuggestedRouting>,   // sparse OK
    #[serde(default)]
    pub env: BTreeMap<String, BTreeMap<RiskLevel, SuggestedRouting>>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicySource { Builtin, System, User, Project, Flag, Ceiling }

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ResolvedTier {
    pub action: SuggestedRouting,
    pub source: PolicySource,
    pub path: Option<PathBuf>,             // None for Builtin/Flag
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ResolvedPolicy {
    pub schema_version: u32,
    pub name: Option<String>,
    pub env: Option<String>,
    pub tiers: BTreeMap<RiskLevel, ResolvedTier>,       // always all 4
    pub digest: String,        // sha256 over canonical JSON of the 4 actions + env + schema_version
}
```

Method contracts: `AuthorizationPolicy::from_toml_str(&str) ->
Result<Self, PolicyError>` (parse + monotonicity + version check);
`ResolvedPolicy::resolve(layers: &[(PolicySource, AuthorizationPolicy)],
ceiling: Option<&AuthorizationPolicy>, env: Option<&str>, fallback:
SafetyLevel) -> Result<Self, PolicyError>`; `ResolvedPolicy::action_for
(RiskLevel) -> SuggestedRouting` (total — never panics, all four tiers
always present). `PolicyError` uses `thiserror`, is serializable, and
carries `{kind, path, key}` for the JSON error envelope.

`src/models/mod.rs` gains only: `FromStr`/serde aliases for
`SuggestedRouting` (`auto`/`log`/`approve`/`block`) and a
`SuggestedRouting::strictness() -> u8` used by the monotonicity/ceiling
checks. `SuggestedRouting` additionally derives `JsonSchema`, `Ord`.

### D4 — CLI surface and audit binding

- `caro policy resolve [--env <e>] [--json]` — prints the
  `ResolvedPolicy` including per-tier provenance and digest. This is the
  explain command whose absence makes analog symptoms 1–3 undiagnosable.
- `caro policy check <file>` — validates a policy file standalone
  (CI-friendly; exit 0 or 11, JSON error report on stdout with `--json`).
- The decision branch that ADR-020 wires reads
  `resolved.action_for(risk)` instead of calling
  `from_risk_and_safety` directly.
- `policy: { digest, name, env, source }` is added (additive, allowed
  within schema_version 1) to: the ADR-024 headless envelope, ADR-032
  receipts, ADR-037 events, and the ADR-038 MCP assessment payload. A
  SIEM can now detect declared-vs-effective drift mechanically — the
  by-design fix for analog symptom 4.

### D5 — Exit code 11 (`PolicyInvalid`), fail-closed

| Exit | Meaning | `status` |
| --- | --- | --- |
| 11 | policy file invalid: unknown key, bad version, non-monotonic tiers, unknown `--env`, ceiling violation, or unreadable file at an explicitly configured path | `policy_invalid` |

Reserved exclusively for policy resolution. **An invalid policy never
silently degrades to defaults** — the run refuses to start. (Absent
files at *default* paths are not errors; they simply contribute nothing.)
Exit codes 0–10 keep their frozen meanings.

### Minimal file set

| File | Change |
| --- | --- |
| `src/config/policy.rs` | **New**: types above + resolver + digest (~350 LOC) |
| `src/config/mod.rs` | Load layers, expose `resolved_policy()`; register keys in `ConfigSchema` |
| `src/models/mod.rs` | `SuggestedRouting` aliases, `strictness()`, extra derives |
| `src/cli/mod.rs` | `policy resolve|check` subcommand; swap decision branch to `action_for` |
| `src/main.rs` | Exit 11 mapping |
| `docs/headless-contract.md` | Envelope `policy` block + exit 11 |
| `tests/policy_contract.rs` | **New**: integration tests below |

No new dependencies (`sha2`, `toml`, `serde`, `schemars`, `thiserror`
are already in-tree). Pure subprocess: resolution happens per invocation
from files + flags; no daemon, no state, no network.

### Integration tests (deterministic input → fixed JSON + exit code)

1. **Default-compat golden test**: no policy files ⇒ for all 12
   (risk × safety) combinations, `action_for` equals
   `from_risk_and_safety` exactly; envelope carries
   `policy.source: "builtin"`. Guarantees zero behavior change for
   existing users.
2. Valid user policy ⇒ exit 0; `caro policy resolve --json` output
   byte-stable across runs (fixed digest for fixed fixture).
3. Unknown key `tires` ⇒ exit 11, `error.kind: "unknown_key"`,
   offending path+key named.
4. Non-monotonic (`high = "auto"`, `critical = "approve"` with
   `moderate = "approve"`) ⇒ exit 11, `error.kind: "non_monotonic"`.
5. Ceiling `high = "approve"` + user `high = "log"` ⇒ exit 11,
   `error.kind: "ceiling_violation"`, both files named.
6. `--env ci` selects override; `--env staging` (undeclared) ⇒ exit 11.
7. Project file present but `allow_project` unset ⇒ ignored, provenance
   shows `user`; with `allow_project = true` ⇒ provenance `project`.
8. Layer shadowing (analog issue #11885's shape): system `high=block`,
   user `high=approve`, flag `--tier high=block` ⇒ effective `block`
   with `source: "flag"` — divergence visible, not silent.

## Consequences

**Positive.** Caro's tier decisions become declarative, per-environment,
org-enforceable, and provably attached to every assessment artifact —
the "show me your tiers" answer the 2026-07 regulatory wave demands,
delivered with determinism the analog demonstrably lacks. The canonical
mapping stays the zero-config default, so nothing changes for current
users.

**Negative / accepted.** A fifth config surface (policy file) beside
`config.toml`, CaroML, hooks, and flags — mitigated by `caro policy
resolve` being the single source of truth for "what will happen".
Fail-closed exit 11 means a typo stops automation until fixed; that is
the point, and `caro policy check` in CI catches it pre-deploy.
`ApprovalMode::Smart` interplay is intentionally conservative (below).

**Interaction rules.** The resolved policy binds *after* static risk
assessment and *before* ADR-035 hooks (hooks may only tighten, mirroring
the ceiling rule). `ApprovalMode::Auto` may downgrade `approve`→`log`
only for tiers the policy left at builtin default; an explicit policy
action is never relaxed by a mode knob. ADR-039's sandbox placement
remains an orthogonal axis carried alongside, not inside, the tier map.

## Alternatives considered

1. **Extend `config.toml` instead of a separate file.** Rejected: the
   policy needs independent review/ownership (security team, not the
   user), a ceiling relationship, and per-repo distribution — different
   lifecycle from personal preferences.
2. **Adopt Codex's `approval_policy` vocabulary verbatim.** Rejected:
   `untrusted`/`on-request`/`never` conflates *when to ask* with *what
   to do* and is not expressible per risk tier; risk→action is the frame
   the regulatory texts use.
3. **Dynamic policy via ADR-035 hooks only.** Rejected as the primary
   mechanism: hooks are Turing-complete and unauditable in the general
   case; a declarative file is diffable, hashable, and reviewable. Hooks
   remain the escape hatch for context-dependent decisions.
4. **Per-pattern allow/deny lists in v1** (Claude Code
   `permissions.allow/deny` style). Deferred: opens an open-ended
   matching space with its own false-positive discipline; the closed
   tier map delivers the compliance story at a fraction of the surface.
5. **Signed policy files.** Deferred to the ADR-032 signing phase; the
   digest field is forward-compatible with a detached signature.

## Out of scope (next version)

- Per-pattern overrides (`[overrides."pattern-id"]`) and path-scoped
  rules
- Remote/centrally fetched policy distribution
- Policy signing and verification (rides ADR-032's signing work)
- A `caro policy init` interactive generator
- Windows system-layer path conventions (`ProgramData`) — tracked, not
  designed here
- Mapping caro tiers to external taxonomies (China tiered-authorization
  classes, x402 roles) — a documentation exercise once the file exists

## References

- Codex config precedence, profiles, `requirements.toml` ceiling:
  [developers.openai.com/codex/config-basic](https://developers.openai.com/codex/config-basic)
- Analog failure modes:
  [openai/codex#11885](https://github.com/openai/codex/issues/11885),
  [#3034](https://github.com/openai/codex/issues/3034),
  [#6667](https://github.com/openai/codex/issues/6667),
  [#5038](https://github.com/openai/codex/issues/5038),
  [#21982](https://github.com/openai/codex/issues/21982),
  [#20720](https://github.com/openai/codex/issues/20720)
- Hermes market scan 2026-07-27 (`.hermes/digests/`), opportunity 2 and
  §2 "Tiered authorization is becoming a compliance requirement"
- `.claude/rules/validation-discipline.md`, ADR-020, ADR-024, ADR-035,
  ADR-037, ADR-039
