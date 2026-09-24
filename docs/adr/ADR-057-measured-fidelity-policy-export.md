# ADR-057: Measured-Fidelity Policy Export — Shipping Caro's Rules Into Other Engines Without Shipping a Lie

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-24
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run)
- **Feature researched**: **Agent Control Specification (ACS)**, the policy layer
  vendored into **Microsoft Agent Governance Toolkit**
  (`microsoft/agent-governance-toolkit`, **MIT**, 5.6k★). Read live 2026-08-24 against
  two primary sources in the repository: `docs/quickstart.md`
  (`last_reviewed: 2026-07-12`) and `docs/tutorials/55-agent-control-specification.md`
  (`last_reviewed: 2026-06-02`). Specifically scoped: the artifact pair
  `cmd_policy_gen` emits — a flat `manifest.yaml` binding a Rego bundle to named
  intervention points — and the `pre_tool_call` verdict contract a host enforces.
  ACS self-describes as **Public Preview**: *"ACS is vendored into AGT under
  `policy-engine/` as the AGT 5.0 policy layer. The APIs and manifest shape may change
  before GA."*
- **Cross-read**: **Execlave** `rego` and `custom_validator` policy types and **Phinq**'s
  tool+argument risk classification (both **second-hand** via Hermes memo 2026-08-20);
  **GuardFall** (Adversa AI, *The Hacker News*, 2026-06-30) which bypassed safety checks
  in 10 of 11 open-source coding agents by exploiting how Bash rewrites commands before
  execution — **second-hand**, not independently reproduced.
- **Implements**: Hermes memo 2026-08-20 **§3.9** — *"ship Caro patterns as content for
  other people's engines"* (Priority: Next, Complexity: M). The memo's standing
  instruction is quoted here because it is the whole thesis: **"be the content and the
  verdict; do not be the console."**
- **Blocked on** (must land first):
  - **Hermes memo §3.6 — `caro.assessment.v1`.** Still **unwritten**; no ADR exists.
    This ADR's `ExportDecision` down-map is a placeholder for a decision taxonomy Caro
    has not yet defined. Publishing an artifact that encodes a guess about Caro's own
    decision vocabulary is worse than publishing nothing. This is the **second** ADR to
    block on §3.6.
  - A **`regorus` direct-dependency build spike** per
    `.claude/rules/external-sdk-integration.md`. Largely pre-discharged —
    `regorus 0.10.0` already resolves in this exact workspace as an `agentmesh`
    transitive (`Cargo.lock:8247`) — but promoting a transitive to a direct dependency
    changes the license and MSRV surface Caro is accountable for, so all five checks get
    recorded.
- **Relates to**: **ADR-055** (coverage-completeness invariant — borrowed wholesale in
  D7), **ADR-053** (portable session chain ledger — whose exit-code-registry complaint
  this ADR repeats), **ADR-056** (observe mode — the *other* adoption on-ramp; that one
  lets you watch Caro decide, this one lets someone else's engine decide *using Caro's
  rules*), **ADR-052** (`Violation`, `RuleId`, fail-closed), **ADR-022** (`caro` safety
  library API), **ADR-023** (`caro scan` shell safety in CI), **ADR-024** (`ExitCode`
  enum — the registry owner this ADR wishes existed), **ADR-035/036** (external policy
  hooks and the agent-guard adapter — the *runtime* half of the same strategy).
- **Does not depend on**: **ADR-007** (AST parser shell validation, Proposed and
  unimplemented). This ADR's central decision (D4) is specifically the answer to *"what
  do we do when the consuming engine cannot parse a shell, and neither, today, can we?"*
  If ADR-007 later lands, it improves the fidelity numbers; it does not change the
  contract.
- **Numbering note**: highest existing is ADR-056. Per `.claude/rules/adr-numbering.md`,
  renumber on merge if another 057 lands first.
