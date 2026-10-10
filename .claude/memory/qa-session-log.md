# QA Session Log

Reading order: most recent first.

---

## 2026-05-24 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C (surface #10: `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 12s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)` — upgraded from 1.3.0 since last run; v1.4.0 released 2026-05-09
- `caro --help` → **PASS**: all subcommands listed; new CaroML and AI verbs visible
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001, 2nd occurrence in 7 days; model download blocked in sandbox — see qa-known-flakes.md)

### Slot B — Recent diff (30 PRs merged since 2026-05-07)

Representative surfaces spot-checked:

- **PR #1110** (runtime-loadable custom patterns): `examples/patterns.example.toml` exists with correct schema. Safety tests pass 20/20. Critical-bypass guard confirmed in `src/safety/mod.rs:407–430` (allowlist skipped when Critical built-in match present). PASS.
- **PR #1109** (`--edit` flag): Flag appears in `--help` with correct description. Source wired in `src/main.rs`. PASS (can't test interactive editing in sandbox).
- **PR #1065** (GNU mkfs flag-evasion pattern): `cargo test --lib -- safety` → 20/20 PASS.

No regressions found in any spot-checked surface.

Surfaces flagged for future Slot C coverage: PR #1108 (candidate pipeline), PR #1112 (`caro.prune` skill) — added to coverage matrix.

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: `--once` documented as "The only mode supported today"; flags accurate.
- `caro ai --once` (no prompt) → **PASS**: graceful error "no prompt provided (pass text, pipe stdin, or use -p)"
- `caro ai --once "list files"` → **FLAKE** (FLAKE-001 — model download blocked; `Backend is not available: Failed to download model after 3 attempts`)
- Unit tests `cargo test --lib -- ai::` → **PASS**: 23/23 (covers session persistence, TTL resume, safety blocking, shell-init rendering, privacy context)
- Session store round-trip: **PASS** (unit test `once_creates_session_and_persists_turn` creates session, persists turn, reloads from disk)
- Safety integration: **PASS** (unit test `once_flags_dangerous_command_from_validator` confirms `rm -rf /` blocked with `allowed=false`)

Overall verdict: **PARTIAL-PASS** — functional logic fully verified through unit tests; end-to-end CLI path blocked by sandbox model-download limitation (FLAKE-001).

### Findings

- No new GitHub issues filed this pass.
- Issue [#1098](https://github.com/wildcard/caro/issues/1098) (CLAUDE.md MSRV 1.83 stale) remains open; version field was partially fixed (now shows 1.4.0) but MSRV is still 1.83 in CLAUDE.md. Confirmed stale again today.
- FLAKE-001: 2nd occurrence (2026-05-07 + 2026-05-24) — not yet at 3/7-day promotion threshold.

### Followups

- FLAKE-001 is at 2/3 occurrences needed for regression promotion. Next occurrence should file an issue.
- Issue #1098 (MSRV drift) still open — needs a fix PR. The `release-version-alignment.md` 6-file checklist still does not include CLAUDE.md, as noted in both #1044 and #1098.
- Surface #10 (`caro ai --once`) should be re-tested from an environment with a pre-downloaded model to get full end-to-end coverage. Add to next Slot C candidate list.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or surface #12 (`caro assess`).
- PR #1108 candidate pipeline and PR #1112 `caro.prune` skill should be exercised in a future Slot C.

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
