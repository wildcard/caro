# Implementation Scope — Landing `caro.assessment.v1` in `src/`

**Feature under analysis:** the **Claude Agent SDK permission-evaluation surface** —
the six-step order `hooks → deny rules → ask rules → permission mode → allow rules →
canUseTool`, the `canUseTool` / `PermissionResult` callback, the `critical paths` floor,
and the `CLAUDE_SDK_CAN_USE_TOOL_SHADOWED` warning — read live **2026-08-26** at
`https://code.claude.com/docs/en/agent-sdk/permissions`.

This is the *sibling* of the surface ADR-058 researched on 2026-08-25. That run read the
**hooks reference** and modelled the decision payload. This run reads the **permissions
reference**, which documents what the host actually *does* with that payload — and it
turns out the host does not do the same thing with an `allow` as it does with a `deny`.

**Equivalent we are scoping for Caro:** not a new contract. **The PR that puts
`caro.assessment.v1` in `src/`** — the module, the verb, the exit codes, the schema
artifact, and the one refactor without which the contract's headline field (`evidence`)
is literally unconstructible from Caro's current code.

**Date:** 2026-08-26 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-059-assessment-implementation-and-verb-namespace.md`

---

> ### Provenance note — and a deliberate departure from the task template
>
> This scheduled task's template leaves `[FEATURE NAME]` unbound and asks, every run, for
> a new feature scope and a new ADR. No user was present. Target selection was mine, and
> this run **departs from the template's default shape**. The reasoning, stated plainly so
> a human can overrule it:
>
> **1. Today's own strategy memo says stop scoping.** `market-scans/2026-08-26-ai-agent-strategy-memo.md`
> — generated this morning by the sibling scheduled task — closes with:
>
> > *"the gap between Caro's ADR backlog and its `src/` tree is now the single largest risk
> > in this memo. Twenty-four proposed governance ADRs in six weeks, zero modules. …
> > A useful forcing function would be a **moratorium on new ADRs in this space** until
> > `caro.assessment.v1` and one lifecycle emitter are merged."*
>
> Its **Build or test next (one thing)** is: *"`caro.assessment.v1` — the struct, the
> taxonomy split, `--json`, versioned. **Not the schema doc, not another ADR.**"*
>
> An autonomous run that answers that memo with ADR-059-in-the-same-space is the failure
> mode the memo names, executed by the process that caused it. So this run does the other
> thing: it researches a genuinely new external surface (Phase 1), uses it to find what is
> **wrong or missing in the existing plan** (Phase 2), and spends Phase 3 on a build plan
> rather than a design.
>
> **2. There is still a real ADR to write, and it is an implementation ADR.** ADR-058
> designed the payload and deliberately left open where the code lives, what the verb is
> called, whether it ships behind a feature flag, and how the schema artifact is produced.
> Those are decisions with consequences and alternatives; they belong in an ADR. ADR-059
> makes **six** of them and explicitly declares itself the last ADR in this space until
> code merges. It changes the ADR-058 contract in exactly **one** additive way (§2.3), and
> that change is forced by evidence gathered today, not by taste.
>
> **3. The build plan turned up a blocker nobody had noticed.** `Evidence::Blended` —
> the field ADR-058 says is *"the reason this ADR exists"* — cannot be constructed from
> Caro's current code. `blend_smart_decision` discards every input it would need
> (§2.4). That is a concrete finding, it is the reason this document exists rather than a
> ticket, and it is exactly what Phase 3 of the template asks for: *solve the specific
> failure mode you found in Phase 1 by design, not by workaround.*
>
> ### Verification status
>
> - **Verified by fetch (2026-08-26):** every quoted sentence and field name from the
>   Agent SDK permissions page, including the six-step order, the hook-`allow`
>   qualification, the `CLAUDE_SDK_CAN_USE_TOOL_SHADOWED` code and its two triggers, the
>   critical-path carve-out, the six permission modes, and the subagent-inheritance rule.
>   Quoted verbatim in §1.
> - **Verified against the Caro tree (2026-08-26, branch `integrator/20260711-postmerge`):**
>   every file path, line number, type, derive list, signature and absence claim in §2 and
>   §3 was produced by reading `src/`, `tests/` and `Cargo.toml`. Line numbers are given so
>   they can be re-checked and will drift.
> - **Second-hand:** the market-structure claims quoted from
>   `market-scans/2026-08-26-ai-agent-strategy-memo.md` are that document's, gathered from
>   Product Hunt launch pages, and its own caveats section disclaims them. Nothing
>   load-bearing here rests on a vendor claim.
> - **Not verified:** whether the SDK's TypeScript and Python permission surfaces agree
>   field-for-field. Only the shared prose reference was read. This matters only for the
>   projection table (§3.5) and is listed as a build-time check, not an assumption.
>
> ### Process compliance
>
> - `.claude/rules/git-workflow.md` — this file and the ADR are written **uncommitted**.
>   A human branches and PRs. Nothing was committed to `main` or to the current branch.
> - `.claude/rules/validation-discipline.md` — Gate 3 (demoware trap) is written at §3.8.
>   Gate 4 (`devils-advocate` review) is **required before the implementation PR opens**
>   and is not discharged here. Gates 1, 2 and 5 attach to the consuming features, not to
>   a serialization of decisions Caro already computes (ADR-058 made this argument; it
>   still holds, and §3.9 marks the one boundary case).
> - `.claude/rules/adr-numbering.md` — ADR-059 is the next sequential number; ADR-058 is
>   the highest present.

---

## Phase 1 — Feature Research

### 1.1 What problem it solves, and for whom

The buyer is the same as ADR-058's: someone who has decided the model cannot police itself
and needs an external answer to *should this tool call happen?* What is new is **who gets
the last word**.

The hooks reference — the document ADR-058 read — describes a payload. The permissions
reference describes an **evaluation order**, and the order is where the authority lives:

> **Hooks** — "Run hooks first. A hook can deny the call outright or pass it on. **A hook
> that returns `allow` does not skip the deny and ask rules below**; those are evaluated
> regardless of the hook result. A `PreToolUse` hook allow also can't approve an `rm` or
> `rmdir` removal targeting a critical path."
>
> **Deny rules** — "If a deny rule matches, the tool is blocked, **even in
> `bypassPermissions` mode**."
>
> **canUseTool callback** — "If not resolved by any of the above, call your `canUseTool`
> callback for a decision. **In `dontAsk` mode, this step is skipped and the tool is
> denied.**"

Read together with the hooks page's guarantee that a hook `deny` blocks even under
`--dangerously-skip-permissions`, the shape is:

**A safety layer's `deny` is authoritative. A safety layer's `allow` is advisory.**

That asymmetry is not in Caro's plan anywhere, and it is the single most consequential
thing this run found.

### 1.2 Core architecture — data flow, key types, separation of concerns

```
tool request
  → hooks              (external process / HTTP / in-process; deny is final, allow is not)
  → deny rules         (declarative; final)
  → ask rules          (declarative; routes to callback, even under bypassPermissions)
  → permission mode    (default | dontAsk | acceptEdits | bypassPermissions | plan | auto)
  → allow rules        (declarative; approves — and skips the callback entirely)
  → canUseTool         (in-process typed callback; only reached if nothing above resolved)
  → execute
