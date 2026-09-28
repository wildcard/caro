# Good Boy Scout Rule

**Principle**: Leave code in better shape than you found it — but stay pragmatic.

## Core Principles

### No Blame
CI failures are everyone's responsibility. When you find a broken test, a failing check, or a config error — fix it. No need to ask who caused it or why. Just fix it.

### Leave It Better
Every PR is an opportunity to improve. If you touch a file, clean up obvious issues in it. If you're near broken code, fix it. Small improvements compound into healthy codebases.

### KISS — Keep It Simple, Stupid
- Prefer the simplest solution that works
- Don't over-engineer fixes
- A one-line config change that turns red to green is better than a refactor
- Avoid adding abstraction layers when a direct fix suffices

### Pragmatic Over Perfect
- Get things working first
- Perfect is the enemy of good
- A passing CI is better than a beautiful failing CI
- Ship fixes quickly; refine later if needed

## Stay in Your Lane: Hand Off, Don't Detour

"Fix it, no blame" applies to what your task already touches. Anything else
you notice is someone else's job, and you must make sure someone gets it:

- **In your diff's files and trivial** (a typo, a missing flag, a one-line
  config): fix it in the same PR.
- **Anywhere else, or not trivial:** search open issues first. If none
  exists, file one. Add an entry to
  [`.claude/memory/broken-windows.md`](../memory/broken-windows.md), mention
  it in your PR body, and **go back to your task**.
- **Security-policy or release decisions** (audit suppressions, safety
  patterns, dependency major bumps, CI gates): always hand off and mark
  `Needs human: yes`. Never apply them to unblock your own PR.
- **Red CI caused by something outside your diff:** say so once on your PR
  with the issue link, then stop. Don't fix `main` from a feature branch.

The **caro broken-window sweep** routine (`trig_01Mus5RzYBtVoS7B8v2uDTjw`,
daily 15:30 UTC, fresh session per run) dedups the issues, syncs the register,
and fixes one `Needs human: no` item per run. The register is how it finds them.

## Triage by Mission-Criticality

Not all failures are equal. Fix in this order:

1. **Blocking failures** — prevent PRs from merging, affect all branches
2. **Security issues** — vulnerabilities in dependencies, audit failures
3. **Build failures** — code doesn't compile
4. **Test failures** — functionality is broken
5. **Lint/format** — style issues, easy wins
6. **Config issues** — CI workflow misconfigurations
7. **Aspirational failures** — jobs referencing non-existent code (disable them)

## Don't Gold-Plate

- Fix what's broken; don't refactor what works
- If it ain't broke, don't fix it
- Resist the urge to rewrite working code while fixing a nearby bug
- Scope your changes to what's needed, not what could be improved

## The Boy Scout Campsite Rule

> "Always leave the campground cleaner than you found it."

Applied to code:
- Fix a typo you notice? Do it.
- See a missing `--check` flag in CI? Add it.
- Find a workflow referencing a non-existent test? Comment it out.
- Spot a formatting issue? Run `cargo fmt`.

But don't camp out all day cleaning — leave it cleaner than you found it and move on.

## See Also

- [`.claude/agents/ponytail-reviewer.md`](../agents/ponytail-reviewer.md) —
  the active, on-demand companion to this passive rule. Where this rule
  *states* the KISS / "don't gold-plate" principle, the ponytail reviewer
  *applies* it to a specific diff: a read-only skeptic that flags
  over-engineering (surplus code, abstractions, dependencies, ceremony)
  without ever trimming safety, security, accessibility, data-loss
  handling, or the tests that guard them. Invoke via the `ponytail-review`
  skill. Rationale: [`docs/adr/ADR-016`](../../docs/adr/ADR-016-ponytail-pragmatic-reviewer.md).
