# Decision Record — Autonomous Mode, Release Scope, and Weekly Demos

**Date**: 2026-07-12
**Decider**: Autonomous session (claude-sonnet-4-6), operating under the
owner's explicit "choose for me" directive — the owner is preoccupied and
cannot review/approve/steer. Every decision below records the alternatives
considered so the owner (or a future agent) can revisit with full context.
**Status**: Accepted, executed this session unless marked otherwise.

---

## D1 — Ship unreleased work as v1.5.0 now; v2.0.0 stays gated

**Decision**: Cut a **v1.5.0 minor release** from the current `main`
`[Unreleased]` CHANGELOG content. Do **not** ship a release labeled
v2.0.0 at this time.

**Context**: v1.4.0 shipped 2026-05-09 — two months of merged, tested work
(custom safety patterns via TOML, MSRV 1.85, static-matcher P0 fix,
RUSTSEC fixes, catastrophic-floor hardening once #1246 merges) is sitting
unreleased. Meanwhile the "v2.0.0 - Distributed Autonomy" milestone (due
Jun 30, now overdue) is 61.7% complete by issue count but **0/5 on its
defining features** (Karo, Dogma, Voice, Self-Healing, Local Context
Indexing) — all blocked at Gate 1 of
[`validation-discipline.md`](../../.claude/rules/validation-discipline.md)
with 0/20 discovery transcripts each.

**Alternatives considered**:

- **(A) Ship the completed milestone items as "v2.0.0"** (the option the
  2026-07-06 weekly report called Option B). Rejected: the unreleased
  delta contains **no breaking changes**, so a major bump violates
  semver; and shipping a release named "Distributed Autonomy" containing
  zero distributed-autonomy features misrepresents the release to users.
- **(B) Hold everything until the 5 core features clear validation.**
  Rejected: discovery hasn't started; that path leaves shipped-quality
  fixes (including a safety regression fix) unreleased indefinitely.
- **(C) Ship v1.5.0 now, keep the v2.0.0 milestone for the validated
  distributed-autonomy features with a reset target date.** **Chosen.**
  Users get two months of fixes; semver stays honest; the milestone keeps
  its identity; no milestone surgery needed.

**Consequences**: ROADMAP gets a v1.5.0 milestone entry; the v2.0.0
milestone's target date needs a reset (owner input preferred; absent
that, next planning run proposes Q4 2026). The 3 unassigned issues
(#1151, #1152, #1075) should still be added to milestone #3 — needs
`gh`/API access this environment lacks; left as a follow-up.

## D2 — Merge PR #1246 despite the red Vercel context

**Decision**: Merge #1246 once the Rust/CI checks are green, treating the
failing `Vercel – caro-foss-website` status as non-blocking.

**Rationale**: The Vercel failure is a documented pre-existing breakage
(Astro 5→6 multi-package migration) that exists on `main` itself and on
every PR; #1246's body root-causes it and correctly scopes it out. All
Rust checks (unit, smoke, safety regression, clippy, builds ×3 targets,
security audit, CodeQL) passed on the PR. Waiting on Vercel green would
block every PR forever.

**Alternatives**: (A) Fix the Astro 6 migration first — rejected as an
unbounded multi-package migration inside a CI-repair critical path;
tracked separately. (B) Leave main red — rejected; red main blocks the
release gate and every downstream agent.

**Follow-up**: The Astro 5→6 migration needs its own tracked issue and
owner (see weekly demo report's "known red" section).

## D3 — Local `cli_interface_contract` failures are environmental, not release blockers

**Decision**: The 6 failures observed running `cargo test` in this
sandbox (`test_basic_command_generation_flow`, `test_concurrent_cli_usage`,
`test_context_awareness`, `test_error_handling_graceful`,
`test_output_format_options`, `test_verbose_mode` in
`tests/cli_interface_contract.rs`) are classified **environment-specific**
(sandbox lacks the embedded model / network the tests exercise) and do
not gate the release.

**Evidence**: The same test suite passes in GitHub Actions on
ubuntu-latest and windows-latest (PR #1246 run 27532336756, Unit Tests
green on both). CI is the release gate of record, not the sandbox.

**Follow-up**: If a future run shows these red **in CI**, that's a real
regression — do not inherit this classification.

## D4 — Weekly demo becomes a standing deliverable of the Monday routine

**Decision**: Every weekly planning run (Mondays, routine
`trig_01KTFtDwFfs4xHiJ2JVbCgEV`) must also produce a demo report at
`docs/demos/<date>-weekly-demo.md` covering every feature merged that
week, each with **evidence** (PR + CI run links), a **demo** (runnable
commands and expected output), and its **regression guard** (the e2e/
contract test that pins the behavior). Codified in
[`.claude/rules/feature-evidence.md`](../../.claude/rules/feature-evidence.md)
so any agent picking up the routine inherits the obligation.

**Alternatives**: (A) A separate scheduled routine just for demos —
rejected: session-scoped cron doesn't survive this sandbox, and a second
routine doubles surface for drift; the Monday routine already reads the
right state. (B) Demo videos via `caro-demo-video` — deferred: higher
production cost; the Remotion pipeline exists but a written, runnable
demo doc is the durable minimum. Videos can layer on top later.

## D5 — Fully-automated operation protocol

**Decision**: Until the owner returns, sessions operate in
choose-for-me mode: proceed without approval on reversible,
goal-aligned actions; record every non-obvious choice in
`docs/decisions/` with alternatives; never bypass Tier-1 rules
(feature branches, no direct commits to main) or the safety floor
(no weakening of `src/safety/` patterns without a human safety owner).
Permission-prompt reduction is an environment setting the owner controls
(remote sessions run with the permission mode chosen at environment
creation) — agents cannot and should not self-escalate; instead they
prefer tool paths that are already allowlisted.

**Hard limits that still require a human** (documented, not waived):
crates.io token rotation, GitHub milestone/settings surgery (no API
access from this environment), production secret changes, and any
relaxation of safety patterns.

---

*Next review: first session after the owner returns, or the next weekly
planning run, whichever comes first.*
