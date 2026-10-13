# QA Session Log

Reading order: most recent first.

---

## 2026-10-13 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (25+ PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 47s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (7493272 2026-10-13)`
- `caro --help` → **PASS**: all subcommands present; new 1.5.0 surface `export` listed correctly
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox); proxy detected at `http://127.0.0.1:40257`
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static_matcher returned `ls -la` immediately (no model needed); telemetry consent appeared again as expected in fresh container (config not persisted across sessions)

### Slot B — Recent diff

PRs merged since 2026-05-07: 25+ (v1.5.0 release + post-release fixes). Key surfaces exercised:

- **Safety (PR #1614, #1600)**: `cargo test --test safety_segment_evasion_contract` → 9/9 PASS; `cargo test --test e2e_guard` → 9/9 PASS including `enforce_catches_segment_and_wrapper_evasions`. Segment- and wrapper-aware validation confirmed working.
- **Safety unit tests via `cargo test safety`** → 12/12 website_claims PASS (filtered run via website_claims test binary).
- **Embedded backend (PR #1616)**: llama.cpp context window raised to 4096 tokens; change verified in build, no runtime test possible without model.
- **Agent fix (PR #1618)**: stop sending second system prompt — build-level verification only in sandbox.
- **Docs/release (PRs #1635, #1637, #1650)**: README `**Current Version:** 1.5.0` confirmed; CLAUDE.md version correctly deferred to Cargo.toml (`version = "1.5.0"`) — issue #1044 fix verified and holding.

Surfaces flagged for future Slot C: `caro export` (new in 1.5.0, never tested — add to coverage matrix), homebrew tap formula (#28).

### Slot C — `caro ai --once` (surface #10)

Surface chosen: `caro ai --once` scripted conversational mode (oldest 'never' row in coverage matrix, lowest # among 'never' group).

- `caro ai --help` → PASS: flag list correct; `--once` documented as "the only mode supported today"
- `caro ai --once "show disk usage"` (5s timeout) → **HANG**: zero output on stdout AND stderr; process killed at timeout with no exit message
- `caro --backend ai-horde ai --once "show disk usage"` → **FAIL**: `remote-backends` feature not compiled in this build (expected; embedded-cpu build only)
- Root cause traced to `run_ai_once()` → `CliApp::with_overrides().await` in `src/main.rs:1152–1160` blocking indefinitely on embedded model download with no progress output; model download progress uses `info!()`/`warn!()` log macros (invisible without `RUST_LOG=info`)
- **Existing open issue**: [#1408](https://github.com/wildcard/caro/issues/1408) (P1) — "caro ai --once provides zero user feedback during model initialization"; open fix PR [#1415](https://github.com/wildcard/caro/pull/1415) ("fix: show ai once backend initialization status") not yet merged as of 2026-10-13
- No new issue filed (duplicate of #1408)

### Findings

- None new this pass — surfaces under test all in good shape except for known P1 #1408 (open, fix PR #1415 pending merge)

### Followups

- FLAKE-001 (model download blocked in sandbox) continues to affect all `ai` and `embedded` backend tests. Observed again this pass on `caro ai --once`; count: 2 observations over 159 days, still below the 3-in-7-days regression threshold.
- PR #1415 has been open since 2026-08-15 (≈59 days). If it doesn't merge before the next QA pass, consider adding a comment to nudge.
- Add `caro export` (new in v1.5.0) to coverage matrix as surface #33 for future Slot C rotation.
- Add homebrew tap formula (surface #28) to the next priority slot — never tested, install path risk.

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
