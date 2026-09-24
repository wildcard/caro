# QA Session Log

Reading order: most recent first.

---

## 2026-09-24 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 09s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed including CaroML verbs and new `skill` subcommand
- `caro doctor` → **PASS**: proxy detected, HuggingFace reachable, no model on first check (model downloaded during dry-run)
- `caro -p 'list files in current directory' --dry-run` → **PARTIAL** (exit 0): WARN `Timeout approaching, skipping refinement`; generated `echo 'Please clarify your request'` (root cause confirmed in Slot C, see #1473)
- FLAKE-001 status: model downloaded successfully this run (1.1GB GGUF present after run) — first successful download; FLAKE-001 occurrence streak NOT extended

### Slot B — Recent diff (110 PRs since 2026-05-07)

Key surfaces exercised:
- **Safety unit tests** (covers P0 fix #1315 quote/escape evasion) → **PASS**: 34/34 tests (`allowlist_catastrophic`, `smart_blend`, CVE patterns, etc.)
- **Backend roster** (#1298 single-source-of-truth fix) → **PASS**: `--backend` flag shows correct 7 backends (embedded, ollama, exo, vllm, mesh, ai-horde, hybrid)
- **Config surface** → **PASS**: `caro config show` returns correct fields
- **Shell-init** → **PASS**: bash, zsh wrappers emit correct content
- **Completions** → **PASS**: `caro completion bash` emits valid completion script
- **i18n #1352** (Hebrew locale overhaul) → **PASS**: `website/src/i18n/locales/he/` exists with all locale JSON files

Flagged for future Slot C: i18n locale coverage (surface #31) — touched by #1352, add to next rotation.

### Slot C — `caro ai --once` (surface #10, first test)

Exercised `caro ai --once` scripted conversational mode for the first time.

- All prompts (`list files`, `show disk usage`, `print working directory`, `what time is it`) → **FAIL**: every prompt returns `echo 'Please clarify your request'` with `confidence=0.85 risk=Safe`
- Root cause: `src/backends/embedded/cpu.rs:63` — placeholder inference stub checks `prompt.contains("rm")` on the full system prompt (which always contains `rm -rf` in safety rules). All invocations match the "clarify" branch.
- Confidence hardcoded to 0.85 regardless of output quality → misleading to users
- Session resume anomaly: sessions were resumed across separate `--once` invocations in the same test run (expected, but noteworthy for scripting use cases)

### Findings

- [#1473](https://github.com/wildcard/caro/issues/1473) — `embedded: CPU backend placeholder always returns "echo 'Please clarify your request'" due to system-prompt contamination` (P1)
- [#1474](https://github.com/wildcard/caro/issues/1474) — `docs: CLAUDE.md version shows 1.4.0 instead of 1.5.0 — regression of #1044` (P2)

### Followups

- FLAKE-001: model downloaded successfully this run. Reset observation counter; previous observation on 2026-05-07 is now 7+ days old and outside the 7-day window. FLAKE-001 remains open but unconfirmed.
- Note: 3 prior QA rotation PRs (#1178, #1373, #1443) were found in the repo but their session-log entries were not in this file. Session log may be incomplete between 2026-05-07 and today. Not critical to reconstruct — log is append-only going forward.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or surface #12 (`caro assess` system assessment).
- i18n locale smoke (surface #31) should be prioritized in next Slot B given #1352 impact.

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
