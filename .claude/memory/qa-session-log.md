# QA Session Log

Reading order: most recent first.

---

## 2026-07-21 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (16 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 13s, no errors; MSRV now 1.85)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands present including CaroML and `ai` verbs
- `caro doctor` → **PASS**: advisory only (no model downloaded, proxy detected at http://127.0.0.1:34433)
- `caro -p 'list files in current directory' --dry-run` → **DEGRADED** (not a clean PASS or FLAKE): returned `echo 'Please clarify your request'` with exit 0. Root cause traced to two bugs — static matcher Pattern 43 regex rejects trailing context (→ #1362, P2) and CPU backend stub's `prompt.contains("rm")` fires on system-prompt words like "format" (→ #1361, P1). Filed as bugs, not tracked as a new flake.
- Telemetry consent auto-declined in non-interactive mode; `~/.config/caro/config.toml` persisted `first_run = false, enabled = false` after first invocation. Second invocation (`caro --version`) showed no prompt. **PASS**.

### Slot B — Recent diff (16 PRs since 2026-05-07)

Key PRs covered:

| PR | Surface | Smoke result |
|----|---------|-------------|
| #1315 `fix(safety): P0 — close quote/escape evasion` | Safety patterns | PASS — `cargo test --lib -- safety` 34/34 |
| #1352 `fix(i18n): load all locale JSON files; overhaul Hebrew translations` | i18n locale JSON | PASS — 115 locale files all valid JSON (python3 json.load) |
| #1244 `feat(agents): add ponytail reviewer` | Agent config only | not CLI-testable |
| #1155–#1159 brand/website design PRs | website (no local server) | deferred to surface #25/#26 in future Slot C |
| #1245 `feat: Fireworks hybrid-harness learnings` | eval/ml changes | deferred; no model available to test embedded inference quality |

Surfaces flagged for future Slot C: website brand surfaces (#25, #26) touched by PRs #1155–#1159 and #1327.

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **#10 — `caro ai --once` scripted conversational mode** (oldest = never; lowest `#` among 'never' surfaces).

- `caro ai --help` → **PASS**: `--once`, `--new-session`, `--continue-session` flags documented; stdout described as generated command only for shell widget injection.
- `caro ai --once 'list files in current directory'` → **FAIL**:
  ```
  # caro-ai: session 1 confidence=0.85 risk=Safe
  echo 'Please clarify your request'
  ```
  Exit code: 0. Shell widgets would inject this placeholder into the user's readline buffer. The `confidence=0.85` value is fabricated — not derived from generation quality.
- Root cause: CPU backend stub in `src/backends/embedded/cpu.rs` uses `prompt.contains("rm")` on the full prompt string; "rm" appears as a substring in common words in the system prompt ("format", "perform", "terminal"), causing the safety-fallback branch to fire for ALL queries regardless of user intent.

### Findings

- [#1361](https://github.com/wildcard/caro/issues/1361) — `ai: caro ai --once returns misleading placeholder command with confidence=0.85 when backend degraded` (P1)
- [#1362](https://github.com/wildcard/caro/issues/1362) — `cli: static matcher Pattern 43 regex rejects "list files in <location>" despite matching required keywords` (P2)

### Followups

- FLAKE-001 behavior changed: previous run (2026-05-07) showed explicit download error; this run showed silent CPU-stub fallback. The CPU stub bug (#1361) is the underlying cause — it was presumably dormant until this run's code path exercised the stub.
- Website brand surfaces (#25, #26) touched by multiple merged PRs; schedule for next available Slot C.
- `caro ai --continue-session` (surface #11) not yet tested — pending model availability.
- 16 PRs merged in the 75-day gap since last run; Slot B only spot-checked the two highest-risk PRs (#1315, #1352). Additional spot checks deferred.

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
