# ADR-016: Ponytail Pragmatic-Skeptic Reviewer (Additive Adoption)

**Status**: Accepted

**Date**: 2026-06-14

**Authors**: Caro maintainers

**Target**: Community

## Context

We evaluated the external project
[`DietrichGebert/ponytail`](https://github.com/DietrichGebert/ponytail)
(MIT, ~6.8k stars, v4.2.0) to decide whether and how to incorporate it.

Ponytail is a Claude Code / multi-agent **skill** (JavaScript + markdown
configuration — not a Rust dependency) that injects a "lazy senior
developer" decision ladder before any code is generated:

1. Does this need to exist? (YAGNI)
2. Does the standard library do it?
3. Is there a native platform feature?
4. Is it already an installed dependency?
5. Can it be one line?
6. Only then: minimal viable implementation.

It ships as a **default-on, persistent** skill with behaviors including
"deletion over addition", "fewest files possible", "ship code first,
tests only when asked", three intensity levels (lite / full / ultra),
and `/ponytail-review` + `/ponytail-audit` commands. It advertises
80–94% less code, 3–6× faster, and 47–77% cheaper output — self-reported
across five toy tasks (email validator, debounce, CSV sum, countdown
timer, rate limiter) on three Claude models, median of ten runs.

**Forces at play:**

- The core *idea* — bias toward the smallest solution that works — is
  genuinely valuable and already aligns with our
  [`good-boy-scout`](../../.claude/rules/good-boy-scout.md) rule (KISS,
  "don't gold-plate", "pragmatic over perfect").
- Several ponytail *defaults* directly conflict with Caro's constitution:
  - "Tests only when asked" vs. our safety-pattern **TDD mandate** and
    "add tests for new functionality" code style.
  - "Deletion over addition / fewest files" vs. Caro's deliberate
    modular layout (`backends/`, `safety/`, `platform/`, …).
  - Default-on persistence vs. our **opt-in** skill/agent model.
  - Marketing-grade benchmark claims are exactly the kind of evidence
    [`validation-discipline`](../../.claude/rules/validation-discipline.md)
    teaches us to distrust (no defended cohort, tiny task set,
    vendor-reported).
- The maintainer steer was explicit: **be additive, do not regress or
  remove any existing practice.** Introduce the skeptical perspective
  without letting it overpower the design and development process.

## Decision

**Do not install or adopt the upstream ponytail skill.** Instead,
harvest its valuable idea additively as a new **read-only, on-demand
reviewer persona** inside Caro's existing agent/skill system:

- A new agent, `.claude/agents/ponytail-reviewer.md`, modeled on the
  existing [`devils-advocate`](../../.claude/agents/devils-advocate.md)
  agent (read-only, evidence-driven, never-blames, 6-section structured
  output). It interrogates a change for **over-engineering and
  over-strictness** — the deliberate complement to devils-advocate, which
  interrogates **under-validation**.
- A thin invocation skill, `.claude/skills/ponytail-review/SKILL.md`
  (`/ponytail-review`), that spawns the agent over the current diff.
- A one-line "See also" cross-reference added to `good-boy-scout.md`
  linking the passive rule to its new active companion.

The reviewer applies ponytail's ladder (YAGNI → stdlib → native →
installed → one-liner → minimal) as a **review lens, never a generation
mandate**, and treats safety, security, accessibility, data-loss
handling, the tests that guard them, and the release/ADR process as
**hard carve-outs it never suggests trimming** ("lazy, not negligent").

## Rationale

- **Additive, not disruptive.** Adding a critical voice changes nothing
  about how Caro designs or builds; it adds one more perspective. A
  proposal can now be pressed from two opposite directions — *is it
  worth building?* (devils-advocate) and *is it bigger than it needs to
  be?* (ponytail-reviewer).
- **Read-only neutralizes the conflict.** Because the agent only flags
  and never applies, none of ponytail's risky defaults ("delete",
  "skip tests") can actually mutate the codebase. The human or parent
  agent decides.
- **Carve-outs preserve the constitution.** Encoding the safety/TDD/a11y/
  data-loss/release lines as explicit non-negotiables means the skeptic
  operates strictly inside Tier 1–2 rules.
- **Consistent with our own evidence discipline.** We adopt the
  *technique* without importing the *claims*; any future product-level
  application is gated through our eval harness (see Future Work).

