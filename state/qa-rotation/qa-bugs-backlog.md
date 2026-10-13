# QA Bugs Backlog

Active bugs filed by caro-qa-agent requiring investigation and fixes.

---

## Watch list

| Issue | Priority | Domain | Summary | Status | Filed |
|-------|----------|--------|---------|--------|-------|
| [#1221](https://github.com/wildcard/caro/issues/1221) | P1 | cli | --backend-info lists invalid backends (static, claude) and omits valid ones (exo, mesh, ai-horde, hybrid) | open | 2026-06-13 |
| [#1222](https://github.com/wildcard/caro/issues/1222) | P1 | cli | e2e_safety_level_configuration test fails — cargo fallback missing --bin caro with multiple binaries | closed | 2026-06-13 |
| [#1319](https://github.com/wildcard/caro/issues/1319) | P2 | docs | CLAUDE.md version banner shows 1.4.0 (GA) after 1.5.0 release | closed (duplicate) | 2026-07-12 |
| [#1332](https://github.com/wildcard/caro/issues/1332) | P1 | cli | `caro config set telemetry.enabled false` fails — advertised opt-out command not wired in | closed (duplicate) | 2026-07-16 |
| [#1277](https://github.com/wildcard/caro/issues/1277) | P1 | embedded | CPU placeholder misfires on non-static-matched queries via `caro -p` and all queries via `caro ai --once` (system prompt contains "rm", triggers danger branch) | closed (duplicate) | — |
| [#1334](https://github.com/wildcard/caro/issues/1334) | P1 | embedded | `caro ai --once` broken (CPU stub, confirmed v1.5.0) | closed (duplicate) | 2026-07-18 |
| [#1366](https://github.com/wildcard/caro/issues/1366) | P2 | docs | CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0 | closed (duplicate) | 2026-07-22 |
| [#1372](https://github.com/wildcard/caro/issues/1372) | P2 | docs | CLAUDE.md version 1.4.0 vs 1.5.0 (recurrence #1044; duplicate of #1442) | closed (duplicate) | 2026-07-25 (from #1373 branch) |
| [#1387](https://github.com/wildcard/caro/issues/1387) | P1 | ai | `caro ai --once` bypasses static matcher; returns fallback for every prompt when LLM unavailable | closed (duplicate) | 2026-08-01 |
| [#1388](https://github.com/wildcard/caro/issues/1388) | P2 | docs | CLAUDE.md version shows 1.4.0 instead of 1.5.0 (recurrence of #1044) | closed (duplicate) | 2026-08-01 |
| [#1375](https://github.com/wildcard/caro/issues/1375) | P1 | ai | `caro ai --once` CpuBackend always returns placeholder on Linux x86_64 | closed (duplicate) | 2026-07-26 |
| [#1422](https://github.com/wildcard/caro/issues/1422) | P2 | cli | `caro ai --once` rejects global `-p`/`--prompt` flag — inconsistent with top-level usage | closed (duplicate) | 2026-07-26 |
| [#1421](https://github.com/wildcard/caro/issues/1421) | P2 | ai | `caro ai --once` returns echo placeholder instead of informative error when no backend available | closed (duplicate) | 2026-07-26 |
| [#1369](https://github.com/wildcard/caro/issues/1369) | P2 | cli | `caro ai --once` hint for --execute omits required global flag placement | open | 2026-07-25 |
| [#1442](https://github.com/wildcard/caro/issues/1442) | P2 | docs | CLAUDE.md version drift 1.4.0 vs actual 1.5.0 | closed (duplicate) | 2026-09-07 |
| [#1449](https://github.com/wildcard/caro/issues/1449) | P2 | cli | caro ai --once hangs silently with no output when embedded model is unavailable | closed (duplicate) | 2026-09-12 |
| [#1450](https://github.com/wildcard/caro/issues/1450) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (re-drift) | closed (duplicate) | 2026-09-12 |
| [#1494](https://github.com/wildcard/caro/issues/1494) | P1 | ai | `caro ai --once` CPU backend always returns deletion-clarification (system prompt contains "rm") — duplicate of canonical [#1269](https://github.com/wildcard/caro/issues/1269) (BW-006) | closed (duplicate) | 2026-10-01 |
| [#1495](https://github.com/wildcard/caro/issues/1495) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 — duplicate of canonical [#1098](https://github.com/wildcard/caro/issues/1098) (BW-002); QA agent re-filed instead of commenting | closed (duplicate) | 2026-10-01 |
| [#1499](https://github.com/wildcard/caro/issues/1499) | P1 | ai | `caro ai --once` hangs indefinitely in non-TTY environments (blocking stdin read) | closed | 2026-10-02 |
| [#1500](https://github.com/wildcard/caro/issues/1500) | P2 | embedded | CPU backend stub returns misleading "What exactly should be deleted?" for all queries | closed (duplicate) | 2026-10-02 |
| [#1501](https://github.com/wildcard/caro/issues/1501) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 — drift recurred after #1044 | closed (duplicate) | 2026-10-02 |
| [#1179](https://github.com/wildcard/caro/issues/1179) | P1 | ai | `caro ai --once` lacks static-matcher fallback — always requires model download | open | 2026-05-28 |
| [#1408](https://github.com/wildcard/caro/issues/1408) | P1 | ai | `caro ai --once` provides zero user feedback during model initialization | open; fix PR #1415 pending | 2026-08-15 |
| [#1540](https://github.com/wildcard/caro/issues/1540) | P2 | ai | caro ai --once hangs silently with no feedback when no backend is ready | closed (duplicate) | 2026-10-08 |
| [#1541](https://github.com/wildcard/caro/issues/1541) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurrence of #1044) | closed (duplicate) | 2026-10-08 |
| pending-2 | P2 | ai | `caro ai --once` hangs silently (no output, no timeout msg) when no model; `doctor` doesn't warn | duplicate of #1408 and #1179 (both open), see #1540 — do not file | 2026-10-10 |
| pending-1 | P2 | docs | CLAUDE.md version banner shows 1.4.0 (GA) instead of 1.5.0 (recurrence of #1044) | fixed by #1478 on 2026-10-11 (closed #1098) — do not file | 2026-10-10 |
| [#1044](https://github.com/wildcard/caro/issues/1044) | P2 | docs | CLAUDE.md version banner shows 1.1.0 (GA) instead of 1.3.0 | closed | 2026-05-07 |
| [#1165](https://github.com/wildcard/caro/issues/1165) | P1 | safety | User allowlist cannot override `rm -rf /tmp/...` despite explicit allowlist pattern (regression from #1110) | open | 2026-05-23 |
| [#1098](https://github.com/wildcard/caro/issues/1098) | P2 | docs | CLAUDE.md version drift (shows 1.4.0, Cargo.toml is 1.5.0) | closed | 2026-05-07 |
| [#1169](https://github.com/wildcard/caro/issues/1169) | P1 | safety | `test_allowlist_functionality` broken by PR #1110 Critical pre-scan guard | closed (duplicate) | 2026-05-25 |
| [#1170](https://github.com/wildcard/caro/issues/1170) | P2 | cli | `cargo test safety` fails — evaluation binary rejects positional filter arg | closed (duplicate) | 2026-05-25 |
| [#884](https://github.com/wildcard/caro/issues/884) | P2 | i18n | EN landing headline refreshed but DE/JA still on old copy; EN title/H1 inconsistent | open | 2026-04-26 |
| [#1099](https://github.com/wildcard/caro/issues/1099) | P2 | cli | `caro test --verbose` output identical to non-verbose — per-test details not shown | open | 2026-05-15 |
| [#1107](https://github.com/wildcard/caro/issues/1107) | P1 | cli | `--backend openrouter` rejected as 'Unknown backend' despite PR #1097 adding backend | open | 2026-05-16 |
| [#1162](https://github.com/wildcard/caro/issues/1162) | P2 | cli | `cargo test safety` fails — evaluation harness rejects standard cargo test filter | closed | 2026-05-21 |
| [#1163](https://github.com/wildcard/caro/issues/1163) | P1 | cli | Global flags before subcommands silently bypass subcommand routing | open | 2026-05-22 |
| [#1164](https://github.com/wildcard/caro/issues/1164) | P2 | cli | `cargo run` fails without `--bin caro` (multi-binary project missing default-run) | closed | 2026-05-22 |
| [#1176](https://github.com/wildcard/caro/issues/1176) | P1 | safety | `test_allowlist_functionality` fails on main — Critical `rm -rf /` pattern too broad | closed (duplicate) | 2026-05-27 |
| [#1177](https://github.com/wildcard/caro/issues/1177) | P1 | cli | `caro config set telemetry.enabled false` advertised in consent screen but rejected as unknown key | closed | 2026-05-27 |
| [#1292](https://github.com/wildcard/caro/issues/1292) | P1 | cli | Telemetry consent screen advertises invalid config key `telemetry.enabled` | closed (duplicate) | 2026-07-11 |
| [#1293](https://github.com/wildcard/caro/issues/1293) | P1 | cli | `--backend-info` shows wrong backends (static/claude shown; mesh/ai-horde/hybrid missing) | closed (duplicate) | 2026-07-11 |
| [#1294](https://github.com/wildcard/caro/issues/1294) | P2 | cli | `--backend` help text missing mesh, ai-horde, hybrid (added PR #1209) | closed (duplicate) | 2026-07-11 |
| [#1295](https://github.com/wildcard/caro/issues/1295) | P2 | ai | `caro ai --once` hangs silently with no feedback when model unavailable | closed (duplicate) | 2026-07-11 |
| [#1361](https://github.com/wildcard/caro/issues/1361) | P1 | ai | `caro ai --once` returns `echo 'Please clarify your request'` with confidence=0.85 and exit 0 when model unavailable; CPU stub `prompt.contains("rm")` fires on system-prompt words | closed (duplicate) | 2026-07-21 |
| [#1362](https://github.com/wildcard/caro/issues/1362) | P2 | cli | Static matcher Pattern 43 regex `^(list|show).*(all)?.*(files?)\s*$` rejects "list files in current directory"; `\s*$` over-anchored | open | 2026-07-21 |
| [#1440](https://github.com/wildcard/caro/issues/1440) | P1 | embedded | Model download hangs indefinitely — HfHubClient has no HTTP timeout | closed | 2026-09-06 |
| [#1444](https://github.com/wildcard/caro/issues/1444) | P2 | docs | CLAUDE.md version (1.4.0) and MSRV (1.83) out of sync with v1.5.0 (MSRV 1.85) | closed (duplicate) | 2026-09-08 |
| [#1473](https://github.com/wildcard/caro/issues/1473) | P1 | embedded | CPU backend placeholder always returns "clarify" due to system-prompt contamination | closed (duplicate) | 2026-09-24 |
| [#1474](https://github.com/wildcard/caro/issues/1474) | P2 | docs | CLAUDE.md version shows 1.4.0 instead of 1.5.0 — regression of #1044 | closed (duplicate) | 2026-09-24 |
| [#1483](https://github.com/wildcard/caro/issues/1483) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (regression of #1044) | closed (duplicate) | 2026-09-29 |
| [#1484](https://github.com/wildcard/caro/issues/1484) | P2 | ai | caro ai --once silently blocks indefinitely when no model is available | closed (duplicate) | 2026-09-29 |
| [#1520](https://github.com/wildcard/caro/issues/1520) | P2 | docs | CLAUDE.md version banner shows 1.4.0 instead of 1.5.0 (recurring drift, root cause unresolved) | closed (duplicate) | 2026-10-03 |
| [#1269](https://github.com/wildcard/caro/issues/1269) | P1 | embedded | CPU stub `rm`-in-system-prompt: all non-static-matched queries return clarification stub | open | 2026-06-26 |
| [#1523](https://github.com/wildcard/caro/issues/1523) | P1 | embedded | all embedded CPU paths return wrong "Clarification needed" error — cpu.rs:63 checks full system prompt (contains "rm -rf") instead of user request text; affects ai --once, --force-llm, and any static-matcher miss | closed (duplicate) | 2026-10-04 |
| [#1248](https://github.com/wildcard/caro/issues/1248) | P2 | ai | caro ai silently hangs in non-TTY; no error or --once suggestion | open | pre-2026-10-05 |
| [#1530](https://github.com/wildcard/caro/issues/1530) | P2 | cli | config set backend has no 'auto' option — can't restore auto-detect without full config reset | open | 2026-10-06 |
| [#1272](https://github.com/wildcard/caro/issues/1272) | P1 | ai | `caro ai --once` hangs silently with zero output when no backend is available | open | 2026-06-26 |
| [#1603](https://github.com/wildcard/caro/issues/1603) | P2 | docs/ops | QA memory files stuck at 2026-05-07 bootstrap — 25+ update PRs unmerged | open | 2026-10-11 |

---

## Template

```markdown
## BUG-XXX: [Short title]

**Reported:** YYYY-MM-DD
**Severity:** Critical / High / Medium / Low
**Component:** [file path or feature area]
**Reproducible:** Always / Sometimes / Rarely

### Steps to Reproduce
1.
2.
3.

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens]

### Screenshots/Logs
[If applicable]

### Environment
- OS:
- caro version:

### Notes
[Additional context]
```

---

## Resolved (closed issues)

- **BUG-001**: Search highlight double-counting with global regex — Fixed 2026-01-02
- [#1044](https://github.com/wildcard/caro/issues/1044) — CLAUDE.md version banner drift — Closed as duplicate of #1098 (2026-10-09 confirmed closed)
