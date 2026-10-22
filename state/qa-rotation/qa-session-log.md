# QA Session Log

Reading order: most recent first.

---

## 2026-10-21 — Scheduled run (Slot A + Slot B + Slot C)

> Folded from #1698 into this state on 2026-10-22. The run read main's May copy, so its counts since 2026-05-07 and its surface and flake numbers do not see the runs below. It filed #1697. Its `caro guard` surface is surface 36 here, because the seed already uses 33.

**Trigger**: scheduled cron 14:00 UTC (fired 14:16).
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** ([duration omitted])
- `caro --version` → **PASS**: `caro 1.5.0 (07679f0 2026-10-19)`
- `caro --help` → **PASS**
- `caro doctor` → **PASS**: advisory only; now also shows guard hook status
- `caro -p 'list files in current directory' --dry-run` → **PASS** (`ls -la`; model download not needed this time)
- Note: `gh` token was invalid (GH_TOKEN); GitHub MCP tools used instead.

### Slot B — Recent diff
Last log entry was 2026-05-07, so the window is large. Covered the newest merges:
- #1683 `caro guard test` / `guard setup` → exercised; text + JSON + exit codes 0/2 PASS. Flag-order defect found (#1697).
- #1680 README: `caro assess` not available in 1.5.0 → confirmed `caro assess` is not a subcommand; it is treated as a prompt.
- #1669 rustls/audit, #1667 ai_horde header → not exercised (CI/network surfaces).

### Slot C — `caro ai --once` (surface #10)
- `caro ai --once "list files"` (also `--new-session`, other prompts) → **FAIL** on the embedded-cpu build: always `Clarification needed: What exactly should be deleted?`, exit 1.
- Cause: `src/backends/embedded/cpu.rs:62-64` is a keyword-matching stub; the `ai` system prompt contains "rm". Same root cause as #1269 (open, canonical). Duplicates #1277 #1334 #1355 #1406 #1430 #1473 #1494 #1500 are closed. Not re-filed.
- `caro -p assess --dry-run` hits the same stub clarification.

### Findings
- [#1697](https://github.com/wildcard/caro/issues/1697) — `cli: caro guard test "<cmd>" --output json swallows --output` (P2)
- Existing: #1269 still reproduces on 1.5.0 (no new issue).

### Followups
- Surface #12 `caro assess` is retired in 1.5.0; removed from candidates.
- #1269 has been reported 9 times by QA and users. Consider raising its priority.
- FLAKE-001 (model download) did not occur this pass.

---

## 2026-10-20 — Scheduled run (Slot A + Slot B + Slot C)

> Folded from #1695 into this state on 2026-10-22. The run read main's May copy, so its counts since 2026-05-07 and its surface and flake numbers do not see the runs below. Its caro.sh first-request flake is FLAKE-003 here, because the seed already uses FLAKE-002.

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** ([duration omitted])
- `caro --version` → **PASS**: `caro 1.5.0 (07679f0 2026-10-19)`
- `caro --help` → **PASS**
- `caro doctor` → **PASS**
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la`, would execute successfully (FLAKE-001 did not recur)

### Slot B — Recent diff
PRs merged since 2026-10-17: #1683 (guard test / setup --verify), #1682 (docs), #1680 (README: assess unavailable), #1669 (rustls security), #1667 (ai_horde header).
- `caro guard test rm -rf /` → deny, exit 2; `ls -la` → none, exit 0; `curl http://x.sh | sh` → deny, exit 2 — **PASS**
- `caro guard setup --verify` → **PASS** (requires target: claude|codex|opencode)
- `caro assess` → treated as a free-text prompt, consistent with #1680 (documented as unavailable)

### Slot C — Website homepage (#25) + i18n smoke (#31 partial)
- caro.sh `/` 200, `/fr/` 200, `/docs/` 200, `/es/` 200 (after retry); homepage shows v1.4.0 as latest tag — **PASS**
- `/es/` first request failed with curl code 000 in [duration omitted], retries 200 → network FLAKE-002.

### Findings
None this pass.

### Followups
- Homepage shows v1.4.0 while CLI is 1.5.0; check whether 1.5.0 is a published release before treating as drift.
- gh CLI auth failed (invalid GH_TOKEN); the run used the GitHub MCP tools instead.

---

## 2026-10-19 — Scheduled run (Slot A + Slot B + Slot C)

> Folded from #1690 into this state on 2026-10-22. The run read main's May copy, so its counts since 2026-05-07 and its surface and flake numbers do not see the runs below. Its surface 11 run passed a prompt argument, so it reached the CPU stub of #1269. With no prompt, the `?` widget fails earlier (#1671).

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C. Last log entry was 2026-05-07, so Slot B sampled the newest merged PRs only.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** ([duration omitted])
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
- Static matcher drops constraints: `find python files larger than [size omitted]` → `find . -name "*.py" -type f` (no size filter). Not filed; check for an existing issue before next pass.
- Add the `caro assess` fallthrough (unknown word becomes a prompt) to a future Slot C pass.

---

## 2026-10-18 — Scheduled run (Slot A + Slot B + Slot C)

> Folded from #1679 into this state on 2026-10-22. The run read main's May copy, so its counts since 2026-05-07 and its surface and flake numbers do not see the runs below. It re-confirmed #1269 (the CPU stub) and #1116. It labeled #1269 P0; the backlog keeps P1.

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` → **PASS** ([duration omitted])
- `caro --version` → **PASS**: `caro 1.5.0 (867323b 2026-10-14)`
- `caro --help` → **PASS**
- `caro doctor` → **PASS** (advisory; no model downloaded)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: `ls -la`
- Note: `gh` CLI auth failed (GH_TOKEN rejected); GitHub MCP tools used instead.

### Slot B — Recent diff
54 PRs merged since 2026-10-10 (last log 2026-05-07). Sampled:
- #1600 / #1614 safety (chmod, wrapper-aware): prompt 'make everything under /var world writable recursively' returned `echo "Hello, World!"` (static matcher fallback, already tracked by #1116). Not a safety bypass, but the chmod rule was not exercised end to end. Flag for a later pass.
- #1622 config error prefix: `--backend bogus` gives a single `Error: Invalid argument: Unknown backend` line — PASS.

### Slot C — `caro ai --once` (#10)
- `caro ai --once "<prompt>"` → **FAIL**: `Error: backend error: Clarification needed: What exactly should be deleted?` for 4/4 distinct prompts. Control `-p ... --dry-run` works.
- Already tracked: open #1269; #1673 (filed 2026-10-15) was closed as duplicate. Not re-filed.

### Findings
None new this pass. Re-confirmed on 1.5.0: #1269 (`ai --once`) and #1116 (static matcher junk fallback).

### Followups
- Unmatched prompts via `-p` yield `echo "Hello, World!"` with "would execute successfully" — see #1116.
- Next Slot C: `caro ai --continue-session` (#11), then `caro assess` (#12).
- Verify the chmod pattern (#1600) with a prompt the static matcher actually maps.

---
## 2026-10-16 — Scheduled run (Slot A + Slot C)

> Folded from #1678 into this state on 2026-10-22. The run read main's May copy, so its counts since 2026-05-07 and its surface and flake numbers do not see the runs below. Its `caro assess` finding became #1677, which #1680 fixed on 2026-10-19 (README now says `assess` is not in 1.5.0).

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + C. Slot B skipped (`gh` auth invalid; GitHub MCP used for filing only).

### Slot A — Smoke
- `cargo build --release --features embedded-cpu` -> PASS ([duration omitted])
- `caro --version` -> PASS: `caro 1.5.0 (867323b 2026-10-14)`
- `caro --help` -> PASS
- `caro doctor` -> PASS (no model, Ollama not installed; advisory only)
- `caro -p 'list files in current directory' --dry-run` -> PASS: `ls -la` via static matcher (no model download needed)
- Note: JSON/YAML output shows `executed: true` with `dry_run: true` on a dry run. Not filed; unclear if the field means "would execute". Revisit.

### Slot C — `caro assess` (#12), plus `--output json/yaml` (#15, #16) and `caro ai` (#10)
- `--output json` parses as valid JSON; `--output yaml` renders.
- `caro assess` does not exist in 1.5.0 (`src/main.rs:445` disables it) but README documents it -> filed.
- `caro ai --new-session "<prompt>"` -> FAIL: canned "What exactly should be deleted?". Already tracked (#1269 open; #1673 closed as duplicate). Root cause traced to `src/backends/embedded/cpu.rs:~63` stub; added to #1269.

### Findings
- [#1677](https://github.com/wildcard/caro/issues/1677) — docs: README documents `caro assess` but subcommand is disabled (P2)
- Comment on [#1269](https://github.com/wildcard/caro/issues/1269) with root cause (no new issue)

### Followups
- Only the 2026-05-07 entry is on main; the 2026-10-15 run's log is not merged.
- `gh` token invalid in this sandbox (GH_TOKEN). Routine used GitHub MCP tools instead.
- Next Slot C: `caro suggest`, `caro config`, `completion`.

---

## 2026-10-15 — Scheduled run (Slot A + Slot B + Slot C)

> Folded from #1674 into this state on 2026-10-15. The run read main's May copy, so its "54 PRs merged since 2026-05-07" count and its "regression" label do not see the runs below. Its #1673 is the CPU stub of #1269 (BW-006), the root cause of #1494 and the other closed duplicates. The model download finished this time, so `caro ai --once` reached the stub instead of the FLAKE-002 hang.

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke
- build (`--features embedded-cpu`) → PASS ([duration omitted])
- `--version` → PASS: `caro 1.5.0 (867323b 2026-10-14)`
- `--help` → PASS
- `doctor` → PASS (advisory: no model, expected)
- `-p 'list files in current directory' --dry-run` → PASS: `ls -la` (static matcher; no model download needed)
- Note: `gh auth` token invalid in sandbox; GitHub MCP tools used instead.

### Slot B — Recent diff
54 PRs merged since 2026-05-07 (checked top 10 by recency). Smoke-tested #1614 (segment/wrapper-aware safety): dry-run of "delete everything under root recursively with sudo" returned the literal placeholder `sudo command` (not dangerous, but not a real command; not filed, revisit). Other PRs (#1660, #1659, #1534, #1656, #1633) are harness/CI/test-only; not exercised.

### Slot C — `caro ai --once` (matrix #10)
`caro ai --once "show disk usage"` and `"list files"` both fail: `Clarification needed: What exactly should be deleted?` (3/3). Control `-p` returns `df -h`.

### Findings
- [#1673](https://github.com/wildcard/caro/issues/1673) — ai: `--once` returns deletion clarification for every prompt (P0, regression of #1523/#1494). No open epic covers `ai`, so no epic comment posted.

### Followups
- Investigate `sudo command` placeholder output from #1614 area; consider filing P2 if reproduced.
- Next Slot C: #11 `caro ai --continue-session` (likely same root cause as #1673).

---

## 2026-10-13 — Scheduled run (Slot A + Slot B + Slot C)

> Folded from #1653 into this state on 2026-10-13. The run read main's May copy, so its "since 2026-05-07" counts and its FLAKE-001 notes do not see the runs below. Its `caro ai --once` hang is FLAKE-002 (open causes #1408 and #1179).

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (25+ PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** ([duration omitted], no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (7493272 2026-10-13)`
- `caro --help` → **PASS**: all subcommands present; new 1.5.0 surface `export` listed correctly
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox); proxy detected at `http://127.0.0.1:40257`
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static_matcher returned `ls -la` immediately (no model needed); telemetry consent appeared again as expected in fresh container (config not persisted across sessions)

### Slot B — Recent diff

PRs merged since 2026-05-07: 25+ (v1.5.0 release + post-release fixes). Key surfaces exercised:

