# QA Session Log

Reading order: most recent first.

---

## 2026-10-06 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (119 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 20s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (75d32f7 2026-10-03)`
- `caro --help` → **PASS**: all subcommands listed; `ai`, `config`, `suggest`, CaroML verbs present
- `caro doctor` → **PASS**: advisory only (no model downloaded; expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` without triggering model download (FLAKE-001 not triggered this pass)
- `caro --explain 'list files in current directory'` → **PASS**: STE-style explain output with `ls -la`; options described correctly

### Slot B — Recent diff

119 PRs merged since 2026-05-07. Representative surfaces exercised:

- **#1509** `fix(explain): STE-lite explain mode` → **PASS** (`--explain` output verified above)
- **#1503** `fix(ai): don't block on stdin when ai --once has trailing prompt` → **cannot verify**; `caro ai --once` hangs before reaching response stage (root cause: #1440 — no HTTP timeout on model download)
- **#1497** `fix(cli): accept every config show key in config set/get` → **PASS**: `config get backend`, `config set backend embedded`, round-trip verified
- **#1488** `fix(deps): reqwest 0.12 / h2 bump (RUSTSEC-2026-0258)` → binary builds clean; security deps updated at build level
- **#1487** `fix(static-matcher): Pattern 43 current-directory qualifier` → **PASS** (static matcher used for ls query in Slot A dry-run)

### Slot C — `caro ai --once` (surface #10, never previously tested)

- `caro ai --once 'show disk usage'` → **FAIL**: hung with zero output; timed out at 30s (EXIT 124)
- `caro ai --once 'list files in current directory'` → **FAIL**: same hang; timed out at 15s (EXIT 124)
- Confirms **#1408** (zero user feedback during model init) and **#1440** (HfHubClient no HTTP timeout) still open
- Could not reproduce **#1523** ("returns wrong clarification for every query") — command never reaches response stage in this sandbox
- Side finding during config reset investigation: `caro config set backend auto` rejected; no per-key unset path exists → filed **#1530**

### Findings

- [#1530](https://github.com/wildcard/caro/issues/1530) — `cli: caro config set backend has no 'auto' option` (P2) ← **new this pass**
- [#1408](https://github.com/wildcard/caro/issues/1408) — `caro ai --once` zero feedback during model init (P1) ← confirmed still open
- [#1440](https://github.com/wildcard/caro/issues/1440) — model download hangs indefinitely; no HTTP timeout (P1) ← confirmed still open via Slot C
- [#1520](https://github.com/wildcard/caro/issues/1520) — CLAUDE.md shows 1.4.0 not 1.5.0 (P2) ← confirmed still open
- [#1419](https://github.com/wildcard/caro/issues/1419) — caro 1.5.0 never tagged/published; users stuck on 1.4.0 (P0) ← still open per issue list

### Followups

- FLAKE-001 not triggered in Slot A dry-run (static matcher handled query). Occurrence count remains 1 (2026-05-07). Below 3/7-day promotion threshold.
- Issue #1523 ("wrong clarification") cannot be reproduced without a working model; defer to model-enabled environment.
- Surface #14 (`caro config get/set/show/reset`) exercised via Slot B; updated in matrix.
- Surface #10 (`caro ai --once`) now formally tested — FAIL. Pending #1408/#1440 resolution before re-exercising.
- P0 #1419 remains open: binary built from source shows 1.5.0 but crates.io / install.sh users are on 1.4.0.

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
