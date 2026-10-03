# QA Session Log

Reading order: most recent first.

---

## 2026-05-25 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (25 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 19s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)`
- `caro --help` → **PASS**: all subcommands present including `ai`, `export`, `skill`; `--edit` flag visible
- `caro doctor` → **PASS**: advisory (no model downloaded; expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001, 2nd occurrence: model download fails in sandbox after 3 retries)
- Telemetry consent: **PASS** — disabled consent shown once, not re-prompted
- Issue #1044 (CLAUDE.md version drift) verified **CLOSED** and fixed — CLAUDE.md now reads `1.4.0 (GA)` ✓

### Slot B — Recent diff

25 PRs merged since 2026-05-07. Key surfaces exercised:

- **PR #1065** (mkfs pattern flag evasion fix) — `cargo test --lib -- safety` → **PASS** (20/20 safety unit tests)
- **PR #1110** (runtime-loadable custom patterns) — patterns.toml sibling file confirmed present at `examples/patterns.example.toml`; `--edit` flag present in help; `from_user_config` path confirmed in source; however `test_allowlist_functionality` in `safety_validator_contract` → **FAIL** (regression introduced by #1110's Critical pre-scan guard)
- **PR #1109** (`--edit` flag) — `--edit` visible in `caro --help` output → **PASS** (structural check only; requires model for full exercise)
- i18n batch (#817–829) — deferred to surface #31 in Slot C future pass

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: flags documented (`--once`, `--new-session`, `--continue-session`)
- `caro ai` (no args) → **PASS**: `Error: no prompt provided (pass text, pipe stdin, or use -p)` — correct
- `caro ai --once "list files"` → **FLAKE** (FLAKE-001: model download blocked in sandbox, same root cause as Slot A dry-run)
- `echo "show disk usage" | caro ai --once --new-session` → **FLAKE** (same)
- CLI surface structural check: **PASS** — arg parsing, help text, no-arg error handling all functional

### Findings

- [#1169](https://github.com/wildcard/caro/issues/1169) — `safety: test_allowlist_functionality broken by PR #1110 Critical pre-scan guard` (P1 — regression)
- [#1170](https://github.com/wildcard/caro/issues/1170) — `cli: cargo test safety fails — evaluation binary rejects positional filter arg` (P2)

### Followups

- FLAKE-001 observed 2nd time (2026-05-25). Still below 3× threshold for regression reclassification. Threshold reached on next observation.
- Surface #31 (i18n locale smoke) should be exercised next pass — 10 locale files updated in batch (#817–829).
- Custom patterns feature (#1110) has no unit tests for `from_user_config` path — gap noted; not a separate issue since #1169 covers the immediate regression.
- `caro ai` structural surface: mark as partially tested (CLI layer PASS, backend layer FLAKE). Would need environment with pre-downloaded model or Ollama for full pass.

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
