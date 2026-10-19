# QA Session Log

Reading order: most recent first.

---

## 2026-10-19 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C. Last log entry was 2026-05-07, so Slot B sampled the newest merged PRs only.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 29s)
- `caro --version` → **PASS**: `caro 1.5.0 (07679f0 2026-10-19)`
- `caro --help` → **PASS**: lists `guard`, `export`, `ai`, `shell-init`, CaroML verbs; no `assess`
- `caro doctor` → **PASS**: advisory only; now also reports the guard hook state
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` (static matcher; no model download attempted, FLAKE-001 not seen)

### Slot B — Recent diff

- #1683 `caro guard test` / `guard setup --verify` → **PASS**: `rm -rf /` deny/critical, `chmod -R 777 /etc` ask/high, `ls -la` none/safe; `guard setup claude --dry-run` previews the hook
- #1614 segment/wrapper-aware safety → **PASS**: `sh -c 'rm -rf /'`, `echo hi && rm -rf ~`, `sudo bash -c "mkfs.ext4 /dev/sda"` all deny/critical; `echo form information` stays safe
- #1600 recursive world-writable chmod → **PASS** (see above)

### Slot C — `caro ai --once` (matrix #10, #11)

- `ai --once` with arg, stdin, `--new-session` and resumed session: 5 of 5 return `Error: backend error: Clarification needed: What exactly should be deleted?` (exit 1).
- Empty stdin → `Error: no prompt provided` (correct).
- Control `-p ... --dry-run` returns correct commands.
- Root cause unchanged: `src/backends/embedded/cpu.rs:63-64` keyword stub matches `rm` in the system prompt.
- `caro assess` is not a subcommand in 1.5.0; it is parsed as a prompt and prints the same bogus clarification. With stdin open and not a TTY it blocks on stdin.
- Already tracked: open #1269 (P1); #1500, #1523 and #1673 closed as duplicates. No new issue filed; comment added to #1269.

### Findings

- None new. Re-confirmed [#1269](https://github.com/wildcard/caro/issues/1269) (P1, fourth report) and commented.

### Followups

- `caro ai --once` is broken on 1.5.0 Linux. Three duplicate issues were closed without a fix. Needs an owner decision.
- Static matcher drops constraints: `find python files larger than 1MB` → `find . -name "*.py" -type f` (no size filter). Not filed; check for an existing issue before next pass.
- Add the `caro assess` fallthrough (unknown word becomes a prompt) to a future Slot C pass.

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
