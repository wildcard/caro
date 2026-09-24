# Implementation Scope — Reformulation-Grade Denial Payload

**Feature under analysis:** **Execlave** `enforcePolicy()` / `enforce_policy()`
— synchronous, in-path pre-execution policy enforcement for AI agents
(commercial, launched on Product Hunt 2026-08-13, $199/mo, self-hostable).
Docs fetched live 2026-08-17:
[docs index](https://www.execlave.com/docs) ·
[Policies & Security](https://www.execlave.com/docs/policies) ·
[SDK Reference](https://www.execlave.com/docs/sdk-reference).

**Equivalent we are scoping for Caro:** a **typed, stable-identity `violations[]`
array** on the existing `-o json` result, carrying per-rule identity, the
already-canonical action vocabulary, the matched span, and — the part nobody
ships — a **remediation class with a machine-readable `retryable` flag**, so a
calling agent can either reformulate correctly or stop trying.

**Date:** 2026-08-17 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-052-reformulation-grade-denial-payload.md`

> **Provenance note (autonomous run).** Produced with no user present; the task
> template's `[FEATURE NAME]` was unbound. Target selection: today's market scan
> (`market-scans/2026-08-17-ai-agent-strategy-memo.md`, produced 02:17 today)
> ranks **"reformulation-grade denial payload" as opportunity A / recommendation
> #2**, names Execlave as the vendor that shipped the reference shape this week,
> and gives the next step verbatim — *"extend the envelope with `violations[]`
> (`pattern_id`, `risk_level`, `enforcement_mode`, `message`) behind the existing
> `-o json` surface."* Recommendation #1 (the deterministic-vs-classifier eval) is
> an eval run plus a docs page — it produces no ADR, no types, and no exit-code
> contract, so it is not a fit for this template; it should be run separately and
> is *not* superseded by this document. Coverage check: ADR-024/029/034 define the
> envelope, schema enforcement, and degradation provenance; ADR-040 defines the
> risk→action tier map; ADR-046 defines the approval payload. **None of them give
> a fired rule a stable identifier**, which is the precondition for all of them.
> Treat the analog choice and the remediation taxonomy (D3) as reviewable
> assumptions. Per `.claude/rules/git-workflow.md` this file is left uncommitted
> for a human to branch and PR.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom

Execlave sells "the gate between your AI agents and the real world": a
synchronous authorization check that runs *before* an agent's LLM call or tool
invocation, evaluated against 19–20 policy types (injection scanning, tool
access control, PII, cost caps, external-call allowlists, OPA Rego bundles, CEL
expressions, agent lineage). The buyer is a platform or security team that has
agents in production and no authorization layer. The distinguishing product
claim is that a denial is **not a refusal string** — it is a structured list of
violations the calling code can branch on.

### Core architecture — data flow, key types, separation of concerns

- **Data flow:** app calls `exe.enforcePolicy({agentId, input, tools?,
  estimatedCost?, metadata?})` → HTTPS round trip to the policy service →
  policies matching the agent are evaluated → a combined decision returns, or an
  error is raised. Tool results get a second gate, `enforceToolOutput()`, before
  they are fed back to the model.
- **Key types:** a policy is `{name, policyType, enforcementMode,
  ruleDefinition, appliesToAgents, isActive, failureMode}`. A violation is
  `{policyType, policyName, severity, message, enforcementMode}`. Four
  enforcement modes: `monitor` (record only), `warn` (record + notify),
  `require_approval` (park in a human queue), `block` (reject).
- **Separation of concerns:** *policy authoring* (dashboard, JSON/YAML bundles,
  `execlave policies lint`) is separate from *evaluation* (server-side, with
  local Rego-to-WASM and local groundedness scoring as zero-egress exceptions),
  which is separate from *enforcement* (the SDK call site, which must exist in
  the integrator's code).
- **Combination rule — the part they get right:** when several policies fire,
  the decision is combined **deterministically by severity**, `block >
  require_approval > warn > monitor`, *independent of evaluation order*. Stated
  plainly in the docs.

### Why it is limited — the failure modes

1. **The denial travels on a different channel than the allow.** The docs shout
   this: *"`enforcePolicy()` THROWS `PolicyBlockedError` when a block-mode policy
   fires — it does not return `{ allowed: false }`. You must catch the error."*
   The reformulation payload (`err.violations`) rides on an exception object, so
   the allow path and the deny path have different shapes, and a caller who
   forgets the `catch` gets a crash instead of a decision. There are now **13+
   distinct error classes** (`PolicyBlockedError`, `PolicyDeniedError`,
   `ValidatorDeniedError`, `ToolIntegrityError`, `CertificateMismatchError`,
   `EnforcementHaltError`, …) — an outcome taxonomy that grew in the *error*
   hierarchy because the *value* type never had room for it.
2. **A violation has no stable identity.** `policyName` is operator-chosen
   display text ("Block Prompt Injection") and `policyType` is one of ~20 coarse
   classes. Neither identifies *which rule* fired. An agent can learn it tripped
   `injection_scan`; it cannot learn which of the tenant's injection rules, and a
   dashboard rename silently breaks any consumer that matched on the name.
3. **No remediation channel at all.** `message` is prose. There is no field
   saying *what class of alternative would pass* and — critically — no field
   saying **this cannot be reformulated, stop retrying**. An agent facing a
   structurally-unsafe request and an agent facing an over-broad target get the
   same shaped answer, so both retry.
4. **Fail-open is the default, twice.** Per-policy `failureMode` defaults to
   `fail_open` (detector or DB unavailable ⇒ policy skipped), and the client's
   `enforcementOnOutage` defaults to `'fail_open'`. The SDK ships an
   `onEnforcementBypassed` callback whose entire job is to tell you an action ran
   **ungoverned**. The secure posture exists (`fail_closed`) but is opt-in.
5. **The gate is opt-in and bypassable by omission.** *"Tracing alone does NOT
   block — this method is the gate."* Enforcement happens only where an
   integrator remembered to call it; `enforceToolOutput` is explicitly *"not
   called for you by the framework adapters."* This is precisely the property
   Cloudflare cited in WriteGuard's launch as the reason client-side controls
   cannot be trusted.
6. **Every decision is a network round trip.** Mitigated by a 60 s
   `policyCacheTtlMs` decision cache — which trades the freshness of the policy
   for latency, and adds a staleness window to a security control.

### Structured output contract (payloads, exit codes, events)

Value-shaped: `EnforcementDecision` on allow; `violations[]` on the thrown error
on block; `202` + `approvalId` for the `require_approval` path, polled and later
verified by `verifyApproval(approvalId, actionContext)` against a
context-bound certificate (replay and swap are rejected — a good pattern, and
the same one ADR-046 lands for Caro). Event-shaped: 16 webhook event types
(`policy.violated`, `agent.paused`, `cost.threshold`, …), HMAC-SHA256 signed with
exponential-backoff retry, plus OTLP export and Splunk/Sentinel routing. There
are **no exit codes** — it is a library, not a process; the CLI exists for
policy authoring, not enforcement.

### Session / context lifecycle — how redundant initialization is avoided

A single long-lived `Execlave` client per process holds connection pooling, a
trace batch buffer (`batchSize`, `flushIntervalMs`), a 15 s kill-switch control
channel poll, in-process agent-credential caching, and the policy-decision
cache. Initialization is amortized by **keeping the process alive** — a daemon
model. That is the correct answer for a long-running service and the wrong
answer for a CLI: none of it survives a subprocess boundary, and each of those
caches is a staleness or bypass window.

---

## Phase 2 — Competitive Differentiation

### What they get right, and we should replicate

- **A violation array, not a reason string.** The unit of denial is a record per
  fired rule, not one flattened sentence.
- **Deterministic severity combination independent of evaluation order.**
  Adopt, and strengthen to a *total* order so the JSON is byte-stable.
- **A `monitor` tier that evaluates without enforcing.** Caro already has the
  equivalent in `SuggestedRouting::AsyncLog`; it is currently invisible in the
  payload because nothing surfaces the rule that caused it.
- **An approval object bound to the action context** so a granted approval
  cannot be replayed against a different action (already Caro's ADR-046).

### Their design gaps we avoid by designing the schema first

| Their gap | Our design answer |
|---|---|
| Deny path is an exception; allow path is a value | `violations` is on **every** result, always present, `[]` when empty. One channel, one shape. |
| `policyName` is renameable display text | `RuleId` is a frozen, machine-readable vocabulary with a golden-file test; `message` stays explicitly unstable. |
| No remediation, no "stop retrying" signal | `Remediation { class, hint, offending_tokens, retryable }` — `retryable: false` is the anti-thrash contract. |
| `fail_open` default × 2 | Fail-closed by construction: validator failure emits a synthetic `caro:internal:validator-unavailable` violation at `block`, never a silent allow. |
| Gate runs only if the integrator calls it | Same code path that emits the envelope sets the exit code (ADR-024 invariant). There is no un-gated path through `caro`. |
| 60 s decision cache staleness | No cache: patterns are `once_cell::Lazy` statics compiled in-process, decision is pure and local. |
| Identity destroyed at the boundary | Fix a live instance of exactly this bug in our own code — see below. |

**Our own instance of gap #2, already in the tree.** `src/safety/cve_patterns.rs`
receives a CVE rule that *has* an id (`p.id`) and then does
`format!("{}: {}", p.id, p.description)` — flattening a machine-readable
identifier into a human-readable string at the validator boundary, which callers
would have to re-parse. `DangerPattern` (`src/safety/mod.rs:334`) has no id field
at all, and `ValidationResult.matched_patterns` /
`SafetyDecision.matched_patterns` are `Vec<String>` of descriptions. Any consumer
branching on those is string-matching prose that we are free to reword in any
patch release. This scope fixes the identity layer first; the payload is a
consequence of it.

### Our unique positioning — what they structurally cannot do

- **Zero-latency, zero-egress, no cache.** No HTTPS round trip means no
  `policyCacheTtlMs` tradeoff and no `enforcementOnOutage` question, because
  there is no outage class. The market scan's point 4 is that latency and
  non-bypassability decide adoption and nobody is arguing it; this payload is
  what makes the argument concrete.
- **Deterministic and reproducible.** The same command, shell, and safety level
  produce byte-identical `violations` on any machine, offline, forever — testable
  by full-JSON equality, which a hosted or LLM-adjudicated gate cannot offer.
- **Span-level evidence.** Regex match offsets into the command give
  `offending_tokens` for free. Execlave evaluates opaque `input` strings and has
  nothing to point at.
- **A remediation taxonomy grounded in shell semantics.** "Scope the target",
  "drop this flag", "no safe reformulation exists" are meaningful because the
  rules are about POSIX commands, not arbitrary prompts. This is the payoff of
  being a specialist, and it does not generalize to their surface.

### Existing infrastructure that already covers part of this

| Capability | Exists today | Location |
|---|---|---|
| 67 built-in `DangerPattern` entries, pre-compiled | ✅ | `src/safety/patterns.rs` |
| CVE ruleset with **real ids**, bincode-embedded | ✅ (id discarded) | `src/safety/cve_patterns.rs`, `src/dogma/` |
| `ValidationResult` / `SafetyDecision`, both `Serialize` | ✅ | `src/safety/mod.rs:175,189` |
| Canonical action vocabulary `SuggestedRouting` | ✅ | `src/models/mod.rs:189` |
| Risk→action tier map, policy file, monotonicity | ✅ scoped | ADR-040 |
| Envelope + exit-code registry (0/1/2/3/4) | ✅ scoped | ADR-024 v2 |
| User-pattern hardening (length, `Critical` reserved) | ✅ | `src/safety/mod.rs::validate_user_pattern` |
| `-o json` output surface | ✅ | `src/cli/mod.rs` (`OutputFormat`, `CliResult`) |

Nothing here needs a new module, a new dependency, or a new action word.

---

## Phase 3 — Scope Definition

### Decision summary

Add a stable rule-identity layer to the safety module and surface it as an
always-present, deterministically-ordered `violations[]` on the JSON result,
each violation carrying a remediation class and a `retryable` flag. Human output
is unchanged. Full rationale, consequences, and alternatives:
`docs/adr/ADR-052-reformulation-grade-denial-payload.md`.

### New types (all in `src/safety/mod.rs`, all `Serialize + Deserialize + JsonSchema`)

```rust
/// Stable, machine-readable identity for one safety rule.
/// Format: `caro:<source>:<slug>` — frozen vocabulary, additive-only.
/// Examples: `caro:builtin:rm-recursive-root`, `caro:cve:CVE-2024-3094`,
/// `caro:user:7f3a91c2`, `caro:internal:validator-unavailable`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct RuleId(String);

impl RuleId {
    pub const fn builtin(slug: &'static str) -> Self;      // compile-time, no alloc in the table
    pub fn cve(cve_id: &str) -> Self;                      // from dogma rule id, not the description
    pub fn user(pattern_src: &str) -> Self;                // `caro:user:` + first 8 hex of sha256(src)
    pub fn source(&self) -> RuleSource;                    // parsed from the second segment
    pub fn as_str(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuleSource { Builtin, Cve, User, Allowlist, Internal }

/// Byte offsets of the match within the validated command. Deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MatchSpan { pub start: usize, pub end: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RemediationClass {
    /// The verb is fine; the target is too broad. Name an explicit path.
    ScopeTarget,
    /// A specific flag caused the match. Removing it may pass.
    DropFlag,
    /// Nothing to change — this needs a human decision.
    RequireApproval,
    /// A safer command achieves the same intent.
    UseAlternative,
    /// Structurally unsafe. No reformulation exists. Do not retry.
    NoReformulation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Remediation {
    pub class: RemediationClass,
    /// Human-readable. NOT part of the stable contract.
    pub hint: String,
    /// Literal tokens from the command that triggered the rule, in order.
    pub offending_tokens: Vec<String>,
    /// `false` ⇒ the caller MUST NOT retry a reformulation of this request.
    /// Always `false` for `NoReformulation`; always `false` for `RequireApproval`.
    pub retryable: bool,
}

/// One fired rule. The unit of denial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Violation {
    pub rule_id: RuleId,
    pub rule_source: RuleSource,
    pub risk_level: RiskLevel,          // reused, unchanged
    pub action: SuggestedRouting,       // reused, unchanged — no new action vocabulary
    /// Human-readable. NOT part of the stable contract; may change in any release.
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<MatchSpan>,
    pub remediation: Remediation,
}
```

**Method contracts.**

- `Violation::from_match(pattern: &CompiledRule, m: &regex::Match, safety: SafetyLevel) -> Self`
  — the single construction path; `action` is always
  `SuggestedRouting::from_risk_and_safety(...)` (ADR-040 will later swap this for
  `ResolvedPolicy::action_for`, a one-line change confined to this constructor).
- `Violation::validator_unavailable(detail: &str) -> Self` — the fail-closed
  synthetic: `rule_id = caro:internal:validator-unavailable`,
  `action = Block`, `retryable = false`. The only way to represent a validator
  failure; there is no code path that returns "no violations" on error.
- `ValidationResult::violations_sorted(&self) -> &[Violation]` — total order:
  `risk_level` desc, then `RuleSource` (`Builtin < Cve < User < Allowlist <
  Internal`), then `rule_id` lexicographic. Byte-stable output; strictly stronger
  than the analog's partial order.
- `ValidationResult::top_action(&self) -> SuggestedRouting` — max action over
  violations, `AutoApprove` when empty. The **only** source of the block decision,
  so the exit code and the payload cannot disagree.
- `ValidationResult::is_retryable(&self) -> bool` — `violations.iter().all(|v|
  v.remediation.retryable)`; `true` when empty.

**Changed existing types (additive only, per the ADR-024 rule).**

- `DangerPattern` gains `#[serde(default)] pub id: Option<RuleId>`. `None` from a
  user's `config.toml` is filled at load with `RuleId::user(&pattern.pattern)` —
  existing user configs keep deserializing untouched.
- `ValidationResult` gains `pub violations: Vec<Violation>` (never
  `skip_serializing_if`). `matched_patterns: Vec<String>` is **retained and
  populated** as `violations.iter().map(|v| v.message.clone())` and documented as
  deprecated.
- `SafetyDecision` gains the same `violations` field, populated by
  `from_validation_result`.
- `CliResult` gains `pub violations: Vec<Violation>`. `blocked_reason:
  Option<String>` is retained, documented as deprecated in favour of
  `violations`.

### Minimal set of files that change

| # | File | Change |
|---|---|---|
| 1 | `src/safety/mod.rs` | The six new types + their impls; three additive fields; `validate_command` builds `Violation`s in the existing match loop. |
| 2 | `src/safety/patterns.rs` | Add `id: RuleId::builtin("…")` and `remediation: RemediationClass::…` to all 67 entries. Mechanical; the diff is the bulk of the PR. |
| 3 | `src/safety/cve_patterns.rs` | Carry `p.id` into `RuleId::cve(&p.id)`; stop prefixing it into `description`. |
| 4 | `src/cli/mod.rs` | `CliResult.violations`; populate from the validator; plain/YAML output paths unchanged. |
| 5 | `tests/denial_payload_contract.rs` | **New** integration test file (below). |
| 6 | `tests/safety_validator_contract.rs` | Extend: `matched_patterns` back-compat assertions. |
| 7 | `docs/adr/ADR-052-…md`, `CHANGELOG.md` | The ADR and an `Added` entry. |

No new module. No new dependency (`serde`, `schemars`, `sha2`, `regex`,
`once_cell` are all present). No new action vocabulary.

### Exit-code / output contract (what machines and scripts depend on)

Exit codes are **unchanged** — this scope adds none and reuses the ADR-024
registry exactly (`0` success · `1` error · `2` arg error · `3` blocked · `4`
refused). What is added is a set of invariants a script may rely on under the
existing schema version:

1. `violations` is **always present** on JSON/NDJSON output — `[]` when nothing
   fired. Never `null`, never absent, on any outcome including success and error.
   (`jq '.violations | length'` is total.)
2. `exit == 3` **⟺** `violations` contains at least one entry with
   `action == "block"`. Both are derived from `top_action()`; they cannot
   disagree.
3. `violations` is sorted by the total order above, so byte equality is a valid
   test assertion.
4. `rule_id` values are stable across patch and minor releases. Removing or
   renaming one is a **breaking change** requiring a schema-version bump; adding
   one is additive. Enforced by the golden test (test 7).
5. `message` and `remediation.hint` are explicitly **not** stable. Consumers that
   branch on prose are unsupported.
6. `remediation.retryable == false` on any violation means the caller must not
   resubmit a reformulation of the same request. Aggregated as
   `is_retryable()`.
7. A validator failure surfaces as a `caro:internal:validator-unavailable`
   violation with `action == "block"` and exit `4`. There is no input for which
   Caro exits `0` with an unevaluated command.

### Integration tests — known input → deterministic JSON + exit code

Run with `--backend static` so generation is deterministic and offline.

1. **Critical block, full JSON equality.** `rm -rf /` ⇒ exit `3`; exactly one
   violation; `rule_id == "caro:builtin:rm-recursive-root"`, `rule_source ==
   "builtin"`, `risk_level == "critical"`, `action == "block"`,
   `remediation.class == "no_reformulation"`, `retryable == false`,
   `offending_tokens == ["-rf", "/"]`. Assert the whole `violations` array
   byte-for-byte.
2. **Empty array on the happy path.** `ls -la` ⇒ exit `0` and the serialized JSON
   **contains the key** `"violations":[]`. Regression guard for analog failure
   mode 1 (deny and allow on different channels).
3. **Deterministic multi-match ordering.** A command matching one CVE rule and
   two built-ins ⇒ the exact expected order; re-run 10× and assert byte
   equality; assert order is unchanged when the pattern table is iterated in
   reverse (guards against evaluation-order leakage).
4. **User-pattern identity is derived and stable.** A `config.toml` custom
   pattern ⇒ `rule_id == "caro:user:<8 hex>"`, identical across processes and
   platforms; a second config with the same regex under a different description
   yields the **same** id; `RuleSource::User` never carries `risk_level ==
   critical` (existing hardening invariant, re-asserted at the payload layer).
5. **CVE identity survives the boundary.** A command matching an embedded CVE
   rule ⇒ `rule_id == "caro:cve:CVE-XXXX-YYYY"` and `message` does **not** contain
   the id twice. Regression guard for the flattening bug in `cve_patterns.rs`.
6. **Fail-closed.** With the pattern set forced to fail to load, any command ⇒ a
   single `caro:internal:validator-unavailable` violation, `action == "block"`,
   `retryable == false`, exit `4`. Assert the process **never** exits `0`.
   Regression guard for analog failure mode 4.
7. **Frozen vocabulary golden file.** Snapshot every built-in `RuleId` to
   `tests/fixtures/rule_ids.golden`. Adding an id updates the file; renaming or
   removing one fails the suite — forcing the schema-version conversation.
8. **Back-compat.** For every case above, `matched_patterns.len() ==
   violations.len()` and the plain-text output is byte-identical to the
   pre-change baseline.
9. **`retryable` discriminates.** `rm -rf /` ⇒ `is_retryable() == false`;
   `chmod 777 /etc/passwd` (scope-target class) ⇒ at least one violation with
   `retryable == true` and `class == "scope_target"`. This is the assertion that
   the feature actually does its job.

### What breaks at 100 real users

- **The assumption:** `RemediationClass` is a static, hand-assigned column on 67
  built-in patterns. It holds while patterns are hand-curated; it degrades if the
  CVE ruleset grows to hundreds of imported rules with no curated class.
- **Failure mode:** imported rules default to `RequireApproval` /
  `retryable: false`, so agents stop reformulating on commands that are in fact
  fixable — a silent conservatism regression, not a safety hole.
- **Instrumentation:** a `caro doctor` count of rules whose class is the default,
  and the ratio of `retryable == false` violations in the existing project-memory
  run journal.
- **Fallback:** if the default-class ratio exceeds a threshold, class assignment
  moves into the Dogma rule schema (a build-time field) rather than staying a
  Rust-side table — an additive change to `src/dogma/`, not a redesign.

### Explicitly out of scope (next version)

- **`caro check "<cmd>"` — a validate-only subcommand** with no generation and no
  model load. Genuinely valuable (pure subprocess, sub-millisecond) but it is a
  new user-facing capability class, so `.claude/rules/validation-discipline.md`
  applies: 20 transcripts before an implementation PR. This scope stays an
  envelope extension and is exempt on that basis; the subcommand is not.
- **Concrete alternative commands.** Emitting an actual safer command string
  requires generation and is non-deterministic. v1 emits the *class* of
  reformulation, never a candidate.
- **Per-rule enforcement overrides in the policy file.** ADR-040 maps risk tiers
  to actions; overriding a single `rule_id` is a v2 extension that this scope
  deliberately unblocks (a stable id is its precondition).
- **Emitting violations as lifecycle / OTel events.** ADR-037 and ADR-041 own the
  event bus; `Violation` is designed to be embeddable there, but wiring it is a
  separate PR (market-scan opportunity C).
- **Session-level violation aggregation** — the ExploitGym gap (market-scan
  opportunity D). Explicitly spec-only, gated on validation discipline.
- **Violations on the MCP proxy and guard-adapter paths** (ADR-043, ADR-036,
  ADR-048). They consume `SafetyDecision`, so they inherit the field for free;
  their own contract tests are separate.
- **i18n of `message` / `hint`.** The stable layer is `rule_id`; prose
  localisation can land later without touching the contract.

---

## Constraint check

| Constraint | How this scope satisfies it |
|---|---|
| Reuse existing validator / safety / config infrastructure | New types live in `src/safety/mod.rs`; risk and action vocabularies reused verbatim; user-pattern hardening reused; no new module, no new dependency. |
| All new types serializable from day one | Every type derives `Serialize + Deserialize + JsonSchema`; `RuleId` is `#[serde(transparent)]`. |
| Pure subprocess call, no daemon, no state | Patterns are `once_cell::Lazy` statics; the decision is a pure function of (command, shell, safety, config). No cache, no polling, no client object — the opposite of the analog's long-lived-client model. |
| Solve the Phase-1 failure mode by design, not workaround | (1) One channel: `violations` present on every outcome, `[]` when empty. (2) Stable identity: frozen `RuleId` vocabulary with a golden test, replacing renameable display text. (3) Anti-thrash: `retryable` is a first-class field, not prose. (4) Fail-closed: validator failure is a synthetic blocking violation, never a silent allow. |

## Sources

- [Execlave — Documentation index](https://www.execlave.com/docs)
- [Execlave — Policies & Security](https://www.execlave.com/docs/policies)
- [Execlave — SDK Reference](https://www.execlave.com/docs/sdk-reference)
- [Execlave — Product Hunt launch](https://www.producthunt.com/products/execlave)
- [Cloudflare — WriteGuard: fine-grained controls for MCP servers](https://blog.cloudflare.com/mcp-portal-writeguard-private-beta/)
- `market-scans/2026-08-17-ai-agent-strategy-memo.md` (opportunity A, recommendation #2)
