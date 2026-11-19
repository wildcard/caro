# QA Session Log

Reading order: most recent first.

---

## 2026-11-19 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 31s)
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS**: all subcommands listed, including `guard`, `export`, `suggest`
- `caro doctor` → **PASS**: advisory only (no model, no Ollama — expected in sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` (static matcher, 0.6s)

### Slot B — Recent diff

Last log entry on `main` is 2026-05-07; 199 PRs merged since. Covered the 20 most recent (2026-10-19 → 2026-11-02):

- #1740 `caro config reset <key>` → PASS (reset `safety` Strict → Moderate; unknown key exits 1 with valid-key list)
- #1737 `caro test --help` → PASS (lists only `static, embedded`)
- #1710 `caro ai` no-prompt error → PASS (suggests `caro ai --once "..."`, no `-p`; exit 1)
- #1683 `caro guard test` → PASS (`curl … | sh` deny, exit 2; `ls -la` exit 0). `guard setup --verify` without target prints usage.
- #1685 `caro guard audit verify/tail` → PASS (empty log message; tail shows shadow-mode record)
- #1687 fetch-and-execute pattern → PASS (`curl … | sh` matched). Label text still says "catastrophic location" — already #1772.
- `git push --force origin main` rated safe by guard — already #1720.
- Docs/CI/security-only PRs (#1716, #1731, #1700, #1689, #1691, #1696, #1669, #1680) — no CLI surface to smoke.

### Slot C — `caro ai --once` scripted conversational mode (#10)

- `caro ai --once "<prompt>"` for 4 different prompts (list files, find python files, disk usage, delete .tmp) → **FAIL**: every prompt returns `Error: backend error: Clarification needed: What exactly should be deleted?`
- Same with `--new-session` and with prompt on stdin. Exit code 1.
- Duplicate of open [#1760](https://github.com/wildcard/caro/issues/1760) (P1, regression) — still reproduces on `aa1b0e6`. No new issue filed.

### Findings

- None new this pass. Re-confirmed #1760 on `aa1b0e6`.

### Followups

- FLAKE-001 (model download) not seen: dry-run now resolves via static matcher without download.
- Memory files on `main` are still at the 2026-05-07 bootstrap (#1603). Open QA PR #1755 (2026-11-08) is unmerged; this PR may conflict with it.
- Slot B backlog: ~179 PRs between 2026-05-07 and 2026-10-19 were not smoke-tested.
- No `.claude/memory/qa_agent_role.md` on `main` yet; no drift to report.
- Next Slot C candidate: #11 `caro ai --continue-session` (never tested; likely blocked by #1760).

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
