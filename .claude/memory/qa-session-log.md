# QA Session Log

Reading order: most recent first.

---

## 2026-10-01 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (many PRs merged since last run 2026-05-07) + C (surface #10).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 12s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (1ad0631 2026-09-30)`
- `caro --help` → **PASS**: all subcommands listed (ai, suggest, assess, config, completion, skill, etc.)
- `caro doctor` → **PASS**: network reachable, proxy detected, no model downloaded (expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` (telemetry consent shown on first run; piped `y` to proceed; second invocation runs without consent prompt)

### Slot B — Recent diff

PRs merged since 2026-05-07 (5+ months, ~50 PRs). Key surfaces smoke-tested:

- **#1487** `fix(static-matcher): accept current-directory qualifier in Pattern 43` → `cargo test --lib -- test_list_files_with_current_directory_qualifier` **PASS**
- **#1488** `fix(deps): upgrade reqwest 0.12 / h2 / rustls (RUSTSEC-2026-0258, -0285)` → Cargo.lock verified: h2 v0.4.19, rustls v0.23.45 — **noted** (security fix landed)
- **#1459** `feat(decision): Jev gap analysis, clarification gates, constrained decoding` → new surface; flagged for future Slot C (see matrix)
- **#1470** `fix: enforce declared limits (executor timeout, guard hooks, loop budgets)` → new surface; flagged for future Slot C

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **#10 `caro ai --once` scripted conversational mode** (oldest 'never' tested).

- `caro ai --help` → **PASS**: `--once` flag documented; "Run one turn and return — no TTY REPL. The only mode supported today"
- `caro ai --once "list files in current directory"` → **FAIL (P1)**: returns `Error: backend error: Clarification needed: What exactly should be deleted?` for every prompt
- Root cause confirmed: `src/backends/embedded/cpu.rs:63` checks `prompt.contains("rm")` on the full system prompt, which always contains `rm -rf` as a negative example (line 218 of `embedded_backend.rs`). Every invocation fires the deletion-clarification branch.
- `--backend` flag cannot be passed after the `ai` subcommand (clap parse error). Passing it before (`caro --backend static ai ...`) fails with "Unknown backend 'static'". No testable backend avoids this bug without a model download or remote service.

### Findings

- [#1494](https://github.com/wildcard/caro/issues/1494) — `ai: caro ai --once always returns deletion-clarification with embedded-cpu backend` (P1)
- [#1495](https://github.com/wildcard/caro/issues/1495) — `docs: CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0 (regression)` (P2)

### Followups

- FLAKE-001 (model download) not reproduced this pass: Slot A dry-run succeeded via static matcher; model download path not exercised.
- #1044 (CLAUDE.md version drift, P2) is **closed** (2026-05-09) but has recurred at v1.5.0; filed as #1495.
- Previous QA rotation PRs #1178, #1373, #1443, #1477 are still open/unmerged — memory updates are accumulating in open PRs. Consider squashing/closing stale rotation PRs.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — also never tested.
- Consider testing surface #29 (`caro --safety strict/moderate/permissive` modes) given the new decision-module gates in #1459.

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
