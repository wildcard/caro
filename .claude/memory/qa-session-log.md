# QA Session Log

Reading order: most recent first.

---

## 2026-10-02 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (25 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 50s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (1ad0631 2026-09-30)` (upgraded from 1.3.0 at last run)
- `caro --help` → **PASS**: all subcommands present including new CaroML verbs
- `caro doctor` → **PASS**: advisory (no model at start; proxy detected and HuggingFace reachable)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` immediately (no model download attempt — static matcher improvement since 1.3.0)
- Note: telemetry consent prompt appeared (fresh sandbox), resolved to disabled

### Slot B — Recent diff

PRs merged since 2026-05-07 (25 PRs from first API page only; pagination not applied — future Slot B passes should fetch all pages to avoid missing older merges):
- **#1315** `fix(safety): P0 — close quote/escape evasion`: `cargo test --lib -- safety` → **PASS** (35/35 tests; 16 new tests since bootstrap including smart_blend_tests, allowlist_catastrophic_tests)
- **#1487** `fix(static-matcher): accept current-directory qualifier in Pattern 43`: validated by Slot A dry-run returning `ls -la` for "list files in current directory" — **PASS**
- **#1459** `feat(decision): typed decisions, calibration, constrained decoding`: safety suite PASS (35/35); decision/calibration unit tests use `decision::` / `evaluation::calibration::` paths not captured by the `-- safety` filter — not separately verified this run
- **#1488** `fix(deps): reqwest/h2/rustls security bumps`: build compiled clean — **PASS**; CVE assessment: explicit `cargo audit` not run in this QA session; relies on Security Audit CI job passing on #1488's merge CI
- i18n, brand, docs, CI PRs: flagged for Slot C coverage matrix tracking (website surfaces #25–#31)

### Slot C — `caro ai --once` scripted conversational mode (surface #10)

Surface chosen: #10 (`caro ai --once`) — oldest "never" by lowest `#` number.

**Testing procedure:**
1. `caro ai --once "show disk usage"` (no stdin redirect, non-TTY) → **FAIL (P1): hung indefinitely** (exit 124 after 15s timeout). Reproduced 2×.
2. `caro ai --once "show disk usage" < /dev/null` → completed; returned `Error: backend error: Clarification needed: What exactly should be deleted?`
3. `caro ai --once "list files" < /dev/null` → same error for a completely different prompt
4. `caro --backend static ai --once "list files" < /dev/null` → `Unknown backend 'static'` (static backend not exposed to end users via CLI)
5. Investigated root cause: model downloaded successfully (qwen2.5-coder-1.5b-instruct-q4_k_m.gguf) during background invocation; CPU stub's `infer()` matches on system prompt (which contains "delete"/"rm" in safety instructions) → all queries hit the `needs_clarification` branch.

**Verdict**: FAIL — two distinct bugs found:
- P1: stdin-read hang in non-TTY (#1499)
- P2: CPU stub keyword match on system prompt returns misleading clarification (#1500)

Also incidentally found: CLAUDE.md version drift recurred (shows 1.4.0, actual 1.5.0) → #1501.

### Findings

- [#1499](https://github.com/wildcard/caro/issues/1499) — `ai: caro ai --once hangs in non-TTY environments` (P1)
- [#1500](https://github.com/wildcard/caro/issues/1500) — `embedded: CPU stub returns misleading clarification for all queries` (P2)
- [#1501](https://github.com/wildcard/caro/issues/1501) — `docs: CLAUDE.md version banner 1.4.0 instead of 1.5.0` (P2)

### Followups

- FLAKE-001 (model download failure) did NOT reproduce this run: model downloaded successfully in a background invocation. Marking occurrence gap in known-flakes. Two prior occurrences needed for promotion to regression.
- Slot C surface #10 is FAIL. Next Slot C candidate: surface #11 (`caro ai --continue-session`) or #12 (`caro assess`) — both "never" tested.
- 25 merged PRs since last run (first API page only — pagination not applied). Actual merge count since 2026-05-07 may be higher; next Slot B should paginate all pages and note any coverage gap. Website i18n, brand, design-system surfaces not tested — recommend adding website surfaces #25–#31 to upcoming Slot C cycles.
- Previous QA rotation PRs (#1178, #1373, #1443, #1477, #1496) are still open/unmerged — their session logs are not on main. This run's state is correctly based on main.

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
