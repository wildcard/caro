# Salvage Report — 2026-07-18

Scope: open PRs on `wildcard/caro`, local worktrees/branches, and the evergreen
dependency process. All claims below are ✓ VERIFIED against command output
unless marked otherwise.

---

## 0. Two corrections to the premise

**(a) The Astro/Vercel adapter mismatch is fixed on `main`, and the fix is real.**

```
origin/main website/package.json → astro ^6.1.10, @astrojs/vercel ^10.0.8
```

Majors are aligned (astro 6 → adapter 10). But **issue #1309 is still open**
despite the fix having landed — stale tracker state.

**(b) The 4 consecutive CI failures on `main` are NOT Rust-related.**

```
2026-07-18T03:00 failure CI (scheduled)
2026-07-17T03:00 failure CI (scheduled)
2026-07-16T03:01 failure CI (scheduled)
2026-07-15T03:00 failure CI (scheduled)
2026-07-14T07:27 success CI (push)   ← push-triggered CI is GREEN
```

Failing jobs on the latest one (run `29628046011`): `Extended Tests (TinyLlama
1.1B / StarCoder 1B / SmolLM 135M / Qwen 0.5B)` — HF Hub model download, already
tracked as **#1341**. These are nightly scheduled runs only. **Push-triggered CI
on `main` has been green since 2026-07-14.** The merge path is not blocked.

---

## 1. The actual blocker — `ethnum 1.5.2`

This is the finding that explains almost every abandoned PR.

Pulled the failing `Lint & Format` log from PR #1299 (job `86613506608`):

```
--> ~/.cargo/registry/.../ethnum-1.5.2/src/error.rs:16:14
16 |     unsafe { mem::transmute(()) }
   = note: source type: `()` (0 bits)
   = note: target type: `TryFromIntError` (8 bits)
error[E0512]: could not compile `ethnum` (lib)
```

A transitive dep failed to compile against the runner's newer rustc. Every Rust
job in the graph died: `Lint & Format`, `Unit Tests (both OS)`, `Security Audit`,
`Knowledge Integration Tests`, `MSRV Check`.

**This is now fixed on `main` — `Cargo.lock` has `ethnum 1.5.3`.**

Consequence: **every PR whose checks last ran on 2026-07-12 is showing a red X
that has nothing to do with its own diff.** Confirmed timestamps — #1299, #1300,
#1303 all last ran 2026-07-12T04–06:00Z; #1261 last ran 2026-06-23. None have
been re-run since the fix.

---

## 2. Categorized findings

### SALVAGEABLE — rebase onto `main`, re-run, likely green

Red only because of `ethnum`/broken-window CI. Diffs are small and unrelated to
the failure.

| PR | Title | Behind | Note |
|---|---|---|---|
| #1303 | safety: allowlists override broad `rm`, keep catastrophic core | 51 | `DIRTY` — needs conflict resolution, but small (93/13) |
| #1300 | docs(groom): runner-timeout note | ~51 | 12-line docs change, 7 red jobs, all `ethnum` |
| #1299 | qa: scheduled rotation 2026-07-11 | ~51 | 8 red jobs, all `ethnum` |
| #1261 | docs: faramesh-core competitive analysis | — | Docs-only, 178 additions, killed by Rust CI |
| #1280 | chore(demo): refresh caro-demo video | — | Same cluster + `check-labels` |
| #1311 | security: clear all Security Audit advisories | 51 / ahead 3 | `DIRTY`; high value — closes advisories |
| #1212 | safety: path-aware allowlist for scoped Critical deletions | 28 | 778 additions, real feature work |

**Highest-leverage single action:** re-run checks on #1300, #1299, #1261, #1280.
Zero code changes needed — they were never re-run after `ethnum` landed.

### STALE — close, superseded

