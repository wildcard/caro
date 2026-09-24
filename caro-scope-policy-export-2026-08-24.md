# Implementation Scope — Measured-Fidelity Policy Export (`caro policy export`)

**Feature under analysis:** **Agent Control Specification (ACS)**, the policy layer
vendored into **Microsoft Agent Governance Toolkit** (`microsoft/agent-governance-toolkit`,
MIT, 5.6k★ / ~930–960 forks at fetch time). Specifically the artifact pair that
`cmd_policy_gen` emits — a flat `manifest.yaml` binding a **Rego bundle** to named
**intervention points** — and the `pre_tool_call` verdict contract that a host enforces.
Read live **2026-08-24** against two primary sources in the repo:
`docs/quickstart.md` (`last_reviewed: 2026-07-12`) and
`docs/tutorials/55-agent-control-specification.md` (`last_reviewed: 2026-06-02`).

**Equivalent we are scoping for Caro:** `caro policy export` — emit Caro's built-in
dangerous-command patterns as a consumable policy artifact for someone else's engine
(ACS/AGT first, generic OPA bundle second), **together with a machine-readable measurement
of how much safety the translation loses.** Caro becomes the *content* inside other
people's gates without ever claiming the exported artifact is as good as the native
validator — because the export itself proves, offline and deterministically, exactly
where it is not.

**Date:** 2026-08-24 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-057-measured-fidelity-policy-export.md`

> **Provenance note (autonomous run).** No user present; the task template's
> `[FEATURE NAME]` was unbound, so target selection was mine. Selection rationale:
> `.hermes/digests/2026-08-20-agent-market-scan.md` lists ten opportunities. §3.8
> (observe mode) was consumed by ADR-056 on 08-21. §3.10 (`suggested_alternative`) is
> reserved by ADR-052. §3.7 is an eval suite, not an ADR. §3.6 (`caro.assessment.v1`)
> is the memo's #2 recommendation and is **still unwritten** — this document does not
> supersede it and in fact depends on it (see §3.3). §3.9 — *"ship Caro patterns as
> content for other people's engines"* — is the largest unscoped item, has **zero**
> mentions anywhere under `docs/adr/` (`grep -rln "rego\|Rego\|PolicyDocument\|policy export" docs/adr/`
> returns nothing), and is the one item whose upstream dependency has **already
> landed**: `agentmesh` is a live optional dependency in `Cargo.toml:119`.
>
> **Verified vs. not.**
> - **Verified by fetch (2026-08-24):** the ACS manifest schema, the Rego verdict
>   shape, the canonical policy-input shape, the `allow`/`transform`/`deny` triple, the
>   fail-closed-on-malformed-manifest behaviour, the "ACS is stateless" claim, the
>   Public-Preview caveat, the `opa`-on-`PATH` prerequisite, and the
>   `agent_control_specification_version: "0.3.1-beta"` pin. All quoted from the two
>   docs named above.
> - **Verified against the Caro tree (2026-08-24):** every count, type, field, path and
>   drift claim in §2.4 and §3.2 was produced by reading `src/`, `Cargo.toml` and
>   `Cargo.lock` at `1.4.0`. Commands shown inline where non-obvious.
> - **Second-hand, not independently verified:** Execlave's `rego` and `custom_validator`
>   policy types and Phinq's classification model (both via the 08-20 Hermes memo);
>   the GuardFall 10-of-11 result (via *The Hacker News*, 2026-06-30, itself reporting
>   Adversa AI's self-published finding). Repo star/fork/issue counts are a point-in-time
>   read of a rendered GitHub page and will drift.
> - **Fetched but not usable:** GitHub's HTML shell dominated both fetches; the raw
>   `raw.githubusercontent.com` paths are outside the fetch provenance set. The Markdown
>   bodies did render in full, which is why the schema quotes above are trustworthy, but
>   nothing was read from `policy-engine/` source — only from docs.
>
> **Gate warning.** `caro policy export` is a new user-facing capability class, so
> `.claude/rules/validation-discipline.md` attaches in full: 20 first-hand transcripts,
> the demoware-trap section (§3.7 below, written), and a `devils-advocate` review
> before any implementation PR opens. Per `.claude/rules/git-workflow.md` this file and
> the ADR are left **uncommitted** for a human to branch and PR.

---

## Phase 1 — Feature Research

### 1.1 What problem it solves, and for whom

The buyer is a platform or security engineer who has been handed an agent that already
works and told to make it governable. They have an enforcement point — an SDK wrapper, a
proxy, a framework adapter — and no rules to put in it. Every product in this market
ships the same shape: an **engine** plus a **blank policy file**.

AGT is the most credible instance of that shape. `pip install agent-governance-toolkit[full]`,
then:

```
python -m agent_os.cli.cmd_policy_gen --template strict --output policies/
agt lint-policy policies/manifest.yaml
```

which produces `manifest.yaml` and `policy.rego`. The manifest binds the Rego policy to
*native intervention points*; the host then mediates its own tool calls through it:

```python
runtime = AgentControl.from_path("policies/manifest.yaml")
session = HostSession(runtime, agent_id="quickstart-agent", session_id="quickstart-session")
evaluation = session.pre_tool_call(tool_name="delete_file", args={"path": "report.txt"})
```

The `--template strict` starter is a *template*. The engineer still has to know which
tool calls are dangerous and encode that knowledge in Rego. That knowledge is precisely
what Caro has 67 TDD-validated instances of and currently no way to hand over.

### 1.2 Core architecture — data flow, key types, separation of concerns

ACS's own diagram, quoted verbatim:

```
Host adapter -> snapshot -> ACS runtime -> verdict -> host enforcement
```

Four separations worth stealing:

1. **Decision runtime vs. enforcement point.** "ACS is the policy decision runtime. Your
   application or adapter is the policy enforcement point." The thing that decides never
   executes; the thing that executes never decides.
2. **Statelessness is explicit and load-bearing.** "ACS is stateless. The host supplies
   all context for every evaluation, including the intervention point, tool call, tool
   result, and any ambient labels or metadata." The quickstart reinforces it from the
   other side: *"The runtime itself remains free of session counters."*
3. **Manifest binds policy to interception site.** A single Rego policy is bound at both
   `pre_tool_call` (evaluating `$.tool_call.args`) and `post_tool_call` (evaluating
   `$.tool_result.value`) via `policy_target` / `policy_target_kind` JSONPath-ish
   selectors.
4. **Canonical policy input.** Rules read a fixed envelope rather than host-local state:

```json
{
  "intervention_point": "pre_tool_call",
  "policy_target": {
    "path": "$.tool_call.args",
    "kind": "tool_args",
    "value": { "to": "customer@example.com", "body": "Your case is ready." }
  },
  "tool": { "name": "send_email", "id": "send_email", "clearance": "internal" }
}
```

The manifest that produces it:

```yaml
agent_control_specification_version: "0.3.1-beta"
metadata:
  name: acs-email-tutorial
