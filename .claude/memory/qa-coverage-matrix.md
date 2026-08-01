# QA Coverage Matrix

**Last updated**: 2026-08-01

This file drives Slot C surface selection. Pick the row with the oldest 'Last tested' value (treat 'never' as oldest). Tie-break randomly.

---

## Smoke (Slot A) coverage

One row per pass. Update 'Last tested' column after every Slot A run.

| Date | Build | --version | --help | doctor | dry-run | Notes |
|------|-------|-----------|--------|--------|---------|-------|
| 2026-05-07 | PASS | PASS (1.3.0) | PASS | PASS | FLAKE | Model download blocked in sandbox (see flakes); first bootstrap run |
| 2026-08-01 | PASS | PASS (1.5.0) | PASS | PASS | FLAKE | FLAKE-001 reproduced (2nd observation); 597 lib tests PASS |

---

## Feature surface table

Slot C selects from this table. Update 'Last tested', 'Result', and 'Linked issue(s)' after each Slot C exercise.

| # | Surface | Domain | Last tested | Result | Linked issue(s) |
|---|---------|--------|-------------|--------|-----------------|
| 1 | CLI smoke (build, --version, --help, doctor) | cli | 2026-08-01 | PASS | — |
| 2 | `caro -p "..." --dry-run` command generation | cli | 2026-08-01 | FLAKE | FLAKE-001 |
| 3 | Telemetry consent persistence across invocations | cli | 2026-05-07 | PASS | — |
| 4 | `caro shell-init bash/zsh/fish` | shell-integration | 2026-05-07 | PASS | — |
| 5 | `caro init` setup wizard (--minimal, --force) | cli | 2026-05-07 | PASS | — |
| 6 | Safety validation unit tests (cargo test safety) | safety | 2026-08-01 | PASS | — |
| 7 | Safety CVE patterns (ruleset load, shell filters) | safety | 2026-05-07 | PASS | — |
| 8 | Full library test suite (cargo test --lib) | cli | 2026-08-01 | PASS | — |
| 9 | CaroML: `caro new / check / list / jobs` | cli | 2026-05-07 | PASS | — |
| 10 | `caro ai --once` scripted conversational mode | ai | 2026-08-01 | FAIL | [#1387](https://github.com/wildcard/caro/issues/1387) |
| 11 | `caro ai --continue-session` shell widget | ai | never | — | — |
| 12 | `caro assess` system assessment | cli | never | — | — |
| 13 | `caro suggest` command suggestions | cli | never | — | — |
| 14 | `caro config get/set/show/reset` | cli | never | — | — |
| 15 | `caro --output json` format correctness | cli | never | — | — |
| 16 | `caro --output yaml` format correctness | cli | never | — | — |
| 17 | `caro completion bash/zsh/fish` | shell-integration | never | — | — |
| 18 | `caro test --backend static` eval harness | cli | never | — | — |
| 19 | Embedded model backend command quality | embedded | never | — | — |
| 20 | Ollama backend (requires ollama installed) | ollama | never | — | — |
| 21 | CaroML: `caro run / generate / render / history` | cli | never | — | — |
| 22 | CaroML: `caro experiment / adopt / why` | cli | never | — | — |
| 23 | CaroML: `caro do` Carofile job runner | cli | never | — | — |
| 24 | `caro skill install` (bundled skill management) | cli | never | — | — |
| 25 | Website homepage caro.sh (curl + parse) | website | never | — | — |
| 26 | Website docs pages (curl + parse) | docs | never | — | — |
| 27 | Install script `scripts/install.sh` | install | never | — | — |
| 28 | Homebrew tap formula | install | never | — | — |
| 29 | `caro --safety strict/moderate/permissive` modes | safety | never | — | — |
| 30 | `caro --verbose` timing output | cli | never | — | — |
| 31 | i18n website locale smoke (curl /es/, /fr/, /ja/) | i18n | never | — | — |
| 32 | `caro doctor` advisory content accuracy | cli | 2026-05-07 | PASS | — |

---

## Surfaces added by bug filings

When a filed issue reveals a new surface gap, add it here so Slot C tracks it in a future pass.

| Issue | Surface | Domain | Filed | Status |
|-------|---------|--------|-------|--------|
| [#1044](https://github.com/wildcard/caro/issues/1044) | CLAUDE.md version field alignment | docs | 2026-05-07 | closed |
| [#1387](https://github.com/wildcard/caro/issues/1387) | `caro ai --once` static matcher bypass | ai | 2026-08-01 | open |
| [#1388](https://github.com/wildcard/caro/issues/1388) | CLAUDE.md version drift (1.4.0→1.5.0) | docs | 2026-08-01 | open |

---

## Notes

- Slot C tie-break: when multiple surfaces share 'never', pick lowest `#` number unless context suggests a riskier surface is more valuable to exercise.
- Website surfaces (#25, #26) can be tested with `curl` + Python parsing alone — no caro build needed.
- Surfaces requiring model download (#19, #20) should be tested from an environment with a pre-downloaded model; note in session log if sandbox blocks download.
- **Slot B search**: use perPage:100 and paginate to avoid missing PRs when the window is large (>20 PRs). A 20-result cap silently drops older-in-window PRs from spot-checking.
- **Next Slot C recommendation**: surface #11 (`caro ai --continue-session`) — oldest untested after #10. Alternatively prioritize #25/#31 (website/i18n) given recent PRs #1325/#1351/#1352.