- **Exit-code note**: this ADR claims exit code **17**. The registry is contended and
  unowned — two ADRs claim 13, ADR-053 claims 14, ADR-055 claims an unnumbered "new,
  distinct" code (presumed 15), ADR-056 claims 16, and the tree still contains exactly
  one exit constant (`EXIT_CODE_EDIT = 201`, `src/main.rs:938`). **The registry needs an
  owner, and it should be ADR-024's `ExitCode` enum, not prose in six sibling ADRs.**

---

## Context

### The problem and who has it

Every agent-governance product shipping in 2026 has the same shape: an **engine** and a
**blank policy file**. AGT is the most credible instance — `pip install
agent-governance-toolkit[full]`, then `cmd_policy_gen --template strict` hands you
`manifest.yaml` and `policy.rego` and leaves the actual knowledge of *which commands are
dangerous* to you.

Caro has that knowledge. Sixty-seven TDD-validated dangerous-command patterns with a
documented zero-false-positive record, and no way whatsoever to hand them to anyone
using a different gate. Meanwhile the market has closed around Caro's previously
unclaimed wedges — the 08-20 scan found a free, MIT, zero-config, locally-installed
competitor (Phinq) and a commercial one (Execlave) inside eight days.

The strategic response the memo lands on is not to build a competing console. It is to
become the **content** inside the consoles that already exist. AGT's install base is
real, its policy format is documented, and its manifest explicitly binds *external* Rego
bundles. The integration surface is a file.

### Why this is not a two-day feature

Because the obvious version of it is actively harmful.

The obvious version serializes 67 regexes into Rego `regex.match` calls, writes a
manifest, and ships. That artifact would carry Caro's name and would not carry Caro's
safety property, for a reason visible in ACS's own documented policy input:

```json
{
  "intervention_point": "pre_tool_call",
  "policy_target": { "path": "$.tool_call.args", "kind": "tool_args",
                     "value": { "to": "customer@example.com", "body": "..." } },
  "tool": { "name": "send_email", "id": "send_email", "clearance": "internal" }
}
```

For a shell tool, `policy_target.value` is `{"command": "<a raw string>"}`. Rego's string
toolkit is `regex.match`, `contains`, `startswith`, `endswith`. Rego has **no tokenizer,
no quote removal, no word splitting, no parameter expansion, no `$IFS` handling, no
alias or builtin resolution** — every one of which is a rewriting step *bash performs
after* the policy has already returned its verdict. That is precisely the gap GuardFall
walked through in 10 of 11 agents; the sole survivor parsed the command the way bash
would, before deciding.

Compounding it, the ACS policy default is **fail-open at the rule layer**:

```rego
default verdict := {"decision": "allow"}
```

The *runtime* fails closed — "malformed manifests, missing paths, policy dispatcher
failures, and invalid transform targets produce `deny` verdicts with reserved
runtime-error reasons" — but a rule that simply does not fire allows. For a
dangerous-command denylist that default is the **correct** one (an allowlist would break
every legitimate command), which means every gap in an exported ruleset is **silent by
construction**.

Put those together: an unmeasured export is not merely incomplete. It is *confidently,
silently* incomplete, wearing Caro's name, installed by a user who reasonably believes
they now have Caro's protection.

### What ACS gets right, and we should not re-litigate

- **Decision runtime vs. enforcement point.** *"ACS is the policy decision runtime. Your
  application or adapter is the policy enforcement point."*
- **Statelessness as a stated property.** *"ACS is stateless. The host supplies all
  context for every evaluation."* And from the quickstart: *"The runtime itself remains
  free of session counters."*
- **Version pin at the top of the manifest** — `agent_control_specification_version: "0.3.1-beta"`.
- **Canonical policy input**, so rules never depend on host-local state.

### What Caro already has (verified against the tree at `1.4.0`, 2026-08-24)