policies:
  email_policy:
    type: rego
    bundle: ./policy
    query: data.agent_control_specification.email_policy.verdict
intervention_points:
  pre_tool_call:
    policy_target: $.tool_call.args
    policy_target_kind: tool_args
    tool_name_from: $.tool_call.name
    policy:
      id: email_policy
tools:
  send_email:
    type: Tool
    id: send_email
    clearance: internal
```

### 1.3 Why it is experimental / limited — the failure modes

ACS says so itself: *"ACS is vendored into AGT under `policy-engine/` as the AGT 5.0
policy layer. The APIs and manifest shape may change before GA."* Five concrete failure
modes, in ascending order of how much they matter to Caro:

**F1 — Install weight.** Python 3.11+, `pip`, a maturin-built native Rust core when
installed from source, and *"OPA-backed Rego examples require the `opa` CLI on `PATH`."*
That is three toolchains before a single rule evaluates. Caro's user has a single static
binary.

**F2 — Preview-grade schema.** `0.3.1-beta`, shape may change. Anything Caro emits must
carry the target schema version it was generated against and must fail loudly rather
than emit a plausible-looking file against an unknown version.

**F3 — `reason` is an unregistered free-form string.** The tutorial returns
`"external_recipient_blocked"`. There is no registry, no namespace, no stability
promise. Two policies from two vendors will collide, and a host branching on `reason`
has no contract to branch against.

**F4 — Policy-level default is fail-*open*.** `default verdict := {"decision": "allow"}`.
The *runtime* fails closed — "malformed manifests, missing paths, policy dispatcher
failures, and invalid transform targets produce `deny` verdicts with reserved
runtime-error reasons" — but a rule that simply *does not fire* allows. For a
dangerous-command denylist this default is correct (an allowlist would break every
legitimate command), which means **every gap in the ruleset is silent by construction.**
An unmeasured export is therefore not merely incomplete; it is confidently, silently
incomplete.

**F5 — the one this ADR exists to solve: `policy_target_kind: tool_args` cannot see a
shell.** The policy receives `input.policy_target.value` as a JSON object of tool
arguments. For a shell tool that object is `{"command": "<a raw string>"}`. Rego's
string toolkit is `regex.match`, `contains`, `startswith`, `endswith`. Rego has no
tokenizer, no quote removal, no word splitting, no parameter expansion, no `$IFS`, no
alias or builtin resolution. Every one of those is a rewriting step **bash performs
after** the policy has already returned `allow`.

This is not a hypothetical. Adversa AI's **GuardFall** (reported by *The Hacker News*,
2026-06-30) bypassed safety checks in **10 of 11** open-source coding agents by
exploiting exactly the gap between the string a checker inspects and the string the
shell executes; the sole survivor parsed the command the way bash would before deciding.

So the naive version of this feature — "serialize 67 regexes into `regex.match` calls" —
does not port Caro's safety property. It ports Caro's *strings* into an engine that
cannot reproduce Caro's *semantics*, under a default that stays silent when they
disagree. Shipping that unlabelled would be the single most damaging thing Caro could do
to its own positioning, because the artifact would carry Caro's name.

**F6 — `transform` mutates the policy target without re-evaluation.** The tutorial's
`transform` verdict rewrites `$policy_target.body` and the host then executes the tool
with the rewritten value. `post_tool_call` evaluates the tool *result*, not the rewritten
*args*. Nothing in the documented flow re-runs `pre_tool_call` against the transformed
target. Caro must therefore never emit a `transform` verdict for a shell string — a
rewritten command is a new command and deserves a fresh verdict it would not get.

### 1.4 Structured output contract

The verdict Rego returns, verbatim from the tutorial:

```rego
default verdict := {"decision": "allow"}

