# QA Session Log

Reading order: most recent first.

---

## 2026-11-25 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.
**State source**: `state/qa-rotation/` on `automation/routine-status` (per `.claude/automation/CONTRACT.md` section 5). This file on `main` is the May bootstrap copy. This entry holds only this run's rows, for folding into the status branch.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** ([duration omitted])
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS** (exit 0; `guard` and `export` listed, `assess` absent)
- `caro doctor` → **PASS** (exit 0; advisory: no model, no Ollama, guard hook not installed)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` from the static matcher

### Slot B — Recent diff
No merges since 2026-11-02 (last: #1740). Re-checked, all PASS:
- #1740 `config reset <key>`: set/reset `safety` and `log_level`; unknown key exits 1 with the valid-key list.
- #1737 `test --help`: `[possible values: static, embedded]`.
- #1710 `caro ai` with no prompt: names `caro ai --once "list files"`, exit 1.
- #1687 / #1683 `guard test`: `curl … | bash` deny rc=2; `bash <(curl …)` ask rc=3; `ls -la` none rc=0. `guard audit verify` with an empty log reports "no decisions logged yet". The floor reason text is still open #1772.

### Slot C — #27 Install script `install.sh` (never tested before)
I first ran Slot C on #10 from main's May copy, then read the live state: #10 was tested 2026-11-21. So I switched to #27, the lowest live "never" row that the sandbox can test (#20 needs Ollama).
- Ran `install.sh` in an isolated `HOME` and `CARO_INSTALL_DIR`, cargo hidden from `PATH`, stdin closed (non-interactive).
- The connectivity check fails in this sandbox (`https://github.com` answers 400 through the proxy). This is an environment limit, not filed. With only that function stubbed in a scratch copy: release lookup v1.5.0, download, **checksum verified**, binary runs (`caro 1.5.0 (d705666 2026-10-12)`).
- **FAIL**: it writes `eval "$(caro --completion bash)"` to `.bashrc`. The flag does not exist, so each new shell prints a clap error and no completion is set. Filed #1787 (P1).
- **FAIL**: it writes `export CARO_SAFETY_LEVEL="strict"` and prints "Safety level set to: strict". caro never reads the variable (`merge_with_env` has no caller) and runs Moderate. Filed #1788 (P1). It is related to #1771 (config value ignored).
- **FAIL** (website, found while tracing the install URLs): the /docs/installation commands are dead: `caro.sh/install.sh` 404 (fails silently with `| sh`), `cargo install cmdai` (old 0.1.0 crate) and the `caro-darwin-arm64` asset 404. Filed #1786 (P1). The homepage `setup.caro.sh` and README paths work.
- Side check of #10 `caro ai --once "list files in current directory"`: no output for 120 s while it starts the model download. `caro -p` answers the same prompt at once from the static matcher. These are known issues #1179, #1408 and #1272. Not re-filed.

### Findings
- [#1786](https://github.com/wildcard/caro/issues/1786) — docs: /docs/installation install commands are dead (P1, release-gap)
- [#1787](https://github.com/wildcard/caro/issues/1787) — install: install.sh writes `caro --completion`, which errors on every new shell (P1, release-gap)
- [#1788](https://github.com/wildcard/caro/issues/1788) — install: installer claims Strict via `CARO_SAFETY_LEVEL`, caro ignores it (P1, release-gap)

### Followups
- Prompt drift, as for #1783 and #1785: the stored prompt says to store memory in a PR, but `CONTRACT.md` section 5 says `bin/routine-status state put`. This session may push only to `claude/peaceful-hopper-y1yhxh`, so the state branch was not written. The owner must fold this entry.
- Prompt drift: the stored prompt names `scripts/install.sh`. The script is at the repo root (`install.sh`).
- `caro --verbose config get safety` is treated as a prompt, not the `config` subcommand. That is open #1163. Not re-filed.
- `delete all log files in this directory` and `remove all node_modules folders recursively` are blocked as "adversarial: Remove all files". Possible false positive in the intent gate. Not filed; a later Slot C on #29 can check it.
- Next: #28 Homebrew tap formula and #35 `caro export` are still "never".

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
