# ADR-059: Implementing `caro.assessment.v1` — Module Placement, Verb Namespace, and Structured Blend Provenance

- **Status**: Proposed (implementation ADR — this one is meant to become code, not another scope)
- **Date**: 2026-08-26
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: the **Claude Agent SDK permission-evaluation surface** — the six-step
  order `hooks → deny rules → ask rules → permission mode → allow rules → canUseTool`, the
  `canUseTool` / `PermissionResult` callback, the critical-path floor, and the
  `CLAUDE_SDK_CAN_USE_TOOL_SHADOWED` warning — read live 2026-08-26 at
  [code.claude.com/docs/en/agent-sdk/permissions](https://code.claude.com/docs/en/agent-sdk/permissions)
- **Implements**: ADR-058 (`caro.assessment.v1`). Does **not** redesign it
- **Amends**: ADR-058 D7 — `Projection` gains one field, `enforcement` (D5 below)
- **Depends on**: nothing. No new dependency, no new module, no upstream ADR
- **Unblocks**: ADR-020, ADR-023, ADR-046, ADR-052, ADR-055, ADR-057 §3.6 — all of which
  block on a type that does not yet exist in `src/`
- **Full scope document**: `caro-scope-assessment-implementation-2026-08-26.md`

> **Provenance and a declared boundary.** Today's strategy memo
> (`market-scans/2026-08-26-ai-agent-strategy-memo.md` §4) proposes *"a moratorium on new
> ADRs in this space until `caro.assessment.v1` and one lifecycle emitter are merged,"*
> and names as the single largest risk in the project the gap between 24 proposed
> governance ADRs and zero shipped modules. This ADR is written **in support of that
> moratorium, not in violation of it**: it decides only what must be decided to write the
> code, it adds exactly one field to the contract, and it declares itself the last ADR in
> this space until `src/safety/assessment.rs` merges. If a reviewer concludes it should
> have been a bead instead of an ADR, that reviewer is right about the general case and
> the disagreement is worth one comment, not a rewrite — D2, D3 and D5 below have
> alternatives and consequences that a bead would have swallowed.
>
> **Validation-discipline note.** Gate 3 (demoware trap) is written — scope §3.8. Gate 4
> (`devils-advocate` review) is **required before the implementation PR opens** and is not
> discharged here. Gates 1, 2 and 5 attach to the features that consume this payload, per
> ADR-058's argument, which still holds.

---

## Context

ADR-058 (2026-08-25) designed `caro.assessment.v1`: one versioned, schema-emitting payload
with the risk taxonomy and the decision taxonomy as separate fields, and with **required**
provenance saying whether a language model touched the verdict. It was explicit that it
contained *"no implementation."*

Writing the implementation plan surfaced three things ADR-058 could not have known, because
it was written against the payload and not against the call sites.

### 1. `Evidence::Blended` is not constructible from Caro's current code

ADR-058 D3 calls `evidence` *"the F7 fix and the reason this ADR exists."* Its `Blended`
variant must carry the judge attribution, **the deterministic decision and risk the judge
overrode**, and a direction of `Relaxed` or `Escalated`.

Caro's blend function, verified at `src/safety/mod.rs:277–330`, returns:

```rust
pub struct SmartDecision {        // :256–263
    pub requires_confirmation: bool,
    pub blocked: bool,
    pub note: Option<String>,
}
```

It receives `static_risk` and drops it; it reads the judged risk and drops it; it never
computes a `SuggestedRouting` at all; it never sees the judge's backend or model; and it
encodes the direction **in the wording of a prose string** — `"smart: relaxed to …"` at
`:306–312` versus `"smart: flagged as …"` at `:324–327`.

That is precisely ADR-058's rejected alternative **A3** (*"emit provenance as a free-text
`note`"*), and the current implementation is structurally incapable of anything else. Any
PR that adds the `Assessment` struct without touching this function must either hard-code
`Evidence::Deterministic` — a lie under `--approval smart` — or regex the note, which is A3
with extra steps. **The refactor is not optional and it is not separable.**

### 2. `caro assess` is already taken, by an unrelated meaning

ADR-058 D8 introduces *"one new verb, `caro assess`."* The name is occupied in three places:

- `src/assessment/` — a **hardware** assessment module (`cpu.rs`, `gpu.rs`, `memory.rs`,
  `profile.rs`, `recommender.rs`), exporting `AssessmentResult`, `SystemProfile`,
  `ModelRecommendation`.
- `src/main.rs:422–434` — a commented-out `Commands::Assess`, documented *"Assess system
  resources and get model recommendations,"* disabled since v1.1.0-beta.1.
- `tests/assess_integration_test.rs` — two `#[ignore]`d tests that invoke `caro assess` and
  assert hardware output.

Shipping `caro assess <command>` for command safety alongside a module where "assessment"
means system profiling reuses one name for two semantics: the exact **F1** failure that
ADR-058's **D1** exists to prevent, committed at the verb layer by the ADR that named it.

### 3. A host's `allow` is not authoritative, and Caro's payload cannot say so

The permissions reference, fetched today, documents an asymmetry the hooks reference does
not:

> "Run hooks first. A hook can deny the call outright or pass it on. **A hook that returns
> `allow` does not skip the deny and ask rules below**; those are evaluated regardless of
> the hook result."

> "**Auto-approved tools never reach `canUseTool`.** A tool call approved at any earlier
> step … skips your `canUseTool` callback, so **permission checks you put there are
> silently bypassed for that tool.**" — with a dedicated warning code,
> `CLAUDE_SDK_CAN_USE_TOOL_SHADOWED`, and the remedy *"use a `PreToolUse` hook instead."*

So: a Caro `block` projected to `deny` survives `--dangerously-skip-permissions` and is
**authoritative**. A Caro `auto_approve` projected to `allow` can be overturned by three
later steps and is **advisory**. ADR-058's `Projection.lossy` cannot express this — `lossy`
means *the value changed shape*, and `auto_approve → allow` is lossless **and** advisory.

The failure this permits is concrete: an integrator reads `projections[].value == "allow"`,
records *"Caro approved"* in an audit trail, and ships a compliance artifact asserting a
decision Caro never had the authority to make.

---

## Decision

Six decisions. Five are implementation; one (D5) amends the contract by one field.

### D1 — The code lives in `src/safety/assessment.rs`. No new module.

One new file under the existing `safety` module, re-exported from `src/safety/mod.rs`.
The path does not collide with `src/assessment/` (hardware), and the payload's natural home
is next to the validator that produces it. Six files are touched, none created beyond this
one and one test file. Dependency delta is **zero** — `serde`, `serde_json` and
`schemars 0.8` are already direct dependencies (`Cargo.toml:41,42,46`), so
`.claude/rules/external-sdk-integration.md` does not attach and no build spike is required.

### D2 — The verb is `caro assess <command>`, and hardware assessment gives up the name.

`caro assess "rm -rf /"` is the command-safety verb. Concretely:

- The dead commented block at `src/main.rs:422–434` is **deleted** in this PR
  (`.claude/rules/good-boy-scout.md`: it is dead code in a file we are already editing).
- The two `#[ignore]`d hardware tests in `tests/assess_integration_test.rs` are **deleted
  or retargeted**. They must not survive asserting hardware output from a live command.
- Hardware assessment is reassigned to `caro doctor --resources` — `caro doctor` already
  exists and is the system-diagnostics verb. A bead is filed; the work is **not** done here.
- `src/assessment/` is **not renamed**. Renaming a module to improve a noun is
  gold-plating; a doc-comment clarifying that it means *system profiling* is enough.

**JSON is the default output for this verb.** `caro assess "<cmd>"` writes exactly one
`Assessment` and one newline to stdout on every exit code. Human-readable output is opt-in
via `--human`. This inverts Caro's usual default deliberately: the verb exists for machines,
and a verb whose stdout shape depends on a flag cannot make the guarantee in D6.

### D3 — `blend_smart_decision` returns structured provenance; `note` is retained verbatim.

```rust
pub struct SmartDecision {
    pub requires_confirmation: bool,   // unchanged meaning and value
    pub blocked: bool,                 // unchanged
    pub note: Option<String>,          // unchanged, verbatim
    pub evidence: Evidence,            // NEW
    pub decision: SuggestedRouting,    // NEW — post-blend routing, computed once
}

pub fn blend_smart_decision(
    static_risk: RiskLevel,
    judge: Option<&RiskJudgment>,
    safety: SafetyLevel,
    auto_confirm: bool,
    attribution: Option<JudgeAttribution>,   // NEW — 5th parameter
) -> SmartDecision
```

Every existing early return acquires an `Evidence` variant (the five-branch table is in
scope §3.4), so none can be forgotten. `from_decision` is computed inside the function from
arguments it already receives — `SuggestedRouting::from_risk_and_safety(static_risk,
safety)` — so the call graph does not change. `attribution` is the only new input and is
built at the single call site (`src/cli/mod.rs:822`) from
`self.backend.backend_info()`, which already exposes `backend_type` and `model_name`
(`src/backends/mod.rs:86–95`). **No trait change.**

A test asserts `note.is_some() == matches!(evidence, Evidence::Blended{..})`, which closes
the prose/structure drift permanently rather than trusting future editors.

The existing branch that today returns `note: None` because the judge agreed with the static
verdict gains `Evidence::ModelConsultedNoEffect { reason: VerdictAgreed }` — information the
current code computes and silently discards.

### D4 — No feature flag. This ships in `default`.

Today's memo recommends *"implement `SafetyAssessmentOutput` behind a feature flag."* This
ADR declines, and the reasoning is the point of the whole exercise: **a contract that is
absent from the default binary is a contract nothing can depend on**, which is the exact
condition that has left ADR-020 and ADR-023 blocked for months on a type nobody could import.
Five ADRs are waiting for something to `use`.

The gating that a flag would have provided comes from three cheaper places: the verb is
opt-in by being a subcommand; the schema is versioned, so `caro.assessment.v2` can change
anything; and no existing code path's behaviour changes. `.claude/rules/external-sdk-integration.md`
mandates a flag for **external SDK dependencies**, of which this has none.

### D5 — `Projection` gains `enforcement: Authoritative | Advisory`. (Amends ADR-058 D7.)

```rust
pub enum Enforcement { Authoritative, Advisory }
```

`block → deny` on Claude Code is `Authoritative` (survives `--dangerously-skip-permissions`).
`auto_approve → allow` is `Advisory` (three later steps can overturn it). `Projection` also
gains `integration_point: String` — `"pre_tool_use_hook"` or `"can_use_tool"` — because
Claude Code has **two** integration points with different vocabularies and different
coverage, and a table keyed only by host cannot represent that.

This is the only change to ADR-058's contract made here. It is additive, it is two enum
variants and one string, and it is forced by a quoted sentence rather than by taste.

### D6 — The payload carries `caro_version`, and the output guarantee is absolute.

`Assessment.caro_version` is `env!("CARGO_PKG_VERSION")`. Reproducibility is therefore
*byte-identical within a version*, not across versions — stated honestly rather than
claimed absolutely. Golden fixtures store a `"<VERSION>"` sentinel and the test substitutes,
so a release bump does not break eight fixtures and provoke someone into deleting the test.

The output guarantee: **stdout is exactly one JSON object and exactly one newline on every
exit code, including `1` and `2`.** A consumer never handles empty stdout. Errors emit
`decision: block`, `fail_mode: fail_closed`, and a namespaced `reason_code` — `assess_or_block`
is infallible by signature, so silence is unrepresentable.

`ExitCode` is one `#[repr(i32)]` enum in `src/safety/assessment.rs` holding every code
claimed by any sibling ADR, with a test asserting no two variants collide:

| Code | Condition |
|---|---|
| `0` | `auto_approve` — and `--schema` |
| `18` | `async_log` — permitted, host **must** record |
| `19` | `human_gate` |
| `20` | `block` |
| `2` | usage error — fail closed, payload still written |
| `1` | internal error — fail closed, payload still written |

Monotonic in restrictiveness, so `[ $? -ge 19 ]` is a valid gate without parsing JSON.
The code is derived from `decision` by one function, so payload and code cannot disagree.

`caro assess --schema` prints `schema_for!(Assessment)` and exits `0`. The existing
`src/bin/generate-schema.rs` gains a second output, `schemas/caro.assessment.v1.schema.json`,
committed to the repo, with a test asserting the committed file byte-equals the generated
one. That is the schema-drift alarm ADR-058 asked for, built from infrastructure that
already exists.

---

## Consequences

### Positive

- **The moratorium's first condition becomes satisfiable.** Five ADRs stop blocking on a
  type that does not exist. `caro guard` (ADR-036) and `caro-mcp` (ADR-015/038) become
  transport spikes over a settled, importable type.
- **`evidence` becomes true rather than aspirational.** D3 is the difference between a
  field that discloses the model's contribution and a field that hard-codes
  `Deterministic` under `--approval smart`.
- **Caro's deterministic-floor positioning acquires a citation.** Anthropic ships a
  non-negotiable critical-path list beneath its own `auto` classifier. "Deterministic floor
  beneath a probabilistic gate" is now the shipped architecture of the host Caro layers
  under, not a contrarian claim — directly useful to the positioning doc the memo's
  opportunity (E) asks for.
- **`enforcement` prevents a class of false compliance artifact** that would otherwise
  have been generated correctly from a correct payload.
- **Zero dependency delta, one new file, no new module, no runtime behaviour change.**

### Negative / risks

- **D2 takes a name from a dormant feature.** Hardware assessment has been commented out
  since v1.1.0-beta.1 with `#[ignore]`d tests, so the cost is a bead and a redirect rather
  than a migration — but if someone was about to revive `caro assess` for hardware, this
  ADR is the reason they cannot, and they should say so on the PR.
- **D4 departs from the memo's own recommendation.** If the payload turns out to be wrong
  in a way a flag would have contained, the containment is now a version bump and a
  deprecation rather than a flag flip. The argument for accepting that is D4's; it is a
  judgement call and it belongs in the devil's-advocate review.
- **D5 amends a contract one day after it was written.** Small and evidence-forced, but a
  contract that acquires a field per research run is not a contract. This should be the
  last amendment before v2; if a second one arrives, that is a signal the contract needed
  another week, not another field.
- **D3 changes a public function signature.** `blend_smart_decision` is `pub` and the
  5th parameter is a breaking change for any external caller. There is exactly one
  in-tree caller (`src/cli/mod.rs:822`); external callers are unlikely at `1.4.0` but not
  impossible, and this should land in a minor version.
- **The projection table will go stale, and stale projections degrade toward allow.**
  Three point-release version gates appear on one documentation page. The omit-rather-than-
  guess rule contains an *unknown* host; it does not contain a *stale* one. The detector
  is a scheduled CI diff and it is out of scope for this PR. Stated as a known gap.
- **A golden-payload test is a maintenance tax.** Intentional. Field order, casing and
  `reason_code` format are the contract; a byte-for-byte assertion is the cheapest way to
  make a silent change loud.

### Neutral

- Runtime behaviour is unchanged. `src/cli/mod.rs` still branches on
  `requires_confirmation` / `blocked_reason`. Making the decision *authoritative* rather
  than *sayable* remains ADR-020's scope.
- `Subject.cwd` is carried and nothing is computed from it. The target axis is ADR-047's.
- `RiskLevel`'s `rename_all` moves `"lowercase"` → `"snake_case"`. Every variant is a
  single word, so the emitted strings are byte-identical; a test asserts it.

---

## Alternatives considered

**A1 — File this as a bead and skip the ADR, honouring the moratorium literally.**
Rejected, narrowly. D2 takes a name from another feature, D4 overrides a written
recommendation, and D5 amends a contract — three decisions with consequences a future
reader will want the reasoning for. A bead records what; an ADR records why. The moratorium's
target is *design* ADRs that defer implementation, and this ADR's entire content is
implementation. The mitigation is the declared boundary at the top: this is the last one.

**A2 — Ship `Assessment` now and refactor `blend_smart_decision` later.**
Rejected. It requires hard-coding `Evidence::Deterministic`, which is false under
`--approval smart`, in the one field ADR-058 says is the reason it exists. A provenance
field that lies is worse than no provenance field: it converts an absence a reader would
notice into an assertion a reader would trust.

**A3 — Name the verb `caro safety assess` and leave `caro assess` to hardware.**
Rejected. It concedes the shorter, more valuable name to a feature that has been disabled
for three minor versions, and it makes every integration example one word longer at exactly
the moment Caro wants to be pasted into other people's READMEs. The nesting also implies a
`caro safety …` family that does not exist and that nothing in the roadmap creates.

**A4 — Put `enforcement` on `Assessment` rather than on `Projection`.**
Rejected. Enforcement is a property of *this decision at this host's integration point*,
not of the assessment. The same `block` is authoritative as a Claude Code hook and merely
advisory to a gateway that logs and continues. Hoisting it would assert one answer for all
hosts, which is the mistake ADR-058 D7 pushed the whole table down to avoid.

**A5 — Model `enforcement` as a third `lossy` state (`lossless` / `lossy` / `unenforceable`).**
Rejected. The two properties are orthogonal — `block → deny` is lossless and authoritative;
`auto_approve → allow` is lossless and advisory; `async_log → ask` is lossy and advisory.
A single enum over an orthogonal pair makes one of the four combinations unrepresentable,
and it would be the one that matters.

**A6 — Ship the payload together with an MCP transport, per the memo's opportunity (B).**
Rejected, consistent with ADR-058 A6. Transports are covered by three existing ADRs and
each has its own failure surface. The contract is the thing five ADRs block on; it lands
alone. Today's F8 finding — *integrate at the hook, not the `canUseTool` callback, because
auto-approved tools never reach the callback* — is recorded as an input to ADR-036, not
acted on here.

**A7 — Feature-flag the payload as the memo recommends (`--features assessment`).**
Rejected; see D4. The concrete precedent is in the tree: ADR-015 told ADR-020 and ADR-023
to block on `SafetyAssessmentOutput`, and `grep -rn "SafetyAssessmentOutput" src/` returns
nothing. A type behind a flag is a weaker version of the same problem — importable in
principle, absent in every default build a downstream consumer actually has.

---

## References

- [Claude Agent SDK — Configure permissions](https://code.claude.com/docs/en/agent-sdk/permissions) (fetched 2026-08-26) — six-step evaluation order, hook-`allow` qualification, `CLAUDE_SDK_CAN_USE_TOOL_SHADOWED`, critical-path floor, permission modes, subagent inheritance
- [Claude Code — Hooks reference](https://code.claude.com/docs/en/hooks) · [Hooks guide](https://code.claude.com/docs/en/hooks-guide) (via ADR-058, fetched 2026-08-25)
- `docs/adr/ADR-058-caro-assessment-v1-decision-contract.md` — the contract this ADR implements and amends by one field
- `caro-scope-assessment-implementation-2026-08-26.md` — three-phase scope, type definitions, file-by-file diff plan, T1–T15 test matrix, demoware-trap analysis
- `caro-scope-assessment-contract-2026-08-25.md` — F1–F7
- `specs/017-structured-assessment-payload/scope.md` (2026-06-03)
- `market-scans/2026-08-26-ai-agent-strategy-memo.md` §3A, §4 — the ship-it recommendation and the ADR moratorium
- `.claude/rules/validation-discipline.md` · `.claude/rules/good-boy-scout.md` · `.claude/rules/git-workflow.md` · `.claude/rules/adr-numbering.md`
- Caro tree at `1.4.0`, branch `integrator/20260711-postmerge` (line numbers re-verified
  2026-08-26 and expected to drift):
  `src/models/mod.rs:152,189,198,288` · `src/safety/mod.rs:189,252,256,277,292,297,302,318` ·
  `src/cli/mod.rs:76,795,811,822` · `src/backends/mod.rs:56,71,79,86` · `src/main.rs:422` ·
  `src/assessment/mod.rs` · `src/bin/generate-schema.rs` · `tests/assess_integration_test.rs` ·
  `Cargo.toml:41,42,46`
