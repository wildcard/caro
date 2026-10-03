# QA Session Log

Reading order: most recent first.

---

## 2026-07-25 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (20 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (build succeeded; MSRV now 1.85 per v1.5.0 changelog)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed; `ai` and CaroML verbs present
- `caro doctor` → **PASS**: correctly reports embedded backend needs model download, Ollama not installed; proxy detected at `http://127.0.0.1:45761`
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: first attempt showed telemetry consent prompt, timed out (exit 124). Second attempt silently hung (exit 124). FLAKE-001 second occurrence — model download blocked by sandbox proxy on HuggingFace binary blob (HTTP 200 reachable, binary blocked at lower layer)
- `caro config show` → **PASS**: shows all config fields; telemetry disabled after first-run consent

### Slot B — Recent diff

PRs merged since 2026-05-07: 20 PRs. Two high-priority surfaces exercised:

**#1315 `fix(safety): P0 — close quote/escape evasion of the command scanner`**
- Added `shell-words` crate; normalises destructive-command statements before pattern scan
- `cargo test --test safety_validator_contract` → **PASS**: 22/22 passed (1 pre-existing `#[ignore]`); new test `test_quote_escape_evasion_is_caught_base` passes — `rm -rf \/`, `rm -rf "/tmp"/*/x`, `rm -\rf \/etc` all blocked; `echo 'rm -rf /'` stays allowed

**#1352 `fix(i18n): load all locale JSON files; overhaul Hebrew translations`**
- `import.meta.glob('./locales/*/*.json')` loader present in `website/src/i18n/index.ts` → **PASS**
- Hebrew landing.json spot-check: old mistranslations gone (`מונעת AI`, `מסקנה מקומית`, `מהיר בזרם הברק`, `כישור מוקדש`, `כל שימושי המקרה`), replacements confirmed present (`מהירה כברק`, `היסק מקומי`, etc.) → **PASS**

Other PRs noted but not deeply exercised: #1304 (v1.5.0 release), #1244 (ponytail reviewer), #1159/#1155/#1156/#1158 (brand/design), #1245 (ML eval harness), #1306 (CI release workflow).

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: flags documented correctly (`--once`, `--new-session`, `--continue-session`, trailing prompt args)
- `caro ai --once` (no prompt) → **PASS**: correct error "no prompt provided (pass text, pipe stdin, or use -p)" (exit 1)
- `caro ai --once "list files in current directory"` → **FLAKE**: silent timeout (exit 124, 35s). Backend auto-selects embedded; model download blocked (FLAKE-001). No user-visible progress indicator during hang — noted as UX gap but not a bug (consistent with dry-run behaviour, same root cause).

### Findings

- [#1372](https://github.com/wildcard/caro/issues/1372) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurrence of #1044)` (P2)

### Followups

- FLAKE-001 second occurrence (2026-07-25). Still 2/7-day window — below 3-in-7 reclassification threshold. Next occurrence promotes to regression.
- Issue #1044 (prev CLAUDE.md drift) closed as completed — but the checklist fix was NOT applied to `release-version-alignment.md`, causing this exact recurrence. Fix direction in #1372 is the same + "add to checklist".
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — tests session persistence across invocations.
- Safety `test_safety_level_configuration` remains `#[ignore]` — pre-existing; expectations don't match current implementation.

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
