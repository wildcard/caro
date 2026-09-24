# ADR-047: Trusted-Targets Registry — Effects-Bound Affirmative Allowlisting with Bounded, Monotone Routing Relief

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-07
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: **Claude Code permission rules**
  (`permissions.allow` / `ask` / `deny` in `settings.json`; docs fetched
  live 2026-08-07 from code.claude.com/docs/en/permissions), read against
  the documented Bash-rule bypass class
  ([anthropics/claude-code#4956](https://github.com/anthropics/claude-code/issues/4956))
  and the pattern of gateway-side action registries announced at Black Hat
  2026 (Cequence API/Skill Registries, Hermes scan 2026-08-07 item 6)
- **Amends**: ADR-040 (tiered authorization policy file — adds the `[trust]`
  section to its schema before implementation; additive-on-paper, same
  precedent as ADR-042), ADR-044 (fail-closed effects resolution — this ADR
  is the first *affirmative* consumer of `EffectResolution`)
- **Depends on**: ADR-020 (`SuggestedRouting`), ADR-024 (headless envelope +
  frozen exit-code registry 0–12), ADR-040 (layered resolution, ceiling,
  digest, exit 11), ADR-044 (`EffectSet`, `ResolutionStatus`, `PathClass`,
  `HostClass`)
- **Relates to**: ADR-027 (approval resolution — fewer prompts is the point),
  ADR-032 (receipts carry the policy digest, which now covers trust),
  ADR-037/041 (events carry the `trust` block by serialization), ADR-042
  (principal selectors — a future trust dimension, reserved not implemented),
  ADR-043 (MCP gateway payloads carry the `trust` block), ADR-045/046
  (approval surfaces — trust reduces how often they fire), Hermes market
  scan 2026-08-07 (opportunity c)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Target selection
> rationale: of the 2026-08-07 Hermes scan's opportunities, (a) and (b) are
> positioning content, and (d) folds into the assessment-schema work already
> scoped on 2026-08-06 (ADR-046). Opportunity (c) — "Caro has
> dangerous-pattern *denial* but no allowlisted-target *affirmation* …
> spec the trusted-targets schema + how it composes with the 52-pattern
> denylist (deny always wins)" — is the only Now/Next engineering item with
> no covering ADR: ADR-040 maps risk→action but explicitly deferred
> path-scoped rules; ADR-042 reserves identity selectors; ADR-044 produces
> target *facts* but only ever escalates on them. Nothing lets a fact
> *quietly relax* routing. The analog choice (Claude Code permission rules,
> the most widely deployed agent allowlist in the field) is a reviewable
> assumption. Treat D4's relief band (Decision) as the load-bearing
> reviewable assumption.
>
> **Validation-discipline note.** Architecture scope, no PMF claim. The
> prompt-fatigue evidence cited (≈93% of permission prompts approved,
> Hermes 08-07 §3c) is market telemetry, not caro discovery data; gate
> obligations under `.claude/rules/validation-discipline.md` attach to the
> implementing spec/PR, not to this document.

---

## 1. Context

### 1.1 The feature researched (Phase 1)

Claude Code's permission rules are the de-facto standard agent allowlist:
`permissions.allow` / `ask` / `deny` arrays of `Tool(specifier)` rules,
layered across managed/user/project/local settings files, evaluated
deny → ask → allow, first match wins. Bash rules glob-match the command
*string* (`Bash(npm run test *)`), path rules use gitignore syntax with
four anchor forms, WebFetch rules match domains with dot-crossing-safe
wildcards. Compound commands are split on `&& || ; | |& &` and newlines,
and each subcommand must match independently. Approvals ("Yes, don't ask
again") auto-append rules to `settings.local.json` — up to 5 per compound
command. Project-supplied allow rules are gated behind a workspace-trust
dialog, because allow rules *grant capability* while deny rules only
restrict.

**Why it remains fragile — the bypass class is structural.** The rules
match **command text**, so every syntactic variation is a potential
bypass, and the vendor's own documentation concedes it:

1. **Chaining** (issue #4956): prefix rules historically approved
   `safe-cmd && other-cmd`; the fix — subcommand splitting — is a
   hand-maintained parser of shell operators.
2. **Argument-position fragility** (documented verbatim in their docs):
   `Bash(curl http://github.com/ *)` fails to constrain curl — options
   before the URL, `https://` vs `http://`, redirects, `$URL` variables,
   even a double space all defeat it.
3. **Wrapper arms race**: a fixed, non-configurable strip list
   (`timeout`, `nice`, `nohup`, `xargs`-without-flags, …) decides what a
   rule "sees". Environment runners (`devbox run`, `npx`, `docker exec`)
   are *not* stripped, so `Bash(devbox run *)` silently approves
   `devbox run rm -rf .` — a documented footgun, not a fixed one.
4. **Implicit allowlist growth**: every "don't ask again" writes rules
   the user never reviews as a set.
5. **Anchor-semantics load**: `/path` means something different in each
   of five settings sources, and `src/**` matches at different depths
   depending on rule *type* — correctness requires reading a 60-page
   document.
6. **No structured verdict**: rule evaluation is internal to the
   interactive CLI. There is no machine-readable "why was this allowed,
   which rule, what did it cover" — nothing another tool can consume.

What their design **gets right**, and this ADR replicates deliberately:
deny always beats allow, across every layer; allow-grants are gated on
provenance/trust while restrictions are not; WebFetch's wildcard rules
prevent registrable-suffix tricks (`example.*` cannot match
`example.evil.com`); and compound commands must be covered *in full*,
never by their first fragment.

### 1.2 The gap in caro (Phase 2)

Caro's safety story today is entirely **negative space**: 52+ dangerous
patterns (deny), a Critical floor, and — once ADR-040/044 land — a
risk→action policy map plus fail-closed *escalation* on unresolvable
commands. There is no way to state the affirmative fact every team
actually has: *"writes under `./build` are fine here; pushes to
`github.com` are fine here"* — and have routing quietly reflect it.
Consequence: Moderate-risk commands touching well-known project targets
prompt exactly as often as Moderate-risk commands touching anything else.
The market data says users answer yes ≈93% of the time; a prompt that is
almost always approved is a prompt users learn to stop reading. Tier
*quality* — fewer, better prompts — is the competitive axis now
(Hermes 08-07 §2).

Existing infrastructure this ADR reuses rather than duplicates:

- **ADR-044 `EffectResolution`** — the facts (`writes`, `deletes`,
  `network`, `family_id`, `PathClass`/`HostClass`, `ResolutionStatus`).
  Without it, an allowlist could only match command text — the exact
  construction Phase 1 shows failing.
- **ADR-040 layering** — system/user/project precedence,
  `allow_project` gating, ceiling file, monotonicity checks, sha-256
  policy digest, exit 11 fail-loud parsing. The registry is a section of
  the same file, resolved by the same pure function, covered by the same
  digest.
- **`SuggestedRouting`** and its strictness order — relief is expressed
  in the one existing action vocabulary.
- **Safety validator invariants** — the Critical pre-scan and
  `blend_smart_decision`'s floor are untouched and take precedence.

**Caro's differentiation vs. the analog** (Phase 2): the registry matches
*resolved effects, not text*, so the entire text-variation bypass class
is designed out rather than patched case-by-case; it is offline and
deterministic (pure string/effects analysis, no filesystem or DNS I/O);
it is agent-agnostic and callable as a pure subprocess by any harness
(Claude Code's rules only guard Claude Code); every decision is a
serializable payload with matched entries and uncovered effects named;
and the allowlist never grows implicitly.

## 2. Decision

Add a **`[trust]` section** to the ADR-040 policy file: a registry of
affirmative target entries (paths, hosts, command families) that, when a
command's **fully resolved** effect set is *entirely covered* by matching
entries, grants a **bounded, monotone routing relief** — never crossing
the existing safety floors, never applying to unresolved commands, and
never composing exceptions into deny.

### D1 — Trust requires resolution: no facts, no relief

A command is eligible for trust evaluation **only** when its ADR-044
resolution status is `Resolved`. `Partial` and `Unresolved` commands —
dynamic operands, unknown flags, compound commands, no family match —
get `trusted: false` with a machine-readable reason and unchanged
routing. This is the design-level answer to Phase 1's failure mode:
there is no command *string* to game, and anything the resolver cannot
fully account for is by definition untrustable. A resolver gap costs
convenience (a prompt that could have been quiet), never safety.

### D2 — Schema (`[trust]` inside `caro-policy.toml`, schema_version 1)

```toml
[trust]
max_relief = 1                  # routing steps an entry may relax (0–2, default 1)

[[trust.path]]
id = "proj-build-writes"        # required, unique, lands in audit artifacts
prefix = "./build"              # anchored at the policy file's own directory
ops = ["write", "delete"]       # subset of: read | write | delete
max_risk = "moderate"           # entry is inert above this risk (cap: "moderate")

[[trust.host]]
id = "github"
host = "*.github.com"           # exact, or leading "*." label wildcard only
max_risk = "moderate"

[[trust.family]]
id = "git-usual"
family = "fam.git"              # ADR-044 family id, verbatim
max_risk = "moderate"
```

- `#[serde(deny_unknown_fields)]`, duplicate `id` is a load error, and
  `max_risk` above `moderate` is a load error — trust can never be
  *configured* into High/Critical territory (exit 11, ADR-040
  semantics). `high`/`critical` are rejected at parse time, not clamped:
  fail-loud, never silently narrow.
- **Path entries**: matched against ADR-044 `PathEffect` values after
  *syntactic* canonicalization only (collapse `.`/`..`, normalize
  separators — no filesystem I/O, no symlink resolution; symlink-aware
  matching requires I/O and is explicitly v2, see Out of scope, with the
  consequence that a `Dynamic`-class path never matches anything).
  `prefix` anchors: `./` = the defining policy file's directory (one
  anchor rule, not five — kills Phase 1 footgun 5), `~/` = home, `/` =
  absolute. Prefix match is segment-aligned (`./build` matches
  `./build/x` but not `./build-scripts`), mirroring the analog's
  word-boundary lesson.
- **Host entries**: exact match, or a leading `*.` that matches one or
  more whole labels — the analog's WebFetch semantics, adopted because
  they are the one part of their matcher design with no known bypass.
  No wildcard anywhere else in the string.
- **Family entries**: trust every command the named family resolves,
  subject to `max_risk` and full coverage of its effect set by this or
  other entries. Useful for read-mostly families (`fam.readonly`,
  `fam.git` query subcommands).

### D3 — Coverage is conjunctive: every effect, or no relief

A command is `trusted` iff **each** effect in `writes ∪ deletes ∪
network` is covered by ≥1 matching entry whose `ops` include that
effect kind and whose `max_risk` ≥ the command's assessed risk. Reads
are exempt in v1 (mirroring the analog's read-only set; reads still
influence risk normally). One uncovered effect ⇒ `trusted: false`, with
the uncovered effects listed in the output. This is the analog's
"every subcommand must match" rule, transplanted from text fragments to
effects — the unit that cannot be smuggled past by syntax.

### D4 — Relief is bounded, monotone, and floor-respecting

When trusted, routing relaxes by at most `max_relief` steps along
`Block > HumanGate > AsyncLog > AutoApprove`, subject to **all** of:

1. **Risk cap**: relief applies only when assessed risk ≤ `moderate`.
   High and Critical routing is *never* modified by trust — the
   52-pattern denylist and the Critical floor win unconditionally
   ("deny always wins", stated as an invariant, not an evaluation
   order).
2. **Ceiling supremacy**: the ADR-040 ceiling file sets minimum actions
   per tier; trust relief cannot go below the ceiling. A ceiling may
   also pin `trust.max_relief = 0`, disabling relief system-wide.
3. **One-way composition with ADR-044**: fail-closed escalation
   (unresolved ⇒ escalate) and trust relief (resolved + covered ⇒
   relax) are disjoint by construction — D1 makes trust inapplicable
   exactly where fail-closed applies. No ordering ambiguity exists.
4. **Provenance gating**: `[trust]` entries in a *project* layer apply
   only when the user config sets `policy.allow_project = true`
   (ADR-040's existing gate — the analog's workspace-trust dialog,
   already designed). System and ceiling layers may *remove* relief
   (via `max_relief = 0`) but a project layer can never widen
   `max_relief` beyond the user/system value: layers tighten, never
   loosen — the same direction ADR-040 already enforces for tiers.

Relief lands in the decision as data, never silently: the routing field
shows the relaxed action, and the `trust` block shows what it would have
been, which entries matched, and why.

### D5 — New types (existing modules, serializable from day one)

In `src/config/policy.rs` (the ADR-040 file):

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrustSection {
    #[serde(default = "default_max_relief")] pub max_relief: u8,   // 0..=2
    #[serde(default)] pub path:   Vec<TrustPathEntry>,
    #[serde(default)] pub host:   Vec<TrustHostEntry>,
    #[serde(default)] pub family: Vec<TrustFamilyEntry>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrustPathEntry {
    pub id: String,
    pub prefix: String,
    pub ops: Vec<EffectKind>,          // Read | Write | Delete
    pub max_risk: RiskLevel,           // load error if > Moderate
}
// TrustHostEntry { id, host, max_risk }; TrustFamilyEntry { id, family, max_risk }
```

In `src/safety/mod.rs` (beside ADR-044's types; no new module):

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UntrustedReason {
    NoTrustConfigured, NotResolved, RiskAboveCap,
    UncoveredEffects, ReliefDisabled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TrustDecision {
    pub trusted: bool,
    pub matched_entries: Vec<String>,          // entry ids
    pub uncovered_effects: Vec<String>,        // human/machine-readable effect refs
    pub reason: Option<UntrustedReason>,       // None when trusted
    pub routing_before: SuggestedRouting,
    pub routing_after: SuggestedRouting,       // == before when untrusted
    pub relief_steps: u8,
}
```

Method contracts: `TrustSection::validate(&self) -> Result<(), PolicyError>`
(id uniqueness, `max_risk` cap, `max_relief` range — called from
`AuthorizationPolicy::from_toml_str`); pure function
`evaluate_trust(&ResolvedTrust, &EffectResolution, RiskLevel,
SuggestedRouting) -> TrustDecision` — total, no I/O, no panics.
`ResolvedPolicy` gains `trust: Option<ResolvedTrust>` (merged entries +
effective `max_relief`, each entry recording its `PolicySource`), and the
**policy digest input extends to cover the resolved trust set** — a
changed trust entry changes the digest in every receipt and event
(ADR-032/037/041) with zero additional wiring.

Additive carriers: `ValidationResult` and the ADR-024 envelope gain
`trust: Option<TrustDecision>` (schema_version stays 1; field is
additive, same precedent as ADR-044's `effects` block). ADR-041 events
and ADR-043 MCP payloads inherit it by serialization.

### D6 — Exit codes and output contract: nothing new

Exit codes 0–12 are frozen; **this ADR adds none.** Trust only ever
*relaxes* routing, so it can change an exit 4 (`NeedsConfirmation`) into
an exit 0 — codes that already exist. Malformed `[trust]` sections are
exit 11 (`PolicyInvalid`), because the registry *is* policy. The one new
introspection surface:

- `caro trust explain "<command>"` — prints
  `{ effects: EffectResolution, trust: TrustDecision }` as JSON,
  exit 0 (evaluated, whatever the outcome) or 11 (policy invalid).
  Pure subprocess: policy layers are re-read and re-resolved per
  invocation, nothing cached, no daemon — identical inputs give
  byte-identical output. This is the explainability surface the analog
  lacks entirely (Phase 1 gap 6).

### D7 — No implicit growth

v1 has **no** "don't ask again"-style auto-append. Trust entries are
written by humans in a reviewed file, versioned with the project, bound
into the digest. An assisted `caro trust suggest` (emit a candidate
entry from the last prompted command, for the human to paste) is
deferred — see Out of scope. This is a deliberate inversion of the
analog's most corrosive behavior (Phase 1 gap 4).

## 3. Files that change (minimal set)

| File | Change |
|---|---|
| `src/config/policy.rs` (ADR-040's file) | `TrustSection` + entries, validation, merge into `ResolvedPolicy`, digest extension |
| `src/safety/mod.rs` | `TrustDecision`, `UntrustedReason`, `evaluate_trust`, call site after effects resolution |
| `src/models/mod.rs` | none beyond ADR-040's planned `strictness()` (reused for relief stepping) |
| `src/cli/mod.rs` | `caro trust explain` subcommand; envelope `trust` field |
| `docs/adr/ADR-047-…` (this file), headless-contract doc | contract docs |

No new modules, no new dependencies, no new exit codes.

## 4. Integration tests (deterministic: input → JSON + exit code)

1. **Happy path untaxed**: `ls -la`, no `[trust]` configured →
   `trust: {trusted:false, reason:"no_trust_configured"}`, routing and
   exit unchanged.
2. **Relief applies**: policy trusts `./build` for write+delete at
   Moderate; `rm -rf ./build` (Resolved, Moderate) → routing
   `human_gate → async_log` (one step), `matched_entries:
   ["proj-build-writes"]`, exit 0.
3. **Floor invariant**: same policy, `rm -rf /` → Critical, trust block
   reports `reason:"risk_above_cap"`, routing `block`, exit 3.
4. **No facts, no relief**: `rm -rf $DIR` → resolution `Partial` →
   `reason:"not_resolved"`, routing unchanged.
5. **Conjunctive coverage**: trust host `*.github.com` only;
   `curl https://github.com/a -o /etc/x` → network covered, write to
   `/etc/x` uncovered → `trusted:false`,
   `uncovered_effects:["write:/etc/x"]`.
6. **Segment alignment**: trust `./build`; command writing
   `./build-scripts/x` → uncovered (no prefix match across segment
   boundary).
7. **Wildcard safety**: trust `*.github.com`; effect host
   `github.com.evil.example` → no match (labels must align right-anchored).
8. **Load-time caps**: entry with `max_risk = "high"` → exit 11;
   duplicate ids → exit 11; project layer raising `max_relief` above the
   user layer → exit 11.
9. **Digest binding**: adding one trust entry changes
   `ResolvedPolicy.digest`; receipts/events reflect it (golden-file).
10. **Explain determinism**: `caro trust explain "git push origin main"`
    is byte-stable across runs against a fixed policy fixture.

## 5. Consequences

**Positive.** Caro gains the affirmation half of a complete authorization
story: deny enumerates the forbidden, fail-closed escalates the unknown,
and trust quiets the known-good — three orthogonal, composable
mechanisms with one action vocabulary and one audit digest. Prompt
volume drops exactly where prompts carry no information (the ≈93%-yes
zone), which is the strongest available defense against approval
fatigue eroding the HumanGate tier. The registry mirrors the enterprise
registry pattern (Cequence) locally and offline, and `trust explain`
gives caro an explainability surface no text-matching allowlist can
offer.

**Negative / risks.** Relief is only as sound as effects resolution;
D1 confines the blast radius of resolver bugs to *missed relief*, but a
resolver that wrongly marks a command `Resolved` with an incomplete
effect set could earn unmerited relief — mitigated by the risk cap
(≤ Moderate), the untouched Critical/High floors, and ADR-044's
"unsure ⇒ Partial" extraction rule, and bounded to one routing step by
default. Path matching without filesystem I/O cannot see symlinks; the
analog's symlink double-check is acknowledged as better *when I/O is
permitted* and deferred with that framing. Two-file governance
(patterns + policy w/ trust) raises documentation load — mitigated by
`caro trust explain` and by trust living inside the existing policy
file rather than a third artifact.

## 6. Alternatives considered

1. **Command-text allowlist (the analog's design).** Rejected on the
   evidence of Phase 1: the bypass class is structural. Caro would be
   re-implementing the wrapper-strip arms race with fewer maintainers.
2. **Separate `caro-trust.toml` file.** Rejected: duplicates ADR-040's
   layering, gating, ceiling, and digest machinery for no expressive
   gain; a second artifact to drift. A section in one policy file keeps
   one resolution function and one digest.
3. **Trust as risk-level *input* (trusted ⇒ lower `RiskLevel`).**
   Rejected: it would corrupt the audit trail (the command's risk is
   what it is; the *decision* is what changed) and let trust leak into
   every risk consumer. Relief acts on routing only, downstream of
   risk, visibly.
4. **Auto-learning from approvals in v1.** Rejected (D7): implicit
   growth is the analog's documented governance failure, and
   validation-discipline requires evidence before shipping
   convenience-that-widens-capability.
5. **Escalate on untrusted targets (allowlist-only mode) in v1.**
   Rejected as duplicate: ADR-044's fail-closed mode already provides
   the escalation ratchet, keyed on resolvability. Adding a second
   ratchet keyed on trust coverage would create two overlapping
   "unknown ⇒ escalate" semantics; revisit only if field evidence shows
   resolvable-but-untrusted commands are a real gap.

## 7. Out of scope (next version)

- `caro trust suggest` / any auto-append from approvals (D7).
- Principal-scoped trust (`agents = ["spiffe://…"]` on entries) — the
  ADR-042 selector block is the reserved seam; restrict-only semantics
  would apply.
- Symlink/realpath-aware path matching (requires filesystem I/O; would
  be an explicit opt-in flag with documented determinism loss).
- Compound-command trust (blocked on ADR-044 v2 decomposition).
- Read-effect coverage and read-scoped entries.
- Central/remote registry distribution and signing (enterprise
  distribution belongs with managed-settings work).
- Relief beyond routing (e.g. skipping ADR-045 strong-auth ceremonies —
  explicitly never: strong-auth applies to CRITICAL, where trust is
  inert by D4.1).
