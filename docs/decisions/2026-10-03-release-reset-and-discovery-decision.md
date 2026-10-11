# Decision Record — Release Reset and Discovery-Rule Fate

**Date**: 2026-10-03
**Decider**: Autonomous follow-up session (claude-opus-4-7), continuing
the 2026-07-12 autonomous-ops protocol. Owner (Kobi) remains preoccupied;
this record names the choices and their trade-offs so the owner can pick
in minutes rather than start from zero.
**Status**: Options stated, **not** executed. The follow-up session
shipped the release-side alignment under option B (fresh v1.5.0 from
HEAD, no tag) as a reversible staging step; every discovery option
remains open and owner-only.

---

## Context — what the four-month audit found

The 2026-05-25 adoption of the founder's-playbook rules
([`validation-discipline.md`](../../.claude/rules/validation-discipline.md),
[STAGE_MAP](../../playbook/STAGE_MAP.md), discovery-debt epic #1188)
reconciled Caro's self-description with its evidence. Four months later
(2026-10-03) a parallel audit against the current repo surfaced two
systemic gaps the 2026-05-25 reconciliation did not predict:

### Gap 1 — Release reset drift

- **2026-07-12**: autonomous session cuts `chore(release): v1.5.0`
  (commit `6f23d37`), bumping `Cargo.toml`, `Cargo.lock`,
  `README.md`, `homebrew-tap/README.md`, and `nuget/tools/install.ps1`
  to 1.5.0. CHANGELOG `[Unreleased]` consolidated into
  `## [1.5.0] - 2026-07-12`.
- **The owner-only step never ran**: tag `v1.5.0`, push, let the
  `publish.yml` workflow upload to crates.io, chain
  `release.yml` for binary assets. The decision doc for 2026-07-12
  (`docs/decisions/2026-07-12-autonomous-mode-release-scope.md`)
  explicitly scoped the tag out of the autonomous session's authority.
- **2026-07-12 → 2026-10-03**: twelve PRs land on `main` on top of the
  bump:
  - #1489 — eval consensus risk labels, Pareto view, ECE release gate
  - #1459 — Jev gap analysis + calibration metrics + typed decisions
    + measured-confidence gates + constrained decoding
  - #1470 — enforce declared limits (executor timeout, guard hooks,
    loop budgets, eval CI) + google/ax research
  - #1488 — reqwest 0.12 / wiremock 0.6 (RUSTSEC-2026-0258, -0285)
  - #1352 — i18n loader fix + Hebrew translation overhaul
  - #1351 — docs-site + storybook to Cloudflare Pages
  - `4d7a855` — vercel adapter astro-drift guard test
  - #1498 / #1505 — pin Lint & Format toolchain to Rust 1.98.1
  - #1497 — accept every `config show` key in `config set/get`
  - #1503 — `ai --once` with trailing prompt no longer blocks on stdin
  - #1487 — static-matcher Pattern 43 current-dir qualifier
  - #1244 — ponytail pragmatic-skeptic reviewer agent
- **Consequence**: crates.io is pinned at v1.4.0 (shipped 2026-05-09),
  nearly five months old. The repo state says 1.5.0, so every
  install-script user and crates.io user sees a different version from
  everyone who checked out `main` since 2026-07-12.

### Gap 2 — Discovery-rule inertia

