# caro-qa-agent — Coverage Matrix

**Last updated**: 2026-05-02 by caro-qa-agent
**Drives**: Slot C random-past-feature selection — pick the surface with the oldest "Last tested" date; break ties with random pick.

---

## Coordination With Sibling QA Routines

This matrix is owned by **caro-qa-agent** (rotation-based regression coverage). A sibling routine **caro-frustrated-beta** (PR #910, merged 2026-04-27) runs daily at 5 AM and tests **website-advertised queries against the binary** under a frustrated-power-user persona. That routine's findings live under `.claude/beta-testing/runs/<date>/` with labels `qa-routine`, `frustrated-beta`, `bug-intent-dropped`, `website-broken-promise`. **Do not duplicate** that scope here — if a surface is already covered by the gtm-use-cases corpus, leave it to frustrated-beta and pick a different rotation candidate.

| Routine | Time (UTC) | Persona | Scope | Filing label |
|---|---|---|---|---|
| caro-frustrated-beta | 12:00 (5 AM Vancouver) | one frustrated power CLI user | website-advertised queries → binary | `qa-routine` + `frustrated-beta` |
| caro-qa-agent (this) | 14:00 (7 AM Vancouver) | rotation across personas | regression candidates over time | `qa` |

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
| 9 | Safety strict mode blocks canonical 5 dangerous commands | 2026-05-03 (embedded-cpu build) | INCONCLUSIVE — model generation failed before safety validation could be exercised; see F-2026-05-03-A in known-flakes | — |
| 10 | Website `caro.sh` landing — no raw i18n keys (en + 2 random) | 2026-04-26 (en, de, ja) | PARTIAL — no raw keys, but EN H1 drifted off DE/JA copy + EN title/H1 mismatch | [#884](https://github.com/wildcard/caro/issues/884) |
| 11 | Website `/docs/installation` matches published versions | never | — | — |
| 12 | Website `/blog/` index lists posts | never | — | — |
| 13 | `caro init --minimal` writes usable config | never | — | — |
| 14 | `caro test --suite <name>` eval suite runs | never | — | — |
| 15 | `caroml` parser + `caro check` (NEW — shipped #893 2026-04-29) | never | — | — |
| 16 | `caro ai --once "<prompt>"` privacy toggles honored (v1.3.0 advertised) | never | — | — |

### Out of scope (track but do not test)

- **caro-terminal** (Ghostty-backed desktop GUI, issues #1009-#1023): planning/foundation phase, not yet shipped to users. Add to rotation only when first binary is published.

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
