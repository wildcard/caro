# QA Coverage Matrix

**Last updated**: 2026-05-27

This file drives Slot C surface selection. Pick the row with the oldest 'Last tested' value (treat 'never' as oldest). Tie-break randomly.

---

## Smoke (Slot A) coverage

One row per pass. Update 'Last tested' column after every Slot A run.

| Date | Build | --version | --help | doctor | dry-run | Notes |
|------|-------|-----------|--------|--------|---------|-------|
| 2026-05-07 | PASS | PASS (1.3.0) | PASS | PASS | FLAKE | Model download blocked in sandbox (see flakes); first bootstrap run |
| 2026-05-27 | PASS | PASS (1.4.0) | PASS | PASS | FLAKE | FLAKE-001 reproduced (2nd occurrence); `--output json` and `suggest` both PASS via static backend |

---

## Feature surface table

Slot C selects from this table. Update 'Last tested', 'Result', and 'Linked issue(s)' after each Slot C exercise.

| # | Surface | Domain | Last tested | Result | Linked issue(s) |
|---|---------|--------|-------------|--------|-----------------|
| 1 | CLI smoke (build, --version, --help, doctor) | cli | 2026-05-07 | PASS | — |
| 2 | `caro -p "..." --dry-run` command generation | cli | 2026-05-07 | FLAKE | — |
| 3 | Telemetry consent persistence across invocations | cli | 2026-05-07 | PASS | — |
| 4 | `caro shell-init bash/zsh/fish` | shell-integration | 2026-05-07 | PASS | — |
| 5 | `caro init` setup wizard (--minimal, --force) | cli | 2026-05-07 | PASS | — |
| 6 | Safety validation unit tests (cargo test safety) | safety | 2026-05-27 | PASS (20/20 lib; integration FAIL #1176) | [#1176](https://github.com/wildcard/caro/issues/1176) |
| 7 | Safety CVE patterns (ruleset load, shell filters) | safety | 2026-05-27 | PASS | — |
| 8 | Full library test suite (cargo test --lib) | cli | 2026-05-27 | PASS (543 passed) | — |
| 9 | CaroML: `caro new / check / list / jobs` | cli | 2026-05-07 | PASS | — |
| 10 | `caro ai --once` scripted conversational mode | ai | 2026-05-27 | PASS (FLAKE on backend) | — |
| 11 | `caro ai --continue-session` shell widget | ai | never | — (TTY REPL unimplemented) | — |
| 12 | `caro assess` system assessment | cli | never | — | — |
| 13 | `caro suggest` command suggestions | cli | 2026-05-27 | PASS | — |
| 14 | `caro config get/set/show/reset` | cli | 2026-05-27 | FAIL (telemetry key missing) | [#1177](https://github.com/wildcard/caro/issues/1177) |
| 15 | `caro --output json` format correctness | cli | 2026-05-27 | PASS | — |
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
| [#1044](https://github.com/wildcard/caro/issues/1044) | CLAUDE.md version field alignment | docs | 2026-05-07 | **closed** (fixed in v1.4.0) |
| [#1176](https://github.com/wildcard/caro/issues/1176) | Safety allowlist: rm -rf /tmp pattern false positive | safety | 2026-05-27 | open |
| [#1177](https://github.com/wildcard/caro/issues/1177) | `caro config set telemetry.enabled false` not implemented | cli | 2026-05-27 | open |

---

## Notes

- Slot C tie-break: when multiple surfaces share 'never', pick lowest `#` number unless context suggests a riskier surface is more valuable to exercise.
- Website surfaces (#25, #26) can be tested with `curl` + Python parsing alone — no caro build needed.
- Surfaces requiring model download (#19, #20) should be tested from an environment with a pre-downloaded model; note in session log if sandbox blocks download.
