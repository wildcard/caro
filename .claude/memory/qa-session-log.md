# QA Session Log

Reading order: most recent first.

---

## 2026-11-20 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C. Commit under test: `aa1b0e6` (main, 2026-11-02).
**Branch**: `claude/peaceful-hopper-eped6h` (session-assigned; the harness allows no other push target).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 18s)
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS**: lists `guard`, `export`, `suggest`, `ai`, all CaroML verbs
- `caro doctor` → **PASS**: advisory only (no model yet, no ready backend; expected)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` (static matcher). FLAKE-001 did not recur: the 1.1 GB qwen2.5-coder-1.5b GGUF downloaded later in the run.
- `caro --show-config` (advertised by doctor) → **FAIL**: blocks forever when stdin is an open pipe → filed #1775.

### Slot B — Recent diff

The last log entry on main is 2026-05-07, so "since last run" spans about 50 commits (shallow clone). Covered the 5 newest merged PRs only:
- #1740 `caro config reset <key>` → **PASS**: resets only `safety`; `log_level` stays. Unknown key → error, rc=1.
- #1737 `caro test --help` → **PASS**: lists only `static, embedded`.
- #1710 `caro ai` no-prompt error → **PASS**: no `-p` suggestion; rc=1.
- #1706 platform `ls --version` timeout → not exercised (needs a slow-`ls` fixture); build + doctor green.
- #1716 harness docs → no runtime surface.
- Also re-checked open #1723 (top-level stdin hang): reproduced (`sleep 20 | caro -p x --dry-run` → rc=124). Fix PR #1754 is open.

### Slot C — `caro ai --once` (matrix #10)

- `--help` text is accurate (`--once` is "the only mode supported today").
- 4 prompts (`list files…`, `show disk usage`, `find all txt files…`, `delete everything in root`), with and without `--new-session` → **FAIL**: every one returns `Error: backend error: Clarification needed: What exactly should be deleted?`, rc=1.
- Cause: `src/backends/embedded/cpu.rs:62-70` is a keyword placeholder, not real inference. The full system prompt contains `rm`, so every request hits the "delete" branch. The top-level path hides this because the static matcher answers first.
- Already tracked: #1760 (open, filed against `aa1b0e6`). Not re-filed. History: #1500, #1523, #1735 closed but the bug came back each time.
- `caro ai` with an open stdin pipe and `-p` → rc=2 (clap rejects `-p`, as #1710 intends).

### Findings

- [#1775](https://github.com/wildcard/caro/issues/1775) — `cli: caro --show-config blocks forever on an open stdin pipe (not covered by #1754)` (P2)
- Reproduced, not re-filed: #1760 (ai --once stub answer, P1-class), #1723 (top-level stdin hang, P1).

### Followups

- QA memory PRs #1178, #1373, #1443, #1477, #1496 were all closed unmerged. Memory on main stayed at 2026-05-07, so each run restarts the same rotation. The owner should merge or say how to store QA state (for example `bin/routine-status state put`).
- The `embedded-cpu` backend is a placeholder on every non-Apple-Silicon build. Issues against it close and reopen in a cycle (#1500 → #1523 → #1735 → #1760). This needs an owner decision, not another bug report.
- No `.claude/memory/qa_agent_role.md` on main; prompt is the only contract.
- Next Slot C candidate: #11 `caro ai --continue-session` (also never). Expect the same #1760 failure; consider #13 `caro suggest` instead.

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