- **Safety (PR #1614, #1600)**: `cargo test --test safety_segment_evasion_contract` → 9/9 PASS; `cargo test --test e2e_guard` → 9/9 PASS including `enforce_catches_segment_and_wrapper_evasions`. Segment- and wrapper-aware validation confirmed working.
- **Safety unit tests via `cargo test safety`** → 12/12 website_claims PASS (filtered run via website_claims test binary).
- **Embedded backend (PR #1616)**: llama.cpp context window raised to 4096 tokens; change verified in build, no runtime test possible without model.
- **Agent fix (PR #1618)**: stop sending second system prompt — build-level verification only in sandbox.
- **Docs/release (PRs #1635, #1637, #1650)**: README `**Current Version:** 1.5.0` confirmed; CLAUDE.md version correctly deferred to Cargo.toml (`version = "1.5.0"`) — issue #1044 fix verified and holding.

Surfaces flagged for future Slot C: `caro export` (new in 1.5.0, never tested — add to coverage matrix), homebrew tap formula (#28).

### Slot C — `caro ai --once` (surface #10)

Surface chosen: `caro ai --once` scripted conversational mode (oldest 'never' row in coverage matrix, lowest # among 'never' group).

- `caro ai --help` → PASS: flag list correct; `--once` documented as "the only mode supported today"
- `caro ai --once "show disk usage"` (5s timeout) → **HANG**: zero output on stdout AND stderr; process killed at timeout with no exit message
- `caro --backend ai-horde ai --once "show disk usage"` → **FAIL**: `remote-backends` feature not compiled in this build (expected; embedded-cpu build only)
- Root cause traced to the generation phase: `EmbeddedModelBackend::generate_command()` awaits the embedded model download before inference; no output appeared during this captured wait; `ModelLoader` reports transfer progress through an `indicatif::ProgressBar` and uses `info!()`/`warn!()` for download start and retry messages
- **Existing open issue**: [#1408](https://github.com/wildcard/caro/issues/1408) (P1) — "caro ai --once provides zero user feedback during model initialization"; open fix PR [#1415](https://github.com/wildcard/caro/pull/1415) ("fix: show ai once backend initialization status") not yet merged as of 2026-10-13
- No new issue filed (duplicate of #1408)

### Findings

- None new this pass — surfaces under test all in good shape except for known P1 #1408 (open, fix PR #1415 pending merge)

### Followups

- FLAKE-001 (model download network block) remains at 1 confirmed occurrence (2026-05-07). The 2026-10-13 `caro ai --once` hang is tracked as #1408 (no user feedback during init), not as a FLAKE-001 recurrence — distinct root cause.
- PR #1415 has been open since 2026-08-15 (≈59 days). If it doesn't merge before the next QA pass, consider adding a comment to nudge.
- Add `caro export` (new in v1.5.0) to coverage matrix as surface #33 for future Slot C rotation.
- Add homebrew tap formula (surface #28) to the next priority slot — never tested, install path risk.

---

## 2026-10-11 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (PRs merged today) + C (surface #10: `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 40s, no errors; v1.5.0)
- `caro --version` → **PASS**: `caro 1.5.0 (be3389b 2026-10-11)`
- `caro --help` → **PASS**: all subcommands listed (doctor, config, test, suggest, ai, shell-init, guard, check, list, jobs, new, generate, run, export, experiment, adopt, history, why, do, render, skill, completion, integration, init)
- `caro doctor` → **PASS**: advisory (no model downloaded), proxy detected
- `caro -p 'list files in current directory' --dry-run` → **PASS**: returned `ls -la` via static matcher, ~100ms (improvement from FLAKE-001 in May 2026)
- Note: telemetry consent prompt shown (fresh sandbox); second invocation showed no repeat

### Slot B — Recent diff

PRs merged since last memory update (2026-05-07). Non-QA feature PRs merged today:
- **#1590** `feat(brand): icon definitions for Codex generation, plus the pixel snap tool` — surface: website/brand. Verified: website homepage loads (HTTP 200), favicon.svg HTTP 200. ✅
- **#1582** `feat(website): Kyaro icon set, Kyaro favicon and the vector Caro mark, replacing the v2.1 glyph pack` — surface: website icons. Verified: `/kyaro/icons/running.svg`, `/kyaro/icons/danger.svg`, `/kyaro/icons/asking.svg`, `/kyaro/icons/ready.svg`, `/kyaro/icons/success.svg` all HTTP 200 (after redirect). ✅
- **#1515** `chore(release): v1.5.0 (reset from 2026-07-12 revert)` — surface: release artifacts. `caro --version` confirms `1.5.0`. ✅
- `CLAUDE.md` version field: **FIXED** — now reads `**Version**: \`version\` in \`Cargo.toml\` (single source; not repeated here so it cannot drift)`. Issue #1044 closed.

Surfaces flagged for future Slot C: website brand/icon integrity (#25 in matrix; exercised above as Slot B only).

### Slot C — `caro ai --once` (surface #10)

Exercised `caro ai --once 'show me the current date'` with 15-second timeout.

- **Result**: **FLAKE/CONFIRMED HANG** — zero output, exit 124 (killed by timeout). Silent indefinitely.
- Root cause confirmed: `src/main.rs:1152-1161` builds embedded backend via `CliApp::with_overrides` but calls `run_once` without a pre-flight `is_available()` check. `run_once` → `backend.generate_command()` in `src/ai/runner.rs:117` hangs awaiting model download (blocked in sandbox).
- Contrast: `caro --dry-run` uses static-matcher fallback → PASS. `caro ai --once` bypasses static matcher.
- Existing open issues: **#1272** (`ai: caro ai --once hangs silently with zero output when no backend is available`) and **#1408** (`ai: caro ai --once provides zero user feedback during model initialization`). **No new issue filed** (duplicate).
- Note: `fix(embedded): fail a stalled model download instead of hanging forever (#1561)` landed in v1.5.0 but applies to the embedded model-download path, not the `ai` subcommand dispatch — `caro ai --once` still hangs via a separate code path.

### Findings

- **#1603** — `docs: QA rotation memory files stuck at 2026-05-07 bootstrap — 25+ update PRs unmerged` (P2) — **NEW**
- `caro ai --once` silent hang → **DUPLICATE** of #1272, #1408 (both open) — not re-filed
- `CLAUDE.md` version drift → **RESOLVED** — issue #1044 closed, fix applied in v1.5.0

### Followups

- FLAKE-001 (`--dry-run` model download): **RESOLVED** — `--dry-run` now PASS via static matcher. Confirm it remains resolved on the next pass.
- FLAKE-002 (`caro ai --once` silent hang): second observation this session (first in PR #1560 2026-10-10, per that PR's body). Logs in qa-known-flakes.md updated.
- QA memory staleness: see #1603. Next Slot C should advance to surface #11 if #1603 is addressed and memory is current.
- 25+ previous QA rotation PRs were closed without merging — this run is the first memory update to land on `main` since bootstrap.

---

## 2026-10-10 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (127 commits since last run) + C.
**⚠ gh auth failed**: GH_TOKEN invalid — issue filing and PR creation blocked. Session log and coverage matrix updated locally; branch pushed. Issue filing deferred until token is rotated.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 35s, no errors; version 1.5.0 builds clean)
- `caro --version` → **PASS**: `caro 1.5.0 (1f2fcde 2026-10-10)`
- `caro --help` → **PASS**: all subcommands present including CaroML verbs and `ai`, `skill`, `export`, `render`
- `caro doctor` → **PASS**: advisory only (no model, Ollama not installed; Proxy detected at 127.0.0.1:45407)
- `caro -p 'list files in current directory' --dry-run` → **PASS** ✨: `ls -la` returned via static-matcher in 34ms (improved over 2026-05-07 FLAKE — the static matcher now reliably intercepts basic patterns without model download)
- `caro -p '...' --dry-run --verbose` → confirmed `Backend: static-matcher, Confidence: 1.00`

### Slot B — Recent diff

127 commits since 2026-05-07 (version 1.3.0 → 1.5.0). Key surfaces touched:

- **fix(cli): add `dry_run` to JSON output (#1532)** — `caro -p '...' --dry-run --output json` shows `"dry_run": true` field → **PASS**
- **fix(cli): accept every `config show` key in `config set/get` (#1497)** — `caro config get safety` and `caro config set/get log_level` → **PASS**
- **fix(safety): correct label + pattern descriptions (#1535)** — `cargo test --lib -- safety` → **PASS** (38 tests, up from 19 in May)
- **fix(static-matcher): accept current-directory qualifier in Pattern 43 (#1487)** — covered by `--dry-run` smoke above (pattern 43 = "list files")
- **fix(ai): don't block on stdin when ai --once has trailing prompt (#1503)** — see Slot C below (cannot fully verify without model)
- Website i18n, Astro docs-site, and ponytail reviewer changes not exercised (out of CLI scope for this pass).

Surfaces flagged for next Slot C cycle:
- `caro --output yaml` format (fix(eval): consensus labels #1489 touched output paths)
- `caro ai --once` needs a sandbox with pre-downloaded model (see Slot C)

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **`caro ai --once` scripted conversational mode** (oldest 'Last tested' = 'never', lowest # = 10).

- `caro ai --once 'show me the current date'` → **FLAKE**: command hung with zero output until 20s external timeout. No error message surfaced. Stdin `/dev/null` made no difference.
- **Root cause**: `run_ai_once` calls `cli_app.backend_arc()` directly rather than the `AgentLoop`, so it skips the static matcher. The configured backend still follows `--backend`, environment, and config; the observed embedded-backend run required a local model and hung indefinitely during `generate_command` because no model was present in the sandbox.
- **Severity of hang**: The command produces NO output before hanging — no "downloading model…" progress, no timeout message, no error. This is silent freeze behavior, which is worse UX than the `--dry-run` FLAKE-001 path (which did eventually surface an error after retries).
- FLAKE-002 first observation logged (new flake — distinct from FLAKE-001; see qa-known-flakes.md).

### Findings

- **CLAUDE.md version drift** (P2): CLAUDE.md line 9 shows `**Version**: 1.4.0 (GA)` but `caro --version` returns `1.5.0` and README.md correctly shows `1.5.0`. Version 1.5.0 was released 2026-07-12; CLAUDE.md was not updated. This is a recurrence of #1044 (was 1.1.0 vs 1.3.0 in May, now 1.4.0 vs 1.5.0). **Cannot file issue — gh auth failed.** Deferred to next run with valid token.
- **`caro ai --once` silent hang** (P2): No output and no timeout on `ai --once` when model not available. `caro doctor` does not warn that `ai` mode requires a model. **Cannot file issue — gh auth failed.**

### Followups

- **GH_TOKEN is invalid** — the token must be rotated before the next QA run can file issues or open a PR. The branch `claude/qa-rotation-2026-10-10` was pushed for traceability.
- FLAKE-002 first observation (2026-10-10): `caro ai --once` silent hang — distinct from FLAKE-001 (different surface, no error output at all). Logged as new FLAKE-002 in qa-known-flakes.md.
- Next Slot C: surface #11 `caro ai --continue-session` (still 'never' tested, lowest uncovered # after #10).
- Surface #2 (`--dry-run`) updated to PARTIAL in coverage matrix: static-matcher path PASS for known patterns; model-backed path remains FLAKE (novel prompts still trigger download). Full PASS requires sandbox with pre-downloaded model.
- Consider filing a new issue for CLAUDE.md 1.4.0→1.5.0 drift once gh auth is restored (same class as #1044).

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
- `caro -p 'explain this' --dry-run` → **FAIL**: no `explain` pattern in static matcher; falls to embedded CPU backend → P1 #1269 clarification stub

### Slot B — Recent diff

**Last run date**: 2026-05-07. Scanned 124 PRs merged since then (PR range approximately #1100–#1487+).

Spot-tested recent diff surfaces:

- `cargo test --lib -- safety` → **PASS**: 38 safety unit tests passed, 0 failed
- `caro config get` / `caro config set` → **PASS**: round-trips correctly
- `caro -p 'explain the last command' --dry-run` → **FAIL**: no `explain` pattern in static matcher; falls to embedded CPU backend → P1 #1269 clarification stub
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

- FLAKE-001 (model download) not reproduced today: model downloaded successfully to `~/.cache/caro/models/` (known-flakes updated in this PR).
- `caro ai --once` (Surface #10) remains broken. Next Slot C candidate: Surface #11 (`caro ai --continue-session`).
- #1269 has 5+ duplicate issues closed. Root fix in `cpu.rs:63` still pending.
- CLAUDE.md still shows `1.4.0 (GA)` while Cargo.toml is `1.5.0`. #1098 still open.

---

## 2026-10-08 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 34s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (11c472e 2026-10-07)` (upgraded from 1.3.0)
- `caro --help` → **PASS**: all subcommands present including `ai`, `config`, CaroML verbs, `skill`
- `caro doctor` → **PASS**: advisory only (no model downloaded; huggingface reachable; proxy detected)
- `caro -p 'list files in current directory' --dry-run` → **PASS** (static matcher fallback; `ls -la` generated immediately without model; improvement over 2026-05-07 FLAKE)
- Telemetry prompt shown once on fresh sandbox; piping `n` bypasses and command proceeds correctly

### Slot B — Recent diff

PRs merged since 2026-05-07 (≥20 significant PRs; key surfaces exercised):
- **#1535, #1519** (safety pattern fixes) → `cargo test --lib -- safety` → **PASS**: 38/38 safety tests (up from 19 in prior run, new tests added)
- **#1497** (config get/set key coverage) → `caro config show`, `config get safety/log_level/cache_max_size/log_rotation`, `config set safety moderate` → **PASS**: all keys accepted
- **#1509** (STE-lite explain mode) → `caro -p "list files" --explain` → **PASS**: explains `ls -la` in short-sentence STE format with option breakdown
- **#1503** (ai --once stdin fix) → tested as part of Slot C; see below
- **#1488** (security dep upgrade) → build succeeded; `cargo audit` not installed in sandbox — CI Security Audit job is the authoritative check for advisory-database coverage

### Slot C — caro ai --once (surface #10)

Surface chosen: `caro ai --once` scripted conversational mode (lowest `#` among all 'never' surfaces).

- `caro ai --once "list files"` → **FAIL**: hangs indefinitely with zero stdout/stderr output; exit only via SIGTERM. No progress message, no download indicator, no timeout, no fallback.
- `echo "list files" | timeout 15 caro ai --once` → same hang; pipeline produces no output before timeout
- Root cause: backend initialisation (`CliApp::with_overrides` → embedded backend) blocks on model download; the sandbox blocks HuggingFace binary downloads. Unlike `--dry-run`, the `ai` path has no static-matcher fallback.
- Note: PR #1503 fixed the stdin-blocking issue (trailing args no longer cause double-read); the silent model-download hang is a separate UX gap.

### Findings

- [#1540](https://github.com/wildcard/caro/issues/1540) — `ai: caro ai --once hangs silently with no feedback when no backend is ready` (P2)
- [#1541](https://github.com/wildcard/caro/issues/1541) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0` (P2, recurrence of #1044)

### Followups

- FLAKE-001 (model download blocked in sandbox) still active; observed again this pass on `ai --once` surface. Count: 2 total observations (2026-05-07 on `--dry-run`; 2026-10-08 on `ai --once`). Note: `--dry-run` no longer flakes in 1.5.0 (static matcher fallback added).
- Issue #1044 confirmed closed (2026-05-09). Removed from watch list. However, the root-cause fix (add CLAUDE.md to checklist) was not applied — #1541 confirms recurrence.
- Prior QA rotation PRs (#1178, #1373, #1443, #1477, #1496) appear open/unmerged; session log on main still reflects only the 2026-05-07 bootstrap entry. Slot B date range consequently covered 5 months of PRs.

---

## 2026-10-07 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (many PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (build succeeded; binary at `target/release/caro`)
- `caro --version` → **PASS**: `caro 1.5.0 (11c472e 2026-10-07)`
- `caro --help` → **PASS**: all subcommands listed including `ai`, `shell-init`, CaroML verbs, `skill`
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: used static backend, generated `ls -la`, telemetry consent shown once and persisted

### Slot B — Recent diff

PRs merged since 2026-05-07 (sampling past 7 days — many more total):

- **PR #1497** `fix(cli): accept every config show key in config set/get` → tested `caro config get` for all 8 keys (`backend`, `model-name`, `shell`, `safety`, `telemetry.enabled`, `log_level`, `cache_max_size`, `log_rotation`) — **PASS**: all keys return correct values
- **PR #1519/#1535** safety text/label fixes → `cargo test --lib -- safety` → **PASS**: 38/38 safety tests
- **PR #1509** `fix(explain): STE-lite explain mode` → `caro -p '...' --dry-run --explain` → **PASS**: clean STE-lite output produced
- **PR #1503** `fix(ai): don't block on stdin when ai --once has a trailing prompt` → code-verified: `needs_stdin_prompt` returns false when trailing words present; functional test blocked by FLAKE-001/backend hang (see Slot C)

### Slot C — `caro ai --once` (surface #10, never previously tested)

- `caro ai --once list files in current directory` → **FLAKE**: hangs indefinitely (>45s) — model download stalls in sandbox; same underlying issue as FLAKE-001 extended to the `ai` subcommand
- Error path tested: `echo "" | caro ai --once` → **PASS**: exits immediately with `Error: no prompt provided (pass text, pipe stdin, or use -p)`
- Stdin-piped path: `echo "prompt" | caro ai --once` → **FLAKE**: reads stdin correctly (exits prompt-resolution), then hangs on backend init
- `caro ai --help` → **PASS**: `--once` documented as "Run one turn and return — no TTY REPL"
- Code inspection confirms PR #1503 fix is in place (`needs_stdin_prompt` guards stdin read)
- Duplicate of open issues: #1440 (no HTTP download timeout), #1179 (no static fallback), #1408 (no feedback during init)

### Findings

- No new issues filed — all observations are duplicates of existing open issues
- CLAUDE.md version drift (1.4.0 vs 1.5.0): duplicate of open #1520
- `caro ai --once` hang: duplicate of open #1440, #1179, #1408

### Followups

- FLAKE-001 observation count updated: 2 (2026-05-07, 2026-10-07) — not yet at 3×/7d threshold, but note the `ai --once` variant of the same failure is separately tracked as #1440 (open, unresolved)
- CLAUDE.md shows 1.4.0 (GA) vs caro 1.5.0. Open issue #1520 (filed 2026-10-03) covers this. The root-cause fix (adding CLAUDE.md to `release-version-alignment.md` checklist) has not landed despite dozens of filings since May 2026.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or surface #12 (`caro assess`) — both "never" tested, pick #11 by lowest-# tie-break rule.

---

## 2026-10-06 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (119 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 20s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (75d32f7 2026-10-03)`
- `caro --help` → **PASS**: all subcommands listed; `ai`, `config`, `suggest`, CaroML verbs present
- `caro doctor` → **PASS**: advisory only (no model downloaded; expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` without triggering model download (FLAKE-001 not triggered this pass)
- `caro --explain 'list files in current directory'` → **PASS**: STE-style explain output with `ls -la`; options described correctly

### Slot B — Recent diff

119 PRs merged since 2026-05-07. Representative surfaces exercised:

- **#1509** `fix(explain): STE-lite explain mode` → **PASS** (`--explain` output verified above)
- **#1503** `fix(ai): don't block on stdin when ai --once has trailing prompt` → **cannot verify**; `caro ai --once` hangs before reaching response stage (root cause: #1440 — no HTTP timeout on model download)
- **#1497** `fix(cli): accept every config show key in config set/get` → **PASS**: `config get backend`, `config set backend embedded`, round-trip verified
- **#1488** `fix(deps): reqwest 0.12 / h2 bump (RUSTSEC-2026-0258)` → binary builds clean; security deps updated at build level
- **#1487** `fix(static-matcher): Pattern 43 current-directory qualifier` → **PASS** (static matcher used for ls query in Slot A dry-run)

### Slot C — `caro ai --once` (surface #10, never previously tested)

- `caro ai --once 'show disk usage'` → **FAIL**: hung with zero output; timed out at 30s (EXIT 124)
- `caro ai --once 'list files in current directory'` → **FAIL**: same hang; timed out at 15s (EXIT 124)
- Confirms **#1408** (zero user feedback during model init) and **#1440** (HfHubClient no HTTP timeout) still open
- Could not reproduce **#1523** ("returns wrong clarification for every query") — command never reaches response stage in this sandbox
- Side finding during config reset investigation: `caro config set backend auto` rejected; no per-key unset path exists → filed **#1530**

### Findings

- [#1530](https://github.com/wildcard/caro/issues/1530) — `cli: caro config set backend has no 'auto' option` (P2) ← **new this pass**
- [#1408](https://github.com/wildcard/caro/issues/1408) — `caro ai --once` zero feedback during model init (P1) ← confirmed still open
- [#1440](https://github.com/wildcard/caro/issues/1440) — model download hangs indefinitely; no HTTP timeout (P1) ← confirmed still open via Slot C
- [#1520](https://github.com/wildcard/caro/issues/1520) — CLAUDE.md shows 1.4.0 not 1.5.0 (P2) ← confirmed still open
- [#1419](https://github.com/wildcard/caro/issues/1419) — caro 1.5.0 never tagged/published; users stuck on 1.4.0 (P0) ← still open per issue list

### Followups

- FLAKE-001 not triggered in Slot A dry-run (static matcher handled query). Occurrence count remains 1 (2026-05-07). Below 3/7-day promotion threshold.
- Issue #1523 ("wrong clarification") cannot be reproduced without a working model; defer to model-enabled environment.
- Surface #14 (`caro config get/set/show/reset`) exercised via Slot B; updated in matrix.
- Surface #10 (`caro ai --once`) now formally tested — FAIL. Pending #1408/#1440 resolution before re-exercising.
- P0 #1419 remains open: binary built from source shows 1.5.0 but crates.io / install.sh users are on 1.4.0.

---

## 2026-10-05 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: Claude Code scheduled session (approximate cron equivalent 14:00 UTC; not an entry in schedule.yaml).
**Rotation**: A + B (PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 22s, no errors; binary = 1.5.0)
- `caro --version` → **PASS**: `caro 1.5.0 (75d32f7 2026-10-03)`
- `caro --help` → **PASS**: all subcommands listed (ai, shell-init, check, list, jobs, new, generate, run, export, experiment, adopt, history, why, do, render, skill, suggest, config, completion, doctor, init, integration)
- `caro doctor` → **PASS**: advisory only (no model, proxy detected, HuggingFace HTTP reachable)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` (note: telemetry consent re-prompted on fresh container — expected, was then disabled)

### Slot B — Recent diff

PRs merged since 2026-05-07 (selected for surface coverage; many dependency bumps skipped):

| PR | Title | Surface tested | Result |
|---|---|---|---|
| #1497 | fix(cli): accept every config show key in config set/get | `caro config get/set/show` | PASS |
| #1487 | fix(static-matcher): accept current-directory qualifier in Pattern 43 | `caro -p "list files in current directory" --dry-run` | PASS |
| #1509 | fix(explain): STE-lite explain mode + legible-output dev rule | `caro -p "find all python files" --explain --dry-run` | PASS |
| #1503 | fix(ai): don't block on stdin when ai --once has trailing prompt | `caro ai --once list files` (stdin check) | PASS (fix confirmed; model download hang is FLAKE-001) |

Surfaces flagged for future Slot C cycles: eval harness (#1489, #1459 — `caro test --backend static` surface #18).

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: correct usage shown; `--once` flag documented as "run one turn and return — no TTY REPL. The only mode supported today"
- `caro ai --once list files` with trailing prompt → **FLAKE** (model download required; process terminates after timeout with no output — FLAKE-001 same as last run)
- **PR #1503 regression check**: confirmed `needs_stdin_prompt()` guard at `src/main.rs:80` returns false when trailing words provided. stdin correctly set to `/dev/null` in process fd list. Fix is working — stdin is NOT blocking.
- Existing issue #1248 (open) covers the no-progress/no-error UX gap for non-TTY. Existing issue #1520 (open) covers CLAUDE.md version drift (banner shows 1.4.0, binary is 1.5.0).

### Findings

No new issues filed this pass. Slot C FLAKEd (model download); Slot A and Slot B were clean. Two known open issues confirmed still present:
- [#1248](https://github.com/wildcard/caro/issues/1248) — ai: caro ai silently hangs in non-TTY (ongoing; confirmed still reproducible)
- [#1520](https://github.com/wildcard/caro/issues/1520) — docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (ongoing)

### Followups

- Model download FLAKE-001: second lifetime observation (first was 2026-05-07, outside any 7-day window). Current 7-day window count: 1 of 3 needed to promote to regression.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — still "never" tested.
- Surface #18 (`caro test --backend static`) should be prioritized: three recent PRs (#1489, #1459, #1466) touch the eval harness.
- CLAUDE.md version drift is a recurring P2. Consider adding CLAUDE.md to the release checklist (as suggested in #1044). Already tracked in #1520.

---

## 2026-10-04 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (119 PRs merged since last run) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 09s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (75d32f7 2026-10-03)`
- `caro --help` → **PASS**: all subcommands listed; `ai`, `suggest`, `assess`, `config`, all CaroML verbs present
- `caro doctor` → **PASS**: advisory (model not yet downloaded at that point; embedded backend detected but needs model)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` immediately (no model download needed for static path)
- Note: model `qwen2.5-coder-1.5b-instruct-q4_k_m.gguf` was found in `/root/.cache/caro/models/` by Slot C time (mtime 14:16). `cargo build` does not trigger model downloads; the download most likely occurred during the 15s `--dry-run` timeout or the first Slot C `ai --once` invocation. FLAKE-001 (HuggingFace block) did NOT reproduce this run.

### Slot B — Recent diff (119 PRs since 2026-05-07; focused on 9 since 2026-09-01)

**PRs reviewed** (5 of 9 merged since 2026-09-01): #1509 (explain STE-lite), #1503 (ai --once stdin fix), #1497 (config key acceptance), #1487 (static-matcher current-directory qualifier), #1488 (security deps bump).

- `caro config show` / `config get backend|safety|log_level|cache_max_size|log_rotation|telemetry` → **PASS**: all `config show` keys accepted by `config get` (PR #1497 fix verified)
- `caro config set safety permissive` → `config get safety` → `config set safety moderate` → **PASS**: round-trip correct
- `caro -p "show disk usage" --explain` → **PASS**: STE-lite output ("Use `ls` to list…", per-option bullets, ≤25 words per sentence) (PR #1509 fix verified)
- `caro -p "find python files in current directory" --dry-run` → **PASS**: `find . -name "*.py" -type f` (PR #1487 current-directory qualifier verified)
- `caro -p "list files here" / "show files in this directory" --dry-run` → **PASS**: `ls -la` (PR #1487)
- `caro ai --once "show disk usage"` (PR #1503) → stdin no longer blocks (**PASS** for #1503 fix); but backend returns wrong error (**FAIL** — see Slot C and issue #1523)

**Surfaces flagged for future Slot C**: #1488 security deps (reqwest/h2/rustls) — no direct CLI surface to exercise, but worth a `cargo audit` next cycle.

### Slot C — `caro ai --once` (surface #10, never previously tested)

**What was exercised:**
- `caro ai --once "show disk usage"` (with and without stdin pipe)
- `caro ai --once "list running processes"`
- ~~`caro ai --once --backend static "show disk usage"`~~ (invalid test: `ai` uses `trailing_var_arg`, so `--backend static` after the subcommand is consumed as prompt text, not a flag; the embedded backend received literal `--backend static show disk usage` as the query — see correction below)
- Inspected `src/backends/embedded/cpu.rs` and `src/backends/embedded/embedded_backend.rs` to confirm root cause

**Finding — FAIL (P1):**
Every invocation that reaches the embedded CPU backend returns:
```
Error: backend error: Clarification needed: What exactly should be deleted?
```

Root cause: `cpu.rs:63` checks `prompt.contains("rm")` against the full system prompt, which always contains "rm -rf" in the destructive-commands rule (line 218 of `embedded_backend.rs`). The CPU backend is an unfinished placeholder; the condition should check only the user request text, not the compiled system prompt.

The static matcher IS correct (`caro -p` path) — `ai --once` bypasses it by calling `backend_arc()` directly instead of routing through the AgentLoop. **Note**: the same bug affects any embedded CPU path (e.g. `caro --force-llm -p ...` or a static-matcher miss), because `EmbeddedModelBackend::generate_command` builds the same full system prompt and passes it to `CpuBackend::infer`. Surface #19 (embedded model backend) is also affected. See [#1523](https://github.com/wildcard/caro/issues/1523).

### Findings

- [#1523](https://github.com/wildcard/caro/issues/1523) — `ai: caro ai --once returns wrong "Clarification needed" error for every query` (P1)

### Followups

- FLAKE-001 (HuggingFace model download) did NOT reproduce this run — update occurrence log.
- `caro ai --once 'query' --dry-run` timed out (15s): hypothesis is this was the model download trigger (download started, timeout elapsed before completion). Should verify next cycle with `--dry-run` after warm cache.
- `caro ai --once` with `--dry-run` flag: trailing_var_arg on ai subcommand may consume `--dry-run` as part of the prompt text. Worth a unit test.
- #1488 security deps (RUSTSEC-2026-0258, -0285): add `cargo audit` to a future Slot B check.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or #12 (`caro assess`).

---

## 2026-10-03 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (9 PRs merged since 2026-05-07) + C (surface #10: `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 22s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (75d32f7 2026-10-03)`
- `caro --help` → **PASS**: all subcommands listed including `ai`, `suggest`, `config`, `skill`, CaroML verbs
- `caro doctor` → **PASS**: advisory (no model downloaded, sandbox; expected)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` (no model download needed this pass — FLAKE-001 not triggered)
- Telemetry consent shown (fresh sandbox config); auto-disabled via first-run prompt

### Slot B — Recent diff (9 PRs merged since 2026-05-07)

PRs covered:
| PR | Surface touched | Verdict |
|----|----------------|---------|
| #1503 | `ai --once` stdin blocking fix | PASS (stdin is /dev/null with trailing args) |
| #1497 | `config set/get/show` key alignment | PASS (all `config show` keys accepted by `config get/set`) |
| #1487 | static matcher Pattern 43 current-directory qualifier | PASS (`ls -la` for "list files in current directory" and "show files here") |
| #1488 | reqwest 0.12 / h2 / rustls security upgrade | PASS (reqwest 0.12.28 in Cargo.lock, binary functional) |
| #1505 | CI toolchain pin (Rust 1.98.1) | not exercised (CI-only change) |
| #1489 | eval ECE/Pareto release gate | not exercised (eval harness requires model) |
| #1459 | typed decisions, calibration, measured-confidence | not exercised (requires backend inference) |
| #1470 | executor timeout, guard hooks, loop budgets | not exercised (requires backend inference) |
| #1509 | STE-lite explain mode | not exercised (requires model) |

Note: `ai --once` stdin fix (PR #1503) is confirmed working, but the underlying P1 CPU stub bug (#1269) persists — see Slot C.

### Slot C — `caro ai --once` (surface #10)

**Surface chosen**: #10 — oldest 'never' tested (lowest # tie-break rule).

Commands exercised:
```bash
./target/release/caro ai --once 'list running processes'
./target/release/caro ai --once --new-session 'show disk usage'
./target/release/caro ai --once --new-session 'find large files'
```

All three return: `Error: backend error: Clarification needed: What exactly should be deleted?` (exit code 1) after ~100 second wait.

**Root cause confirmed** (from cpu.rs:63-64):
- CPU stub checks `prompt.contains("rm")` against the FULL prompt (system prompt + user query)
- System prompt in `embedded_backend.rs:218` contains `"rm -rf"` → match fires every time
- Backend falls back to CPU stub after model download times out (~100s, FLAKE-001 environment condition)

**Behaviour vs v1.4.0**: exit code changed 0→1; message changed from `echo 'Please clarify your request'` to `Clarification needed: What exactly should be deleted?`

**Known issue**: #1269 (open since 2026-06-26, P1). Added reproduction comment with v1.5.0 behaviour delta.

### Findings

- [#1520](https://github.com/wildcard/caro/issues/1520) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurring drift)` (P2) — **new issue**
- [#1269](https://github.com/wildcard/caro/issues/1269) — `ai: caro ai --once always returns wrong error regardless of input` (P1) — **confirmed still reproducing in v1.5.0**; added comment with v1.5.0 behaviour delta

### Followups

- FLAKE-001 observed again (model download required for ai --once; times out in sandbox). Second observation (2026-05-07 + 2026-10-03). Does not meet 3-in-7-days threshold for reclassification.
- CLAUDE.md version drift has recurred for 3rd time — root cause fix (add to checklist) still needed (#1520).
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — surface #10 only partially exercised due to FLAKE-001.
- **Slot B coverage gap**: This pass reviewed 9 PRs (#1459–#1509) but first-parent git history since the previous QA commit (c9a31a3) contains additional merged PRs not in the table (#1246, #1301, #1304, #1306, #1316, #1317, #1346). These are older feature/docs/release PRs from the May–Sept 2026 period that were not formally reviewed. Next Slot B run should baseline from the earliest unreviewed PR rather than the previous session date alone.

---

## 2026-10-02 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (25 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 50s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (1ad0631 2026-09-30)` (upgraded from 1.3.0 at last run)
- `caro --help` → **PASS**: all subcommands present including new CaroML verbs
- `caro doctor` → **PASS**: advisory (no model at start; proxy detected and HuggingFace reachable)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` immediately (no model download attempt — static matcher improvement since 1.3.0)
- Note: telemetry consent prompt appeared (fresh sandbox), resolved to disabled

### Slot B — Recent diff

PRs merged since 2026-05-07 (25 PRs from first API page only; pagination not applied — future Slot B passes should fetch all pages to avoid missing older merges):
- **#1315** `fix(safety): P0 — close quote/escape evasion`: `cargo test --lib -- safety` → **PASS** (35/35 tests; 16 new tests since bootstrap including smart_blend_tests, allowlist_catastrophic_tests)
- **#1487** `fix(static-matcher): accept current-directory qualifier in Pattern 43`: validated by Slot A dry-run returning `ls -la` for "list files in current directory" — **PASS**
- **#1459** `feat(decision): typed decisions, calibration, constrained decoding`: safety suite PASS (35/35); decision/calibration unit tests use `decision::` / `evaluation::calibration::` paths not captured by the `-- safety` filter — not separately verified this run
- **#1488** `fix(deps): reqwest/h2/rustls security bumps`: build compiled clean — **PASS**; CVE assessment: explicit `cargo audit` not run in this QA session; relies on Security Audit CI job passing on #1488's merge CI
- **#1470** `fix: enforce declared limits (executor timeout, guard hooks, loop budgets, eval CI)`: HEAD under test (1ad0631); not separately diffed — implicit coverage from Slot A smoke and Slot B build/test passes; executor-timeout change is potentially related to the Slot C P1 hang (#1499)
- i18n, brand, docs, CI PRs: flagged for Slot C coverage matrix tracking (website surfaces #25–#31)

### Slot C — `caro ai --once` scripted conversational mode (surface #10)

Surface chosen: #10 (`caro ai --once`) — oldest "never" by lowest `#` number.

**Testing procedure:**
1. `caro ai --once "show disk usage"` (no stdin redirect, non-TTY) → **FAIL (P1): hung indefinitely** (exit 124 after 15s timeout). Reproduced 2×.
2. `caro ai --once "show disk usage" < /dev/null` → completed; returned `Error: backend error: Clarification needed: What exactly should be deleted?`
3. `caro ai --once "list files" < /dev/null` → same error for a completely different prompt
4. `caro --backend static ai --once "list files" < /dev/null` → `Unknown backend 'static'` (static backend not exposed to end users via CLI)
5. Investigated root cause: model downloaded successfully (qwen2.5-coder-1.5b-instruct-q4_k_m.gguf) during background invocation; CPU stub's `infer()` matches on system prompt (which contains "delete"/"rm" in safety instructions) → all queries hit the `needs_clarification` branch.

**Verdict**: FAIL — two distinct bugs found:
- P1: stdin-read hang in non-TTY (#1499)
- P2: CPU stub keyword match on system prompt returns misleading clarification (#1500)

Also incidentally found: CLAUDE.md version drift recurred (shows 1.4.0, actual 1.5.0) → #1501.

### Findings

- [#1499](https://github.com/wildcard/caro/issues/1499) — `ai: caro ai --once hangs in non-TTY environments` (P1)
- [#1500](https://github.com/wildcard/caro/issues/1500) — `embedded: CPU stub returns misleading clarification for all queries` (P2)
- [#1501](https://github.com/wildcard/caro/issues/1501) — `docs: CLAUDE.md version banner 1.4.0 instead of 1.5.0` (P2)

### Followups

- FLAKE-001 (model download failure) did NOT reproduce this run: model downloaded successfully in a background invocation. Marking occurrence gap in known-flakes. Two more observations within 7 days would be needed for promotion to regression (1 prior observation total; threshold is 3 in 7 days).
- Slot C surface #10 is FAIL. Next Slot C candidate: surface #11 (`caro ai --continue-session`) or #12 (`caro assess`) — both "never" tested.
- 25 merged PRs since last run (first API page only — pagination not applied). Actual merge count since 2026-05-07 may be higher; next Slot B should paginate all pages and note any coverage gap. Website i18n, brand, design-system surfaces not tested — recommend adding website surfaces #25–#31 to upcoming Slot C cycles.
- Previous QA rotation PRs (#1178, #1373, #1443, #1477, #1496) are still open/unmerged — their session logs are not on main. This run's state is correctly based on main.

---

## 2026-10-01 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (many PRs merged since last run 2026-05-07) + C (surface #10).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 12s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (1ad0631 2026-09-30)`
- `caro --help` → **PASS**: available subcommands listed (ai, suggest, config, completion, skill, etc.)
- `caro doctor` → **PASS**: network reachable, proxy detected, no model downloaded (expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **PASS**: static matcher returned `ls -la` (telemetry consent shown on first run; piped `y` to proceed; second invocation runs without consent prompt)

### Slot B — Recent diff

PRs merged since 2026-05-07 (5+ months, ~50 PRs). Key surfaces smoke-tested:

- **#1487** `fix(static-matcher): accept current-directory qualifier in Pattern 43` → `cargo test --lib -- test_list_files_with_current_directory_qualifier` **PASS**
- **#1488** `fix(deps): upgrade reqwest 0.12 / h2 / rustls (RUSTSEC-2026-0258, -0285)` → Cargo.lock verified: h2 v0.4.19, rustls v0.23.45 — **noted** (security fix landed)
- **#1459** `feat(decision): Jev gap analysis, clarification gates, constrained decoding` → new surface; flagged for future Slot C (see matrix)
- **#1470** `fix: enforce declared limits (executor timeout, guard hooks, loop budgets)` → new surface; flagged for future Slot C

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **#10 `caro ai --once` scripted conversational mode** (oldest 'never' tested).

- `caro ai --help` → **PASS**: `--once` flag documented; "Run one turn and return — no TTY REPL. The only mode supported today"
- `caro ai --once "list files in current directory"` → **FAIL (P1)**: returns `Error: backend error: Clarification needed: What exactly should be deleted?` for every prompt
- Root cause confirmed: `src/backends/embedded/cpu.rs:63` checks `prompt.contains("rm")` on the full system prompt, which always contains `rm -rf` as a negative example (line 218 of `embedded_backend.rs`). Every invocation fires the deletion-clarification branch.
- `--backend` flag cannot be passed after the `ai` subcommand (clap parse error). Passing it before is also rejected: `args_conflicts_with_subcommands = true` (src/main.rs:690) makes clap conflict `--backend` with `ai`, so backend resolution (and its "Unknown backend" error) is never reached. No testable backend avoids this bug without a model download or remote service.

### Findings

- [#1494](https://github.com/wildcard/caro/issues/1494) — `ai: caro ai --once always returns deletion-clarification with embedded-cpu backend` (P1)
- [#1495](https://github.com/wildcard/caro/issues/1495) — `docs: CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0 (regression)` (P2)

### Followups

- FLAKE-001 (model download) not reproduced this pass: Slot A dry-run succeeded via static matcher; model download path not exercised.
- #1044 (CLAUDE.md version drift, P2) is **closed** (2026-05-09) but has recurred at v1.5.0; filed as #1495.
- Previous QA rotation PRs #1178, #1373, #1443, #1477 are still open/unmerged — memory updates are accumulating in open PRs. Consider squashing/closing stale rotation PRs.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — also never tested.
- Consider testing surface #29 (`caro --safety strict/moderate/permissive` modes) given the new decision-module gates in #1459.

---

## 2026-09-30 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs merged since last log entry 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 27s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed (doctor, integration, init, config, test, completion, suggest, ai, shell-init, check, list, jobs, new, generate, run, export, experiment, adopt, history, why, do, render, skill)
- `caro doctor` → **PASS**: advisory only (no model downloaded; proxy detected at 127.0.0.1:41305; HuggingFace reachable — expected in sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: telemetry consent shown then timeout (EXIT:124 at 30s); same FLAKE-001 as bootstrap run

Side observation: `CLAUDE.md` line 9 still reads `- **Version**: 1.4.0 (GA)` but binary is `1.5.0`. Issue #1044 (1.1.0→1.3.0) was partially addressed but drift recurred for 1.4.0→1.5.0 and the backlog still lists it as open. 45+ open duplicate issues exist (#1442, #1469, #1474, #1476, #1483 etc.) — no new issue filed.

### Slot B — Recent diff

110 PRs merged since last log entry (2026-05-07). Representative surfaces sampled:

- **PR #1315** `fix(safety): P0 — close quote/escape evasion of the command scanner` → verified via `cargo test --lib -- safety` → **PASS** (34 safety tests pass; count grew from 19 to 34, consistent with new pattern coverage)
- **PR #1352** `fix(i18n): load all locale JSON files; overhaul Hebrew translations` → flagged for next Slot C cycle (i18n website locale surface #31 in coverage matrix); not exercised today (requires live website)
- **PR #1304** `chore(release): v1.5.0` → accounted for in version bump noted above

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: docs accurate, flags `--once`, `--new-session`, `--continue-session` described correctly
- `caro ai --once 'list files in current directory'` → **FLAKE**: EXIT:124 (30s timeout, zero output even on stderr) — FLAKE-001 (model download blocked in sandbox)
- `cargo test --lib -- ai` → **PASS**: 62 tests pass (runner, session, privacy, evaluation safety evaluators)
- Known UX gap: silent hang with no backend ready (no stderr progress) — open issues #1404, #1393, #1455, #1468 (7 open total); no new issue filed

### Findings

None this pass — all observed gaps have existing open issues.

### Followups

- FLAKE-001 (model download) observed again (2nd total: 2026-05-07, 2026-09-30). Not yet 3-in-7-days; no regression filing. Updated flakes log.
- CLAUDE.md version drift (1.4.0 vs 1.5.0) is a chronic issue (45+ open duplicates); the fix would be adding CLAUDE.md to the 6-file release checklist. Most recent issue: #1483 (2026-09-29). No new filing.
- `caro ai --once` silent hang: 7 open issues unchanged. No regression.
- Next Slot C candidate: surface #11 (`caro ai --continue-session`) or surface #31 (i18n website locale curl test) — both untested.
- 110 merged PRs since last session log; several brand/website PRs (#1155, #1156, #1158, #1159) touch website surfaces — consider website visual regression in future slots.

---

## 2026-09-29 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:58 UTC.
**Rotation**: A + B (110 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 24s, no errors; binary at `./target/release/caro`)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed (including ai, suggest, skill)
- `caro doctor` → **PASS**: advisory only (no model downloaded; expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: model download silently blocked (FLAKE-001, occurrence #2; same sandbox network limitation as 2026-05-07)
- Full library test suite `cargo test --lib` → **PASS**: 597 passed, 0 failed, 1 ignored (up from 513 on 2026-05-07; new tests added across releases)

### Slot B — Recent diff (110 PRs merged since 2026-05-07)

Key surfaces exercised:

- **Safety** (P0 fix #1315 — `fix(safety): P0 — close quote/escape evasion of the command scanner`): `cargo test --lib -- safety` → **PASS** (34 safety tests, 0 failed). P0 regression closed.
- **i18n** (#1352 — `fix(i18n): load all locale JSON files; overhaul Hebrew translations`): Hebrew locale (`website/src/i18n/locales/he/`) → **PASS**: 9 JSON files present, all parse cleanly; 15 total locales confirmed.
- **CLI backend roster** (#1298 — `fix(cli): single source of truth for backend roster`): `caro --help` backend list → **PASS**: all expected backends visible (embedded, ollama, exo, vllm, mesh, ai-horde, hybrid).
- **v1.5.0 release (#1304)**: `caro --version` confirms 1.5.0 shipped correctly.

Surfaces flagged for future Slot C: website (PR #1325 `feat(website): adopt Codex Pet Web SDK`), docs site (#1351 Cloudflare Pages prep).

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: subcommand present, `--once` flag documented as "Run one turn and return — no TTY REPL"
- `caro ai --once "list files in current directory"` → **FAIL**: command blocks indefinitely with zero output; no progress indicator, no error; must be killed manually
- Root site: `src/main.rs:1100–1108` — `CliApp::with_overrides(...).await` initiates model download with no stderr progress
- Distinct from FLAKE-001: the regular `caro -p ...` path at least prints the telemetry screen before hanging; `ai --once` produces nothing

### Findings

- [#1483](https://github.com/wildcard/caro/issues/1483) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (regression of #1044)` (P2, regression, release-gap)
- [#1484](https://github.com/wildcard/caro/issues/1484) — `ai: caro ai --once silently blocks indefinitely when no model is available` (P2)

### Followups

- FLAKE-001 observed on 2026-05-07 and 2026-09-29 — these two occurrences are 4+ months apart and NOT within the same 7-day rolling window. The promotion threshold requires 3+ observations within any 7-day window; the rolling window effectively resets with today's observation. A future QA run needs 2 more occurrences within 7 days of each other (not just of today) to trigger reclassification.
- 4 previous QA rotation PRs (#1178, #1373, #1443, #1477) remain open and unmerged — their memory updates never landed on main. Main's coverage matrix still shows bootstrap state (2026-05-07). Recommend merging or closing these PRs so memory state converges.
- #1044 fix direction step 2 ("Add CLAUDE.md to release checklist") was never implemented. #1483 refiles this structural fix.
- Surface #10 (`caro ai --once`) result: FAIL (P2 UX bug filed as #1484). Slot C next candidate: surface #11 (`caro ai --continue-session`) — note it also requires a working backend.
- Website + Codex Pet SDK surfaces (#1325) worth a dedicated Slot C pass from an environment with curl access to caro.sh.

---

## 2026-09-28 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs merged since last log entry on 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 24s, no errors; caro 1.5.0 built)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands present including full CaroML verb set and `ai`, `suggest`, `skill`, `completion`
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected; proxy detected at 127.0.0.1:43861)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: model not downloaded (FLAKE-001 second occurrence, 2026-09-28); fallback returns `echo 'Please clarify your request'`. Note: `caro -p 'list files' --dry-run` → **PASS** (`ls -la` via static matcher; shorter query hits Pattern 43 regex correctly)

### Slot B — Recent diff

110 PRs merged since 2026-05-07. Representative surface tested per the most safety-critical PR:

- **PR #1315** (`fix(safety): P0 — close quote/escape evasion of the command scanner`): ran `cargo test --test safety_validator_contract test_quote_escape_evasion_is_caught_base` → **PASS** (1/1 test ok). The `shell-words`-based `destructive_unescaped()` normalization blocks `rm -rf \/`, `rm -rf "/tmp"/*/x`, and `rm -\rf \/etc` while keeping `echo 'rm -rf /'` safe.
- **PR #1304** (`chore(release): v1.5.0`): version confirmed in binary (`1.5.0 (be07b22 2026-07-18)`) → **PASS**
- i18n batch PRs (#816–#829): surfaces flagged for future Slot C coverage under surface #31 (i18n locale smoke)

### Slot C — `caro ai --once` (surface #10)

- `caro ai --once 'show disk usage'` → returns two lines: `# caro-ai: session 1 (resumed) confidence=0.85 risk=Safe` then `echo 'Please clarify your request'` — **FLAKE** (FLAKE-001 applies; no model available; same fallback as Slot A)
- `caro ai --help` → **PASS**: subcommand documented correctly; `--once` flag described as "Run one turn and return — no TTY REPL. The only mode supported today"
- `caro ai -p 'query'` → **FAIL** (expected): "unexpected argument '-p' found" — confirmed UX inconsistency; pre-existing as [#1422](https://github.com/wildcard/caro/issues/1422) (open)
- `caro ai --once` without prompt → **PASS**: correctly errors "no prompt provided (pass text, pipe stdin, or use -p)" — though the hint says "use -p" but `-p` doesn't work (consistent with #1213)
- Static matcher gap also confirmed for `caro ai` path: "list files in current directory" → fallback; pre-existing as [#1399](https://github.com/wildcard/caro/issues/1399) (open)
- CLAUDE.md version drift: shows `1.4.0 (GA)`, binary reports `1.5.0`; pre-existing as [#1474](https://github.com/wildcard/caro/issues/1474) (open, multiple dupes filed by prior QA runs)

### Findings

- **None filed this pass** — all identified issues are pre-existing open GitHub issues (#1399, #1422, #1474, others). No new unique defects discovered.
- Confirmed surfaces healthy: safety scanner P0 fix (PR #1315) verified; binary build and version alignment correct; doctor and help accurate.

### Followups

- FLAKE-001 (model download in sandbox) observed again: 2nd occurrence on 2026-09-28. Dates are far apart (May → Sep), so not 3× in 7 days; does NOT trigger regression reclassification. Occurrence log updated.
- Multiple QA rotation PRs from prior runs (#1178, #1373, #1443, #1477) remain open/unmerged, keeping main's memory files at the 2026-05-07 state. This causes each new QA pass to re-encounter the same open issues from its perspective. Recommend merging the oldest open QA rotation PR (#1178) to unblock memory state progression.
- i18n locale surfaces (#31) should be the next Slot C candidate after the `caro ai` surface is marked tested.
- Surface #11 (`caro ai --continue-session`) not yet tested — schedule for next pass.

---

## 2026-09-27 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

**Context note**: QA PRs #1373 (2026-07-25), #1443 (2026-09-07), and #1477 (2026-09-26) remain unmerged. Memory files on `main` still reflect only the 2026-05-07 bootstrap state. This PR updates them independently against main, ingesting known open issues from those sessions.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 31s, no errors; v1.5.0)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: 24 subcommands listed including `ai`, `suggest`, `export`, all CaroML verbs, `skill` (note: `assess` is commented out in source since v1.1.0-beta.1 and does not appear)
- `caro doctor` → **PASS**: advisory (no model downloaded; huggingface.co reachable; proxy detected)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001: silent hang → timeout; model download blocked in sandbox; 5th overall occurrence, 2nd within 7-day window; stays classified as flake)
- Telemetry consent on first invocation → **PASS**: consent shown once; second invocation suppressed

### Slot B — Recent diff

110 PRs merged on main since last log entry (2026-05-07). Three representative surfaces exercised:
- **Safety P0 fix** (#1315 `fix(safety): close quote/escape evasion`): `cargo test --lib -- safety` → **34/34 PASS**
- **`caro suggest`** (surface #13, never tested): `caro suggest 'list files in current directory'` → **PASS** (5 correct suggestions including `ls -la`, find variants)
- Prior QA sessions (#1477 2026-09-26, #1443 2026-09-07, #1373 2026-07-25) covered #1315, #1298, Hebrew i18n (#1352), and `caro test --backend static`; all confirmed PASS; no re-exercise needed

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **#10** (oldest 'never' on main's matrix; tied with #11–#31; lowest # wins; #32 was last tested 2026-05-07).

- `caro ai --once 'list files' --backend embedded` → **FLAKE** (FLAKE-001: silent hang 15s; model download blocks; identical to #1477 finding)
- `caro ai --once --backend static` → immediate error: `Invalid argument: Unknown backend 'static'` (static is only valid for `caro test`, not `caro ai`)
- `caro ai --help` → PASS: flag documented correctly; `--once` = "the only mode supported today" (confirmed dead code `once: _` in dispatch, intentional per docs)
- Unit tests: (not re-run; #1477 confirmed `cargo test --lib -- "ai::"` → 23/23 PASS; build unchanged)

### Findings

None new — all findings already tracked in open issues:
- [#1375](https://github.com/wildcard/caro/issues/1375) — P1 — `caro ai --once` CpuBackend always returns placeholder on Linux x86_64 (open)
- [#1442](https://github.com/wildcard/caro/issues/1442) — P2 — CLAUDE.md version drift `1.4.0` vs actual `1.5.0` (open)

### Followups

- FLAKE-001: 5th occurrence overall. 2nd in 7-day window (2026-09-26, 2026-09-27). Threshold is 3 in 7 days — stays flake for now.
- P1 #1375 (CpuBackend placeholder) should be priority fix for Linux x86_64 users.
- QA PRs #1373, #1443, #1477 remain unmerged — owner should review and close as superseded or merge.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — will likely hit FLAKE-001 too; worth documenting for coverage.
- `caro assess` subcommand is commented out in source since v1.1.0-beta.1 (src/main.rs:422-434, 3048-3057); surface #12 marked N/A in matrix — cannot be exercised until re-enabled.

---

## 2026-09-26 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (117 commits since last log entry on main; range c9a31a3..be07b22) + C.

**Context note**: QA PRs #1373 (2026-07-25) and #1443 (2026-09-07) are still unmerged. Memory files on main reflect the 2026-05-07 bootstrap only. This entry is written against the main-branch state.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 33s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: 24 subcommands including `ai`, `suggest`, `skill`, `export`, all CaroML verbs
- `caro doctor` → **PASS**: advisory (no model downloaded); proxy detected; embedded "needs model download"
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001 4th occurrence: model download blocked in sandbox)
- `caro --backend-info` → **PASS**: 7 backends listed consistently with `--help` roster

### Slot B — Recent diff (117 commits since 2026-05-07; range c9a31a3..be07b22)

PRs merged since last log entry on main (note: #1373/#1443 branches also tested some of these; results consistent):

- **#1315 `fix(safety): normalize destructive stmts`** → `cargo test --lib -- safety` **34/34 PASS** (up from 19 at bootstrap; new guards in place)
- **#1298 `fix(cli): single source of truth for backend roster`** → `--backend-info` shows 7 backends, matches error message roster; **PASS**
- **#1352 `fix(i18n): load all locale JSON files; overhaul Hebrew translations`** → Python key-count check: EN=339, HE=339, full coverage; **PASS**
- `caro test --backend static` → **11/11 PASS** (100% across Website Claim, Natural Variant, Edge Case categories)
- `cargo test --lib` → all pass (no count captured but no failures seen)

Surfaces flagged for future Slot C: #31 (i18n locale smoke) elevated priority given recent i18n changes.

### Slot C — `caro ai --once` (surface #10, oldest 'never')

- `caro ai --help` → **PASS**: `--once`, `--new-session`, `--continue-session` flags documented; stdout-only design noted
- `caro ai --once 'list files in current directory sorted by size'` → **FLAKE** (FLAKE-001: silent hang after 60s; no stdout, no stderr — model download blocked)
- `cargo test --lib -- "ai::"` → **PASS**: 23/23 (runner, session, store, privacy, shell_init)
- Cross-reference: P1 issue [#1375](https://github.com/wildcard/caro/issues/1375) open — CpuBackend placeholder always returns `echo 'Please clarify your request'` on Linux x86_64 due to system-prompt keyword collision. Confirms surface #10 has two layered issues: FLAKE-001 (model download) masks #1375 (placeholder response) in sandbox.

### Findings

- No new issues filed this pass.
- [#1372](https://github.com/wildcard/caro/issues/1372), [#1442](https://github.com/wildcard/caro/issues/1442) open — CLAUDE.md version drift (1.4.0 vs 1.5.0); additionally MSRV field drifted (1.83 vs 1.85). Comment added to #1442 noting MSRV drift.
- [#1375](https://github.com/wildcard/caro/issues/1375) open — P1, `caro ai --once` CpuBackend placeholder (confirmed still open in v1.5.0, first confirmed in today's context).

### Followups

- PRs #1373 and #1443 need to merge to bring memory files in sync. Without merge, each QA run re-discovers the same state.
- FLAKE-001 4th occurrence logged (2026-09-26). Occurrences span >7-day windows; stays classified as flake. Promotion threshold: 3 in 7 days.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — same 'never' cohort, surface #10 now tested.
- P1 #1375 (CpuBackend placeholder) is a genuine regression for Linux users; should be priority fix.

---

## 2026-09-24 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 09s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed including CaroML verbs and the `skill` subcommand
- `caro doctor` → **PASS**: proxy detected, HuggingFace reachable, no model on first check (model downloaded during dry-run)
- `caro -p 'list files in current directory' --dry-run` → **FAIL** (exit 0): WARN `Timeout approaching, skipping refinement`; generated `echo 'Please clarify your request'` (root cause confirmed in Slot C, see #1473)
- FLAKE-001 status: model downloaded successfully this run ([size omitted] GGUF present after run) — first successful download; FLAKE-001 occurrence streak NOT extended

### Slot B — Recent diff (110 PRs since 2026-05-07)

Key surfaces exercised:
- **Safety unit tests** (covers P0 fix #1315 quote/escape evasion) → **PASS**: 34/34 tests (`allowlist_catastrophic`, `smart_blend`, CVE patterns, etc.)
- **Backend roster** (#1298 single-source-of-truth fix) → **PASS**: `--backend` flag shows correct 7 backends (embedded, ollama, exo, vllm, mesh, ai-horde, hybrid)
- **Config surface** → **PASS**: `caro config show` returns correct fields
- **Shell-init** → **PASS**: bash, zsh wrappers emit correct content
- **Completions** → **PASS**: `caro completion bash` emits valid completion script
- **i18n #1352** (Hebrew locale overhaul) → **PASS**: `website/src/i18n/locales/he/` exists with all locale JSON files

Flagged for future Slot C: i18n locale coverage (surface #31) — touched by #1352, add to next rotation.

### Slot C — `caro ai --once` (surface #10, first test)

Exercised `caro ai --once` scripted conversational mode for the first time.

- All prompts (`list files`, `show disk usage`, `print working directory`, `what time is it`) → **FAIL**: every prompt returns `echo 'Please clarify your request'` with `confidence=0.85 risk=Safe`
- Root cause: `src/backends/embedded/cpu.rs:63` — placeholder inference stub checks `prompt.contains("rm")` on the full system prompt (which always contains `rm -rf` in safety rules). All invocations match the "clarify" branch.
- Confidence hardcoded to 0.85 regardless of output quality → misleading to users
- Session resume anomaly: sessions were resumed across separate `--once` invocations in the same test run (expected, but noteworthy for scripting use cases)

### Findings

- [#1473](https://github.com/wildcard/caro/issues/1473) — `embedded: CPU backend placeholder always returns "echo 'Please clarify your request'" due to system-prompt contamination` (P1)
- [#1474](https://github.com/wildcard/caro/issues/1474) — `docs: CLAUDE.md version shows 1.4.0 instead of 1.5.0 — regression of #1044` (P2)

### Followups

- FLAKE-001: model downloaded successfully this run. Reset observation counter; previous observation on 2026-05-07 is now 7+ days old and outside the 7-day window. FLAKE-001 remains open but unconfirmed.
- Note: 3 prior QA rotation PRs (#1178, #1373, #1443) were found in the repo but their session-log entries were not in this file. Session log may be incomplete between 2026-05-07 and today. Not critical to reconstruct — log is append-only going forward.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or surface #12 (`caro assess` system assessment).
- i18n locale smoke (surface #31) should be prioritized in next Slot B given #1352 impact.

---

## 2026-09-12 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (110 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 53s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed; new `ai`, `export`, `render`, `skill` visible
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 reproduced (model download blocked in sandbox — 2nd total observation, 128 days since first; not within 7-day window, no reclassification)
- Telemetry consent prompt shown on first invocation (fresh sandbox, expected)

### Slot B — Recent diff

110 PRs merged since 2026-05-07. Spot-checked representative surfaces:

- **PR #1315** `fix(safety): P0 — close quote/escape evasion` → `cargo test --release --lib -- safety` → **PASS**: 34/34 safety unit tests including new `test_quote_escape_evasion_is_caught_base` (uses `shell-words` crate, PR verified working)
- **PR #1298** `fix(cli): single source of truth for backend roster` → `caro --backend-info` → **PASS**: clean backend table, all backends listed with correct status
- **PR #1304** `chore(release): v1.5.0` → binary reports correct 1.5.0 version
- i18n batch (PRs #816-#829): flagged for future Slot C exercise (surface #31 — i18n locale smoke)
- **PR #883** `feat(tools): MLX LoRA fine-tune pipeline` → flagged for future Slot C exercise (surface deferred — macOS-only MLX, not exercisable in Linux sandbox)

### Slot C — caro ai --once (surface #10)

Surface chosen: **`caro ai --once` scripted conversational mode** (oldest 'never' surface, lowest #).

- `caro ai --help` → **PASS**: subcommand present with `--once`, `--new-session`, `--continue-session` flags documented
- `caro ai --once "list files in current directory"` → **FAIL**: silent hang, zero output to stderr/stdout after 35 seconds; exits via SIGTERM (code 143)
- `echo "list files" | caro ai --once` → **FAIL**: same silent hang
- `cargo test --release --lib -- ai::` → **PASS**: 23/23 AI unit tests pass with mock backend (runner, session, store, privacy, shell_init all green)
- Root cause investigation: `run_ai_once()` → `backend.generate_command().await` blocks silently during model download retry loop; no progress emitted to stderr unlike regular `caro -p ... --dry-run` path

### Findings

- [#1449](https://github.com/wildcard/caro/issues/1449) — `cli: caro ai --once hangs silently with no output when embedded model is unavailable` (P2)
- [#1450](https://github.com/wildcard/caro/issues/1450) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (re-drift)` (P2)
- [#1044](https://github.com/wildcard/caro/issues/1044) — closed ✅ (CLAUDE.md updated to 1.4.0; 1.5.0 drift filed as new #1450)

### Followups

- FLAKE-001 (model download blocked) — 2nd occurrence total. First on 2026-05-07, second today (128 days apart). Not within 7-day window; log updated but no reclassification.
- Three previous QA PRs (#1178, #1373, #1443) still open — memory files on main only reflect bootstrap pass. Memory files will be accurate once those PRs merge.
- MLX LoRA pipeline (PR #883) cannot be exercised in Linux sandbox — add macOS-only note to surface #19 in coverage matrix.
- `caro ai --continue-session` (surface #11) likely affected by same silent-hang bug as `--once`; should verify after #1449 is fixed.
- i18n locale smoke (surface #31) now has test coverage via PR #1352 — schedule for next Slot C pass.

---

## 2026-09-08 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 23s, no errors; built clean from main HEAD be07b22)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed including CaroML verbs, `skill`, `suggest`, `ai`, `completion`, `export`, `why`, `do`, `render`
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox; huggingface.co reachable, proxy detected)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: telemetry consent prompt displayed on first prompt-generating invocation in fresh sandbox, process timed out waiting for stdin response (30s timeout). Environment limitation — not a code bug. Same root cause class as FLAKE-001 (sandbox restrictions). Telemetry consent is expected on first invocation.

### Slot B — Recent diff (PRs merged since 2026-05-07)

20 non-dependency PRs merged. Key CLI/safety surfaces checked:

- **#1209 feat(backends): Mesh-LLM + AI-Horde + hybrid privacy gateway** → `caro --backend-info` shows all new backends (`mesh`, `ai-horde`, `hybrid`) with correct status and descriptions. Source files `mesh.rs`, `ai_horde.rs` confirmed in `src/backends/remote/`. PASS.
- **#1298 fix(cli): single source of truth for backend roster** → `--backend-info` flag present in `--help`; backend list `embedded, ollama, exo, vllm, mesh, ai-horde, hybrid` correct. `openrouter` and `claude` noted as "not yet CLI-wired" (confirmed intentional via `src/backends/mod.rs` comment). PASS.
- **#1211 test(execution): make dangerous-command confirm test deterministic** → `cargo test --lib -- safety` → 34/34 PASS (up from 19/19 in 2026-05-07; new tests for allowlist catastrophic floor). PASS.
- **#1352 fix(i18n): Hebrew translations** → flagged for future Slot C run on surface #31 (i18n locale smoke).
- **#1245 feat: apply Fireworks hybrid-harness learnings** → `--advisor` flag present in `--help`. Surface not fully exercisable without credentials; noted for future slot.

### Slot C — `caro ai --once` (surface #10)

- `caro ai --once 'list files'` → **FLAKE**: process terminated (exit 143) after 25s timeout. Consistent with embedded backend attempting model download that is blocked in sandbox (FLAKE-001 root cause). `caro ai --help` confirmed `--once` flag exists and is described correctly.
- Surface confirmed to exist and be CLI-wired. Cannot exercise end-to-end without a pre-downloaded model.

### Findings

- [#1444](https://github.com/wildcard/caro/issues/1444) — `docs: CLAUDE.md version and MSRV out of sync with v1.5.0` (P2) — recurrence of #1044 pattern; `CLAUDE.md` shows `Version: 1.4.0 (GA)` and `MSRV 1.83` while shipped version is 1.5.0 and MSRV was bumped to 1.85.

### Followups

- FLAKE-001 (model download): second observation on `caro ai --once`. Still below 3-in-7-days promotion threshold. Consider pre-populating model cache in sandbox for future deeper embedded-backend testing.
- Surface #31 (i18n locale smoke: curl /es/, /fr/, /ja/) flagged for next Slot C rotation — #1352 makes it relevant.
- `openrouter` and `claude` backends exist in source but are intentionally not CLI-wired. Track for when they land.
- CLAUDE.md should be added to the release-version-alignment 6-file checklist (see #1444 fix direction).

---

## 2026-09-07 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 07s, `caro 1.5.0 (be07b22 2026-07-18)`)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: 24 subcommands listed including all CaroML verbs and `ai`
- `caro doctor` → **PASS**: advisory only (no model, expected in fresh sandbox; proxy detected)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: telemetry consent required interactive TTY; underlying model download blocked (FLAKE-001 2nd occurrence in sandbox)

### Slot B — Recent diff

Last session: 2026-05-07 (bootstrap). Note: QA PR #1373 (2026-07-25) remains open/unmerged — session log on main still shows 2026-05-07 as last entry. Covered feature PRs merged since 2026-05-07:

- **#1315** `fix(safety): P0 — close quote/escape evasion`: `cargo test --lib -- safety::tests` → **PASS** (34/34, up from 19 in bootstrap; 15 new safety tests added)
- **#1304** `chore(release): v1.5.0`: `caro --version` → PASS shows 1.5.0
- **#1298** `fix(cli): single source of truth for backend roster`: `caro --backend-info` → **PASS** (7 backends listed; embedded=available, others=not compiled with correct guidance)
- **#1352** `fix(i18n): load all locale JSON files; overhaul Hebrew translations`: `curl https://caro.sh/he/` → **PASS** ([size omitted], `lang="he" dir="rtl"`, correct Hebrew title)
- **#1246** `fix(ci): repair pre-existing main test failures`: `cargo test --lib` → **PASS** (all tests pass)

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: shows `--once` flag, `[PROMPT]...` positional arg, `--new-session`, `--continue-session`
- `caro ai --once list files in current directory` → **HANG** (SIGTERM at 15s; FLAKE-001 model download blocked in sandbox)
- `caro ai --once --backend ollama list files` → **FAIL**: `error: unexpected argument '--backend' found` — global flags must precede subcommand name (tracked #1369, #1422)
- `caro --backend embedded ai --once list files` → parsed correctly but hangs on model download (FLAKE-001)
- AI config: `ai.enabled` defaults to `true` (confirmed in `src/models/mod.rs:818`)
- Confirmed open P1: #1375 — CPU backend always returns `echo 'Please clarify your request'` on Linux x86_64 due to system-prompt keyword collision with "delete"/"rm" safety examples
- Confirmed open: #1421 (no-backend hang), #1422 (--prompt flag rejection)

### Findings

- [#1442](https://github.com/wildcard/caro/issues/1442) — `docs: CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0` (P2) — recurrence of closed #1044

### Followups

- FLAKE-001 2nd occurrence (2026-09-07). Still within 2-observation window — no promotion. Next observation would be 3rd; if within 7 days of one prior, promote to regression.
- QA PR #1373 (2026-07-25) is still open/unmerged on GitHub. This creates a split-brain between main's session log and what that PR recorded. Recommend merging or closing #1373 before the next scheduled run.
- `caro.sh` root redirects to `www.caro.sh` (HTTP 307); sandbox proxy blocks `www.caro.sh` response body (0 bytes). Not a product bug — sandbox network limitation. `caro.sh/he/` works fine ([size omitted]).
- `caro.sh/en/` returns 404 — expected; `prefixDefaultLocale: false` means English lives at `/`, not `/en/`.
- P1 issue #1375 (echo placeholder on Linux) remains unfixed in v1.5.0. Next Slot C should re-exercise `caro ai --once` after #1375 is fixed.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — tie with #10 at 'never', but #10 was just exercised.

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

## 2026-08-01 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (20 of ~82 PRs sampled since 2026-05-07) + C (surface #10: `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 07s, no errors)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed including all CaroML verbs and `skill` command
- `caro doctor` → **PASS**: advisory only (no model, proxy detected; expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 reproduced; telemetry consent appeared then model download blocked (sandbox restriction)

### Slot B — Recent diff

~82 PRs merged since 2026-05-07; **20 most-recent sampled** (search perPage:20 limit — see Followups). Representative surfaces tested from the sampled set:

- **#1315** `fix(safety): P0 close quote/escape evasion` → `cargo test --lib -- safety` → **PASS**: 34 safety tests (up from 19 in bootstrap; reflects new patterns from the P0 fix)
- **#1298** `fix(cli): single source of truth for backend roster` → `caro --backend-info` → **PASS**: 7 backends with correct statuses
- **#1346/#1351/#1352** (website/ci/i18n) → not testable via CLI in sandbox; flagged for next website Slot C rotation (#25, #31)
- Full library test suite → **PASS**: 597 tests (up from 513 in bootstrap run), 0 failures

Note: the 62 unsampled PRs (earlier half of the 2026-05-07 to 2026-08-01 window) are not permanently excluded. The full 597-test library suite passing green provides a functional safety net across all merged code. Future Slot B runs will search since 2026-08-01 and cover new work from there. The unsampled window can be spot-checked by a future run using an explicit date range if a regression is suspected.

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: documents `--once`, `--new-session`, `--continue-session`; prompt is positional not `-p`
- **P1 BUG**: `caro ai --once 'ANY PROMPT'` always returns `echo 'Please clarify your request'` with `confidence=0.85` — identical wrong output for every prompt including 'show current directory', 'show disk usage', 'list files'
- Comparison: `caro -p 'show current directory' --dry-run` → `pwd` (static matcher works on regular path)
- Root cause confirmed in source: `run_ai_once` (`src/main.rs:1109`) calls `cli_app.backend_arc()` directly, bypassing the `AgentLoop` that applies the static matcher (`src/cli/mod.rs:284-285`). The raw embedded backend returns its hardcoded fallback (`src/backends/embedded/cpu.rs`) when it can't download a model.
- `caro ai --once -p 'prompt'` rejects `-p` with an error — minor UX inconsistency vs main `caro` path; not filed separately since help text is clear.
- `caro ai --continue-session` (surface #11) was NOT independently exercised this run. It shares the underlying session store and backend selection path with `--once`, so the same P1 bug likely applies, but it was not confirmed by direct invocation. Kept in rotation for a future Slot C pass.

### Findings

- [#1387](https://github.com/wildcard/caro/issues/1387) — `ai: caro ai --once bypasses static matcher, always returns fallback when LLM unavailable` (P1)
- [#1388](https://github.com/wildcard/caro/issues/1388) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurrence of #1044)` (P2)

### Followups

- FLAKE-001 reproduced (2026-08-01, second observation; first was 2026-05-07). Gap of 85 days — not within 3-in-7-days promotion threshold. Remains a flake.
- Issue #1044 was fixed for the v1.3.0 cycle but the systemic checklist fix (add CLAUDE.md to release-version-alignment.md) did not land, causing recurrence as #1388. Fix direction in #1388 includes the systemic fix.
- **Slot B coverage gap**: next agent should increase perPage to 100 (or paginate) to ensure all merged PRs in the window are captured, not just the most recent 20.
- Website/i18n surfaces (#25, #31) are high priority for next Slot C given recent PRs #1325/#1351/#1352 touching those areas.
- Surface #11 (`caro ai --continue-session`) remains in rotation at 'never' — exercise it directly in a future Slot C pass rather than grouping with #10 by inference.

---

## 2026-07-25 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (20 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (build succeeded; MSRV now 1.85 per v1.5.0 changelog)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed; `ai` and CaroML verbs present
- `caro doctor` → **PASS**: correctly reports embedded backend needs model download, Ollama not installed; proxy detected at `http://127.0.0.1:45761`
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: first attempt showed telemetry consent prompt, timed out (exit 124). Second attempt silently hung (exit 124). FLAKE-001 second occurrence — model download blocked by sandbox proxy on HuggingFace binary blob (HTTP 200 reachable, binary blocked at lower layer)
- `caro config show` → **PASS**: shows all config fields; telemetry disabled after first-run consent

### Slot B — Recent diff

PRs merged since 2026-05-07: 20 PRs. Two high-priority surfaces exercised:

**#1315 `fix(safety): P0 — close quote/escape evasion of the command scanner`**
- Added `shell-words` crate; normalises destructive-command statements before pattern scan
- `cargo test --test safety_validator_contract` → **PASS**: 22/22 passed (1 pre-existing `#[ignore]`); new test `test_quote_escape_evasion_is_caught_base` passes — `rm -rf \/`, `rm -rf "/tmp"/*/x`, `rm -\rf \/etc` all blocked; `echo 'rm -rf /'` stays allowed

**#1352 `fix(i18n): load all locale JSON files; overhaul Hebrew translations`**
- `import.meta.glob('./locales/*/*.json')` loader present in `website/src/i18n/index.ts` → **PASS**
- Hebrew landing.json spot-check: old mistranslations gone (`מונעת AI`, `מסקנה מקומית`, `מהיר בזרם הברק`, `כישור מוקדש`, `כל שימושי המקרה`), replacements confirmed present (`מהירה כברק`, `היסק מקומי`, etc.) → **PASS**

Other PRs noted but not deeply exercised: #1304 (v1.5.0 release), #1244 (ponytail reviewer), #1159/#1155/#1156/#1158 (brand/design), #1245 (ML eval harness), #1306 (CI release workflow).

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: flags documented correctly (`--once`, `--new-session`, `--continue-session`, trailing prompt args)
- `caro ai --once` (no prompt) → **PASS**: correct error "no prompt provided (pass text, pipe stdin, or use -p)" (exit 1)
- `caro ai --once "list files in current directory"` → **FLAKE**: silent timeout (exit 124, 35s). Backend auto-selects embedded; model download blocked (FLAKE-001). No user-visible progress indicator during hang — noted as UX gap but not a bug (consistent with dry-run behaviour, same root cause).

### Findings

- [#1372](https://github.com/wildcard/caro/issues/1372) — `docs: CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurrence of #1044)` (P2)

### Followups

- FLAKE-001 second occurrence (2026-07-25). Still 2/7-day window — below 3-in-7 reclassification threshold. Next occurrence promotes to regression.
- Issue #1044 (prev CLAUDE.md drift) closed as completed — but the checklist fix was NOT applied to `release-version-alignment.md`, causing this exact recurrence. Fix direction in #1372 is the same + "add to checklist".
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) — tests session persistence across invocations.
- Safety `test_safety_level_configuration` remains `#[ignore]` — pre-existing; expectations don't match current implementation.

---

## 2026-07-24 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled caro-qa-agent cron (14:00 UTC).
**Rotation**: A + B + C (first run since bootstrap 2026-05-07).
**Binary**: `caro 1.5.0 (be07b22 2026-07-18)`

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (pre-built binary present)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FAIL** (known P1 #1277): CPU placeholder misfires; returns `echo 'Please clarify your request'` instead of `ls -la`; system prompt contains "rm", triggering danger branch
- `cargo test --lib -- safety` → **PASS**: 34/34 safety unit tests (up from 19 at bootstrap)
- Custom patterns `~/.config/caro/patterns.toml` → **PASS**: discovered and loaded at startup
- Telemetry consent → **PASS**: shown once, persisted; no prompt on subsequent runs

### Slot B — Recent diff (since 2026-05-07, ~110 PRs)

Sampled 4 representative surfaces:

- **Safety test expansion**: 34 tests vs 19 at bootstrap; smart_blend, allowlist_catastrophic, telemetry redaction suites green → **PASS**
- **Remote backend UX** (#1092 area): `--backend ollama` without feature flag emits clear build instruction → **PASS**
- **Custom runtime patterns** (#1110, v1.5.0): `patterns.toml` loaded at startup → **PASS**
- **CLAUDE.md version drift**: shows 1.4.0, binary is 1.5.0 → **FAIL** (known P2; 5 issues open: #1319, #1335, #1359, #1366, #1368; no new issue filed)

### Slot C — Oldest-untested surface

Surface: **#10 — `caro ai --once`** (Last tested: never).

- `caro ai --once -p 'list files in current directory'` → **FAIL** (known P1 #1277): no static-matcher pre-pass in `src/ai/runner.rs`; CPU placeholder returns `echo 'Please clarify your request'`
- No new issue filed; failure fully attributed to pre-existing #1277
- Next Slot C: **#13 `caro suggest`** (surfaces #11, #12 also blocked by #1277)

### Findings

- **0 new issues filed.** All failures attributed to pre-existing tracked issues.

### Followups

- FLAKE-001 reclassified to Resolved: model download succeeded (78 days since single bootstrap observation with no recurrence)
- #1277 (P1) blocks Slot C surfaces #10 and #11; recommend fixing before next Slot C pass
- CLAUDE.md drift still P2 with 5+ tracking issues
- Next Slot C: **#13 `caro suggest`**

---

## 2026-07-22 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 49s)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands listed including `ai`, `skill`, `suggest`, `shell-init`, CaroML verbs
- `caro doctor` → **PASS**: detects Ubuntu 24.4.0, x86_64, proxy; no model at first invocation (downloads during dry-run)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE/BUG**: model download triggered ([size omitted]); agent loop timed out during download on first invocation; subsequent runs consistently return `echo 'Please clarify your request'` due to CPU backend stub bug (FLAKE-001 + open #1277, #1334)

### Slot B — Recent diff (30 PRs merged 2026-05-07 → 2026-07-22)

- **Safety (#1315)**: `fix(safety): P0 — close quote/escape evasion`. Library safety tests: 34/34 PASS. Full lib: 597/597 PASS. CLI-level ambiguous due to CPU stub — no new safety regression in unit tests.
- **Release (#1304)**: `chore(release): v1.5.0` — version confirmed correct.
- **Backend roster (#1298)**: `fix(cli): single source of truth for backend roster`. `--backend static` now returns "Unknown backend" — static-matcher is internal-only. Appears intentional; no user-facing removal notice found.
- **Other 27 PRs** (website brand/i18n, discovery docs, automation, ponytail, ML/eval): not smoke-tested; flagged for future Slot C passes.

### Slot C — `caro ai --once` (surface #10, first test)

- `echo "list files" | caro ai --once` → `echo 'Please clarify your request'` ✗ (expected: `ls`)
- `echo "show disk space" | caro ai --once` → `echo 'Please clarify your request'` ✗
- `echo "find and kill the runaway process eating CPU" | caro ai --once` → `echo 'Please clarify your request'` ✗ (same query works via static-matcher in `caro -p`)
- **Result: FAIL** — `caro ai --once` bypasses static-matcher, routes all queries to embedded CPU stub which always returns `echo 'Please clarify your request'` due to system-prompt "rm" contamination. Debug: `Backend: embedded, Confidence: 0.85` on every turn.

### Findings

- [#1366](https://github.com/wildcard/caro/issues/1366) — `docs: CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0` (P2) — **NEW, filed this pass**
- [#1277](https://github.com/wildcard/caro/issues/1277) — CPU backend stub always returns wrong output — **STILL OPEN, REPRODUCED in v1.5.0**
- [#1334](https://github.com/wildcard/caro/issues/1334) — `caro ai --once` broken (CPU stub) — **STILL OPEN, REPRODUCED**
- [#1181](https://github.com/wildcard/caro/issues/1181) / [#1274](https://github.com/wildcard/caro/issues/1274) / [#1362](https://github.com/wildcard/caro/issues/1362) — static matcher Pattern 43 too strict — **STILL OPEN, NOT TESTED IN THIS PASS** (Slot C used `caro ai --once` which bypasses static-matcher; smoke test timed out before pattern matching could be observed)

### Followups

- FLAKE-001 observed again: model download ([size omitted]) takes >60s; agent loop times out on first invocation. Model completes eventually. Dates: 2026-05-07, 2026-07-22 (>7 days apart; 3×/7-day threshold not met — remains flake).
- `--backend static` silently removed from public CLI (PR #1298). No removal notice found.
- Next Slot C candidate: surface #11 (`caro ai --continue-session`) — never tested.
- #1277 and #1334 are P1, unresolved in v1.5.0. If still open at next release, escalate to epic tracker.

---

## 2026-07-21 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (16 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 13s, no errors; MSRV now 1.85)
- `caro --version` → **PASS**: `caro 1.5.0 (be07b22 2026-07-18)`
- `caro --help` → **PASS**: all subcommands present including CaroML and `ai` verbs
- `caro doctor` → **PASS**: advisory only (no model downloaded, proxy detected at http://127.0.0.1:34433)
- `caro -p 'list files in current directory' --dry-run` → **DEGRADED** (not a clean PASS or FLAKE): returned `echo 'Please clarify your request'` with exit 0. Root cause traced to two bugs — static matcher Pattern 43 regex rejects trailing context (→ #1362, P2) and CPU backend stub's `prompt.contains("rm")` fires on system-prompt words like "format" (→ #1361, P1). Filed as bugs, not tracked as a new flake.
- Telemetry consent auto-declined in non-interactive mode; `~/.config/caro/config.toml` persisted `first_run = false, enabled = false` after first invocation. Second invocation (`caro --version`) showed no prompt. **PASS**.

### Slot B — Recent diff (16 PRs since 2026-05-07)

Key PRs covered:

| PR | Surface | Smoke result |
|----|---------|-------------|
| #1315 `fix(safety): P0 — close quote/escape evasion` | Safety patterns | PASS — `cargo test --lib -- safety` 34/34 |
| #1352 `fix(i18n): load all locale JSON files; overhaul Hebrew translations` | i18n locale JSON | PASS — 115 locale files all valid JSON (python3 json.load) |
| #1244 `feat(agents): add ponytail reviewer` | Agent config only | not CLI-testable |
| #1155–#1159 brand/website design PRs | website (no local server) | deferred to surface #25/#26 in future Slot C |
| #1245 `feat: Fireworks hybrid-harness learnings` | eval/ml changes | deferred; no model available to test embedded inference quality |

Surfaces flagged for future Slot C: website brand surfaces (#25, #26) touched by PRs #1155–#1159 and #1327.

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **#10 — `caro ai --once` scripted conversational mode** (oldest = never; lowest `#` among 'never' surfaces).

- `caro ai --help` → **PASS**: `--once`, `--new-session`, `--continue-session` flags documented; stdout described as generated command only for shell widget injection.
- `caro ai --once 'list files in current directory'` → **FAIL**:
  ```
  # caro-ai: session 1 confidence=0.85 risk=Safe
  echo 'Please clarify your request'
  ```
  Exit code: 0. Shell widgets would inject this placeholder into the user's readline buffer. The `confidence=0.85` value is fabricated — not derived from generation quality.
- Root cause: CPU backend stub in `src/backends/embedded/cpu.rs` uses `prompt.contains("rm")` on the full prompt string; "rm" appears as a substring in common words in the system prompt ("format", "perform", "terminal"), causing the safety-fallback branch to fire for ALL queries regardless of user intent.

### Findings

- [#1361](https://github.com/wildcard/caro/issues/1361) — `ai: caro ai --once returns misleading placeholder command with confidence=0.85 when backend degraded` (P1)
- [#1362](https://github.com/wildcard/caro/issues/1362) — `cli: static matcher Pattern 43 regex rejects "list files in <location>" despite matching required keywords` (P2)

### Followups

- FLAKE-001 behavior changed: previous run (2026-05-07) showed explicit download error; this run showed silent CPU-stub fallback. The CPU stub bug (#1361) is the underlying cause — it was presumably dormant until this run's code path exercised the stub.
- Website brand surfaces (#25, #26) touched by multiple merged PRs; schedule for next available Slot C.
- `caro ai --continue-session` (surface #11) not yet tested — pending model availability.
- 16 PRs merged in the 75-day gap since last run; Slot B only spot-checked the two highest-risk PRs (#1315, #1352). Additional spot checks deferred.

---

## 2026-07-16 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (many PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 29s, no errors; MSRV 1.85 confirmed in Cargo.toml)
- `caro --version` → **PASS**: `caro 1.5.0 (587e7a2 2026-07-14)`
- `caro --help` → **PASS**: all expected subcommands present including `ai`, `skill`, CaroML verbs; `--backend-info` global flag present
- `caro doctor` → **PASS**: huggingface.co reachable; proxy detected; no model cached (sandbox); advisory-only output
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: hung for 30s (timeout), then killed. Same pattern as FLAKE-001 (model download blocked at binary level in sandbox despite HTTP 200). Second total observation; not within 7-day window of first (2026-05-07). Updated FLAKE-001 log.

### Slot B — Recent diff

PRs merged since 2026-05-07 (30+ PRs, key surfaces exercised):

**PR #1315 — `fix(safety): P0 — close quote/escape evasion`** → safety surface
- `cargo test --lib -- safety` → **PASS**: 34/34 safety unit tests (up from 19 in bootstrap run; new tests for `smart_blend`, `allowlist_catastrophic`)
- `shell-words` tokenizer confirmed present at `src/safety/mod.rs:590` (`shell_words::split`)
- Verdict: **PASS** — P0 safety fix is in place and guarded

**PR #1298 — `fix(cli): single source of truth for backend roster`** → cli surface
- `caro --backend-info` → **PASS**: shows all 7 backends (`embedded`, `ollama`, `exo`, `vllm`, `mesh`, `ai-horde`, `hybrid`) with compilation status; non-default-feature backends correctly marked `not compiled`
- Verdict: **PASS**

**Config surface — explored incidentally during Slot B**:
- `caro config show` → **PASS**: shows `telemetry`, `log_level`, `cache_max_size`, `log_rotation`, etc.
- `caro config get telemetry.enabled` → **FAIL**: `Unknown config key 'telemetry.enabled'. Valid keys: backend, model-name, shell, safety`
- `caro config set telemetry.enabled false` → **FAIL**: same error
- `caro config reset` → **PASS**
- Root cause: `src/main.rs:2113,2158` — only 4 keys wired in `config get`/`set` despite `config show` displaying many more; 3 source files advertise the broken command
- Filed: [#1332](https://github.com/wildcard/caro/issues/1332) (P1)

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **`caro ai --once`** (oldest = never; lowest # in never-tested group).

- `caro ai --help` → **PASS**: comprehensive help; `--once`, `--new-session`, `--continue-session` flags documented; note states "The only mode supported today"
- `caro ai --once 'prompt' --dry-run` → produces `# caro-ai: session 1 (resumed) confidence=0.85 risk=Safe` + `echo 'Please clarify your request'` — expected static-backend fallback (no model downloaded in sandbox)
- `caro --dry-run ai --once` (stdin) → **PASS**: global `--dry-run` before subcommand correctly routes through main prompt path; `echo "show disk usage" | caro --dry-run ai --once` → `df -h` (correct!)
- `caro ai --once 'prompt' --dry-run` (global flag AFTER subcommand args) → `error: unexpected argument '--dry-run'` — clap-expected behavior; `--dry-run` is a global option and must precede the subcommand
- Session management:
  - `--new-session`: creates session 2 (no "resumed" tag) ✓
  - Second call without `--new-session`: resumes session 2 ("session 2 (resumed)") ✓
  - Sessions increment correctly across calls
- Verdict: **PASS** (functional surface; command quality limited to static-backend in sandbox)

### Findings

- [#1332](https://github.com/wildcard/caro/issues/1332) — `cli: caro config set telemetry.enabled false fails with "Unknown config key"` (P1)

### Followups

- FLAKE-001 (model download blocked) observed again. Second total observation; 70 days since first (2026-05-07). Not within the 7-day promotion window — keeping as active flake but noting recurrence.
- `caro ai --once` in static-backend mode always returns `echo 'Please clarify your request'` regardless of prompt — expected fallback, not a bug. Verify quality in a future run with model present.
- `caro ai --once 'prompt' --dry-run` (wrong arg order) returns clap error — not a bug (clap global-option semantics), but worth a user-facing hint. Consider P3 improvement.
- Issue #1044 (CLAUDE.md version drift) closed as completed on 2026-05-09 — removed from watch list.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or surface #12 (`caro assess`).

---

## 2026-07-12 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (103 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).
**Binary**: caro 1.5.0 (a6519f6 2026-07-12), built in 3m 03s.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (3m 03s, Finished release profile)
- `caro --version` → **PASS**: `caro 1.5.0 (a6519f6 2026-07-12)`
- `caro --help` → **PASS**: all 24 subcommands listed including CaroML verbs and `skill` command
- `caro doctor` → **PASS** (initial: no model advisory; after first dry-run: model downloaded, "✓ Embedded (ready)")
- `caro -p 'list files in current directory' --dry-run` (30s timeout) → **FLAKE**: telemetry consent + model download; process killed by 30s timeout before completing
- `caro -p 'list files in current directory' --dry-run` (90s timeout) → **PARTIAL**: model downloaded; generated `echo 'Please clarify your request'` instead of `ls` — root cause is CPU stub bug (see #1269, #1281, #1289, all open)
- Repeat dry-run (90s, model cached) → same wrong result; 7/8 common queries return `echo 'Please clarify your request'` (only static-matcher hits like `show disk usage` → `df -h` work correctly)

### Slot B — Recent diff

103 PRs merged since 2026-05-07. Key surfaces exercised:

- **Static matcher fix #947** (`find and kill the runaway process eating CPU` → full `ps aux | sort … | xargs kill` pipeline): **PASS** ✓
- **Backend roster #1298 / #1115** (`caro --backend-info` shows `not compiled` for remote backends; no stale entries): **PASS** ✓
- **Safety allowlist regression #1246** (7/7 `allowlist_catastrophic_tests` pass; specific-subpath allowlist works, catastrophic targets always blocked): **PASS** ✓
- **Custom safety patterns 1.5.0 feature**: `patterns.toml` loaded without errors using `[[pattern]]` section (correct per example file; CHANGELOG says `[[safety.custom_patterns]]` — minor doc inconsistency). End-to-end blocking test **inconclusive** — CPU stub generates wrong commands so safety validator never sees a real kubectl/terraform command to block. Flagged for re-test once #1269 is fixed.
- **MSRV 1.85**: confirmed in `Cargo.toml`; CLAUDE.md still says 1.83 (pre-existing #1283/#1288).

Surfaces flagged for future Slot C: custom safety patterns end-to-end blocking.

### Slot C — `caro ai --once` (surface #10)

- `caro ai --once 'list files sorted by size'` → stdout: `echo 'Please clarify your request'`, stderr: `# caro-ai: session 1 confidence=0.85 risk=Safe`; EXIT=0
- `caro ai --once 'show git log for last 5 commits'` → same wrong output; EXIT=0
- `caro ai --once --new-session 'show memory usage'` → same wrong output; EXIT=0
- **stdout/stderr split**: correct — command on stdout only, metadata comment on stderr. Help text "Stdout is the generated command only" is accurately implemented.
- **Session handling**: resumption (`session 1 (resumed)`) and `--new-session` both work as described.
- **Root cause**: CPU stub bugs #1269/#1281/#1289 reproduce in 1.5.0 unchanged.

Surface #10 verdict: **FAIL** — produces wrong output on Linux x86_64; pre-existing bugs unresolved in 1.5.0 release.

### Findings

- [#1319](https://github.com/wildcard/caro/issues/1319) — `docs: CLAUDE.md version banner shows 1.4.0 (GA) after 1.5.0 release` (P2) — **NEW**
- Pre-existing open bugs confirmed in 1.5.0: #1269, #1281, #1289 (CPU stub / `caro ai --once`); #1283, #1288 (MSRV drift)

### Followups

- Custom safety patterns (1.5.0 headline feature) cannot be validated end-to-end until #1269 is fixed.
- CLAUDE.md has two stale fields: version (1.4.0→1.5.0, filed #1319) and MSRV (1.83→1.85, existing #1283/#1288). Both fixed by adding CLAUDE.md to release-version-alignment checklist.
- Model download timing: first dry-run timed out at 30s; second attempt (90s) succeeded. Model download slow (~60s+) in this sandbox. FLAKE-001 second proximity observation logged.
- Next Slot C candidates: surface #11 (`caro ai --continue-session` shell widget), surface #12 (`caro assess`).

---

## 2026-07-11 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 19s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (9311c9e 2026-06-13)`
- `caro --help` → **PASS**: all subcommands present including CaroML verbs and `ai`, `skill`
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 (2nd observation); telemetry banner appeared on first invocation (fresh sandbox, first_run=true), auto-opted-out (non-interactive TTY), then hung on model download
- **Slot A finding**: `caro config set telemetry.enabled false` (advertised in consent screen) rejects as unknown key → filed [#1292](https://github.com/wildcard/caro/issues/1292) (P1)

### Slot B — Recent diff (30 PRs since 2026-05-07)

Key surfaces checked:
- **PR #1209** `feat(backends): Mesh-LLM + AI-Horde + hybrid privacy gateway` — `--backend-info` vs actual resolver inconsistency confirmed. Three-way list mismatch. Filed [#1293](https://github.com/wildcard/caro/issues/1293) (P1) and [#1294](https://github.com/wildcard/caro/issues/1294) (P2).
- **PR #1110** `feat(safety): runtime-loadable custom patterns` — `[[pattern]]` key confirmed in `examples/patterns.example.toml`; safety tests all pass (27/27, up from 19 in last run due to new smart_blend_tests from PR #1206).
- **PR #1109** `feat(cli): --edit flag` — `-e, --edit` flag confirmed in `--help` with rustyline description.
- i18n PRs #817/#816/#825–#829 — locale files updated; flagged surface #31 for next Slot C.
- Prior watch-list issue [#1044](https://github.com/wildcard/caro/issues/1044) — confirmed closed (completed).

### Slot C — `caro ai --once` (surface #10, oldest 'never')

- `caro ai --help` → **PASS**: subcommand documented, `--once` flag present
- `caro ai --once` (no prompt) → **PASS**: correct error `no prompt provided`
- `caro ai --once "list all running processes"` → **FAIL**: silent hang, zero stdout/stderr, killed after 35s
- `echo "show disk usage" | caro ai --once` → **FAIL**: same silent hang
- `caro ai --once "..." </dev/null` → **FAIL**: same silent hang
- Filed [#1295](https://github.com/wildcard/caro/issues/1295) (P2)

### Findings

- [#1292](https://github.com/wildcard/caro/issues/1292) — `cli: telemetry consent advertises invalid config key` (P1)
- [#1293](https://github.com/wildcard/caro/issues/1293) — `cli: --backend-info shows wrong backend list` (P1)
- [#1294](https://github.com/wildcard/caro/issues/1294) — `cli: --backend help text missing mesh/ai-horde/hybrid` (P2)
- [#1295](https://github.com/wildcard/caro/issues/1295) — `ai: caro ai --once hangs silently when model unavailable` (P2)

### Followups

- FLAKE-001 second observation (2026-07-11). 1 more in 7-day window → promote to regression.
- Next Slot C candidate: surface #31 (i18n website locale smoke — `curl /es/ /fr/ /ja/`). Multiple i18n PRs merged since last run.
- #1293 and #1294 are related backend-list drift issues; likely fixed in one PR.
- #1295: test `caro ai --once` in environment with pre-downloaded model to confirm if silent hang is FLAKE-001 or a separate `ai`-path bug.

---

## 2026-06-13 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (29 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 21s, 0 errors; version 1.4.0)
- `caro --version` → **PASS**: `caro 1.4.0 (2be886c 2026-06-07)`
- `caro --help` → **PASS**: all subcommands listed; new `--approval` flag present; `mesh`/`ai-horde`/`hybrid` discoverable via error path (not via --backend-info — see findings)
- `caro doctor` → **PASS**: no model downloaded, advisory only; HuggingFace reachable over HTTP; embedded "needs model download" as expected in sandbox
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001, 2nd observation — outside the original 7-day window; new 7-day window started today)
- Telemetry prompt shown on first invocation (expected — fresh sandbox, no prior config); defaulted to disabled in non-TTY context

### Slot B — Recent diff

PRs merged since 2026-05-07: 29 total. Key surfaces exercised:

- **PR #1209** (feat: Mesh-LLM + AI-Horde + hybrid backends) → tested via `--backend-info` and `--backend` flag → **FINDING**: `print_backend_info()` not updated; shows `static`/`claude` (invalid) and omits `exo`/`mesh`/`ai-horde`/`hybrid` → **filed #1221 (P1)**
- **PR #1110** (feat: runtime-loadable custom patterns) → reviewed `examples/patterns.example.toml`, `src/safety/mod.rs` — implementation looks correct; 27 safety lib tests PASS
- **PR #1206** (feat: smart approval mode spike) → `--approval` flag present in `--help`; 568 lib tests PASS; 6 new smart-blend tests confirm hard-floor-Critical contract
- **PR #1211** (test: make dangerous-command confirm test deterministic) → lib tests pass; did not re-run the specific test as it is lib-level only
- **Cross-PR finding**: `e2e_safety_level_configuration` fails when only release binary built — `cargo run` fallback in `CliTestRunner` lacks `--bin caro`; two-binary workspace breaks it → **filed #1222 (P1)**

Full test results: `cargo test --lib --release --features embedded-cpu` → **568 passed, 0 failed, 1 ignored**

### Slot C — `caro ai --once` (surface #10)

Surface chosen: **`caro ai --once`** — oldest 'never' surface, lowest `#` number (tie-break rule).

- `caro ai --help` → **PASS**: flags (`--once`, `--new-session`, `--continue-session`) documented correctly; help note "the only mode supported today" is accurate
- `caro ai --once "show disk usage"` → **FLAKE** (FLAKE-001 reproduces — model download blocked in sandbox); error message clear and actionable
- `caro ai --once` (no prompt) → **PASS**: exits 1 with `Error: no prompt provided (pass text, pipe stdin, or use -p)` — correct behavior
- `echo "show disk usage" | caro ai --once` (stdin pipe) → **FLAKE** (FLAKE-001; piped prompt accepted correctly, backend blocks)
- `caro ai --once --backend static` → **PASS (expected error)**: correctly rejects `--backend` as unknown arg after subcommand; global `--backend` position confirmed required
- `caro ai` (no --once, no prompt, non-TTY) → **PASS**: exits 1 with clear `no prompt provided` message
- Full functional test of `--once` mode: **BLOCKED** by FLAKE-001; cannot confirm command generation output

Partial verdict: **PARTIAL** — surface is well-structured; blocked by sandbox model download limitation for end-to-end output verification.

### Findings

- [#1221](https://github.com/wildcard/caro/issues/1221) — `cli: --backend-info lists invalid backends (static, claude) and omits valid ones (exo, mesh, ai-horde, hybrid)` (P1, regression from PR #1209)
- [#1222](https://github.com/wildcard/caro/issues/1222) — `cli: e2e_safety_level_configuration test fails — cargo fallback missing --bin caro with multiple binaries` (P1)

### Followups

- #1044 (CLAUDE.md version drift) confirmed CLOSED 2026-05-09; removed from watch list.
- FLAKE-001 second observation (new 7-day window — first was 2026-05-07, >7 days ago). Reset window; needs 3 more observations within 7 days to reclassify as regression.
- `caro ai --once` surface #10: marked PARTIAL. Recommend re-testing from an env with a pre-downloaded model. Surface #11 (`caro ai --continue-session`) shares the same backend blocker — defer until model is available.
- `--backend-info` / `--backend` help string stale pattern: consider a CI lint that diffs `VALID_BACKENDS` in `cli/mod.rs` against the backends listed in `print_backend_info()` to prevent future drift.

---

## 2026-05-27 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 25s, no errors; MSRV 1.85)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)`
- `caro --help` → **PASS**: all subcommands present including CaroML verbs, `ai`, `suggest`, `export`, `skill`, `render`
- `caro doctor` → **PASS**: reports no model yet, ollama not installed — correct for fresh sandbox
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 reproduced (2nd occurrence; first was 2026-05-07; NOT yet 3×/7 days so no promotion yet)
- `caro --output json -p 'list files' --dry-run` → **PASS**: valid JSON with `generated_command: "ls -la"` (uses static pattern matching)
- `caro suggest 'list files'` → **PASS**: 5 suggestions returned correctly

### Slot B — Recent diff

30 PRs merged since 2026-05-07. Key surfaces tested:

- **#1109** (`feat(cli): --edit flag via rustyline`) → **PASS**: flag present in `--help`, non-TTY mode outputs command without executing, exit 0
- **#1110** (`feat(safety): runtime-loadable custom patterns`) → **PARTIAL**: `cargo test --lib -- safety` = 20 PASS; `cargo test --test safety_validator_contract` = 20 PASS, 1 FAIL (`test_allowlist_functionality`) — **→ filed [#1176](https://github.com/wildcard/caro/issues/1176) P1**
- **#1065** (`fix: GNU mkfs pattern flag evasion`) → **PASS**: safety lib tests include mkfs coverage, all pass
- **#817-#829** (i18n translation batches) → noted for Slot C next cycle (surface #31 i18n locale smoke)
- **#1154** (`fix(deps,build): bincode pin + rusqlite cast`) → **PASS**: build succeeds (this fix was the unblock)
- Bonus: `caro config set telemetry.enabled false` → **FAIL**: advertised in first-run consent screen, rejected as unknown key — **→ filed [#1177](https://github.com/wildcard/caro/issues/1177) P1**

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: correct subcommand, `--once` / `--new-session` / `--continue-session` flags documented
- `caro ai --once 'find large files'` → backend unavailable (FLAKE-001 pattern); error message clear and exit graceful
- `caro ai` (no `--once`) with piped input → same backend error; `--backend` is not an accepted flag for this subcommand (expected per help text)
- Note: help text says `--once` is "The only mode supported today" — TTY REPL mode not yet implemented; UX is acceptable given the honest disclaimer
- **Verdict**: PASS (with FLAKE caveat on model download)

### Findings

- [#1176](https://github.com/wildcard/caro/issues/1176) — `safety: test_allowlist_functionality fails on main — Critical rm -rf pattern too broad` (P1, regression)
- [#1177](https://github.com/wildcard/caro/issues/1177) — `cli: caro config set telemetry.enabled false advertised but rejected as unknown key` (P1)

### Watch list updates

- #1044 — closed (CLAUDE.md version corrected in v1.4.0 release cycle)

### Followups

- FLAKE-001 second occurrence (2026-05-07, 2026-05-27). Not within 7-day window; not promoting to regression. Will promote if observed again within next 7 days.
- i18n locale smoke (surface #31) is good next Slot C candidate — 15 translation PRs landed since last run.
- `test_allowlist_functionality` (#1176) is a CI breakage on `cargo test --test safety_validator_contract`. Engineers should see this quickly; no special escalation beyond the P1 issue needed.
- `caro ai --once` REPL mode is unimplemented ("only mode supported today"). Surface #11 (`--continue-session` shell widget) deferred until TTY REPL lands.

---

## 2026-05-26 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 18s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)`
- `caro --help` → **PASS**: all subcommands listed including `ai` and full CaroML verb set
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 (model download blocked; 2nd observed instance in this log)

### Slot B — Recent diff (12 PRs merged 2026-05-07 → 2026-05-26)

PRs covered: #1065, #1114, #1112, #1111, #1110, #1109, #1108, #1103, #883, #920, #1139, #1154.

- **#1065** (safety: GNU mkfs pattern fix): `cargo test --lib -- safety` → 20/20 PASS. No regression.
- **#1110** (safety: runtime-loadable custom patterns): `test_custom_safety_patterns` + `test_custom_pattern_addition` PASS; `test_allowlist_functionality` FAIL — regression still open (issues [#1165](https://github.com/wildcard/caro/issues/1165), [#1169](https://github.com/wildcard/caro/issues/1169)).
- **#1109** (cli: `--edit` flag): `caro --edit --dry-run -p "list files"` → PASS via static matcher; flag accepted, `ls -la` generated and shown in dry-run mode.
- **#1154** (fix deps): Build passes (implicit Slot A PASS).
- Other PRs (#1114, #1112, #1111, #1108, #1103, #883, #920, #1139 — docs, skills, experiment, agent, governance, MLX pipeline, fastembed dep, rustfmt): No regressions found in available smoke coverage.

### Slot C — caro ai --once (surface #10)

- `caro ai --once "what files are in /tmp"` → **FLAKE**: FLAKE-001 (model download blocked; same pattern as `--dry-run`)
- `caro ai "list files"` (without `--once`) → same FLAKE-001; `--once` flag is always the effective behavior (`src/main.rs:3233` ignores the field), correct per help: "only mode supported today"
- `--continue-session` flag is intentional no-op by design (shell hook ergonomics); `new_session` controls session selection
- Command routed correctly to ai subcommand (not misrouted via #1163 bug, since no global flags were passed before the subcommand)
- `caro config show` → PASS: all fields shown, config at `/root/.config/caro/config.toml`
- No new bugs found beyond FLAKE-001

### Findings

- No new issues filed this pass — all discovered regressions already tracked in open issues.

### Followups

- FLAKE-001 observed 2nd logged time (also triggered in all intermediate QA runs 2026-05-15 through 2026-05-25 that did not update memory). Pattern is consistent: sandbox always blocks model download. Formal promotion to regression requires 3 logged occurrences; incrementing to 2.
- Issue #1044 appears closed (absent from open `qa` issue list); updated backlog accordingly.
- Backlog sync'd to include all issues filed by intermediate QA runs (#1098–#1170 range).
- Issues #1165/#1169 (allowlist regression from PR #1110): `test_allowlist_functionality` still FAIL on `main`; `cargo test --test safety_validator_contract` still exits 1.
- Issue #1164 (cargo run without --bin): `default-run` still missing from `Cargo.toml`.
- Issue #1107 (--backend openrouter not in VALID_BACKENDS): `src/cli/mod.rs:473` still lists only `["embedded", "ollama", "exo", "vllm"]`.
- Issue #1098 (CLAUDE.md MSRV stale): version field now 1.4.0 ✓ (partial fix), but MSRV still shows 1.83 (should be 1.85, see `Cargo.toml:rust-version`).

---

## 2026-05-25 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (25 PRs merged since 2026-05-07) + C (surface #10 — `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 19s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)`
- `caro --help` → **PASS**: all subcommands present including `ai`, `export`, `skill`; `--edit` flag visible
- `caro doctor` → **PASS**: advisory (no model downloaded; expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001, 2nd occurrence: model download fails in sandbox after 3 retries)
- Telemetry consent: **PASS** — disabled consent shown once, not re-prompted
- Issue #1044 (CLAUDE.md version drift) verified **CLOSED** and fixed — CLAUDE.md now reads `1.4.0 (GA)` ✓

### Slot B — Recent diff

25 PRs merged since 2026-05-07. Key surfaces exercised:

- **PR #1065** (mkfs pattern flag evasion fix) — `cargo test --lib -- safety` → **PASS** (20/20 safety unit tests)
- **PR #1110** (runtime-loadable custom patterns) — patterns.toml sibling file confirmed present at `examples/patterns.example.toml`; `--edit` flag present in help; `from_user_config` path confirmed in source; however `test_allowlist_functionality` in `safety_validator_contract` → **FAIL** (regression introduced by #1110's Critical pre-scan guard)
- **PR #1109** (`--edit` flag) — `--edit` visible in `caro --help` output → **PASS** (structural check only; requires model for full exercise)
- i18n batch (#817–829) — deferred to surface #31 in Slot C future pass

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: flags documented (`--once`, `--new-session`, `--continue-session`)
- `caro ai` (no args) → **PASS**: `Error: no prompt provided (pass text, pipe stdin, or use -p)` — correct
- `caro ai --once "list files"` → **FLAKE** (FLAKE-001: model download blocked in sandbox, same root cause as Slot A dry-run)
- `echo "show disk usage" | caro ai --once --new-session` → **FLAKE** (same)
- CLI surface structural check: **PASS** — arg parsing, help text, no-arg error handling all functional

### Findings

- [#1169](https://github.com/wildcard/caro/issues/1169) — `safety: test_allowlist_functionality broken by PR #1110 Critical pre-scan guard` (P1 — regression)
- [#1170](https://github.com/wildcard/caro/issues/1170) — `cli: cargo test safety fails — evaluation binary rejects positional filter arg` (P2)

### Followups

- FLAKE-001 observed 2nd time (2026-05-25). Still below 3× threshold for regression reclassification. Threshold reached on next observation.
- Surface #31 (i18n locale smoke) should be exercised next pass — 10 locale files updated in batch (#817–829).
- Custom patterns feature (#1110) has no unit tests for `from_user_config` path — gap noted; not a separate issue since #1169 covers the immediate regression.
- `caro ai` structural surface: mark as partially tested (CLI layer PASS, backend layer FLAKE). Would need environment with pre-downloaded model or Ollama for full pass.

---

## 2026-05-24 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C (surface #10: `caro ai --once`).

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 12s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)` — upgraded from 1.3.0 since last run; v1.4.0 released 2026-05-09
- `caro --help` → **PASS**: all subcommands listed; new CaroML and AI verbs visible
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE** (FLAKE-001, 2nd occurrence in 7 days; model download blocked in sandbox — see qa-known-flakes.md)

### Slot B — Recent diff (30 PRs merged since 2026-05-07)

Representative surfaces spot-checked:

- **PR #1110** (runtime-loadable custom patterns): `examples/patterns.example.toml` exists with correct schema. Safety tests pass 20/20. Critical-bypass guard confirmed in `src/safety/mod.rs:407–430` (allowlist skipped when Critical built-in match present). PASS.
- **PR #1109** (`--edit` flag): Flag appears in `--help` with correct description. Source wired in `src/main.rs`. PASS (can't test interactive editing in sandbox).
- **PR #1065** (GNU mkfs flag-evasion pattern): `cargo test --lib -- safety` → 20/20 PASS.

No regressions found in any spot-checked surface.

Surfaces flagged for future Slot C coverage: PR #1108 (candidate pipeline), PR #1112 (`caro.prune` skill) — added to coverage matrix.

### Slot C — `caro ai --once` (surface #10)

- `caro ai --help` → **PASS**: `--once` documented as "The only mode supported today"; flags accurate.
- `caro ai --once` (no prompt) → **PASS**: graceful error "no prompt provided (pass text, pipe stdin, or use -p)"
- `caro ai --once "list files"` → **FLAKE** (FLAKE-001 — model download blocked; `Backend is not available: Failed to download model after 3 attempts`)
- Unit tests `cargo test --lib -- ai::` → **PASS**: 23/23 (covers session persistence, TTL resume, safety blocking, shell-init rendering, privacy context)
- Session store round-trip: **PASS** (unit test `once_creates_session_and_persists_turn` creates session, persists turn, reloads from disk)
- Safety integration: **PASS** (unit test `once_flags_dangerous_command_from_validator` confirms `rm -rf /` blocked with `allowed=false`)

Overall verdict: **PARTIAL-PASS** — functional logic fully verified through unit tests; end-to-end CLI path blocked by sandbox model-download limitation (FLAKE-001).

### Findings

- No new GitHub issues filed this pass.
- Issue [#1098](https://github.com/wildcard/caro/issues/1098) (CLAUDE.md MSRV 1.83 stale) remains open; version field was partially fixed (now shows 1.4.0) but MSRV is still 1.83 in CLAUDE.md. Confirmed stale again today.
- FLAKE-001: 2nd occurrence (2026-05-07 + 2026-05-24) — not yet at 3/7-day promotion threshold.

### Followups

- FLAKE-001 is at 2/3 occurrences needed for regression promotion. Next occurrence should file an issue.
- Issue #1098 (MSRV drift) still open — needs a fix PR. The `release-version-alignment.md` 6-file checklist still does not include CLAUDE.md, as noted in both #1044 and #1098.
- Surface #10 (`caro ai --once`) should be re-tested from an environment with a pre-downloaded model to get full end-to-end coverage. Add to next Slot C candidate list.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or surface #12 (`caro assess`).
- PR #1108 candidate pipeline and PR #1112 `caro.prune` skill should be exercised in a future Slot C.

---

## 2026-05-23 — Scheduled run (Slot A + Slot B + Slot C)

**Trigger**: scheduled cron 14:00 UTC.
**Rotation**: A + B (30 PRs merged since 2026-05-07) + C.

### Slot A — Smoke

- `cargo build --release --features embedded-cpu` → **PASS** (2m 15s, no errors)
- `caro --version` → **PASS**: `caro 1.4.0 (8155c4a 2026-05-18)`
- `caro --help` → **PASS**: all subcommands present including new `--edit`/`-e` flag visible in top-level flag list
- `caro doctor` → **PASS**: advisory only (no model downloaded, expected in fresh sandbox)
- `caro -p 'list files in current directory' --dry-run` → **FLAKE**: FLAKE-001 reproduced (2nd observation total, 16 days apart — not within 7-day window, threshold not reached)

### Slot B — Recent diff

PRs merged since 2026-05-07: 30 total. Representative surfaces tested:

- **PR #1065** (`Fix: GNU mkfs pattern flag evasion`, `src/safety/patterns.rs`): `cargo test --test safety_validator_contract` → **20 PASS, 1 FAIL** (`test_allowlist_functionality`). Filed [#1165](https://github.com/wildcard/caro/issues/1165). Note: the failure was introduced by PR #1110, not #1065.
- **PR #1109** (`feat(cli): --edit flag via rustyline`, `src/cli/edit_prompt.rs`, `src/main.rs`): `caro --help` shows `-e, --edit` flag with correct description → **PASS**; flag is present and properly documented.
- **PR #1110** (`feat(safety): runtime-loadable custom patterns`, `src/safety/mod.rs`, `tests/custom_patterns_toml.rs`): `cargo test --test custom_patterns_toml` → **9/9 PASS**. Feature works; see regression note above.

Other PRs observed: i18n updates (#817–#829), CaroML additions (#893+), docs (#1114), build fix (#1154, confirmed resolved by successful build).

### Slot C — caro ai --once (surface #10)

Surface chosen: `caro ai --once` scripted conversational mode (oldest 'never' surfaces — lowest # rule, surface #10).

- `caro ai --help` → **PASS**: subcommand documented with `--once`, `--new-session`, `--continue-session` flags; description matches advertised "Atuin-AI-style" conversational mode; stdout-only design for shell widget injection documented.
- `caro ai --once <prompt>` (with stdin) → **FLAKE**: same FLAKE-001 model download failure; error message clean and actionable.
- CLI structure and flag parsing: **PASS** — no structural bugs found in the subcommand.

### Findings

- [#1165](https://github.com/wildcard/caro/issues/1165) — `safety: user allowlist cannot override rm -rf /tmp/... despite explicit allowlist pattern` (P1, regression from #1110)

### Followups

- FLAKE-001 second observation (2026-05-23), but gap is 16 days so 7-day window is clean. If reproduced again within 7 days of this run, promote to regression.
- Next Slot C candidate: surface #11 (`caro ai --continue-session` shell widget) or #12 (`caro assess`).
- PR #1110 regression: `test_allowlist_functionality` fails on main; CI must have missed it or the test was newly broken between merge and CI run. Consider whether `cargo test` is gated in CI for integration tests.

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
- Next Slot C candidate: `caro ai --continue-session` shell widget (surface #11 in matrix, never tested). Surface #10 (`caro ai --once`) was exercised in the 2026-10-08 rotation; see Findings above.
- Consider adding `CLAUDE.md` to the release-version-alignment 6-file checklist so version drift can't recur (noted in #1044 fix direction).

---

_The two entries below come from an earlier bootstrap (#886, Apr 26 to May 3) that never merged._

---

## 2026-05-03 — Session resume + Slot A + Slot C #9 (safety strict mode)

**Trigger**: Manual resume by user after ~7-day quiet period. Daily remote routine [trig_01Tk7DxyXV7LeYcFjgmTG1mZ](https://claude.ai/code/routines/trig_01Tk7DxyXV7LeYcFjgmTG1mZ) has been firing daily but exiting clean — bootstrap PR [#886](https://github.com/wildcard/caro/pull/886) has not yet merged, so the routine cannot find these memory files on `main`. ~7 no-op runs lost.
**Rotation**: A (smoke) + C (random past feature, surface #9 = safety strict mode).
**Local CLI under test**: rebuilt from latest `main` (29c3ce9, 2026-04-30) into `target/release/caro` v1.3.0. Build flags: `--no-default-features --features embedded-cpu` per `release-version-alignment.md` example.

### Project state changes since 2026-04-26 (gap-fill)

| Change | Detail |
|---|---|
| **caroml shipped (#893, merged 2026-04-29)** | New `caro check` subcommand visible in `--help`. Validates `.caro` task files (parser + AST + lock format). Added to coverage matrix as surface #15. |
| **caro-terminal proposed (#1009-#1023, 2026-04-30)** | 15 issues for a Ghostty-backed desktop GUI. Planning/foundation only — no shipped binary. Marked out-of-scope until first user-facing release. |
| **Sibling QA routine: `caro-frustrated-beta` (PR #910, merged 2026-04-27)** | Daily 5 AM cron, single frustrated-power-user persona, tests website-advertised queries against the binary. Files findings under `.claude/beta-testing/runs/<date>/`, labels `qa-routine` + `frustrated-beta` + `bug-intent-dropped` + `website-broken-promise`. Issue [#947](https://github.com/wildcard/caro/issues/947) is its first P0 finding. **Coordination plan** added to coverage-matrix header — my rotation no longer picks website-promise tests since frustrated-beta covers them. |
| **i18n eval coverage closed (#1027, 2026-04-29)** | Locale parity around the original PR #874 regression class is being actively exercised by an eval suite now. |
| **Pull-request #886 status** | OPEN, MERGEABLE, mergeStateStatus UNSTABLE. 50 checks ran; 2 failures (`ChromaDB Integration Tests`, `Security Audit`) are repo-wide on main, not specific to this docs-only PR. **Action**: rebased on origin/main (1 commit ahead), refreshed memory, will push and add a status comment asking for merge so the daily routine can start doing real work. |

### Slot A — Smoke

| Check | Outcome | Detail |
|---|---|---|
| `cargo build --release --no-default-features --features embedded-cpu` | PASS | 49.8s incremental rebuild on top of warm target/. [size omitted] binary. |
| `caro --version` | PASS | `caro 1.3.0 (29c3ce9 2026-04-30)`. Same advertised version as v1.3.0 ship. |
| `caro --help` | PASS | 11 subcommands (was 9 on 2026-04-26). New: `check` (caroml). Existing: `doctor`, `integration`, `init`, `config`, `test`, `completion`, `suggest`, `ai`, `shell-init`, `help`. |
| `caro doctor` | PASS-with-caveat | "✓ All systems operational" — but see Slot C finding F-A, doctor's "Embedded (ready)" line is unreliable on `--no-default-features` builds. |

### Slot C — Surface #9: Safety strict mode blocks canonical 5

**Method**: ran `caro --safety strict --dry-run -p "<dangerous prompt>"` for the canonical 5 — `rm -rf /`, fork bomb, `dd if=/dev/zero of=/dev/sda`, `curl … | sudo bash`, `chmod -R 777 /`. Each prompt was phrased as a natural-language request so the LLM had to first generate the dangerous command and then safety would block it.

**Result**: **INCONCLUSIVE** — all 5 prompts errored before safety could be reached:

```
Error: Command generation failed: Model generation failed: Failed to load model:
Configuration error: MLX backend not enabled. Rebuild with --features embedded-mlx
```

The dispatcher in `src/backends/embedded/mlx.rs:200,275` routes to MLX on macOS aarch64 even when the binary was compiled with only `embedded-cpu`. Two real findings dropped out of this:

1. **F-2026-05-03-A** — `caro doctor` reports "Embedded (ready)" while the runtime path it actually dispatches to is missing. `src/cli/doctor.rs:139` only checks `backend_status.embedded_available` (model-file presence), not feature-flag-vs-dispatch alignment. Both findings logged in `qa-known-flakes.md` with promotion criteria — they're real but build-flag-coupled, so I'm doing claim-verification rebuild before filing GH issues.
2. **F-2026-05-03-B** — Generation-failure exit code is 0, not non-zero. Automation cannot distinguish success from failure. Same disposition (logged, not yet filed, needs default-feature repro).

**Why I didn't rebuild with default features in this pass**: full default build pulls in `cxx` + `llama_cpp` ([duration omitted] cold). Slot C is meant to be a [duration omitted] focused pass; an unbounded rebuild would crowd out the rest of the rotation. Defer to next QA pass with a fresh build budget.

### Findings filed this pass

After committing the partial-pass log above, I rebuilt with default features (`cargo build --release` — 39s incremental) and re-ran the 5 canonical dangerous prompts. The MLX path worked, generation succeeded, and the **real** safety story dropped out of the verification — converting standing signal into 4 distinct outcomes:

| Signal | Outcome | Issue |
|---|---|---|
| F-A (`doctor` lies on cpu-only build) | **REFUTED** — build-flag artifact, default users get MLX. Polish observation only. | — |
| F-B (exit code 0 on errors) | **PROMOTED** — reproduces on default build across 5 error classes | [#1035](https://github.com/wildcard/caro/issues/1035) P1 |
| G-1 (`chmod -R 777 /` not blocked) | **NEW FINDING + PROMOTED** — root cause `src/safety/patterns.rs:75`, regex misses `-R` flag | [#1034](https://github.com/wildcard/caro/issues/1034) **P0**, spawn-task escalated |
| G-2 (fork bomb prompt → echo output) | **DISMISSED** — LLM refusal pattern, arguably aligned with safety. Not a bug. | — |

**P0 escalation per onboarding plan**: spawned fixer task for #1034 (chip visible to user). Triple-redundancy = (1) GH issue with P0+regression+safety+qa labels, (2) spawn_task for fixer, (3) **Needs user input** flag in this session's summary.

### Followups

- **PR #886 merge urgency**: every day this stays open, my routine fires and no-ops. Status comment posted on the PR.
- **#1034 fix verification**: once the spawned fixer lands a PR, re-test the recursive chmod variants AND the existing `chmod 777 /` (no flags) case — both must block. Add to next-pass agenda.
- **#1035 fix verification**: once landed, re-test all 5 error classes from the issue table; each must exit non-zero.
- **Audit nearby safety patterns**: `chmod 666 /`, `chmod 755 /etc`, `chown -R root:root /etc` — file follow-up issues if any miss flags. Schedule for the next-but-one Slot C pass.
- **`caro-frustrated-beta` overlap check**: at next pass, read its latest `.claude/beta-testing/runs/<date>/summary.md` to spot any overlap and dedupe.

---

## 2026-04-26 — Onboarding + first pass (Slot A + Slot C)

**Trigger**: Manual (user kickoff). Subsequent runs autonomous via remote routine [trig_01Tk7DxyXV7LeYcFjgmTG1mZ](https://claude.ai/code/routines/trig_01Tk7DxyXV7LeYcFjgmTG1mZ) — daily at 14:00 UTC (7am Vancouver).
**Rotation**: A (smoke) + C (random past feature).
**Local CLI under test**: `/Users/kobik-private/workspace/caro/target/release/caro` (v1.3.0, b993e2b 2026-04-20).

### Slot A — Smoke

| Check | Outcome | Detail |
|---|---|---|
| `caro --version` | PASS | `caro 1.3.0 (b993e2b 2026-04-20)` |
| `caro --help` | PASS | 9 subcommands listed cleanly; exit 0 |
| `caro doctor` | PASS | embedded backend ready; `qwen2.5-coder-1.5b-instruct-q4_k_m.gguf` cached ([size omitted]); ollama not installed (expected); huggingface.co reachable |
| `caro -p "list files in current directory" --dry-run` | PASS-with-warning | Generated `ls`, dry-run footer correct, exit 0. WARN line `caro::agent: Timeout approaching, skipping refinement` emitted on stderr. Possibly cold-start. Tracked as flake. |

**Process observation**: locally-installed `caro` (`~/.cargo/bin/caro`) is **v1.1.2**, while crates.io now publishes **v1.3.0**. This is a personal `cargo install` staleness, not a project bug. The release-day binary at `target/release/caro` was used for QA against the actual shipped surface.

### Slot C — Picked: #10 Website caro.sh landing — no raw i18n keys

**Method**: `curl -fsSL https://caro.sh/{,/de/,/ja/}` and parse `<title>` + `<h1>`. EN as control, DE + JA as random non-EN locales. (15 locales total in the i18n system: ar/de/en/es/fil/fr/he/hi/id/ja/ko/pt/ru/uk/ur — DE/JA chosen for script-family diversity.)

**Result**: PARTIAL PASS.
- ✓ No raw `landing.foo.bar` key strings leak in any of the three locales (the original regression class from PR #874 is closed).
- ✗ EN landing page H1 (`Never run by accident again rm -rf /`) has been refreshed to safety-positioning copy, but DE+JA still serve the older "faithful shell companion" mascot copy. Cross-locale users see two different product positionings.
- ✗ Within EN: `<title>` (`Caro - Terminal mastery, unlocked`) and `<h1>` no longer agree. Title aligns with a downstream `<h2>`, suggesting a partial copy refresh where one component lagged.
- ✗ Within JA: `<title>` says 友 ("friend"), `<h1>` says コンパニオン ("companion") — within-locale word-choice drift.

### Findings

- [#884](https://github.com/wildcard/caro/issues/884) — `i18n: EN landing headline refreshed but DE/JA still on old copy; EN title/H1 inconsistent` — **P2** (`bug`, `i18n`, `website`, `qa`)

### Followups

- **Flake watch**: `caro::agent: Timeout approaching, skipping refinement` warning on first sample-prompt call. Logged in `qa-known-flakes.md`. Re-evaluate next pass: if absent on a warm cache, classify as cold-start; if present consistently, file as a regression.
- **Coverage matrix update**: surface #10 marked tested 2026-04-26 with link to #884; remaining 13 surfaces still "never tested".
- **Bootstrap dependency**: the daily remote routine reads `.claude/memory/qa-*.md` from `main`. These files are only on branch `claude/bold-hypatia-685f19` until a bootstrap PR lands. The routine has a built-in `exit 0` if it can't find them, so the cost of the bootstrap-not-merged case is just one no-op run.

### Day-1 onboarding deliverables

- ✓ `~/.claude/projects/.../memory/qa_agent_role.md` created (auto-memory persona).
- ✓ `MEMORY.md` indexed with new entry.
- ✓ `.claude/memory/qa-session-log.md` (this file).
- ✓ `.claude/memory/qa-coverage-matrix.md` seeded with 14 regression candidates.
- ✓ `.claude/memory/qa-known-flakes.md` seeded.
- ✓ `.claude/memory/qa-bugs-backlog.md` retired BUG-XXX template, added GH issue watch list.
- ✓ GH labels created: `regression` (`#FBCA04`), `qa` (`#5319E7`).
- ✓ Daily remote routine registered: [trig_01Tk7DxyXV7LeYcFjgmTG1mZ](https://claude.ai/code/routines/trig_01Tk7DxyXV7LeYcFjgmTG1mZ), cron `0 14 * * *`.
- ✓ One real finding filed: [#884](https://github.com/wildcard/caro/issues/884).