```

Four separations worth naming:

1. **Two integration points with different coverage.** A hook runs on *every* call. The
   callback runs only on calls nothing else resolved. Same decision, different guarantee.
2. **The typed callback is a different serialization of the same verdict.** `PermissionResult`
   is `{behavior: "allow", updatedInput}` or `{behavior: "deny", message, interrupt}` — the
   same two-valued core as the hook's four-valued `permissionDecision`, with a *narrower*
   vocabulary at the point that runs *less often*. A projection table that assumes one
   vocabulary per host is already wrong: Claude Code has two.
3. **A deterministic floor exists and is host-side.** Critical paths cannot be approved by
   an allow rule, by `bypassPermissions`, or by a hook `allow` — only the modes that prompt,
   the `auto` classifier, or a `dontAsk` deny can dispose of them. This is Anthropic
   shipping a non-negotiable pattern list underneath a model classifier.
4. **The mode is mutable mid-session and inherited by subagents.**
   `setPermissionMode()` changes it live; subagents inherit it and
   `bypassPermissions | acceptEdits | auto` **cannot be overridden per subagent**.

### 1.3 Why it is limited — the failure modes

ADR-058 catalogued F1–F7 from the hooks page. The permissions page adds three, and one of
them is worse than anything on the earlier list.

- **F8 — Silent bypass, acknowledged by a warning code.** Quoted:
  > *"**Auto-approved tools never reach `canUseTool`.** A tool call approved at any earlier
  > step, by `acceptEdits` or `bypassPermissions`, or by an allow rule, skips your
  > `canUseTool` callback, so **permission checks you put there are silently bypassed for
  > that tool**."*

  The SDK emits a Node process warning, code `CLAUDE_SDK_CAN_USE_TOOL_SHADOWED`, on two
  configurations (`permissionMode: 'bypassPermissions'`, and each bare `allowedTools`
  entry). A warning code is what a vendor ships when a design cannot be fixed without a
  breaking change. The documented remedy is *"To gate every tool call regardless of mode
  and rules, use a `PreToolUse` hook instead."* — i.e. **use the other integration point.**

  For Caro this is decisive: **`caro guard` must be a hook, not a `canUseTool` callback**,
  and the payload must say which one produced it, because the two have different coverage
  guarantees and no way to tell them apart after the fact.

- **F9 — `allow` is not `allow`.** A hook's `allow` is overridable by three later steps.
  Nothing in the emitted payload records that the emitter's approval was advisory. Two
  layers both said "allow" and only one of them was binding; the audit log shows two
  identical strings.

- **F10 — Behaviour is version-pinned in prose.** Three separate version gates appear on
  one page: MCP `requiresUserInteraction` needs *"Claude Code v2.1.199 or later"*, the
  classifier routing for critical-path removals needs *"v2.1.218 or later"*, plan-mode
  shell-command routing needs *"v2.1.212 or later"*. The host's decision semantics change
  at point releases and are discoverable only by reading paragraphs. ADR-058's
  `target_vocabulary_version` on `Projection` anticipated this and is vindicated; what it
  does not capture is that the *behaviour*, not just the *vocabulary*, is what moved.

### 1.4 Output contract, exit codes, events

The SDK surface is in-process and typed, so there is no exit code. Its contract is:

| | Hook (`PreToolUse`) | Callback (`canUseTool`) |
|---|---|---|
| Transport | subprocess / HTTP / in-process | in-process only |
| Coverage | every call | only unresolved calls (F8) |
| Vocabulary | `allow`/`deny`/`ask`/`defer` | `allow`/`deny` |
| Deny authority | final, even under bypass | final for calls it sees |
| Allow authority | advisory (F9) | binding |
| Input rewrite | `updatedInput`, racy (ADR-058 F4) | `updatedInput`, single-writer |
| Failure default | fail-open for `command`/`http` | fail-closed |

The last row is the one Caro already knew (ADR-058 F2/F3) and the one that makes `fail_mode`
a payload field rather than an operator's problem.

### 1.5 Session / context lifecycle

There is none to reuse and that is the point. Permission evaluation is stateless per tool
call; the only session-scoped state is the mutable permission mode, which lives in the host,
not the decider. `setPermissionMode()` is a host call, not a hook return value.

This directly confirms ADR-058's D8 (`pure subprocess, no daemon, no state`) against the
richest available counter-example: even Anthropic's *in-process* integration point takes no
lifecycle obligation. Caro inherits zero initialization cost because there is nothing to
initialize — the assessment is a pure function of `(command, cwd, shell, config, optional
judgment)`. Redundant initialization is avoided by having no state to initialize, not by
caching.

---

## Phase 2 — Competitive Differentiation

### 2.1 What they get right that we should replicate

1. **The deterministic floor is host-side and non-negotiable.** Critical-path `rm`/`rmdir`
   cannot be approved by any rule, any mode, or any hook. Anthropic ships a pattern list
   beneath its classifier. Caro's `RiskLevel::Critical` hard floor in
   `blend_smart_decision` (`src/safety/mod.rs:290–293`) is the same construct, and Caro
   should say so out loud in the positioning doc the memo's opportunity (E) asks for. The
   argument *"a deterministic floor beneath a probabilistic gate"* is no longer Caro's
   contrarian position — it is the shipped architecture of the product Caro layers under.
2. **Evaluation order is published, numbered, and diagrammed.** Six steps, each named,
   with the override rules spelled out per step. Caro's equivalent — static match →
   safety level → approval mode → judge blend → routing — exists only as code and prose
   across three files. It should be one published table.
3. **Version-gating behaviour changes explicitly**, even if only in prose.
4. **A warning code for a design flaw you cannot fix** (`CLAUDE_SDK_CAN_USE_TOOL_SHADOWED`)
   is more honest than silence, and is a pattern worth copying for Caro's own known-lossy
   paths.

### 2.2 Their bugs and design gaps we avoid by designing the schema first

| Their gap | Caro's design answer |
|---|---|
| F8 silent bypass at the callback | Integrate at the hook, and record the integration point in the payload (`§3.5` `Projection.enforcement`) |
| F9 `allow` and `allow` are not the same allow | Same field |
| F10 behaviour moves at point releases | `target_vocabulary_version`, `omit-rather-than-guess` (ADR-058 D7) |
| F5 free-form `permissionDecisionReason` | `reason_code` namespaced on content-addressed `RuleId` (ADR-058 D6) |
| F7 no provenance anywhere | `evidence` required, non-optional (ADR-058 D3) — **but see §2.4** |
| Two vocabularies per host, one table | The projection table is keyed per *integration point*, not per *host* |

### 2.3 The one contract change this run recommends

**`Projection` gains `enforcement: Enforcement`, where `Enforcement` is
`Authoritative | Advisory`.**

Justification, in full, because a contract change on the day the memo says *ship it* needs
one:

- A Caro `block` projected to Claude Code's `deny` **is** authoritative — the hooks page
  guarantees it survives `--dangerously-skip-permissions`.
- A Caro `auto_approve` projected to `allow` **is not** — three later steps can still deny
  it, and the permissions page says so in the same sentence that introduces hooks.
- ADR-058's `lossy: bool` does not cover this. `lossy` says *the value changed shape*.
  `enforcement` says *the value may not be honoured*. A projection can be lossless and
  advisory at the same time — `block → deny` is lossless and authoritative; `auto_approve
  → allow` is lossless and advisory. One flag cannot carry both.
- The failure it prevents is concrete and expensive: an integrator reads
  `projections[].value == "allow"`, writes it to an audit log as *"Caro approved"*, and
  ships a compliance artifact asserting a decision Caro never had the power to make.

Cost: one two-variant enum, one field, one line per projection-table entry, one test.
This is the **only** change to ADR-058's contract proposed here, and ADR-059 owns it.

### 2.4 The blocker: `Evidence::Blended` is not constructible today

This is the finding that turns a ticket into a scope document.

ADR-058 D3 requires `Evidence::Blended` to carry the `JudgeAttribution` (backend, model,
confidence, threshold), **the deterministic decision and risk the judge overrode**, and a
`BlendDirection` of `Relaxed` or `Escalated`. ADR-058 calls this *"the F7 fix and the
reason this ADR exists."*

Caro's blend function, verified at `src/safety/mod.rs:277–330`, is:

```rust
pub fn blend_smart_decision(
    static_risk: RiskLevel,
    judge: Option<&RiskJudgment>,
    safety: SafetyLevel,
    auto_confirm: bool,
) -> SmartDecision
```

```rust
pub struct SmartDecision {          // src/safety/mod.rs:256–263
    pub requires_confirmation: bool,
    pub blocked: bool,
    pub note: Option<String>,
}
```

Five things `Evidence::Blended` needs and `SmartDecision` does not carry:

| Needed | Present? | Where it goes today |
|---|---|---|
| deterministic `SuggestedRouting` before the blend | **no** | never computed — the function works in `bool` pairs, not routings |
| deterministic `RiskLevel` before the blend | **no** | passed *in* as `static_risk`, dropped on return |
| judged `RiskLevel` | **no** | read from `judge`, dropped on return |
| direction (`Relaxed` / `Escalated`) | **prose only** | encoded in the wording of `note`: `"smart: relaxed to …"` (`:306–312`) vs `"smart: flagged as …"` (`:324–327`) |
| judge backend / model | **no** | never enters the function |

So the current state of affairs is precisely the one ADR-058 rejected as alternative **A3**
— *"emit provenance as a free-text `note`"* — and the implementation is structurally unable
to do anything else. **`Evidence` cannot be populated without changing this function.**
Any PR that ships the `Assessment` struct without touching `blend_smart_decision` will
either hard-code `Evidence::Deterministic` (a lie under `--approval smart`) or regex the
note (A3 with extra steps).

The fix is small and additive (§3.4), but it must be in the same PR, and nobody had
noticed it, because ADR-058 was written against the payload and not against the call site.

### 2.5 The second blocker: `caro assess` is already taken

ADR-058 D8 introduces *"one new verb, `caro assess`."* The name is occupied — by an
unrelated meaning, in three places:

- `src/assessment/` — a **hardware** assessment module (`cpu.rs`, `gpu.rs`, `memory.rs`,
  `profile.rs`, `recommender.rs`, `result.rs`), exporting `AssessmentResult`,
  `AssessmentError`, `SystemProfile`, `ModelRecommendation`.
- `src/main.rs:422–434` — a commented-out `Commands::Assess` variant, documented as
  *"Assess system resources and get model recommendations"*, with the note *"disabled in
  v1.1.0-beta.1 … will be implemented in a future release."*
- `tests/assess_integration_test.rs` — two `#[ignore]`d tests that shell out to
  `caro assess` and assert **hardware** output, plus `tests/assessment_tests.rs`.

