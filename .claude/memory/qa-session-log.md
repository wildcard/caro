# QA Session Log

Reading order: most recent first.

---

## 2026-11-18 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C. State read from `automation/routine-status:state/qa-rotation/` (main's copy is the stale May bootstrap, #1603). Slot C picked from that state, not from main's matrix.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS**
- `caro --version` → **PASS**: `caro 1.5.0 (aa1b0e6 2026-11-02)`
- `caro --help` → **PASS**
- `caro doctor` → **PASS** (advisory: no ready backend, expected)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` (static matcher)

### Slot B — Recent diff
No new merges since the 2026-11-17 run (main head still aa1b0e6). Spot-checked the newest five merges:
- #1740 `config reset <key>`: PASS (per-key reset, unknown key exits 1).
- #1737 `test --help`: PASS (lists static, embedded).
- #1710 `caro ai` no-prompt error: PASS (no `-p` hint).
- #1685 `caro guard audit`: PASS (help and subcommands present).
- #1687 fetch-and-execute patterns: verdict PASS (`curl | sh` → deny, `guard test` exit 2). The reason text is wrong → #1772.

### Slot C — `caro --safety strict/moderate/permissive` modes (#29)
Picked #29: oldest "never" rows #19 and #20 need a model or Ollama, which the sandbox lacks.
- Valid `--safety permissive` reaches the run (`--verbose` shows `Safety: Permissive`). PASS.
- `caro config set safety strict` then a run with no flag → `--verbose` shows `Safety: Moderate`. FAIL → #1771.
- `--safety strct` / `--safety bogus` → exit 0, silent Moderate. FAIL → #1771.
- End-to-end block/confirm differences could not be shown: static matcher has no Medium/High output for the prompts tried, and the embedded CPU stub answers "What exactly should be deleted?" (#1269).

### Findings
- [#1771](https://github.com/wildcard/caro/issues/1771) — safety: `config set safety strict` ignored; `--safety <typo>` silently Moderate (P1)
- [#1772](https://github.com/wildcard/caro/issues/1772) — safety: `curl | sh` labelled "targets a catastrophic, irrecoverable location" (P3)

### Followups
- Not filed (already tracked): `delete old log files older than 7 days` → `find . -type f -mtime -7` (wrong direction, no delete; #1628 family). `kill the process on port 3000` → literal `kill PID` (#1767 family).
- `--output json` still shows `executed: true` with `dry_run: true`; that is documented behaviour since #1217 (`src/cli/mod.rs:79`), so not a bug.
- Prompt drift: the stored prompt still says to read main's memory and open a PR; the state contract (`.claude/automation/CONTRACT.md` §5, #1603) says use `bin/routine-status state`. This run followed the prompt and opened a PR on the session branch `claude/peaceful-hopper-733wo2`.
- FLAKE-001 not observed (dry-run used the static matcher).

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