## Consequences

### Benefits

- A concrete, on-demand counter-pressure to AI's bias toward addition.
- Symmetry with `devils-advocate`: two opposing critical axes.
- Zero behavioral change to existing rules, agents, or build process.
- No new runtime dependency, no license entanglement, no binary impact.

### Trade-offs

- Another agent/skill to maintain and keep aligned with the constitution.
- The reviewer is advisory only; its findings have no enforcement teeth
  by design (this is intentional, but means value depends on people
  actually invoking it).

### Risks

- **Risk**: the reviewer suggests trimming something safety-critical →
  **Mitigation**: explicit carve-out section (§4 of its output contract)
  plus the skill's guardrail to discard any carve-out-violating objection.
- **Risk**: skeptic-creep into "skip the tests" culture → **Mitigation**:
  the agent is forbidden from arguing to skip tests/safety; it may only
  flag a test as redundant *with another test*, never absent.
- **Risk**: over-zealous nit-picking → **Mitigation**: the contract caps
  objections (2–6), legitimizes "Keep as-is", and states shorter is not
  always simpler.

## Alternatives Considered

### Alternative 1: Install upstream ponytail as a default-on skill
- Description: Adopt the marketplace plugin directly.
- Pros: Zero authoring effort; gets the commands for free.
- Cons: Default-on persistence, "tests only when asked", and
  deletion-as-mandate conflict head-on with the constitution; importing
  unverified benchmark claims; gives an external, mutating agent edit
  authority over a safety-critical codebase. **Rejected.**

### Alternative 2: Do nothing; rely on the existing good-boy-scout rule
- Description: Treat the KISS principle as already covered.
- Pros: No new surface.
- Cons: A passive rule in a file is not an active voice on a specific
  diff; AI agents drift toward addition regardless. The whole value is
  making the simplicity question concrete and on-demand. **Rejected.**

### Alternative 3: Fold the lens into devils-advocate
- Description: Extend the existing adversarial agent.
- Pros: One fewer agent.
- Cons: Conflates two opposite critical axes (under-validation vs.
  over-building); muddies devils-advocate's tight validation-discipline
  scope. Keeping them separate keeps each sharp. **Rejected.**

## Implementation Notes

- `+ .claude/agents/ponytail-reviewer.md` — the agent (read-only,
  6-section contract, carve-outs, `model: sonnet`).
- `+ .claude/skills/ponytail-review/SKILL.md` — `/ponytail-review`
  invocation wrapper over `git diff`.
- `~ .claude/rules/good-boy-scout.md` — single additive "See also" line.
- `~ docs/adr/README.md` — add the ADR-016 row (sequential per
  [`adr-numbering`](../../.claude/rules/adr-numbering.md)).
- Verify by spawning the agent on (a) a non-safety website/docs diff —
  expect a full 6-section critique — and (b) a `src/safety/patterns.rs`
  change — expect §4 to refuse trimming the pattern or its test.

## Success Metrics

- Reviewer produces all six sections and a verdict on a real diff.
- Reviewer never proposes trimming a carve-out surface (manually spot
  -checked on a safety diff).
- Adoption is opt-in: no existing workflow changes behavior unless a
  user invokes `/ponytail-review` or spawns the agent.

## Future Work (deferred — not part of this ADR's change)

Ponytail's "native / stdlib / one-liner before custom code" ladder maps
onto a *product* idea: Caro's command generation could prefer the
simplest, most-portable single POSIX command. This belongs in the
embedded prompt (`src/prompts/command_templates.rs`) and must be run as a
**`prompt-tuner` experiment measured by the existing eval harness**
(`src/eval/`, `cargo run --bin caro-eval`) — adopted only if our own
numbers improve, never on ponytail's self-reported benchmarks.

## References

- [`DietrichGebert/ponytail`](https://github.com/DietrichGebert/ponytail)
  — the project evaluated.
- `.claude/agents/devils-advocate.md` — the opposite critical axis this
  agent complements.
- `.claude/rules/good-boy-scout.md` — the passive principle operationalized.
- `.claude/rules/validation-discipline.md` — why we did not import the
  benchmark claims.
- `.claude/rules/adr-numbering.md` — sequential numbering discipline.

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-14 | Caro maintainers | Initial draft, Accepted |