verdict := {
  "decision": "deny",
  "reason": "external_recipient_blocked",
  "message": "Messages to external recipients are blocked."
} if { ... }
```

with a third arm carrying `"decision": "transform"` plus
`"transform": {"path": "$policy_target.body", "value": "..."}`.

Host-side, denial surfaces two ways depending on entry point:
`evaluation.verdict.decision.permits` (a boolean on the `pre_tool_call` path) and a
raised `AgentControlBlocked` / `PolicyViolationError` (on the `run_tool()` path), with
`PolicyViolationError.from_evaluation_result(evaluation)`,
`error.evaluation_result.audit_record()`, and *"The public exception text is sanitized."*

Decision vocabulary: **`allow` | `transform` | `deny`**. Note what is missing versus the
rest of the market: there is no `warn` and no `require_approval` at the verdict layer
(AGT documents approval workflows separately). Caro's export must therefore *down-map*
its own richer decision space into three values and record what it lost — another
fidelity axis, and one the 3.6 contract will need to define before this is buildable.

### 1.5 Session / context lifecycle

There is none, deliberately, and this is the best news in the whole document. ACS is
stateless per evaluation; the host owns identity (`agent_id`, `session_id`) and passes a
complete snapshot each time. The only stateful note in either document is a billing one:
*"Attempted tool calls are charged before evaluation, including denied attempts."*

Redundant initialization is avoided the ordinary way — `AgentControl.from_path(...)`
once, reuse the runtime object — not by any session-cache protocol. Caro's export is a
build-time artifact producer and inherits zero lifecycle obligations. This is what makes
the "pure subprocess, no daemon, no state" constraint free rather than expensive here.

---

## Phase 2 — Competitive Differentiation

### 2.1 What they get right, and we replicate

- **Manifest binds rules to interception sites; rules don't know about hosts.** Caro's
  export should emit the same two-file shape (manifest + bundle) and never require a
  Caro-specific host.
- **A canonical policy input.** Rules read a fixed envelope. Caro's exported rules must
  read *only* ACS canonical fields — never a Caro-shaped input — or the artifact isn't
  actually portable.
- **Explicit statelessness, stated in the docs as a property.** Caro should state it
  the same way, in the manifest itself, not only in prose.
- **Runtime fails closed on malformed input.** Caro's `policy verify` must adopt the same
  stance: an unparseable or version-mismatched artifact is a hard error, not a warning.
- **Version pin in the artifact.** `agent_control_specification_version: "0.3.1-beta"`
  sits at the top of the manifest. Caro's export writes both the ACS version it targeted
  and the Caro version that produced it.

### 2.2 Their gaps, avoided by designing the schema first

| Gap | Their state | Caro's schema decision |
|---|---|---|
| F3 — unregistered `reason` | free-form string, no namespace | every emitted reason is `caro.<risk>.<rule_id>`; `rule_id` is stable, content-addressed, and exported in the report |
| F4 — silent gaps under fail-open default | no gap accounting exists | **the export refuses to omit a measurement**: a `fidelity` block enumerates every rule the translation weakened, by id |
| F5 — no shell semantics in Rego | not acknowledged in the docs | rules are partitioned into *literal-safe* and *semantics-required*; the latter are never emitted as silent `allow` |
| F6 — `transform` without re-evaluation | documented flow does not re-check | Caro emits **no** `transform` verdicts, ever, for shell targets |
| Decision vocabulary loss | three values, no `warn`/`require_approval` | down-mapping table is data in the manifest, not logic in the exporter |
| Preview schema drift | "may change before GA" | `--acs-version` is an explicit flag with exactly one supported value; unknown versions are a hard error, never best-effort |

### 2.3 Our unique positioning

Four things Caro can do here that no competitor in the 08-20 scan can:

1. **Measure the loss offline, in-process, with no second toolchain.** `regorus 0.10.0`
   is already in `Cargo.lock` (line 8247) as a transitive dependency of `agentmesh`. It
   is a pure-Rust Rego interpreter. Caro can evaluate its own exported Rego against its
   own corpus **inside `cargo test`**, on a laptop, on a plane, with no `opa` binary, no
   Python, no network. AGT's own tutorial requires `opa` on `PATH` to do the same thing.
   *Caro can audit its exported policy more cheaply than the engine that consumes it can.*
2. **Own the layer below the tool call.** Phinq, Execlave, AGT and ACS all classify by
   tool + argument. Caro re-parses the command string. The export is the first artifact
   where that distinction becomes *legible to a buyer* rather than a claim on a website —
   because the fidelity report is a diff between the two layers, printed.
3. **Ship content, not a console.** The 08-20 memo's standing instruction is *"be the
   content and the verdict; do not be the console."* This is the literal implementation
   of that sentence, and it reaches AGT's install base without asking anyone to switch
   gates.
4. **Offline and reproducible by construction.** Same Caro version + same config ⇒
   byte-identical artifact. No registry fetch, no telemetry, no clock. That is testable
   as an equality assertion (§3.5, T4) and none of the SaaS gates can claim it.

### 2.4 What Caro already has (verified against the tree at `1.4.0`, 2026-08-24)

| Asset | Location | State |
|---|---|---|
| `DangerPattern { pattern, risk_level, description, shell_specific }` | `src/safety/mod.rs:334` | **already `Serialize + Deserialize`**; `shell_specific` is `#[serde(default)]` |
| Built-in pattern set | `src/safety/patterns.rs` | **67** `DangerPattern` literals (`grep -c "DangerPattern {"`), not "52+" |
| `RiskLevel` | `src/models/mod.rs:152` | `Safe \| Moderate \| High \| Critical`, `Serialize`, `JsonSchema`, `#[serde(rename_all = "lowercase")]`, and **`Ord`** — the ordering the down-map needs is already there |
| `ShellType` | `src/models/mod.rs:419` | 7 variants, same derives, same lowercase wire form |
| `SafetyConfig { safety_level, max_command_length, custom_patterns, allowlist_patterns }` | `src/safety/mod.rs:166` | layered TOML load already implemented (`src/safety/mod.rs:697`), with `[[safety.custom_patterns]]` and an external-file path |
| `ValidationResult` / `SafetyDecision` | `src/safety/mod.rs:175` / `:189` | the native verdict the export is measured against |
| Pattern accessors | `src/safety/patterns.rs:512,532,540,568` | `validate_patterns`, `get_patterns_for_shell`, `get_patterns_by_risk`, `get_compiled_patterns_for_shell` — the exporter needs **no new traversal code** |
| `regorus 0.10.0` | `Cargo.lock:8247` | present transitively via `agentmesh`; **not** currently a direct dependency |
| `cedar-policy 2.4.2` | `Cargo.lock:1599` | present; out of scope this version (§3.6) |
| `serde_yaml 0.9` | `Cargo.toml:43` | **already a direct dependency** — manifest emission needs no new crate |
| `governance` feature + `agentmesh 3.6` | `Cargo.toml:119,204` | Phase-0 build spike only; `src/governance/mod.rs` is a single `build_spike()` |
| Custom-pattern integration test | `tests/custom_patterns_toml.rs` | the precedent for how a pattern-set test is written here |

