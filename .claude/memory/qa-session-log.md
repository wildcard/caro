# QA Session Log

Reading order: most recent first.

---

## 2026-10-10 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (127 commits since last run) + C.
**⚠ gh auth failed**: GH_TOKEN invalid — issue filing and PR creation blocked. Session log and coverage matrix updated locally; branch pushed. Issue filing deferred until token is rotated.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 35s, no errors; version 1.5.0 builds clean)
- `caro --version` → **PASS**: `caro 1.5.0 (1f2fcde 2026-10-10)`
- `caro --help` → **PASS**: all subcommands present including CaroML verbs and `ai`, `skill`, `export`, `render`
- `caro doctor` → **PASS**: advisory only (no model, Ollama not installed; Proxy detected at 127.0.0.1:45407)
- `caro -p 'list files in current directory' --dry-run` → **PASS** ✨: `ls -la` returned via static-matcher in 34ms (improved over 2026-05-07 FLAKE — the static matcher now reliably intercepts basic patterns without model download)
- `caro -p '...' --dry-run --verbose` → confirmed `Backend: static-matcher, Confidence: 1.00`

### Slot B — Recent diff

127 commits since 2026-05-07 (version 1.3.0 → 1.5.0). Key surfaces touched:

- **fix(cli): add `dry_run` to JSON output (#1532)** — `caro -p '...' --dry-run --output json` shows `"dry_run": true` field → **PASS**
- **fix(cli): accept every `config show` key in `config set/get` (#1497)** — `caro config get safety` and `caro config set/get log_level` → **PASS**
- **fix(safety): correct label + pattern descriptions (#1535)** — `cargo test --lib -- safety` → **PASS** (38 tests, up from 19 in May)
- **fix(static-matcher): accept current-directory qualifier in Pattern 43 (#1487)** — covered by `--dry-run` smoke above (pattern 43 = "list files")
- **fix(ai): don't block on stdin when ai --once has trailing prompt (#1503)** — see Slot C below (cannot fully verify without model)
- Website i18n, Astro docs-site, and ponytail reviewer changes not exercised (out of CLI scope for this pass).

Surfaces flagged for next Slot C cycle:
- `caro --output yaml` format (fix(eval): consensus labels #1489 touched output paths)
- `caro ai --once` needs a sandbox with pre-downloaded model (see Slot C)

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **`caro ai --once` scripted conversational mode** (oldest 'Last tested' = 'never', lowest # = 10).

- `caro ai --once 'show me the current date'` → **FLAKE**: command hung with zero output until 20s external timeout. No error message surfaced. Stdin `/dev/null` made no difference.
- **Root cause**: `run_ai_once` calls `cli_app.backend_arc()` (raw embedded backend) not the `AgentLoop` (which has the static matcher). So `caro ai` always requires a real model; the static matcher bypass that makes `caro --dry-run` work does NOT apply to `caro ai`. In a sandbox with no pre-downloaded model, the command hangs indefinitely during `generate_command`.
- **Severity of hang**: The command produces NO output before hanging — no "downloading model…" progress, no timeout message, no error. This is silent freeze behavior, which is worse UX than the `--dry-run` FLAKE-001 path (which did eventually surface an error after retries).
- FLAKE-001 second observation logged.

### Findings

- **CLAUDE.md version drift** (P2): CLAUDE.md line 9 shows `**Version**: 1.4.0 (GA)` but `caro --version` returns `1.5.0` and README.md correctly shows `1.5.0`. Version 1.5.0 was released 2026-07-12; CLAUDE.md was not updated. This is a recurrence of #1044 (was 1.1.0 vs 1.3.0 in May, now 1.4.0 vs 1.5.0). **Cannot file issue — gh auth failed.** Deferred to next run with valid token.
- **`caro ai --once` silent hang** (P2): No output and no timeout on `ai --once` when model not available. `caro doctor` does not warn that `ai` mode requires a model. **Cannot file issue — gh auth failed.**

### Followups

- **GH_TOKEN is invalid** — the token must be rotated before the next QA run can file issues or open a PR. The branch `claude/qa-rotation-2026-10-10` was pushed for traceability.
- FLAKE-001 second overall observation (2026-10-10). Still within tolerance (not 3x in 7 days); logged in qa-known-flakes.md.
- Next Slot C: surface #11 `caro ai --continue-session` (still 'never' tested, lowest uncovered # after #10).
- Surface #2 (`--dry-run`) updated to PARTIAL in coverage matrix: static-matcher path PASS for known patterns; model-backed path remains FLAKE (novel prompts still trigger download). Full PASS requires sandbox with pre-downloaded model.
- Consider filing a new issue for CLAUDE.md 1.4.0→1.5.0 drift once gh auth is restored (same class as #1044).

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
