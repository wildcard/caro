# ADR-054: Offline Package-Install Advisory Gate — Evidence-Bound Supply-Chain Verdicts Without a Package-Manager Wrapper

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-19
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: **Prismor** (formerly *Immunity Agent*, PrismorSec;
  Apache-2.0, Python) — specifically `prismor supplychain`, the package-manager
  wrapper that scores each package in an install command against live threat
  intelligence and blocks compromised ones before the install script runs.
  Read live 2026-08-19 against <https://www.prismor.dev/docs/prismor>:
  Quick Start, Supply Chain Enforcement (risk-scoring table, `supplychain/ioc.py`
  sample, `harden`), Detection Rules (17 built-ins), Hooks & Enforcement,
  Network Isolation, Enterprise enrollment. The GitHub README could not be
  fetched (outside tool provenance) and was **not** read; source-level claims
  are quoted from the docs' own code sample.
- **Depends on**: ADR-024 (headless JSON envelope, `ExitCode` registry,
  additive-field rule), ADR-052 (`violations[]`, `RuleId`, `RuleSource`,
  `Remediation`, fail-closed D4, total ordering D6)
- **Relates to**: ADR-044 (unresolved-as-escalation — the precedent this ADR
  applies to unparseable installs), ADR-047 (trusted-targets registry — the
  allowlist this ADR deliberately does **not** duplicate), ADR-023
  (`caro scan` — where lockfile/manifest scanning belongs, explicitly out of
  scope here), ADR-036 / ADR-043 / ADR-048 (guard and proxy adapters, which
  inherit advisory findings for free through `SafetyDecision`), ADR-007
  (AST parser — a future upgrade path for the argv parser, not a dependency)
- **Companion scope document**: `caro-scope-supply-chain-advisories-2026-08-19.md`
- **Numbering note**: highest existing is ADR-053; per
  `.claude/rules/adr-numbering.md`, renumber on merge if another 054 lands first.
- **Exit-code note**: this ADR claims **no new exit code**. It reuses ADR-024's
  `Blocked = 3` and `NeedsConfirmation = 4` unchanged. The double-claim on code
  `13` (ADR-049 vs ADR-051) found while scoping ADR-053 remains open and is not
  touched here.

> **Provenance note (autonomous run).** Produced with no user present; the task
> template's `[FEATURE NAME]` was unbound, so target selection was the agent's.
> Opportunities A and D from `market-scans/2026-08-17-ai-agent-strategy-memo.md`
> became ADR-052 and ADR-053 on the two previous runs; C is covered by
> ADR-037/040/041; E is a build spike. **B — the deterministic-vs-classifier
> eval and `SAFETY_PHILOSOPHY.md` page — is still the memo's top
> recommendation and is not superseded by this document.** This run went to the
> live market instead and picked the one command class Caro's 67 built-in
> patterns are structurally blind to. Per `.claude/rules/git-workflow.md` this
> file is left uncommitted for a human to branch and PR.

---

## 1. Context

Caro validates the *shape* of a command. All 67 built-in patterns and every
CVE rule in `data/cve_rules/` ask the same question: does this command string
match something dangerous? For `rm -rf /`, `curl … | sudo bash`, and
`dd if=/dev/zero of=/dev/sda`, that question is the whole story.

For `npm install @scope/pkg@1.2.3` it is not. The command is well-formed,
non-destructive, and ordinary. The danger is entirely in the *identity of the
package*, and it is invisible to a regex over argv. Grepping `src/safety/patterns.rs`
for package management finds three patterns — forced removal, `pip install
--break-system-packages`, and `pip install --user` — all of which are about
*how* the install is performed, none about *what* is being installed. An agent
that reads a compromised README and runs the install it suggests passes every
gate Caro has.

This is not hypothetical volume. Agents install dependencies as a routine part
of the work, and registry compromises now arrive as namespace-wide events —
Prismor's own worked example is 42 `@tanstack/*` packages compromised in a
single May 2026 CI/CD cache-poisoning incident.

