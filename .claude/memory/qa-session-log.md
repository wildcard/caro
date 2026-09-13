# QA Session Log

Reading order: most recent first.

---

## 2026-09-12 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 53s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed; new `ai`, `export`, `render`, `skill` visible
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 reproduced (model download blocked in sandbox — 2nd total observation, 128 days since first; not within 7-day window, no reclassification)
- Telemetry consent prompt shown on first invocation (fresh sandbox, expected)

### Slot B — Recent diff

110 PRs merged since 2026-05-07. Spot-checked representative surfaces:

- **PR #1315** `fix(safety): P0 — close quote/escape evasion` → `cargo test --release --lib -- safety` → **PASS**: 34/34 safety unit tests including new `test_quote_escape_evasion_is_caught_base` (uses `shell-words` crate, PR verified working)
- **PR #1298** `fix(cli): single source of truth for backend roster` → `caro --backend-info` → **PASS**: clean backend table, all backends listed with correct status
- **PR #1304** `chore(release): v1.5.0` → binary reports correct 1.5.0 version
- i18n batch (PRs #816-#829): flagged for future Slot C exercise (surface #31 — i18n locale smoke)
- **PR #883** `feat(tools): MLX LoRA fine-tune pipeline` → flagged for future Slot C exercise (surface deferred — macOS-only MLX, not exercisable in Linux sandbox)

### Slot C — caro ai --once (surface #10)

Surface chosen: **`caro ai --once` scripted conversational mode** (oldest 'never' surface, lowest #).

- `caro ai --help` → **PASS**: subcommand present with `--once`, `--new-session`, `--continue-session` flags documented
- `caro ai --once "list files in current directory"` → **FAIL**: silent hang, zero output to stderr/stdout after 35 seconds; exits via SIGTERM (code 143)
- `echo "list files" | caro ai --once` → **FAIL**: same silent hang
- `cargo test --release --lib -- ai::` → **PASS**: 23/23 AI unit tests pass with mock backend (runner, session, store, privacy, shell_init all green)
- Root cause investigation: `run_ai_once()` → `backend.generate_command().await` blocks silently during model download retry loop; no progress emitted to stderr unlike regular `caro -p ... --dry-run` path

### Findings

- [#1449](https://github.com/wildcard/caro/issues/1449) — `cli: caro ai --once hangs silently with no output when embedded model is unavailable` (P2)
- [#1450](https://github.com/wildcard/caro/issues/1450) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (re-drift)` (P2)
- [#1044](https://github.com/wildcard/caro/issues/1044) — closed ✅ (CLAUDE.md updated to 1.4.0; 1.5.0 drift filed as new #1450)

### Followups

- FLAKE-001 (model download blocked) — 2nd occurrence total. First on 2026-05-07, second today (128 days apart). Not within 7-day window; log updated but no reclassification.
- Three previous QA PRs (#1178, #1373, #1443) still open — memory files on main only reflect bootstrap pass. Memory files will be accurate once those PRs merge.
- MLX LoRA pipeline (PR #883) cannot be exercised in Linux sandbox — add macOS-only note to surface #19 in coverage matrix.
- `caro ai --continue-session` (surface #11) likely affected by same silent-hang bug as `--once`; should verify after #1449 is fixed.
- i18n locale smoke (surface #31) now has test coverage via PR #1352 — schedule for next Slot C pass.

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
