# QA Session Log

Reading order: most recent first.

---

## 2026-09-07 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 07s, `caro 1.5.0 (be07b22 2026-07-18)`)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: 24 subcommands listed including all CaroML verbs and `ai`
- `caro doctor` → **PASS**: advisory only (no model, expected in fresh sandbox; proxy detected)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: telemetry consent required interactive TTY; underlying model download blocked (FLAKE-001 2nd occurrence in sandbox)

### Slot B — Recent diff

Last session: 2026-05-07 (bootstrap). Note: QA PR #1373 (2026-07-25) remains open/unmerged — session log on main still shows 2026-05-07 as last entry. Covered feature PRs merged since 2026-05-07:

- **#1315** `fix(safety): P0 — close quote/escape evasion`: `cargo test --lib -- safety::tests` → **PASS** (34/34, up from 19 in bootstrap; 15 new safety tests added)
- **#1304** `chore(release): v1.5.0`: `caro --version` → PASS shows 1.5.0
- **#1298** `fix(cli): single source of truth for backend roster`: `caro --backend-info` → **PASS** (7 backends listed; embedded=available, others=not compiled with correct guidance)
- **#1352** `fix(i18n): load all locale JSON files; overhaul Hebrew translations`: `curl https://caro.sh/he/` → **PASS** (1.69MB, `lang="he" dir="rtl"`, correct Hebrew title)
- **#1246** `fix(ci): repair pre-existing main test failures`: `cargo test --lib` → **PASS** (all tests pass)

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: shows `--once` flag, `[PROMPT]...` positional arg, `--new-session`, `--continue-session`
- `caro ai --once list files in current directory` → **HANG** (SIGTERM at 15s; FLAKE-001 model download blocked in sandbox)
- `caro ai --once --backend ollama list files` → **FAIL**: `error: unexpected argument '--backend' found` — global flags must precede subcommand name (tracked #1369, #1422)
- `caro --backend embedded ai --once list files` → parsed correctly but hangs on model download (FLAKE-001)
- AI config: `ai.enabled` defaults to `true` (confirmed in `src/models/mod.rs:818`)
- Confirmed open P1: #1375 — CPU backend always returns `echo 'Please clarify your request'` on Linux x86_64 due to system-prompt keyword collision with "delete"/"rm" safety examples
- Confirmed open: #1421 (no-backend hang), #1422 (--prompt flag rejection)

### Findings

- [#1442](https://github.com/wildcard/caro/issues/1442) — `docs: CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0` (P2) — recurrence of closed #1044

### Followups

- FLAKE-001 2nd occurrence (2026-09-07). Still within 2-observation window — no promotion. Next observation would be 3rd; if within 7 days of one prior, promote to regression.
- QA PR #1373 (2026-07-25) is still open/unmerged on GitHub. This creates a split-brain between main's session log and what that PR recorded. Recommend merging or closing #1373 before the next scheduled run.
- `caro.sh` root redirects to `www.caro.sh` (HTTP 307); sandbox proxy blocks `www.caro.sh` response body (0 bytes). Not a product bug — sandbox network limitation. `caro.sh/he/` works fine (1.69MB).
- `caro.sh/en/` returns 404 — expected; `prefixDefaultLocale: false` means English lives at `/`, not `/en/`.
- P1 issue #1375 (echo placeholder on Linux) remains unfixed in v1.5.0. Next Slot C should re-exercise `caro ai --once` after #1375 is fixed.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — tie with #10 at 'never', but #10 was just exercised.

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