Shipping `caro assess <command>` for command safety while `src/assessment/` means system
profiling reuses one name for two semantics — the exact **F1** failure ADR-058's **D1**
exists to prevent, committed at the verb layer rather than the field layer, by the ADR that
named the failure. And the two `#[ignore]`d tests would go from "dormant" to "asserting the
wrong thing about a live command," which is worse than either.

ADR-059 **D2** disposes of this.

### 2.6 What we can do that they cannot

Unchanged from ADR-058 and reinforced by today's page:

- **The assessed subject is a shell command string**, not `tool_name` + `args`. Every
  example on the permissions page gates at the tool granularity — `Bash(rm *)`,
  `Edit(//secrets/**)` — i.e. **glob patterns over command text**, hand-maintained by the
  user in `settings.json`. That is Caro's 67 TDD-validated patterns, done manually, per
  project, with no risk taxonomy and no test suite. The gap is not conceptual; it is that
  Anthropic ships the mechanism and expects the user to supply the content.
- **Offline, reproducible, zero-latency, prompt-injection-proof.** `auto` mode is a model
  classifier; Caro's floor is not.
- **Agent-agnostic.** The projection table exists because Caro is not one host's component.
- **Standalone binary, no daemon.** Confirmed as the right shape by §1.5.

### 2.7 Existing Caro infrastructure that already covers part of this