| Asset | Location | Note |
|---|---|---|
| `DangerPattern` | `src/safety/mod.rs:334` | already `Serialize + Deserialize` |
| Built-in patterns | `src/safety/patterns.rs` | **67** literals |
| `RiskLevel` | `src/models/mod.rs:152` | `Safe\|Moderate\|High\|Critical`, lowercase wire, **`Ord`** |
| `ShellType` | `src/models/mod.rs:419` | 7 variants, same derives |
| `SafetyConfig` + layered TOML load | `src/safety/mod.rs:166`, `:697` | custom patterns already flow through |
| Pattern accessors | `src/safety/patterns.rs:512,532,540,568` | no new traversal code needed |
| `regorus 0.10.0` | `Cargo.lock:8247` | pure-Rust Rego, present transitively |
| `serde_yaml 0.9` | `Cargo.toml:43` | already direct — manifest needs no new crate |

Confirmed absent: no `Policy`/`Assess`/`Scan`/`Audit` variant on `Commands` in
`src/main.rs`; `src/assessment/` is **hardware** assessment (`cpu.rs`, `gpu.rs`,
`memory.rs`) and is not related; `docs/adr/` has **zero** mentions of Rego, OPA,
`PolicyDocument` or policy export.

**Four drifts found while verifying**, each of which an unguarded exporter would
propagate into a published artifact:

- **D-a** — `README.md:82,946` and `CLAUDE.md:32,108` claim **"52+"** patterns; the tree
  has **67**.
- **D-b** — `CLAUDE.md:108` describes risk levels as *"CRITICAL, HIGH, MEDIUM, LOW"*.
  The actual enum has **no `Medium` and no `Low`**. Hand-written Rego copying the docs
  would emit unmatchable strings.
- **D-c** — `CLAUDE.md` states MSRV 1.83; `Cargo.toml:5` says `1.85`.
- **D-d** — `src/governance/mod.rs` cites
  `.claude/plans/intgrate-https-github-com-microsoft-agen-witty-scroll.md`, which is
  **not in the tree**. The AGT plan referenced by the merged Phase-0 spike is missing;
  this ADR reconstructs Phase 1 from primary sources instead.

---

## Decision

Caro ships `caro policy export` and `caro policy verify`. The export emits a policy
artifact for a foreign engine **and, in the same artifact, a machine-readable
measurement of how much of Caro's safety property the translation lost.** The
measurement is not optional, not a flag, and not a separate tool.

Eight decisions.

### D1 — Export is a pure function, rendered by a single I/O boundary

`build(&SafetyConfig, PolicyTarget, &Corpus) -> Result<PolicyBundle, ExportError>` does
no I/O, reads no clock, reads no environment. `render(&PolicyBundle, &Path)` is the only
function in the module that touches the filesystem. `verify(&Path, &Corpus)`
re-evaluates a rendered bundle and re-derives the report without mutating anything.

Consequence: same Caro version + same config ⇒ **byte-identical output**. That is
asserted as a test, not claimed as a property. No daemon, no state, no network — a pure
subprocess call, which is also what makes it trivially usable from CI and from
`agt lint-policy` pipelines.

### D2 — The bundle carries three version identities, and unknown target versions are a hard error

