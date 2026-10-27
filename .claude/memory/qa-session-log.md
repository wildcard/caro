# QA Session Log

Reading order: most recent first.

---

## 2026-10-27 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC (fired 14:14).
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** (2m 16s)
- `caro --version` → **PASS**: `caro 1.5.0 (0766bf8 2026-10-21)`
- `caro --help` → **PASS**
- `caro doctor` → **PASS** (advisory: no model downloaded)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la` (static matcher; no model download needed)

### Slot B — Recent diff
Last log entry (2026-05-07) is stale; 186 PRs merged since. Sampled #1683 (`caro guard test`): `guard test 'rm -rf /'` → deny, exit 2; `guard test 'ls -la'` → none, exit 0. **PASS**.

### Slot C — `--output json/yaml` format correctness (#15, #16)
- JSON and YAML output for a dry-run parse cleanly (`json.load` / `yaml.safe_load`) and carry `generated_command`. **PASS**.
- `--output bogus` is silently accepted (exit 0, plain output). **FAIL** → #1726.
- Observed: unmatched `-p` queries (`x`, `y`, `hello`) print "What exactly should be deleted?" — CPU stub at `src/backends/embedded/cpu.rs:64`. Already tracked by open #1269 (closed dupes #1500, #1523, #1673). Not refiled.

### Findings
- [#1726](https://github.com/wildcard/caro/issues/1726) — `cli: invalid --output value silently accepted` (P2)

### Followups
- `gh` CLI token was invalid in this sandbox; used GitHub MCP tools instead.
- Earlier QA runs (e.g. 2026-06-26, 2026-10-15; see #1269, #1673) have no entries in this log: their memory PRs apparently never merged.
- #1269 still reproduces via `-p` on unrecognized input at HEAD; consider a comment there.
- Surfaces #25, #26, #31 (website) are good next Slot C candidates.

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
