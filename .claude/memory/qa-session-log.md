# QA Session Log

Reading order: most recent first.

---

## 2026-10-21 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC (fired 14:16).
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** (2m 59s)
- `caro --version` → **PASS**: `caro 1.5.0 (07679f0 2026-10-19)`
- `caro --help` → **PASS**
- `caro doctor` → **PASS**: advisory only; now also shows guard hook status
- `caro -p 'list files in current directory' --dry-run` → **PASS** (`ls -la`; model download not needed this time)
- Note: `gh` token was invalid (GH_TOKEN); GitHub MCP tools used instead.

### Slot B — Recent diff
Last log entry was 2026-05-07, so the window is large. Covered the newest merges:
- #1683 `caro guard test` / `guard setup` → exercised; text + JSON + exit codes 0/2 PASS. Flag-order defect found (#1697).
- #1680 README: `caro assess` not available in 1.5.0 → confirmed `caro assess` is not a subcommand; it is treated as a prompt.
- #1669 rustls/audit, #1667 ai_horde header → not exercised (CI/network surfaces).

### Slot C — `caro ai --once` (surface #10)
- `caro ai --once "list files"` (also `--new-session`, other prompts) → **FAIL** on the embedded-cpu build: always `Clarification needed: What exactly should be deleted?`, exit 1.
- Cause: `src/backends/embedded/cpu.rs:62-64` is a keyword-matching stub; the `ai` system prompt contains "rm". Same root cause as #1269 (open, canonical). Duplicates #1277 #1334 #1355 #1406 #1430 #1473 #1494 #1500 are closed. Not re-filed.
- `caro -p assess --dry-run` hits the same stub clarification.

### Findings
- [#1697](https://github.com/wildcard/caro/issues/1697) — `cli: caro guard test "<cmd>" --output json swallows --output` (P2)
- Existing: #1269 still reproduces on 1.5.0 (no new issue).

### Followups
- Surface #12 `caro assess` is retired in 1.5.0; removed from candidates.
- #1269 has been reported 9 times by QA and users. Consider raising its priority.
- FLAKE-001 (model download) did not occur this pass.

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
