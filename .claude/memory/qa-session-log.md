# QA Session Log

Reading order: most recent first.

---

## 2026-07-12 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (103 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).
**Binary**: caro 1.5.0 (a6519f6 2026-07-12), built in 3m 03s.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 03s, Finished release profile)
- `caro --version` → **PASS**: `caro 1.5.0 (a6519f6 2026-07-12)`
- `caro --help` → **PASS**: all 24 subcommands listed including CaroML verbs and `skill` command
- `caro doctor` → **PASS** (initial: no model advisory; after first dry-run: model downloaded, "✓ Embedded (ready)")
- `caro -p 'list files in current directory' --dry-run` (30s timeout) → **FLAKE**: telemetry consent + model download; process killed by 30s timeout before completing
- `caro -p 'list files in current directory' --dry-run` (90s timeout) → **PARTIAL**: model downloaded; generated `echo 'Please clarify your request'` instead of `ls` — root cause is CPU stub bug (see #1269, #1281, #1289, all open)
- Repeat dry-run (90s, model cached) → same wrong result; 7/8 common queries return `echo 'Please clarify your request'` (only static-matcher hits like `show disk usage` → `df -h` work correctly)

### Slot B — Recent diff

103 PRs merged since 2026-05-07. Key surfaces exercised:

- **Static matcher fix #947** (`find and kill the runaway process eating CPU` → full `ps aux | sort … | xargs kill` pipeline): **PASS** ✓
- **Backend roster #1298 / #1115** (`caro --backend-info` shows `not compiled` for remote backends; no stale entries): **PASS** ✓
- **Safety allowlist regression #1246** (7/7 `allowlist_catastrophic_tests` pass; specific-subpath allowlist works, catastrophic targets always blocked): **PASS** ✓
- **Custom safety patterns 1.5.0 feature**: `patterns.toml` loaded without errors using `[[pattern]]` section (correct per example file; CHANGELOG says `[[safety.custom_patterns]]` — minor doc inconsistency). End-to-end blocking test **inconclusive** — CPU stub generates wrong commands so safety validator never sees a real kubectl/terraform command to block. Flagged for re-test once #1269 is fixed.
- **MSRV 1.85**: confirmed in `Cargo.toml`; CLAUDE.md still says 1.83 (pre-existing #1283/#1288).

Surfaces flagged for future Slot C: custom safety patterns end-to-end blocking.

### Slot C — `caro ai --once` (surface #10)

- `caro ai --once 'list files sorted by size'` → stdout: `echo 'Please clarify your request'`, stderr: `# caro-ai: session 1 confidence=0.85 risk=Safe`; EXIT=0
- `caro ai --once 'show git log for last 5 commits'` → same wrong output; EXIT=0
- `caro ai --once --new-session 'show memory usage'` → same wrong output; EXIT=0
- **stdout/stderr split**: correct — command on stdout only, metadata comment on stderr. Help text "Stdout is the generated command only" is accurately implemented.
- **Session handling**: resumption (`session 1 (resumed)`) and `--new-session` both work as described.
- **Root cause**: CPU stub bugs #1269/#1281/#1289 reproduce in 1.5.0 unchanged.

Surface #10 verdict: **FAIL** — produces wrong output on Linux x86_64; pre-existing bugs unresolved in 1.5.0 release.

### Findings

- [#1319](https://github.com/wildcard/caro/issues/1319) — `docs: CLAUDE.md version banner shows 1.4.0 (GA) after 1.5.0 release` (P2) — **NEW**
- Pre-existing open bugs confirmed in 1.5.0: #1269, #1281, #1289 (CPU stub / `caro ai --once`); #1283, #1288 (MSRV drift)

### Followups

- Custom safety patterns (1.5.0 headline feature) cannot be validated end-to-end until #1269 is fixed.
- CLAUDE.md has two stale fields: version (1.4.0→1.5.0, filed #1319) and MSRV (1.83→1.85, existing #1283/#1288). Both fixed by adding CLAUDE.md to release-version-alignment checklist.
- Model download timing: first dry-run timed out at 30s; second attempt (90s) succeeded. Model download slow (~60s+) in this sandbox. FLAKE-001 second proximity observation logged.
- Next Slot C candidates: surface #11 (`caro ai --continue-session` shell widget), surface #12 (`caro assess`).

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