Everything below exists and is reused rather than rebuilt. This is the "do not duplicate"
constraint, discharged concretely.

| Need | Already in tree | Location |
|---|---|---|
| Risk taxonomy | `RiskLevel {Safe, Moderate, High, Critical}`, derives `Serialize, Deserialize, JsonSchema, PartialOrd, Ord` | `src/models/mod.rs:148–158` (enum at `:152`) |
| Decision taxonomy | `SuggestedRouting {AutoApprove, AsyncLog, HumanGate, Block}`, `#[serde(rename_all="snake_case")]` at `:188` | `src/models/mod.rs:187–195` (enum at `:189`) |
| Risk × safety matrix | `SuggestedRouting::from_risk_and_safety` | `src/models/mod.rs:198–210` |
| Structured decision | `SafetyDecision` (`Serialize + Deserialize`) + `from_validation_result` | `src/safety/mod.rs:189–234` (doc from `:184`) |
| Pattern engine | `SafetyValidator`, pre-compiled regex, **67** `DangerPattern` literals + CVE rules | `src/safety/patterns.rs`, `src/safety/cve_patterns.rs` |
| Bounded judge + hard floor | `blend_smart_decision`, `SMART_JUDGE_MIN_CONFIDENCE = 0.7` | `src/safety/mod.rs:252`, `277` |
| Judge attribution source | `BackendInfo { backend_type, model_name, version, … }` via `CommandGenerator::backend_info()` | `src/backends/mod.rs:79, 86–95` |
| JSON output plumbing | `CliResult`, `OutputFormat::Json` | `src/cli/mod.rs:76–107` |
| Schema emission | `schemars 0.8` as a direct dep + a working `generate-schema` binary | `Cargo.toml:46`, `src/bin/generate-schema.rs` |
| Config / safety level | `SafetyConfig`, `SafetyLevel`, `ApprovalMode` | `src/safety/mod.rs:165`, `src/models/mod.rs:286–300` |

**Dependency delta: zero.** `serde`, `serde_json`, `schemars 0.8` are already direct
dependencies, so `.claude/rules/external-sdk-integration.md` does not attach and no build
spike is required.

