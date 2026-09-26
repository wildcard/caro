# QA Session Log

Reading order: most recent first.

---

## 2026-09-26 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (117 commits since last log entry on main; range c9a31a3..be07b22) + C.

**Context note**: QA PRs #1373 (2026-07-25) and #1443 (2026-09-07) are still unmerged. Memory files on main reflect the 2026-05-07 bootstrap only. This entry is written against the main-branch state.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 33s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: 24 subcommands including `ai`, `suggest`, `skill`, `export`, all CaroML verbs
- `caro doctor` → **PASS**: advisory (no model downloaded); proxy detected; embedded "needs model download"
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001 4th occurrence: model download blocked in sandbox)
- `caro --backend-info` → **PASS**: 7 backends listed consistently with `--help` roster

### Slot B — Recent diff (138 commits since 2026-05-07)

PRs merged since last log entry on main (note: #1373/#1443 branches also tested some of these; results consistent):

- **#1315 `fix(safety): normalize destructive stmts`** → `cargo test --lib -- safety` **34/34 PASS** (up from 19 at bootstrap; new guards in place)
- **#1298 `fix(cli): single source of truth for backend roster`** → `--backend-info` shows 7 backends, matches error message roster; **PASS**
- **#1352 `fix(i18n): load all locale JSON files; overhaul Hebrew translations`** → Python key-count check: EN=339, HE=339, full coverage; **PASS**
- `caro test --backend static` → **11/11 PASS** (100% across Website Claim, Natural Variant, Edge Case categories)
- `cargo test --lib` → all pass (no count captured but no failures seen)

Surfaces flagged for future Slot C: #31 (i18n locale smoke) elevated priority given recent i18n changes.

### Slot C — `caro ai --once` (surface #10, oldest 'never')

- `caro ai --help` → **PASS**: `--once`, `--new-session`, `--continue-session` flags documented; stdout-only design noted
- `caro ai --once 'list files in current directory sorted by size'` → **FLAKE** (FLAKE-001: silent hang after 60s; no stdout, no stderr — model download blocked)
- `cargo test --lib -- "ai::"` → **PASS**: 23/23 (runner, session, store, privacy, shell_init)
- Cross-reference: P1 issue [#1375](https://github.com/wildcard/caro/issues/1375) open — CpuBackend placeholder always returns `echo 'Please clarify your request'` on Linux x86_64 due to system-prompt keyword collision. Confirms surface #10 has two layered issues: FLAKE-001 (model download) masks #1375 (placeholder response) in sandbox.

### Findings

- No new issues filed this pass.
- [#1372](https://github.com/wildcard/caro/issues/1372), [#1442](https://github.com/wildcard/caro/issues/1442) open — CLAUDE.md version drift (1.4.0 vs 1.5.0); additionally MSRV field drifted (1.83 vs 1.85). Comment added to #1442 noting MSRV drift.
- [#1375](https://github.com/wildcard/caro/issues/1375) open — P1, `caro ai --once` CpuBackend placeholder (confirmed still open in v1.5.0, first confirmed in today's context).

### Followups

- PRs #1373 and #1443 need to merge to bring memory files in sync. Without merge, each QA run re-discovers the same state.
- FLAKE-001 4th occurrence logged (2026-09-26). Occurrences span >7-day windows; stays classified as flake. Promotion threshold: 3 in 7 days.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — same 'never' cohort, surface #10 now tested.
- P1 #1375 (CpuBackend placeholder) is a genuine regression for Linux users; should be priority fix.

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
