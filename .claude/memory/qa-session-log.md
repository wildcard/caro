# QA Session Log

Reading order: most recent first.

---

## 2026-10-16 — Scheduled run (Slot A + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + C. Slot B skipped (`gh` auth invalid; GitHub MCP used for filing only).

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` -> PASS (2m28s)
- `caro --version` -> PASS: `caro 1.5.0 (867323b 2026-10-14)`
- `caro --help` -> PASS
- `caro doctor` -> PASS (no model, Ollama not installed; advisory only)
- `caro -p 'list files in current directory' --dry-run` -> PASS: `ls -la` via static matcher (no model download needed)
- Note: JSON/YAML output shows `executed: true` with `dry_run: true` on a dry run. Not filed; unclear if the field means "would execute". Revisit.

### Slot C — `caro assess` (#12), plus `--output json/yaml` (#15, #16) and `caro ai` (#10)
- `--output json` parses as valid JSON; `--output yaml` renders.
- `caro assess` does not exist in 1.5.0 (`src/main.rs:445` disables it) but README documents it -> filed.
- `caro ai --new-session "<prompt>"` -> FAIL: canned "What exactly should be deleted?". Already tracked (#1269 open; #1673 closed as duplicate). Root cause traced to `src/backends/embedded/cpu.rs:~63` stub; added to #1269.

### Findings
- [#1677](https://github.com/wildcard/caro/issues/1677) — docs: README documents `caro assess` but subcommand is disabled (P2)
- Comment on [#1269](https://github.com/wildcard/caro/issues/1269) with root cause (no new issue)

### Followups
- Only the 2026-05-07 entry is on main; the 2026-10-15 run's log is not merged.
- `gh` token invalid in this sandbox (GH_TOKEN). Routine used GitHub MCP tools instead.
- Next Slot C: `caro suggest`, `caro config`, `completion`.

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