---

## Phase 3 — Scope Definition

### 3.1 What this PR is

One PR. It makes `caro.assessment.v1` real: the types, the verb, the exit codes, the
schema artifact, and the blend refactor that makes `evidence` truthful. It changes **no
runtime behaviour** on any existing code path.

Companion ADR: `docs/adr/ADR-059-assessment-implementation-and-verb-namespace.md`, which
records six decisions (verb namespace, no feature flag, blend refactor shape, schema
artifact, `Enforcement` field, version-in-payload) with alternatives and consequences.

### 3.2 New types

All in **one new file**, `src/safety/assessment.rs`. Every type derives
`Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq` unless noted.
Every enum is `#[serde(rename_all = "snake_case")]`, tag style `#[serde(tag = "kind")]`
for the two sum types, so a consumer branches on a string rather than an object shape.

```rust
/// The one payload. `schema` serializes first and is a literal.
pub struct Assessment {
    pub schema: SchemaId,                  // always "caro.assessment.v1"
    pub caro_version: String,              // env!("CARGO_PKG_VERSION") — see D6
    pub subject: Subject,
    pub risk: RiskLevel,                   // what this command IS
    pub decision: SuggestedRouting,        // what the host SHOULD DO
    pub reason_code: String,               // "caro.<risk>.<rule_id>"
    pub message: String,
    pub message_truncated: bool,
    pub matched_patterns: Vec<String>,
    pub confidence: f64,
    pub evidence: Evidence,                // REQUIRED. never Option.
    pub fail_mode: FailMode,
    pub suggested_alternative: Option<String>,
    pub projections: Vec<Projection>,
}

pub struct Subject {
    pub command: String,
    pub cwd: Option<String>,
    pub shell: ShellType,                  // reuse crate::platform
    pub safety_level: SafetyLevel,         // reuse crate::models
    pub approval_mode: ApprovalMode,       // reuse crate::models
}

#[serde(tag = "kind")]
pub enum Evidence {
    Deterministic { pattern_count: usize },
    ModelConsultedNoEffect { judge: JudgeAttribution, reason: NoEffectReason },
    Blended {
        judge: JudgeAttribution,
        direction: BlendDirection,         // Relaxed | Escalated
        from_risk: RiskLevel,              // deterministic risk the judge overrode
        from_decision: SuggestedRouting,   // deterministic decision it overrode
    },
}

pub enum NoEffectReason { NoVerdict, BelowConfidenceThreshold, CriticalFloor, VerdictAgreed }

pub struct JudgeAttribution {
    pub backend: String,                   // BackendInfo.backend_type
    pub model: String,                     // BackendInfo.model_name
    pub confidence: f64,
    pub threshold: f64,                    // SMART_JUDGE_MIN_CONFIDENCE
}

pub enum BlendDirection { Relaxed, Escalated }
pub enum FailMode { Normal, FailClosed }

pub struct Projection {
    pub host: String,                      // "claude-code"
    pub integration_point: String,         // "pre_tool_use_hook" | "can_use_tool"
    pub field: String,                     // "hookSpecificOutput.permissionDecision"
    pub value: String,                     // "deny"
    pub lossy: bool,
    pub enforcement: Enforcement,          // NEW — §2.3
    pub note: Option<String>,
    pub target_vocabulary_version: String,
}

pub enum Enforcement { Authoritative, Advisory }

#[repr(i32)]
pub enum ExitCode { Ok = 0, AsyncLog = 18, HumanGate = 19, Block = 20, Usage = 2, Internal = 1 }
```

**Method contracts** (the only public API this PR adds):

| Signature | Contract |
|---|---|
| `Assessment::assess_or_block(subject: Subject, validator: &SafetyValidator, blend: Option<&SmartDecision>) -> Assessment` | **Infallible by signature.** Never returns `Result`. Any internal failure produces `decision: Block`, `fail_mode: FailClosed`, a namespaced `reason_code`, and a well-formed payload. Silence is unrepresentable. |
| `Assessment::exit_code(&self) -> ExitCode` | Total function of `self.decision` and `self.fail_mode`. The **only** place a code is derived, so payload and code cannot disagree. |
| `Assessment::project(&self, host: &str) -> Option<&Projection>` | Returns `None` for unknown hosts. Never guesses. |
| `Assessment::to_json(&self) -> String` | Single-line JSON, `schema` first, no trailing newline ambiguity — exactly one `\n` written by the caller. |
| `SchemaId::CURRENT` | `&'static str = "caro.assessment.v1"` |

Serialization rules, asserted by test, not by convention: field order is declaration order;
all enums snake_case; `f64` fields serialize with at most 3 decimal places via a custom
serializer so `0.7000000000000001` never appears in a golden file.

### 3.3 Minimal set of files that change

**One new source file. Six touched. One new test file. No new module.**

