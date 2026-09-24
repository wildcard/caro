# ADR-066: `caro.confinement.v1` — Answer Whether a Command's Path Operands Stay Inside Their Roots

- **Status**: Proposed (implementation ADR — meant to become code, and deliberately buildable on
  the tree that exists today, not on the paper stack)
- **Date**: 2026-09-07
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **Claude Code restricted mode** — the `--restricted` flag /
  `CLAUDE_CODE_RESTRICTED=1` shipped in v2.1.248 (2026-08-27), which "removes the built-in tools
  that run commands or code and WebFetch (unless named in `--tools`), keeps file tools inside the
  working directory, refuses `bypassPermissions`, and ignores user, project and local settings
  files" — together with the path-rule language it fences with: gitignore-syntax `Read()`/`Edit()`
  rules, the `//`, `/`, `~/` and `./` anchors, `additionalDirectories`,
  `permissions.blockReadsOutsideWorkingDirectories`, and the Bash redirect-target check.
  Read 2026-09-07 at [code.claude.com/docs/en/permissions](https://code.claude.com/docs/en/permissions)
  and [code.claude.com/docs/en/whats-new](https://code.claude.com/docs/en/whats-new); flag text
  cross-read against the v2.1.248 release note
  ([github.com/anthropics/claude-code/releases/tag/v2.1.248](https://github.com/anthropics/claude-code/releases/tag/v2.1.248)).
- **Failure-mode corpus** (read 2026-09-07):
  [CVE-2025-59829 / GHSA-66m2-gx93-v996](https://github.com/anthropics/claude-code/security/advisories/GHSA-66m2-gx93-v996)
  — CWE-61, deny rules did not account for symlinks, so a denied file was reachable through a link
  to it (patched v1.0.120);
  [#61148](https://github.com/anthropics/claude-code/issues/61148) — `@../` attachment syntax
  reportedly reaches files outside the workspace root through a preprocessing path that does not
  run the permission layer;
  [#47821](https://github.com/anthropics/claude-code/issues/47821) — symlinks inside an allowed
  directory prompt anyway, because permissions are evaluated against the resolved target;
  [#22155](https://github.com/anthropics/claude-code/issues/22155) — open feature request to
  auto-restrict the sandbox to the working directory, i.e. the fence users expect is not the
  default. Plus two self-documented limits in the permission docs themselves: `Bash(command:rm *)`
  is *ignored with a startup warning* because a compound command would bypass it, and a relative
  redirect target that follows a `cd` in the same command "needs your approval" because Claude Code
  cannot determine what it resolves against.
- **Depends on**: nothing unlanded. `RiskLevel`, `SuggestedRouting`, `SafetyValidator`,
  `ValidationResult`, `SafetyLevel`, `clap`, `serde`, `schemars` and `assert_cmd` all ship in
  1.4.0. No filesystem I/O, no network, no daemon, no new crate, no new top-level module
- **Relates to**: ADR-064 (`caro.egress.v1` — owns the *source→sink* network-exfiltration axis and
  needs to recognise credential-bearing file paths for its `SourceKind`; D7 below makes this ADR's
  operand extractor the single primitive 064 consumes rather than minting a second path parser),
  ADR-065 (`caro.unattended.v1` — owns the *interruption/placement* axis; exit-code convention
  inherited from it, see D8), ADR-039 (sandbox-aware verdict tier — *may this run inside a
  sandbox*; this is *does what it does stay inside a boundary*), ADR-033 (`caro undo` — the
  recovery path for the writes this ADR tries to catch first), ADR-007 (shell AST — deliberately
  **not** depended on; see D6)
- **Does not depend on**: ADR-024 (headless envelope / exit-code enum), ADR-058/059
  (`caro.assessment.v1`), ADR-060 (`caro.eval.v1`), ADR-063 (`caro.bench.v1`), ADR-064
  (`caro.egress.v1`), ADR-065 (`caro.unattended.v1`). All are paper as of this date; `src/safety/`
  still contains only `mod.rs`, `patterns.rs` and `cve_patterns.rs`
- **Full scope document**: `caro-scope-path-confinement-2026-09-07.md`
- **Numbering note**: highest existing is ADR-065. Per `.claude/rules/adr-numbering.md`, renumber
  on merge if another 066 lands first

---

> **Provenance note (autonomous run).** The task template left `[FEATURE NAME]` unbound and ran
> with no user present, so target selection was mine. Rationale: `--restricted` is eleven days old
> at time of writing, is the first Claude Code feature to make *filesystem scope* a first-class
> product surface rather than a per-prompt permission, and its fence is enforced by tool removal —
> which means the fence has a documented, deliberate hole the moment a shell tool is named in
> `--tools`. That hole is exactly the shape of Caro's product. No existing Caro ADR covers path
> scope: ADR-064 owns network egress, ADR-065 owns interruption, and grep across `docs/adr/` finds
> no ADR that reasons about *where on the filesystem a command lands*.
>
> **Moratorium note.** ADR-059 declared itself "the last ADR in this space until
> `src/safety/assessment.rs` merges." Re-verified 2026-09-07: it has not merged (`src/assessment/`
> is still the *hardware* recommender — `cpu.rs`, `gpu.rs`, `memory.rs`, `recommender.rs` — and
> `Commands::Assess` is still a commented-out block at `src/main.rs:422-434`). This ADR mints no
> policy vocabulary, no approval exchange, no lifecycle event and no new routing tier: it reuses
> `RiskLevel` and `SuggestedRouting` unchanged. It mints one report struct and one operand struct,
> and D7 commits both to nesting inside `caro.assessment.v1` when that lands. Treat that as the
> reviewable seam.

---

## Context

### What the host actually fences

Claude Code's path model is a **tool-input** model. Every rule in the permission language names a
tool and constrains that tool's primary content field: `Read(./src/**)`, `Edit(//tmp/scratch.txt)`,
`Cd(~/projects/**)`. The rules use gitignore syntax with four anchors (`//` filesystem root,
`/` settings-source-relative, `~/` home, bare/`./` cwd-relative), they match symlinks by checking
both the link and its target, and `--restricted` layers a hard fence on top: file tools are pinned
to the working directory and settings files that could widen the fence are not loaded at all. As a
capability boundary for *file tools*, this is careful, well-specified work — noticeably more
careful than the ambient "the agent can read your home directory" default it replaces.

The model has one seam, and the docs name it themselves. For the Bash tool, the primary content
field is `command`, and **a rule may not constrain it**: the docs state that `Bash(command:rm *)`
"would be bypassable by a compound command, so Claude Code ignores it and emits a startup
warning." What Bash rules match instead is a *command-prefix string* — `Bash(npm run *)`,
`Bash(git * main)` — with a documented wrapper-stripping table (`timeout`, `nice`, `nohup`,
bare `xargs`, …) and a documented list of forms that defeat prefix matching entirely
(`find -exec`, `find -delete`, `watch`, `setsid`, `flock`). The docs are candid: "Bash permission
patterns that try to constrain command arguments are fragile."

The one place the host *does* look at a path inside a shell command is redirection: `> file`,
`>> file` and `2> file` are checked against `Edit` rules, protected paths and the working
directories, and `< file` gained the same treatment for `Read` rules in v2.1.257.

Everything else is unfenced. Not by oversight — by architecture. There is no vocabulary in the
permission language capable of expressing "the path operands of this command must stay under
`$CWD`", so:

```
cp .env /tmp/x                     # Write, outside. No rule can express the constraint.
cat ~/.ssh/id_rsa                  # Read, outside. Not a redirect; not checked.
tar -C / -xf pkg.tgz               # Write to /, expressed as a flag value.
git -C ~/other-project checkout .  # Destructive write, in another repo, via a flag.
rsync -a ./ ../backup/             # Write, one segment outside, via traversal.
```

`--restricted` closes this by removing the shell tool. That is a real answer, and for the
review-only sessions the flag targets it is the right one. But `--restricted` explicitly
re-admits removed tools named in `--tools`, and the overwhelming majority of Claude Code sessions
do not run restricted at all. In both of those populations the filesystem fence is *tool-shaped*,
not *effect-shaped*, and a shell command walks through it.

### Why the failure modes are structural, not bugs

The corpus above is four different symptoms of one root cause: **a lexical check being reported as
a resolved one, and a resolved check being reported as a lexical one, with no type distinguishing
them.**

- CVE-2025-59829 (CWE-61): deny rules matched the path *as written* and the symlink resolved
  elsewhere. Lexical answer, presented as authority over the filesystem.
- #47821: the inverse. Permissions evaluated against the *resolved* target, so a legitimate link
  inside an allowed directory prompts. Resolved answer, presented as authority over the workspace.
- #61148: a second ingestion pipeline (`@` attachments) reached paths the first pipeline's rules
  denied — one check, two doors.
- The `cd`-then-relative-redirect carve-out: the host correctly recognises it cannot resolve the
  operand, and its only expressible response is a prompt.

Caro cannot fix any of these in Claude Code. What it can do is refuse to reproduce them: build the
*same* analysis in a form where the lexical/resolved distinction is a required field of the
payload, where "I cannot tell" is a value rather than a prompt, and where there is exactly one
door because there is exactly one input — a command string.

### What exists in Caro today

Verified against `integrator/20260711-postmerge`, 2026-09-07:

| Capability | Present? |
|---|---|
| Extract file-path operands from a command | **No.** Zero code anywhere in `src/` |
| Glob expansion / `~` expansion / `..` normalisation | **No.** Zero hits |
| `canonicalize`, symlink resolution | **No.** Zero hits |
| A notion of project root / allowed directories | **No.** `ExecutionContext.cwd` is an environment snapshot rendered into prompts; `DirectoryContext::scan` detects project *type*, non-recursively, and never walks up |
| Path awareness of any kind | Only as regex literals inside `src/safety/patterns.rs` (`r">\s*/etc/"`, `r"(rm\|mv\|chmod\|chown)\s+.*(/bin\|/sbin\|/usr/bin\|…)"`) and as substring checks in `src/caroml/validators/side_effects.rs` (`has_systemwide_write` tests `cmd.contains(" /etc/")` and eight siblings). Neither extracts the path it matched |
| Shell tokenizer / AST | **No.** No `shell-words`, `shlex`, `yash-syntax`, `conch-parser`, `tree-sitter` in `Cargo.toml` or `src/`. `src/caroml/ast.rs` is the `.caro` task DSL, not shell |
| Reusable risk vocabulary | **Yes.** `RiskLevel{Safe,Moderate,High,Critical}` and `SuggestedRouting{AutoApprove,AsyncLog,HumanGate,Block}` with `from_risk_and_safety`, both in `src/models/mod.rs`; `ValidationResult{allowed,risk_level,explanation,warnings,matched_patterns,confidence_score}` in `src/safety/mod.rs`; `schemars` at `Cargo.toml:46` |

So this ADR adds a genuinely new primitive to a tree that has the vocabulary to express its
verdict but no way to compute it — and it is the third consecutive ADR (064, 065, 066) to need
composition analysis and decline to build it.

---

## Decision

Add one stateless verb, **`caro confine`**, that answers a question nothing in the pipeline
currently answers: *given a command and a set of roots it is supposed to stay inside, which of its
path operands leave?*

```
caro confine "tar -C / -xf pkg.tgz" --root /w --safety moderate -o json
```

One command string in, one serialized report out, one exit code. No filesystem is touched, no
process is spawned, nothing is remembered.

**D1 — The verdict is about the command text, and the payload says so.**
`ConfinementReport.resolution` is a required field whose only v1 value is `Lexical`. A `Contained`
verdict is defined as *"no operand in this command names a path outside the roots, as written"* —
never as *"this command cannot touch a path outside the roots."* This is the CVE-2025-59829 class
designed out rather than patched: the payload cannot be mistaken for a filesystem claim because it
declares its own resolution mode. `Resolution` is an enum, not a bool, so `Resolved` can be added
later (§Out of scope) without a schema break.

**D2 — `..` never yields `Inside`.** Lexical normalisation of `..` is unsound in the presence of
symlinks, which is precisely CWE-61. Therefore: an operand whose lexical normalisation lands
outside the roots is `Escaping`; an operand containing a `..` segment that lexically *stays*
inside is `Unverifiable { Traversal }` — never `Inside`. `/w/../w/README.md` is not contained.
This is deliberately conservative and will generate noise; D5 says what happens to that noise.

**D3 — `Unverifiable` is a value, and it is not `Contained`.** Home-anchored (`~/…`),
variable-expanded (`$X`, `${X}`, `$(…)`), glob-bearing and post-`cd` relative operands cannot be
placed without information Caro refuses to gather. Each gets a typed reason. A report containing
any `Unverifiable` operand has verdict `Indeterminate`, and `Indeterminate` routes by safety level
through the *existing* `SuggestedRouting::from_risk_and_safety` shape — it does not mint a tier.

**D4 — Access intent is per-operand.** `PathOperand.access` is `Read | Write | ReadWrite |
Unknown`, derived from a head-keyed operand table (`cat FILE`→read; `cp SRC DST`→read, write;
`tar -C DIR`→write; `> file`→write; `< file`→read). An escaping *write* and an escaping *read* are
different findings and the payload keeps them different. Claude Code's model can express this
distinction for file tools (`Read()` vs `Edit()`) and cannot express it for shell commands at all;
matching it is the point.

**D5 — Precedence is fixed and tested.** When one operand attracts several classifications:
`Escaping` (lexically leaves) > `Outside` (names an absolute path outside the roots) >
`Unverifiable` > `Inside`. When operands disagree, the report's verdict takes the worst. `cat
../*.env` is an `Escape`, not an `Indeterminate{Glob}` — traversal is decided before the glob is
noticed. A regression here is a fail-open and the test asserting it says so in its name.

**D6 — No shell AST in v1.** Any command containing `|`, `&&`, `||`, `;`, `&`, a newline, command
substitution or an `sh -c` wrapper returns verdict `Indeterminate { Composed }` with an empty
operand list. This matches ADR-064 D4 and ADR-065 D6 verbatim and for the same reason: ADR-007 is
Proposed and unbuilt, and pretending to analyse `cd /w/src && cat ../../etc/passwd` with a
head-keyed table is how you ship a fail-open. Three consecutive ADRs now carry this carve-out;
§Consequences records that as the argument for landing ADR-007 next, not as a cost of this one.

**D7 — The operand extractor is shared infrastructure, not a private helper.**
`src/safety/confinement::operands(command, shell) -> Result<Vec<PathOperand>, Composed>` is public
and is *the* path-extraction primitive for the repository. ADR-064's `SourceKind` must recognise
credential-bearing file paths (`$HOME/.ssh/id_rsa`) to populate an `EgressPair`; it consumes this
function rather than writing a second parser. If 064 lands first, it lands the function and this
ADR consumes it. Two path parsers in one safety module is the failure this clause exists to
prevent.

**D8 — Exit codes follow ADR-065, and the conflict is named rather than inherited silently.**

| Code | Meaning |
|---|---|
| `0` | Report produced; verdict `Contained`, or `Indeterminate` at a safety level that does not refuse it |
| `2` | Report produced; verdict `Escape` at any safety level, or `Indeterminate` under `SafetyLevel::Strict` |
| `1` | Caro failed or was misused (bad root, unsupported shell, empty input). **No verdict was reached; must never be read as approval.** stdout is empty |
| `201` | Untouched (`EXIT_CODE_EDIT`) |

`2` is chosen because this verb's first consumer is a `PreToolUse`-shaped hook, where exit 2 is the
host's own "block" convention, so no translation layer is needed — ADR-065's reasoning, applied to
the same class of consumer. This conflicts with ADR-064, which chose `3` for its blocked state to
pre-align with ADR-024's unbuilt `ExitCode::Blocked`. **Two ADRs, two conventions, is the actual
defect.** This ADR does not resolve it unilaterally; it files it: whichever of 064/065/066 merges
first mints `pub enum CaroExitCode` in `src/main.rs` with `Ok = 0`, `Error = 1`, `Blocked = 2`,
`Edit = 201`, and the others adopt it in their implementation PR. This ADR votes `2` and will
adopt whatever the enum says.

**D9 — Roots are absolute or the run fails.** `--root` accepts only an absolute, already-normalised
path. A relative root, a `~`-prefixed root, or a root containing `..` exits 1 with a message. This
designs out the host's own documented footgun — that in its rule language `/Users/alice/file` is
*not* an absolute path but an anchor-relative one, requiring `//Users/alice/file`. Caro has exactly
one anchor and it is the filesystem root. Omitting `--root` entirely means "advise only": operands
are extracted and classified for access intent, every placement is `Unverifiable { NoRoot }`, and
the exit code is 0.

**D10 — Footprint.** New file `src/safety/confinement.rs` inside the existing `src/safety/` module;
new types in `src/models/mod.rs` beside `RiskLevel` and `SuggestedRouting`; one new `Commands`
variant and one dispatch arm in `src/main.rs`; new `tests/confinement_contract.rs`. No new
top-level module, no new crate, no new feature flag, no new config key. `ConfinementReport` is
surfaced as one additive `Option<…>` field on `CliResult` with `#[serde(default,
skip_serializing_if = "Option::is_none")]`, following ADR-064 §3 — no envelope, so ADR-024 is not
pre-empted.

---

## Consequences

**What gets better.** A Claude Code hook, a CI step, or a `just` recipe can ask a single
subprocess whether a shell command's writes leave the repository, and get a deterministic,
machine-readable answer with per-operand access intent — a question that today has no expressible
form in the host's permission language and no implementation in Caro. Caro's differentiator is
that this works with no session, no daemon, no network and no vendor: the same binary answers for
Claude Code, for a Makefile, for a shell hook, and offline.

**What gets worse.** Conservatism is expensive. D2 alone means every `../` in a legitimate
monorepo command is `Unverifiable`, and D6 means every pipeline is `Indeterminate`. The measured
`unverifiable_rate` on the existing evaluation corpus is the number that decides whether this verb
is useful or is theatre, and the scope document makes it a reported metric from day one. The
predictable bad fix — a maintainer collapsing `Unverifiable` into `Inside` to cut the noise — is
a silent fail-open of exactly the class this ADR exists to catch, which is why it is guarded by a
named property test rather than by a comment.

**What this makes harder.** A third ADR now carries the "no AST, `Indeterminate` on composed"
carve-out. Composition is no longer a corner case in Caro's roadmap; it is the dominant unhandled
class across three separate axes. This ADR treats that as the argument for prioritising ADR-007,
and deliberately structures `operands()` so that its `Composed` early-return is the single call
site an AST would replace.

**Reversibility.** High. The verb is additive and advisory; the report is an optional field; a
consumer that ignores the exit code is exactly as safe as it is today. Deleting the verb removes
one file, one enum variant, one dispatch arm and one test file.

---

## Alternatives considered

**A1 — Resolve paths for real (`canonicalize`, symlink following).** Rejected for v1. It makes the
verb non-deterministic, host-dependent and unusable in CI against a checkout that differs from the
author's machine; it puts host paths in the payload, breaking the byte-identical-output property
the sibling ADRs assert; and it turns a pure function into one that touches the filesystem, which
contradicts the standing subprocess constraint. D1's `Resolution` enum is the seam that lets a
future `--resolve` add this without a schema break — and when it does, it must report
`Resolution::Resolved` so consumers can tell the two claims apart. That distinction is the whole
lesson of CVE-2025-59829.

**A2 — Extend `patterns.rs` with regexes for out-of-root paths.** Rejected. Patterns match and
discard; they cannot say *which* path matched, cannot carry access intent, and cannot be
parameterised by a caller-supplied root. `has_systemwide_write` in
`src/caroml/validators/side_effects.rs` is exactly this design and it is warn-only, hardcoded to
nine literals, and unable to answer the question. Reusing that shape would be duplicating the
limitation, not the infrastructure.

**A3 — Ingest Claude Code's `permissions` block and evaluate its gitignore rules.** Rejected as
scope, deferred as integration. Reimplementing gitignore semantics, four anchors, symlink pairing
and settings precedence is a large surface with a compatibility obligation to a moving target, and
it would make Caro's answer only as good as the host's. `--root` is vendor-neutral and a caller
that wants host semantics can populate it from `additionalDirectories` itself. Ingestion of host
settings is ADR-064's `SandboxProfile`, and that is where it should stay.

**A4 — Make this a CaroML validator angle instead of a verb.** Rejected for v1. The CaroML
`Validator` trait's `ValidatorContext` has no root set and no place to put one, `ValidationOutcome`
and `Verdict` derive only `Debug, Clone, PartialEq` — they are not `Serialize` — and the chain's
three-valued `Pass/Warn/Fail` cannot carry `Unverifiable` without overloading `Warn`. Registering
an angle is a clean follow-up once `ValidatorContext` grows a root; it is not the v1 delivery.

**A5 — Do nothing; `--restricted` is the answer.** Rejected. `--restricted` answers the question
by deleting the tool, which is correct for review sessions and unavailable to every session that
needs a shell — including sessions that re-admit one through `--tools`. It also does not travel:
it is one vendor's flag in one client. The unmet need is a portable answer for the case where the
command runs.

---

## References

- Claude Code permissions reference — rule syntax, anchors, symlink pairing, redirect checks,
  `blockReadsOutsideWorkingDirectories`, the `Bash(command:…)` ignore rule:
  <https://code.claude.com/docs/en/permissions> (read 2026-09-07)
- Claude Code "What's new" and v2.1.248 release note — `--restricted`:
  <https://code.claude.com/docs/en/whats-new>,
  <https://github.com/anthropics/claude-code/releases/tag/v2.1.248> (read 2026-09-07)
- CVE-2025-59829 / GHSA-66m2-gx93-v996, CWE-61 symlink deny bypass:
  <https://github.com/anthropics/claude-code/security/advisories/GHSA-66m2-gx93-v996>
- anthropics/claude-code issues [#61148](https://github.com/anthropics/claude-code/issues/61148),
  [#47821](https://github.com/anthropics/claude-code/issues/47821),
  [#22155](https://github.com/anthropics/claude-code/issues/22155)
- Caro tree, `integrator/20260711-postmerge`: `src/models/mod.rs` (`RiskLevel`,
  `SuggestedRouting`, `SafetyLevel`), `src/safety/mod.rs` (`SafetyValidator`, `ValidationResult`,
  `DangerPattern`), `src/caroml/validators/side_effects.rs` (`has_systemwide_write`),
  `src/cli/mod.rs` (`CliResult`, `OutputFormat`), `src/main.rs` (`Commands`, `EXIT_CODE_EDIT`),
  `tests/caroml_e2e.rs` and `tests/e2e_cli_tests.rs` (test idioms)
- ADR-007 (shell AST, Proposed), ADR-064 (`caro.egress.v1`), ADR-065 (`caro.unattended.v1`),
  `.claude/rules/adr-numbering.md`, `.claude/rules/validation-discipline.md`
