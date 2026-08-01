# QA Session Log

Reading order: most recent first.

---

## 2026-08-01 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (20 of ~82 PRs sampled since 2026-05-07) + C (surface #10: `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 07s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed including all CaroML verbs and `skill` command
- `caro doctor` → **PASS**: advisory only (no model, proxy detected; expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 reproduced; telemetry consent appeared then model download blocked (sandbox restriction)

### Slot B — Recent diff

~82 PRs merged since 2026-05-07; **20 most-recent sampled** (search perPage:20 limit — see Followups). Representative surfaces tested from the sampled set:

- **#1315** `fix(safety): P0 close quote/escape evasion` → `cargo test --lib -- safety` → **PASS**: 34 safety tests (up from 19 in bootstrap; reflects new patterns from the P0 fix)
- **#1298** `fix(cli): single source of truth for backend roster` → `caro --backend-info` → **PASS**: 7 backends with correct statuses
- **#1346/#1351/#1352** (website/ci/i18n) → not testable via CLI in sandbox; flagged for next website Slot C rotation (#25, #31)
- Full library test suite → **PASS**: 597 tests (up from 513 in bootstrap run), 0 failures

Note: the 62 unsampled PRs (earlier half of the 2026-05-07 to 2026-08-01 window) are not permanently excluded. The full 597-test library suite passing green provides a functional safety net across all merged code. Future Slot B runs will search since 2026-08-01 and cover new work from there. The unsampled window can be spot-checked by a future run using an explicit date range if a regression is suspected.

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: documents `--once`, `--new-session`, `--continue-session`; prompt is positional not `-p`
- **P1 BUG**: `caro ai --once 'ANY PROMPT'` always returns `echo 'Please clarify your request'` with `confidence=0.85` — identical wrong output for every prompt including 'show current directory', 'show disk usage', 'list files'
- Comparison: `caro -p 'show current directory' --dry-run` → `pwd` (static matcher works on regular path)
- Root cause confirmed in source: `run_ai_once` (`src/main.rs:1109`) calls `cli_app.backend_arc()` directly, bypassing the `AgentLoop` that applies the static matcher (`src/cli/mod.rs:284-285`). The raw embedded backend returns its hardcoded fallback (`src/backends/embedded/cpu.rs`) when it can't download a model.
- `caro ai --once -p 'prompt'` rejects `-p` with an error — minor UX inconsistency vs main `caro` path; not filed separately since help text is clear.
- `caro ai --continue-session` (surface #11) was NOT independently exercised this run. It shares the underlying session store and backend selection path with `--once`, so the same P1 bug likely applies, but it was not confirmed by direct invocation. Kept in rotation for a future Slot C pass.

### Findings

- [#1387](https://github.com/wildcard/caro/issues/1387) — `ai: caro ai --once bypasses static matcher, always returns fallback when LLM unavailable` (P1)
- [#1388](https://github.com/wildcard/caro/issues/1388) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurrence of #1044)` (P2)

### Followups

- FLAKE-001 reproduced (2026-08-01, second observation; first was 2026-05-07). Gap of 85 days — not within 3-in-7-days promotion threshold. Remains a flake.
- Issue #1044 was fixed for the v1.3.0 cycle but the systemic checklist fix (add CLAUDE.md to release-version-alignment.md) did not land, causing recurrence as #1388. Fix direction in #1388 includes the systemic fix.
- **Slot B coverage gap**: next agent should increase perPage to 100 (or paginate) to ensure all merged PRs in the window are captured, not just the most recent 20.
- Website/i18n surfaces (#25, #31) are high priority for next Slot C given recent PRs #1325/#1351/#1352 touching those areas.
- Surface #11 (`caro ai --continue-session`) remains in rotation at 'never' — exercise it directly in a future Slot C pass rather than grouping with #10 by inference.

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
