# QA Session Log

Reading order: most recent first.

---

## 2026-07-16 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (many PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 29s, no errors; MSRV 1.85 confirmed in Cargo.toml)
- `caro --version` → **PASS**: `caro 1.5.0 (587e7a2 2026-07-14)`
- `caro --help` → **PASS**: all expected subcommands present including `ai`, `skill`, CaroML verbs; `--backend-info` global flag present
- `caro doctor` → **PASS**: huggingface.co reachable; proxy detected; no model cached (sandbox); advisory-only output
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: hung for 30s (timeout), then killed. Same pattern as FLAKE-001 (model download blocked at binary level in sandbox despite HTTP 200). Second total observation; not within 7-day window of first (2026-05-07). Updated FLAKE-001 log.

### Slot B — Recent diff

PRs merged since 2026-05-07 (30+ PRs, key surfaces exercised):

**PR #1315 — `fix(safety): P0 — close quote/escape evasion`** → safety surface
- `cargo test --lib -- safety` → **PASS**: 34/34 safety unit tests (up from 19 in bootstrap run; new tests for `smart_blend`, `allowlist_catastrophic`)
- `shell-words` tokenizer confirmed present at `src/safety/mod.rs:590` (`shell_words::split`)
- Verdict: **PASS** — P0 safety fix is in place and guarded

**PR #1298 — `fix(cli): single source of truth for backend roster`** → cli surface
- `caro --backend-info` → **PASS**: shows all 7 backends (`embedded`, `ollama`, `exo`, `vllm`, `mesh`, `ai-horde`, `hybrid`) with compilation status; non-default-feature backends correctly marked `not compiled`
- Verdict: **PASS**

**Config surface — explored incidentally during Slot B**:
- `caro config show` → **PASS**: shows `telemetry`, `log_level`, `cache_max_size`, `log_rotation`, etc.
- `caro config get telemetry.enabled` → **FAIL**: `Unknown config key 'telemetry.enabled'. Valid keys: backend, model-name, shell, safety`
- `caro config set telemetry.enabled false` → **FAIL**: same error
- `caro config reset` → **PASS**
- Root cause: `src/main.rs:2113,2158` — only 4 keys wired in `config get`/`set` despite `config show` displaying many more; 3 source files advertise the broken command
- Filed: [#1332](https://github.com/wildcard/caro/issues/1332) (P1)

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **`caro ai --once`** (oldest = never; lowest # in never-tested group).

- `caro ai --help` → **PASS**: comprehensive help; `--once`, `--new-session`, `--continue-session` flags documented; note states "The only mode supported today"
- `caro ai --once 'prompt' --dry-run` → produces `# caro-ai: session 1 (resumed) confidence=0.85 risk=Safe` + `echo 'Please clarify your request'` — expected static-backend fallback (no model downloaded in sandbox)
- `caro --dry-run ai --once` (stdin) → **PASS**: global `--dry-run` before subcommand correctly routes through main prompt path; `echo "show disk usage" | caro --dry-run ai --once` → `df -h` (correct!)
- `caro ai --once 'prompt' --dry-run` (global flag AFTER subcommand args) → `error: unexpected argument '--dry-run'` — clap-expected behavior; `--dry-run` is a global option and must precede the subcommand
- Session management:
  - `--new-session`: creates session 2 (no "resumed" tag) ✓
  - Second call without `--new-session`: resumes session 2 ("session 2 (resumed)") ✓
  - Sessions increment correctly across calls
- Verdict: **PASS** (functional surface; command quality limited to static-backend in sandbox)

### Findings

- [#1332](https://github.com/wildcard/caro/issues/1332) — `cli: caro config set telemetry.enabled false fails with "Unknown config key"` (P1)

### Followups

- FLAKE-001 (model download blocked) observed again. Second total observation; 70 days since first (2026-05-07). Not within the 7-day promotion window — keeping as active flake but noting recurrence.
- `caro ai --once` in static-backend mode always returns `echo 'Please clarify your request'` regardless of prompt — expected fallback, not a bug. Verify quality in a future run with model present.
- `caro ai --once 'prompt' --dry-run` (wrong arg order) returns clap error — not a bug (clap global-option semantics), but worth a user-facing hint. Consider P3 improvement.
- Issue #1044 (CLAUDE.md version drift) closed as completed on 2026-05-09 — removed from watch list.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or surface #12 (`caro assess`).

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
