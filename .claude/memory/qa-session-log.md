# QA Session Log

Reading order: most recent first.

---

## 2026-07-24 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled caro-qa-agent cron (14:00 UTC).
**Rotation**: A + B + C (first run since bootstrap 2026-05-07).
**Binary**: `caro 1.5.0 (be07b22 2026-07-18)`

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (pre-built binary present)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FAIL** (known P1 #1277): CPU placeholder misfires; returns `echo 'Please clarify your request'` instead of `ls -la`; system prompt contains "rm", triggering danger branch
- `cargo test --lib -- safety` → **PASS**: 34/34 safety unit tests (up from 19 at bootstrap)
- Custom patterns `~/.config/caro/patterns.toml` → **PASS**: discovered and loaded at startup
- Telemetry consent → **PASS**: shown once, persisted; no prompt on subsequent runs

### Slot B — Recent diff (since 2026-05-07, ~110 PRs)

Sampled 4 representative surfaces:

- **Safety test expansion**: 34 tests vs 19 at bootstrap; smart_blend, allowlist_catastrophic, telemetry redaction suites green → **PASS**
- **Remote backend UX** (#1092 area): `--backend ollama` without feature flag emits clear build instruction → **PASS**
- **Custom runtime patterns** (#1110, v1.5.0): `patterns.toml` loaded at startup → **PASS**
- **CLAUDE.md version drift**: shows 1.4.0, binary is 1.5.0 → **FAIL** (known P2; 5 issues open: #1319, #1335, #1359, #1366, #1368; no new issue filed)

### Slot C — Oldest-untested surface

Surface: **#10 — `caro ai --once`** (Last tested: never).

- `caro ai --once -p 'list files in current directory'` → **FAIL** (known P1 #1277): no static-matcher pre-pass in `src/ai/runner.rs`; CPU placeholder returns `echo 'Please clarify your request'`
- No new issue filed; failure fully attributed to pre-existing #1277
- Next Slot C: **#13 `caro suggest`** (surfaces #11, #12 also blocked by #1277)

### Findings

- **0 new issues filed.** All failures attributed to pre-existing tracked issues.

### Followups

- FLAKE-001 reclassified to Resolved: model download succeeded (78 days since single bootstrap observation with no recurrence)
- #1277 (P1) blocks Slot C surfaces #10 and #11; recommend fixing before next Slot C pass
- CLAUDE.md drift still P2 with 5+ tracking issues
- Next Slot C: **#13 `caro suggest`**

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