- **2026-05-25**: `.claude/rules/validation-discipline.md` is adopted
  (PR #1174). Gate 1 requires 20 first-hand interview transcripts per
  new product line / major feature spec; Gate 4 requires a
  devils-advocate review on any AI-drafted spec.
- **2026-05-31**: discovery-debt epic #1188 is filed, with sub-issues
  #1189 (Karo), #1190 (Dogma), #1191 (Voice), #1192 (Self-healing),
  #1193 (Local Context Indexing). PR #1223 ships the interview script
  (`docs/discovery/transcripts/interview-enterprise-dashboard.md`).
- **2026-05-31 → 2026-10-03** (18 weeks): **zero** touches on #1188
  or any sub-issue. `docs/discovery/transcripts/` contains **zero**
  transcripts. **Zero** devil's-advocate reviews filed. The
  interview script has never been used.
- **Observation**: the rule bit its own author (Kobi wrote the rule
  and is also the sole person who would interview enterprise
  customers). An evidence requirement imposed on the owner without a
  mechanism for the owner to meet it inside their actual time budget
  will be ignored, not fought. This is not a failure of will; it is
  a design error in how the rule was shipped.

---

## D1 — Release reset

**Question**: With `Cargo.toml` sitting at 1.5.0 (bumped 2026-07-12,
never tagged) and twelve PRs of new work on top, how does the project
reconcile the repo state with what's actually published?

### Option A — Re-ship v1.5.0 at its 2026-07-12 content, as planned

Rewind the CHANGELOG `[Unreleased]` work to `[1.5.1]` (or hold it), tag
`v1.5.0` at commit `6f23d37` (or the latest commit matching that
scope), let `publish.yml` push the 2026-07-12 content to crates.io.

- **Trade-off**: honours the plan the 2026-07-12 session wrote, keeps
  that CHANGELOG entry verbatim.
- **Cost**: the tagged commit (`6f23d37`) is three months old; CI on
  it may no longer be green (deps drifted). Users who install today
  see July's binary — not what `main` has been since July.
- **Verdict**: slow, revisionist, and still leaves the 12 recent PRs
  unreleased.

### Option B — Cut fresh v1.5.0 from current HEAD (**staged by this session**)

Keep `Cargo.toml` at 1.5.0, consolidate CHANGELOG `[Unreleased]` +
`[1.5.0] - 2026-07-12` into a single `[1.5.0] - 2026-10-03` entry that
honestly reflects everything shipping, update ROADMAP's Last-Updated
and status row, verify the install-script defaults still match. The
companion release PR (filed by this session) does exactly this,
**without** pushing the tag — the owner still runs the final step:

```
git tag -a v1.5.0 -m "v1.5.0" <merge-sha>
git push origin v1.5.0
```

- **Trade-off**: CHANGELOG now reads honest (what crates.io v1.5.0
  will actually contain = four months of work, not two). Keeps the
  version at a semver-honest minor bump (nothing in the twelve PRs
  is a user-visible breaking change — the deps upgrade is
  internal-only, calibration metrics are additive, `ai --once`/
  `config` fixes restore documented behavior).
- **Cost**: discards the 2026-07-12 CHANGELOG entry as a historical
  artefact. The repo keeps `6f23d37` in `git log` for anyone who
  wants to see the earlier scope; `docs/decisions/2026-07-12-...` is
  unchanged.
- **Verdict**: lowest friction to a crates.io release that matches
  reality. **Recommended** as the path, but the owner authorizes it
  by pushing the tag.

### Option C — Codify "shipped-under-1.4-forever"

Revert `Cargo.toml`, `README.md`, `homebrew-tap/README.md`,
`nuget/tools/install.ps1` back to 1.4.0 and stop shipping minor
releases. All future improvements become `1.4.0+<n>` build metadata
or a one-way jump to v2.0.0.

- **Trade-off**: honest about the project's actual release cadence
  (one release in 2026, perhaps). No more drift between repo state
  and crates.io.
- **Cost**: nukes semantic versioning, confuses every downstream
  consumer, forces every future bump to be a major-only hop.
  Reversing this would itself need a release-reset decision.
- **Verdict**: not recommended; written here for completeness because
  "stop pretending we release minors" is a real position some
  projects take.

### What the follow-up session actually did

Staged option B in a release PR titled
`chore(release): v1.5.0 (reset from 2026-07-12 revert)`.
The PR touches only the six files listed in
[`release-version-alignment.md`](../../.claude/rules/release-version-alignment.md),
leaves `src/safety/` untouched (Tier-1 rule), and does **not** push a
tag. The owner's one-line decision on this doc moves it from staged
to shipped (merge + push tag) or discards it (close PR, choose A or
C).

---

## D2 — Discovery-rule fate

**Question**: What do we do with
[`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md)
and the open discovery-debt epic (#1188 + #1189–#1193) after 18 weeks
of zero activity?

### Option A — Resource it

Kobi commits to one 25-minute interview per week (one person, one
transcript, one entry in `docs/discovery/hypothesis-ledger.md`). The
script already exists (`interview-enterprise-dashboard.md`). The
transcripts queue up — Gate 1 for the enterprise-dashboard product
line hits 20 after ~20 weeks. The rule stays as written.

- **Trade-off**: honours the rule, slowly produces the evidence it
  requires, keeps every downstream validation claim honest.
- **Cost**: ~25 min/week owner-time, indefinitely. Must survive
  context switches, travel, every other founder demand.
- **When it works**: if the enterprise-dashboard product line is in
  the owner's current roadmap and the interviews feed decisions the
  owner is already making this quarter.
- **When it doesn't**: if 25 min/week against this specific line is
  not actually a priority — then rebalancing hopes against reality
  is Option C's job.

### Option B — Retire the epic as `not_planned`

Close #1188 and sub-issues #1189–#1193 with `state_reason: not_planned`
and the note "no interview capacity this cycle; rule stands for future
product lines but current debt will not be retired by interview, it
will be retired by graduating these feature slots to not-shipped
status". `playbook/STAGE_MAP.md` already names v2.0 lines as
Stage-1-unvalidated.

- **Trade-off**: honest. The 18 weeks of inactivity are evidence that
  these feature slots are not in flight; closing the issues reflects
  that.
- **Cost**: whatever political signal "closing issues as `not_planned`"
  sends. In practice: the owner is the only reader, and the signal is
  "stop reading these as commitments".
- **Verdict**: lowest-friction tidy-up. The rule itself stays active —
  future product-line specs still have to pass five gates — but the
  historical debt is retired rather than carried.

### Option C — Pause with a hard 30-day deadline

File a decision doc dated 2026-10-03 that says: "the discovery epic is
on hold until 2026-11-03. If interviews #1 of 20 has not been
conducted by 2026-11-03, Option B triggers automatically and the
epic closes." Set a `send_later` or routine fire for 2026-11-03 that
re-opens this decision doc with the retire-or-resource question.

- **Trade-off**: forcing function. Makes the no-decision explicit.
- **Cost**: adds a scheduled trigger that fires in 30 days with the
  same question. If the owner was unable to interview in 18 weeks,
  the probability that 30 more days changes the equation is low —
  so this is really Option B with a delay.
- **When it works**: if the 18-week gap was circumstance (travel,
  context, another priority) and the owner expects it to clear in a
  month.
- **When it doesn't**: if the gap was design (the rule asks for time
  the owner structurally doesn't have), in which case 30 days is a
  reheated no.

### What this session did not do

- **Did not send discovery outreach.** The 2026-05-25 rule names the
  owner as the only person who can conduct Gate-1 interviews.
- **Did not retire the epic.** Option B is the owner's call. The
  reasoning is public here, and `COMPANY.md`'s decision log carries
  the pointer, but no issue was closed.
- **Did not pick an option** on behalf of the owner. The three
  trade-offs are stated; the owner reads them in 90 seconds and
  marks one.

---

## How to decide (owner path)

A one-line reply on either the release PR or this doc resolves both
questions:

- **D1**: "ship B" → merge the release PR + run the tag command the
  PR body documents. "ship A" → close the PR, I'll re-cut from
  `6f23d37`. "ship C" → close the PR, I'll draft the revert PR.
- **D2**: "resource" → open #1188, add a weekly interview routine.
  "retire" → close #1188 + sub-issues as `not_planned`. "pause 30
  days" → file the pause doc + schedule the 2026-11-03 wake.

Both questions are **owner-only** because the publishing step and
the discovery-outreach step both require authority this session does
not hold.

---

## See also

- [`.claude/rules/release-version-alignment.md`](../../.claude/rules/release-version-alignment.md)
  — the 6-file checklist option B followed
- [`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md)
  — the rule D2 is about
- [`docs/decisions/2026-07-12-autonomous-mode-release-scope.md`](./2026-07-12-autonomous-mode-release-scope.md)
  — the earlier autonomous-ops decision this one continues
- [`playbook/STAGE_MAP.md`](../../playbook/STAGE_MAP.md) —
  reconciled 2026-10-03 with the "Failure modes observed" section
  this doc expands
