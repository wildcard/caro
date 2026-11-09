# QA Session Log

Reading order: most recent first.

---

## 2026-11-09 — Scheduled run (Slot A + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + C (B: no PRs merged; main still at aa1b0e6, 2026-11-02).

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → PASS (2m 12s)
- `caro --version` → PASS (`caro 1.5.0 (aa1b0e6 2026-11-02)`)
- `caro --help` → PASS
- `caro doctor` → PASS (no model cached, advisory only)
- `caro -p 'list files in current directory' --dry-run` → PASS (`ls -la`); FLAKE-001 did not recur

### Slot B — Recent diff
No PRs merged since the last run.

### Slot C — `caro config` + `--output json/yaml`
- `config set safety strict`, `get`, `reset` → PASS; invalid value exits with a clear error listing valid values
- `--output json` stdout parses as valid JSON; `--output yaml` renders correctly → PASS
- `--safety strict -p 'delete everything in /' --dry-run` → asks "What exactly should be deleted?" (clarification, no command emitted)

### Findings
None this pass.

### Followups
- Earlier rotation PRs (#1743, #1744, #1746, #1749, #1751) were closed unmerged, so memory on main still ends at 2026-05-07. #1750 (`caro completion <unknown-shell>` exits 0) was filed 2026-11-05 and is not tracked on main yet.
- `gh auth` token is invalid in the sandbox; GitHub MCP tools work.

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
