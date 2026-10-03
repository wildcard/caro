# QA Session Log

Reading order: most recent first.

---

## 2026-05-23 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 15s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)`
- `caro --help` → **PASS**: all subcommands present including new `--edit`/`-e` flag visible in top-level flag list
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 reproduced (2nd observation total, 16 days apart — not within 7-day window, threshold not reached)

### Slot B — Recent diff

PRs merged since 2026-05-07: 30 total. Representative surfaces tested:

- **PR #1065** (`Fix: GNU mkfs pattern flag evasion`, `src/safety/patterns.rs`): `cargo test --test safety_validator_contract` → **20 PASS, 1 FAIL** (`test_allowlist_functionality`). Filed [#1165](https://github.com/wildcard/caro/issues/1165). Note: the failure was introduced by PR #1110, not #1065.
- **PR #1109** (`feat(cli): --edit flag via rustyline`, `src/cli/edit_prompt.rs`, `src/main.rs`): `caro --help` shows `-e, --edit` flag with correct description → **PASS**; flag is present and properly documented.
- **PR #1110** (`feat(safety): runtime-loadable custom patterns`, `src/safety/mod.rs`, `tests/custom_patterns_toml.rs`): `cargo test --test custom_patterns_toml` → **9/9 PASS**. Feature works; see regression note above.

Other PRs observed: i18n updates (#817–#829), CaroML additions (#893+), docs (#1114), build fix (#1154, confirmed resolved by successful build).

### Slot C — caro ai --once (surface #10)

Surface chosen: `caro ai --once` scripted conversational mode (oldest 'never' surfaces — lowest # rule, surface #10).

- `caro ai --help` → **PASS**: subcommand documented with `--once`, `--new-session`, `--continue-session` flags; description matches advertised "Atuin-AI-style" conversational mode; stdout-only design for shell widget injection documented.
- `caro ai --once <prompt>` (with stdin) → **FLAKE**: same FLAKE-001 model download failure; error message clean and actionable.
- CLI structure and flag parsing: **PASS** — no structural bugs found in the subcommand.

### Findings

- [#1165](https://github.com/wildcard/caro/issues/1165) — `safety: user allowlist cannot override rm -rf /tmp/... despite explicit allowlist pattern` (P1, regression from #1110)

### Followups

- FLAKE-001 second observation (2026-05-23), but gap is 16 days so 7-day window is clean. If reproduced again within 7 days of this run, promote to regression.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or #12 (`caro assess`).
- PR #1110 regression: `test_allowlist_functionality` fails on main; CI must have missed it or the test was newly broken between merge and CI run. Consider whether `cargo test` is gated in CI for integration tests.

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
