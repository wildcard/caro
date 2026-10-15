# QA Coverage Matrix

**Last updated**: 2026-10-15

This file drives Slot C surface selection. Pick the row with the oldest 'Last tested' value (treat 'never' as oldest). Tie-break by lowest surface # (see Notes for exceptions).

---

## Smoke (Slot A) coverage

One row per pass. Update 'Last tested' column after every Slot A run.

| Date | Build | --version | --help | doctor | dry-run | Notes |
|------|-------|-----------|--------|--------|---------|-------|
| 2026-05-07 | PASS | PASS (1.3.0) | PASS | PASS | FLAKE | Model download blocked in sandbox (see flakes); first bootstrap run |
| 2026-05-23 | PASS | PASS (1.4.0) | PASS | PASS | FLAKE | FLAKE-001 reproduced (2nd total, 16-day gap; 7-day window clean); new --edit flag present |
| 2026-05-24 | PASS | PASS (1.4.0) | PASS | PASS | FLAKE | FLAKE-001 2nd occurrence; v1.4.0 confirmed shipped |
| 2026-05-25 | PASS | PASS (1.4.0) | PASS | PASS | FLAKE | FLAKE-001 2nd occurrence; #1044 verified closed; Slot B revealed P1 regression (#1169) |
| 2026-05-26 | PASS | PASS (1.4.0) | PASS | PASS | FLAKE | FLAKE-001 (2nd logged); static matcher confirmed working via --edit --dry-run |
| 2026-05-27 | PASS | PASS (1.4.0) | PASS | PASS | FLAKE | FLAKE-001 reproduced (2nd occurrence); `--output json` and `suggest` both PASS via static backend |
| 2026-06-13 | PASS | PASS (1.4.0) | PASS | PASS | FLAKE | FLAKE-001 reproduced (2nd obs, new 7-day window); 568 lib tests pass |
| 2026-07-11 | PASS | PASS (1.4.0) | PASS | PASS | FLAKE | FLAKE-001 2nd observation; telemetry banner on fresh sandbox then model download hang |
| 2026-07-12 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE/PARTIAL | Model download ~60s; 30s timeout killed process; 90s succeeded but wrong output (CPU stub #1269); second FLAKE-001 proximity observation |
| 2026-07-16 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001 recurrence (2nd total obs, not in 7-day window); MSRV 1.85; 34 safety tests |
| 2026-07-21 | PASS | PASS (1.5.0) | PASS | PASS | DEGRADED | CPU stub fallback returned placeholder — P1 bug #1361; static matcher miss — P2 bug #1362 |
| 2026-07-22 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE/BUG | Model downloaded ([size omitted]) but agent loop timed out; all runs return echo clarify (CPU stub bug #1277) |
| 2026-07-24 | PASS | PASS (1.5.0) | PASS | PASS | FAIL (known P1 #1277) | CPU placeholder misfires on non-static-matched query |
| 2026-07-25 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001: 2nd occurrence (from PR #1373, unmerged) |
| 2026-08-01 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001 reproduced (2nd observation); 597 lib tests PASS |
| 2026-09-06 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001 2nd observation; behavior changed to silent hang (regression filed #1440) |
| 2026-09-07 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001: 3rd occurrence (from PR #1443, unmerged) |
| 2026-09-08 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | Telemetry consent blocked first prompt invocation in fresh sandbox (env limitation) |
| 2026-09-12 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001 reproduced (2nd total); 34/34 safety tests pass including PR #1315 evasion fix |
| 2026-09-24 | PASS | PASS (1.5.0) | PASS | PASS | FAIL | Model downloaded ([size omitted]), but CPU stub generates wrong cmd for all prompts — see #1473 |
| 2026-09-26 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001: 4th occurrence (from PR #1477, unmerged) |
| 2026-09-27 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001: model download blocked in sandbox; 5th occurrence |
| 2026-09-28 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001 (model download, 2nd occurrence); `list files` static matcher works; `list files in current directory` falls through |
| 2026-09-29 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001 occurrence #2; 597 lib tests all pass |
| 2026-09-30 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001 again (EXIT:124, 30s timeout post-telemetry consent); proxy detected at 127.0.0.1:41305 |
| 2026-10-01 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher handled dry-run (ls -la); no model download needed; telemetry consent shown on first run |
| 2026-10-02 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher returned `ls -la` immediately; model auto-downloaded during background test run |
| 2026-10-03 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher handled dry-run (no FLAKE-001 this pass) |
| 2026-10-04 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher returned ls -la; model found in cache by Slot C (mtime 14:16); FLAKE-001 did not reproduce |
| 2026-10-05 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher returned ls -la; telemetry consent re-prompted (fresh container, expected) |
| 2026-10-06 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher used for dry-run; FLAKE-001 not triggered; explain PASS |
| 2026-10-07 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static backend used for dry-run (ls -la); no model download needed |
| 2026-10-08 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher fallback in 1.5.0 resolves previous dry-run FLAKE |
| 2026-10-09 | PASS | PASS (1.5.0) | PASS | PASS | PASS | dry-run via static matcher (Pattern 43); model downloaded to cache this run |
| 2026-10-10 | PASS | PASS (1.5.0) | PASS | PASS | PASS | static-matcher intercepts basic patterns; no model download needed. gh auth failed — no issue filing this pass |
| 2026-10-11 | PASS | PASS (1.5.0) | PASS | PASS | PASS | --dry-run now PASS via static matcher; telemetry consent on fresh sandbox (expected) |
| 2026-10-13 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher handled dry-run without model; telemetry prompt appeared (fresh container) |
| 2026-10-15 | PASS | PASS (1.5.0) | PASS | PASS | PASS | Static matcher answered dry-run; gh token invalid, MCP used |

---

## Feature surface table

Slot C selects from this table. Update 'Last tested', 'Result', and 'Linked issue(s)' after each Slot C exercise.

| # | Surface | Domain | Last tested | Result | Linked issue(s) |
|---|---------|--------|-------------|--------|-----------------|
| 1 | CLI smoke (build, --version, --help, doctor) | cli | 2026-10-11 | PASS | — |
| 2 | `caro -p "..." --dry-run` command generation | cli | 2026-10-11 | PASS | — |
| 3 | Telemetry consent persistence across invocations | cli | 2026-10-11 | PASS | — |
| 4 | `caro shell-init bash/zsh/fish` | shell-integration | 2026-05-07 | PASS | — |
| 5 | `caro init` setup wizard (--minimal, --force) | cli | 2026-05-07 | PASS | — |
| 6 | Safety validation unit tests (cargo test safety) | safety | 2026-10-10 | PASS | — (38 tests PASS in Slot B, up from 19 in May; fix(safety) #1535) |
| 7 | Safety CVE patterns (ruleset load, shell filters) | safety | 2026-10-08 | PASS | — |
| 8 | Full library test suite (cargo test --lib) | cli | 2026-09-07 | PASS | — |
| 9 | CaroML: `caro new / check / list / jobs` | cli | 2026-05-07 | PASS | — |
| 10 | `caro ai --once` scripted conversational mode | ai | 2026-10-15 | FAIL | #1269 (CPU stub; #1673 is its duplicate), #1272, #1408 (open) |
| 11 | `caro ai --continue-session` shell widget | ai | never | — | Likely affected by same bug as #1449; test after fix. The `?` widget cannot take a request (#1671) |
| 12 | `caro assess` system assessment | cli | N/A | N/A | (cmd disabled/commented-out since v1.1.0-beta.1; excluded from Slot C selection) |
| 13 | `caro suggest` command suggestions | cli | 2026-09-27 | PASS | — |
| 14 | `caro config get/set/show/reset` | cli | 2026-10-10 | PARTIAL | — (get/set verified for safety + log_level keys in Slot B via fix(cli) #1497; show/reset not tested) |
| 15 | `caro --output json` format correctness | cli | 2026-10-10 | PASS | — (--dry-run --output json shows dry_run:true field; Slot B via fix(cli) #1532) |
| 16 | `caro --output yaml` format correctness | cli | never | — | — |
| 17 | `caro completion bash/zsh/fish` | shell-integration | never | — | — |
| 18 | `caro test --backend static` eval harness | cli | 2026-07-25 | PASS | (from prior session PR #1373, unmerged) |
| 19 | Embedded model backend command quality | embedded | never | — | [#1523](https://github.com/wildcard/caro/issues/1523) (expected FAIL: same cpu.rs:63 bug as surface #10) |
| 20 | Ollama backend (requires ollama installed) | ollama | never | — | — |
| 21 | CaroML: `caro run / generate / render / history` | cli | never | — | — |
| 22 | CaroML: `caro experiment / adopt / why` | cli | never | — | — |
| 23 | CaroML: `caro do` Carofile job runner | cli | never | — | — |
| 24 | `caro skill install` (bundled skill management) | cli | never | — | — |
| 25 | Website homepage caro.sh (curl + parse) | website | 2026-10-11 | PASS | — |
| 26 | Website docs pages (curl + parse) | docs | never | — | — |
| 27 | Install script `scripts/install.sh` | install | never | — | — |
| 28 | Homebrew tap formula | install | never | — | — |
| 29 | `caro --safety strict/moderate/permissive` modes | safety | never | — | — |
| 30 | `caro --verbose` timing output | cli | 2026-10-10 | PASS | — (--dry-run --verbose confirmed Backend: static-matcher, Confidence: 1.00 in Slot A) |
| 31 | i18n website locale smoke (curl /es/, /fr/, /ja/) | i18n | 2026-09-07 | PASS | — |
| 32 | `caro doctor` advisory content accuracy | cli | 2026-10-13 | PASS | — |
| 33 | `caro ai` decision gates (clarification, Jev gap, constrained decoding) (#1459) | ai | never | — | — |
| 34 | Executor timeout / guard hooks / loop budget enforcement (#1470) | cli | never | — | — |
| 35 | `caro export` platform runbook writer (new in v1.5.0) | cli | never | — | — |

---

## Surfaces added by bug filings

When a filed issue reveals a new surface gap, add it here so Slot C tracks it in a future pass.

| Issue | Surface | Domain | Filed | Status |
|-------|---------|--------|-------|--------|
| [#1221](https://github.com/wildcard/caro/issues/1221) | `--backend-info` stale backend list | cli | 2026-06-13 | open |
| [#1222](https://github.com/wildcard/caro/issues/1222) | `e2e_cli_tests` CliTestRunner fallback | cli | 2026-06-13 | closed |
| [#1319](https://github.com/wildcard/caro/issues/1319) | CLAUDE.md version field alignment | docs | 2026-07-12 | closed (duplicate) |
| [#1372](https://github.com/wildcard/caro/issues/1372) | CLAUDE.md version drift (1.4.0 vs 1.5.0) | docs | 2026-07-25 | closed (duplicate) |
| [#1449](https://github.com/wildcard/caro/issues/1449) | caro ai --once silent hang when model unavailable | ai | 2026-09-12 | closed (duplicate) |
| [#1450](https://github.com/wildcard/caro/issues/1450) | CLAUDE.md version field re-drift (1.4.0 vs 1.5.0) | docs | 2026-09-12 | closed (duplicate) |
| [#1375](https://github.com/wildcard/caro/issues/1375) | `caro ai --once` CpuBackend placeholder on Linux x86_64 | ai | 2026-07-26 | closed (duplicate) |
| pending (gh auth failed) | `caro ai --once` silent hang when no model | ai | 2026-10-10 | duplicate of #1408 and #1179 |
| [#1044](https://github.com/wildcard/caro/issues/1044) | CLAUDE.md version field alignment | docs | 2026-05-07 | closed |
| [#1165](https://github.com/wildcard/caro/issues/1165) | Allowlist bypass broken for rm -rf /tmp/... by Critical pre-scan | safety | 2026-05-23 | open |
| [#1098](https://github.com/wildcard/caro/issues/1098) | CLAUDE.md version drift (shows 1.4.0, Cargo.toml is 1.5.0) | docs | 2026-05-07 | closed |
| — | PR #1112 `caro.prune` skill | cli | 2026-05-24 | flagged for future Slot C |
| [#1169](https://github.com/wildcard/caro/issues/1169) | test_allowlist_functionality broken by PR #1110 Critical pre-scan | safety | 2026-05-25 | closed (duplicate) |
| [#1170](https://github.com/wildcard/caro/issues/1170) | cargo test safety fails — evaluation binary rejects positional filter | cli | 2026-05-25 | closed (duplicate) |
| [#1176](https://github.com/wildcard/caro/issues/1176) | Safety allowlist: rm -rf /tmp pattern false positive | safety | 2026-05-27 | closed (duplicate) |
| [#1177](https://github.com/wildcard/caro/issues/1177) | `caro config set telemetry.enabled false` not implemented | cli | 2026-05-27 | closed |
| [#1292](https://github.com/wildcard/caro/issues/1292) | `caro config set telemetry.enabled false` invalid key | cli | 2026-07-11 | closed (duplicate) |
| [#1293](https://github.com/wildcard/caro/issues/1293) | `--backend-info` wrong backend list | cli | 2026-07-11 | closed (duplicate) |
| [#1294](https://github.com/wildcard/caro/issues/1294) | `--backend` help text missing mesh/ai-horde/hybrid | cli | 2026-07-11 | closed (duplicate) |
| [#1295](https://github.com/wildcard/caro/issues/1295) | `caro ai --once` silent hang, no model | ai | 2026-07-11 | closed (duplicate) |
| [#1332](https://github.com/wildcard/caro/issues/1332) | `config set/get` telemetry.enabled not supported | cli | 2026-07-16 | closed (duplicate) |
| [#1361](https://github.com/wildcard/caro/issues/1361) | CPU backend stub keyword matching on full prompt | ai | 2026-07-21 | closed (duplicate) |
| [#1362](https://github.com/wildcard/caro/issues/1362) | Static matcher Pattern 43 regex over-anchored | cli | 2026-07-21 | closed (duplicate) |
| [#1366](https://github.com/wildcard/caro/issues/1366) | CLAUDE.md version shows 1.4.0 instead of 1.5.0 | docs | 2026-07-22 | closed (duplicate) |
| [#1387](https://github.com/wildcard/caro/issues/1387) | `caro ai --once` static matcher bypass | ai | 2026-08-01 | closed (duplicate) |
| [#1388](https://github.com/wildcard/caro/issues/1388) | CLAUDE.md version drift (1.4.0→1.5.0) | docs | 2026-08-01 | closed (duplicate) |
| [#1440](https://github.com/wildcard/caro/issues/1440) | HTTP timeout on embedded model download (`ModelLoader` / `hf_hub` API) | embedded | 2026-09-06 | closed |
| [#1442](https://github.com/wildcard/caro/issues/1442) | CLAUDE.md version drift 1.4.0 vs 1.5.0 | docs | 2026-09-07 | closed (duplicate) |
| [#1444](https://github.com/wildcard/caro/issues/1444) | CLAUDE.md version + MSRV drift after v1.5.0 | docs | 2026-09-08 | closed (duplicate) |
| [#1473](https://github.com/wildcard/caro/issues/1473) | CPU embedded backend always returns "clarify" (system-prompt contamination) | embedded | 2026-09-24 | closed (duplicate) |
| [#1474](https://github.com/wildcard/caro/issues/1474) | CLAUDE.md version 1.4.0 vs shipped 1.5.0 — regression of #1044 | docs | 2026-09-24 | closed (duplicate) |
| [#1483](https://github.com/wildcard/caro/issues/1483) | CLAUDE.md version banner regression (1.4.0 vs 1.5.0) | docs | 2026-09-29 | closed (duplicate) |
| [#1484](https://github.com/wildcard/caro/issues/1484) | caro ai --once silent hang — no progress when no backend | ai | 2026-09-29 | closed (duplicate) |
| [#1494](https://github.com/wildcard/caro/issues/1494) | `caro ai --once` CPU backend always returns deletion-clarification | ai | 2026-10-01 | closed (duplicate) |
| [#1495](https://github.com/wildcard/caro/issues/1495) | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (regression) | docs | 2026-10-01 | closed (duplicate) |
| [#1499](https://github.com/wildcard/caro/issues/1499) | `caro ai --once` hangs in non-TTY (stdin read blocks) | ai | 2026-10-02 | closed |
| [#1500](https://github.com/wildcard/caro/issues/1500) | CPU stub returns misleading clarification for all queries | embedded | 2026-10-02 | closed (duplicate) |
| [#1501](https://github.com/wildcard/caro/issues/1501) | CLAUDE.md version banner drift (1.4.0 not 1.5.0) | docs | 2026-10-02 | closed (duplicate) |
| [#1520](https://github.com/wildcard/caro/issues/1520) | CLAUDE.md version field alignment | docs | 2026-10-03 | closed (duplicate) |
| [#1269](https://github.com/wildcard/caro/issues/1269) | CPU stub rm-in-system-prompt false positive (all non-static queries fail) | embedded | 2026-06-26 | open |
| [#1248](https://github.com/wildcard/caro/issues/1248) | `caro ai` silent hang in non-TTY | ai | pre-2026-10-05 | open |
| [#1530](https://github.com/wildcard/caro/issues/1530) | config set backend no auto-detect option | cli | 2026-10-06 | open |
| [#1540](https://github.com/wildcard/caro/issues/1540) | ai --once silent hang when no backend ready | ai | 2026-10-08 | closed (duplicate) |
| [#1541](https://github.com/wildcard/caro/issues/1541) | CLAUDE.md shows 1.4.0 instead of 1.5.0 | docs | 2026-10-08 | closed (duplicate) |
| [#1603](https://github.com/wildcard/caro/issues/1603) | QA memory files stuck at bootstrap — 25+ update PRs unmerged | docs/ops | 2026-10-11 | open |
| [#1673](https://github.com/wildcard/caro/issues/1673) | ai --once deletion clarification (CPU stub, duplicate of #1269) | ai | 2026-10-15 | open (duplicate of #1269) |

---

## Notes

- Slot C tie-break: when multiple surfaces share 'never', pick lowest `#` number unless context suggests a riskier surface is more valuable to exercise. Skip surfaces where Result = N/A (disabled/unreachable commands).
- Website surfaces (#25, #26) can be tested with `curl` + Python parsing alone — no caro build needed.
- Surfaces requiring model download (#19, #20) should be tested from an environment with a pre-downloaded model; note in session log if sandbox blocks download.
- Next Slot C candidates (tie on 'never', pick lowest #): surface #11 (`caro ai --continue-session`), surface #12 (`caro assess`).
- Surface #14 (`config get/set/show/reset`): partially passes — `show` and `reset` work; `get` and `set` only support 4 keys (`backend`, `model-name`, `shell`, `safety`); `telemetry.enabled` rejected (P1, see #1332).
- Next Slot C candidate: surface #11 (`caro ai --continue-session`) or #12 (`caro assess`) — both 'never'.
- Surfaces #25/#26 flagged for priority: touched by PRs #1155–#1159 (brand design system) and #1327 (canonical pet domains) merged 2026-07-12 to 2026-07-14.
- **Slot B search**: use perPage:100 and paginate to avoid missing PRs when the window is large (>20 PRs). A 20-result cap silently drops older-in-window PRs from spot-checking.
- **Next Slot C recommendation**: surface #11 (`caro ai --continue-session`) — oldest untested after #10. Alternatively prioritize #25/#31 (website/i18n) given recent PRs #1325/#1351/#1352.
- **Next Slot C candidate**: surface #11 (`caro ai --continue-session`) uses the embedded backend and is testable once #1440 is resolved; prefer surface #12 (`caro assess`) in the meantime.
- `caro.sh` root redirects to `www.caro.sh` (HTTP 307); sandbox proxy blocks redirect target. `/he/` and other locale paths work directly.
- `caro.sh/en/` returns 404 by design: `prefixDefaultLocale: false` in `website/astro.config.mjs` — English lives at `/`.
- P1 bug #1375: `caro ai --once` CPU backend always returns `echo 'Please clarify your request'` on Linux x86_64. Re-test surface #10 after fix.
- Surface #11 (`caro ai --continue-session`) is next Slot C candidate (lowest 'never' after #10 exercised 2026-09-08).
- Surface #31 (i18n locale smoke) elevated priority after #1352 (Hebrew translations overhaul) merged 2026-07-19.
