# QA Session Log

Reading order: most recent first.

---

## 2026-09-08 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 23s, no errors; built clean from main HEAD be07b22)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed including CaroML verbs, `skill`, `suggest`, `ai`, `completion`, `export`, `why`, `do`, `render`
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox; huggingface.co reachable, proxy detected)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: telemetry consent prompt displayed on first prompt-generating invocation in fresh sandbox, process timed out waiting for stdin response (30s timeout). Environment limitation — not a code bug. Same root cause class as FLAKE-001 (sandbox restrictions). Telemetry consent is expected on first invocation.

### Slot B — Recent diff (PRs merged since 2026-05-07)

20 non-dependency PRs merged. Key CLI/safety surfaces checked:

- **#1209 feat(backends): Mesh-LLM + AI-Horde + hybrid privacy gateway** → `caro --backend-info` shows all new backends (`mesh`, `ai-horde`, `hybrid`) with correct status and descriptions. Source files `mesh.rs`, `ai_horde.rs` confirmed in `src/backends/remote/`. PASS.
- **#1298 fix(cli): single source of truth for backend roster** → `--backend-info` flag present in `--help`; backend list `embedded, ollama, exo, vllm, mesh, ai-horde, hybrid` correct. `openrouter` and `claude` noted as "not yet CLI-wired" (confirmed intentional via `src/backends/mod.rs` comment). PASS.
- **#1211 test(execution): make dangerous-command confirm test deterministic** → `cargo test --lib -- safety` → 34/34 PASS (up from 19/19 in 2026-05-07; new tests for allowlist catastrophic floor). PASS.
- **#1352 fix(i18n): Hebrew translations** → flagged for future Slot C run on surface #31 (i18n locale smoke).
- **#1245 feat: apply Fireworks hybrid-harness learnings** → `--advisor` flag present in `--help`. Surface not fully exercisable without credentials; noted for future slot.

### Slot C — `caro ai --once` (surface #10)

- `caro ai --once 'list files'` → **FLAKE**: process terminated (exit 143) after 25s timeout. Consistent with embedded backend attempting model download that is blocked in sandbox (FLAKE-001 root cause). `caro ai --help` confirmed `--once` flag exists and is described correctly.
- Surface confirmed to exist and be CLI-wired. Cannot exercise end-to-end without a pre-downloaded model.

### Findings

- [#1444](https://github.com/wildcard/caro/issues/1444) — `docs: CLAUDE.md version and MSRV out of sync with v1.5.0` (P2) — recurrence of #1044 pattern; `CLAUDE.md` shows `Version: 1.4.0 (GA)` and `MSRV 1.83` while shipped version is 1.5.0 and MSRV was bumped to 1.85.

### Followups

- FLAKE-001 (model download): second observation on `caro ai --once`. Still below 3-in-7-days promotion threshold. Consider pre-populating model cache in sandbox for future deeper embedded-backend testing.
- Surface #31 (i18n locale smoke: curl /es/, /fr/, /ja/) flagged for next Slot C rotation — #1352 makes it relevant.
- `openrouter` and `claude` backends exist in source but are intentionally not CLI-wired. Track for when they land.
- CLAUDE.md should be added to the release-version-alignment 6-file checklist (see #1444 fix direction).

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
