# ADR-070: `caro.delegation.v1` — A Machine-Decidable Lexicon of Which Argument Is a Program

- **Status**: Proposed (implementation ADR — buildable on the tree that ships in 1.4.0; no unlanded
  ADR is a prerequisite)
- **Date**: 2026-09-14
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **[GTFOBins](https://gtfobins.github.io/)** (GPL-3.0,
  [GTFOBins/GTFOBins.github.io](https://github.com/GTFOBins/GTFOBins.github.io)) — the
  community-curated catalogue of ways a legitimate executable can be made to spawn a program, read
  a file or write one. Schema read from [gtfobins.org/contributing](https://gtfobins.org/contributing/)
  and the [`git` entry](https://gtfobins.org/gtfobins/git/), both 2026-09-14.
- **Failure-mode corpus** (all read 2026-09-14):
  - **Black Hat USA 2026 (2026-08-05), Elad Meged / Novee Security** — one GitHub issue from an
    account with no repository privileges reached CI runner secrets in the vendors' *own*
    repositories for Claude Code, Gemini CLI and OpenAI Codex, each in the vendor's own default
    workflow. Primary write-ups:
    [CSA AI Safety Initiative research note](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-coding-agent-cicd-secrets-20260808-csa/) (2026-08-08),
    [The Hacker News](https://thehackernews.com/2026/08/claude-code-and-gemini-cli-flaws-let.html) (2026-08-07),
    [Novee Security](https://novee.security/blog/critical-flaws-in-anthropic-google-and-openais-coding-agents/).
  - **Round 1 — the delegated flag.** Verbatim from The Hacker News: *"Claude Code's command
    validator strips single-quoted text before its 23 checks run, which is correct behavior for
    bash, so a payload in the value of `git push --receive-pack`, **a flag git executes**, reached
    the runner untouched. That chain has **no CVE and no publicly stated fixed version**."*
  - **Round 2 — capability keyed on the program name.** The first patch exempted a hardcoded
    "read-only" list — `tac`, `rev`, `fold`, `expand`, `unexpand` — from path validation, and those
    binaries were then used to read the file the validator was protecting.
  - **[CVE-2026-12537](https://nvd.nist.gov/vuln/detail/CVE-2026-12537) / [GHSA-wpqr-6v78-jr5g](https://github.com/google-github-actions/run-gemini-cli/security/advisories/GHSA-wpqr-6v78-jr5g)** —
    CVSS v4 **10.0**. Gemini CLI's `coreTools` allowlist was *"parsed and matched during tool
    registration but never actually enforced at the point of execution, so the restriction existed
    in name only."* **The archetype: a declared restriction with no artifact tying it to the
    execution path.**
  - **[CVE-2026-54316](https://nvd.nist.gov/vuln/detail/CVE-2026-54316) / [GHSA-fg94-h982-f3mm](https://github.com/anthropics/claude-code/security/advisories/GHSA-fg94-h982-f3mm)** —
    round 3 of the same engagement; ADR-068 territory, cited here only to date the chain.
  - **[GuardFall](https://adversa.ai/blog/opensource-ai-coding-agents-shell-injection-vulnerability/)**
    (Adversa AI, June 2026) — *"ten of eleven tested open-source coding agents used command guards
    that inspected commands as plain strings while the shell that actually executed those commands
    rewrote them first."*
- **Failure mode found in Caro itself** (the load-bearing finding; Context §3): replaying all **67**
  compiled patterns from `src/safety/patterns.rs` under a faithful re-implementation of
  `SafetyValidator::is_dangerous_in_context` (`src/safety/mod.rs:432-455`) against a 30-row
  delegated-execution corpus returns **29 false negatives**. The single hit is `perl -e 'system(…)'`,
  caught by a head-position interpreter rule. **Zero delegated-flag rows flag.** Re-run with the
  quote filter disabled and every value unquoted — i.e. ADR-067's fix simulated generously —
  **14 of 15 are still missed**. Five control rows produce zero false positives in both runs.
- **Depends on**: nothing unlanded. `RiskLevel`, `SafetyLevel`, `ShellType`, `SuggestedRouting`,
  `SafetyValidator`, `ValidationResult`, `clap` 4.5, `serde`, `schemars` 0.8, `once_cell`,
  `assert_cmd` all ship in 1.4.0. The `data/cve_rules/` → `src/dogma/compiler.rs` → `build.rs` →
  bincode → `include_bytes!` pipeline ships and is reused verbatim. No network, no filesystem read,
  no daemon, no new crate, no new top-level module.
- **Relates to**: **ADR-067** (`caro.decompose.v1` — this ADR is the variant 067's `SiteKind` does
  not have; D8 commits the two to one merge PR and hands 067 the word splitter rather than letting
  it write a second), ADR-069 (`caro.preflight.v1` — owns `core.fsmonitor` as an *ambient config
  file*; this ADR owns it as a *command-line flag*, and D13 records the boundary), ADR-064
  (`caro.egress.v1` — `Locus::Remote`'s host is egress vocabulary and is deliberately not minted
  here), ADR-068 (`caro.disclosure.v1` — round 3 of the same attack chain), ADR-039 (sandbox-aware
  verdict tier — owns the container locus this ADR declines to mint), ADR-007 (shell AST —
  deliberately **not** depended on; see D2), ADR-059 (verb-namespace moratorium — honoured, see
  Consequences).
- **Amends**: nothing. Additive.
- **Full scope document**: [`caro-scope-delegated-execution-2026-09-14.md`](../../caro-scope-delegated-execution-2026-09-14.md)
- **Numbering note**: highest existing is ADR-069. Per
  [`.claude/rules/adr-numbering.md`](../../.claude/rules/adr-numbering.md), renumber on merge if
  another 070 lands first.

---

> **Provenance note (autonomous run).** The task template left `[FEATURE NAME]` unbound and ran with
> no user present, so target selection was mine. Rationale: ADR-067 (six days ago) built a
> structural model of *where a command's execution sites are* and, by its own coverage rules,
> attributes flags and operands as "non-executable data". That attribution is false for an
> enumerable set of `(program, flag)` pairs, and the first link of the Black Hat chain against
> Anthropic's own CI runner is exactly one of those pairs — the one link in that chain with no CVE
> and no stated fix. A ten-minute replay shows Caro misses 29 of 30 such rows today and 14 of 15
> after 067's fix. That gap is measured, dated, and owned by nobody.
>
> **Moratorium note.** ADR-059 declared itself the last ADR in its space until
> `src/safety/assessment.rs` merges. Re-verified 2026-09-14: it has not (`src/assessment/` is still
> the hardware recommender; `Commands::Assess` is still commented out at `src/main.rs:422-434`).
> This ADR mints no policy vocabulary, no approval exchange, no lifecycle event and no new routing
> tier. It reuses `RiskLevel`, `SafetyLevel` and `SuggestedRouting` unchanged and reuses
> `SafetyValidator` rather than re-implementing matching. It mints one report struct, one lexicon
> schema, and the delegation vocabulary those need. D8 commits it to nesting inside
> `caro.assessment.v1` when that lands.
>
> **Validation-discipline note.** ADR-069's first draft was rejected for claiming an exemption from
> [`validation-discipline.md`](../../.claude/rules/validation-discipline.md) that it could not
> defend. This ADR claims no exemption and states each gate's status explicitly in Consequences
> §4. **Gates 1 and 4 are unmet and block the verb PR.**

---

## Context

### 1. What the researched feature models, and models well

GTFOBins is the only community-curated corpus in the ecosystem that answers "what else can this
binary be made to do". Its schema splits the question cleanly:

- **functions** — `shell`, `command`, `reverse-shell`, `bind-shell`, `file-write`, `file-read`,
  `upload`, `download`, `library-load`, `inherit`
- **contexts** — `unprivileged`, `sudo`, `suid`, `capabilities`
- **`inherit` / `from`** — transitive capability, e.g. `git help config` inherits every `less`
  vector, which is how `!/bin/sh` inside a pager becomes a `git` finding

That function/context split is the right decomposition and this ADR inherits it: *how the
delegation is expressed* and *where the delegated program runs* are orthogonal and get separate
types.

### 2. Why it cannot be consumed as it stands

Four structural properties, none of which are defects in GTFOBins — they follow from it being a
reference for humans doing offensive work, not a predicate for a pre-execution matcher.

| | Property | Consequence for a matcher |
|---|---|---|
| G1 | `code` is a free-text snippet with human placeholders (`/path/to/command`, `x...x`) | There is no predicate. Nothing can mechanically decide whether a given argv matches `ln -s /bin/sh git-x` + `git --exec-path=. x`. |
| G2 | Framing is privilege escalation *given you can already run the binary*; contexts are `sudo`/`suid` | The agent question is the inverse — the binary is allowlisted and run unprivileged **on purpose**, and the question is whether this approved invocation spawns an unapproved program. |
| G3 | No locus | `git push --receive-pack=X` runs `X` on the **remote** — unless the transport resolves to a local path or `file://`, which is precisely the Novee vector. GTFOBins has no field for this because for privesc the answer is always "here". |
| G4 | No conjunctions | `ssh -o LocalCommand=X` is inert without `PermitLocalCommand=yes`. Shipping the bare flag is a guaranteed false positive; shipping the conjunction is inexpressible. |

A fifth point, recorded so it is not re-litigated: GTFOBins is GPL-3.0 and Caro is AGPL-3.0. GPLv3
§13 permits combination in that direction, so vendoring would be **licit**. It is rejected on G1–G4,
not on licence.

### 3. What Caro does today — measured, not asserted

Method: extract every `pattern:` literal from `src/safety/patterns.rs` (67 compile cleanly under
both the `regex` crate's syntax subset and Python's `re`), replay them under a faithful
re-implementation of `is_dangerous_in_context` (`src/safety/mod.rs:432-455` — discard a match
preceded by an odd count of unescaped quotes), against 30 rows that place an attacker-chosen
program in a delegated argument position plus 5 legitimate control rows.

```
patterns compiled: 67
delegation rows that MUST flag: 30
FALSE NEGATIVES: 29 / 30
control rows: 5      false positives: 0
```

The one hit is `perl -e 'system("/tmp/evil.sh")'`, via
`(python|perl|ruby)\s+-[ec]\s+.*system\s*\(` — a head-position rule, i.e. ADR-067's case.
`git push --receive-pack=…`, `git -c core.fsmonitor=…`, `ssh -o ProxyCommand=…`, `rsync -e …`,
`tar --use-compress-program=…`, `find … -exec …`, `env … X`, `make -f …`: **all silent.**

The decisive control — quote filter disabled entirely, every delegated value written unquoted,
which is strictly more generous than ADR-067 promises:

```
rows: 15      still missed: 14
```

The single hit fires on the payload's own `curl … | sh` text, not on the delegation; substitute a
script path and it disappears. **ADR-067's fix does not close this class.** The missing artifact is
not a better parser. It is a lexicon.

### 4. Why the three vendor failures are one failure

The Hacker News summarised all three in one sentence: *"one part marked a value safe, and a later
part acted on that value with more authority."*

- Round 1 assumed **a program's arguments are inert**.
- Round 2 assumed **a program's name determines its capability**.
- Gemini CLI assumed **a restriction parsed at registration is a restriction at execution**.

All three are the assumption that capability is a property of the *executable*. It is not. It is a
property of the **pair** — and there is no artifact anywhere in the ecosystem that says so in a
form a machine can consult offline.

---

## Decision

Ship `caro.delegation.v1`: a **community-curated, build-time-compiled, offline lexicon of
`(head, argument-position)` pairs whose value becomes a program**, plus a read-only verb
`caro delegates` that resolves a command against it and emits one JSON report and one exit code.

### D1 — The lexicon key is the pair, never the head

`LexiconEntry` requires both `heads: Vec<String>` and `matcher: LexiconMatcher`; the build-time
compiler rejects an entry with one and not the other. There is **no field in which to write
"`git` is safe" or "`tac` is read-only"**, which makes the round-2 class structurally
unrepresentable rather than merely discouraged.

### D2 — Word-split first, fail closed, no regex over the raw string

`split_words(input, shell) -> Option<Vec<Word>>` is a conservative POSIX splitter (~140 LOC) that
returns `None` — never a partial result — on an unbalanced quote, an expansion in program position,
or a non-POSIX dialect. There is no code path from an unsplit string to a clean answer. This is the
minimum honest response to GuardFall, and it deliberately does **not** adopt ADR-007's full shell
AST: the question "which word is a program" does not need one, and 067 §D5 already recorded the AST
as deferred.

### D3 — Delivery is the `data/cve_rules/` pipeline, verbatim

`data/delegation/*.yaml` → `src/dogma/compiler.rs` → `build.rs` → bincode → `include_bytes!` →
`Lazy`, with a `scripts/validate-delegation-yaml.ts` sibling of the existing CVE linter, and
`test_cases` compiled into `$OUT_DIR/delegation_generated_tests.yaml` for the eval suite exactly as
`cve_generated_tests.yaml` is today. Feature `delegation-lexicon`, **in `default`**, mirroring
`cve-rules`. Nothing about the contribution loop is new; contributors who can add a CVE rule can
add a delegation entry.

### D4 — The lexicon's identity is a required field of every report

`DelegationReport.lexicon: LexiconIdentity { entries: usize, digest: String }` — entry count and
BLAKE3 digest of the compiled blob — is **required and never defaulted**. A clean report produced
by a zero-entry lexicon is textually distinguishable from one produced by a 24-entry lexicon, by
any consumer, with no further call. `caro delegates --lexicon-only` exposes the same identity as a
one-line shell check.

This is the direct structural answer to CVE-2026-12537. A restriction that "existed in name only"
was invisible because nothing in the output named the artifact that was supposed to enforce it.

### D5 — Unreadable is a verdict, not an absence

`DelegationVerdict` has five variants: `None`, `Declared`, `Finding`, `Unresolved`, `Unparsed`.
`Unresolved` (the delegated program's text is not in the input) and `Unparsed` (the splitter
refused) are **separate variants with separate exit codes**, and neither can coexist with a clean
result. `WordAccounting.refused` must be empty for `None` or `Declared`. A drop is representable
only as a number that does not add up, and a number that does not add up downgrades the verdict.

### D6 — `TransportDependent` computes as `Local`

`git push --receive-pack=X ssh://host/r` executes `X` on the peer; `git push --receive-pack=X
file:///tmp/r` executes it here. Resolving which requires interpreting the transport operand, which
requires expansion, which is out of scope. `Locus::TransportDependent { operand }` records the
ambiguity and `risk_level` is computed **as if `Local`**. Fail-closed. The alternative — assuming
`Remote` — is how the Novee vector reached a CI runner.

### D7 — A delegated value is never a value

Every site with `provenance == Literal` has its `program` re-entered into **the caller's own
`SafetyValidator`** via `validate_command`, and the result appears as `site.inner`. Both the outer
construct's risk and the inner command's risk are in the report. There is no authority gradient
between "the part that marked it safe" and "the part that acted on it", because there is one
validator and the report shows its answer twice.

This also means `patterns.toml`, profiles and the `cve-rules` set apply to a delegated program
exactly as they apply to a top-level command, with no second configuration surface.

### D8 — This nests inside ADR-067, and ships the splitter 067 needs

`Span` and `InnerValidation` are structurally identical to ADR-067 §2.1 and §2.5. When 067 lands,
both definitions here are deleted and 067's imported, and a `SiteKind::DelegatedArg { lexicon_id }`
carries the site into `DecompositionReport`. One PR, and that PR does not have to choose between
two word splitters because this ADR ships the only one. Correspondingly, when `caro.assessment.v1`
lands, `DelegationReport` nests under it rather than standing alone.

### D9 — Derive, do not vendor

`data/delegation/*.yaml` is independently authored, cites GTFOBins (and NVD, and the vendor
advisories) per entry in a `references` list a reviewer spot-checks, and is shaped as predicates
rather than recipes. No GTFOBins file is copied. Recorded reason: G1–G4, not licence.

### D10 — `FileReferenced` never opens the file

`make -f /tmp/evil.mk` reports `provenance: file_referenced` and `verdict: unresolved`. Caro does
not read `/tmp/evil.mk`. The constraint *pure subprocess call, no daemon, no state* is not
negotiable, and a validator that opens files named by its untrusted input is exactly the shape of
CVE-2025-41390 (TruffleHog compromised by the repository it scanned) that ADR-069 documents.

### D11 — Verdict is the contract; the exit code is a shortcut

Stdout is exactly one newline-terminated `caro.delegation.v1` document. A consumer keys on
`verdict`. `exit_code()` is a total function of `verdict`, exposed as a method so it is testable
without spawning a process.

| Exit | Verdict |
|---|---|
| `0` | `None`, `Declared` |
| `3` | `Finding` |
| `4` | `Unresolved` |
| `5` | `Unparsed` |
| `1` | internal error, no payload |
| `2` | **reserved for `clap` usage errors — never emitted** |

`2` is deliberately unused. ADR-065/066/067/068 each use `2` for a finding and therefore collide
with `clap`'s own usage exit, which ADR-068 §7.4 already records as needing a consolidation PR.
This verb adds no fifth collision and proposes **`3..=15` as the reserved range for verb findings**
as that PR's target.

### D12 — JSON only; no human renderer in v1

`--json` is the default and only format. A human renderer is a second contract to keep in sync and
this verb's consumer is a harness, not a person. Adding one later is additive.

### D13 — The boundary with ADR-069

ADR-069 owns `core.fsmonitor` as an **ambient configuration file** discovered in a workspace.
ADR-070 owns it as a **flag on a command line**. `git -c core.fsmonitor=X status` is not an ambient
config and `.git/config` containing `core.fsmonitor` is not a command line; the two never both fire
on one input. When 069 lands, its `PF-GIT-EXEC-*` finding IDs and this lexicon's `DEL-GIT-C-*`
entry IDs must cross-reference in one PR so a reader of either knows the other exists.

### D14 — Three PRs, and the risky one is not in v1

| PR | Contents | Behaviour change |
|---|---|---|
| 1 | splitter, resolver, lexicon, compiler, build, data, unit tests | **none** — no CLI surface, no verdict change |
| 2 | `caro delegates` verb + contract tests | additive, read-only |
| 3 | `SafetyValidator` consults the lexicon so plain `caro "…"` benefits | **out of scope for v1** |

PR 3 is where every false positive lives. Shipping it inside v1 would recreate the
v2.1.259-shipped / v2.1.260-reverted whiplash ADR-067 §1 documents, in which argument analysis was
bolted onto a matcher with no structural model and had to be reverted within a day.

---

## Consequences

### 1. Positive

- The 29/30 false-negative class in §3 becomes reportable, with the `(program, flag)` pairs named
  in reviewable YAML rather than buried in regex.
- The artifact is **portable**. "`git` is safe; `git --exec-path=…` is a program spawn" is a claim
  any harness can consume, offline, from a single binary — which is the only form this can take
  given the Novee finding covers three vendors and, per CSA, *"well over a hundred public
  repositories running configurations functionally identical to the vulnerable defaults."*
- `find . -type f -exec rm {} +` lands at `Declared`, **exit 0**. The design's default answer to a
  ubiquitous legitimate construct is "yes, this delegates, and the program is clean" — not a block.
- Contribution cost is one YAML file with citations and test cases, reviewable without Rust.
- `caro delegates --lexicon-only` makes the Gemini-CLI condition ("the restriction existed in name
  only") a one-line shell check.

### 2. Negative / risks

- **A lexicon is a denylist, and denylists are incomplete by construction.** v1 covers 24 entries
  across 14 heads; GTFOBins covers 400+ binaries. `WordAccounting` and `Unresolved` keep the
  incompleteness visible, but a consumer that reads `verdict: none` as "this command spawns
  nothing" is wrong, and the schema cannot stop them.
- **Curation burden.** `data/delegation/` needs the same ongoing attention `data/cve_rules/` needs,
  and an unmaintained lexicon that still reports `entries: 24` is worse than none because D4's
  identity check passes.
- **New surface in the most safety-critical module.** ~500 LOC in `src/safety/`, including a hand-
  rolled word splitter. Mitigation: PR 1 changes no existing behaviour, and the splitter fails
  closed.
- **Known holes, shipped deliberately.** `awk 'BEGIN{system(…)}'`, `vim -c '!…'`, `gdb -ex`,
  `psql -c '\!…'` and `docker --entrypoint` are out of scope and a contract test asserts the hole
  rather than hiding it. That is honest and it is still a hole.
- **Binary size.** A few KB of bincode in every default build.

### 3. Neutral

- No existing verdict, exit code, config key or output format changes in v1.
- `src/safety/patterns.rs` is untouched. ADR-067's quote-filter finding remains 067's to fix.

### 4. Validation-discipline status (`.claude/rules/validation-discipline.md`)

Stated, not claimed away:

| Gate | Status |
|---|---|
| 1 — twenty first-hand transcripts | **NOT MET.** Zero exist for this feature. Blocks PR 2. |
| 2 — no surveys as sole evidence | **Vacuously met.** No survey cited; evidence is a disclosed attack chain, four primary sources, and an in-tree 29/30 replay. |
| 3 — "what breaks at 100 real users" | **Met** — scope §7.1, and Consequences §2 above. The assumption that breaks first is lexicon completeness; the instrumentation is `WordAccounting` and `LexiconIdentity`; the fallback is that `Unresolved`/`Unparsed` cannot be clean. |
| 4 — devil's-advocate review | **PENDING, REQUIRED.** An autonomous run cannot review itself. The three objections it should start from are scope §8. |
| 5 — Sean Ellis with a defended cohort | **N/A.** No product-market-fit claim is made anywhere in this ADR or its scope. |

PR 1 is ungated: it is remediation of a demonstrated false-negative class with an in-tree
reproduction, which the rule's opening scope note excludes. **PR 2 does not merge until Gates 1 and
4 are cleared or a human records a deliberate waiver with a one-line justification.**

---

## Alternatives Considered

**A. Add `--receive-pack`, `ProxyCommand`, `--exec-path` … as regexes to `src/safety/patterns.rs`.**
Rejected. It is the smallest diff and it reproduces the exact failure GuardFall documented across
ten of eleven agents: pattern matching a string the shell will rewrite. It also has nowhere to put
`Locus`, nowhere to put the `PermitLocalCommand` conjunction, and no way to distinguish
`find -exec rm {} +` from `find -exec /tmp/x.sh {} +` without writing the inner command's regex
inside the outer one. The 29/30 measurement is an indictment of the instrument, not of its coverage.

**B. Vendor GTFOBins' `_gtfobins/*.yaml`.** Rejected on shape (G1–G4), not licence — GPLv3 §13
permits the direction. The `code` field is a human recipe; there is no predicate to evaluate.

**C. Wait for ADR-067 and add a `SiteKind` variant there.** Rejected by measurement: with 067's fix
simulated as generously as possible, 14 of 15 rows are still missed, because 067's gap is not
parsing — it is the absence of the lexicon. D8 commits to the merge once 067 lands; waiting for it
first would leave the disclosed vector open for the duration.

**D. Adopt ADR-007's full shell AST.** Rejected as disproportionate. "Which word is a program" is
answerable with a fail-closed splitter; an AST is a much larger commitment that ADR-067 §D5 already
deferred, and adopting it here would make this ADR depend on unlanded work.

**E. Runtime enforcement — `ptrace`, `seccomp`, an exec audit hook.** Rejected. It would catch every
row in the corpus and it violates the constraint that the feature works as a pure subprocess call
with no daemon and no state. It is also the wrong product: Caro answers *before* execution.

**F. Ship the lexicon behind a non-default feature.** Rejected. A verb that reports `verdict: none`
because its data feature was compiled out is the precise fail-open condition D4 exists to prevent.
`cve-rules` is in `default` for the same reason.

**G. Extend the lexicon to read/write capabilities in v1** (the round-2 `tac`/`rev`/`fold` half).
Rejected as scope. It is the same lexicon with a different column, but adding it requires renaming
`DelegationKind` to a capability vocabulary — a schema break. v1 says `delegation`, not
`capability`, precisely so v2 can make that break knowingly.

---

## References

- Scope document: [`caro-scope-delegated-execution-2026-09-14.md`](../../caro-scope-delegated-execution-2026-09-14.md)
- CSA AI Safety Initiative, [*Three AI Coding Agents, One GitHub Issue: CI/CD Secrets Exposed*](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-coding-agent-cicd-secrets-20260808-csa/), 2026-08-08
- The Hacker News, [*Claude Code and Gemini CLI Flaws Let a GitHub Issue Reach CI Workflow Secrets*](https://thehackernews.com/2026/08/claude-code-and-gemini-cli-flaws-let.html), 2026-08-07
- Novee Security, [*Critical Flaws in Anthropic, Google, and OpenAI's Coding Agents*](https://novee.security/blog/critical-flaws-in-anthropic-google-and-openais-coding-agents/)
- Adversa AI, [*GuardFall*](https://adversa.ai/blog/opensource-ai-coding-agents-shell-injection-vulnerability/), June 2026
- [CVE-2026-12537](https://nvd.nist.gov/vuln/detail/CVE-2026-12537) · [CVE-2026-54316](https://nvd.nist.gov/vuln/detail/CVE-2026-54316)
- GTFOBins — [contributing](https://gtfobins.org/contributing/) · [`git`](https://gtfobins.org/gtfobins/git/) · [repo](https://github.com/GTFOBins/GTFOBins.github.io)
- Anthropic, [*Claude Code Action: Security*](https://github.com/anthropics/claude-code-action/blob/main/docs/security.md)
- In-tree: `src/safety/mod.rs:432-455` · `src/safety/patterns.rs` · `src/safety/cve_patterns.rs` · `src/dogma/compiler.rs` · `build.rs:123-160` · `data/cve_rules/README.md` · `src/models/mod.rs:152-255,419-427`
- ADRs: [067](./ADR-067-execution-site-decomposition.md) · [068](./ADR-068-output-disclosure-classification.md) · [069](./ADR-069-ambient-configuration-preflight.md) · [064](./ADR-064-sandbox-egress-conjunction-gate.md) · [039](./ADR-039-sandbox-aware-verdict-tier.md) · [007](./ADR-007-ast-parser-shell-validation.md)
- Rules: [`validation-discipline.md`](../../.claude/rules/validation-discipline.md) · [`git-workflow.md`](../../.claude/rules/git-workflow.md) · [`adr-numbering.md`](../../.claude/rules/adr-numbering.md)
