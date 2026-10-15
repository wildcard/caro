# QA Session Log

Reading order: most recent first.

---

## 2026-10-15 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke
- build (`--features embedded-cpu`) → PASS (2m 36s)
- `--version` → PASS: `caro 1.5.0 (867323b 2026-10-14)`
- `--help` → PASS
- `doctor` → PASS (advisory: no model, expected)
- `-p 'list files in current directory' --dry-run` → PASS: `ls -la` (static matcher; no model download needed)
- Note: `gh auth` token invalid in sandbox; GitHub MCP tools used instead.

### Slot B — Recent diff
54 PRs merged since 2026-05-07 (checked top 10 by recency). Smoke-tested #1614 (segment/wrapper-aware safety): dry-run of "delete everything under root recursively with sudo" returned the literal placeholder `sudo command` (not dangerous, but not a real command; not filed, revisit). Other PRs (#1660, #1659, #1534, #1656, #1633) are harness/CI/test-only; not exercised.

### Slot C — `caro ai --once` (matrix #10)
`caro ai --once "show disk usage"` and `"list files"` both fail: `Clarification needed: What exactly should be deleted?` (3/3). Control `-p` returns `df -h`.

### Findings
- [#1673](https://github.com/wildcard/caro/issues/1673) — ai: `--once` returns deletion clarification for every prompt (P0, regression of #1523/#1494). No open epic covers `ai`, so no epic comment posted.

### Followups
- Investigate `sudo command` placeholder output from #1614 area; consider filing P2 if reproduced.
- Next Slot C: #11 `caro ai --continue-session` (likely same root cause as #1673).

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
