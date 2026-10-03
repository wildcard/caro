# QA Session Log

Reading order: most recent first.

---

## 2026-10-03 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (9 PRs merged since 2026-05-07) + C (surface #10: `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 22s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (75d32f7 2026-10-03)`
- `caro --help` → **PASS**: all subcommands listed including `ai`, `suggest`, `config`, `skill`, CaroML verbs
- `caro doctor` → **PASS**: advisory (no model downloaded, sandbox; expected)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` (no model download needed this pass — FLAKE-001 not triggered)
- Telemetry consent shown (fresh sandbox config); auto-disabled via first-run prompt

### Slot B — Recent diff (9 PRs merged since 2026-05-07)

PRs covered:
| PR | Surface touched | Verdict |
|----|----------------|---------|
| #1503 | `ai --once` stdin blocking fix | PASS (stdin is /dev/null with trailing args) |
| #1497 | `config set/get/show` key alignment | PASS (all `config show` keys accepted by `config get/set`) |
| #1487 | static matcher Pattern 43 current-directory qualifier | PASS (`ls -la` for "list files in current directory" and "show files here") |
| #1488 | reqwest 0.12 / h2 / rustls security upgrade | PASS (reqwest 0.12.28 in Cargo.lock, binary functional) |
| #1505 | CI toolchain pin (Rust 1.98.1) | not exercised (CI-only change) |
| #1489 | eval ECE/Pareto release gate | not exercised (eval harness requires model) |
| #1459 | typed decisions, calibration, measured-confidence | not exercised (requires backend inference) |
| #1470 | executor timeout, guard hooks, loop budgets | not exercised (requires backend inference) |
| #1509 | STE-lite explain mode | not exercised (requires model) |

Note: `ai --once` stdin fix (PR #1503) is confirmed working, but the underlying P1 CPU stub bug (#1269) persists — see Slot C.

### Slot C — `caro ai --once` (surface #10)

**Surface chosen**: #10 — oldest 'never' tested (lowest # tie-break rule).

Commands exercised:
```bash
./target/release/caro ai --once 'list running processes'
./target/release/caro ai --once --new-session 'show disk usage'
./target/release/caro ai --once --new-session 'find large files'
```

All three return: `Error: backend error: Clarification needed: What exactly should be deleted?` (exit code 1) after ~100 second wait.

**Root cause confirmed** (from cpu.rs:63-64):
- CPU stub checks `prompt.contains("rm")` against the FULL prompt (system prompt + user query)
- System prompt in `embedded_backend.rs:218` contains `"rm -rf"` → match fires every time
- Backend falls back to CPU stub after model download times out (~100s, FLAKE-001 environment condition)

**Behaviour vs v1.4.0**: exit code changed 0→1; message changed from `echo 'Please clarify your request'` to `Clarification needed: What exactly should be deleted?`

**Known issue**: #1269 (open since 2026-06-26, P1). Added reproduction comment with v1.5.0 behaviour delta.

### Findings

- [#1520](https://github.com/wildcard/caro/issues/1520) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurring drift)` (P2) — **new issue**
- [#1269](https://github.com/wildcard/caro/issues/1269) — `ai: caro ai --once always returns wrong error regardless of input` (P1) — **confirmed still reproducing in v1.5.0**; added comment with v1.5.0 behaviour delta

### Followups

- FLAKE-001 observed again (model download required for ai --once; times out in sandbox). Second observation (2026-05-07 + 2026-10-03). Does not meet 3-in-7-days threshold for reclassification.
- CLAUDE.md version drift has recurred for 3rd time — root cause fix (add to checklist) still needed (#1520).
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — surface #10 only partially exercised due to FLAKE-001.
- **Slot B coverage gap**: This pass reviewed 9 PRs (#1459–#1509) but first-parent git history since the previous QA commit (c9a31a3) contains additional merged PRs not in the table (#1246, #1301, #1304, #1306, #1316, #1317, #1346). These are older feature/docs/release PRs from the May–Sept 2026 period that were not formally reviewed. Next Slot B run should baseline from the earliest unreviewed PR rather than the previous session date alone.

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
