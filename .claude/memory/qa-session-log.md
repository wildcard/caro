# QA Session Log

Reading order: most recent first.

---

## 2026-10-04 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (119 PRs merged since last run) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 09s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (75d32f7 2026-10-03)`
- `caro --help` → **PASS**: all subcommands listed; `ai`, `suggest`, `assess`, `config`, all CaroML verbs present
- `caro doctor` → **PASS**: advisory (model not yet downloaded at that point; embedded backend detected but needs model)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` immediately (no model download needed for static path)
- Note: model `qwen2.5-coder-1.5b-instruct-q4_k_m.gguf` was auto-downloaded to `/root/.cache/caro/models/` during or shortly after the build step. FLAKE-001 (HuggingFace block) did NOT reproduce this run.

### Slot B — Recent diff (119 PRs since 2026-05-07; focused on 9 since 2026-09-01)

**PRs reviewed**: #1509 (explain STE-lite), #1503 (ai --once stdin fix), #1497 (config key acceptance), #1487 (static-matcher current-directory qualifier), #1488 (security deps bump).

- `caro config show` / `config get backend|safety|log_level|cache_max_size|log_rotation|telemetry` → **PASS**: all `config show` keys accepted by `config get` (PR #1497 fix verified)
- `caro config set safety permissive` → `config get safety` → `config set safety moderate` → **PASS**: round-trip correct
- `caro -p "show disk usage" --explain` → **PASS**: STE-lite output ("Use `ls` to list…", per-option bullets, ≤25 words per sentence) (PR #1509 fix verified)
- `caro -p "find python files in current directory" --dry-run` → **PASS**: `find . -name "*.py" -type f` (PR #1487 current-directory qualifier verified)
- `caro -p "list files here" / "show files in this directory" --dry-run` → **PASS**: `ls -la` (PR #1487)
- `caro ai --once "show disk usage"` (PR #1503) → stdin no longer blocks (**PASS** for #1503 fix); but backend returns wrong error (**FAIL** — see Slot C and issue #1523)

**Surfaces flagged for future Slot C**: #1488 security deps (reqwest/h2/rustls) — no direct CLI surface to exercise, but worth a `cargo audit` next cycle.

### Slot C — `caro ai --once` (surface #10, never previously tested)

**What was exercised:**
- `caro ai --once "show disk usage"` (with and without stdin pipe)
- `caro ai --once "list running processes"`
- ~~`caro ai --once --backend static "show disk usage"`~~ (invalid test: `ai` uses `trailing_var_arg`, so `--backend static` after the subcommand is consumed as prompt text, not a flag; the embedded backend received literal `--backend static show disk usage` as the query — see correction below)
- Inspected `src/backends/embedded/cpu.rs` and `src/backends/embedded/embedded_backend.rs` to confirm root cause

**Finding — FAIL (P1):**
Every invocation that reaches the embedded CPU backend returns:
```
Error: backend error: Clarification needed: What exactly should be deleted?
```

Root cause: `cpu.rs:63` checks `prompt.contains("rm")` against the full system prompt, which always contains "rm -rf" in the destructive-commands rule (line 218 of `embedded_backend.rs`). The CPU backend is an unfinished placeholder; the condition should check only the user request text, not the compiled system prompt.

The static matcher IS correct (`caro -p` path) — `ai --once` bypasses it by calling `backend_arc()` directly instead of routing through the AgentLoop. **Note**: the same bug affects any embedded CPU path (e.g. `caro --force-llm -p ...` or a static-matcher miss), because `EmbeddedModelBackend::generate_command` builds the same full system prompt and passes it to `CpuBackend::infer`. Surface #19 (embedded model backend) is also affected. See [#1523](https://github.com/wildcard/caro/issues/1523).

### Findings

- [#1523](https://github.com/wildcard/caro/issues/1523) — `ai: caro ai --once returns wrong "Clarification needed" error for every query` (P1)

### Followups

- FLAKE-001 (HuggingFace model download) did NOT reproduce this run — update occurrence log.
- `caro ai --once 'query' --dry-run` timed out (15s): hypothesis is this was the model download trigger (download started, timeout elapsed before completion). Should verify next cycle with `--dry-run` after warm cache.
- `caro ai --once` with `--dry-run` flag: trailing_var_arg on ai subcommand may consume `--dry-run` as part of the prompt text. Worth a unit test.
- #1488 security deps (RUSTSEC-2026-0258, -0285): add `cargo audit` to a future Slot B check.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or #12 (`caro assess`).

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