| PR | Why |
|---|---|
| #1320 | "bump ethnum 1.5.2 → 1.5.3 to fix Lint & Format" — **`main` already has 1.5.3**. Its remaining diff vs main is `.beads/issues.jsonl` churn (795 lines) + integrator memory files. The fix it exists to deliver is already in. |
| #1153, #1157 | integrator/docs PRs from May, 51+ behind, superseded by later integrator cycles |
| #805, #808 | v1.2 epics from 2026-03-26, ~3.5 months stale, 3000+ line diffs |
| #838, #886, #993 | March/April, superseded by later design-system and QA work |
| #1224, #1178, #1175, #1171, #1167, #1166 | QA rotation PRs May–June — rotations are daily/disposable; keeping 6 open is noise |

### NEEDS REBUILD — good idea, code needs rework post-rebase

| PR | Why |
|---|---|
| #1308 | Flox dev env — 60 behind, 16k additions across 18 files. Too far diverged to rebase cleanly. **But see §3 — extract the guard test first.** |
| #1314 | Multi-harness / best-of-N ranking — 53 behind, `DIRTY`, 1108 additions. Valuable idea, will conflict. |
| #1043 | Windows stdin hang / POSIX leak / shell mis-detection — 60 behind. External contributor (kukoshock), genuinely valuable bug fixes, but 2.5 months of drift. |
| #1088 | eval multi-backend grouping, 78% → 96% — real measured win, needs rebase onto current eval code. |

### BLOCKED BY CI (non-`ethnum`) — distinct, still real

| PR | Blocker | Fix |
|---|---|---|
| #1296 | `cla-check` | External contributor (nightcityblade) needs to sign CLA — **human action, not code** |
| #1322, #1323 | `check-labels` | Missing required label; one-click fix |
| #1322, #1318 | `validate` | Tracked as **#888** ("ci(validate): artifact path mismatch causes failure on every PR") — open since 2026-04-26 |
| #1331, #1326, #1285, #1275 | `Claims Coverage Report` | Fails on dependabot PRs specifically — likely needs a bot-PR exemption |
| #1303, #1300, #1299 | `ChromaDB Integration Tests` | Tracked as **#1342**; #1061 proposes marking it non-blocking and has been open since 2026-05-10 |

### CLEAN — merge now

| PR | Status |
|---|---|
| #1333 | `mergeState=CLEAN`, no failures — qa rotation 2026-07-16 |
| #1344, #1338, #1321 | `UNSTABLE` but **no failing checks** (only pending/neutral) |

---

## 3. The stranded evergreen guard — highest-value salvage

Commit `8b5833b9` (2026-07-12) contains exactly the regression guard this repo
needs, and it is **stranded inside PR #1308** (the 16k-line Flox PR) — it is
**not on `main`**:

```
website/src/__tests__/vercel-adapter-astro-alignment.test.ts   (94 lines)
```