| # | File | Change |
|---|---|---|
| 1 | `src/safety/assessment.rs` | **NEW** — everything in §3.2 |
| 2 | `src/safety/mod.rs` | `pub mod assessment;` + `pub use assessment::{Assessment, Evidence, ExitCode, …};`; extend `SmartDecision` (§3.4); extend `blend_smart_decision` signature (§3.4) |
| 3 | `src/models/mod.rs` | `SuggestedRouting` gains `JsonSchema, PartialOrd, Ord` (declaration order = restrictiveness order, so `max()` is the precedence rule); `RiskLevel` `rename_all` `"lowercase"` → `"snake_case"` (byte-identical output — every variant is one word — with a test that asserts it) |
| 4 | `src/cli/mod.rs` | Call site at `:822–829`: build `JudgeAttribution` from `self.backend.backend_info()`, pass it in, destructure the new `evidence` field. **No behaviour change** — `requires_confirmation` / `blocked_reason` / `note` keep their current values |
| 5 | `src/main.rs` | Add `Commands::Assess { command, json, schema }` per ADR-059 D2; **delete** the dead commented block at `:422–434`; exactly one `process::exit(assessment.exit_code() as i32)` site |
| 6 | `src/bin/generate-schema.rs` | Also emit `schemas/caro.assessment.v1.schema.json` (committed to the repo) |
| 7 | `tests/assessment_contract.rs` | **NEW** — §3.6 |
| 8 | `tests/assess_integration_test.rs` | Retarget the two `#[ignore]`d hardware-`assess` tests per ADR-059 D2, or delete them. They must not survive as-is |

Explicitly **not** touched: `src/agent/`, `src/execution/`, `src/governance/`,
`src/telemetry/`, `src/backends/` (the `BackendInfo` it already exposes is enough), and the
~40 ad-hoc `process::exit` sites in `main.rs` (boy-scout rule: leave better, do not
gold-plate — the registry governs the new verb only).

### 3.4 The blend refactor (solves §2.4 by design, not workaround)

```rust
// src/safety/mod.rs — additive; the three existing fields keep their meaning and values.
pub struct SmartDecision {
    pub requires_confirmation: bool,
    pub blocked: bool,
    pub note: Option<String>,
    pub evidence: Evidence,          // NEW — the structured form of `note`
    pub decision: SuggestedRouting,  // NEW — the post-blend routing, computed once
}

pub fn blend_smart_decision(
    static_risk: RiskLevel,
    judge: Option<&RiskJudgment>,
    safety: SafetyLevel,
    auto_confirm: bool,
    attribution: Option<JudgeAttribution>,   // NEW — 5th param
) -> SmartDecision
```

Rules, one per existing branch, so every early return acquires an `Evidence` and none can
be forgotten:

| Existing branch (`src/safety/mod.rs`) | `Evidence` emitted |
|---|---|
| `static_risk == Critical` → `unchanged` (`:291–294`) | `ModelConsultedNoEffect { reason: CriticalFloor }` if a judge was present, else `Deterministic` |
| no judge / `confidence < 0.7` → `unchanged` (`:296–300`) | `ModelConsultedNoEffect { reason: NoVerdict \| BelowConfidenceThreshold }`, or `Deterministic` when `judge` is `None` and mode is not `Smart` |
| `judgment.risk <= static_risk`, decision actually moved (`:302–317`) | `Blended { direction: Relaxed, from_risk: static_risk, from_decision: <pre-blend routing> }` |
| `judgment.risk <= static_risk`, decision did **not** move (the `note: None` case at `:307`) | `ModelConsultedNoEffect { reason: VerdictAgreed }` — information the current code computes and silently discards |
| `judgment.risk > static_risk` → escalate (`:318–329`) | `Blended { direction: Escalated, … }` |

`from_decision` is `SuggestedRouting::from_risk_and_safety(static_risk, safety)` — computed
inside the function from arguments it already receives. No new inputs, no new call graph.
`attribution` is the only genuinely new input and is available at the one call site from
`backend_info()`.

`note` is retained verbatim so the human-facing string and the existing tests do not move.
A test asserts `note.is_some() == matches!(evidence, Evidence::Blended{..})` — closing the
prose/structure drift permanently.

### 3.5 Exit code and output contract

What machines and scripts depend on. This table is the contract.

| Exit | Condition | stdout | stderr |
|---|---|---|---|
| `0` | `decision: auto_approve` | one `Assessment`, one `\n` | empty |
| `18` | `decision: async_log` — permitted, host **must** record | one `Assessment` | empty |
| `19` | `decision: human_gate` | one `Assessment` | empty |
| `20` | `decision: block` | one `Assessment` | empty |
| `2` | usage error (missing arg, bad flag) | one `Assessment`, `fail_mode: fail_closed`, `decision: block` | one line, human-readable |
| `1` | internal error (panic caught, IO failure) | one `Assessment`, `fail_mode: fail_closed`, `decision: block` | one line |
| `0` | `caro assess --schema` | the JSON Schema, pretty-printed | empty |

Guarantees a script may rely on:

1. **`[ $? -ge 19 ]` is a valid "needs a human or worse" gate** without parsing JSON —
   codes are monotonic in restrictiveness.
2. **stdout is always exactly one JSON object followed by exactly one newline**, on every
   exit code including `1` and `2`. A consumer never has to handle an empty stdout.
3. **stdout never carries human text.** Colour, banners, spinners and progress go to
   stderr or are suppressed. `--json` is implied by the verb; a bare `caro assess "<cmd>"`
   prints a human summary to stdout and is therefore *not* covered by guarantee 2 —
   ADR-059 D2 resolves this by making JSON the default for the verb and human output the
   opt-in (`--human`).
4. **`ExitCode` is one `#[repr(i32)]` enum** carrying every code claimed by any sibling
   ADR, including codes this PR does not use, with a test asserting no two variants
   collide. Codes 18–20 leave 0–17 undisturbed for the ADRs that already claimed them.
