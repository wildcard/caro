# QA Session Log

Reading order: most recent first.

---

## 2026-09-27 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

**Context note**: QA PRs #1373 (2026-07-25), #1443 (2026-09-07), and #1477 (2026-09-26) remain unmerged. Memory files on `main` still reflect only the 2026-05-07 bootstrap state. This PR updates them independently against main, ingesting known open issues from those sessions.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 31s, no errors; v1.5.0)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: 24 subcommands listed including `ai`, `assess`, `suggest`, `export`, all CaroML verbs, `skill`
- `caro doctor` → **PASS**: advisory (no model downloaded; huggingface.co reachable; proxy detected)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001: silent hang → timeout; model download blocked in sandbox; 5th overall occurrence, 2nd within 7-day window; stays classified as flake)
- Telemetry consent on first invocation → **PASS**: consent shown once; second invocation suppressed

### Slot B — Recent diff

110 PRs merged on main since last log entry (2026-05-07). Three representative surfaces exercised:
- **Safety P0 fix** (#1315 `fix(safety): close quote/escape evasion`): `cargo test --lib -- safety` → **34/34 PASS**
- **`caro suggest`** (surface #13, never tested): `caro suggest 'list files in current directory'` → **PASS** (5 correct suggestions including `ls -la`, find variants)
- Prior QA sessions (#1477 2026-09-26, #1443 2026-09-07, #1373 2026-07-25) covered #1315, #1298, Hebrew i18n (#1352), and `caro test --backend static`; all confirmed PASS; no re-exercise needed

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **#10** (oldest 'never' on main's matrix; tied with #11–#32; lowest # wins).

- `caro ai --once 'list files' --backend embedded` → **FLAKE** (FLAKE-001: silent hang 15s; model download blocks; identical to #1477 finding)
- `caro ai --once --backend static` → immediate error: `Invalid argument: Unknown backend 'static'` (static is only valid for `caro test`, not `caro ai`)
- `caro ai --help` → PASS: flag documented correctly; `--once` = "the only mode supported today" (confirmed dead code `once: _` in dispatch, intentional per docs)
- Unit tests: (not re-run; #1477 confirmed `cargo test --lib -- "ai::"` → 23/23 PASS; build unchanged)

### Findings

None new — all findings already tracked in open issues:
- [#1375](https://github.com/wildcard/caro/issues/1375) — P1 — `caro ai --once` CpuBackend always returns placeholder on Linux x86_64 (open)
- [#1442](https://github.com/wildcard/caro/issues/1442) — P2 — CLAUDE.md version drift `1.4.0` vs actual `1.5.0` (open)

### Followups

- FLAKE-001: 5th occurrence overall. 2nd in 7-day window (2026-09-26, 2026-09-27). Threshold is 3 in 7 days — stays flake for now.
- P1 #1375 (CpuBackend placeholder) should be priority fix for Linux x86_64 users.
- QA PRs #1373, #1443, #1477 remain unmerged — owner should review and close as superseded or merge.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — will likely hit FLAKE-001 too; worth documenting for coverage.
- `caro assess` subcommand is commented out in source since v1.1.0-beta.1 (src/main.rs:422-434, 3048-3057); surface #12 marked N/A in matrix — cannot be exercised until re-enabled.

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
