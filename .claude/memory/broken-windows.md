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

### BW-001: `cargo audit` red on main — h2 0.3.27 (RUSTSEC-2026-0258)

**Found:** 2026-09-24, PR #1470 session
**Issue:** #1472 (reqwest 0.11→0.12 + wiremock 0.5→0.6); lockfile part is #1471
**Status:** fixed (#1488, merged 2026-10-01). reqwest 0.12 / wiremock 0.6.4,
h2 0.4.19, rustls 0.23.45; h2 0.3.27 is gone from default builds. Maintainer
approved a scoped `.cargo/audit.toml` ignore for RUSTSEC-2026-0258 (h2 0.3 only
via the optional `chromadb`/`knowledge` features), review by 2026-12-31.
**Needs human?:** no (decided)
**Next step:** close #1471, #1472 and #1446 as fixed by #1488. Follow-ups:
upgrade lancedb 0.23 → 0.39 and replace the `chromadb` crate (then remove the
ignore); migrate `deny.toml` off keys removed in cargo-deny (it fails to parse,
and its CI step has `continue-on-error`).

### BW-002: CLAUDE.md version/MSRV drift, filed 42× (canonical + 41 duplicates)

**Found:** 2026-09-27, PR #1470 session
**Issue:** canonical #1098 (oldest open); 41 duplicates closed 2026-10-03; new dup #1520 (see "Dedup pending")
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

### BW-005: static matcher Pattern 43 drops "list files in current directory"

**Found:** 2026-09-30 sweep dedup
**Issue:** #1181 (dups #1274, #1362, #1396, #1399, #1412, closed)
**Status:** fixed (#1487, merged 2026-10-01)
**Needs human?:** no
**Next step:** none; duplicates closed 2026-10-03. Delete after 2026-10-08.

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
**Status:** fixed by #1522 (`default-run = "caro"`), pending merge
**Needs human?:** no
**Next step:** none once #1522 merges; delete 7 days after.

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
**Status:** claimed-by sweep/2026-10-06-BW-014 (#1532). Adds a `dry_run` field to the JSON. `executed` keeps its pinned meaning ("passed safety checks"); contract tests depend on it.
**Needs human?:** no
**Next step:** merge the PR.

### BW-015: main CI `Extended Tests` (4 model jobs) red: HF Hub model download fails

**Found:** 2026-09-30 sweep, CI run 36658876598
**Issue:** #1341
**Status:** open
**Needs human?:** no (probably CI caching or network; root-cause first)
**Next step:** the logs show `Failed to download model after 3 attempts` for every e2e test.

### BW-016: 1.5.0 declared in-repo but never tagged or published

**Found:** 2026-09-30 sweep
**Issue:** #1419
**Status:** open
**Needs human?:** yes (release)
**Next step:** maintainer decides whether to tag 1.5.0. This is also the root of the BW-002 churn.

### BW-017: `pr-merged.yml` `add-contributor` job fails on every merge

**Found:** 2026-10-01 sweep
**Issue:** #1491 (action `all-contributors/add-contributor` not found)
**Status:** claimed-by sweep/2026-10-05-BW-017 (#1527). The job is removed: the action repo does not exist and the repo has no `.all-contributorsrc`.
**Needs human?:** no (CI config). Pin a published action, or remove the job.
**Next step:** merge the removal PR. Re-add a working all-contributors job only if the maintainer wants a contributors list.

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
**Issue:** #1221 (15 duplicates, see "Dedup pending")
**Status:** open
**Needs human?:** no
**Next step:** derive `--backend-info`, the `--backend` help text and the "Unknown backend" error from one list, filtered by compiled features.

### BW-021: user allowlist cannot override the Critical `rm -rf` pre-scan (regression from #1110)

**Found:** 2026-10-03 sweep dedup
**Issue:** #1165 (5 duplicates, see "Dedup pending"); `test_allowlist_functionality` fails
**Status:** open
**Needs human?:** yes (safety pattern / allowlist policy)
**Next step:** maintainer decides whether user allowlists may bypass Critical patterns, then fix code or the contract test via `safety-pattern-developer`.

### BW-022: `cargo test safety` (documented in CLAUDE.md) fails

**Found:** 2026-10-03 sweep dedup
**Issue:** #1162 (dup #1170)
**Status:** open
**Needs human?:** no
**Next step:** make the evaluation harness accept a positional filter, or document `cargo test --lib safety`.

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

---

## Dedup pending (2026-10-03)

The 2026-09-30 table was cleared on 2026-10-03 (86 duplicates closed with the
maintainer's approval). The groups below were found on 2026-10-03; the sweep's
issue-close writes were refused again, so a maintainer (or an approved run)
should close each as a duplicate of its canonical issue.

| Canonical | Duplicates |
|---|---|
| #1098 | #1520 |
| #1221 | #1247 #1250 #1251 #1256 #1257 #1258 #1268 #1270 #1278 #1284 #1287 #1291 #1293 #1294 #1392 |
| #1165 | #1169 #1176 #1201 #1204 #1205 |
| #1162 | #1170 |
| #1163 | #1328 |

Left open on purpose, because they are related but not the same defect: #1400
(Pattern 43 plus the CPU stub together), #1408 (init feedback, vs #1272),
#1267 (TTY hang), #1446 vs #1472 (vulnerability vs upgrade plan), #1164 vs
#1222 (same root cause, fixed together by BW-010), #1183/#1203/#1508 (missing
waitlist/playbook locale files; #1508 is the CI symptom of both).
