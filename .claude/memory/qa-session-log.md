# QA Session Log

Reading order: most recent first.

---

## 2026-11-24 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.
**State source**: `state/qa-rotation/` on `automation/routine-status` (per `.claude/automation/CONTRACT.md` section 5). This file on `main` is the May bootstrap copy. This entry holds only this run's rows, for folding into the status branch.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** (1m 49s)
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS** (exit 0; `guard` and `export` listed)
- `caro doctor` → **PASS** (exit 0; advisory: no ready backend before the model download)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` from the static matcher in 0.3 s
- The model (qwen2.5-coder-1.5b q4_k_m) downloaded later in the run. FLAKE-001 did not recur.

### Slot B — Recent diff
No merges since 2026-11-02 (last: #1740). The 2026-11-23 run already covered #1740, #1737, #1710, #1687 and #1685. I re-checked them:
- #1740 `config reset <key>` → PASS (set/reset `safety` and `log_level`; unknown key exits 1 with the valid-key list)
- #1737 `test --help` → PASS (`[possible values: static, embedded]`)
- #1710 `caro ai` with no prompt → PASS (message names `caro ai --once "list files"`, exit 1)
- #1687 normalization views → PASS (`r""m -rf /` deny; `curl | bash` deny rc=2; `bash <(curl …)` ask rc=3; `ls` none rc=0)
- #1685/#1686 guard log → PASS (one Claude hook payload logged; `guard audit verify` = CHAIN INTACT; `guard report` counts it)
- The JSON reason for `curl … | bash` still adds "targets a catastrophic, irrecoverable location". That is open #1772. Not re-filed.

### Slot C — #23 `caro do` Carofile job runner (and #19 embedded quality)
- #19 (oldest 'never'): `caro -b embedded -p 'convert every flac file in music to mp3 with ffmpeg' --dry-run` → `What exactly should be deleted?`. Any prompt the static matcher misses hits the `cpu.rs` stub. FAIL, open #1269. `caro ai --once` fails the same way for every prompt (#1760, a duplicate of #1269).
- #23 `caro do`: `jobs` lists JOBs. A bad `RUN` alias gives a line-numbered error. `do <job|alias|native|bare> --dry-run` prints the right plan.
- **FAIL**: external aliases (`USE "<cmd>" AS x`) go to `CommandExecutor::execute` with no safety validation. The docs and the `jobs.rs` module comment say safety scans them. `--dry-run` of `USE "rm -rf /"` shows no risk. Filed #1784 (P1). I did not execute any alias. A live harmless proof was blocked by the sandbox policy, which is correct.

### Findings
- [#1784](https://github.com/wildcard/caro/issues/1784) — safety: `caro do` runs Carofile `USE "<command>"` aliases with no safety validation (P1, release-gap)

### Followups
- Prompt drift: the stored prompt says to store memory in a PR. The in-repo `.claude/automation/CONTRACT.md` section 5 says to use `bin/routine-status state put`. This session may push only to `claude/peaceful-hopper-8u06ui`, so the state branch was not written. The owner must fold this entry, as for #1783.
- `caro run` (`src/caroml/runner.rs` `execute_plan`) also ignores the lock's `risk_level`. It is noted in #1784. A later Slot C on #21 can confirm it with a hand-edited lock in `--dry-run`.
- Static matcher quality, seen in passing: `count lines in every markdown file under docs` → `wc -l README.md`; `compress the logs folder` → `tar -czf archive.tar.gz directory/`. Similar to open #1767. Not filed.

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