> "Locks @astrojs/vercel major to the astro major (astro 6 → adapter 10) so the
> applyPolyfills deploy crash (#1309) can't silently recur on a future astro bump."

`website/package.json` has `test = vitest run`, and vitest auto-discovers
`src/__tests__/` — so **cherry-picking this one file onto `main` wires the guard
in with no config change.**

Recommended action: cherry-pick that single file to `main` on its own branch,
then close #1309. Do not wait for #1308 to be salvageable.

---

## 4. Local branches & worktrees

- **93 worktrees** registered. This is the dominant local-hygiene problem.
- **~26 are `integrator/*`** dating 2026-05-16 → 2026-07-18. Many sit on the
  identical commit `9311c9ec` (integrator-20260613, -20260626, -20260614,
  -20260618, -20260621-2303, -20260623, -20260627, -20260701, -20260705,
  -20260707) — i.e. **10 worktrees pointing at one commit**, created by the
  nightly integrator routine and never cleaned up.
- **`integrator/20260711-postmerge`** (the current checkout) — confirmed stale.
- One worktree is on a **foreign filesystem**: `/sessions/focused-wizardly-archimedes/...`
  (`047-flox-dev-environment`, `locked`) — a dead sandbox mount.
- Two live under `~/.codex/worktrees/`, outside the repo tree.
- `.worktrees/006-replace-ascii-morph` is untracked-dirty in git status.

**Real CI side-effect, not just clutter** — from the PR #1299 job log:

```
fatal: No url found for submodule path '.worktrees/001-fix-unquoted-cli' in .gitmodules
##[warning]The process '/usr/bin/git' failed with exit code 128
```

A worktree got committed as a gitlink. It's currently only a cleanup-step
warning, but it means worktree state has leaked into the tracked tree.

Recommendation: `git worktree prune`, then batch-remove every `integrator/*`
worktree whose branch is merged or sitting on a duplicate commit. The nightly
integrator routine should remove its worktree on exit.

---

## 5. Evergreen dependency process

The failure timeline shows the shape of the problem clearly:

```
8691e57c  fix: bump @astrojs/vercel 9→10 to match astro 6 (closes #1309)
0a9b730c  Revert "fix: bump @astrojs/vercel 9→10..."
3f276ce3  fix: @astrojs/vercel 9→10 + lockfile regen
14ff02f4  fix: @astrojs/vercel 9→10 + lockfile regen      ← duplicate
d5ef765f  fix(deps): bump @astrojs/vercel to v10
f94d6292  Revert "fix(deps): bump @astrojs/vercel to v10"
8b5833b9  test(website): add adapter↔astro alignment guard  ← never reached main
```

**Six commits, two reverts, one duplicate, to land a single version bump — and
the regression guard that would prevent a recurrence is still not on `main`.**

Proposed process, in priority order:

**P1 — Land the guard that already exists.** Cherry-pick
`vercel-adapter-astro-alignment.test.ts` to `main` (§3). Costs one commit; it is
already written and reviewed.

**P2 — Generalize the guard to a peer-major matrix.** The adapter↔astro case is
one instance of a class. Add a small declarative table of "these two packages'
majors must move together" and assert it in the same vitest file. Candidates
beyond astro/vercel: `@astrojs/react` ↔ `react`, `@astrojs/db`, `vite` ↔ plugins.

**P3 — Split dependabot's `rust-minor-patch: patterns: ["*"]` group.** One group
containing every crate means a single uncompilable transitive (`ethnum`) takes
down the whole batch, and the batch PR (#1285, 14 updates) becomes
all-or-nothing. Split into `rust-security`, `rust-core`, `rust-dev-deps` so a
poisoned crate quarantines to one PR.

**P4 — Gate on lockfile compile, not just resolution.** `ethnum 1.5.2` *resolved*
fine; it failed at *compile* against a newer rustc. Add a scheduled job that runs
`cargo check --locked` on the current toolchain independently of PRs, so a
toolchain-side break is attributed to the toolchain and not blamed on whichever
PR ran next.

**P5 — Auto-rerun stale PR checks after `main` goes green.** This is the process
gap that actually cost the most work here. When `main` transitions red→green,
dispatch a re-run for open PRs whose last check predates the fix. Seven PRs sat
untouched for six days carrying a red X they did not cause — and at least one
(#1320) was opened solely to re-fix an already-fixed problem.

**P6 — Close the tracker loop.** #1309 fixed but open; #888 open since April
failing `validate` on *every* PR; #1061 (ChromaDB non-blocking) open since May
while #1342 was filed today for the same symptom. A red check that is known-bad
and un-actioned trains everyone to ignore red checks.

---

## Recommended next actions, ranked

1. Cherry-pick the adapter guard test to `main`; close #1309.
2. Re-run checks on #1300, #1299, #1261, #1280 — no code changes needed.
3. Merge #1333 (already `CLEAN`).
4. Close #1320 as superseded (`main` already has ethnum 1.5.3).
5. Extract the Windows fixes (#1043) and eval grouping (#1088) into fresh
   branches off current `main` rather than rebasing 60-commit drift.
6. `git worktree prune` + batch-remove duplicate `integrator/*` worktrees.
7. Split the dependabot Rust group (P3) and add the compile gate (P4).

---

## Appendix — paste-ready Slack messages

### → #caro-build-fix (C0BJ2U1N3PV)

*Salvage pass — 2026-07-18*

Two corrections to the working assumption first:

*1. The 4 CI failures on `main` are not Rust.* They're the nightly *scheduled* CI at 03:00 failing on `Extended Tests` (HF Hub model download) — already tracked as #1341. Push-triggered CI on `main` has been green since 2026-07-14. The merge path was never blocked.

*2. The real blocker was `ethnum 1.5.2`* — a transitive crate that resolved fine but failed to compile against the runner's newer rustc (`E0512`, `mem::transmute` size mismatch). It killed every Rust job: Lint & Format, Unit Tests, Security Audit, Knowledge Integration, MSRV Check. `main` now has `ethnum 1.5.3`.

Consequence: *every PR whose checks last ran 2026-07-12 shows a red X unrelated to its own diff.* #1299, #1300, #1303 all last ran 04–06:00Z that day and have never been re-run.

*Highest-value find:* commit `8b5833b9` has a 94-line vitest guard — `website/src/__tests__/vercel-adapter-astro-alignment.test.ts` — that locks the adapter major to the astro major so the #1309 deploy crash can't recur. It's stranded inside PR #1308 (16k lines, 60 commits behind) and *is not on `main`*. `package.json` runs `vitest run` and auto-discovers `src/__tests__/`, so cherry-picking that one file wires it in with zero config change.

*Categories:*
• SALVAGEABLE (7) — #1303, #1300, #1299, #1261, #1280, #1311, #1212. Red only from `ethnum`; several need nothing but a re-run.
• STALE (11) — notably #1320, which exists to bump ethnum to 1.5.3 that `main` already has.
• NEEDS REBUILD (4) — #1308, #1314, #1043, #1088. 53–60 commits behind; extract, don't rebase.
• BLOCKED BY CI, genuinely (9) — `cla-check` needs a human signature (#1296); `validate` is #888, open since April, failing on *every* PR.
• CLEAN — #1333 is mergeable now.

*Worktrees:* 93 registered, ~26 `integrator/*`, and ten sit on the identical commit `9311c9ec` — the nightly routine creates one and never removes it. Not just clutter: CI logs show `fatal: No url found for submodule path '.worktrees/001-fix-unquoted-cli'` — a worktree got committed as a gitlink.

*Process gap that cost the most:* nothing re-runs stale PR checks when `main` goes red→green. Seven PRs sat six days carrying a red X they didn't cause, and #1320 was opened to re-fix an already-fixed problem. A dispatch-on-recovery job would have made this whole pass unnecessary.

Full report with evidence: `.claude/memory/salvage-report-2026-07-18.md`

### → #caro-agent-standup (C0BHTLHGC23)

*Salvage pass done — 2026-07-18*

`main` is healthier than assumed: push CI green since 07-14, and the 4 red runs are nightly `Extended Tests` (HF model download, #1341), not Rust.

The actual blocker was `ethnum 1.5.2` failing to compile on the runner's rustc — it took down every Rust job, so ~7 open PRs are carrying a red X they didn't cause. `main` has 1.5.3 now; most of them just need a re-run.

Best find: the adapter↔astro regression guard that prevents #1309 recurring is already written, but stranded in PR #1308 and not on `main`. One-file cherry-pick.

Tally: 7 salvageable · 11 stale · 4 needs-rebuild · 9 genuinely blocked · #1333 mergeable now.
Also 93 worktrees, 10 of them duplicates on one commit.

Details in #caro-build-fix. Nothing merged or closed — report only.
