# QA Session Log

Reading order: most recent first.

---

## 2026-07-22 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 49s)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed including `ai`, `skill`, `suggest`, `shell-init`, CaroML verbs
- `caro doctor` → **PASS**: detects Ubuntu 24.4.0, x86_64, proxy; no model at first invocation (downloads during dry-run)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE/BUG**: model download triggered (~1.1 GB); agent loop timed out during download on first invocation; subsequent runs consistently return `echo 'Please clarify your request'` due to CPU backend stub bug (FLAKE-001 + open #1277, #1334)

### Slot B — Recent diff (30 PRs merged 2026-05-07 → 2026-07-22)

- **Safety (#1315)**: `fix(safety): P0 — close quote/escape evasion`. Library safety tests: 34/34 PASS. Full lib: 597/597 PASS. CLI-level ambiguous due to CPU stub — no new safety regression in unit tests.
- **Release (#1304)**: `chore(release): v1.5.0` — version confirmed correct.
- **Backend roster (#1298)**: `fix(cli): single source of truth for backend roster`. `--backend static` now returns "Unknown backend" — static-matcher is internal-only. Appears intentional; no user-facing removal notice found.
- **Other 27 PRs** (website brand/i18n, discovery docs, automation, ponytail, ML/eval): not smoke-tested; flagged for future Slot C passes.

### Slot C — `caro ai --once` (surface #10, first test)

- `echo "list files" | caro ai --once` → `echo 'Please clarify your request'` ✗ (expected: `ls`)
- `echo "show disk space" | caro ai --once` → `echo 'Please clarify your request'` ✗
- `echo "find and kill the runaway process eating CPU" | caro ai --once` → `echo 'Please clarify your request'` ✗ (same query works via static-matcher in `caro -p`)
- **Result: FAIL** — `caro ai --once` bypasses static-matcher, routes all queries to embedded CPU stub which always returns `echo 'Please clarify your request'` due to system-prompt "rm" contamination. Debug: `Backend: embedded, Confidence: 0.85` on every turn.

### Findings

- [#1366](https://github.com/wildcard/caro/issues/1366) — `docs: CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0` (P2) — **NEW, filed this pass**
- [#1277](https://github.com/wildcard/caro/issues/1277) — CPU backend stub always returns wrong output — **STILL OPEN, REPRODUCED in v1.5.0**
- [#1334](https://github.com/wildcard/caro/issues/1334) — `caro ai --once` broken (CPU stub) — **STILL OPEN, REPRODUCED**
- [#1181](https://github.com/wildcard/caro/issues/1181) / [#1274](https://github.com/wildcard/caro/issues/1274) / [#1362](https://github.com/wildcard/caro/issues/1362) — static matcher Pattern 43 too strict — **STILL OPEN, REPRODUCED**

### Followups

- FLAKE-001 observed again: model download (~1.1 GB) takes >60s; agent loop times out on first invocation. Model completes eventually. Dates: 2026-05-07, 2026-07-22 (>7 days apart; 3×/7-day threshold not met — remains flake).
- `--backend static` silently removed from public CLI (PR #1298). No removal notice found.
- Next Slot C candidate: surface #11 (`caro ai --continue-session`) — never tested.
- #1277 and #1334 are P1, unresolved across v1.4.0 → v1.5.0. If still open at next release, escalate to epic tracker.

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