Prismor is the clearest shipped answer to this in the agent-safety category.
Its architecture is also where the interesting design constraints show up,
because four of its choices are ones Caro cannot make and one is a gap Caro is
uniquely placed to close:

1. **The gate is wrapper-gated, and the vendor documents the hole.** "Runtime
   scoring only fires when an install goes through `prismor supplychain`. A CI
   step or agent calling `npm install` directly bypasses it." The offered
   remedy, `prismor supplychain harden`, is a *different* control (writing
   `ignore-scripts` into `.npmrc`) compensating for a gap in the first one.
   Agents write commands, not aliases, so the bypass is the common case.
2. **Live threat intelligence sits on the decision path** — a network round
   trip per install, in a tool whose value is blocking before execution.
   Offline behaviour is unspecified, and the same command on two days can
   return two verdicts with no record of which intelligence version decided.
3. **The IOC database is hand-edited Python** (`supplychain/ioc.py`: append a
   `re.compile` tuple), with no schema, no known-positive, no known-negative,
   and a package release required per indicator.
4. **Priors are mixed into the verdict, and one is wall-clock.** "Published
   < 7 days ago" is `+25`; a postinstall script is `+20`; single maintainer is
   `+10`. That reaches the `≥60` block threshold on a legitimate new package
   with an ordinary build step — and the score changes as the calendar moves,
   with no change in evidence.
5. **Absence of evidence resolves to ALLOW.** `score < 30 → ALLOW`, and
   "maintainer data unavailable" contributes `+8`. There is no third outcome
   for *unknown*.

What Prismor gets right and this ADR adopts: an IOC match **skips the
threshold entirely and forces a block** — evidence dominating priors,
terminally. And every contributing signal is itemized in the denial output.

Caro already owns the infrastructure to do this differently. `src/dogma/compiler.rs`
compiles schema-checked, test-case-carrying YAML into a bincode blob at build
time; `build.rs` wires it; `src/safety/cve_patterns.rs` loads it with a `Lazy`
static and an empty-set fallback. That is a build-time intelligence pipeline
with a community contribution path, already shipping, already tested by
`tests/cve_enforcement.rs`. Advisories are a second ruleset through it.

## 2. Decision

Add an **offline, build-time-compiled package advisory set** and a **pure argv
install parser** to `src/safety/`, surfaced through ADR-052's existing
`violations[]` and ADR-024's existing exit codes. No new module, no new
dependency, no new exit code, no network.

Seven design commitments, each answering one Phase-1 failure mode by
construction rather than by mitigation.

### D1 — Assess the command, never wrap the manager

The input is the command string, at the boundary Caro already occupies:
`SafetyValidator::validate_command`. Caro does not alias, wrap, or re-exec
`npm`. `npm install X` typed by a human, emitted by an agent, or run in CI is
one input with one verdict. There is no configuration under which the gate is
installed but inactive, because there is nothing to install. *(Answers failure
mode 1 — and removes the need for the `harden` compensating control from the
threat model entirely.)*

### D2 — The advisory set is compiled into the binary; nothing is fetched

`data/advisories/*.yaml` → `build.rs` → `$OUT_DIR/advisories.bin` →
`ADVISORY_SET: Lazy<CompiledAdvisorySet>`, exactly as CVE rules work today.
Per-invocation initialization is a bincode deserialize of an embedded blob:
no file I/O, no socket, no clock. A verdict is a pure function of
`(command, binary)`, so "same input, same output, forever, offline" is a test
assertion rather than a claim. *(Answers failure mode 2, and is the answer to
"how does it avoid redundant initialization?" — by moving the work to build
time, not by keeping a process warm.)*

### D3 — Advisories are schema-checked data with mandatory test cases

`RawAdvisory` carries `test_cases` as a **required** field, each a concrete
install command plus `expected_behavior`, including known-negatives.
`compile_advisories_with_tests` emits `$OUT_DIR/advisory_generated_tests.yaml`
for the eval harness. An advisory whose regex-free version range fails to parse,
or that lacks a known-negative, **fails the build**. Contributing an advisory
requires no Rust. *(Answers failure mode 3.)*

