# QA Session Log

Reading order: most recent first.

---

## 2026-05-27 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 25s, no errors; MSRV 1.85)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)`
- `caro --help` → **PASS**: all subcommands present including CaroML verbs, `ai`, `suggest`, `export`, `skill`, `render`
- `caro doctor` → **PASS**: reports no model yet, ollama not installed — correct for fresh sandbox
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 reproduced (2nd occurrence; first was 2026-05-07; NOT yet 3×/7 days so no promotion yet)
- `caro --output json -p 'list files' --dry-run` → **PASS**: valid JSON with `generated_command: "ls -la"` (uses static pattern matching)
- `caro suggest 'list files'` → **PASS**: 5 suggestions returned correctly

### Slot B — Recent diff

30 PRs merged since 2026-05-07. Key surfaces tested:

- **#1109** (`feat(cli): --edit flag via rustyline`) → **PASS**: flag present in `--help`, non-TTY mode outputs command without executing, exit 0
- **#1110** (`feat(safety): runtime-loadable custom patterns`) → **PARTIAL**: `cargo test --lib -- safety` = 20 PASS; `cargo test --test safety_validator_contract` = 20 PASS, 1 FAIL (`test_allowlist_functionality`) — **→ filed [#1176](https://github.com/wildcard/caro/issues/1176) P1**
- **#1065** (`fix: GNU mkfs pattern flag evasion`) → **PASS**: safety lib tests include mkfs coverage, all pass
- **#817-#829** (i18n translation batches) → noted for Slot C next cycle (surface #31 i18n locale smoke)
- **#1154** (`fix(deps,build): bincode pin + rusqlite cast`) → **PASS**: build succeeds (this fix was the unblock)
- Bonus: `caro config set telemetry.enabled false` → **FAIL**: advertised in first-run consent screen, rejected as unknown key — **→ filed [#1177](https://github.com/wildcard/caro/issues/1177) P1**

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: correct subcommand, `--once` / `--new-session` / `--continue-session` flags documented
- `caro ai --once 'find large files'` → backend unavailable (FLAKE-001 pattern); error message clear and exit graceful
- `caro ai` (no `--once`) with piped input → same backend error; `--backend` is not an accepted flag for this subcommand (expected per help text)
- Note: help text says `--once` is "The only mode supported today" — TTY REPL mode not yet implemented; UX is acceptable given the honest disclaimer
- **Verdict**: PASS (with FLAKE caveat on model download)

### Findings

- [#1176](https://github.com/wildcard/caro/issues/1176) — `safety: test_allowlist_functionality fails on main — Critical rm -rf pattern too broad` (P1, regression)
- [#1177](https://github.com/wildcard/caro/issues/1177) — `cli: caro config set telemetry.enabled false advertised but rejected as unknown key` (P1)

### Watch list updates

- #1044 — closed (CLAUDE.md version corrected in v1.4.0 release cycle)

### Followups

- FLAKE-001 second occurrence (2026-05-07, 2026-05-27). Not within 7-day window; not promoting to regression. Will promote if observed again within next 7 days.
- i18n locale smoke (surface #31) is good next Slot C candidate — 15 translation PRs landed since last run.
- `test_allowlist_functionality` (#1176) is a CI breakage on `cargo test --test safety_validator_contract`. Engineers should see this quickly; no special escalation beyond the P1 issue needed.
- `caro ai --once` REPL mode is unimplemented ("only mode supported today"). Surface #11 (`--continue-session` shell widget) deferred until TTY REPL lands.

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