**Four drifts found while verifying, each of which the export would otherwise propagate
into a published artifact:**

- **D-a.** `README.md:82,946` and `CLAUDE.md:32,108` say **"52+"** patterns; the tree has
  **67**. The export's headline number must be computed from `DANGEROUS_PATTERNS.len()`,
  never quoted from docs, and the fidelity report should carry it so the docs can be
  fixed from data.
- **D-b.** `CLAUDE.md:108` describes risk levels as **"CRITICAL, HIGH, MEDIUM, LOW"**.
  The actual enum is `Safe | Moderate | High | Critical` — *there is no `Medium` and no
  `Low`.* The 08-20 memo repeats the wrong taxonomy. Any hand-written Rego copying the
  doc would emit unmatchable strings.
- **D-c.** `CLAUDE.md` states MSRV **1.83**; `Cargo.toml:5` says `rust-version = "1.85"`.
- **D-d.** `src/governance/mod.rs` points at
  `.claude/plans/intgrate-https-github-com-microsoft-agen-witty-scroll.md`. That file is
  **not in the tree** — `.claude/plans/` contains only `caro-safety-library-api-scope.md`.
  The AGT integration plan referenced by the merged Phase-0 spike is missing, which is
  why this scope reconstructs Phase 1 from primary sources rather than from the plan.

