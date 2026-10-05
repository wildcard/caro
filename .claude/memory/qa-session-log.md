# QA Session Log

Reading order: most recent first.

---

## 2026-10-05 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 22s, no errors; binary = 1.5.0)
- `caro --version` → **PASS**: `caro 1.5.0 (75d32f7 2026-10-03)`
- `caro --help` → **PASS**: all subcommands listed (ai, shell-init, check, list, jobs, new, generate, run, export, experiment, adopt, history, why, do, render, skill, suggest, config, completion, doctor, init, integration)
- `caro doctor` → **PASS**: advisory only (no model, proxy detected, HuggingFace HTTP reachable)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` (note: telemetry consent re-prompted on fresh container — expected, was then disabled)

### Slot B — Recent diff

PRs merged since 2026-05-07 (selected for surface coverage; many dependency bumps skipped):

| PR | Title | Surface tested | Result |
|---|---|---|---|
| #1497 | fix(cli): accept every config show key in config set/get | `caro config get/set/show` | PASS |
| #1487 | fix(static-matcher): accept current-directory qualifier in Pattern 43 | `caro -p "list files in current directory" --dry-run` | PASS |
| #1509 | fix(explain): STE-lite explain mode + legible-output dev rule | `caro -p "find all python files" --explain --dry-run` | PASS |
| #1503 | fix(ai): don't block on stdin when ai --once has trailing prompt | `caro ai --once list files` (stdin check) | PASS (fix confirmed; model download hang is FLAKE-001) |

Surfaces flagged for future Slot C cycles: eval harness (#1489, #1459 — `caro test --backend static` surface #18).

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: correct usage shown; `--once` flag documented as "run one turn and return — no TTY REPL. The only mode supported today"
- `caro ai --once list files` with trailing prompt → **FLAKE** (model download required; process terminates after timeout with no output — FLAKE-001 same as last run)
- **PR #1503 regression check**: confirmed `needs_stdin_prompt()` guard at `src/main.rs:80` returns false when trailing words provided. stdin correctly set to `/dev/null` in process fd list. Fix is working — stdin is NOT blocking.
- Existing issue #1248 (open) covers the no-progress/no-error UX gap for non-TTY. Existing issue #1520 (open) covers CLAUDE.md version drift (banner shows 1.4.0, binary is 1.5.0).

### Findings

None new this pass — surfaces under test all in good shape. Two known open issues confirmed still present:
- [#1248](https://github.com/wildcard/caro/issues/1248) — ai: caro ai silently hangs in non-TTY (ongoing; confirmed still reproducible)
- [#1520](https://github.com/wildcard/caro/issues/1520) — docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (ongoing)

### Followups

- Model download FLAKE-001 second observation (2026-10-05). One previous in 7-day window? Last run 2026-05-07 is >7 days ago, so this is a fresh window. Logging as second observation in flake register.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — still "never" tested.
- Surface #18 (`caro test --backend static`) should be prioritized: three recent PRs (#1489, #1459, #1466) touch the eval harness.
- CLAUDE.md version drift is a recurring P2. Consider adding CLAUDE.md to the release checklist (as suggested in #1044). Already tracked in #1520.

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
