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

### BW-002: CLAUDE.md version/MSRV drift, filed 40× (canonical + 39 duplicates)

**Found:** 2026-09-27, PR #1470 session
**Issue:** canonical #1098 (oldest open); 39 duplicates, see "Dedup pending" below
**Status:** claimed-by integrator/20260903 (#1432). #1478 carries an overlapping
fix; both are open and unmerged.
**Needs human?:** no
**Next step:** merge one of #1432 / #1478, then close the #1098 row of the
"Dedup pending" table.
Root cause: the daily QA routine (`trig_01Tk7DxyXV7LeYcFjgmTG1mZ`, 14:00 UTC)
re-files instead of commenting on the existing issue, and nothing fixes.

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
**Issue:** #1272 (11 duplicates, see "Dedup pending")
**Status:** open. PR #1415 (init status feedback) may cover part of it.
**Needs human?:** no
**Next step:** check #1415; else add a timeout plus a clear error.

### BW-005: static matcher Pattern 43 drops "list files in current directory"

**Found:** 2026-09-30 sweep dedup
**Issue:** #1181 (dups #1274, #1362, #1396, #1399, #1412)
**Status:** fixed (#1487, merged 2026-10-01)
**Needs human?:** no
**Next step:** close the #1181 row of "Dedup pending".

### BW-006: embedded CPU stub always returns `echo 'Please clarify your request'`

**Found:** 2026-09-30 sweep dedup
**Issue:** #1269 (11 duplicates). Root cause: the stub keyword-matches "rm" in the system prompt (`src/backends/embedded/cpu.rs`).
**Status:** open
**Needs human?:** yes. Product call: fail with a clear error on the CPU variant, or fall back to the static matcher. Epic #1460 / #1462 also touch this path.
**Next step:** maintainer picks the behaviour.

### BW-007: placeholder command reported with fake confidence=0.85 / risk=Safe

**Found:** 2026-09-30 sweep dedup
**Issue:** #1281 (dups #1361, #1421, #1424)
**Status:** open
**Needs human?:** yes. It goes together with the BW-006 decision.
**Next step:** fix after BW-006 is decided.

### BW-008: `config show` lists keys that `config get/set` reject

**Found:** 2026-09-30 sweep dedup
**Issue:** #1216 (dups #1286, #1330, #1380, #1457)
**Status:** claimed-by sweep/2026-10-01-BW-009 (#1497)
**Needs human?:** no
**Next step:** make get/set accept every key that show prints (log_level, cache_max_size, log_rotation, telemetry).

### BW-009: first-run consent advertises invalid `config set telemetry.enabled false`

**Found:** 2026-09-30 sweep dedup
**Issue:** #1177 (dups #1292, #1332, #1403)
**Status:** claimed-by sweep/2026-10-01-BW-009 (#1497)
**Needs human?:** no
**Next step:** accept the key, or fix the consent text. Probably fixed together with BW-008.

### BW-010: CLI test runner falls back to ambiguous `cargo run` (multi-binary)

**Found:** 2026-09-30 sweep dedup
**Issue:** #1222 (dups #1252, #1413)
**Status:** open
**Needs human?:** no
**Next step:** add `--bin caro` to the fallback, or set `default-run` in Cargo.toml (#1164).

### BW-011: `caro ai --once` has no static-matcher first pass

**Found:** 2026-09-30 sweep dedup
**Issue:** #1179 (dup #1387)
**Status:** open
**Needs human?:** no
**Next step:** route `ai --once` through the same static-first chain as the top-level command.

### BW-012: `caro ai` rejects `-p/--prompt` while its error text suggests it

**Found:** 2026-09-30 sweep dedup
**Issue:** #1213 (dup #1422)
**Status:** open
**Needs human?:** no
**Next step:** accept `-p`, or fix the message.

### BW-013: `caro config reset <key>` unsupported

**Found:** 2026-09-30 sweep dedup
**Issue:** #1260 (dup #1381)
**Status:** open
**Needs human?:** no
**Next step:** add per-key reset.

### BW-014: `--output json` reports `executed: true` under `--dry-run`

**Found:** 2026-09-30 sweep dedup
**Issue:** #1217 (dup #1417)
**Status:** open
**Needs human?:** no
**Next step:** set `executed` from the real execution path.

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
**Status:** open
**Needs human?:** no (CI config). Pin a published action, or remove the job.
**Next step:** confirm which all-contributors action and version the repo means to use.

### BW-018: Claude Code plugin marketplace.json shape / install one-liner likely stale

**Found:** 2026-10-01 sweep (nightly-discovery)
**Issue:** #1490
**Status:** open
**Needs human?:** no
**Next step:** check the current plugin marketplace schema and `/plugin install` syntax, then update the docs and marketplace.json.

### BW-019: Lint & Format red on every PR since Rust 1.99 clippy

**Found:** 2026-10-01 (while driving #1497)
**Issue:** #1498. Clippy 1.99 `double_must_use` (via `#[async_trait]`) and `redundant_field_names` (via `thiserror` `#[from] source`), 27 errors, none in changed code.
**Status:** open
**Needs human?:** yes (CI gate / lint policy). Options: pin the lint toolchain to 1.98.1, add a dated crate-level `allow`, or bump async-trait (syn 3; check MSRV).
**Next step:** maintainer picks an option on #1498.

---

## Dedup pending (2026-09-30)

The sweep could not close these duplicates: the session's permission classifier
refused issue comment and close writes. A maintainer, or a run with that
permission, should close each one as a duplicate of its canonical issue.

| Canonical | Duplicates |
|---|---|
| #1098 | #1495 #1214 #1215 #1271 #1283 #1288 #1319 #1335 #1359 #1366 #1368 #1372 #1376 #1383 #1385 #1388 #1391 #1395 #1397 #1398 #1401 #1405 #1407 #1411 #1414 #1416 #1420 #1425 #1426 #1427 #1431 #1434 #1442 #1444 #1450 #1456 #1469 #1474 #1476 #1483 |
| #1272 | #1290 #1295 #1384 #1393 #1404 #1418 #1449 #1455 #1458 #1468 #1484 |
| #1269 | #1494 #1277 #1289 #1334 #1355 #1360 #1375 #1382 #1406 #1410 #1430 #1473 |
| #1281 | #1361 #1421 #1424 |
| #1181 (closed by #1487) | #1274 #1362 #1396 #1399 #1412 |
| #1216 | #1286 #1330 #1380 #1457 |
| #1177 | #1292 #1332 #1403 |
| #1222 | #1252 #1413 |
| #1179 | #1387 |
| #1213 | #1422 |
| #1260 | #1381 |
| #1217 | #1417 |

Left open on purpose, because they are related but not the same defect: #1400
(Pattern 43 plus the CPU stub together), #1408 (init feedback), #1446 vs #1472
(vulnerability vs upgrade plan), and the `--backend-info` list family
(#1221 and others). No member of that family is in the newest 100 issues.
