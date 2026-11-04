# QA Session Log

Reading order: most recent first.

---

## 2026-11-04 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** (2m 50s)
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS**: subcommands listed (doctor, config, test, ai, guard, check, ...)
- `caro doctor` → **PASS**: advisory only; no model cached (expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la`, dry-run banner shown (FLAKE-001 did not recur; telemetry consent prompt shown once on fresh HOME, as designed)
- Note: `gh auth status` reported an invalid GH_TOKEN; the GitHub MCP tools worked and were used instead.

### Slot B — Recent diff
Merged since last log entry: many; sampled the 5 most recent (all merged 2026-11-02):
- #1740 `config reset <key>` → PASS (see Slot C)
- #1737 `caro test --help` backends → PASS (`[possible values: static, embedded]`)
- #1710 `caro ai` no-prompt error → PASS (rc=1, message suggests `caro ai --once "list files"` not `-p`)
- #1706 `ls --version` timeout not read as BSD → not exercised (platform detection; doctor reports Ubuntu x86_64 correctly)
- #1716 harness docs cleanup → docs only, skipped

### Slot C — `caro config get/set/show/reset` (#14)
Exercised with a fresh HOME: `show`, `get`/`set`/`reset` on `safety` and `log_level`, `reset` of one key leaves other keys intact, unknown key (rc=1 with valid-key list), invalid value `safety bogus` (rc=1, lists valid values), `telemetry.enabled false`. All as documented. Minor nit (not filed): `config show` prints `telemetry: disabled` while the key is `telemetry.enabled`.

### Findings
None this pass.

### Followups
- Earlier QA rotation PRs #1743, #1744, #1746 (2026-11-01..03) show as closed without merge, so this log still ended at the 2026-05-07 bootstrap entry. Slot B used the most recent merged PRs, not a date window.
- `qa_agent_role.md` not present; no prompt drift check possible.

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
