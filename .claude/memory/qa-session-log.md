# QA Session Log

Reading order: most recent first.

---

## 2026-09-29 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:58 UTC.
**Rotation**: A + B (110 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 24s, no errors; binary at `./target/release/caro`)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed (including ai, suggest, assess, skill)
- `caro doctor` → **PASS**: advisory only (no model downloaded; expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: model download silently blocked (FLAKE-001, occurrence #2; same sandbox network limitation as 2026-05-07)
- Full library test suite `cargo test --lib` → **PASS**: 597 passed, 0 failed, 1 ignored (up from 513 on 2026-05-07; new tests added across releases)

### Slot B — Recent diff (110 PRs merged since 2026-05-07)

Key surfaces exercised:

- **Safety (P0 fix #1315** — `fix(safety): P0 — close quote/escape evasion of the command scanner`): `cargo test --lib -- safety` → **PASS** (34 safety tests, 0 failed). P0 regression closed.
- **i18n (#1352** — `fix(i18n): load all locale JSON files; overhaul Hebrew translations`): Hebrew locale (`website/src/i18n/locales/he/`) → **PASS**: 9 JSON files present, all parse cleanly; 15 total locales confirmed.
- **CLI backend roster (#1298** — `fix(cli): single source of truth for backend roster`): `caro --help` backend list → **PASS**: all expected backends visible (embedded, ollama, exo, vllm, mesh, ai-horde, hybrid).
- **v1.5.0 release (#1304)**: `caro --version` confirms 1.5.0 shipped correctly.

Surfaces flagged for future Slot C: website (PR #1325 `feat(website): adopt Codex Pet Web SDK`), docs site (#1351 Cloudflare Pages prep).

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: subcommand present, `--once` flag documented as "Run one turn and return — no TTY REPL"
- `caro ai --once "list files in current directory"` → **FAIL**: command blocks indefinitely with zero output; no progress indicator, no error; must be killed manually
- Root site: `src/main.rs:1100–1108` — `CliApp::with_overrides(...).await` initiates model download with no stderr progress
- Distinct from FLAKE-001: the regular `caro -p ...` path at least prints the telemetry screen before hanging; `ai --once` produces nothing

### Findings

- [#1483](https://github.com/wildcard/caro/issues/1483) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (regression of #1044)` (P2, regression, release-gap)
- [#1484](https://github.com/wildcard/caro/issues/1484) — `ai: caro ai --once silently blocks indefinitely when no model is available` (P2)

### Followups

- FLAKE-001 now observed twice. One more occurrence within 7 days triggers reclassification as regression.
- 4 previous QA rotation PRs (#1178, #1373, #1443, #1477) remain open and unmerged — their memory updates never landed on main. Main's coverage matrix still shows bootstrap state (2026-05-07). Recommend merging or closing these PRs so memory state converges.
- #1044 fix direction step 2 ("Add CLAUDE.md to release checklist") was never implemented. #1483 refiles this structural fix.
- Surface #10 (`caro ai --once`) result: FAIL (P2 UX bug filed as #1484). Slot C next candidate: surface #11 (`caro ai --continue-session`) — note it also requires a working backend.
- Website + Codex Pet SDK surfaces (#1325) worth a dedicated Slot C pass from an environment with curl access to caro.sh.

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
