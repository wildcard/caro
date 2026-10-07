# QA Session Log

Reading order: most recent first.

---

## 2026-10-07 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (many PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (build succeeded; binary at `target/release/caro`)
- `caro --version` → **PASS**: `caro 1.5.0 (11c472e 2026-10-07)`
- `caro --help` → **PASS**: all subcommands listed including `ai`, `shell-init`, CaroML verbs, `skill`
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: used static backend, generated `ls -la`, telemetry consent shown once and persisted

### Slot B — Recent diff

PRs merged since 2026-05-07 (sampling past 7 days — many more total):

- **PR #1497** `fix(cli): accept every config show key in config set/get` → tested `caro config get` for all 8 keys (`backend`, `model-name`, `shell`, `safety`, `telemetry.enabled`, `log_level`, `cache_max_size`, `log_rotation`) — **PASS**: all keys return correct values
- **PR #1519/#1535** safety text/label fixes → `cargo test --lib -- safety` → **PASS**: 38/38 safety tests
- **PR #1509** `fix(explain): STE-lite explain mode` → `caro -p '...' --dry-run --explain` → **PASS**: clean STE-lite output produced
- **PR #1503** `fix(ai): don't block on stdin when ai --once has a trailing prompt` → code-verified: `needs_stdin_prompt` returns false when trailing words present; functional test blocked by FLAKE-001/backend hang (see Slot C)

### Slot C — `caro ai --once` (surface #10, never previously tested)

- `caro ai --once list files in current directory` → **FLAKE**: hangs indefinitely (>45s) — model download stalls in sandbox; same underlying issue as FLAKE-001 extended to the `ai` subcommand
- Error path tested: `echo "" | caro ai --once` → **PASS**: exits immediately with `Error: no prompt provided`
- Stdin-piped path: `echo "prompt" | caro ai --once` → **FLAKE**: reads stdin correctly (exits prompt-resolution), then hangs on backend init
- `caro ai --help` → **PASS**: `--once` documented as "Run one turn and return — no TTY REPL"
- Code inspection confirms PR #1503 fix is in place (`needs_stdin_prompt` guards stdin read)
- Duplicate of open issues: #1440 (no HTTP download timeout), #1179 (no static fallback), #1408 (no feedback during init)

### Findings

- No new issues filed — all observations are duplicates of existing open issues
- CLAUDE.md version drift (1.4.0 vs 1.5.0): duplicate of open #1520
- `caro ai --once` hang: duplicate of open #1440, #1179, #1408

### Followups

- FLAKE-001 observation count updated: 2 (2026-05-07, 2026-10-07) — not yet at 3×/7d threshold, but note the `ai --once` variant of the same failure is separately tracked as #1440 (open, unresolved)
- CLAUDE.md shows 1.4.0 (GA) vs caro 1.5.0. Open issue #1520 (filed 2026-10-03) covers this. The root-cause fix (adding CLAUDE.md to `release-version-alignment.md` checklist) has not landed despite dozens of filings since May 2026.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or surface #12 (`caro assess`) — both "never" tested, pick #11 by lowest-# tie-break rule.

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
