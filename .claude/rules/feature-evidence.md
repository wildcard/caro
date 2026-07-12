# Feature Evidence, Demo, and Regression Guard

**APPLIES TO**: Every feature PR (feat:), every user-visible fix (fix:)
that changes behavior, and every safety-pattern change.
**DOES NOT APPLY TO**: docs, refactors with no behavior change, CI
config, dependency bumps (those are covered by tests that already exist).

Codified 2026-07-12 under the autonomous-operations directive (see
[`docs/decisions/2026-07-12-autonomous-mode-release-scope.md`](../../docs/decisions/2026-07-12-autonomous-mode-release-scope.md)).
The project is developed by multiple agents in parallel with a
preoccupied owner; the only durable trust signals are evidence, demos,
and regression guards — not PR prose.

## The Three Artifacts

Every in-scope PR must carry, in its body or linked from it:

### 1. Evidence — "it works"

- The CI run(s) proving the change green (link the workflow run, not
  just "CI passed").
- For behavior changes: before/after output, captured from a real
  invocation (`cargo run -- "<query>"`, `--dry-run` is fine).

### 2. Demo — "here is how you see it work"

- A runnable command sequence a reviewer (or the weekly demo report)
  can copy-paste, with expected output. Three lines is enough.
- Demos aggregate weekly: the Monday planning routine compiles every
  merged feature's demo into `docs/demos/<date>-weekly-demo.md`.

### 3. Regression guard — "it cannot silently break"

- A test that fails if the feature regresses: e2e (`tests/e2e_*.rs`),
  contract (`tests/*_contract.rs`), or unit where the surface is
  internal. Name the test in the PR body.
- Safety-relevant changes MUST use the TDD flow in
  `skill: safety-pattern-developer` — the guard comes first.
- Past features stay guarded: never delete or `#[ignore]` an existing
  contract/e2e test to make a new feature pass. If a guard genuinely
  must change, the PR body explains why the old pinned behavior is
  wrong, and the change gets its own commit.

## Weekly Demo Report

The Monday planning routine (routine `trig_01KTFtDwFfs4xHiJ2JVbCgEV`)
produces `docs/demos/<date>-weekly-demo.md` containing, for every
feature merged that week: what shipped, the demo commands, the evidence
links, and the regression guard's test name. A week with no feature
merges still gets a one-line report ("no feature merges this week")
so absence of a report always means the routine failed, not that
nothing happened.

## Reviewer checklist (agents included)

- [ ] PR body links a green CI run
- [ ] PR body contains a copy-pasteable demo (or links one)
- [ ] PR body names the regression-guard test
- [ ] No existing contract/e2e test deleted or ignored without its own
      justified commit

A PR missing these gets the fixes pushed (Good Boy Scout) or, if the
author context is gone, a `[agent]` comment naming exactly which of the
three artifacts is missing.

## See also

- `.claude/rules/validation-discipline.md` — gates *whether* to build;
  this rule gates *how* it lands
- `.claude/rules/release-version-alignment.md` — the same
  checklist-as-grep pattern at release time
- `docs/demos/` — the weekly demo reports this rule feeds