Also verified absent, confirming the gap is real: no `Policy`, `Assess`, `Scan` or
`Audit` variant exists in `src/main.rs`'s `Commands` enum (the existing `Export` variant
is CaroML runbook export and is unrelated); `src/assessment/` is **hardware** assessment
(`cpu.rs`, `gpu.rs`, `memory.rs`) and must not be confused with safety assessment; the
only exit-code constant in the tree is `EXIT_CODE_EDIT = 201` at `src/main.rs:938`.

---

## Phase 3 — Scope Definition

### 3.1 ADR

`docs/adr/ADR-057-measured-fidelity-policy-export.md` — context, decision (D1–D8),
consequences, alternatives considered. Highest existing is ADR-056; per
`.claude/rules/adr-numbering.md`, renumber on merge if another 057 lands first.

### 3.2 New types

All live in **one new module**, `src/safety/export/`, under `src/safety/` because they
are a projection of the pattern set and must not drift from it. All derive
`Debug, Clone, Serialize, Deserialize` from day one; all use
`#[serde(rename_all = "snake_case")]` and `#[serde(deny_unknown_fields)]` on read paths.

```rust
/// The exported ruleset, before it is rendered to any target format.
/// Ordering is the source order of `DANGEROUS_PATTERNS`, always.
pub struct PolicyBundle {
    pub schema_version: u32,          // 1
    pub caro_version: String,         // env!("CARGO_PKG_VERSION")
    pub target: PolicyTarget,
    pub rules: Vec<ExportedRule>,
    pub fidelity: FidelityReport,
    pub digest: String,               // sha256 over the canonical rules+target encoding
}

pub enum PolicyTarget {
    /// ACS manifest + Rego bundle. Carries the exact ACS schema string.
    Acs { acs_version: String },      // only "0.3.1-beta" accepted in v1
    /// Plain OPA bundle, no manifest, for regorus/OPA consumers.
    OpaBundle,
    /// Caro's own canonical JSON — lossless, the reference the others are measured against.
    Json,
}

pub struct ExportedRule {
    pub rule_id: RuleId,
    pub risk_level: RiskLevel,        // reuses src/models — no new taxonomy
    pub description: String,
    pub regex: String,                // the DangerPattern source, unmodified
    pub shell_specific: Option<ShellType>,
    pub decision: ExportDecision,
    pub reason_code: String,          // "caro.<risk>.<rule_id>", generated, never authored
    pub fidelity_class: FidelityClass,
}

/// Stable across runs and across pattern reordering: `caro.` + first 12 hex of
/// sha256(regex ++ 0x1f ++ shell_specific-wire-form). Deliberately NOT the index.
pub struct RuleId(String);

/// The ACS verdict vocabulary, down-mapped. `Transform` is intentionally absent
/// (see D5) — Caro never rewrites a shell string in someone else's engine.
pub enum ExportDecision { Allow, Deny }

pub enum FidelityClass {
    /// Native validator and exported regex agree on every corpus case, and the
    /// pattern contains no construct whose match depends on shell rewriting.
    LiteralSafe,
    /// The exported regex is a strict subset of native behaviour: everything it
    /// denies, the native validator also denies. It under-blocks; it never over-blocks.
    Subset { missed_cases: u32 },
    /// Correct decision requires shell parsing the target engine cannot perform.
    /// Emitted as `deny` only if `Subset` also holds; otherwise omitted and counted.
    SemanticsRequired,
}

pub struct FidelityReport {
    pub corpus_id: String,            // which corpus produced these numbers
    pub corpus_cases: u32,
    pub total_patterns: u32,          // DANGEROUS_PATTERNS.len() — see drift D-a
    pub exported: u32,
    pub omitted: u32,
    pub by_class: BTreeMap<FidelityClass, u32>,   // BTreeMap: deterministic order
    pub false_negatives: Vec<FidelityGap>,        // native denies, export allows
    pub false_positives: Vec<FidelityGap>,        // export denies, native allows — MUST be empty
    pub decision_downmap: Vec<DownMapEntry>,
}

pub struct FidelityGap {
    pub rule_id: Option<RuleId>,      // None when no rule fired at all
    pub corpus_case_id: String,
    pub native_decision: String,
    pub exported_decision: ExportDecision,
}

pub struct DownMapEntry {
    pub native: RiskLevel,
    pub exported: ExportDecision,
    pub note: String,                 // e.g. "no `warn` in ACS 0.3.1-beta verdict vocabulary"
}
```

