# Implementation Scope — Trusted-Targets Registry (Affirmative Allowlisting)

**Feature under analysis:** Claude Code **permission rules**
(`permissions.allow` / `ask` / `deny` in `settings.json` — command-text
allowlisting with glob matchers, layered settings, workspace-trust gating;
docs fetched live 2026-08-07), plus its documented bypass class
([anthropics/claude-code#4956](https://github.com/anthropics/claude-code/issues/4956)).
**Equivalent we are scoping for Caro:** a `[trust]` registry in the ADR-040
policy file — affirmative target entries (paths, hosts, command families)
matched against ADR-044 *resolved effects* rather than command text, granting
bounded, monotone routing relief that never crosses safety floors.

**Date:** 2026-08-07 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-047-trusted-targets-registry.md`

> The task template left `[FEATURE NAME]` unfilled and ran unattended. Target
> selection: the 2026-08-07 Hermes scan's opportunity (c) — trusted-targets
> registry — is the only Now/Next engineering item with no covering ADR.
> Coverage check: ADR-040 maps risk→action (and deferred path-scoped rules),
> ADR-042 reserves identity selectors, ADR-044 produces target facts but only
> escalates on them; nothing provides target *affirmation*. Opportunities (a)
> and (b) are positioning content; (d) folded into ADR-046's schema work on
> 2026-08-06. The analog choice is a reviewable assumption.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom
Claude Code prompts for permission on every non-read-only tool call.
Permission rules let users and orgs pre-answer: `allow` rules run without
prompts, `ask` rules force prompts, `deny` rules block. Audience: developers
tired of approving `npm test` forty times a day, and platform teams
standardizing what agents may do across an org (managed settings). It is the
most widely deployed agent allowlist in existence — which makes both its good
decisions and its failure modes unusually well documented.

### Core architecture observed
- **Rule shape**: `Tool` or `Tool(specifier)` strings in three arrays.
  Bash rules glob the command string with word-boundary semantics
  (`Bash(ls *)` ≠ `Bash(ls*)`); Read/Edit rules use gitignore patterns with
  four anchor forms (`//abs`, `~/home`, `/settings-relative`, `relative`);
  WebFetch rules match hostnames with dot-crossing-safe wildcards; MCP rules
  name server/tool; parameter rules (`Agent(model:opus)`) match scalar
  inputs on deny/ask only.
- **Evaluation**: deny → ask → allow, first match, specificity irrelevant.
  Deny composes no exceptions (a broad deny beats a narrow allow).
- **Compound commands**: split on `&& || ; | |& &` + newlines; every
  subcommand must match independently. A fixed, non-configurable wrapper
  list (`timeout`, `nice`, `nohup`, bare `xargs`, safe env-var assignments)
  is stripped before matching.
- **Layering**: managed > CLI > local > project > user; deny at any layer is
  final. Project allow rules apply only after a workspace-trust dialog
  (allow grants capability; deny/ask only restrict — so only allow is
  gated). `allowManagedPermissionRulesOnly` gives admins exclusivity.
- **Lifecycle**: rules load from the settings chain at startup; `/permissions`
  lists rules with their source file; unmatchable rules produce startup
  warnings; approving "don't ask again" auto-writes rules (≤5 per compound
  command) to `settings.local.json`. No daemon; per-session initialization.

### Why it is limited — the failure modes
1. **Text is the wrong matching unit (structural).** Rules constrain the
   command *string*; the docs themselves enumerate five bypasses of
   `Bash(curl http://github.com/ *)` (flag order, protocol, redirects,
   variables, whitespace) and recommend abandoning Bash rules for URL
   filtering entirely. Issue #4956 documented full bypass via chaining
   before subcommand splitting shipped.
2. **Arms-race patching.** Chaining → operator splitting; wrappers → a
   built-in strip list; env-runners (`devbox run`, `npx`, `docker exec`) →
   *documented as unsolved*; `find -exec`, `watch`, `setsid` → special-cased
   to always prompt. Each fix is a new hand-maintained parser rule.
3. **Implicit allowlist growth.** "Yes, don't ask again" appends rules the
   user never reviews as a set; approving one compound command can write
   five rules.
4. **Cognitive-load semantics.** `/path` anchors differently in five
   settings sources; `src/**` matches different depths depending on rule
   type; symlink handling differs between allow (both paths must match) and
   deny (either path matches).
5. **No structured output.** Evaluation is internal to the interactive CLI:
   no machine-readable verdict, no "which rule matched, what was uncovered",
   nothing a second tool can consume. In headless `-p` mode, untrusted
   project rules are silently ignored.

### Structured output contract
None — that is the finding. Configuration in: JSON settings arrays. Verdict
out: an interactive prompt or its absence. Startup warnings on stderr for
typos. The permission system is a UI feature, not an API.

### Session/context lifecycle
Settings chain read per session; local-settings rules keyed to the git repo
root (v2.1.211+); workspace trust cached per repo root. Avoids redundant
initialization by session-scoped loading — but the state (which rules exist,
which are trusted) is invisible to external tooling.

## Phase 2 — Competitive Differentiation

### What they get right (replicate)
- **Deny always wins, everywhere** — no exception-composition into deny.
  Adopted as an invariant (trust relief inert at High/Critical; floors
  untouched).
- **Capability grants are provenance-gated** — workspace trust for project
  allow rules. Caro already has the analog seam: ADR-040's
  `policy.allow_project`, default off. Reused verbatim.
- **WebFetch wildcard semantics** — leading `*.` only, no dot-crossing
  wildcards elsewhere; the one matcher in their design with no known bypass.
  Adopted for host entries.
- **Conjunctive coverage of compounds** — every fragment must match.
  Transplanted from text fragments to effects: every write/delete/network
  effect must be covered, or no relief.
- **Fail-loud on unmatchable rules** — startup warnings for typos. Caro
  does one better: exit 11 hard errors (ADR-040 precedent).

### Their design gaps we avoid by designing the schema first
- **Matching unit**: effects, not text (kills bypass class 1–2 wholesale —
  there is no string to vary; an unresolvable command is untrustable by
  D1 of ADR-047).
- **Growth**: no auto-append in v1; entries are hand-written, reviewed,
  digest-bound (kills 3).
- **Anchors**: one rule — `./` = defining file's directory, `~/` = home,
  `/` = absolute; segment-aligned prefix matching (kills 4).
- **Contract**: `TrustDecision` is a serde type in every envelope, receipt,
  and event; `caro trust explain` is a pure-subprocess explain surface
  (kills 5).

### Unique positioning
Offline and deterministic (no I/O in matching); agent-agnostic (any harness
calls the same subprocess — their rules guard only their own CLI); one audit
digest covering tiers *and* trust (ADR-032/037/041 inherit it for free);
explainable (matched entries + uncovered effects named in every verdict);
and composable with fail-closed mode: deny enumerates the forbidden,
fail-closed escalates the unknown, trust quiets the known-good — three
orthogonal mechanisms, one action vocabulary.

### Existing infrastructure covering part of this
`SuggestedRouting` + strictness ordering (ADR-020/040) — relief vocabulary;
ADR-040 layering/ceiling/digest/exit-11 — the registry is a section of that
file; ADR-044 `EffectResolution`/`EffectSet`/`PathClass`/`HostClass` — the
facts; Critical pre-scan + `blend_smart_decision` floor — the unbreakable
deny; ADR-024 envelope — the additive `trust` block carrier.

## Phase 3 — Scope Definition

### ADR
`docs/adr/ADR-047-trusted-targets-registry.md` (written alongside this
scope): context, decision (D1–D7), consequences, five alternatives
considered. Amends ADR-040 on paper (additive `[trust]` section, ADR-042
precedent); first affirmative consumer of ADR-044.

### New types (existing modules only)
- `src/config/policy.rs`: `TrustSection { max_relief, path[], host[],
  family[] }`, `TrustPathEntry { id, prefix, ops, max_risk }`,
  `TrustHostEntry`, `TrustFamilyEntry` — all
  `#[serde(deny_unknown_fields)]` + `JsonSchema`; `max_risk > Moderate`
  and duplicate ids rejected at load; `ResolvedPolicy` gains
  `trust: Option<ResolvedTrust>`; **digest input extends over the resolved
  trust set**.
- `src/safety/mod.rs`: `TrustDecision { trusted, matched_entries,
  uncovered_effects, reason, routing_before, routing_after, relief_steps }`,
  `UntrustedReason { NoTrustConfigured | NotResolved | RiskAboveCap |
  UncoveredEffects | ReliefDisabled }`; pure total function
  `evaluate_trust(&ResolvedTrust, &EffectResolution, RiskLevel,
  SuggestedRouting) -> TrustDecision`.

### Minimal file set
| File | Change |
|---|---|
| `src/config/policy.rs` | trust schema, validation, merge, digest extension |
| `src/safety/mod.rs` | decision types + `evaluate_trust` + call site after effects resolution |
| `src/cli/mod.rs` | `caro trust explain`, envelope `trust` field |
| docs (this ADR, headless contract) | contract documentation |

No new modules, no new dependencies.

### Exit code / output contract
**No new exit codes** (0–12 stay frozen). Relief converts exit 4 → exit 0
paths; malformed `[trust]` is exit 11 (`PolicyInvalid`); `caro trust
explain "<cmd>"` prints `{effects, trust}` JSON, exits 0 or 11. Envelope
gains additive optional `trust` block (schema_version unchanged). Machines
depend on: `trust.trusted`, `trust.reason`, `trust.routing_after`,
`matched_entries` ids, and digest stability.

### Integration tests (known input → deterministic JSON + exit code)
Ten cases specified in ADR-047 §4, headline invariants: happy path untaxed
(no trust configured ⇒ no behavior change); relief applies one step at
Moderate with entry ids named; `rm -rf /` stays exit 3 regardless of trust
(floor test); `rm -rf $DIR` gets no relief (`not_resolved`); uncovered
write defeats covered network (conjunctive test); `./build` does not match
`./build-scripts` (segment alignment); `*.github.com` does not match
`github.com.evil.example` (wildcard safety); `max_risk="high"` entry →
exit 11; trust-entry change → digest change (golden file); `trust explain`
byte-stable.

### Constraints compliance
- **Reuse, don't duplicate**: layering/ceiling/digest/exit-11 from ADR-040;
  effects facts from ADR-044; routing vocabulary from ADR-020; floors from
  the existing validator. Zero parallel machinery.
- **Serializable from day one**: every new type derives serde (+ JsonSchema
  in policy).
- **Pure subprocess**: matching is string/effects analysis only — no
  filesystem, DNS, daemon, or cache; `trust explain` re-resolves per run.
- **Phase-1 failure mode solved by design**: the analog's bypass class
  exists because trust is expressed over command text. Caro expresses
  trust over resolved effects and makes unresolved commands untrustable
  (ADR-047 D1) — a bypass now requires defeating the resolver, and the
  resolver's "unsure ⇒ Partial" rule means its failure direction is
  *missed relief*, never unearned trust.

### Out of scope (next version)
`caro trust suggest` / any auto-append from approvals; principal-scoped
entries (ADR-042 seam); symlink/realpath matching (needs I/O — explicit
opt-in later); compound-command trust (needs ADR-044 v2 decomposition);
read-effect coverage; central registry distribution/signing; any relief at
High/Critical or of ADR-045 strong-auth ceremonies (never).

---

*Sources: [Claude Code permissions docs](https://code.claude.com/docs/en/permissions) ·
[anthropics/claude-code#4956](https://github.com/anthropics/claude-code/issues/4956) ·
Hermes market scan `.hermes/digests/2026-08-07-weekly-agent-market-scan.md` ·
ADR-020/024/040/042/044 in `docs/adr/`.*