`schema_version` (Caro's own bundle schema, `1`), `caro_version`
(`env!("CARGO_PKG_VERSION")`), and the target's version string —
`PolicyTarget::Acs { acs_version }`, for which **`"0.3.1-beta"` is the only accepted
value in v1**. `--acs-version 0.4.0` exits `2` with nothing written.

ACS says its shape may change before GA. Best-effort emission against an unknown schema
produces a file that looks right and is wrong. Fail closed.

### D3 — Rule identity is content-addressed, never positional

`RuleId` = `"caro."` + first 12 hex of `sha256(regex ++ 0x1f ++ shell_specific_wire)`.
Reason codes are **generated**, never authored: `caro.<risk>.<rule_id>`.

This fixes ACS failure mode F3 (`reason` is an unregistered free-form string — the
tutorial returns `"external_recipient_blocked"`, with no namespace and no stability
promise). Two vendors' policies will not collide, a host can branch on a stable string,
and reordering `patterns.rs` does not churn every id in every downstream artifact.

### D4 — The failure mode is solved by measurement, not by pretending it isn't there

**This is the ADR's central decision.** Every exported rule is assigned a
`FidelityClass`, computed at export time by evaluating the *rendered Rego* through
`regorus` against Caro's corpus and diffing it against the *native* `SafetyValidator`:

- **`LiteralSafe`** — the two agree on every corpus case that exercises the rule, and
  the pattern contains no construct whose match depends on shell rewriting.
- **`Subset { missed_cases }`** — the exported rule is a strict subset of native
  behaviour. It under-blocks; it never over-blocks.
- **`SemanticsRequired`** — a correct decision needs shell parsing the target engine
  cannot perform. Emitted as `deny` only when `Subset` also holds; otherwise omitted and
  **counted**.

Every disagreement lands in `FidelityReport.false_negatives` **by corpus-case id and
rule id**. There is no code path that produces an artifact without a report, and no flag
that suppresses one.

This is what makes the export honest rather than harmful. Caro is not claiming the Rego
bundle re-parses the shell — it demonstrably cannot. Caro is claiming, with numbers
regenerable offline by anyone holding the binary, *exactly which commands the exported
policy stops and which it does not.* A competitor shipping a rule pack cannot make that
claim, because measuring it requires owning the reference implementation.

### D5 — Caro never emits a `transform` verdict

ACS supports `allow` / `transform` / `deny`. Caro emits **only** `allow` and `deny`.

`transform` rewrites the policy target and the host then executes the tool with the
rewritten value; `post_tool_call` evaluates the tool *result*, not the rewritten *args*.
Nothing in the documented flow re-runs `pre_tool_call` against the transformed target.
**A rewritten command is a new command and deserves a fresh verdict it would not
receive.** This is permanent, not deferred.

### D6 — Over-blocking is an error, not a warning

`FidelityReport.false_positives` — cases the native validator allows and the exported
rule denies — **must be empty**. A non-empty vector makes `build` return `Err`.

Under-blocking is a measured, disclosed limitation. Over-blocking breaks a stranger's
working agent, and the artifact wearing Caro's name gets deleted along with any chance
of a second look. The asymmetry is deliberate.

### D7 — Coverage completeness, borrowed from ADR-055

`exported + omitted == total_patterns`, and `total_patterns == DANGEROUS_PATTERNS.len()`
— **computed from code, never quoted from docs** (see drift D-a: the docs are wrong by
22%). `by_class` sums to `exported`. The report may not silently drop a rule.

### D8 — Unmeasured is not safe; it is `SemanticsRequired`

Every rule carries `corpus_coverage: u32`, the number of corpus cases that exercised it.
**A rule with `corpus_coverage == 0` is classed `SemanticsRequired` regardless of how
simple its regex looks**, and the export exits `17` naming it. Additionally, `build`
cross-compiles every pattern through `regorus` and refuses any pattern the two regex
engines parse differently, listing it in `omitted`.

This exists because of the demoware failure in §"Consequences" below: user-supplied
`[[safety.custom_patterns]]` have no corpus cases at all, and a fidelity report that
silently rates them as clean is worse than no report.

### Output and exit contract

```
caro policy export --format {acs,opa,json} [--out DIR] [--acs-version 0.3.1-beta]
                   [--min-risk {safe,moderate,high,critical}] [--shell SHELL] [--json]
caro policy verify --dir DIR [--json]
```

`--json` writes exactly one `PolicyBundle` (export) or `FidelityReport` (verify) to
stdout; all human text to stderr. Rendered files: `manifest.yaml` (ACS only),
`policy/caro_policy.rego` (ACS + OPA), `caro-policy.json` and `caro-fidelity.json`
(always).

| Code | Meaning |
|---|---|
| `0` | Succeeded, **zero** false negatives — full measured parity, safe to publish |
| `17` | Succeeded **with measured loss** — not a failure; gaps enumerated in `caro-fidelity.json`; CI may gate on it |
| `2` | Usage error (unknown format, unsupported `--acs-version`) — fail closed, nothing written |
| `1` | Internal error — fail closed, partial output removed |

### Files that change

New: `src/safety/export/{mod,acs,rego}.rs`, `docs/adr/ADR-057-*.md`,
`tests/policy_export_contract.rs`. Touched: `src/safety/mod.rs` (one `pub mod` line),
`src/main.rs` (one `Commands` variant + dispatch arm), `Cargo.toml` (`regorus` optional
+ `policy-export` feature, off by default).

Explicitly **not** touched: `src/safety/patterns.rs` (read, never edited),
`src/safety/validator.rs`, `src/models/`, `src/governance/` (the AGT spike stays a
spike — this feature emits *files*, it does not link AGT), any config schema.
`serde_yaml` is already a direct dependency, so `regorus` is the only dependency delta.

---

## Consequences

### Positive

- Caro's rules reach AGT's install base without asking anyone to switch gates, and
  without Caro building a console.
- **Caro can audit its exported policy more cheaply than the engine consuming it can.**
  `regorus` is pure Rust and already in the lock file; ACS's own tutorial requires the
  `opa` CLI on `PATH` for the same job. Verification runs inside `cargo test`, offline.
- The fidelity report makes the shell-semantics differentiator **legible to a buyer**
  rather than a claim on a website. It is a printed diff between the tool-call layer and
  the command-string layer — which is the entire remaining moat, per the 08-20 scan.
- Reproducibility is an assertable equality, not marketing.
- Zero lifecycle obligations. ACS is stateless; the export is a build-time artifact
  producer. The "no daemon, no state" constraint costs nothing here.

### Negative, and honestly

- **The deliverable includes a document arguing against itself.** A fidelity report
  saying "this export catches 41 of 67 rules' worth of cases" is, read uncharitably, a
  reason not to use the export. This objection is real and goes to the
  `devils-advocate` review, not into a rebuttal here. The answer this ADR would offer:
  the alternative is not a better artifact, it is the same artifact with the number
  hidden.
- Exit `17` will be read as failure by naive CI. Documentation burden.
- ACS is Public Preview. `0.3.1-beta` will move and the renderer will need updating;
  D2 makes that a loud break rather than a quiet wrong file, which is the right trade
  but is still recurring maintenance.
- `regorus` and Rust's `regex` are different engines. D8's cross-compile check will
  omit some patterns, which lowers exported coverage — correctly, and visibly.
- A second target format (Cedar) multiplies the fidelity matrix. Deliberately deferred.

### The demoware trap — what breaks at 100 real users

*Required by `.claude/rules/validation-discipline.md` Gate 3.*

**The assumption.** The demo exports 67 built-in patterns against a curated corpus and
prints a clean report. It assumes the corpus is representative of production traffic and
that users export roughly the default set.

**How it fails.** Both break in the same direction. Real users carry
`[[safety.custom_patterns]]` — regexes Caro's authors never wrote, never TDD-validated,
and for which the corpus has **zero** cases. Those rules get a `FidelityClass` computed
from a corpus that never exercises them, producing a **fidelity report that is
confidently wrong**. That is worse than no report: the user reads `false_negatives: []`,
concludes parity, installs the bundle into AGT as their only gate, and Rego's fail-open
default silently allows everything their custom patterns existed to stop.

**Instrumentation.** `corpus_coverage: u32` per rule. `corpus_coverage == 0` means
*unmeasured*, which is not the same as safe.

**Fallback.** D8. Unmeasured ⇒ `SemanticsRequired` ⇒ exit `17` naming the rule.
Cross-engine regex disagreement ⇒ omitted and listed. **Fail closed on unmeasured.**
This is why `corpus_coverage` is a required field in schema v1 rather than a v2 addition
— the trap is designed out before implementation, not patched after a retro.

---

## Alternatives considered

**A. Export the regexes, no fidelity report.** The two-day version. Rejected: ACS's
fail-open rule default (F4) plus Rego's inability to reason about shell rewriting (F5)
means the artifact is silently, confidently incomplete while carrying Caro's name. This
is the single most damaging thing Caro could ship to its own positioning.

**B. Refuse to export at all; insist callers invoke `caro` at runtime.** Philosophically
clean — full fidelity or nothing. Rejected: it cedes the entire "be the content"
strategy, and users will hand-write worse Rego from Caro's README anyway (a README that,
per drift D-b, documents a risk taxonomy Caro does not have).

