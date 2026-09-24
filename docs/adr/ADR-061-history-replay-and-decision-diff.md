# ADR-061: `caro replay` — Offline Rehearsal Over the User's Own Shell History, With a Verdict Diff

- **Status**: Proposed (implementation ADR — this one is meant to become code)
- **Date**: 2026-08-31
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **`gator`**, the Open Policy Agent Gatekeeper offline policy CLI
  (`gator test` / `verify` / `sync test` / `bench`), documented as **"v3.11+ (beta)"**, plus
  Gatekeeper's four-enforcement-point action model — read live 2026-08-31 at
  [open-policy-agent.github.io/gatekeeper/website/docs/gator/](https://open-policy-agent.github.io/gatekeeper/website/docs/gator/),
  `/docs/enforcement-points/`, `/docs/audit/`, and issues
  [#2863](https://github.com/open-policy-agent/gatekeeper/issues/2863),
  [#2945](https://github.com/open-policy-agent/gatekeeper/issues/2945) (open),
  [#3023](https://github.com/open-policy-agent/gatekeeper/issues/3023)
- **Cross-read**: **Kyverno CLI** (`kyverno.io/docs/subprojects/kyverno-cli/`; issues #4255,
  #7681, #9804), **Conftest** (`conftest.dev/options/`; issue #387), **Envoy RBAC
  `shadow_rules`**, **AWS IAM Access Analyzer `check-no-new-access`**, and — as unverified
  vendor marketing, load-bearing nowhere — **Kastra**'s Product Hunt founder comments
- **Implements**: ADR-023's exit-code and JSON contract, over one input source. Does **not**
  supersede or redesign ADR-023
- **Unblocks**: nothing formally, and that is the point — this ADR depends on no unlanded ADR
- **Depends on**: `SafetyValidator` and `ValidationResult`, both shipped in 1.4.0. No new
  crate, no new module tree, no daemon, no network
- **Relates to**: ADR-056 (this is its unblocked subset), ADR-060 (`--stable` byte-equality
  pattern reused verbatim), ADR-059 (the moratorium this ADR stays outside of)
- **Full scope document**: `caro-scope-history-replay-2026-08-31.md`
- **Numbering note**: highest existing is ADR-060. Per `.claude/rules/adr-numbering.md`,
  renumber on merge if another 061 lands first
- **Exit-code note**: this ADR claims **no new exit code.** It adopts ADR-023's table
  (`0/1/2/3/10/11`) verbatim. The contended registry that ADR-053 and ADR-056 complain
  about gets no new entrant here

> **Provenance and boundaries.** No user was present; the task template's `[FEATURE NAME]`
> was unbound, so target selection was mine, and it was constrained twice.
>
> **ADR-059 declared itself "the last ADR in this space until `src/safety/assessment.rs`
> merges."** Re-verified 2026-08-31: `src/safety/` contains `cve_patterns.rs`, `mod.rs`,
> `patterns.rs` and nothing else. The moratorium holds. This ADR is outside it — it defines
> no policy vocabulary, no verdict payload, no approval exchange, no lifecycle event. It
> reads a file and calls a validator that shipped seven weeks ago.
>
> **Today's strategy memo** (`.hermes/digests/2026-08-31-agent-launch-scan.md`) names
> `caro replay --history` as the single "build or test next" item and instructs that it
> **not** be coupled to the unlanded assessment schema. This ADR follows that instruction
> literally: the only types it touches are ones that exist in `src/` today.
>
> **Validation-discipline note.** Gate 3 (demoware trap) is discharged in scope §3.7.
> **Gate 1 is not discharged**, and the PR must claim the diagnostic-on-existing-behaviour
> exemption explicitly and argue it — see scope §3.9. Gate 4 (`devils-advocate`) is required
> before the implementation PR opens.

---

## Context

### The problem

Nobody enables a gate that can refuse things on day one. The person evaluating Caro has a
working shell and is being asked to install something whose only observable effect, until
the day it saves them, is friction. Caro's strongest asset — 52+ deterministic patterns
with a zero-false-positive record — is currently a **claim in a README**. A claim is
answered with skepticism; a diff against the user's own history is answered by looking.

Every mature policy engine surveyed reaches the same conclusion and puts the answer in the
onboarding path. Envoy states it in one line: *"shadow mode won't affect real users, it is
used to test that a new set of policies work before rolling out to production."* Gatekeeper
formalizes it as four enforcement points with per-point actions so a rule can be `deny` in
shift-left CI and `warn` at live admission. Kyverno ships `Audit` vs `Enforce` plus
`--audit-warn` / `--warn-exit-code` so the same corpus is a warning this quarter and a hard
failure next.

### What we can do that they cannot

gator, Kyverno and Conftest replay **authored configuration manifests**. Kastra (unverified)
replays **its own stored decisions** — a corpus that exists only because Kastra was already
deployed, so you must adopt before you can evaluate.

Caro replays `~/.zsh_history`. **The corpus predates Caro.** A person who has never run
`caro` can, in ten seconds, see every command they personally typed that Caro would have
stopped. That inverts the funnel from adopt-then-measure to measure-then-adopt, and no
product in the survey can do it.

Two things follow. First, replay is only meaningful because the validator is
**deterministic** — an LLM-judged safety layer cannot offer it, because two runs over the
same corpus are not guaranteed to agree and a diff is therefore not attributable to the
policy change. Second, the corpus is the user's shell history, which contains live
credentials; any hosted competitor asking for it is asking for something no security team
approves. Local-first stops being a preference here and becomes the only viable
architecture.

### The failure modes we are designing against

Phase 1 of the scope document found five, each with a citation. Three matter to every
decision below.

**FM-1 — the rehearsal silently defaults context the live path actually had.** gator:
*"Gator cannot determine if a type is Namespace-scoped or not, so it does not assign
objects to the default Namespace automatically."* Gatekeeper: `input.review.userInfo`
*"cannot be populated … for audit reviews."* Kyverno: the CLI *"does not embed the
Kubernetes control plane components."* Kastra's self-disclosed version is the clearest:
*"pre-inference rules replay exactly, but rules matching raw prompt content don't, since we
store prompts only hashed."* Three independent tools, one bug — and all of them **default**
the missing input rather than declaring it unknown.

**FM-2 — "the tool broke" and "the tool found something" share an exit code.** gator
documents this against itself: *"An error during evaluation … will result in a `1` exit
status … Policy violations will generate a `1` exit status as well."* Conftest's default is
the same shape. AWS returns `{"result": "FAIL"}` inside a *successful* API response and
documents no process exit code at all.

**FM-3 — an item that could not be evaluated is counted as passing.** Kyverno CLI before
v1.14 hit an `apiCall` with no cluster, logged `disabled loading of APICall context entry`,
skipped the rule, and reported the resource as passing; the doc's own retrospective calls
the result *"misleading pass/fail counts in CI."*

### What is already in the tree, and three things that are broken in it

`SafetyValidator::new` / `validate_command` (`src/safety/mod.rs:344`, `:459`),
`ValidationResult` (`:175`, `Serialize + Deserialize`),
`SafetyDecision::from_validation_result` (`:225`, already computes `SuggestedRouting`),
`OutputFormat` (`src/cli/mod.rs:105`), `schemars 0.8` (`Cargo.toml:46`) and
`src/bin/generate-schema.rs` all exist. There is exactly one validator implementation, so
Caro starts where gator ended up: no offline/online engine divergence is possible.

Absent, each verified by grep on 2026-08-31: no `Scan` / `Validate` / `Replay` subcommand;
**no shell-history reading anywhere in `src/`**; no `tests/scan_integration.rs`; no
`schemas/`; no `enum ExitCode`; no `RuleId`.

Three defects fell out of reading `src/safety/` against the failure modes above:

- **DEF-1** — the allowlist is recompiled **per command** (`src/safety/mod.rs:499`:
  `if let Ok(regex) = regex::Regex::new(allow_pattern)`), while custom patterns are compiled
  once in `new()` and cached at `:161`/`:378`. Over a 20,000-line history that is 20,000 × M
  needless regex compilations, in the exact loop this feature introduces.
- **DEF-2** — the same `if let Ok(...)` silently discards an invalid allowlist regex. A user
  with a typo gets stricter behaviour than configured and is never told. This is FM-3 in
  Rust.
- **DEF-3** — `COMPILED_PATTERNS` (`src/safety/patterns.rs:551`) uses
  `filter_map(|p| Regex::new(&p.pattern).ok().map(...))`, so a built-in pattern that fails
  to compile is **silently dropped**. `validate_patterns()` prints `WARN:` and continues.
  Net effect: **the "52+ patterns" claim is not enforced at runtime** and coverage can
  shrink with every test still green. Relatedly: `DANGEROUS_PATTERNS` holds **67** entries
  today while `src/safety/mod.rs:8` still documents "52 pre-compiled regex patterns" —
  nothing in the tree knows the real number, which is how DEF-3 stays invisible. D9's
  assertion is relative (`COMPILED_PATTERNS.len() == DANGEROUS_PATTERNS.len()`), not a
  hard-coded count, so it cannot rot the same way.

---

## Decision

Ship **`caro replay`**: a pure subprocess that reads a corpus of commands the user already
ran, loops the existing `SafetyValidator` over it, and emits a deterministic report with an
exit code, plus an optional diff against a stored baseline.

**D1 — `caro replay` is the first implementation of ADR-023's contract, not a second
contract.** Exit codes, `--fail-on`, `schema_version` and the JSON envelope are ADR-023's,
adopted verbatim. `caro scan` remains Proposed and, when it lands, is the same code path
with a different input source.

**D2 — the record carries its own context; unknown context is `null`, never defaulted.**
The answer to FM-1. `shell` comes from `--shell`, else from *which history file was read*
(`~/.zsh_history` ⇒ `Zsh`), else `ShellType::detect()`, else `Unknown` — and the report
header records both the value and its `ContextProvenance`. `cwd`, `timestamp` and
`exit_status` are `null` unless the source format carries them. **A verdict that depends on
a `null` field is `Indeterminate`, not a guess.**

**D3 — five outcomes, borrowed from Kyverno, because four is the bug.**
`Clean | Flagged | Indeterminate | Errored | Skipped`. Nothing is ever folded into `Clean`.

**D4 — a reconciliation invariant, enforced rather than documented.**
`total == clean + flagged + indeterminate + errored + skipped` is asserted before every
emission. On violation, stderr gets the discrepancy and the process exits `11` — it does
not print an under-counting report.

**D5 — ADR-023's exit table, unchanged, plus one flag.** `0` clean · `1` block · `2` human
gate · `3` async-log warn · `10` input error · `11` internal error. `--fail-on-indeterminate`
maps onto **2**, not a new code, because "a human must look at this" is what 2 already
means. This closes gator #2945 by construction and never conflates FM-2's two cases.

**D6 — deterministic by construction, and tested for it.** Records emit in **corpus order**
(re-sorting destroys the thing the user recognizes). All maps are `BTreeMap`. The report
body carries no timestamp, path, duration or hostname; `--stable` zeroes the header's
`generated_at` and `caro_version` so two runs are **byte-identical**, asserted in CI. Diffs
key on `(index, command_digest)`, never on message text — gator's Rego-dedupes-on-message
gotcha is designed out.

**D7 — redaction is the default; raw command text is opt-in.** JSON carries
`command_digest` and a redacted rendering; `--include-raw` is required for raw text and
flips `redacted: false` in the header. Human-readable output prints the command, because it
goes to the user's own terminal and recognition is the entire demo.

**D8 — diff mode is verdict-level in v1.** `--baseline <report.json>` yields
`unchanged | flipped_stricter | flipped_looser | added | removed`. `--fail-on-flip` defaults
to `looser`, because a policy edit that newly *allows* something previously blocked is the
regression worth failing on. Rule-level diffing needs stable rule identity, which Caro does
not have — see Consequences.

**D9 — DEF-1 and DEF-3 are fixed in this PR.** Allowlist regexes compile once in `new()`
into a `compiled_allowlist` field; a compile failure becomes a loud
`ValidationError::PatternError`, consistent with how custom patterns are already handled
(fixing DEF-2 as a side effect). A test asserts
`COMPILED_PATTERNS.len() == DANGEROUS_PATTERNS.len()`. Neither change alters any existing
verdict; both are directly in this feature's hot path, per
`.claude/rules/good-boy-scout.md`.

### Files

Nine, one generated, two documentation: `src/safety/replay.rs` (new), `src/safety/mod.rs`,
`src/safety/patterns.rs`, `src/main.rs` (one enum variant, one dispatch arm),
`src/bin/generate-schema.rs`, `schemas/caro.replay.v1.json` (new, committed),
`tests/replay_contract.rs` (new), plus `README.md` and `CLAUDE.md`. No new module tree, no
new crate, no new dependency.

### Types

`ReplayReport { schema_version, header, summary, records, diff }` with `ReplayHeader`,
`ReplaySummary`, `ReplayRecord`, `RecordContext`, `RecordOutcome`, `ContextProvenance`,
`CorpusSource`, `ReplayDiff`, `RecordFlip`, `FlipDirection` — all deriving
`Serialize + Deserialize + JsonSchema + PartialEq` from day one, per the standing
constraint. `ValidationResult` gains `JsonSchema` (one derive) so the generator can reach
it. Full field lists and method contracts in scope §3.3.

---

## Consequences

### Positive

- The strongest existing asset becomes personally verifiable in ten seconds, on a corpus
  that requires no prior adoption of Caro. No competitor in the survey can do this.
- Determinism acquires a *demonstration* rather than a claim, at the exact moment the
  market has started arguing about deterministic versus LLM-judged safety.
- ADR-023's contract gets exercised in code for the first time since 2026-06-12, which
  narrows the ADR-backlog-versus-`src/`-tree gap the 2026-08-26 strategy memo named as the
  project's largest risk — with a feature that depends on **no** unlanded ADR.
- Three real defects in the shipped safety path are fixed, one of which (DEF-3) means the
  "52+ patterns" claim becomes mechanically enforced instead of aspirational.
- `caro scan` (ADR-023) drops to a small follow-up PR: same code, different input source.

### Negative / risks

- **Redaction is best-effort and this is the most serious risk in the feature.** A secret
  shaped like nothing in the deny-list — a bare token as a positional argument — will
  survive into a report a user might paste into a GitHub issue. Mitigated by omitting
  `command` from JSON entirely by default (the digest is what the diff needs) and by
  labelling redaction as best-effort in the docs. Not eliminated.
- **`matched_patterns` is prose, so v1 cannot diff at rule level.** It holds
  `description.to_lowercase()` (`src/safety/mod.rs:523`, `:537`, `:550`); a typo fix silently changes the
  diff key. D8 confines v1 to verdict-level diffing and the scope names `RuleId` as the top
  follow-up rather than half-building it.
- **Gate 1 is not discharged.** The feature ships on a claimed exemption
  (diagnostic-on-existing-behaviour) that a reviewer may reject, in which case it waits for
  the interviews.
- **`--fail-on-flip looser` will fail CI for a user who deliberately relaxes a pattern.**
  Defensible default, one reviewer opinion away from being wrong.
- History parsing is a long tail — zsh `EXTENDED_HISTORY`, fish's format, multi-line
  continuations, non-UTF-8 bytes. Mis-parsing distorts the single headline percentage the
  feature exists to produce. Mitigated by making `skipped` a first-class count with a
  reason and warning above a 5% skip ratio; not eliminated.
- One more subcommand on an already-25-variant `Commands` enum.

### Neutral

- The exit-code registry gains no entrant, so ADR-053's and ADR-056's complaint that it
  needs an owner is neither helped nor worsened.
- `ReplayRecord` will gain one additive field when `caro.assessment.v1` eventually lands;
  `schema_version` stays `1`.

---

## Alternatives Considered

**A1 — implement `caro scan` (ADR-023) first, then layer replay on it.** Rejected on
sequencing, not on merit. `caro scan`'s natural corpus is script files in CI, which nobody
has asked for; replay's corpus is on every developer's disk today. Same code, better first
user. `caro scan` follows in a small PR (OOS-4).

**A2 — wait for ADR-056 to unblock.** Rejected. ADR-056 has been blocked on ADR-024's
`ExitCode` enum and ADR-032's `ExecutionReceipt` for ten days, both still at zero hits in
`src/`. Waiting on unlanded ADRs is the pattern this project has too much of. ADR-061 takes
the subset that needs neither and leaves the transcript corpus and hash chain to ADR-056.

**A3 — human-readable only in v1, JSON later** (today's memo's own recommendation).
Rejected with reasons stated in scope §3.4: ADR-023 already froze the JSON envelope and the
exit table, and `ValidationResult` already derives `Serialize`. The expensive part of a
contract is deciding it, and that decision was made on 2026-06-12. Emitting it is a derive
and a `to_string_pretty`. Shipping human-readable-only would leave the contract unexercised
for a third month.

**A4 — mint a new exit code for `indeterminate`.** Rejected. The registry is already
contended (two ADRs claim 13; ADR-053 claims 14; ADR-055 claims an unnumbered one).
"Indeterminate" is semantically "a human must look at this," which is exactly ADR-023's
code 2.

**A5 — default the missing shell to `Bash` and evaluate anyway.** Rejected: this is FM-1
exactly, and it is the bug three shipping tools document against themselves. An
`Indeterminate` count that a user can see is strictly better than a confident wrong verdict
they cannot.

**A6 — sort records by risk so the interesting ones are first.** Rejected for the default.
Corpus order is what makes the output recognizable as *the user's own history*, and D6's
byte-equality guarantee is cheaper to hold when order is an input property. A `--sort`
flag is a fine follow-up.

**A7 — build `caro replay` as a new `src/replay/` module.** Rejected. It is one file that
loops an existing validator. A new module tree for that is the failure mode the strategy
memo named.

**A8 — ship replay and defer DEF-1.** Rejected. DEF-1 is a per-command regex recompilation
that is invisible at one command and quadratic-feeling at 20,000. Replay is the feature that
exposes it; shipping the exposure without the fix would make the flagship demo slow on
exactly the largest, most impressive corpora.

---

## Implementation Checklist

- [ ] `src/safety/mod.rs`: `compiled_allowlist` in `new()`; invalid allowlist regex ⇒
      `ValidationError::PatternError` (DEF-1, DEF-2)
- [ ] `src/safety/patterns.rs`: `COMPILED_PATTERNS.len() == DANGEROUS_PATTERNS.len()` test (DEF-3)
- [ ] `src/safety/mod.rs`: `JsonSchema` on `ValidationResult`; `pub mod replay;`
- [ ] `src/safety/replay.rs`: types (scope §3.3), `reconcile()`, `exit_code()`,
      `diff_against()`, zsh `EXTENDED_HISTORY` parsing, redaction deny-list
- [ ] `src/main.rs`: `Replay` variant + dispatch arm only; no logic in `main.rs`
- [ ] `src/bin/generate-schema.rs` + committed `schemas/caro.replay.v1.json`
- [ ] `tests/replay_contract.rs`: 18 cases from scope §3.6
- [ ] `benches/`: 10,000-record corpus, before/after DEF-1
- [ ] PR description: explicit gate-1 exemption claim, argued (scope §3.9)
- [ ] **Gate 4**: `devils-advocate` review posted on the PR before merge
- [ ] Feature branch per `.claude/rules/git-workflow.md`

---

## Open Questions

1. ~~**Is `sha2` or `blake3` a direct dependency?**~~ **Resolved 2026-08-31**:
   `sha2 = "0.10"` at `Cargo.toml:90`. `command_digest` adds no dependency.
2. **Does `--fail-on-flip looser` default correctly?** It reds the CI of a user who
   deliberately relaxes a pattern.
3. **`caro replay` on a machine with no history file — exit 10, or a friendly exit 0?**
   Contract says 10; the flagship demo then fails loudly in a fresh container. Recommend 10
   with a human hint on stderr.
4. **Does ADR-023's author accept D1?** This ADR implements that contract without
   superseding it; if the exit table is wrong, now is when to say so.

---

## References

- `caro-scope-history-replay-2026-08-31.md` — full Phase 1–3 scope with line-level evidence
- `.hermes/digests/2026-08-31-agent-launch-scan.md` §3.1, §4 — target selection and the
  dependency-decoupling instruction
- ADR-023 — `caro scan`; exit-code contract adopted verbatim in D1/D5
- ADR-056 — enforcement rehearsal; this ADR is its unblocked subset
- ADR-059 — the moratorium this ADR stays outside of
- ADR-060 — the `--stable` byte-equality pattern reused in D6
- `.claude/rules/validation-discipline.md`, `.claude/rules/good-boy-scout.md`,
  `.claude/rules/adr-numbering.md`, `.claude/rules/git-workflow.md`
- gator, Kyverno CLI, Conftest, Envoy RBAC, AWS Access Analyzer — URLs in the scope §3.12
- Kastra — Product Hunt founder comments; **unverified vendor marketing**, cited once,
  load-bearing nowhere
