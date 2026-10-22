# QA Session Log

Reading order: most recent first.

---

## 2026-10-22 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** (2m 36s)
- `caro --version` → **PASS**: `caro 1.5.0 (0766bf8 2026-10-21)`
- `caro --help` → **PASS**: new `guard` and `export` subcommands listed; `assess` no longer listed (matches #1680)
- `caro doctor` → **PASS**: advisory only (no model, no Ollama; expected in sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` via static matcher, no model download needed

### Slot B — Recent diff
Last run 2026-05-07; 184 PRs merged since. Sampled the most relevant:
- #1614 (segment/wrapper-aware safety): `-p 'delete everything in root directory recursively'` blocked as adversarial → PASS
- #1683 (`caro guard test`): `rm -rf /` → deny rc=2; `echo hi && sudo rm -rf /` → deny rc=2 (wrapper-aware); `ls -la` → none rc=0 → PASS
- #1680 (assess removed from README): `assess` absent from `--help` → consistent

### Slot C — `caro ai --once` (matrix #10, oldest never-tested, lowest #)
- `caro ai --once "list files in current directory"` → **FAIL**: `Error: backend error: Clarification needed: What exactly should be deleted?`, rc=1. Reproduces with `--new-session` and with a different prompt (`show disk usage`).
- Cause: `src/backends/embedded/cpu.rs:56-70` is still a placeholder; `prompt.contains("rm")` matches the system prompt text, so every call returns the deletion clarification.
- Already tracked: open #1269 (product decision, register BW-006); closed duplicates #1673, #1523, #1494, #1500. No new issue filed.

### Findings
None this pass (known issue #1269 reconfirmed on 1.5.0 at 0766bf8).

### Followups
- FLAKE-001 (model download) not observed: dry-run was served by the static matcher.
- #1269 has now produced 6 duplicate reports; consider a decision on it (stub removal or honest "no model" error).
- Next Slot C candidates: #11 `caro ai --continue-session`, #13 `caro suggest`, #14 `caro config`.

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
