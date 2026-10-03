# QA Session Log

Reading order: most recent first.

---

## 2026-09-06 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 18s, no errors; MSRV now 1.85 per v1.5.0 changelog)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands present including `ai`, `suggest`, `skill`, `export`, all CaroML verbs
- `caro doctor` → **PASS**: advisory only (no model downloaded, proxy detected, expected in sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: process hangs indefinitely; no error output, no retries — regression vs v1.3.0 which produced "Failed to download model after 3 attempts". Root cause confirmed: `HfHubClient` has no HTTP timeout (see #1440). FLAKE-001 second observation; behavior now CHANGED (silent hang instead of retry-with-error).
- `caro config show` → **PASS**: all config fields shown correctly
- `caro --backend-info` → **PASS**: correctly lists embedded (available), all remote backends (not compiled), with install note

### Slot B — Recent diff

110 PRs merged since 2026-05-07. Key surfaces exercised:

- **Safety unit tests** (PR #1315 — P0 quote/escape evasion fix): `cargo test --lib -- safety` → **PASS**: 34/34 (was 19 in v1.3.0; 15 new tests added by P0 fix including `evasions_are_closed`, `allowlist_cannot_reenable_catastrophe`)
- **Full lib suite**: `cargo test --lib` → **PASS**: 597 passed, 0 failed, 1 ignored (was 513 in v1.3.0; 84 new tests)
- **Backend roster** (PR #1298 — single source of truth): `caro --backend-info` → **PASS**: `embedded, ollama, exo, vllm, mesh, ai-horde, hybrid` all listed correctly
- **Slot B flagged for future Slot C**: PR #1209 (Mesh-LLM + AI-Horde + hybrid) — remote backends require `--features remote-backends`; surfaces #20+ in matrix should be tested from an environment with those features compiled

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **`caro ai --once`** (oldest 'Last tested' = never; lowest # tie-break among never-tested surfaces).

- `caro ai --help` → **PASS**: help text renders correctly; `--once`, `--new-session`, `--continue-session` flags documented
- `caro ai --once 'list files'` → **FAIL**: process hangs indefinitely with zero output (no error, no progress, no TTY prompt). Confirmed not stdin-blocking (tested with `< /dev/null`). Process enters sleeping state (6 threads) — blocked on model binary download via `HfHubClient` which has no HTTP timeout (`src/cache/http_client.rs:42`). Behavior identical to `caro -p '...' --dry-run` FLAKE-001, but `caro ai` has no static-matcher fallback so the hang is guaranteed whenever no model is cached.
- `caro suggest 'list files'` → **PASS** (comparison surface): returns multi-suggestion output via static backend immediately.

### Findings

- [#1440](https://github.com/wildcard/caro/issues/1440) — `embedded: model download hangs indefinitely — HfHubClient has no HTTP timeout` (P1, regression)

### Followups

- FLAKE-001 second observation (2026-09-06). The BEHAVIOR CHANGED from v1.3.0 (retry-with-error) to v1.5.0 (silent hang). Filed as #1440 (P1 regression). Update FLAKE-001 entry in qa-known-flakes.md accordingly.
- Next Slot C candidate: surface #11 (`caro ai --continue-session`) OR surface #12 (`caro assess`) — both never tested. Recommend #12 since it doesn't require LLM backend.
- Surface #10 remains FAIL until #1440 is resolved; mark it as blocked on #1440 in next pass.
- Slot B found 15 new safety tests from PR #1315 P0 fix — all green. Safety surface in excellent shape.
- PR #1209 (remote backends) not exercisable from this sandbox without `--features remote-backends`. Add note to coverage matrix.

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
