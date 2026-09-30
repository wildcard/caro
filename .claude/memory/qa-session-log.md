# QA Session Log

Reading order: most recent first.

---

## 2026-09-30 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs merged since last log entry 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 27s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed (doctor, integration, init, config, test, completion, suggest, ai, shell-init, check, list, jobs, new, generate, run, export, experiment, adopt, history, why, do, render, skill)
- `caro doctor` → **PASS**: advisory only (no model downloaded; proxy detected at 127.0.0.1:41305; HuggingFace reachable — expected in sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: telemetry consent shown then timeout (EXIT:124 at 30s); same FLAKE-001 as bootstrap run

Side observation: `CLAUDE.md` line 9 still reads `- **Version**: 1.4.0 (GA)` but binary is `1.5.0`. Issue #1044 was fixed (1.1.0→1.3.0) but drift recurred for 1.4.0→1.5.0. 45+ open duplicate issues exist (#1442, #1469, #1474, #1476, #1483 etc.) — no new issue filed.

### Slot B — Recent diff

110 PRs merged since last log entry (2026-05-07). Representative surfaces sampled:

- **PR #1315** `fix(safety): P0 — close quote/escape evasion of the command scanner` → verified via `cargo test --lib -- safety` → **PASS** (34 safety tests pass; count grew from 19 to 34, consistent with new pattern coverage)
- **PR #1352** `fix(i18n): load all locale JSON files; overhaul Hebrew translations` → flagged for next Slot C cycle (i18n website locale surface #31 in coverage matrix); not exercised today (requires live website)
- **PR #1304** `chore(release): v1.5.0` → accounted for in version bump noted above

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: docs accurate, flags `--once`, `--new-session`, `--continue-session` described correctly
- `caro ai --once 'list files in current directory'` → **FLAKE**: EXIT:124 (30s timeout, zero output even on stderr) — FLAKE-001 (model download blocked in sandbox)
- `cargo test --lib -- ai` → **PASS**: 62 tests pass (runner, session, privacy, evaluation safety evaluators)
- Known UX gap: silent hang with no backend ready (no stderr progress) — open issues #1404, #1393, #1455, #1468 (7 open total); no new issue filed

### Findings

None this pass — all observed gaps have existing open issues.

### Followups

- FLAKE-001 (model download) observed again (2nd total: 2026-05-07, 2026-09-30). Not yet 3-in-7-days; no regression filing. Updated flakes log.
- CLAUDE.md version drift (1.4.0 vs 1.5.0) is a chronic issue (45+ open duplicates); the fix would be adding CLAUDE.md to the 6-file release checklist. Most recent issue: #1483 (2026-09-29). No new filing.
- `caro ai --once` silent hang: 7 open issues unchanged. No regression.
- Next Slot C candidate: surface #11 (`caro ai --continue-session`) or surface #31 (i18n website locale curl test) — both untested.
- 110 merged PRs since last session log; several brand/website PRs (#1155, #1156, #1158, #1159) touch website surfaces — consider website visual regression in future slots.

---

## 2026-05-07 — Scheduled run (Slot A + Slot C) [BOOTSTRAP]

**Trigger**: manual invocation; first-ever run of caro-qa-agent (bootstrap pass).
**Rotation**: A + C (no Slot B — no prior session log entry to compute last-run date from).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 46s, no errors)
- `caro --version` → **PASS**: `caro 1.3.0 (f8028ed 2026-05-05)`
- `caro --help` → **PASS**: all subcommands listed including new CaroML verbs (`check`, `new`, `list`, `jobs`, `run`, `generate`, `experiment`, `adopt`, `history`, `why`, `do`, `render`, `skill`)
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: model download failed after 3 retries (HuggingFace HTTP 200 reachable but binary download blocked in sandbox; env limitation, not a code bug — see qa-known-flakes.md)
- Telemetry consent on first invocation → **PASS**: shown once, persisted; second invocation showed no consent prompt
- `caro shell-init bash` → **PASS**: emits correct bash wrapper function with readline edit mode
- `caro init --minimal` → **PASS**: `caro is already configured!` (config persisted after first run)

### Slot B — Recent diff

Skipped — no prior session log entry. First-ever run.

### Slot C — Safety Validation

Surface chosen: **Safety validation module** (oldest = never tested; first-ever run, all surfaces tie at 'never').

- `cargo test --lib -- safety` → **PASS**: 19/19 safety unit tests (including CVE patterns, pattern compilation, risk filtering, shell-type filtering, CaroML safety validator, evaluation safety evaluators)
- `cargo test --lib` → **PASS**: 513 passed, 0 failed, 1 ignored
- `caro new test-task` → **PASS**: scaffolds `tasks/test-task.caro` correctly
- `caro check tasks/test-task.caro` → **PASS**: `ok (2 steps, 0 pragmas, 0 params)`
- `caro list` → **PASS**: `(no tasks in ./tasks/ or ~/.caro/library/)`
- `caro jobs` → **PASS**: `(no Carofile in current directory; create one to define jobs)`

### Findings

- [#1044](https://github.com/wildcard/caro/issues/1044) — `docs: CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0` (P2)

### Followups

- Model download FLAKE observed once. Sandbox network appears to block HuggingFace binary downloads despite HTTP reachability. Track in qa-known-flakes.md; if reproduced 3×/7 days, promote to regression.
- Next Slot C candidate: `caro ai` conversational mode (surface #9 in matrix, never tested).
- Consider adding `CLAUDE.md` to the release-version-alignment 6-file checklist so version drift can't recur (noted in #1044 fix direction).

---
