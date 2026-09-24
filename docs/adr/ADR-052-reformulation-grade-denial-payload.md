# ADR-052: Reformulation-Grade Denial Payload — Stable Rule Identity and Machine-Readable Remediation

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-17
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: **Execlave** `enforcePolicy()` — synchronous in-path
  policy enforcement for AI agents (commercial; Product Hunt 2026-08-13;
  19–20 policy types; four enforcement modes; violations array on denial).
  Researched live 2026-08-17 against
  [docs](https://www.execlave.com/docs),
  [Policies & Security](https://www.execlave.com/docs/policies), and the
  [SDK Reference](https://www.execlave.com/docs/sdk-reference).
- **Depends on**: ADR-024 (headless envelope, exit-code registry, additive-field
  rule), ADR-020 (`SuggestedRouting` vocabulary)
- **Relates to**: ADR-040 (tiered authorization policy file — this ADR's
  `rule_id` is the precondition for per-rule overrides), ADR-046 (approval
  exchange — `violations` enriches the approval payload), ADR-034 (degraded
  result contract — same stabilization-boundary pattern), ADR-029 (schema
  enforcement — same "frozen vocabulary + golden test" discipline), ADR-037 /
  ADR-041 (lifecycle events — future sink for `Violation`), ADR-036 / ADR-043 /
  ADR-048 (guard, MCP-proxy and hook adapters — inherit the field via
  `SafetyDecision`)
- **Companion scope document**: `caro-scope-denial-payload-2026-08-17.md`
- **Numbering note**: highest existing is ADR-051; per
  `.claude/rules/adr-numbering.md`, renumber on merge if another 052 lands first.

> **Provenance note (autonomous run).** Produced with no user present; the task
> template's `[FEATURE NAME]` was unbound. Target selection: today's market scan
> (`market-scans/2026-08-17-ai-agent-strategy-memo.md`) ranks the
> reformulation-grade denial payload as opportunity A and recommendation #2, and
> names Execlave as the vendor that shipped the reference shape this week.
> Recommendation #1 (a deterministic-vs-classifier eval) produces no ADR and is
> not superseded by this document — it should be run separately. Coverage check:
> no existing ADR gives a fired safety rule a stable identifier. Treat the analog
> choice and the `RemediationClass` taxonomy as reviewable assumptions. Per
> `.claude/rules/git-workflow.md` this file is left uncommitted for a human to
> branch and PR.

---

## 1. Context

### 1.1 What changed in the market

Four independent vendors shipped enforcement layers below the agent harness in
the week of Aug 10–16, 2026. The one that matters for this decision is Execlave,
because it defined a shape: **a denial is a structured list of violations, not a
refusal string**. Its `PolicyBlockedError` carries `violations[]` of
`{policyType, policyName, severity, message, enforcementMode}` so the calling
agent can branch, reformulate, and resubmit. Kane CLI (machine-readable verdicts
consumed in-loop by a coding agent) and Ito (a mandatory provenance field on
every finding) converged on the same principle from different directions.

The bar has moved from *"deny with a reason"* to *"deny with enough structure
that the agent can act on it."*

### 1.2 What Caro emits today

A block produces `blocked_reason: Option<String>` on `CliResult` and
`matched_patterns: Vec<String>` on `ValidationResult` / `SafetyDecision` — a list
of **human-readable descriptions**. There is no identifier for the rule that
fired. `DangerPattern` (`src/safety/mod.rs:334`) has no id field. Worse,
`src/safety/cve_patterns.rs` receives CVE rules that *do* carry a machine
identifier and flattens it into prose:

```rust
let description = format!("{}: {}", p.id, p.description);
```

Identity exists upstream and is destroyed at the validator boundary. Any consumer
branching on the current output is string-matching prose we are free to reword in
a patch release.

The practical consequence, in the words of the market scan: *"Caro blocks a
command and the calling agent dead-ends or retries blindly. A block is currently
a verdict, not usable information."*

### 1.3 The failure modes in the analog, which we must not inherit

1. **The deny path is a different channel from the allow path.** Execlave's own
   docs shout that `enforcePolicy()` *throws* rather than returning
   `{allowed:false}`. The reformulation payload rides on an exception. The
   outcome taxonomy consequently grew into **13+ error classes** because the
   value type never had room for it.
2. **A violation has no stable identity.** `policyName` is operator-chosen
   display text; `policyType` is one of ~20 coarse classes. A dashboard rename
   silently breaks consumers.
3. **There is no remediation channel, and no way to say "stop retrying."** An
   agent facing a structurally unsafe request and an agent facing an over-broad
   target receive identically shaped answers, so both retry.
4. **Fail-open is the default, twice** — per-policy `failureMode` and the client's
   `enforcementOnOutage`. The SDK ships an `onEnforcementBypassed` callback whose
   job is to report that an action ran ungoverned.
5. **The gate is opt-in and bypassable by omission** — the exact property
   Cloudflare cited when it said client-side controls cannot be trusted.

### 1.4 Forces

- Caro's differentiation after Aug 14 is **deterministic, offline, zero-latency,
  reproducible**. A payload asserted to be byte-stable is that claim made
  testable; a payload keyed on prose is that claim given away.
- ADR-040 (per-tier policy), ADR-041 (lifecycle events), ADR-046 (approval
  exchange) and the guard adapters all want to name a specific rule. None can
  until a rule has a name.
- The additive-field rule from ADR-024 means this must not break existing
  consumers of `matched_patterns` or existing user `config.toml` files.

---

## 2. Decision

Introduce a **stable rule-identity layer** in `src/safety/` and surface it as an
always-present, deterministically-ordered `violations[]` on the JSON result, with
each violation carrying a remediation class and an explicit `retryable` flag.
Human-readable output is unchanged.

Six design commitments, each answering one Phase-1 failure mode by construction:

### D1 — One channel, always present

`violations` is serialized on **every** outcome — success, blocked, refused,
error — and is `[]` when nothing fired. It is never `null` and never omitted (no
`skip_serializing_if`). There is no separate error shape for a denial;
`jq '.violations | length'` is a total function. *(Answers failure mode 1.)*

### D2 — `RuleId` is a frozen, machine-readable vocabulary

Format `caro:<source>:<slug>` — `caro:builtin:rm-recursive-root`,
`caro:cve:CVE-2024-3094`, `caro:user:7f3a91c2`,
`caro:internal:validator-unavailable`. Built-ins get compile-time constant ids;
CVE rules carry `p.id` through instead of flattening it into the description
(fixing §1.2); user patterns derive `caro:user:` + the first 8 hex of
`sha256(pattern_source)` so the same regex yields the same id on every machine.
`message` and `remediation.hint` are documented as **explicitly unstable**.
A golden file pins every built-in id: adding one is additive, renaming or removing
one fails the suite and forces a schema-version conversation. *(Answers failure
mode 2.)*

### D3 — `Remediation` with a first-class `retryable` flag

```
ScopeTarget | DropFlag | RequireApproval | UseAlternative | NoReformulation
```

plus `offending_tokens` (literal tokens taken from the regex match span) and
`retryable: bool`, which is always `false` for `NoReformulation` and
`RequireApproval`. `retryable == false` is a contract: the caller must not
resubmit a reformulation of the same request. This is the anti-thrash signal the
analog has no field for, and it is only expressible because Caro's rules are
about POSIX commands rather than arbitrary prompts. *(Answers failure mode 3.)*

### D4 — Fail-closed by construction

There is no `fail_open` mode and no configuration knob for one. A validator
failure — pattern set unloadable, regex evaluation error — produces a synthetic
`caro:internal:validator-unavailable` violation with `action = Block`,
`retryable = false`, and exit `4`. `Violation::validator_unavailable()` is the
only representation of that state, so no code path can return "no violations" on
error. *(Answers failure mode 4.)*

### D5 — The payload and the exit code have one source

`ValidationResult::top_action()` is the sole producer of both the block decision
and the exit code, preserving ADR-024's invariant that the terminal payload and
the process exit can never disagree. `exit == 3` **⟺** some violation has
`action == "block"`. There is no ungated path through the binary. *(Answers
failure mode 5.)*

### D6 — Total ordering, not partial

Execlave combines by severity independent of evaluation order — correct, and we
adopt it. We strengthen it to a **total** order (`risk_level` desc, then
`RuleSource` `Builtin < Cve < User < Allowlist < Internal`, then `rule_id`
lexicographic) so the serialized array is byte-stable and full-JSON equality is a
valid test assertion. Reproducibility is the product claim; the ordering is what
makes it enforceable.

### Non-goals of this decision

No new action vocabulary (`SuggestedRouting` is reused verbatim), no new module,
no new dependency, no new exit code, no change to plain or YAML output, and no
change to when a command is blocked. This ADR changes what Caro *says* about a
decision, not what it decides.

---

## 3. Consequences

### Positive

- A blocked agent learns *which* rule fired, *where* in the command, *what class*
  of alternative would pass, and *whether to try at all* — the difference between
  reusable infrastructure and a speed bump users disable.
- `rule_id` unblocks four already-scoped ADRs that need to name a rule: per-rule
  policy overrides (ADR-040), violation events on the lifecycle bus (ADR-041),
  evidence in the approval payload (ADR-046), and richer verdicts in the guard
  and MCP-proxy adapters (ADR-036 / ADR-043 / ADR-048). All inherit the field
  through `SafetyDecision` without further work.
- Byte-stable output makes "deterministic, reproducible, offline" a test
  assertion rather than marketing copy — precisely the ground a hosted LLM
  classifier cannot contest.
- The existing CVE-identity flattening bug is fixed as a side effect.

### Negative / costs

- **A frozen id vocabulary is a permanent maintenance commitment.** 67 built-in
  patterns each acquire a hand-assigned id and remediation class, and renaming
  one becomes a breaking change. Mitigated by the golden file making violations
  mechanical to catch, and it is the same discipline ADR-024 and ADR-029 already
  accepted.
- **The bulk of the implementation diff is a mechanical table edit** across
  `patterns.rs`. Low risk, tedious review; the golden test is what makes it
  reviewable.
- **Two fields now describe the same thing** (`matched_patterns` and
  `violations`, `blocked_reason` and `violations`) until a major version removes
  the older ones. Accepted deliberately: the additive-field rule forbids breaking
  existing consumers, and back-compat is asserted by test.
- **`RemediationClass` assignment is a judgement call per pattern.** The demoware
  trap is a large imported CVE ruleset defaulting to `RequireApproval` /
  `retryable: false` and quietly making Caro over-conservative. Instrumented via
  a `caro doctor` default-class count; the escape hatch is moving class assignment
  into the Dogma rule schema, an additive change to `src/dogma/`.
- **Payload size grows** on commands that match many patterns. Bounded by the
  pattern count and irrelevant to the plain-text path.

### Neutral

- No latency change: patterns are already `once_cell::Lazy` statics and the match
  loop already computes everything a `Violation` needs. Construction is the only
  added cost, on a path that already allocates description strings.
- Interactive users see no difference. This is a machine-surface change.

---

## 4. Alternatives considered

**A. Do nothing; let consumers parse `matched_patterns` prose.** Rejected. It is
the analog's failure mode 2 with extra steps, it makes every description edit a
silent breaking change, and it forecloses ADR-040's per-rule overrides.

**B. Mirror the analog exactly — `{policyType, policyName, severity, message,
enforcementMode}`.** Rejected. It reproduces the identity gap (a "policy name" is
display text) and omits remediation entirely, which is the only part of the
payload an agent can actually act on. Copying the shape without the schema fix
would mean shipping their bug on purpose.

**C. Emit a concrete safer command instead of a remediation class.** Rejected for
v1. Generating an alternative requires the model, making the payload
non-deterministic and network- or latency-bearing — surrendering the exact
property that makes Caro worth layering under a classifier. Deferred; the class
is the deterministic subset.

**D. A `caro check "<cmd>"` validate-only subcommand as the delivery vehicle.**
Deferred, not rejected. It is genuinely valuable (pure subprocess, no model
load), but it is a new user-facing capability class, so
`.claude/rules/validation-discipline.md` applies — 20 transcripts, demoware-trap
section, devil's-advocate review before an implementation PR. Extending the
existing `-o json` envelope is exempt on those grounds and delivers the payload
now; the subcommand can consume it later.

**E. Sequential integer rule ids (`caro:builtin:0042`).** Rejected. Insertion and
reordering churn the ids, and the identifier carries no meaning for a human
reading a log. Slugs are stable under reordering and self-documenting.

**F. Per-rule enforcement mode on the violation (the analog's fourth field).**
Rejected as redundant. ADR-040 already owns risk→action mapping with a
monotonicity invariant; adding a second, per-rule mode would create two sources
of truth for one decision. `Violation.action` reports the *resolved* routing from
the existing vocabulary.

**G. A `fail_open` configuration knob for parity.** Rejected on strategy. A
locally-evaluated validator has no outage class to fail open *for*; the only
thing the knob could buy is a way to turn the safety layer off silently, which is
the property the market spent the week arguing against.

---

## 5. References

- Companion scope: `caro-scope-denial-payload-2026-08-17.md` (Phase 1–3, new type
  definitions, file list, nine integration tests, out-of-scope list)
- Market scan: `market-scans/2026-08-17-ai-agent-strategy-memo.md` — opportunity
  A, recommendation #2
- [Execlave — Policies & Security](https://www.execlave.com/docs/policies)
- [Execlave — SDK Reference](https://www.execlave.com/docs/sdk-reference)
- [Execlave — Product Hunt launch](https://www.producthunt.com/products/execlave)
- [Cloudflare — WriteGuard: fine-grained controls for MCP servers](https://blog.cloudflare.com/mcp-portal-writeguard-private-beta/)
- Code under change: `src/safety/mod.rs` (`ValidationResult:175`,
  `SafetyDecision:189`, `DangerPattern:334`), `src/safety/patterns.rs`
  (67 entries), `src/safety/cve_patterns.rs` (id flattening),
  `src/cli/mod.rs` (`CliResult`)