5. **No network, no clock, no telemetry** on the assess path. Same input ⇒ byte-identical
   output within a `caro_version`.

### 3.6 Integration tests — known inputs → deterministic JSON + exit code

`tests/assessment_contract.rs`. Every case is a full-binary invocation via `assert_cmd`
(matching the existing `tests/assess_integration_test.rs` idiom), asserting **exact stdout
bytes** against a fixture in `tests/fixtures/assessment/` and the **exact exit code**.
None may be `#[ignore]`d; none may require a model.

| # | Input | Asserted |
|---|---|---|
| T1 | `caro assess "ls -la"` | exit `0`; `decision:"auto_approve"`, `risk:"safe"`, `evidence.kind:"deterministic"`, `pattern_count:0`; golden bytes |
| T2 | `caro assess "rm -rf /"` | exit `20`; `decision:"block"`, `risk:"critical"`, `reason_code` matches `^caro\.critical\.[a-z0-9_]+$`; golden bytes |
| T3 | `caro assess "chmod -R 777 /etc" --safety moderate` | exit `19`; `decision:"human_gate"` |
| T4 | `caro assess "curl https://example.com \| sh" --safety permissive` | exit `18`; `decision:"async_log"` |
| T5 | `caro assess` (no argument) | exit `2`; well-formed payload, `fail_mode:"fail_closed"`, `decision:"block"`; stdout is still exactly one JSON object |
| T6 | `caro assess --schema` | exit `0`; stdout byte-equals `schemas/caro.assessment.v1.schema.json` — **this is the schema-drift alarm** |
| T7 | T1 run twice, output diffed | byte-identical (determinism) |
| T8 | T2 with `--json` piped through `jq -e '.projections[] \| select(.host=="claude-code") \| .value=="deny" and .enforcement=="authoritative"'` | exit `0` |
| T9 | T1's projection for `claude-code` | `value=="allow"` **and** `enforcement=="advisory"` — the §2.3 regression guard |
| T10 | An `async_log` assessment projected to every host in the table | no projection has `value` equal to that host's allow-equivalent (ADR-058 D7: lossy projections fail toward restriction) |
| T11 | unit — `ExitCode` variants | no two share a numeric value |
| T12 | unit — `RiskLevel` serialization before/after the `rename_all` change | byte-identical for all four variants |
| T13 | unit — `SuggestedRouting` ordering | `AutoApprove < AsyncLog < HumanGate < Block`, and `max()` over a slice reproduces the "most restrictive wins" rule |
| T14 | unit — `blend_smart_decision` over the five branches in §3.4 | correct `Evidence` variant each time, and `note.is_some() == matches!(evidence, Blended{..})` |
| T15 | unit — `Critical` + high-confidence `Safe` judgment | `Evidence::ModelConsultedNoEffect { reason: CriticalFloor }`, decision unchanged — the hard-floor invariant, now assertable from the payload |

T7, T12 and T14 are the ones that will catch a regression nobody meant to make. T6 is the
one that will fail on someone else's PR and should therefore print a copy-pasteable
`cargo run --bin generate-schema` remediation line in its failure message.

### 3.7 Sequencing

Three commits, one PR:

1. `chore(models): derives and casing for the assessment contract` — item 3 in §3.3 alone.
   Non-breaking on the wire; T12/T13 prove it.
2. `refactor(safety): structured provenance from blend_smart_decision` — items 2 and 4.
   No behaviour change; T14/T15 prove it.
3. `feat(safety): caro.assessment.v1 — assess verb, payload, exit codes` — items 1, 5, 6,
   7, 8.

Splitting into three PRs is worse: (1) and (2) are unreviewable in isolation because
nothing consumes them, and a reviewer cannot tell whether the derives are correct without
seeing the payload. Three commits in one PR gives a bisectable history and a reviewable
diff.

### 3.8 What breaks at 100 real users (validation-discipline Gate 3)

**The assumption that holds at demo scale:** the projection table is small enough to keep
correct by hand, and each host has one vocabulary.

**Why it fails.** §1.2 already shows Claude Code has two integration points with two
vocabularies and three separate point-release version gates on one documentation page. At
100 integrators across four hosts, the table is 8+ entries, each pinned to a host version
that moves without notice, and a stale entry projects a string the host no longer accepts —
which most hosts treat as a **non-blocking** error. **A stale projection degrades to
allow-everything.** That is the worst possible failure direction and it arrives silently.

**Instrumentation that would tell us.** T6's schema-drift test catches *our* changes, not
theirs. The detector for theirs is a scheduled CI job that fetches each host's documented
vocabulary and diffs it against `target_vocabulary_version`. **It is out of scope for this
PR** (§3.9) and is the first item of the next one; until it exists, the table is manually
audited at each release as part of the 6-file checklist.

**Fallback if it triggers.** ADR-058 D7's omit-rather-than-guess rule is the containment:
an unknown or unversioned host vocabulary is **absent** from `projections[]`, and an
integrator reading `project("claude-code") == None` gets a loud failure instead of a wrong
string. The residual risk is a projection that is present, well-formed, and *stale* — for
which the only defence is the audit above. This is stated as a known gap, not a solved
problem.

**Second assumption:** that `caro assess` is called per command and stays cheap. It is a
pure function with no network and no model on the default path (`ApprovalMode::Prompt`),
so it is regex-bound. Under `--approval smart` it makes a model call and the latency is the
backend's; a host calling it in a loop under `smart` will notice. Not solved here; the
mitigation is that `Smart` is non-default and stays non-default in v1 (ADR-058).

