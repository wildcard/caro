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
