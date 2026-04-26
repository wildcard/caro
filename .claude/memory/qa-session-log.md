# caro-qa-agent — Session Log

**Last updated**: 2026-04-26 by caro-qa-agent
**Reading order**: most recent first.

This is the append-only chronicle of every QA pass. Each entry records the rotation slots run, what was tested, what was found, and where findings were filed. Other agents reading this file: scroll to the top for the latest state; the headline tells you whether the project is in a clean or known-flaky state right now.

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
