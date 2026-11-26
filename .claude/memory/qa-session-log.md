# QA Session Log

Reading order: most recent first.

---

## 2026-11-26 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.
**State source**: `state/qa-rotation/` on `automation/routine-status` (per `.claude/automation/CONTRACT.md` section 5), read with `bin/routine-status state get`. This file on `main` is the May bootstrap copy. This entry holds only this run's rows, for folding into the status branch.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS**
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS** (`guard` and `export` listed, `assess` absent)
- `caro doctor` → **PASS** (advisory: no model, no Ollama, guard hook not installed)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` from the static matcher. The run also downloaded the embedded model to the cache.

### Slot B — Recent diff
No merges since 2026-11-02 (last: #1740). Re-checked, all PASS:
- #1740 `config reset <key>`: `set safety strict` then `reset safety` gives Moderate; an unknown key exits 1 with the valid-key list.
- #1737 `test --help`: `[possible values: static, embedded]`.
- #1710 `caro ai` with no prompt: names `caro ai --once "list files"`, exit 1.
- #1687 / #1683 `guard test`: `curl … | sh` and `wget -qO- … | bash` give critical/deny; `ls -la` gives safe/none.

### Slot C — #28 Homebrew tap formula (never tested before)
I first started Slot C on #10 from main's May copy, then read the live state: #10 was tested 2026-11-21. So I switched to #28, the lowest live "never" row that the sandbox can test (#20 needs Ollama). `brew` is not installed, so the checks are static plus the release assets.
- **PASS**: `version "1.5.0"`; all four asset URLs return 200; all four `sha256` values match the downloaded assets; the formula `test do` (`caro --version` contains `caro 1.5.0`) passes on the linux-amd64 asset.
- **FAIL**: the caveats (`homebrew-tap/Formula/caro.rb:50-52`) and the tap README tell users to run `caro --completion <shell>`. It exits 2 with a clap error. Same wrong flag as #1787 (install.sh only). Filed #1790 (P1).
- **FAIL** (same issue): the tap README and `update-homebrew.yml` say `brew tap wildcard/caro https://github.com/wildcard/caro.git`. The formula is nested at `homebrew-tap/Formula/`, which brew does not search. README and /docs/installation use a third name, `wildcard/tap/caro`. Whether `wildcard/homebrew-tap` exists was not checked (GitHub access is limited to `wildcard/caro`).
- Side check of #10 `caro ai --once` with the model now in the cache: every prompt (also with `--new-session`) gives `Clarification needed: What exactly should be deleted?`, exit 1. Known open #1760 / #1269 (CPU stub, `cpu.rs:63`). Not re-filed.

### Findings
- [#1790](https://github.com/wildcard/caro/issues/1790) — install: Homebrew formula caveats use `caro --completion`; tap README points brew at a layout it cannot read (P1, release-gap)

### Followups
- Prompt drift (as on 11-23 to 11-25): the stored prompt says to store memory in a PR, but `CONTRACT.md` section 5 says `bin/routine-status state put`. This session may push only to `claude/peaceful-hopper-s11npz`, so the state branch was not written. The owner must fold this entry.
- Prompt drift: the stored prompt says website surfaces are #10, #11 and #12. In the matrix they are #25, #26 and #31.
- Next "never" rows: #33 `caro ai` decision gates, #34 executor timeout / guard hooks, #35 `caro export`.

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