### 3.9 Explicitly out of scope

Belongs in the next version, listed so nobody adds it to this PR:

- **All transports.** MCP `assess_command` tool (ADR-015/038), `caro guard` hook adapter
  (ADR-036), MCP gateway proxy (ADR-043). Each has its own failure surface; shipping one
  inside the contract PR gives the contract that transport's bugs. §1.3's F8 finding —
  *integrate at the hook, not the callback* — is an input to ADR-036, recorded there, not
  acted on here.
- **The scheduled projection-drift CI job** (§3.8). First item of the next PR.
- **Wiring `decision` into the execution path.** `src/cli/mod.rs` continues to branch on
  `requires_confirmation` / `blocked_reason`. Making the decision *authoritative* rather
  than *sayable* is ADR-020's scope and its own argument.
- **`caro scan`** (ADR-023) and any CI-oriented multi-command mode.
- **The target axis** (ADR-047) — `Subject.cwd` is carried and nothing is computed from it.
- **Lifecycle events** (ADR-037/041). The `Assessment` is the payload an emitter would
  emit; the emitter is the next build item and the memo's opportunity (D).
- **Renaming `src/assessment/`** (hardware) or reviving hardware assessment under a new
  verb. ADR-059 D2 reassigns the name and files a bead; the rename is not done here.
- **Reformulation-grade denial** (ADR-052), **evidence packets** (ADR-049), **session
  chain** (ADR-053). All consume this payload; none extends it.
- **Any further ADR in this space.** ADR-059 declares itself the last until code merges.

### 3.10 Definition of done

- [ ] `cargo test` green; `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean
- [ ] T1–T15 pass, none `#[ignore]`d, none requiring a model or a network
- [ ] `schemas/caro.assessment.v1.schema.json` committed and byte-equal to `--schema` output
- [ ] `tests/assess_integration_test.rs` no longer asserts hardware output from `caro assess`
- [ ] `devils-advocate` review posted as a `## Devil's Advocate Review` comment on the PR,
      with each objection either addressed or recorded as a deliberate decision
      (`.claude/rules/validation-discipline.md` Gate 4)
- [ ] ADR-058's status changes `Proposed` → `Accepted`; ADR-059 lands `Accepted`
- [ ] ADR-015's dangling `SafetyAssessmentOutput` reference corrected in a separate docs PR
      so ADR-020 and ADR-023 stop blocking on a name that never existed
- [ ] A one-line note added to the next weekly memo: the moratorium's first condition is met
- [ ] Follow-up filed (not done here): `docs/adr/README.md`'s index table is current only
      through ADR-015 — ADR-016 → ADR-059 exist as files and are unindexed. A staleness note
      was added in place of 43 hand-written rows; the backfill is its own small PR

---

## References

**Fetched 2026-08-26**
- [Claude Agent SDK — Configure permissions](https://code.claude.com/docs/en/agent-sdk/permissions) — six-step order, `canUseTool`, `CLAUDE_SDK_CAN_USE_TOOL_SHADOWED`, critical paths, permission modes, subagent inheritance
- [Handling Permissions — Claude Docs](https://docs.claude.com/en/docs/agent-sdk/permissions) (redirects to the above)

**Prior research, not re-fetched this run**
- [Claude Code — Hooks reference](https://code.claude.com/docs/en/hooks) · [Hooks guide](https://code.claude.com/docs/en/hooks-guide) (fetched 2026-08-25, via ADR-058)
- [OpenAI Agents SDK — Human in the loop](https://openai.github.io/openai-agents-python/human_in_the_loop/) (via ADR-058)
- [MCP 2025-11-25 — Tools / `ToolAnnotations`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) (via ADR-058)

**Caro internal**
- `docs/adr/ADR-058-caro-assessment-v1-decision-contract.md` — the contract this PR implements
- `caro-scope-assessment-contract-2026-08-25.md` — the contract's full scope, type definitions, F1–F7
- `specs/017-structured-assessment-payload/scope.md` (2026-06-03) — the original payload scope
- `market-scans/2026-08-26-ai-agent-strategy-memo.md` §3A, §4 — the "stop scoping, ship it" recommendation and the moratorium proposal
- `.claude/rules/validation-discipline.md`, `.claude/rules/git-workflow.md`, `.claude/rules/good-boy-scout.md`, `.claude/rules/adr-numbering.md`
- Tree at `1.4.0`, branch `integrator/20260711-postmerge` (line numbers re-verified
  2026-08-26 and expected to drift): `src/models/mod.rs:152,189,198,288`,
  `src/safety/mod.rs:189,252,256,277,292,297,302,318`, `src/cli/mod.rs:76,795,811,822`,
  `src/backends/mod.rs:56,71,79,86`, `src/main.rs:422`, `src/assessment/mod.rs`,
  `src/bin/generate-schema.rs`, `tests/assess_integration_test.rs`, `Cargo.toml:41,42,46`

---

**Quick Actions:**
  `y` = yes | `c` = continue | `ta` = try again | `n` = next | `rp` = recommended plan

**💡 Recommended:** Branch (`bin/sk-new-feature "caro.assessment.v1 implementation"`), then
land §3.7's three commits as one PR. Run the `devils-advocate` review before opening it.
