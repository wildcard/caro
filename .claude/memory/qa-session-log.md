# QA Session Log

Reading order: most recent first.

---

## 2026-11-23 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 17s)
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS**: all subcommands listed (`assess` absent, matches the 1.5.0 README note)
- `caro doctor` → **PASS**: advisory only (no model, no Ollama), exit 0
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` from the static matcher in 0.4 s

### Slot B — Recent diff

The last log entry on `main` is 2026-05-07 (see #1603: later QA memory PRs did not merge). 50 commits landed since then. I spot-checked the newest feature and fix merges:

- #1740 `caro config reset <key>` → **PASS**: set `log_level debug`, reset to `Info`. An unknown key exits 1.
- #1737 `caro test --help` → **PASS**: lists only `static, embedded`.
- #1710 `caro ai` with no prompt → **PASS**: the error suggests `caro ai --once "..."` or stdin, not `-p`. Exit 1.
- #1687 safety normalization → **PASS** with `caro guard test`: `bash <(curl …)` is high/ask. `eval "$(echo cm0gLXJmIC8= | base64 -d)"` is critical/deny with floor. A plain `echo … | base64 -d` is safe.
- #1685 `caro guard audit verify/tail/report` → **PASS** on an empty log only. Flagged in the matrix for a populated-log pass.

### Slot C — `caro skill install` (matrix #24)

The matrix row with the oldest date is #10 (`caro ai --once`). Open issue #1760 already tracks it, and recent QA issues cover #11–#17. I picked #24 instead: it was never tested and has no QA issue.

- From the repo root: install and uninstall work. Both files are copied to `~/.claude/skills/caro-scaffold/`.
- From any other directory, including `src/`: `Error: skill source not found at .claude/skills/caro-scaffold`, exit 1.
- Cause: `src/caroml/skill.rs:34` uses a relative source path. `Cargo.toml` `include` has no `.claude/`, so crates.io users never have the files.
- A second `uninstall` prints "No skill installed" and exits 0. That is fine.

### Findings

- [#1782](https://github.com/wildcard/caro/issues/1782) — `cli: caro skill install fails outside the caro repo root` (P1)

### Followups

- Prompt drift: the routine prompt asks for the model ID `claude-sonnet-4-6` in issue and PR bodies. The harness forbids model IDs in pushed artifacts, so this run omitted it. `qa_agent_role.md` is still not on `main`.
- Prompt drift: the routine prompt names branch `claude/qa-rotation-<date>`. The session pins `claude/peaceful-hopper-44t2sh`, so this run used that branch.
- The memory files on `main` lag behind the daily runs (#1603). Coverage dates here can be older than the real last test. Check the recent `qa`-labelled issues before you pick Slot C.
- FLAKE-001 is not reproducible with the current smoke prompt (static matcher). Use an LLM-only prompt to exercise the model download again.

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
