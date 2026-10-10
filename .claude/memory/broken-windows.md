# Broken Windows — Handoff Register

Things an agent found broken **outside its own task**. The finder records the
problem here and moves on. A fixer claims an entry and fixes it. The fixer is
either the **caro broken-window sweep** routine (`trig_01TAiuoMoZ6nofW3ceYL3Csy`,
daily 15:30 UTC) or any agent already working in that area.

Rules (see `.claude/rules/good-boy-scout.md` → "Stay in your lane"):

- **Found one?** Search open issues first. If none exists, file one, then add
  an entry here that links it. Never spend your own task on it.
- **Claiming:** set `Status: claimed-by <branch>` before starting, and fix it
  in its own PR (not bundled into unrelated work).
- **Needs human: yes** means a security-policy, release or product decision
  (audit suppressions, safety patterns, dependency majors). Don't apply the
  change yourself; the issue is the ask.
- **Fixed:** set `Status: fixed (<PR>)` and leave the entry for a week so
  finders can see it's done, then delete it.

---

### BW-002: CLAUDE.md version/MSRV drift, filed 42× (canonical + 41 duplicates)

**Found:** 2026-09-27, PR #1470 session
**Issue:** canonical #1098 (oldest open); 41 duplicates closed 2026-10-03; dups #1520, #1541 closed 2026-10-09
**Status:** claimed-by integrator/20260903 (#1432). #1478 carries an overlapping
fix; both are open and unmerged.
**Needs human?:** no
**Next step:** merge one of #1432 / #1478.
Root cause: the daily QA routine (`trig_01Tk7DxyXV7LeYcFjgmTG1mZ`, 14:00 UTC)
re-files instead of commenting on the existing issue. A "search before filing"
edit was drafted 2026-10-03; only the maintainer can apply it (the routine was
created via the API, so agents cannot update it).

### BW-003: stale harness references (caro-eval, current-tasks.md, AGENTS.md, .kittify, --skill)

**Found:** 2026-09-24, google/ax harness survey
**Issue:** #1479
**Status:** open. #1478 fixes the `current-tasks.md` row.
**Needs human?:** no
**Next step:** see the issue table. Each row is a one-line docs/config fix.
Remaining: `caro-eval` (use `cargo test --test evaluation`), `AGENTS.md` line in
dev-process.md, `.kittify` helpers, `claude --skill` in modulization.yml,
hardcoded `/Users/kobik-private` paths.

### BW-004: `caro ai --once` hangs silently when no backend/model is ready

**Found:** 2026-09-30 sweep dedup
**Issue:** #1272 (11 duplicates closed 2026-10-03)
**Status:** open. Stdin half fixed (#1503, merged 2026-10-03; closed #1499). Two causes,
reproduced 2026-10-02: (1) `--once` read stdin even with a trailing prompt, so an
open pipe (scripts, CI) blocked forever (#1499); (2) the first run silently
downloads the ~1 GB default model, with no progress output (#1272, #1484).
**Needs human?:** no
**Next step:** cause (2) is still open:
PR #1415 only prints a one-time "Initializing backend…" line, not download
progress. Keep #1272 open until first-run download progress (or a clear
"downloading model" message) lands; #1415 is a partial step.

### BW-006: embedded CPU stub always returns `echo 'Please clarify your request'`

**Found:** 2026-09-30 sweep dedup
**Issue:** #1269 (13 duplicates, closed). Root cause: the stub keyword-matches "rm" in the system prompt (`src/backends/embedded/cpu.rs`).
**Status:** open
**Needs human?:** yes. Product call: fail with a clear error on the CPU variant, or fall back to the static matcher. Epic #1460 / #1462 also touch this path.
**Next step:** maintainer picks the behaviour.

### BW-007: placeholder command reported with fake confidence=0.85 / risk=Safe

**Found:** 2026-09-30 sweep dedup
**Issue:** #1281 (dups #1361, #1421, #1424, closed)
**Status:** open
**Needs human?:** yes. It goes together with the BW-006 decision.
**Next step:** fix after BW-006 is decided.

### BW-008: `config show` lists keys that `config get/set` reject

**Found:** 2026-09-30 sweep dedup
**Issue:** #1216 (dups #1286, #1330, #1380, #1457, closed)
**Status:** fixed (#1497, merged 2026-10-03)
**Needs human?:** no
**Next step:** none; get/set accept every `config show` key. Delete after 2026-10-10.

### BW-009: first-run consent advertises invalid `config set telemetry.enabled false`

**Found:** 2026-09-30 sweep dedup
**Issue:** #1177 (dups #1292, #1332, #1403, closed)
**Status:** fixed (#1497, merged 2026-10-03)
**Needs human?:** no
**Next step:** none; `telemetry.enabled` is accepted and clears first-run. Delete after 2026-10-10.

### BW-010: CLI test runner falls back to ambiguous `cargo run` (multi-binary)

**Found:** 2026-09-30 sweep dedup
**Issue:** #1222 (dups #1252, #1413, closed); same root cause as #1164
**Status:** fixed (#1522, merged 2026-10-09)
**Needs human?:** no
**Next step:** none. Delete after 2026-10-16.

### BW-011: `caro ai --once` has no static-matcher first pass

**Found:** 2026-09-30 sweep dedup
**Issue:** #1179 (dup #1387, closed)
**Status:** open
**Needs human?:** no
**Next step:** route `ai --once` through the same static-first chain as the top-level command.

### BW-012: `caro ai` rejects `-p/--prompt` while its error text suggests it

**Found:** 2026-09-30 sweep dedup
**Issue:** #1213 (dup #1422, closed)
**Status:** open
**Needs human?:** no
**Next step:** accept `-p`, or fix the message.

### BW-013: `caro config reset <key>` unsupported

**Found:** 2026-09-30 sweep dedup
**Issue:** #1260 (dup #1381, closed)
**Status:** open
**Needs human?:** no
**Next step:** add per-key reset.

### BW-014: `--output json` reports `executed: true` under `--dry-run`

**Found:** 2026-09-30 sweep dedup
**Issue:** #1217 (dup #1417, closed)
**Status:** fixed (#1532, merged 2026-10-10). Adds a `dry_run` field to the JSON. `executed` keeps its pinned meaning ("passed safety checks"); contract tests depend on it.
**Needs human?:** no
**Next step:** none. Delete after 2026-10-17.

### BW-015: main CI `Extended Tests` (4 model jobs) red: HF Hub model download fails

**Found:** 2026-09-30 sweep, CI run 36658876598
**Issue:** #1341
**Status:** open. Root-caused 2026-10-09 (nightly run 37873894155). Two causes, 9 of 29 e2e tests fail in each job:
(1) SmolLM 135M and StarCoder 1B: their `src/model_catalog.rs` repos (`HuggingFaceTB/SmolLM-135M-Instruct-GGUF`,
`TheBloke/starcoderbase-1b-GGUF`) no longer exist on Hugging Face (anonymous API returns 401), so every download fails.
PR smoke tests stay green only because the model cache restores an old file. The download error text also tells
users to `export CARO_MODEL=smollm-135m-q4`, which cannot download.
(2) Qwen 0.5B and TinyLlama 1.1B download, then abort in llama.cpp Metal: `GGML_ASSERT ggml-metal.m:870 "unsupported op"`
(`llama_cpp = "0.3"`, `n_gpu_layers: 99` in `src/backends/embedded/mlx.rs`).
**Needs human?:** yes. (1) Pick replacement repos (for example `HuggingFaceTB/smollm-135M-instruct-v0.2-Q8_0-GGUF`, which bundle.yml uses), or drop the models. (2) Bump llama_cpp, or run CI with CPU layers only.
**Next step:** maintainer picks both options on #1341.

### BW-016: 1.5.0 declared in-repo but never tagged or published

**Found:** 2026-09-30 sweep
**Issue:** #1419
**Status:** open
**Needs human?:** yes (release)
**Next step:** maintainer decides whether to tag 1.5.0. This is also the root of the BW-002 churn.

### BW-017: `pr-merged.yml` `add-contributor` job fails on every merge

**Found:** 2026-10-01 sweep
**Issue:** #1491 (action `all-contributors/add-contributor` not found)
**Status:** fixed (#1527, merged 2026-10-09). The job is removed: the action repo does not exist and the repo has no `.all-contributorsrc`.
**Needs human?:** no (CI config). Pin a published action, or remove the job.
**Next step:** none. Delete after 2026-10-16. Re-add a working all-contributors job only if the maintainer wants a contributors list.

### BW-018: Claude Code plugin marketplace.json shape / install one-liner likely stale

**Found:** 2026-10-01 sweep (nightly-discovery)
**Issue:** #1490
**Status:** open
**Needs human?:** no
**Next step:** check the current plugin marketplace schema and `/plugin install` syntax, then update the docs and marketplace.json.

### BW-019: Lint & Format red on every PR since Rust 1.99 clippy

**Found:** 2026-10-01 (while driving #1497)
**Issue:** #1498. Clippy 1.99 `double_must_use` (via `#[async_trait]`) and `redundant_field_names` (via `thiserror` `#[from] source`), 27 errors, none in changed code.
**Status:** mitigated (#1505, merged 2026-10-03): Lint & Format and the publish.yml clippy step run on Rust 1.98.1.
**Needs human?:** no (maintainer chose the pin)
**Next step:** remove the pin once clippy or async-trait/thiserror stop linting macro-generated code; then close #1498.

### BW-020: backend lists disagree across `--backend-info`, `--help` and the error text

**Found:** 2026-10-03 sweep dedup
**Issue:** #1221 (15 duplicates, closed 2026-10-09)
**Status:** open
**Needs human?:** no
**Next step:** derive `--backend-info`, the `--backend` help text and the "Unknown backend" error from one list, filtered by compiled features.

### BW-021: user allowlist cannot override the Critical `rm -rf` pre-scan (regression from #1110)

**Found:** 2026-10-03 sweep dedup
**Issue:** #1165 (5 duplicates, closed 2026-10-09); `test_allowlist_functionality` fails
**Status:** open
**Needs human?:** yes (safety pattern / allowlist policy)
**Next step:** maintainer decides whether user allowlists may bypass Critical patterns, then fix code or the contract test via `safety-pattern-developer`.

### BW-022: `cargo test safety` (documented in CLAUDE.md) fails

**Found:** 2026-10-03 sweep dedup
**Issue:** #1162 (dup #1170)
**Status:** fixed (#1548, merged 2026-10-10). The `evaluation` harness (`harness = false`) now accepts libtest's positional filter and skips itself when the filter does not match its name.
**Needs human?:** no
**Next step:** none. Delete after 2026-10-17.

### BW-023: global flags before a subcommand swallow the subcommand

**Found:** 2026-10-03 sweep dedup
**Issue:** #1163 (dup #1328)
**Status:** open
**Needs human?:** no
**Next step:** fix clap routing (`args_conflicts_with_subcommands`) so `caro --no-telemetry config show` runs `config`.

### BW-024: `caro config set backend` has no `auto` value to restore auto-detect

**Found:** 2026-10-06 sweep
**Issue:** #1530
**Status:** open
**Needs human?:** no
**Next step:** accept `auto` (or `default`) in `config set backend` and clear the stored backend.

### BW-025: GNU coreutils misdetected as BSD when `ls --version` exceeds 500 ms

**Found:** 2026-10-06, CI on #1532 (`test_coreutils_detection` failed on ubuntu-24.04)
**Issue:** #1533
**Status:** open
**Needs human?:** no
**Next step:** raise the `ls --version` timeout in `src/platform/mod.rs` to about 2 s; treat a timeout as unknown in `detect_bsd_utils`.

### BW-031: ai_horde `Client-Agent` header hardcoded to `caro:1.4.0`

**Found:** 2026-10-09 sweep register sync (issue filed 2026-09-22)
**Issue:** #1467
**Status:** open (#1454 mentions it)
**Needs human?:** no
**Next step:** build the header from `env!("CARGO_PKG_VERSION")` in `src/backends/remote/ai_horde.rs`.

### BW-032: embedded model download hangs: `HfHubClient` has no HTTP timeout

**Found:** 2026-10-09 sweep register sync (issue filed 2026-09-06)
**Issue:** #1440
**Status:** claimed-by sweep/2026-10-10-BW-032 (#1561). The real path is `ModelLoader` → hf-hub 0.5, whose own client has no timeout (not `HfHubClient`). A progress watchdog fails an attempt after 60 s with no bytes. Each attempt runs on its own runtime, so a stalled attempt's hf-hub chunk tasks are cancelled before the retry. All errors keep the 3-attempt retry, which ends with a clear error.
**Needs human?:** no
**Next step:** merge #1561, then close #1440.

### BW-033: Gemini CLI row in integrations status is stale (product shut down 2026-06-18)

**Found:** 2026-10-09 sweep register sync (issue filed 2026-09-06)
**Issue:** #1439
**Status:** open
**Needs human?:** no
**Next step:** retire the Gemini CLI row in `.claude/memory/integrations-status.md` and add Antigravity CLI.

### BW-034: `caro ai --help` contradicts itself; `--continue-session` is a no-op

**Found:** 2026-10-09 sweep register sync (issue filed 2026-09-05)
**Issue:** #1435 (overlaps #1197, which covers only `--continue-session`)
**Status:** open
**Needs human?:** no
**Next step:** fix the `--once` help text and map `--continue-session` to `SessionMode::ResumeStrict`.

### BW-035: `caro ai --once` shows no feedback during model initialization

**Found:** 2026-10-09 sweep register sync (issue filed 2026-08-15)
**Issue:** #1408 (related to BW-004 / #1272, kept separate)
**Status:** open (#1415 prints a one-time "Initializing backend…" line)
**Needs human?:** no
**Next step:** merge #1415, then add download progress (BW-004 cause 2).

### BW-036: `caro config set backend` rejects mesh, ai-horde, hybrid

**Found:** 2026-10-09 sweep register sync (issue filed 2026-07-27)
**Issue:** #1379 (related: BW-020, BW-024)
**Status:** open. PR #1348 (`integrator/20260718`) unifies the backend rosters; it is mergeable but idle since 2026-09-24. `src/main.rs` still hardcodes `["embedded", "ollama", "exo", "vllm"]`.
**Needs human?:** no
**Next step:** review and merge #1348, or rebase it.

### BW-037: `caro ai --once` hint puts `--execute` after the subcommand

**Found:** 2026-10-09 sweep register sync (issue filed 2026-07-23)
**Issue:** #1369 (same root as BW-023: global flags and subcommand order)
**Status:** open (#1443 mentions it)
**Needs human?:** no
**Next step:** fix the hint text to `caro --execute ai --once`, or accept `--execute` on `ai`.

### BW-038: no config-dir override, so `caro config set` writes the real config in tests

**Found:** 2026-10-09 sweep register sync (issue filed 2026-07-19)
**Issue:** #1349
**Status:** open
**Needs human?:** no
**Next step:** honor a `CARO_CONFIG_DIR` env var in `src/config/mod.rs` and use it in the config tests.

### BW-039: `ChromaDB Integration Tests` CI job fails after ~14 minutes

**Found:** 2026-10-09 sweep register sync (issue filed 2026-07-18)
**Issue:** #1342
**Status:** open
**Needs human?:** yes if the fix is to delete the job (CI gate); no for a root-cause fix.
**Next step:** read the failing step's log and root-cause it.

### BW-040: static matcher "Find Python files (simple)" swallows content searches

**Found:** 2026-10-03 (while fixing "search for TODO in all python files")
**Issue:** #1516. `search for <X> in python files` returns `find . -name "*.py" -type f` for any X other than TODO.
**Status:** fixed (#1517, merged 2026-10-10: content-search handler in `StaticMatcher::content_search_command`)
**Needs human?:** no
**Next step:** none. Delete after 2026-10-17.

### BW-041: an unquoted prompt that starts with "list" runs the `list` subcommand

**Found:** 2026-10-10 sweep register sync (issue filed 2026-10-10, from nightly run 38016340315)
**Issue:** #1550. `caro list files` fails with `unexpected argument 'files'`; 2 e2e tests fail in every Extended Tests job.
**Status:** open
**Needs human?:** yes (CLI design: how a prompt and a subcommand name share the first word)
**Next step:** maintainer picks the parsing rule in #1550; then fix `src/main.rs` routing.

### BW-042: 8 stray `.worktrees/*` gitlinks on main with no `.gitmodules`

**Found:** 2026-10-10 sweep register sync (issue filed 2026-10-10)
**Issue:** #1551. Commit `9599ca7` added 8 local worktree dirs as mode-160000 gitlinks.
**Status:** open
**Needs human?:** no
**Next step:** `git rm --cached` the 8 gitlinks and confirm `.worktrees/` is in `.gitignore`.

### BW-043: CLA check never records a signature, so every external PR stays red

**Found:** 2026-10-10 sweep register sync (issue filed 2026-10-10)
**Issue:** #1555. `custom-pr-sign-comment` in `.github/workflows/cla.yml` holds the whole notice, not the sign phrase. Blocks #1296, #1390, #1415.
**Status:** open
**Needs human?:** yes (CI gate / contributor policy)
**Next step:** maintainer approves the 2-line `cla.yml` change proposed in #1555.

### BW-044: `caro run` runbook path and step path keep shell state differently

**Found:** 2026-10-10 sweep register sync (issue filed 2026-10-10)
**Issue:** #1557. The runbook runs all steps in one `bash`; the step path uses a fresh shell per step.
**Status:** open
**Needs human?:** yes (product: which behavior is the contract)
**Next step:** maintainer picks an option in #1557; recommended option 1 (subshell per step in the runbook).

### BW-045: static safety validator rates reverse shells, exfiltration and DROP DATABASE as Safe

**Found:** 2026-10-10 sweep register sync (issue filed 2026-10-10)
**Issue:** #1559
**Status:** open
**Needs human?:** yes (safety patterns)
**Next step:** maintainer approves the pattern set; then add patterns with the `safety-pattern-developer` TDD flow.

### BW-046: upgrade reqwest 0.11 → 0.12 to clear h2 RUSTSEC-2026-0258 (wiremock is already 0.6.4)

**Found:** 2026-10-10 sweep register sync (issue filed 2026-09-24)
**Issue:** #1472 (related: #1446, the vulnerability report)
**Status:** open
**Needs human?:** yes (dependency major-version bump)
**Next step:** maintainer approves the bump; note that hf-hub 0.5 already pulls reqwest 0.12.

---

## Dedup done (2026-10-09)

The 2026-10-03 and 2026-10-07 tables are closed: 26 duplicates closed on
2026-10-09 (#1520 #1541 → #1098; #1247 … #1392 → #1221; #1169 … #1205 → #1165;
#1170 → #1162; #1328 → #1163; #1523 → #1269; #1540 → #1272), plus #985 → #917.
#1329 closed as fixed by #1503 (guard: `test_needs_stdin_prompt`).

Left open on purpose, because they are related but not the same defect: #1400
(Pattern 43 plus the CPU stub together), #1408 (init feedback, vs #1272),
#1267 (TTY hang), #1446 vs #1472 (vulnerability vs upgrade plan), #1164 vs
#1222 (same root cause, fixed together by BW-010), #1183/#1203/#1508 (missing
waitlist/playbook locale files; #1508 is the CI symptom of both).