Method contracts (three functions, no trait, no builder):

```rust
/// Pure. No I/O, no clock, no env. Same inputs ⇒ identical bundle, byte for byte.
pub fn build(cfg: &SafetyConfig, target: PolicyTarget, corpus: &Corpus)
    -> Result<PolicyBundle, ExportError>;

/// Renders to files. The ONLY function in this module that touches the filesystem.
pub fn render(bundle: &PolicyBundle, out_dir: &Path) -> Result<Vec<PathBuf>, ExportError>;

/// Re-evaluates a rendered bundle with regorus and re-derives the FidelityReport.
/// Used by `caro policy verify` and by T2/T3 below. Never mutates.
pub fn verify(bundle_dir: &Path, corpus: &Corpus) -> Result<FidelityReport, ExportError>;
```

`build` takes `&SafetyConfig` rather than reading globals, so `custom_patterns` and
`allowlist_patterns` flow through the existing layered TOML loader
(`src/safety/mod.rs:697`) with zero duplication. `ExportError` is a `thiserror` enum per
house style.

### 3.3 Minimal set of files that change

One new module directory, four touched files, no new top-level module.

| File | Change |
|---|---|
| `src/safety/export/mod.rs` | **new** — the types above, `build`, `render`, `verify` |
| `src/safety/export/acs.rs` | **new** — `manifest.yaml` + `caro_policy.rego` renderer |
| `src/safety/export/rego.rs` | **new** — Rego text generation shared by ACS and `OpaBundle` |
| `src/safety/mod.rs` | `pub mod export;` — one line |
| `src/main.rs` | one `Policy { .. }` variant on `Commands` + its dispatch arm |
| `Cargo.toml` | `regorus = { version = "0.10", optional = true }`; `policy-export = ["dep:regorus"]` |
| `docs/adr/ADR-057-*.md` | **new** |

Deliberately **not** changed: `src/safety/patterns.rs` (the exporter reads it, never
edits it), `src/safety/validator.rs`, `src/models/`, `src/governance/` (the AGT spike
stays a spike; this feature emits *files*, it does not link AGT), `src/assessment/`
(hardware, unrelated), any config schema. `serde_yaml` is already present, so the only
dependency delta is `regorus` — and per `.claude/rules/external-sdk-integration.md`
its five checks are largely pre-discharged, since `regorus 0.10.0` already resolves in
this exact workspace as an `agentmesh` transitive (`Cargo.lock:8247`). The spike PR
should still record all five explicitly; promoting a transitive to a direct dependency
changes the license and MSRV surface that Caro is accountable for.

### 3.4 Exit-code / output contract

```
caro policy export --format {acs,opa,json} [--out DIR] [--acs-version 0.3.1-beta]
                   [--min-risk {safe,moderate,high,critical}] [--shell SHELL] [--json]
caro policy verify --dir DIR [--json]
```

`--json` on either verb writes exactly one `PolicyBundle` (export) or `FidelityReport`
(verify) to **stdout** and nothing else; all human text goes to stderr. `--out` defaults
to `./caro-policy/`. Rendered files:

```
caro-policy/
  manifest.yaml            # acs only; carries agent_control_specification_version
  policy/caro_policy.rego  # acs + opa
  caro-policy.json         # always — the lossless reference bundle
  caro-fidelity.json       # always — the FidelityReport, standalone
```

| Code | Meaning | Machine contract |
|---|---|---|
| `0` | Export/verify succeeded, **zero** false negatives | artifact is at full measured parity; safe to publish |
| `17` | Export succeeded **with measured loss** — `false_negatives` non-empty | *not a failure.* CI may gate on it; the artifact is valid and its gaps are enumerated in `caro-fidelity.json` |
| `2` | Usage error — unknown `--format`, unsupported `--acs-version` | fail closed; nothing written |
| `1` | Internal error — corpus unreadable, regorus evaluation failed, I/O | fail closed; partial output removed before exit |

Exit code **17** is claimed here. **The registry remains contended and unowned**: two
ADRs claim 13, ADR-053 claims 14, ADR-055 claims an unnumbered "new, distinct" code
(presumed 15), ADR-056 claims 16, and the tree still contains exactly one exit constant
(`EXIT_CODE_EDIT = 201`). ADR-053's complaint stands and this ADR repeats it: **the
exit-code registry needs an owner, and that owner should be ADR-024's `ExitCode` enum,
not prose in five sibling ADRs.**

