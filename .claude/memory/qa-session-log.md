# QA Session Log

Reading order: most recent first.

---

## 2026-11-27 — Scheduled run (Slot A + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + C (no Slot B: no merges since the 2026-11-26 run; last merge is still #1740, 2026-11-02).
**State source**: `state/qa-rotation/` on `automation/routine-status` (per `.claude/automation/CONTRACT.md` section 5), read with `git show`. This file on `main` is the May bootstrap copy. This entry holds only this run's rows, for folding into the status branch.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS**
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS** (exit 0)
- `caro doctor` → **PASS** (exit 0; advisory: no model, no Ollama)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` from the static matcher

### Slot B — Recent diff
Skipped. No PRs merged since 2026-11-26 (search `is:pr is:merged merged:>=2026-11-26` gives 0).

### Slot C — #33 `caro ai` decision gates (#1459) (never tested before)
Lowest live "never" row. #20 needs Ollama, which the sandbox does not have.
- **PASS**: `tests/clarification_gate_contract.rs` (2 passed); lib tests for `decision::`, `prompts::intent`, `risk_judge`, `calibration` (48 passed).
- **PASS**: main path. A non-static prompt (`rename every jpeg in this folder to include its exif date`) prints the question with no `Error:` prefix, exit 0. `--output json` gives `{"needs_clarification": true, "p": 0.9, "question": ...}`, exit 0.
- **PASS**: static matcher reports measured confidence (`Confidence: 1.00` for a regex hit in `--verbose`).
- **FAIL**: `caro ai --once` with the same prompt prints `Error: backend error: Clarification needed: …` and exits 1. `src/ai/runner.rs:121` wraps every generator error as `backend error`, so the typed gate is lost. Filed #1792 (P2).
- Not filed (known): the question text is wrong for every prompt (CPU stub, #1269 / #1760). `caro --output json ai --once …` and `caro --verbose doctor` route the subcommand words into the prompt (#1163).
- Not filed (observation): `compress the logs folder but skip anything older than a week` gets `tar -czf archive.tar.gz directory/` at confidence 1.00 from a regex match. The age filter is dropped. Regex hits always score 1.0 by design (#1461).

### Findings
- [#1792](https://github.com/wildcard/caro/issues/1792) — ai: `caro ai --once` reports a clarification question as "Error: backend error", exit 1 (P2)

### Followups
- Prompt drift (as on 11-23 to 11-26): the stored prompt says to store memory in a PR. `CONTRACT.md` section 5 says `bin/routine-status state put`. This session may push only to `claude/peaceful-hopper-t2mkhh`, so the state branch was not written. The owner must fold this entry.
- Next "never" rows: #34 executor timeout / guard hooks, #35 `caro export`. #20 stays blocked (no Ollama).
- Static-matcher regex confidence of 1.0 on a prompt with a dropped constraint may be worth a calibration look under epic #1460.

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