### D4 — Evidence only; no priors, no clock

Package age, maintainer count, postinstall presence, download counts, and
registry metadata are **not inputs to the verdict**. There is no score and no
threshold. An advisory match is terminal; nothing else changes the outcome.
Nothing in the decision path reads the system clock, so the verdict cannot
drift with the calendar. If we later want advisory *signals* (e.g. "this
package is very new"), they land as warnings that never alter the verdict on
their own. *(Answers failure mode 4, and generalizes Prismor's one correct
scoring rule into the whole design.)*

### D5 — Unknown and unpinned are typed outcomes, not allows

```
MatchResolution = PinnedInRange | PinnedOutOfRange | RangeOverlaps | Unpinned | Unknown
```

`Unpinned` — an install of a package that carries an advisory, with no version
on the command line — **escalates to `NeedsConfirmation`**. It does not allow,
because the resolved version is decided at install time by the registry, not by
the string Caro assessed. `Unknown` (no advisory data for this package) is
serialized distinctly from a clean match, so a consumer can tell coverage from
clearance. *(Answers failure modes 5 and 6 together; F5 becomes unrepresentable
in the type.)*

### D6 — Unparseable installs escalate, per ADR-044

`InstallParse` is a tri-state: `NotAnInstall | Parsed | Unresolved`. A bare
`npm install` (packages live in the manifest, not on the command line), a
`$VAR`-expanded package name, or an unrecognized subcommand of a known manager
all return `Unresolved` and escalate to `NeedsConfirmation` with
`caro:internal:install-unresolved`. There is no fail-open path and no config
knob for one, matching ADR-052 D4 and ADR-044's unresolved-as-escalation
precedent. `NotAnInstall` is free: parse, miss, return, advisory set never
touched.

### D7 — Coverage is the evidence gate

`.claude/rules/validation-discipline.md` attaches to this as a new user-facing
capability class. Rather than run the gate as paperwork alongside the
engineering, the two are the same artifact: the 20 required transcripts are
collected from users running agents that install dependencies, and each one
must produce either (a) a real install command the parser must handle, or
(b) a stated reason the class is out of scope. The transcript corpus **is** the
parser's test corpus. The demoware-trap section answers one question: what
happens at 100 users when the advisory set is three months stale — and the
answer must be a field in the payload, not a README caveat. No implementation
PR opens before the five gates clear.

### Non-goals of this decision

No network refresh, no lockfile or manifest scanning, no transitive resolution,
no heuristic scoring, no config hardening, no bespoke supply-chain allowlist
(ADR-047 owns allowlisting), no install-script or tarball content analysis, no
new exit code, no new verdict vocabulary, and no change to when Caro blocks
anything that is not a package install.

## 3. Consequences

### Positive

- Closes a whole command class — dependency installation — that 67 built-in
  patterns and the CVE ruleset are structurally blind to, without changing what
  Caro decides about anything else.
- The differentiation is unusually clean and unusually defensible: offline,
  deterministic, reproducible, no wrapper, no per-harness install, no Python
  runtime guarding `pip install`. Every one of those is a property a
  live-intelligence competitor cannot adopt without abandoning its own design.
- Reuses the entire CVE pipeline — compiler, build wiring, loader shape, data
  format, contribution template, test file — so the net new machinery is one
  argv parser and one version-range matcher.
- `data/advisories/*.yaml` is a contribution path a security researcher can use
  with no Rust, extending the community layer `data/cve_rules/` established.
- Advisory findings reach the guard adapter (ADR-036), the MCP proxy (ADR-043),
  and the inference-hook adapter (ADR-048) for free, because they travel inside
  `SafetyDecision`.

### Negative / costs

- **Curation is a standing commitment.** A stale advisory set is worse than
  none because it implies coverage. Mitigated by `snapshot_date` being always
  present in the payload and in `--version`, and by CI failing when the newest
  advisory lags the release date beyond a stated window — but the underlying
  cost is ongoing human attention, not a one-time build.
- **False positives here are unusually expensive.** A blocked `npm install`
  stops work cold, and a developer who hits one bad block disables the tool.
  Mitigated by only `KnownMalicious` blocking while `Vulnerable` escalates, and
  by every advisory citing a public ID — but the block set must stay small on
  purpose, which caps the ceiling of what this feature can catch.
- **Binary size and build time grow with the advisory set**, and unlike CVE
  rules (a few hundred, slow-growing) advisories are numerous and fast-growing.
  The feature flag exists partly for this; a size budget is needed before the
  set gets large.
- **The argv parser is the real risk surface.** Nine ecosystems × many
  subcommands × shell quoting is where this becomes a project. Mitigated by
  shipping npm / PyPI / crates.io only and routing everything ambiguous to
  `Unresolved` — but that means honest coverage is narrow at launch, and
  `Unresolved` escalations are a friction source if the parser is too timid.
- **Attribution is a real obligation.** GHSA / PyPI / Go advisory data is
  CC-BY 4.0 (Rust's is CC0 1.0). Shipping derived records in an AGPL-3.0 binary
  is fine; omitting the `NOTICE` entry is not.
- **`RuleSource` gains a variant** (`Builtin < Cve < Advisory < User <
  Allowlist < Internal`). If ADR-052 has shipped by then, that is a
  schema-version conversation rather than a silent addition.

### Neutral

- Human-readable output gains one line per finding; nothing existing changes.
- `advisory_snapshot` is an additive field per ADR-024, so existing consumers
  are unaffected.

## 4. Alternatives considered

**A. Wrap the package managers, as Prismor does.** Rejected. It is the source
of the vendor's own documented bypass, it requires the user to modify their
shell rc, it makes Caro responsible for correctly forwarding arbitrary package
manager arguments, and it buys nothing Caro cannot get by reading the command
it is already being handed.

**B. Query OSV.dev at validation time.** Rejected. It puts a network round trip
on the pre-execution path, makes the verdict unreproducible and unavailable
offline, leaks the user's dependency list to a third party, and forfeits the
one property that makes Caro worth layering under an LLM classifier. It is also
the exact failure mode this ADR exists to design around.

**C. Ship a signed advisory feed updated out of band** (Prismor's Ed25519
enrollment model). Deferred, not rejected — this is the right v2 and deserves
its own ADR. It needs a signing key, a distribution channel, a revocation
story, and a fail-closed verification path, none of which should be entangled
with the first version of the parser and the data schema.

**D. Adopt a scored model with thresholds.** Rejected. Scores that mix evidence
with priors are how a supply-chain gate becomes a false-positive generator, and
wall-clock inputs make the verdict irreproducible — which is the property Caro
is selling. Signals may return later as non-verdict-changing warnings.

**E. Express advisories as regex patterns in the existing CVE pipeline.**
Rejected. `@scope/pkg` version ranges are not a regex problem; encoding
`>=1.0.0 <1.2.4` as a pattern is how you get both false positives and silent
misses. A version-range type is a few dozen lines and makes the test cases
meaningful.

**F. Do nothing; treat this as `caro scan`'s job (ADR-023).** Rejected as the
*only* answer. Scanning a lockfile in CI is valuable and is where manifest
analysis belongs — but it runs after the agent already installed the package.
The pre-execution gate and the CI scan are complementary, and only one of them
is in the path when the agent acts.

---

## References

- [Prismor documentation](https://www.prismor.dev/docs/prismor) — read 2026-08-19
- [PrismorSec/prismor](https://github.com/PrismorSec/prismor) — Apache-2.0; README not read (fetch blocked)
- [OSV schema](https://ossf.github.io/osv-schema/) — ecosystem names, `affected[].ranges`, per-database licensing
- [OSV.dev data quality](https://google.github.io/osv.dev/data_quality.html)
- `caro-scope-supply-chain-advisories-2026-08-19.md` — full scope, type definitions, file list, test table
- `market-scans/2026-08-17-ai-agent-strategy-memo.md` — opportunity context
- `.claude/rules/validation-discipline.md`, `.claude/rules/adr-numbering.md`, `.claude/rules/git-workflow.md`
