# QA Session Log

Reading order: most recent first.

---

## 2026-06-13 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (29 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 21s, 0 errors; version 1.4.0)
- `caro --version` → **PASS**: `caro 1.4.0 (2be886c 2026-06-07)`
- `caro --help` → **PASS**: all subcommands listed; new `--approval` flag present; `mesh`/`ai-horde`/`hybrid` discoverable via error path (not via --backend-info — see findings)
- `caro doctor` → **PASS**: no model downloaded, advisory only; HuggingFace reachable over HTTP; embedded "needs model download" as expected in sandbox
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001, 2nd observation — outside the original 7-day window; new 7-day window started today)
- Telemetry prompt shown on first invocation (expected — fresh sandbox, no prior config); defaulted to disabled in non-TTY context

### Slot B — Recent diff

PRs merged since 2026-05-07: 29 total. Key surfaces exercised:

- **PR #1209** (feat: Mesh-LLM + AI-Horde + hybrid backends) → tested via `--backend-info` and `--backend` flag → **FINDING**: `print_backend_info()` not updated; shows `static`/`claude` (invalid) and omits `exo`/`mesh`/`ai-horde`/`hybrid` → **filed #1221 (P1)**
- **PR #1110** (feat: runtime-loadable custom patterns) → reviewed `examples/patterns.example.toml`, `src/safety/mod.rs` — implementation looks correct; 27 safety lib tests PASS
- **PR #1206** (feat: smart approval mode spike) → `--approval` flag present in `--help`; 568 lib tests PASS; 6 new smart-blend tests confirm hard-floor-Critical contract
- **PR #1211** (test: make dangerous-command confirm test deterministic) → lib tests pass; did not re-run the specific test as it is lib-level only
- **Cross-PR finding**: `e2e_safety_level_configuration` fails when only release binary built — `cargo run` fallback in `CliTestRunner` lacks `--bin caro`; two-binary workspace breaks it → **filed #1222 (P1)**

Full test results: `cargo test --lib --release --features embedded-cpu` → **568 passed, 0 failed, 1 ignored**

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **`caro ai --once`** — oldest 'never' surface, lowest `#` number (tie-break rule).

- `caro ai --help` → **PASS**: flags (`--once`, `--new-session`, `--continue-session`) documented correctly; help note "the only mode supported today" is accurate
- `caro ai --once "show disk usage"` → **FLAKE** (FLAKE-001 reproduces — model download blocked in sandbox); error message clear and actionable
- `caro ai --once` (no prompt) → **PASS**: exits 1 with `Error: no prompt provided (pass text, pipe stdin, or use -p)` — correct behavior
- `echo "show disk usage" | caro ai --once` (stdin pipe) → **FLAKE** (FLAKE-001; piped prompt accepted correctly, backend blocks)
- `caro ai --once --backend static` → **PASS (expected error)**: correctly rejects `--backend` as unknown arg after subcommand; global `--backend` position confirmed required
- `caro ai` (no --once, no prompt, non-TTY) → **PASS**: exits 1 with clear `no prompt provided` message
- Full functional test of `--once` mode: **BLOCKED** by FLAKE-001; cannot confirm command generation output

Partial verdict: **PARTIAL** — surface is well-structured; blocked by sandbox model download limitation for end-to-end output verification.

### Findings

- [#1221](https://github.com/wildcard/caro/issues/1221) — `cli: --backend-info lists invalid backends (static, claude) and omits valid ones (exo, mesh, ai-horde, hybrid)` (P1, regression from PR #1209)
- [#1222](https://github.com/wildcard/caro/issues/1222) — `cli: e2e_safety_level_configuration test fails — cargo fallback missing --bin caro with multiple binaries` (P1)

### Followups

- #1044 (CLAUDE.md version drift) confirmed CLOSED 2026-05-09; removed from watch list.
- FLAKE-001 second observation (new 7-day window — first was 2026-05-07, >7 days ago). Reset window; needs 3 more observations within 7 days to reclassify as regression.
- `caro ai --once` surface #10: marked PARTIAL. Recommend re-testing from an env with a pre-downloaded model. Surface #11 (`caro ai --continue-session`) shares the same backend blocker — defer until model is available.
- `--backend-info` / `--backend` help string stale pattern: consider a CI lint that diffs `VALID_BACKENDS` in `cli/mod.rs` against the backends listed in `print_backend_info()` to prevent future drift.

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