A **non-empty `false_positives`** array is not an exit code — it is an assertion
failure. Caro over-blocking inside someone else's engine breaks their agent and gets the
artifact deleted. `build` returns `Err` if that vector is non-empty (§3.5, T3).

### 3.5 Integration tests — deterministic input → fixed JSON + exit code

New file `tests/policy_export_contract.rs`, following `tests/custom_patterns_toml.rs`.
Fixtures under `tests/fixtures/policy_export/`. Every test is offline and hermetic.

**T1 — golden ACS manifest.** `build(default_config, Acs{"0.3.1-beta"}, corpus)` →
`render` → assert `manifest.yaml` equals a checked-in golden byte-for-byte. Locks the
`agent_control_specification_version` pin, key ordering, and the
`data.caro.policy.verdict` query path. Exit `0` or `17`.

**T2 — regorus round-trip on a known-dangerous case.** Feed the rendered Rego a
canonical ACS policy input with
`policy_target.value = {"command": "rm -rf /"}` and `tool.name = "bash"`. Assert the
returned verdict is exactly
`{"decision":"deny","reason":"caro.critical.<id>","message":"..."}` and that `<id>`
matches the `rule_id` in `caro-policy.json`. This is the test that proves the artifact
works in a real Rego engine without an `opa` binary.

**T3 — no over-blocking, ever.** For every corpus case the native `SafetyValidator`
allows, assert the rendered Rego also returns `allow`. `false_positives` must be empty;
a non-empty vector fails the test *and* `build` returns `Err`. This is the hard
invariant.

**T4 — determinism.** `build(...)` twice in one process and `render` to two temp dirs;
assert every rendered file is byte-identical and `bundle.digest` matches. Guards against
`HashMap` iteration order, timestamps, and absolute paths leaking into output.

**T5 — fidelity accounting is complete.** Assert
`exported + omitted == total_patterns`, `total_patterns == DANGEROUS_PATTERNS.len()`,
and that `by_class` sums to `exported`. Borrowed wholesale from ADR-055's
coverage-completeness invariant: the report may not silently drop a rule.

**T6 — shell-rewriting gap is declared, not hidden.** Take a corpus case the native
validator denies and the exported regex misses (e.g. a quoted/split rewriting of a
denied command). Assert exit code `17`, assert the case appears by id in
`false_negatives`, and assert its rule is classed `SemanticsRequired`. **This test
failing to fail is the bug**: it exists to prove Caro cannot ship a silent gap.

**T7 — unsupported ACS version fails closed.** `--acs-version 0.4.0` → exit `2`,
stderr names the supported version, and `--out` is empty afterwards.

### 3.6 Explicitly out of scope (next version)

- **Cedar export.** `cedar-policy 2.4.2` is in the lock and AGT documents Cedar
  alongside Rego. A second target multiplies the fidelity matrix; land Rego first.
- **`caro serve --validator`** (an Execlave `custom_validator` HTTPS shim). That is a
  *daemon*, which this ADR's constraints forbid. Separate ADR, separate argument.
- **Linking AGT at runtime.** No `agentmesh` call appears in this feature. Emitting a
  file that AGT reads is not an integration; `governance` stays Phase-0.
- **`transform` verdicts.** Permanently out, not deferred (D5).
- **`warn` / `require_approval`.** Blocked on the memo's §3.6 `caro.assessment.v1`
  decision contract, which is still unwritten. This version emits `allow`/`deny` and
  records the down-map so the schema needs no major version when 3.6 lands.
- **Publishing the bundle anywhere** — no registry, no OCI push, no `caro policy
  publish`. The user copies files.
- **Custom-pattern *round-trip*.** `custom_patterns` are exported; importing a foreign
  Rego bundle back into Caro is not in scope and may never be.
