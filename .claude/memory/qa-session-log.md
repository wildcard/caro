# QA Session Log

Reading order: most recent first.

---

## 2026-10-08 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 34s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (11c472e 2026-10-07)` (upgraded from 1.3.0)
- `caro --help` → **PASS**: all subcommands present including `ai`, `config`, CaroML verbs, `skill`
- `caro doctor` → **PASS**: advisory only (no model downloaded; huggingface reachable; proxy detected)
- `caro -p 'list files in current directory' --dry-run` → **PASS** (static matcher fallback; `ls -la` generated immediately without model; improvement over 2026-05-07 FLAKE)
- Telemetry prompt shown once on fresh sandbox; piping `n` bypasses and command proceeds correctly

### Slot B — Recent diff

PRs merged since 2026-05-07 (≥20 significant PRs; key surfaces exercised):
- **#1535, #1519** (safety pattern fixes) → `cargo test --lib -- safety` → **PASS**: 38/38 safety tests (up from 19 in prior run, new tests added)
- **#1497** (config get/set key coverage) → `caro config show`, `config get safety/log_level/cache_max_size/log_rotation`, `config set safety moderate` → **PASS**: all keys accepted
- **#1509** (STE-lite explain mode) → `caro -p "list files" --explain` → **PASS**: explains `ls -la` in short-sentence STE format with option breakdown
- **#1503** (ai --once stdin fix) → tested as part of Slot C; see below
- **#1488** (security dep upgrade) → build succeeded; `cargo audit` not installed in sandbox — CI Security Audit job is the authoritative check for advisory-database coverage

### Slot C — caro ai --once (surface #10)

Surface chosen: `caro ai --once` scripted conversational mode (lowest `#` among all 'never' surfaces).

- `caro ai --once "list files"` → **FAIL**: hangs indefinitely with zero stdout/stderr output; exit only via SIGTERM. No progress message, no download indicator, no timeout, no fallback.
- `echo "list files" | timeout 15 caro ai --once` → same hang; pipeline produces no output before timeout
- Root cause: backend initialisation (`CliApp::with_overrides` → embedded backend) blocks on model download; the sandbox blocks HuggingFace binary downloads. Unlike `--dry-run`, the `ai` path has no static-matcher fallback.
- Note: PR #1503 fixed the stdin-blocking issue (trailing args no longer cause double-read); the silent model-download hang is a separate UX gap.

### Findings

- [#1540](https://github.com/wildcard/caro/issues/1540) — `ai: caro ai --once hangs silently with no feedback when no backend is ready` (P2)
- [#1541](https://github.com/wildcard/caro/issues/1541) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0` (P2, recurrence of #1044)

### Followups

- FLAKE-001 (model download blocked in sandbox) still active; observed again this pass on `ai --once` surface. Count: 2 total observations (2026-05-07 on `--dry-run`; 2026-10-08 on `ai --once`). Note: `--dry-run` no longer flakes in 1.5.0 (static matcher fallback added).
- Issue #1044 confirmed closed (2026-05-09). Removed from watch list. However, the root-cause fix (add CLAUDE.md to checklist) was not applied — #1541 confirms recurrence.
- Prior QA rotation PRs (#1178, #1373, #1443, #1477, #1496) appear open/unmerged; session log on main still reflects only the 2026-05-07 bootstrap entry. Slot B date range consequently covered 5 months of PRs.

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
