# QA Session Log

Reading order: most recent first.

---

## 2026-09-28 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs merged since last log entry on 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 24s, no errors; caro 1.5.0 built)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands present including full CaroML verb set and `ai`, `suggest`, `skill`, `completion`
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected; proxy detected at 127.0.0.1:43861)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: model not downloaded (FLAKE-001 second occurrence, 2026-09-28); fallback returns `echo 'Please clarify your request'`. Note: `caro -p 'list files' --dry-run` → **PASS** (`ls -la` via static matcher; shorter query hits Pattern 43 regex correctly)

### Slot B — Recent diff

110 PRs merged since 2026-05-07. Representative surface tested per the most safety-critical PR:

- **PR #1315** (`fix(safety): P0 — close quote/escape evasion of the command scanner`): ran `cargo test --test safety_validator_contract test_quote_escape_evasion_is_caught_base` → **PASS** (1/1 test ok). The `shell-words`-based `destructive_unescaped()` normalization blocks `rm -rf \/`, `rm -rf "/tmp"/*/x`, and `rm -\rf \/etc` while keeping `echo 'rm -rf /'` safe.
- **PR #1304** (`chore(release): v1.5.0`): version confirmed in binary (`1.5.0 (be07b22 2026-07-18)`) → **PASS**
- i18n batch PRs (#816–#829): surfaces flagged for future Slot C coverage under surface #31 (i18n locale smoke)

### Slot C — `caro ai --once` (surface #10)

- `caro ai --once 'show disk usage'` → returns two lines: `# caro-ai: session 1 (resumed) confidence=0.85 risk=Safe` then `echo 'Please clarify your request'` — **FLAKE** (FLAKE-001 applies; no model available; same fallback as Slot A)
- `caro ai --help` → **PASS**: subcommand documented correctly; `--once` flag described as "Run one turn and return — no TTY REPL. The only mode supported today"
- `caro ai -p 'query'` → **FAIL** (expected): "unexpected argument '-p' found" — confirmed UX inconsistency; pre-existing as [#1422](https://github.com/wildcard/caro/issues/1422) (open)
- `caro ai --once` without prompt → **PASS**: correctly errors "no prompt provided (pass text, pipe stdin, or use -p)" — though the hint says "use -p" but `-p` doesn't work (consistent with #1213)
- Static matcher gap also confirmed for `caro ai` path: "list files in current directory" → fallback; pre-existing as [#1399](https://github.com/wildcard/caro/issues/1399) (open)
- CLAUDE.md version drift: shows `1.4.0 (GA)`, binary reports `1.5.0`; pre-existing as [#1474](https://github.com/wildcard/caro/issues/1474) (open, multiple dupes filed by prior QA runs)

### Findings

- **None filed this pass** — all identified issues are pre-existing open GitHub issues (#1399, #1422, #1474, others). No new unique defects discovered.
- Confirmed surfaces healthy: safety scanner P0 fix (PR #1315) verified; binary build and version alignment correct; doctor and help accurate.

### Followups

- FLAKE-001 (model download in sandbox) observed again: 2nd occurrence on 2026-09-28. Dates are far apart (May → Sep), so not 3× in 7 days; does NOT trigger regression reclassification. Occurrence log updated.
- Multiple QA rotation PRs from prior runs (#1178, #1373, #1443, #1477) remain open/unmerged, keeping main's memory files at the 2026-05-07 state. This causes each new QA pass to re-encounter the same open issues from its perspective. Recommend merging the oldest open QA rotation PR (#1178) to unblock memory state progression.
- i18n locale surfaces (#31) should be the next Slot C candidate after the `caro ai` surface is marked tested.
- Surface #11 (`caro ai --continue-session`) not yet tested — schedule for next pass.

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