- **Fixing drifts D-a…D-d.** Each is a one-line docs PR and must not ride along inside a
  feature PR (`.claude/rules/good-boy-scout.md`: leave it better, don't gold-plate).

### 3.7 Demoware trap — what breaks at 100 real users

*Required by `.claude/rules/validation-discipline.md` Gate 3.*

**Assumption that holds at demo scale.** The demo exports 67 built-in patterns against a
curated corpus and shows a clean fidelity report. It assumes (i) the corpus is
representative of what the exported rules will meet in production, and (ii) users export
roughly the default pattern set.

**How it fails at 100 users.** Both assumptions break in the same direction. Real users
carry `[[safety.custom_patterns]]` — regexes Caro's authors never wrote, never
TDD-validated, and that the corpus has zero cases for. Those patterns will be exported
with a `FidelityClass` computed from a corpus that does not exercise them, producing a
**fidelity report that is confidently wrong**. The failure is worse than no report: a
user reads `false_negatives: []`, concludes their exported policy is at parity, and
installs it into AGT as their only gate. Rego's fail-open default (F4) then silently
allows everything their custom patterns were supposed to stop.

Secondary failure: `regorus` and Rust's `regex` crate are different engines. Any pattern
using a construct one supports and the other does not — or that behaves differently —
translates into a rule that behaves differently in the target engine than at home.
Backtracking constructs are the obvious class; there will be others.

**Instrumentation that tells us it is breaking.** The fidelity report already carries
`corpus_id` and `corpus_cases`. Add a per-rule `corpus_coverage: u32` — the number of
corpus cases that exercised that rule. Any rule with `corpus_coverage == 0` is
**unmeasured**, and unmeasured is not the same as safe.

**Fallback when it triggers.** A rule with `corpus_coverage == 0` is classed
`SemanticsRequired` — never `LiteralSafe` — regardless of how simple its regex looks,
and the export exits `17` with that rule named. Additionally, `build` cross-compiles
every pattern through `regorus` at export time and refuses any pattern the two engines
parse differently, listing it in `omitted`. **Fail closed on unmeasured, not open.**
Concretely: this makes `corpus_coverage` a required field in v1's schema, not a v2
addition — which is the whole reason the demoware section belongs in the scope and not
in the retro.

---

## Recommended next step

Do **not** open an implementation PR from this document. In order:

1. **Write the memo's §3.6 ADR (`caro.assessment.v1`) first.** The decision-vocabulary
   down-map in §3.2 (`DownMapEntry`) is a placeholder for a contract that does not exist
   yet. Exporting `allow`/`deny` before Caro has decided what its own decisions *are*
   bakes a guess into a published artifact. That ADR was the 08-20 memo's #2
   recommendation and is still unwritten; this is now the second ADR to block on it.
2. **Run the `devils-advocate` review on ADR-057.** The obvious attack: *"a fidelity
   report that says 'this export is 60% as good as Caro' is a document arguing against
   using the export — why ship it?"* That objection deserves a real answer in the PR, not
   a rebuttal here.
3. **Then the `regorus` direct-dependency spike** (≤100 LOC, five checks, feature flag
   off by default), per `.claude/rules/external-sdk-integration.md`.
4. **Then T3 and T6 before any renderer code.** The two invariants — never over-block,
   never hide a gap — are the feature. Write them failing first.

And independently of all of the above, one small thing that needs no ADR: **fix drift
D-a**. Caro has 67 dangerous-command patterns and has been telling the world it has 52.
Undercounting your own safety corpus by 22% in the README is a free win sitting on the
floor.

---

## Sources

Fetched and read 2026-08-24:

- [microsoft/agent-governance-toolkit — `docs/quickstart.md`](https://github.com/microsoft/agent-governance-toolkit/blob/main/docs/quickstart.md) (`last_reviewed: 2026-07-12`)
- [microsoft/agent-governance-toolkit — `docs/tutorials/55-agent-control-specification.md`](https://github.com/microsoft/agent-governance-toolkit/blob/main/docs/tutorials/55-agent-control-specification.md) (`last_reviewed: 2026-06-02`)
- [microsoft/agent-governance-toolkit — repository](https://github.com/microsoft/agent-governance-toolkit)
- [Governing MCP tool calls in .NET with the Agent Governance Toolkit — .NET Blog](https://devblogs.microsoft.com/dotnet/governing-mcp-tool-calls-in-dotnet-with-the-agent-governance-toolkit/)
- [Govern AI Agents on App Service with the Microsoft Agent Governance Toolkit — Microsoft Community Hub](https://techcommunity.microsoft.com/blog/appsonazureblog/govern-ai-agents-on-app-service-with-the-microsoft-agent-governance-toolkit/4510962)
- [Open Policy Agent — WebAssembly](https://www.openpolicyagent.org/docs/wasm)
- [Open Policy Agent — Bundles](https://openpolicyagent.org/docs/v0.40.0/management-bundles)

Referenced, not re-fetched this run:

- `.hermes/digests/2026-08-20-agent-market-scan.md` — §3.6, §3.7, §3.9, §4
- [GuardFall Exposes Open-Source AI Coding Agents — The Hacker News, 2026-06-30](https://thehackernews.com/2026/06/guardfall-exposes-open-source-ai-coding.html) (second-hand via the memo)
- Caro tree at `1.4.0`: `src/safety/`, `src/models/mod.rs`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`
