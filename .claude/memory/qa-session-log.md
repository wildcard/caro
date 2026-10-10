# QA Session Log

Reading order: most recent first.

---

## 2026-07-11 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 19s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (9311c9e 2026-06-13)`
- `caro --help` → **PASS**: all subcommands present including CaroML verbs and `ai`, `skill`
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 (2nd observation); telemetry banner appeared on first invocation (fresh sandbox, first_run=true), auto-opted-out (non-interactive TTY), then hung on model download
- **Slot A finding**: `caro config set telemetry.enabled false` (advertised in consent screen) rejects as unknown key → filed [#1292](https://github.com/wildcard/caro/issues/1292) (P1)

### Slot B — Recent diff (30 PRs since 2026-05-07)

Key surfaces checked:
- **PR #1209** `feat(backends): Mesh-LLM + AI-Horde + hybrid privacy gateway` — `--backend-info` vs actual resolver inconsistency confirmed. Three-way list mismatch. Filed [#1293](https://github.com/wildcard/caro/issues/1293) (P1) and [#1294](https://github.com/wildcard/caro/issues/1294) (P2).
- **PR #1110** `feat(safety): runtime-loadable custom patterns` — `[[pattern]]` key confirmed in `examples/patterns.example.toml`; safety tests all pass (27/27, up from 19 in last run due to new smart_blend_tests from PR #1206).
- **PR #1109** `feat(cli): --edit flag` — `-e, --edit` flag confirmed in `--help` with rustyline description.
- i18n PRs #817/#816/#825–#829 — locale files updated; flagged surface #31 for next Slot C.
- Prior watch-list issue [#1044](https://github.com/wildcard/caro/issues/1044) — confirmed closed (completed).

### Slot C — `caro ai --once` (surface #10, oldest 'never')

- `caro ai --help` → **PASS**: subcommand documented, `--once` flag present
- `caro ai --once` (no prompt) → **PASS**: correct error `no prompt provided`
- `caro ai --once "list all running processes"` → **FAIL**: silent hang, zero stdout/stderr, killed after 35s
- `echo "show disk usage" | caro ai --once` → **FAIL**: same silent hang
- `caro ai --once "..." </dev/null` → **FAIL**: same silent hang
- Filed [#1295](https://github.com/wildcard/caro/issues/1295) (P2)

### Findings

- [#1292](https://github.com/wildcard/caro/issues/1292) — `cli: telemetry consent advertises invalid config key` (P1)
- [#1293](https://github.com/wildcard/caro/issues/1293) — `cli: --backend-info shows wrong backend list` (P1)
- [#1294](https://github.com/wildcard/caro/issues/1294) — `cli: --backend help text missing mesh/ai-horde/hybrid` (P2)
- [#1295](https://github.com/wildcard/caro/issues/1295) — `ai: caro ai --once hangs silently when model unavailable` (P2)

### Followups

- FLAKE-001 second observation (2026-07-11). 1 more in 7-day window → promote to regression.
- Next Slot C candidate: surface #31 (i18n website locale smoke — `curl /es/ /fr/ /ja/`). Multiple i18n PRs merged since last run.
- #1293 and #1294 are related backend-list drift issues; likely fixed in one PR.
- #1295: test `caro ai --once` in environment with pre-downloaded model to confirm if silent hang is FLAKE-001 or a separate `ai`-path bug.

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
