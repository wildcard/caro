# ADR-075: `caro.lex.v1` — Match What The Shell Will Run, Not What The String Says

- **Status**: **Proposed (correctness ADR — this is not a new feature, it is the shipped
  product being wrong).** Gate 1 N/A (see *Validation discipline*), Gate 3 and Gate 4 pending.
- **Date**: 2026-09-21
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **shell-aware command evaluation** — the guard design that survived
  the GuardFall survey, and the four guard tiers that did not. All sources read 2026-09-21:
  - [CSA, *GuardFall: Shell Injection Defeats AI Coding Agent Guardrails*](https://labs.cloudsecurityalliance.org/research/csa-research-note-guardfall-ai-coding-agent-shell-injection/)
    (published 2026-07-06, updated 2026-07-11) — the five bypass classes, the four guard
    maturity tiers, and Continue's five-component evaluator
  - [Adversa AI / Omer Ben Simon, *AI coding agents vulnerability: GuardFall shell injection*](https://adversa.ai/blog/opensource-ai-coding-agents-shell-injection-vulnerability/)
    (2026-06-30) — primary disclosure
  - [CSA, *DeepSeek Harness Sandbox Escape and Agent Containment*](https://labs.cloudsecurityalliance.org/research/csa-research-note-deepseek-harness-sandbox-escape-20260910-c/)
    (2026-09-10) — the rebroadcast that puts GuardFall in a three-failure pattern
  - [`brush-parser` 0.4.0](https://docs.rs/brush-parser/latest/brush_parser/) (MIT) and
    [`yash-syntax` 0.23.1](https://docs.rs/yash-syntax/latest/yash_syntax/) (GPL-3.0-or-later) —
    the two maintained Rust POSIX/bash parsers evaluated as the tokenizer
- **Depends on**: nothing. Phase 0 (D0) is a test file and needs no new type, dependency,
  configuration key or verb.
- **Relates to**: **ADR-063** (`destructive-v1` benchmark — this ADR supplies the *mutation*
  axis its corpus lacks and reuses its harness rather than building a second one), **ADR-058/059**
  (`caro.assessment.v1` — `LexicalAnalysis` is an input to an assessment, never a replacement
  for one), **ADR-020** (tiered approval — `HumanGate` is the floor D2 enforces), **ADR-074**
  (`caro.pending.v1` — re-assessment on resume is only meaningful if the assessment is correct),
  **ADR-067** (execution-site decomposition — this is what the execution site must know)
- **Full scope document**: [`caro-scope-shell-lexing-2026-09-21.md`](../../caro-scope-shell-lexing-2026-09-21.md)
- **Run report and measurement harness**: [`docs/research/caro-research-scoping-2026-09-21.md`](../research/caro-research-scoping-2026-09-21.md)

> **Provenance note (autonomous run).** The scheduled task's template still leaves
> `[FEATURE NAME]` unbound (outstanding since 2026-06-02) and ran with no user present, so target
> selection is the agent's own. This run did not select freely.
> `market-scans/2026-09-21-ai-agent-strategy-memo.md` §4 names **"the differential obfuscation
> test suite"** as the single *build or test next* item, with the explicit instruction to start
> at `rm -rf "/"` and `echo "hi ; rm -rf /`. This ADR is scoped around that test and the smallest
> change that makes it pass.
>
> **Counter-signal recorded, not suppressed.** The run report for ADR-074 ends with
> *"the next run of this task should be a code run, not a scoping run"*, and the same market memo
> observes that fifteen consecutive ADRs are untracked working-tree files. Both are correct. The
> response here is structural rather than rhetorical: **D0 is a pull request that needs no ADR at
> all.** A reviewer who wants to ignore this document entirely can still merge D0 tomorrow, get a
> number, and decide the rest on evidence. Everything after D0 is explicitly sequenced behind that
> number.

---

## Context

### 1. A named, published failure class that describes this product exactly

On 2026-06-30 Adversa AI disclosed **GuardFall**: eleven widely deployed open-source coding and
computer-use agents, roughly 548,000 GitHub stars between them, tested against five classes of
shell-injection bypass. **Ten of the eleven failed.** CSA's write-up states the root cause without
hedging: *"safety filters inspect commands as literal text, while the bash shell quietly rewrites
that text through quoting, variable expansion, command substitution, and piping before it ever
executes."*

The five classes, as CSA tabulates them:

| Class | Mechanism | Representative example |
|---|---|---|
| A | Quote removal | `r''m` is rewritten to `rm` |
| B | Variable / `$IFS` expansion | `rm$IFS-rf$IFS/` expands to `rm -rf /` |
| C | Command substitution | `$(echo rm) -rf /` |
| D | Encoded pipelines | `echo <base64> \| base64 -d \| sh` |
| E | Destructive flag variants | `find /x -delete`, `dd of=/dev/sda`, `sed -i` |

CSA further sorts the surveyed guards into four maturity tiers. **Tier 1 — "applies a regular
expression directly to the raw command string" — "was defeated across essentially every test
case."** Tier 2 tokenizes but does not model expansion. Tier 3 has no static guard at all. Tier 4
sandboxes by default but lets a config flag turn the sandbox off.

Caro's safety validator is Tier 1. `SafetyValidator::validate_command`
(`src/safety/mod.rs:459`) matches compiled `regex::Regex` values against the raw `&str`. There is
no `shlex`, no `shell_words`, no tokenizer and no expansion pass anywhere under `src/`
(`Cargo.toml` verified; `grep -rn "shlex\|shell_words" src/` is empty).

Three disclosures in one year — GuardFall (command-level guardrails), the trust-handoff flaw
(2026-07-22), and CVE-2026-82533 (control-plane authentication, CVSS 9.4) — let CSA state the
pattern: *"coding-agent containment is failing at multiple independent layers … within the same
year and the same product category."* The market position Caro has been building toward for two
quarters is *the deterministic component inside somebody else's gate.* That position requires
being the one that does not fail this test.

### 2. Caro is in the failing group, and the number is 99 of 99

This was measured, not inferred. All 67 `DangerPattern` entries were extracted from
`src/safety/patterns.rs`, compiled (67/67 compile cleanly under Python `re`), and driven through a
**faithful line-by-line port of `is_dangerous_in_context`** (`src/safety/mod.rs:432–455`) — the
same match-then-count-preceding-quotes algorithm, including the `\\'` and `\\\"` subtractions.
Eleven canonical destructive commands were used as baselines and mutated across GuardFall classes
A–D plus two Caro-specific classes. No-op mutations were discarded. Harness and full output are in
the run report.

| Class | Description | Applied | Caught | Leaked |
|---|---|---:|---:|---:|
| A1 | quoted argument — `rm -rf "/"` | 4 | 0 | **4** |
| A2 | split verb — `r''m -rf /` | 9 | 0 | **9** |
| A3 | split verb, double quotes — `r""m -rf /` | 9 | 0 | **9** |
| B1 | `$IFS` — `rm$IFS-rf$IFS/` | 11 | 0 | **11** |
| B2 | `${IFS}` | 11 | 0 | **11** |
| C1 | command substitution — `$(echo rm) -rf /` | 11 | 0 | **11** |
| C2 | backtick substitution | 11 | 0 | **11** |
| D1 | base64 pipeline to `sh` | 11 | 0 | **11** |
| F1 | **odd-quote prefix** — `echo "hi ; rm -rf /` | 11 | 0 | **11** |
| F2 | odd single-quote prefix | 11 | 0 | **11** |
| — | **adversarial total (A–F)** | **99** | **0** | **99 — 100%** |
| G1 | *control:* double spaces — `rm  -rf  /` | 11 | 11 | 0 — 0% |

Baselines: **11 of 11 caught** unmutated. The control class confirms the harness is not simply
broken: whitespace elasticity is handled correctly by the `\s+` in the patterns, and every
double-spaced variant is still caught. Every adversarial mutation, without exception, turns a
`Critical` or `High` verdict into `Safe`.

**Class F is not from GuardFall. It is Caro's own, and it is cheaper than anything GuardFall
published.** `is_dangerous_in_context` discards a genuine match when an odd number of unescaped
quotes precedes it. So `echo "hi ; rm -rf /` — one unbalanced quote, no obfuscation of the
dangerous token at all — returns `Safe`. This is worse than having no quote handling, because the
heuristic can only ever *subtract* detections.

### 3. The same heuristic also hides the interpreter sink

`is_dangerous_in_context` was written to suppress false positives: `echo 'rm -rf /'` should not
be flagged, and today it is not. But quote parity cannot distinguish *data* from *code*, so the
same rule produces:

```
sh   -c 'rm -rf /'     →  Safe
bash -c "rm -rf /"     →  Safe
```

A one-token prefix and a pair of quotes. This is Continue's *"inspects pipeline destinations for
interpreters such as `sh` or `python`"* component, absent. The false-negative class and the
false-positive suppression are **the same mechanism**, which is why neither can be fixed by tuning
it — the heuristic asks the wrong question. The right question is not *"is this text quoted?"* but
*"which token is the command word, and is its sink an interpreter?"*

Under that question both cases resolve correctly and for a principled reason: `echo` is not an
interpreter, so its arguments are data; `sh -c` is, so its argument is code.

### 4. Why Continue's answer is the right shape but the wrong last step

Continue was the only agent to survive the survey — all 21 bypass cases and 12 canonical
destructive patterns blocked. Its evaluator has five components: **tokenize with shell-aware
parsing; detect variable expansion; recursively evaluate command substitutions; inspect pipeline
destinations for interpreters; compare the resolved command against an explicit denylist.**

Four of those five are exactly right and this ADR adopts them. The fifth — *recursively evaluate
command substitutions* — is where Continue's own coverage stops being total. CSA records that its
protection held in the default editor mode, *"though its command-line auto-run mode is weaker with
a few payloads slipping through."*

That is the predictable consequence of trying to resolve the unresolvable. `$(curl …)`,
`$RANDOM`, `$1`, a variable set three lines earlier in a script — the post-expansion text is not a
function of the command string. A guard that evaluates substitutions is writing a partial shell
interpreter, and a partial interpreter has two problems: it is wrong on the cases it does not
cover, and it is itself attack surface. The year's other two containment failures were both
*evaluation* bugs in a trusted component — a control API trusting a client-supplied `Host` header,
a trusted process executing a sandboxed agent's file without re-validating.

**The failure mode found in Phase 1 is therefore: treating "could not resolve" as "resolved to
something benign."** This ADR's design answer is D1 and D2 — never resolve, and make
unresolvability a risk input rather than a parse detail.

---

## Decision

Introduce **`caro.lex.v1`**: a deterministic, offline, purely lexical analysis of a command
string, produced before any pattern is matched, whose output is a serializable
`LexicalAnalysis`. Pattern matching moves from the raw string onto that structure.

**D0 — Phase 0 is a failing test, and it ships first and alone.**
`tests/lex_differential.rs` plus a frozen mutation corpus. It adds no type, no dependency, no
verb, no config key. It asserts that a mutated variant carries a verdict no lower than its
baseline, and it is expected to fail loudly — the reproduced number above is 99 of 99. Merging a
large red bar is the point: the number is the finding, and every decision below is contingent on
a reviewer seeing it. **No decision after D0 may be implemented before D0 is merged.**

**D1 — Analysis, never evaluation.** `caro.lex.v1` performs exactly the transformations the shell
performs *deterministically from the command text alone*: quote removal, word splitting on
unquoted blanks, operator recognition, and command-list segmentation. It performs **none** of the
transformations that depend on state Caro does not have: no parameter expansion, no command
substitution, no arithmetic expansion, no globbing, no alias resolution, no `$IFS` reinterpretation.
Caro never executes, spawns, or simulates a shell to answer a safety question.

**D2 — Opacity is a risk input, not a parse detail.** Every construct D1 refuses to resolve is
recorded as an `OpaqueConstruct` with its byte span and the token position it occupies. A command
carrying an opaque construct in a **decisive position** — the command word of any segment, an
operand of a segment whose command word is a known-destructive verb, or a redirection target —
**cannot be routed `AutoApprove`.** Its routing floors at `HumanGate`, regardless of whether any
pattern matched. `rm$IFS-rf$IFS/` and `$(echo rm) -rf /` are stopped by this rule, not by a
pattern, and would still be stopped if the pattern set were empty.

**D3 — Segment scoping replaces quote parity, and `is_dangerous_in_context` is deleted.** The
analysis splits the command into `Segment`s at unquoted `;`, `&&`, `||`, `|`, `&` and newline.
Each segment carries its own token vector. Patterns are matched against a per-segment canonical
form, so no construct anywhere else in the line can suppress a match. Class F ceases to exist as a
category, and the function whose doc comment lists its own limitations
(`src/safety/mod.rs:417–423`) is removed rather than patched.

**D4 — Interpreter sinks are recursed into, but only on literal payloads.** A fixed, versioned
`INTERPRETER_SINKS` set (`sh`, `bash`, `zsh`, `dash`, `ksh`, `python`, `python3`, `perl`, `ruby`,
`node`, `eval`, `source`, `.`) is recognised in two positions: as a segment command word taking a
`-c`-style payload, and as the destination of a pipe or redirect. When the payload is a **fully
literal** token — no opaque construct inside it — it is re-analysed recursively to a bounded depth
(**3**, then `Opaque`), and the inner verdict is lifted to the outer command. When the payload is
not fully literal, it is **not** decoded, guessed, or evaluated: it is recorded as
`OpaqueConstruct::InterpreterPayload` and D2 applies. `sh -c 'rm -rf /'` is caught by recursion;
`echo <b64> | base64 -d | sh` is caught by D2, because the interpreter's input is opaque.

**D5 — Unparsable is never `Safe`.** If the tokenizer cannot produce a token vector — unterminated
quote, unbalanced substitution, invalid byte sequence — the result is
`ParseOutcome::Unparsable { reason }`. Legacy raw matching still runs and its verdict is retained
if it is higher, but the floor is `HumanGate` and the `caro lex` verb exits **3**. A command Caro
cannot read is never a command Caro approves.

**D6 — Non-POSIX shells are declared unsupported, not silently claimed.** `ShellType::PowerShell`
and `ShellType::Cmd` have different quoting and expansion rules, and a POSIX lexer applied to them
would be confidently wrong. For those, `LexicalAnalysis::support` is `Unsupported`, the legacy raw
path runs unchanged, and the analysis says so in a field callers can read. `Bash`, `Zsh`, `Sh` and
`Unknown` are `Supported`. `Fish` is `PartialPosix` in v1 — tokenized, but `Opaque` on any `(` it
sees, because fish command substitution has no `$` marker.

**D7 — Serializable from day one, and versioned.** Every type derives
`Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq`; every enum is
`#[serde(rename_all = "snake_case")]` and `#[non_exhaustive]`. `LEX_SCHEMA: &str = "caro.lex/1"`
is a required, literal-checked field. `LexicalAnalysis` is designed to be embedded as a field of
`caro.assessment.v1` when that module exists, never to become it.

**D8 — The tokenizer arrives behind a build spike and a feature flag.** Per
[`.claude/rules/external-sdk-integration.md`](../../.claude/rules/external-sdk-integration.md),
the first PR touching an external parser is a ≤100-LOC spike: license direction, MSRV, optional
dep, one code reference that forces compilation, two verification builds. `brush-parser` 0.4.0 is
the candidate (MIT — clean absorption into AGPL-3.0; `tokenize_str`, `unquote_str`, a full
POSIX/bash AST, optional `serde`). Its MSRV is **not published on docs.rs and must be measured by
the spike** against caro's `rust-version = "1.85"`. `yash-syntax` is rejected on license direction
(GPL-3.0-or-later would make the safety module's license a per-embedder question, which is the
opposite of the strategy). If the spike fails on MSRV or dependency weight, the documented fallback
is `shlex` (MIT/Apache-2.0, minimal) for quote removal and word splitting, plus a hand-written
**detector** — not evaluator — for classes B, C and D. Detection of `$`, `` ` ``, `$(`, `${`,
`<(` and `$((` is a lexical scan and is safe to hand-roll; expansion is not, and is never
hand-rolled here.

**D9 — No new exit code.** The validator change is internal and alters no process contract. The
one new verb, `caro lex <command> -o json`, is an *analysis* verb, not a gate: it reports what the
shell will do, not whether it may. It reuses the meanings already in the tree — `0` analysis
produced, `1` I/O or internal error, `2` usage, `3` could not analyse (D5), matching ADR-063's
"the harness could not measure" sense of 3. Four Proposed verbs already claim codes independently;
this ADR deliberately claims none, and the repository-wide exit-code table (ADR-024) remains the
outstanding debt it has been since June.

---

## Consequences

**Good.** The single claim Caro makes that nothing else in the market makes — deterministic,
intent-aware validation of what a POSIX command actually does — becomes true against adversarial
input rather than only against well-formed input. The claim also becomes *checkable*: D0 produces
a number a buyer can reproduce, and a competitor running the GuardFall corpus against Caro in a
bake-off finds what the repository already published. The false-positive suppression that
`is_dangerous_in_context` was written for survives, for a better reason: `echo 'rm -rf /'` is safe
because `echo` is not an interpreter, not because a quote count came out odd.

**Costs, stated plainly.**

- *D2 will raise the confirmation rate, and nobody has measured by how much.* Every command
  containing `$VAR`, `$(…)` or a glob in a decisive position stops being `AutoApprove`. In
  ordinary developer use that is a large fraction of commands — `cd $HOME`, `kill $(pgrep foo)`,
  `rm *.log`. **This is the single biggest risk in the ADR and the reason Gate 3 exists below.**
  Mitigation is scoped, not hand-waved: D2's floor is `HumanGate`, never `Block`; the
  decisive-position rule is deliberately narrower than "anywhere in the command"; and the
  false-positive rate must be measured on a real command corpus **before** the flag flips to
  default, not after. If the measured rate is unacceptable, the correct outcome is that D2 ships
  gated and off, with the number published — not that D2 is quietly widened.
- *A second matching path exists during the transition.* Until the flag is default, raw matching
  and lexical matching both run, and they can disagree. The disagreement is the point — it is what
  the differential test reports — but it is also two code paths to maintain and a period in which
  the answer depends on a flag.
- *`caro.lex.v1` cannot close class E.* `find / -delete` and `sed -i` are not obfuscations; they
  are different commands that the pattern set does not know. A tokenizer makes them no easier to
  catch. Class E belongs to ADR-063's corpus and to pattern coverage, and this ADR must not be
  read as claiming it.
- *Recursion is a bounded lie.* Depth 3 is a guess. A payload nested four deep is reported
  `Opaque` and floors at `HumanGate`, which is safe but imprecise.
- *An external parser is new supply-chain surface in the safety path.* `brush-parser` pulls
  `bon`, `cached`, `peg`, `winnow`, `getrandom` and `uuid`. That is the explicit trade being made
  against a hand-rolled lexer, and it is the trade CSA's four-tier analysis argues for: the
  agents that hand-rolled tokenizers landed in Tier 2 and still leaked.

---

## Alternatives Considered

1. **Patch `is_dangerous_in_context` to balance quotes properly.** Cheapest, and fixes class F
   only. Classes A through D are untouched, because they are not quoting bugs — they are the
   absence of tokenization. Rejected: it would let the repository believe the problem was fixed.
2. **Normalize by canonicalizing the string (strip `''`, `""`, collapse `$IFS` to a space) and
   keep matching raw.** Attractive because it is ~40 LOC. Rejected: a normalizer that rewrites
   `$IFS` to a space is an evaluator that has guessed a variable's value, and it is wrong the
   moment `IFS` has been reassigned. It also cannot express opacity, so it must silently choose
   between fail-open and fail-closed on every construct it does not understand — the exact
   decision D2 exists to make explicit.
3. **Adopt Continue's evaluator wholesale, including recursive substitution evaluation.** It is
   the only design with a published passing score. Rejected on the evidence in its own survey
   entry: the mode where it evaluates most aggressively is the mode where payloads slipped
   through. D1/D2 take the four components that are decidable and refuse the fifth.
4. **Execute the command under `bash -n`, or in a throwaway sandbox, and observe.** Highest
   fidelity by construction. Rejected: it makes Caro's safety answer depend on spawning a shell,
   which is both a latency and an execution-surface regression, and CVE-2026-82533 is this year's
   demonstration of what happens when the analysis layer has a live control path.
5. **`yash-syntax` as the parser.** Mature, POSIX-focused, good AST. Rejected on license
   direction: GPL-3.0-or-later in the safety path turns "can I embed Caro's validator?" into a
   legal question for every downstream, which contradicts the component-inside-someone-else's-gate
   strategy. `brush-parser` is MIT.
6. **Do nothing until `caro.assessment.v1` exists.** Five consecutive memos have recommended the
   decision contract and it has not shipped. Rejected: publishing a portable decision contract out
   of a validator that a pair of quotes defeats exports the defect instead of fixing it.

---

## Out of scope (belongs to the next version)

- **Class E coverage** — new patterns for `find -delete`, `sed -i`, `truncate`, `shred`. Pattern
  work, tracked against ADR-063's corpus.
- **The `--approval smart` judge's interaction with opacity.** `blend_smart_decision`
  (`src/safety/mod.rs:277`) lets a judge relax `High` and `Moderate`. Whether a judge may relax a
  D2 opacity floor is a real question and is **deliberately unanswered here**; until it is
  answered, the conservative reading is that it may not. Successor to the market memo's §3D.
- **Windows shells.** D6 declares `Unsupported`; a PowerShell lexer is its own ADR.
- **Argument-level semantics** — knowing that `-rf` implies recursion, or that `/` is the root.
  `caro.lex.v1` produces tokens; interpreting them is the pattern layer's job in v1 and an
  argument-model's job later.
- **Caching.** `brush-parser` has a `cached` dependency; using it is a latency decision that
  belongs with ADR-062's hook-path latency budget, not here.
- **Emitting `LexicalAnalysis` over OTel or into receipts.** Blocked on the decision contract.

---

## Validation discipline

Assessed against [`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md).

- **Gate 1 (20 transcripts) — not applicable, and the rule says so.** The rule exempts work that
  is "a response to evidence we already have (broken thing …)" and exempts the core
  natural-language → safety-validated-POSIX loop from the forward-dated requirement. This ADR is a
  correctness fix to that loop, evidenced by a published third-party survey and a reproduced
  measurement, not a new product line. **A reviewer who disagrees should reject on this specific
  ground**, because the exemption is the load-bearing claim that lets this document exist without
  interviews.
- **Gate 2 (no surveys as sole evidence) — met.** No survey was used. The evidence is GuardFall's
  test results and a first-party measurement of this tree.
- **Gate 3 (what breaks at 100 real users) — see the scope document, §7.** The honest answer is
  *the confirmation rate*, and the assumption at risk is that opaque constructs are rare in real
  commands. They are not. The instrumentation, the threshold, and the fallback are specified
  there; the threshold must be met before D2 leaves the feature flag.
- **Gate 4 (devil's-advocate review) — pending.** Required before merge. The three questions to
  put to it: is the Gate 1 exemption legitimate; is D2's decisive-position rule narrow enough to
  be usable; and is D8's external-parser trade correct given that hand-rolled tokenizers put
  Cline and Roo-Code in Tier 2 rather than Tier 4.
- **Gate 5 (PMF claim) — N/A.** No PMF claim is made.

---

## Files changed

Seven, three of them new. One new dependency, optional and feature-gated, arriving in its own
spike PR per D8.

| # | File | Change | Phase |
|---|---|---|---|
| 1 | `tests/lex_differential.rs` | **new** — the differential harness, expected red | **0** |
| 2 | `tests/fixtures/guardfall/mutations-v1.yaml` | **new** — frozen mutation corpus + canaries | **0** |
| 3 | `Cargo.toml` | `brush-parser` optional + `shell-lex` feature (not in `default`) | 1 (spike) |
| 4 | `src/safety/lex.rs` | **new** — all types, `analyze()`, `INTERPRETER_SINKS`, ~400 LOC + tests | 2 |
| 5 | `src/safety/mod.rs` | `pub mod lex;` + re-export; `is_dangerous_in_context` **deleted**; `validate_command` matches per-segment; D2 routing floor | 2 |
| 6 | `src/main.rs` | `Commands::Lex { … }` variant (enum at `:381`) + one dispatch arm | 3 |
| 7 | `tests/lex_contract.rs` | **new** — the 14 contract tests, scope §6 | 2 |

`src/safety/patterns.rs` is **not** changed. `src/safety/cve_patterns.rs` is **not** changed. The
pattern set is not the defect.

---

## Sequencing

| Phase | Deliverable | Gate to the next phase |
|---|---|---|
| 0 | Differential test + corpus, merged red | A published number |
| 1 | `brush-parser` build spike, ≤100 LOC, feature off | Two green verification builds; MSRV ≤ 1.85 |
| 2 | `src/safety/lex.rs` + validator rewire, behind `shell-lex` | Differential test green; FP rate measured (Gate 3) |
| 3 | `caro lex` verb; flag to `default` | Gate 3 threshold met and published |

Phase 0 is independent of every decision in this document and should be a pull request this week
whether or not the rest of the ADR is accepted.

---

## Amendment — 2026-09-22: D0 delivered, and D0's merge instruction was wrong

Added by the `caro-research--scoping-process` run of **2026-09-22** (autonomous, no user
present). The 2026-09-21 run report asked the next run to *"merge phase 0, then stop scoping
for a while"*, and said that if the next run scoped again it should open with why phase 0 did
not merge. It did not merge. This run wrote it instead of scoping something new.

**Why it did not merge: D0 as written is unmergeable.** D0 says the differential test "is
expected to fail loudly" and that "merging a large red bar is the point."
`.claude/rules/dev-process.md` requires all unit tests to pass before review. A pull request
whose entire contribution is a failing `cargo test` cannot clear that bar, and nobody can
approve it without suspending a Tier 2 rule. The instruction and the constitution were in
direct conflict, and the conflict — not the difficulty of the work; the harness is 565 lines
and compiled on the first attempt — is the most likely reason the window passed unused.

**D0 is amended as follows.** The red bar is kept in full and moved behind `#[ignore]`:

| Test | State | Role |
|---|---|---|
| `corpus_integrity` | green | corpus well-formedness; no validator involved |
| `canaries_stay_safe` | green | the false-positive floor phase 2 may not breach |
| `controls_are_still_caught` | green | class G still caught; proves the gap is real |
| `census_is_reported` | green | prints the census every CI run; asserts no verdict |
| `mutation_never_lowers_verdict` | red, `#[ignore]` | **the bar**; phase 2 turns it green |
| `interpreter_sinks_are_classified` | red, `#[ignore]` | D4 |
| `flag_variants_are_classified` | red, `#[ignore]` | GuardFall class E |
| `known_false_positives_are_fixed` | red, `#[ignore]` | over-matching, found by this run |

The number still reaches the reviewer — `census_is_reported` prints the full census on the
green path, so every CI log carries it — and `cargo test --test lex_differential -- --ignored`
reproduces the red bar on demand. **"No decision after D0 may be implemented before D0 is
merged" is unchanged.**

### The measurement

First execution inside the real pipeline (`SafetyValidator::validate_command`,
`SafetyConfig::moderate()`, `ShellType::Bash`, `--no-default-features --features cve-rules`):

```
family  applied  caught   leaked
  A          22       0       22
  B          22       0       22
  C          22       0       22
  D          11       0       11
  F          22       0       22
  G          11      11        0   (control)

adversarial (A,B,C,D,F): 99 leaked of 99
```

**The 2026-09-21 Python port was accurate to the case.** 99 of 99 reproduced, 99 of 99
measured. The caveat that figure carried — "a reproduction, not a test result" — is
discharged. CVE rules are compiled in and change nothing.

### Two findings the scoping run did not have

1. **`# rm -rf /` is `Critical`.** A shell comment. Inert by definition, blocked by the
   product. Same root cause as the leaks — text matching without shell semantics — with the
   sign reversed. Recorded as `known_false_positives` in the corpus rather than as a canary,
   so the green path stays green while the finding stays in the repository. This is direct
   evidence for the *Consequences* worry that D2 will raise the confirmation rate: some of
   that rate is already being paid, for nothing.
2. **Class E is a total miss, not a partial one.** `find /etc -name '*.conf' -delete`,
   `sed -i 's/.*//' /etc/passwd`, `shred -u /etc/shadow`, `truncate -s 0 /var/log/syslog` —
   all four return `Safe`. Class E is on this ADR's out-of-scope list as "pattern coverage."
   That framing survives: lexing would not catch these, and a pattern would. But the
   out-of-scope list should now say *four of four measured cases leak*, not *coverage is
   incomplete*.

`sh -c 'rm -rf /'` and its three siblings return `Safe`, exactly as D4 predicted.

### What did not change

No decision D1–D8 is amended. No type, dependency, verb or configuration key was added.
`src/` is untouched. Phase 1 remains the `brush-parser` build spike and remains gated on D0
being merged — which now means merged **green**, with the bar reproducible by one flag.
