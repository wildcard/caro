# QA Session Log

Reading order: most recent first.

---

## 2026-11-01 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** (2m 33s)
- `caro --version` → **PASS**: `caro 1.5.0 (d0f4d3f 2026-10-27)`
- `caro --help` → **PASS**: lists `guard`, `ai`, `suggest`, CaroML verbs
- `caro doctor` → **PASS**: advisory only (no model, no Ollama; expected in sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la`
- Note: `gh` token invalid (GH_TOKEN); GitHub MCP tools and `gh api search` worked and were used instead.

### Slot B — Recent diff
Memory on main is stale (last entry 2026-05-07; QA PRs #1733-#1741 were closed unmerged). Sampled only the newest merged PR:
- #1687 (safety normalization, spec 011 P2.6): `caro guard` on `eval "$(echo cm0gLXJmIC8= | base64 -d)"` → shadow "would deny: critical" → **PASS**. `ls -la` → no opinion → **PASS**.

### Slot C — `caro ai --once` (#10)
- `caro ai --once "list files"` → exit 1, `Clarification needed: What exactly should be deleted?` → **FAIL (known)**. Same for `"show disk usage"`.
- `caro ai --once ""` → clear "no prompt provided" error → PASS.
- Cause: stub in `src/backends/embedded/cpu.rs:62` matches `rm`/`delete` in the prompt text.
- Already tracked: #1269 (open, canonical, waits on owner decision); #1735, #1673, #1523, #1494, #1500 closed as dupes. No new issue filed.

### Findings
- None new. Known bug #1269 re-confirmed on d0f4d3f.

### Followups
- Memory files on main lag behind (several unmerged QA PRs). Matrix rows from May reflect only the bootstrap run.
- Next Slot C candidate: #11 `caro ai --continue-session`.

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