**C. Ship a `caro serve --validator` HTTPS shim so foreign engines call Caro live.**
Full fidelity, no translation loss. Rejected **for this ADR**: it is a daemon, which
these constraints forbid, and it moves Caro from artifact-producer to
long-running-service with an availability contract. It is a good idea and deserves its
own ADR and its own argument.

**D. Emit `transform` verdicts that rewrite dangerous commands into safe ones.**
Tempting — Caro's patterns are named and templated. Rejected on two independent grounds:
F6 (the transformed target is never re-evaluated) and the standing strategic rule that
remediation must come from a deterministic pattern→template map with no model in the
decision path. ADR-052 reserves `suggested_alternative` for that conversation.

**E. Link `agentmesh` and evaluate in-process rather than emitting files.** Rejected:
that is a runtime integration with one vendor, not a portable artifact. The `governance`
feature stays a Phase-0 spike. Emitting a file that AGT reads is not an integration —
which is exactly why it also works for OPA, regorus, and anything else that eats a Rego
bundle.

**F. Use the `opa` CLI to verify exported policy.** Rejected: it adds a Go binary to
Caro's test prerequisites for a job `regorus` already does in-process, offline, from a
crate already resolved in this workspace.

---

## Follow-ups this ADR does not own

- **Write the §3.6 `caro.assessment.v1` ADR.** Two ADRs now block on it.
- **Give the exit-code registry an owner** (ADR-024's `ExitCode` enum). Six ADRs are
  currently allocating process exit codes in prose against a tree containing one
  constant.
- **Fix drift D-a in a standalone docs PR.** Caro has 67 dangerous-command patterns and
  has been telling the world it has 52. Per `.claude/rules/good-boy-scout.md` this does
  not ride along inside a feature PR.

---

## Sources

Fetched and read 2026-08-24:

- [microsoft/agent-governance-toolkit — `docs/quickstart.md`](https://github.com/microsoft/agent-governance-toolkit/blob/main/docs/quickstart.md)
- [microsoft/agent-governance-toolkit — `docs/tutorials/55-agent-control-specification.md`](https://github.com/microsoft/agent-governance-toolkit/blob/main/docs/tutorials/55-agent-control-specification.md)
- [microsoft/agent-governance-toolkit — repository](https://github.com/microsoft/agent-governance-toolkit)
- [Open Policy Agent — WebAssembly](https://www.openpolicyagent.org/docs/wasm)
- [Open Policy Agent — Bundles](https://openpolicyagent.org/docs/v0.40.0/management-bundles)

Referenced, not re-fetched:

- `.hermes/digests/2026-08-20-agent-market-scan.md` §3.6, §3.9, §4
- [GuardFall Exposes Open-Source AI Coding Agents — The Hacker News, 2026-06-30](https://thehackernews.com/2026/06/guardfall-exposes-open-source-ai-coding.html) (second-hand)
- Companion scope: `caro-scope-policy-export-2026-08-24.md`
