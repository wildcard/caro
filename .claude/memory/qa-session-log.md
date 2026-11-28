# QA Session Log

Reading order: most recent first.

---

## 2026-11-28 — Scheduled run (Slot A + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + C (no Slot B: no PRs merged since the 2026-11-27 run; last merge is still #1740, 2026-11-02).
**State source**: `state/qa-rotation/` on `automation/routine-status` (per `.claude/automation/CONTRACT.md` section 5), read with `bin/routine-status state get`. This file on `main` is the May bootstrap copy. This entry holds only this run's rows, for folding into the status branch.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS**
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS** (exit 0; `guard` and `export` listed)
- `caro doctor` → **PASS** (exit 0; advisory: no model at start, no Ollama, guard hook not installed)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` from the static matcher. The embedded model downloaded later in the run (FLAKE-001 did not recur).

### Slot B — Recent diff
Skipped: no merges since 2026-11-27. The run first read main's stale May copy, so it re-checked the 11-02 to 10-27 merges before it found the state branch. All PASS, nothing new:
- #1740 `config reset safety` → back to Moderate; unknown key exits 1 and lists the valid keys.
- #1737 `caro test --help` → `[possible values: static, embedded]`.
- #1710 `caro ai` with no prompt → suggests `caro ai --once "list files"`, exit 1.
- #1687 / #1683 `caro guard test` → `curl … | sh` deny rc=2, `wget … | sudo bash` deny rc=2, `bash <(curl …)` ask rc=3, `ls -la` none rc=0.

### Slot C — #35 `caro export` platform runbook writer (never tested before)
Lowest live "never" row. #20 needs Ollama; #34 has no single CLI entry point.
- **PASS**: no lock → `run `caro generate disk` first`, exit 1. Missing task, unknown platform (`bogus`) and a platform with no active variant (`macos`) each exit 1 with a clear message. No argument → clap usage, exit 2.
- **PASS**: `caro export disk` writes `tasks/disk.linux.sh` (shebang, `set -euo pipefail`, TASK/WHY/DO comments). `-o out/run.sh` creates the missing parent directory.
- Known, not filed: the steps are mock `echo "<intent>"` lines (#1777). The header says `` `caro upgrade` to refresh `` (#1778).
- Observation, not filed: `run_caroml_export` (`src/main.rs`) writes lock commands without a safety re-check. Same class as #1784. A tampered-lock test was not run in the sandbox.

### Findings
None this pass. Re-observed known bugs only: `caro ai --once` gives "What exactly should be deleted?" for every prompt (#1269 / #1760), and as an error with exit 1 (#1792). `caro -p 'show git status' --dry-run` (auto backend) prints the same stub question, exit 0 (covered by #1269 and #1792's title). `guard test '<cmd>' --output json` swallows the flag (#1697); the swallowed words also add a second pattern match.

### Followups
- Prompt drift (as on 11-23 to 11-27): the stored prompt says to read and write memory in `.claude/memory/qa-*.md` via a PR. `CONTRACT.md` section 5 says `bin/routine-status state get/put`. This session may push only to `claude/peaceful-hopper-78e0b4`, so the state branch was not written. The owner must fold this entry. The stored prompt should be updated to read state first; this run lost time re-testing known surfaces because it read main's copy first.
- Next "never" rows: #34 executor timeout / guard hooks / loop budget. #20 stays blocked (no Ollama).

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
