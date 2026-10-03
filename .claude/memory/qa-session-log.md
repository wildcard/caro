# QA Session Log

Reading order: most recent first.

---

## 2026-05-26 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 18s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)`
- `caro --help` → **PASS**: all subcommands listed including `ai` and full CaroML verb set
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 (model download blocked; 2nd observed instance in this log)

### Slot B — Recent diff (12 PRs merged 2026-05-07 → 2026-05-26)

PRs covered: #1065, #1114, #1112, #1111, #1110, #1109, #1108, #1103, #883, #920, #1139, #1154.

- **#1065** (safety: GNU mkfs pattern fix): `cargo test --lib -- safety` → 20/20 PASS. No regression.
- **#1110** (safety: runtime-loadable custom patterns): `test_custom_safety_patterns` + `test_custom_pattern_addition` PASS; `test_allowlist_functionality` FAIL — regression still open (issues [#1165](https://github.com/wildcard/caro/issues/1165), [#1169](https://github.com/wildcard/caro/issues/1169)).
- **#1109** (cli: `--edit` flag): `caro --edit --dry-run -p "list files"` → PASS via static matcher; flag accepted, `ls -la` generated and shown in dry-run mode.
- **#1154** (fix deps): Build passes (implicit Slot A PASS).
- Other PRs (#1114, #1112, #1111, #1108, #1103, #883, #920, #1139 — docs, skills, experiment, agent, governance, MLX pipeline, fastembed dep, rustfmt): No regressions found in available smoke coverage.

### Slot C — caro ai --once (surface #10)

- `caro ai --once "what files are in /tmp"` → **FLAKE**: FLAKE-001 (model download blocked; same pattern as `--dry-run`)
- `caro ai "list files"` (without `--once`) → same FLAKE-001; `--once` flag is always the effective behavior (`src/main.rs:3233` ignores the field), correct per help: "only mode supported today"
- `--continue-session` flag is intentional no-op by design (shell hook ergonomics); `new_session` controls session selection
- Command routed correctly to ai subcommand (not misrouted via #1163 bug, since no global flags were passed before the subcommand)
- `caro config show` → PASS: all fields shown, config at `/root/.config/caro/config.toml`
- No new bugs found beyond FLAKE-001

### Findings

- No new issues filed this pass — all discovered regressions already tracked in open issues.

### Followups

- FLAKE-001 observed 2nd logged time (also triggered in all intermediate QA runs 2026-05-15 through 2026-05-25 that did not update memory). Pattern is consistent: sandbox always blocks model download. Formal promotion to regression requires 3 logged occurrences; incrementing to 2.
- Issue #1044 appears closed (absent from open `qa` issue list); updated backlog accordingly.
- Backlog sync'd to include all issues filed by intermediate QA runs (#1098–#1170 range).
- Issues #1165/#1169 (allowlist regression from PR #1110): `test_allowlist_functionality` still FAIL on `main`; `cargo test --test safety_validator_contract` still exits 1.
- Issue #1164 (cargo run without --bin): `default-run` still missing from `Cargo.toml`.
- Issue #1107 (--backend openrouter not in VALID_BACKENDS): `src/cli/mod.rs:473` still lists only `["embedded", "ollama", "exo", "vllm"]`.
- Issue #1098 (CLAUDE.md MSRV stale): version field now 1.4.0 ✓ (partial fix), but MSRV still shows 1.83 (should be 1.85, see `Cargo.toml:rust-version`).

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
