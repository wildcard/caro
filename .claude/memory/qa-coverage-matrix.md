# caro-qa-agent — Coverage Matrix

**Last updated**: 2026-04-26 by caro-qa-agent
**Drives**: Slot C random-past-feature selection — pick the surface with the oldest "Last tested" date; break ties with random pick.

---

## Regression candidates pool

| # | Surface | Last tested | Result | Linked issue(s) |
|---|---|---|---|---|
| 1 | `caro --dry-run` (preview without execution) | never | — | — |
| 2 | Embedded backend offline (no Ollama running) | never | — | — |
| 3 | Edit-mode exit code 201 + clipboard fallback | never | — | — |
| 4 | Config priority chain (CLI > env > file > auto) | never | — | — |
| 5 | Install script arch detection (`curl -fsSL https://setup.caro.sh \| bash`) | never | — | — |
| 6 | Homebrew formula version (`brew info wildcard/tap/caro`) | never | — | — |
| 7 | `caro completion {bash,zsh,fish}` script validity | never | — | — |
| 8 | `caro suggest "<query>"` knowledge-index match | never | — | — |
| 9 | Safety strict mode blocks canonical 5 dangerous commands | never | — | — |
| 10 | Website `caro.sh` landing — no raw i18n keys (en + 2 random) | 2026-04-26 (en, de, ja) | PARTIAL — no raw keys, but EN H1 drifted off DE/JA copy + EN title/H1 mismatch | [#884](https://github.com/wildcard/caro/issues/884) |
| 11 | Website `/docs/installation` matches published versions | never | — | — |
| 12 | Website `/blog/` index lists posts | never | — | — |
| 13 | `caro init --minimal` writes usable config | never | — | — |
| 14 | `caro test --suite <name>` eval suite runs | never | — | — |

---

## Smoke (Slot A) coverage

| Pass | Date | caro --version | --help | doctor | sample prompt | Notes |
|---|---|---|---|---|---|---|
| 1 | 2026-04-26 | PASS (1.3.0, b993e2b 2026-04-20) | PASS | PASS (model cached, embedded ready, ollama not installed) | PASS (ls in dry-run) | One agent-timeout warning during sample prompt; tracked in known-flakes |

---

## Recent diff (Slot B) coverage

| Pass | Date | PRs covered | Result | Notes |
|---|---|---|---|---|
| — | — | — | — | — |

---

## Surfaces added by bug filings

When I file a bug, the affected surface enters this pool so I exercise it again later.

| Surface | First seen | Linked issue | Reason for inclusion |
|---|---|---|---|
| Landing headline cross-locale parity (EN vs DE/JA + within-EN title/H1) | 2026-04-26 | [#884](https://github.com/wildcard/caro/issues/884) | Slot C found drift; revisit after fix lands to verify |
