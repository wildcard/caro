# QA Session Log

Reading order: most recent first.

---

## 2026-10-09 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: automated cron; second-ever run of caro-qa-agent.
**Rotation**: A + B + C.

### Slot A — Smoke

- `cargo build` → **PASS** (debug build, no errors)
- `caro --version` → **PASS**: `caro 1.5.0`
- `caro --help` → **PASS**: all subcommands listed
- `caro doctor` → **PASS**: advisory output only
- `caro -p 'list files in current directory' --dry-run` → **PASS** via static matcher (Pattern 43); no model download required for this query
- `caro config show` → **PASS**: shows active config
- `caro -p 'explain this' --dry-run` → **PASS**: static-matched, no LLM required

### Slot B — Recent diff

**Last run date**: 2026-05-07. Scanned 124 PRs merged since then (PR range approximately #1100–#1487+).

Spot-tested recent diff surfaces:

- `cargo test --lib -- safety` → **PASS**: 38 safety unit tests passed, 0 failed
- `caro config get` / `caro config set` → **PASS**: round-trips correctly
- `caro -p 'explain the last command' --dry-run` → **PASS**: static-matched
- `caro ai --once "list files"` → **FAIL** (pre-existing P1 #1269, see Findings)
- `caro -p 'find files in current directory' --dry-run` → **FAIL** (falls to embedded CPU stub; P1 #1269)

PR #1487 (Pattern 43 extension for current-directory qualifier) tested:

- `caro -p 'list files here' --dry-run` → **PASS**: matched by extended Pattern 43
- `caro -p 'show files in this directory' --dry-run` → **PASS**: matched by extended Pattern 43

### Slot C — `caro ai --once` (Surface #10)

Surface: **`caro ai --once` scripted conversational mode** (oldest 'never' tested surface, #10 in matrix).

- `caro ai --once "list files in home directory"` → **FAIL**
  - Embedded CPU stub hit (model downloaded successfully to `~/.cache/caro/models/`)
  - `cpu.rs:63` checks `prompt.contains("rm")` against the full system prompt
  - System prompt always contains "rm -rf" in its safety instructions
  - All non-static-matched queries return clarification stub: `"What exactly should be deleted?"`
  - Root cause: pre-existing P1 bug [#1269](https://github.com/wildcard/caro/issues/1269) (open since 2026-06-26)

### Findings

- **0 new issues filed**. Both bugs confirmed pre-existing:
  - [#1269](https://github.com/wildcard/caro/issues/1269) — CPU stub `rm`-in-system-prompt false positive (P1, open)
  - [#1098](https://github.com/wildcard/caro/issues/1098) — CLAUDE.md version drift (P2, open; #1044 closed as dup)

### Followups

- FLAKE-001 (model download) not reproduced today: model downloaded successfully to `~/.cache/caro/models/`. Update known-flakes accordingly.
- `caro ai --once` (Surface #10) remains broken. Next Slot C candidate: Surface #11 (`caro ai --continue-session`).
- #1269 has 5+ duplicate issues closed. Root fix in `cpu.rs:63` still pending.
- CLAUDE.md still shows `1.4.0 (GA)` while Cargo.toml is `1.5.0`. #1098 still open.

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
