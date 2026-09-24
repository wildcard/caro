# Implementation Scope — Offline Package-Install Advisory Gate

**Feature under analysis:** **Prismor** (formerly *Immunity Agent*, PrismorSec)
— `prismor supplychain`, the package-manager wrapper that scores every
`npm install` / `pip install` / `cargo add` against live threat intelligence
and blocks compromised packages before the install script runs. Apache-2.0,
Python, installs as a `PreToolUse` hook into Claude Code, Cursor, Windsurf,
OpenClaw, Hermes, Grok Build, Kiro CLI "and 55+ other agents". Docs read live
**2026-08-19** at <https://www.prismor.dev/docs/prismor> — Quick Start, Supply
Chain Enforcement (risk-scoring table, IOC section, `harden`), Detection Rules
(17 built-ins), Network Isolation, Hooks & Enforcement, Enterprise enrollment.

**Equivalent we are scoping for Caro:** the same class of decision — *is this
package installation known-bad?* — with the intelligence turned inside out:
from a **live-queried, wrapper-gated, heuristically-scored** verdict into a
**build-time compiled, command-intercepted, evidence-bound** one. Caro stays a
pure function of `(command, embedded advisory snapshot)` and never becomes the
package manager.

**Date:** 2026-08-19 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-054-offline-package-advisory-gate.md`

> **Provenance note (autonomous run).** Produced with no user present; the task
> template's `[FEATURE NAME]` was unbound, so target selection was mine.
> Of the five opportunities in `market-scans/2026-08-17-ai-agent-strategy-memo.md`:
> A → ADR-052 (2026-08-17), D → ADR-053 (2026-08-18), C → already covered by
> ADR-037/040/041, E is a build spike that produces no ADR, and **B remains the
> memo's single best next thing to build and is *not* superseded by this
> document** — it is an eval run plus a docs page and should still be run.
> This run therefore went outside the memo to the live market and picked the
> one command class Caro's 67 built-in patterns are structurally blind to.
>
> **Verified vs. not.** Everything attributed to Prismor below comes from the
> vendor documentation page cited above, read in full. The GitHub README
> (`PrismorSec/prismor`) could **not** be fetched — the raw URL was outside the
> tool's provenance set. Line-level claims about `supplychain/ioc.py` are
> therefore quoted from the docs' own code sample, not from reading the source.
> All vendor performance and coverage claims are self-reported and unaudited.
>
> **Gate warning.** This introduces a new user-facing capability class, so
> `.claude/rules/validation-discipline.md` attaches: 20 transcripts, a
> demoware-trap section, and a devil's-advocate review before an implementation
> PR opens. Decision **D7** in the ADR is built so the evidence gate and the
> engineering gate are the same gate. Per `.claude/rules/git-workflow.md` both
> files are left **uncommitted** on `integrator/20260711-postmerge` for a human
> to branch and PR.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom

For anyone whose coding agent installs dependencies. The agent reads a README,
decides it needs `@scope/pkg`, and runs `npm install @scope/pkg`. Every
command-level guardrail in the market — Caro's included — sees a well-formed,
non-destructive, entirely ordinary command and allows it. The damage is not in
the command; it is in the `postinstall` script of a package whose maintainer
account was compromised nine days ago.

Prismor's framing is that the package manager is the enforcement point:
`prismor supplychain npm install express` wraps the manager, scores each
package before the install runs, and blocks with a reason. The docs' own
worked example blocks `@tanstack/react-router` at score 100 with the note
"42 packages compromised May 11 2026 via CI/CD cache poisoning. SLSA
attestations do NOT protect against this."

### Core architecture — data flow, key types, separation of concerns

- **Interception:** a `PreToolUse` hook per agent, written into each harness's
  own config file (`~/.claude/settings.json`, `.grok/hooks/prismor.json`,
  `.openclaw/plugins.json`, `.hermes/plugins.json`, `.kiro/agents/kiro_default.json`).
  Two enforcement modes, `observe` (log only, the default) and `enforce`.
- **Policy:** 17 YAML detection rules, overridable per project in
  `.prismor/policy.yaml` (`rules[]` with `id`/`severity`/`category`/`patterns`/`action`,
  plus `allowlists[]` and a `settings.egress_allowlist`). Committing the file
  shares rules across a team.
- **Supply-chain path (separate from the 17 rules):** `prismor supplychain <pm> <args>`
  parses the install, checks each package against an IOC database and a
  heuristic scorer, and prints `BLOCK` / `WARN` / `ALLOW` per package with the
  contributing signals itemized. Non-install commands pass through untouched,
  which is what makes `alias npm="prismor supplychain npm"` viable.
- **Scoring:** additive points with two thresholds — `<30` ALLOW, `30–59` WARN,
  `≥60` BLOCK. Signals range from `+100 (force block)` for an IOC match or a C2
  domain in an install script, through `+50` credential-env access and `+35`
  git-dependency, down to `+25` "published < 7 days ago", `+15` "< 30 days ago",
  `+10` single maintainer, `+8` "maintainer data unavailable". **An IOC match
  skips the threshold entirely and forces a block regardless of total** — the
  one place evidence is allowed to dominate priors.
- **IOC data:** hand-edited Python at `supplychain/ioc.py` —
  `_COMPROMISED_VERSIONS`, `_COMPROMISED_NAMESPACES`, `C2_DOMAINS`, and a
  `_SCRIPT_PATTERNS` list of `(re.compile(...), message, "CRITICAL")` tuples.
- **Second, complementary layer:** `prismor supplychain harden` writes
  `ignore-scripts=true`, `save-exact=true` into `.npmrc`, `enableScripts: false`
  into `.yarnrc.yml`, and similar into `pip.conf` / `.cargo/config.toml`.
  Existing keys are never overwritten. The docs are explicit that hardening and
  runtime scoring are "complementary, not redundant."
- **Ecosystems:** npm, pnpm, yarn, bun, pip, uv, poetry, cargo, go — nine
  managers, with the specific subcommands enumerated per manager.
- **Enterprise:** `prismor enroll <token>` exchanges for a device key at
  `~/.prismor/identity.json` and pulls **Ed25519-signed, fail-closed** org
  policy — "tampered or unverifiable policy is ignored, and core protections
  can never be weakened remotely."

### Why it is limited — the failure modes

| # | Failure mode | Evidence |
|---|---|---|
| **F1** | **The supply-chain gate is wrapper-gated, and the vendor says so.** "Runtime scoring only fires when an install goes through `prismor supplychain`. A CI step or agent calling `npm install` directly bypasses it." The remedy offered is `harden` — writing flags into `.npmrc` — which is a *different* control mitigating a gap in the first one. An agent that shells out to `npm install` (the overwhelmingly common case, since agents write the command, not the alias) is outside the gate. |
| **F2** | **Live threat intelligence sits in the decision path.** "It checks each package against live threat intelligence." That is a network round trip per install, on the critical path, in a tool whose value proposition is blocking before execution. Offline, air-gapped, and flaky-network behaviour is unspecified; so is what happens on lookup failure. It also makes the verdict unreproducible: the same command, same machine, two different days, two different answers, with no record of which intelligence version decided. |
| **F3** | **The IOC database is hand-edited program source with no schema and no tests.** The documented way to add an indicator is to open `supplychain/ioc.py` and append a `re.compile` tuple. There is no validation that the regex compiles against a known-positive, no known-negative to catch over-matching, and every indicator update requires shipping a new release of the Python package. |
| **F4** | **Priors are mixed into the verdict, and one of them is wall-clock.** "Published < 7 days ago" is `+25` and "< 30 days ago" is `+15`. Two of those, plus a `+20` postinstall script, is `60` — a **block** on a legitimate new package with an ordinary build step. Worse for a safety tool that wants to be trusted: the score is a function of *today's date*, so the same `npm install foo@1.0.0` blocks this week and allows next month with no change in evidence. |
| **F5** | **Absence of evidence resolves to ALLOW.** `score < 30 → ALLOW`, and "maintainer data unavailable" contributes only `+8`. A package the system knows nothing about, whose registry metadata could not be fetched, lands in the same bucket as `express` at score 0. The docs' own example prints `ALLOW score 0 express` — there is no third outcome for *unknown*. |
| **F6** | **The scored object is the command line, not the resolution.** Version ranges, lockfiles, and transitive dependencies are not in the documented scoring inputs. `npm install express` — unpinned — is scored as the package name, so whatever version the registry resolves to at install time was never the thing that was assessed. |

Credit where due, and we should match it rather than merely note it: the
IOC-beats-threshold rule (evidence dominates priors) is correct and we adopt
it; org policy being Ed25519-signed **and** unable to weaken core protections
remotely is a stronger fail-closed posture than most commercial policy
distribution; `harden` never overwriting existing keys is the right default;
and the per-package itemized output — every contributing signal printed with
its point value — is better denial ergonomics than most tools ship.

### Structured output contract

Human-readable per package (`BLOCK score 100 @tanstack/react-router age 1d, 3
maintainers` followed by the itemized signals). Machine-readable surfaces exist
but are adjacent to the supply-chain path rather than part of it: `prismor
scan --json`, `prismor audit --json`, and `prismor analyze --input session.jsonl
--sarif` for GitHub Code Scanning. **Exit codes are documented for `prismor
audit` only** — `2` critical, `1` high/medium, `0` clean. No exit-code or JSON
contract is documented for `prismor supplychain` itself; the override path is
prose ("add to `supply_chain.allowlist` in `.prismor/policy.yaml`"). A caller
wanting to branch on a supply-chain verdict is parsing a rendered table.

### Session / context lifecycle — how redundant initialization is avoided

It is not, in the subprocess sense. State lives in "local workspace databases"
read by `prismor status`, `prismor sessions`, `prismor dashboard`, and a
`prismor serve` web UI that **polls every 30 seconds**. `prismor scope` holds
session-scoped policy rules and `prismor learn` mines recorded session history
for candidate rules. Each hook invocation is a fresh Python process that
re-reads `.prismor/policy.yaml` and re-queries intelligence; the amortization
story is a database and a long-lived dashboard, neither of which survives the
translation to a stateless Rust subprocess. **The part that does translate is
the inverse:** compile the intelligence at build time so per-invocation
initialization is a `Lazy` deserialize of an embedded blob, not an I/O or
network event at all.

---

## Phase 2 — Competitive Differentiation

### What they get right that we should replicate

1. **Evidence beats priors, terminally.** An IOC match skipping the score
   threshold is the single best design decision in their scorer. Caro's
   equivalent: an advisory match is *terminal* and never averaged against
   anything.
2. **Per-signal itemization in the denial.** Every contributing factor printed
   with its weight. Caro already committed to this shape in ADR-052's
   `violations[]`; the supply-chain path must land inside it, not beside it.
3. **The wrapper is opt-in and pass-through.** Non-install commands are
   untouched. Caro's analog is that a command that is not an install must cost
   nothing — parse, miss, return, no advisory set touched.
4. **Hardening as a separate, complementary layer.** They are right that
   `ignore-scripts` is a different control from runtime scoring. We should say
   the same thing and then *not build it* (see Out of Scope).

### Their gaps we avoid by designing the schema first

| Their gap | Our schema decision |
|---|---|
| F1 wrapper-gated | Assess the **command string** at the existing validator boundary. No alias, no wrapper, no re-exec. `npm install` typed directly is the same input as `npm install` behind an alias. |
| F2 network in path | The advisory set is **compiled into the binary at build time** by the existing dogma pipeline. Zero network, zero file I/O, zero variance. |
| F3 unschema'd hand-edited source | Advisories are YAML with a schema check **and mandatory `test_cases`** — the exact contract `data/cve_rules/*.yaml` already enforces, including known-negatives. |
| F4 wall-clock priors | Package age, maintainer count, and registry metadata are **not inputs**. The verdict is a function of `(command, snapshot)` only. Nothing in the decision reads the clock. |
| F5 unknown ⇒ allow | `MatchResolution` makes *unknown* and *unpinned* first-class outcomes distinct from *clean*. An unpinned install of a package that carries an advisory escalates rather than allows. |
| F6 command ≠ resolution | We assess exactly what the command *pins*, and say so: an unpinned range is reported as unpinned, never silently treated as the latest safe version. |
| No verdict exit code / JSON | Reuse ADR-024's `ExitCode` verbatim and ADR-052's `violations[]`. Zero new codes, zero new payload shapes. |

### Our unique positioning

- **Offline and deterministic.** Same command + same binary = same verdict,
  forever, on a disconnected machine. Nobody in this category can say that,
  and it is exactly the ground a hosted classifier or a live-intelligence
  lookup cannot contest.
- **Universal, at the right boundary.** Caro is already the thing the agent's
  shell command passes through. Prismor must be installed *per harness* into
  each of 55+ config files and then additionally aliased per package manager;
  Caro needs neither.
- **One binary, no runtime.** No Python, no venv, no `pip install` to install
  the thing that guards `pip install`.
- **Community layer already exists.** `data/cve_rules/` with test-case-carrying
  YAML is a contribution path a security researcher can use without touching
  Rust. Advisories reuse it verbatim.

### What Caro already has that covers part of this

| Existing | Covers |
|---|---|
| `src/dogma/compiler.rs` (`CompiledRuleset`, `RawRule`, `compile_with_tests`, `discover_rule_files`) | The whole build-time YAML → bincode pipeline. Advisories are a second ruleset through the same compiler. |
| `build.rs::compile_cve_ruleset` + `cargo:rerun-if-changed=data/cve_rules` | The exact build wiring to copy, including the generated-tests emission for the eval harness. |
| `src/safety/cve_patterns.rs` (`Lazy` bincode load, empty-set fallback on deserialize failure) | The runtime loader shape, including "safety must never be a hard dependency for caro to run". |
| `data/cve_rules/*.yaml` + `EXAMPLE-TEMPLATE.yaml` + `tests/cve_enforcement.rs` | The data format, the contribution template, and the enforcement test file to mirror. |
| `ValidationResult` / `SafetyDecision` / `RiskLevel` / `SuggestedRouting` | The verdict types. No new verdict vocabulary is needed or wanted. |
| ADR-024 `ExitCode` | The exit contract. `Blocked = 3`, `NeedsConfirmation = 4`. |
| ADR-052 `violations[]`, `RuleId`, `RuleSource`, `Remediation` | The denial payload. Advisory findings are violations, not a parallel channel. |
| ADR-044 unresolved-as-escalation | The precedent for "cannot resolve ⇒ escalate", which is exactly how an unparseable install must behave. |
| ADR-047 trusted-targets registry | The allowlist. Prismor adds a bespoke `supply_chain.allowlist`; we must not. |
| `validate_user_pattern` (`Critical` reserved for built-ins, ReDoS length bound) | The hardening precedent for any user-supplied advisory overlay. |

---

## Phase 3 — Scope Definition

Full decision record: **`docs/adr/ADR-054-offline-package-advisory-gate.md`**.
Summary follows.

### New types

All serializable from day one. Build-time types extend `src/dogma/compiler.rs`
(so `build.rs` can construct them without the runtime crate); runtime types live
in `src/safety/`.

**In `src/dogma/compiler.rs`** — mirroring `CompiledPattern` / `RawRule`:

- `RawAdvisory` — the YAML shape in `data/advisories/*.yaml`:
  `id` (`GHSA-…` / `RUSTSEC-…` / `CVE-…`), `ecosystem`, `package`,
  `affected` (list of `{ introduced, fixed }`), `kind`, `risk_level`, `source`
  (advisory URL), `published`, `summary`, `test_cases` (**mandatory**, each an
  install command string plus `expected_behavior`).
- `CompiledAdvisory` — the same, minus `test_cases`, with the version bounds
  pre-parsed and the ecosystem lowered to its OSV canonical string.
- `CompiledAdvisorySet { advisories: Vec<CompiledAdvisory>, metadata: AdvisorySetMetadata }`.
- `AdvisorySetMetadata { advisory_count, snapshot_date, per_ecosystem_counts, attribution: Vec<String> }`
  — `snapshot_date` is the newest `published` in the set, not the build date, so
  it is reproducible across rebuilds.

**In `src/safety/install_intent.rs`** (new file inside an existing module —
justified below):

- `Ecosystem` — `Npm | PyPI | CratesIo | Go | RubyGems | Maven | NuGet | Packagist | Hex`,
  `#[serde(rename_all)]`'d to the OSV canonical spellings so advisory data and
  parser output cannot drift.
- `PackageRef { name: String, version_req: Option<String>, origin: PackageOrigin }`.
- `PackageOrigin { Registry | Git | Tarball | LocalPath }`.
- `InstallIntent { ecosystem, packages: Vec<PackageRef>, registry_override: Option<String>, from_manifest: bool }`
  — `from_manifest: true` is a bare `npm install` / `pip install -r`, i.e. the
  set of packages is *not* on the command line.
- `InstallParse { NotAnInstall | Parsed(InstallIntent) | Unresolved(UnresolvedReason) }`
  — the fail-closed tri-state.
- `UnresolvedReason { UnknownSubcommand | ShellExpansion | ManifestNotOnCommandLine | MalformedSpec }`.
- Method contract: `InstallParse::parse(command: &str, shell: ShellType) -> InstallParse`
  — pure, no I/O, no clock, `O(len(command))`.

**In `src/safety/advisories.rs`** (mirrors `cve_patterns.rs`):

- `ADVISORY_SET: Lazy<CompiledAdvisorySet>` — `include_bytes!` + bincode,
  `unwrap_or_default()` on failure, exactly as `CVE_COMPILED` does.
- `AdvisoryFinding { advisory_id, package: String, ecosystem: Ecosystem, resolution: MatchResolution, risk_level: RiskLevel, source: String, summary: String }`.
- `MatchResolution { PinnedInRange | PinnedOutOfRange | RangeOverlaps | Unpinned | Unknown }`
  — the type that makes F5 unrepresentable.
- `pub fn assess_install(intent: &InstallIntent) -> Vec<AdvisoryFinding>` — pure.

**Extension to ADR-052 (must land in the same schema version):**
`RuleSource` gains an `Advisory` variant, ordered `Builtin < Cve < Advisory <
User < Allowlist < Internal`; rule ids are `caro:advisory:<ADVISORY-ID>`.
If ADR-052 has already shipped when this lands, adding the variant is a
schema-version conversation, not a silent addition.

### Files that change

No new top-level module. Two new files inside `src/safety/`, which is the same
shape `patterns.rs` / `cve_patterns.rs` already established; putting a ~300-LOC
argv parser in `mod.rs` would be worse.

| # | File | Change |
|---|---|---|
| 1 | `data/advisories/*.yaml` + `EXAMPLE-TEMPLATE.yaml` | New data directory, mirroring `data/cve_rules/`. |
| 2 | `src/dogma/compiler.rs` | Add `RawAdvisory` / `CompiledAdvisory` / `CompiledAdvisorySet` / `AdvisorySetMetadata` and `compile_advisories_with_tests`. |
| 3 | `build.rs` | `compile_advisory_set()` → `$OUT_DIR/advisories.bin`; `cargo:rerun-if-changed=data/advisories`; `cargo:rustc-env=CARO_ADVISORY_COUNT` and `CARO_ADVISORY_SNAPSHOT`. |
| 4 | `src/safety/advisories.rs` | **New.** Runtime loader + `assess_install`. |
| 5 | `src/safety/install_intent.rs` | **New.** The parser. |
| 6 | `src/safety/mod.rs` | `pub mod` the two; call `InstallParse::parse` inside `validate_command`; fold findings into `ValidationResult`. |
| 7 | `Cargo.toml` | `advisories = []` feature, added to `default` alongside `cve-rules`. |
| 8 | `src/version.rs` | Report advisory count + snapshot date in `--version`, as CVE rules already are. |
| 9 | `tests/advisory_enforcement.rs` | **New.** Mirrors `tests/cve_enforcement.rs`. |
| 10 | `NOTICE` | **New file** — the repo has none today. CC-BY 4.0 attribution for GHSA/PyPI/Go-derived advisory records. |

**Boy-scout note, not fixed here:** `docs/adr/README.md` indexes nothing above
ADR-049 — ADR-050 through ADR-054 are all missing from the table. Per
`.claude/rules/adr-numbering.md` the index is supposed to track every ADR.
Worth a one-commit cleanup on whichever branch lands next; deliberately not
bundled into this scope.

### Exit code / output contract

**No new exit codes.** ADR-024's `ExitCode` is reused verbatim:

| Outcome | `ExitCode` | Payload |
|---|---|---|
| Not an install, or install with no advisory match | `Ok = 0` | `violations: []`, `advisory_snapshot` present |
| `PinnedInRange`, `kind: KnownMalicious` | `Blocked = 3` | violation, `action: block`, `retryable: false`, `remediation: NoReformulation` |
| `PinnedInRange`, `kind: Vulnerable` with a `fixed` bound | `NeedsConfirmation = 4` | violation, `remediation: UseAlternative`, `offending_tokens: [the pinned spec]`, `retryable: true` |
| `RangeOverlaps` or `Unpinned` against an advised package | `NeedsConfirmation = 4` | violation, `remediation: ScopeTarget` ("pin to a version outside the advised range") |
| `InstallParse::Unresolved` | `NeedsConfirmation = 4` | `caro:internal:install-unresolved`, `retryable: false` |
| Advisory blob unloadable | `Blocked = 3` | `caro:internal:validator-unavailable` per ADR-052 D4 |

Two contract additions, both additive per ADR-024:

- `advisory_snapshot: { snapshot_date, advisory_count }` — always serialized,
  never null, on every outcome including success. This is the anti-F2 field:
  a consumer can always tell which intelligence version produced the verdict.
- `violations[]` entries with `rule_id` prefix `caro:advisory:` carry
  `ecosystem`, `package`, and `resolution` in their existing evidence slot.

Human-readable output gains one line per finding and is otherwise unchanged.

### Integration tests (`tests/advisory_enforcement.rs`)

Every case is `(command, --output json) → (exact violations[], exit code)`,
asserted on full JSON equality, which ADR-052's total ordering makes valid.
The fixture advisory set is pinned in `tests/fixtures/` so the suite does not
move when `data/advisories/` grows.

| Input | Expected |
|---|---|
| `npm install express` | exit `0`, `violations: []` |
| `npm install @fixture/known-bad@1.2.3` (in range) | exit `3`, one `caro:advisory:` violation, `resolution: pinned_in_range`, `retryable: false` |
| `npm install @fixture/known-bad@2.0.0` (fixed) | exit `0`, `violations: []` |
| `npm install @fixture/known-bad` (unpinned) | exit `4`, `resolution: unpinned`, `remediation: scope_target` |
| `npm install @fixture/known-bad@^1.0.0` (range overlap) | exit `4`, `resolution: range_overlaps` |
| `pip install fixture-bad==0.9` | exit `3` — proves ecosystem routing |
| `cargo add fixture-bad@1.0.0` | exit `3` |
| `npm install` (bare, manifest-driven) | exit `4`, `caro:internal:install-unresolved`, reason `manifest_not_on_command_line` |
| `npm install $PKG` | exit `4`, reason `shell_expansion` |
| `npm run build` | exit `0` — non-install pass-through |
| `echo "npm install @fixture/known-bad@1.2.3"` | exit `0` — the string is not an install (guards the context-aware-matching invariant already in `patterns.rs`) |
| Same command, run twice, `TZ` and system clock changed between runs | byte-identical JSON — the anti-F4 assertion |
| Corrupted `advisories.bin` fixture | exit `3`, `caro:internal:validator-unavailable` |

Plus the build-time layer: every `data/advisories/*.yaml` `test_cases` block is
compiled into `$OUT_DIR/advisory_generated_tests.yaml` and run by the eval
harness, exactly as CVE rules are — so contributing an advisory without a
known-negative fails the build.

### Explicitly out of scope (next version)

- **Any network refresh.** No `caro advisory update`, no OSV API client, no
  background fetch. New advisories arrive via a Caro release, and that is the
  feature, not a limitation. *(A signed, offline-verifiable out-of-band feed is
  the obvious v2 and deserves its own ADR — it is where Prismor's Ed25519
  enrollment design becomes worth borrowing.)*
- **Lockfile and manifest scanning.** `package-lock.json`, `Cargo.lock`,
  `requirements.txt`. That is `caro scan` territory (ADR-023), not the
  pre-execution validator.
- **Transitive dependency resolution.** Requires a registry, i.e. the network.
- **Heuristic scoring.** Package age, maintainer count, postinstall presence,
  registry-override penalties. Deliberately excluded — F4 is the reason.
- **Config hardening.** Writing `ignore-scripts` into `.npmrc`. Caro does not
  mutate user config; it is a correct control and it belongs to the user's
  bootstrap, not to a validator.
- **A bespoke supply-chain allowlist.** ADR-047 trusted targets is the
  allowlist. If it cannot express "this package at this version", that is a
  bug to fix in ADR-047.
- **C2-domain and install-script content analysis.** Requires fetching and
  inspecting the tarball. Not a pure function of the command.
- **Rule mining from session history** (`prismor learn`'s analog) and any
  dashboard.

### Risks

1. **Curation is a standing commitment.** An advisory set that ships stale is
   worse than none, because it implies coverage. Mitigation: `snapshot_date` is
   always in the payload, surfaced in `--version`, and CI fails if the newest
   advisory is older than the release date by more than a stated window.
2. **False positives are exceptionally expensive here** — a blocked
   `npm install` stops work cold. Mitigation: only `KnownMalicious` blocks;
   `Vulnerable` escalates. The block set stays small and every entry cites a
   public advisory ID.
3. **Attribution.** GHSA / PyPI / Go advisory data is CC-BY 4.0 (Rust's is
   CC0 1.0). Shipping derived records inside an AGPL-3.0 binary is fine, but
   the attribution requirement is real — hence file #10, and hence the rule
   that every advisory YAML carries a `source` URL.
4. **Parser scope creep.** Nine ecosystems × many subcommands is where this
   becomes a project. Mitigation: ship npm / PyPI / crates.io only; the
   `Ecosystem` enum carries the rest as parse-time `NotAnInstall` until their
   parsers land, and `Unresolved` covers anything ambiguous.

---

## Sources

- [Prismor (formerly Immunity Agent) — official docs](https://www.prismor.dev/docs/prismor) — read in full 2026-08-19; all Prismor claims above are from this page.
- [PrismorSec/prismor on GitHub](https://github.com/PrismorSec/prismor) — repository referenced; README fetch was blocked by tool provenance and was **not** read.
- [OSV schema — ecosystems, `affected[].ranges`, and per-database licensing](https://ossf.github.io/osv-schema/)
- [OSV.dev data-quality guidance](https://google.github.io/osv.dev/data_quality.html)
- Internal: `market-scans/2026-08-17-ai-agent-strategy-memo.md`, `docs/adr/ADR-024`, `ADR-044`, `ADR-047`, `ADR-052`, `ADR-053`, `.claude/rules/validation-discipline.md`, `.claude/rules/git-workflow.md`.
