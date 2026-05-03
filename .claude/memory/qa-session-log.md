# caro-qa-agent — Session Log

**Last updated**: 2026-05-03 by caro-qa-agent
**Reading order**: most recent first.

This is the append-only chronicle of every QA pass. Each entry records the rotation slots run, what was tested, what was found, and where findings were filed. Other agents reading this file: scroll to the top for the latest state; the headline tells you whether the project is in a clean or known-flaky state right now.

---

## 2026-05-03 — Session resume + Slot A + Slot C #9 (safety strict mode)

**Trigger**: Manual resume by user after ~7-day quiet period. Daily remote routine [trig_01Tk7DxyXV7LeYcFjgmTG1mZ](https://claude.ai/code/routines/trig_01Tk7DxyXV7LeYcFjgmTG1mZ) has been firing daily but exiting clean — bootstrap PR [#886](https://github.com/wildcard/caro/pull/886) has not yet merged, so the routine cannot find these memory files on `main`. ~7 no-op runs lost.
**Rotation**: A (smoke) + C (random past feature, surface #9 = safety strict mode).
**Local CLI under test**: rebuilt from latest `main` (29c3ce9, 2026-04-30) into `target/release/caro` v1.3.0. Build flags: `--no-default-features --features embedded-cpu` per `release-version-alignment.md` example.

### Project state changes since 2026-04-26 (gap-fill)

| Change | Detail |
|---|---|
| **caroml shipped (#893, merged 2026-04-29)** | New `caro check` subcommand visible in `--help`. Validates `.caro` task files (parser + AST + lock format). Added to coverage matrix as surface #15. |
| **caro-terminal proposed (#1009-#1023, 2026-04-30)** | 15 issues for a Ghostty-backed desktop GUI. Planning/foundation only — no shipped binary. Marked out-of-scope until first user-facing release. |
| **Sibling QA routine: `caro-frustrated-beta` (PR #910, merged 2026-04-27)** | Daily 5 AM cron, single frustrated-power-user persona, tests website-advertised queries against the binary. Files findings under `.claude/beta-testing/runs/<date>/`, labels `qa-routine` + `frustrated-beta` + `bug-intent-dropped` + `website-broken-promise`. Issue [#947](https://github.com/wildcard/caro/issues/947) is its first P0 finding. **Coordination plan** added to coverage-matrix header — my rotation no longer picks website-promise tests since frustrated-beta covers them. |
| **i18n eval coverage closed (#1027, 2026-04-29)** | Locale parity around the original PR #874 regression class is being actively exercised by an eval suite now. |
| **Pull-request #886 status** | OPEN, MERGEABLE, mergeStateStatus UNSTABLE. 50 checks ran; 2 failures (`ChromaDB Integration Tests`, `Security Audit`) are repo-wide on main, not specific to this docs-only PR. **Action**: rebased on origin/main (1 commit ahead), refreshed memory, will push and add a status comment asking for merge so the daily routine can start doing real work. |

### Slot A — Smoke

| Check | Outcome | Detail |
|---|---|---|
| `cargo build --release --no-default-features --features embedded-cpu` | PASS | 49.8s incremental rebuild on top of warm target/. 4.7 MB binary. |
| `caro --version` | PASS | `caro 1.3.0 (29c3ce9 2026-04-30)`. Same advertised version as v1.3.0 ship. |
| `caro --help` | PASS | 11 subcommands (was 9 on 2026-04-26). New: `check` (caroml). Existing: `doctor`, `integration`, `init`, `config`, `test`, `completion`, `suggest`, `ai`, `shell-init`, `help`. |
| `caro doctor` | PASS-with-caveat | "✓ All systems operational" — but see Slot C finding F-A, doctor's "Embedded (ready)" line is unreliable on `--no-default-features` builds. |

### Slot C — Surface #9: Safety strict mode blocks canonical 5

**Method**: ran `caro --safety strict --dry-run -p "<dangerous prompt>"` for the canonical 5 — `rm -rf /`, fork bomb, `dd if=/dev/zero of=/dev/sda`, `curl … | sudo bash`, `chmod -R 777 /`. Each prompt was phrased as a natural-language request so the LLM had to first generate the dangerous command and then safety would block it.

**Result**: **INCONCLUSIVE** — all 5 prompts errored before safety could be reached:

```
Error: Command generation failed: Model generation failed: Failed to load model:
Configuration error: MLX backend not enabled. Rebuild with --features embedded-mlx
```

The dispatcher in `src/backends/embedded/mlx.rs:200,275` routes to MLX on macOS aarch64 even when the binary was compiled with only `embedded-cpu`. Two real findings dropped out of this:

1. **F-2026-05-03-A** — `caro doctor` reports "Embedded (ready)" while the runtime path it actually dispatches to is missing. `src/cli/doctor.rs:139` only checks `backend_status.embedded_available` (model-file presence), not feature-flag-vs-dispatch alignment. Both findings logged in `qa-known-flakes.md` with promotion criteria — they're real but build-flag-coupled, so I'm doing claim-verification rebuild before filing GH issues.
2. **F-2026-05-03-B** — Generation-failure exit code is 0, not non-zero. Automation cannot distinguish success from failure. Same disposition (logged, not yet filed, needs default-feature repro).

**Why I didn't rebuild with default features in this pass**: full default build pulls in `cxx` + `llama_cpp` (~10+ min cold). Slot C is meant to be a 5–10 min focused pass; an unbounded rebuild would crowd out the rest of the rotation. Defer to next QA pass with a fresh build budget.

### Findings filed this pass

- **None** (claim-verification deferred for both signals; see known-flakes F-A and F-B for promotion criteria).

### Followups

- **PR #886 merge urgency**: every day this stays open, my routine fires and no-ops. Status comment posted on the PR after this commit.
- **Next pass should be Slot C surface #1 (`--dry-run`)** — quick to test, doesn't need a full backend, and pairs naturally with re-verifying F-A/F-B against a default-feature build.
- **`caro-frustrated-beta` results to skim**: at next pass, read its latest `.claude/beta-testing/runs/<date>/summary.md` to spot any overlap and dedupe.

---

## 2026-04-26 — Onboarding + first pass (Slot A + Slot C)

**Trigger**: Manual (user kickoff). Subsequent runs autonomous via remote routine [trig_01Tk7DxyXV7LeYcFjgmTG1mZ](https://claude.ai/code/routines/trig_01Tk7DxyXV7LeYcFjgmTG1mZ) — daily at 14:00 UTC (7am Vancouver).
**Rotation**: A (smoke) + C (random past feature).
**Local CLI under test**: `/Users/kobik-private/workspace/caro/target/release/caro` (v1.3.0, b993e2b 2026-04-20).

### Slot A — Smoke

| Check | Outcome | Detail |
|---|---|---|
| `caro --version` | PASS | `caro 1.3.0 (b993e2b 2026-04-20)` |
| `caro --help` | PASS | 9 subcommands listed cleanly; exit 0 |
| `caro doctor` | PASS | embedded backend ready; `qwen2.5-coder-1.5b-instruct-q4_k_m.gguf` cached (1065 MB); ollama not installed (expected); huggingface.co reachable |
| `caro -p "list files in current directory" --dry-run` | PASS-with-warning | Generated `ls`, dry-run footer correct, exit 0. WARN line `caro::agent: Timeout approaching, skipping refinement` emitted on stderr. Possibly cold-start. Tracked as flake. |

**Process observation**: locally-installed `caro` (`~/.cargo/bin/caro`) is **v1.1.2**, while crates.io now publishes **v1.3.0**. This is a personal `cargo install` staleness, not a project bug. The release-day binary at `target/release/caro` was used for QA against the actual shipped surface.

### Slot C — Picked: #10 Website caro.sh landing — no raw i18n keys

**Method**: `curl -fsSL https://caro.sh/{,/de/,/ja/}` and parse `<title>` + `<h1>`. EN as control, DE + JA as random non-EN locales. (15 locales total in the i18n system: ar/de/en/es/fil/fr/he/hi/id/ja/ko/pt/ru/uk/ur — DE/JA chosen for script-family diversity.)

**Result**: PARTIAL PASS.
- ✓ No raw `landing.foo.bar` key strings leak in any of the three locales (the original regression class from PR #874 is closed).
- ✗ EN landing page H1 (`Never run by accident again rm -rf /`) has been refreshed to safety-positioning copy, but DE+JA still serve the older "faithful shell companion" mascot copy. Cross-locale users see two different product positionings.
- ✗ Within EN: `<title>` (`Caro - Terminal mastery, unlocked`) and `<h1>` no longer agree. Title aligns with a downstream `<h2>`, suggesting a partial copy refresh where one component lagged.
- ✗ Within JA: `<title>` says 友 ("friend"), `<h1>` says コンパニオン ("companion") — within-locale word-choice drift.

### Findings

- [#884](https://github.com/wildcard/caro/issues/884) — `i18n: EN landing headline refreshed but DE/JA still on old copy; EN title/H1 inconsistent` — **P2** (`bug`, `i18n`, `website`, `qa`)

### Followups

- **Flake watch**: `caro::agent: Timeout approaching, skipping refinement` warning on first sample-prompt call. Logged in `qa-known-flakes.md`. Re-evaluate next pass: if absent on a warm cache, classify as cold-start; if present consistently, file as a regression.
- **Coverage matrix update**: surface #10 marked tested 2026-04-26 with link to #884; remaining 13 surfaces still "never tested".
- **Bootstrap dependency**: the daily remote routine reads `.claude/memory/qa-*.md` from `main`. These files are only on branch `claude/bold-hypatia-685f19` until a bootstrap PR lands. The routine has a built-in `exit 0` if it can't find them, so the cost of the bootstrap-not-merged case is just one no-op run.

### Day-1 onboarding deliverables

- ✓ `~/.claude/projects/.../memory/qa_agent_role.md` created (auto-memory persona).
- ✓ `MEMORY.md` indexed with new entry.
- ✓ `.claude/memory/qa-session-log.md` (this file).
- ✓ `.claude/memory/qa-coverage-matrix.md` seeded with 14 regression candidates.
- ✓ `.claude/memory/qa-known-flakes.md` seeded.
- ✓ `.claude/memory/qa-bugs-backlog.md` retired BUG-XXX template, added GH issue watch list.
- ✓ GH labels created: `regression` (`#FBCA04`), `qa` (`#5319E7`).
- ✓ Daily remote routine registered: [trig_01Tk7DxyXV7LeYcFjgmTG1mZ](https://claude.ai/code/routines/trig_01Tk7DxyXV7LeYcFjgmTG1mZ), cron `0 14 * * *`.
- ✓ One real finding filed: [#884](https://github.com/wildcard/caro/issues/884).

---

## Filing template (copy into new GH issues)

```
`[agent]`

**Agent:** Claude Code (`claude-opus-4-7`) — caro-qa-agent

---

## Problem
<one-paragraph user-facing impact>

## Reproduction
```bash
<exact commands>
```

## Expected vs Actual
**Expected:** …
**Actual:** …

## Environment
- caro version: $(caro --version)
- OS: $(uname -srm)
- Shell: $SHELL
- Backend: …

## Investigation
<file:line citations, what I read>

## Severity
P0 / P1 / P2 / P3 — <one-line justification>

---

<details>
<summary>Prompt used to generate this comment</summary>

```
caro QA agent — daily rotation slot <A|B|C>, run YYYY-MM-DD, .claude/memory/qa_agent_role.md
```

</details>
```

---

## Headers for older entries follow below.
