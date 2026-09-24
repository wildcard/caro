# ADR-058: `caro.assessment.v1` — One Decision Contract, With Provenance, Projected Into Every Host Vocabulary

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-25
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: the **`PreToolUse` hook decision contract in Claude Code**
  (`hookSpecificOutput.permissionDecision` ∈ `allow | deny | ask | defer`), read live
  2026-08-25 at [code.claude.com/docs/en/hooks](https://code.claude.com/docs/en/hooks) and
  [/hooks-guide](https://code.claude.com/docs/en/hooks-guide); compared against the
  **OpenAI Agents SDK** approval model and **MCP elicitation** (`accept | decline | cancel`)
  plus `ToolAnnotations` at MCP spec revision **2025-11-25**
- **Supersedes**: the name `SafetyAssessmentOutput` (ADR-015), which has never existed in
  the tree despite ADR-020 and ADR-023 being told to block on it
- **Depends on**: nothing. No new dependency, no new module, no upstream ADR
- **Unblocks**: ADR-020 (tiered approval — this is what an approver receives),
  ADR-023 (`caro scan`), ADR-055 and ADR-057 §3.6 (both deferred their decision down-map to
  "the `caro.assessment.v1` contract, which is still unwritten")
- **Relates to**: ADR-015/038 (MCP validator tool — a transport for this payload),
  ADR-036 (`caro guard` hook adapter — a transport), ADR-043 (MCP gateway proxy — a
  transport), ADR-041 (lifecycle events — a sink), ADR-047 (trusted targets — the target
  axis, deliberately out of scope here), ADR-024 (headless JSON contract — exit-code band)
- **Full scope document**: `caro-scope-assessment-contract-2026-08-25.md`

> **Provenance note (autonomous run).** Produced by the `caro-research--scoping-process`
> scheduled task, whose template left `[FEATURE NAME]` unbound and which ran with no user
> present. Target selection rationale: `.hermes/digests/2026-08-25-weekly-agent-market-scan.md`
> §5 carries the decision taxonomy forward unaddressed for the second consecutive run and
> marks it as blocking opportunity 3.3; ADR-057's *Recommended next step #1* is *"Write the
> memo's §3.6 ADR (`caro.assessment.v1`) first"*; and a duplication sweep of all 57 ADRs
> finds the **transport** for an assessment covered three times (ADR-015, ADR-038, ADR-043)
> and the **payload** covered zero times. Treat the decision to reuse `SuggestedRouting`
> rather than define a new decision enum (D2) as a reviewable assumption.
>
> **Validation-discipline note.** This is a serialization contract over decisions Caro
> already computes, not a new capability class, so Gate 1 (20 transcripts) attaches to the
> features that *consume* it (scan opportunities 3.3, 3.4) rather than to the schema. Gate 3
> (demoware trap) is written — scope §3.7. Gate 4 (`devils-advocate` review) is required
> before any implementation PR opens. The `caro assess` verb (D8) is the boundary case and
> is argued below.

---

## Context

### The problem, stated once

Every agent host has independently invented a channel for one question — *should this tool
call happen?* — and the three shipping designs do not interoperate:

| System | Vocabulary | Notes |
|---|---|---|
| Claude Code `PreToolUse` | `allow` / `deny` / `ask` / `defer` | totally ordered by restrictiveness; `defer` only in non-interactive `-p` mode |
| OpenAI Agents SDK | approve / reject | binary; pause-and-resume via serializable `RunState` |
| MCP elicitation | `accept` / `decline` / `cancel` | `decline` vs `cancel` encodes *user intent*, not policy |

A search for a cross-vendor standard published in August 2026 found none. CISA's agentic-AI
guidance and CoSAI's ID-JAG token exchange govern **identity and credentials** at trust
boundaries, not decisions about individual calls. MCP's `ToolAnnotations` look like a fourth
vocabulary and are explicitly disclaimed by the spec: *"clients **MUST** consider tool
annotations to be untrusted unless they come from trusted servers."*

Caro must speak into all of them and has no serialized decision at all.

### What is actually missing in Caro (verified against the tree at `1.4.0`, 2026-08-25)

The 08-25 market scan's standing complaint — *"`CRITICAL/HIGH/MEDIUM/LOW` is a risk
taxonomy where the market has converged on a decision taxonomy"* — is **half wrong in
Caro's favour**. The decision taxonomy exists:

```rust
// src/models/mod.rs:189
pub enum SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }
```

Four values, mapping 1:1 onto Claude Code's four, in the same restrictiveness order, with
the complete risk×safety matrix already written at `from_risk_and_safety`
(`src/models/mod.rs:198`) and a `SafetyDecision` struct already wrapping it
(`src/safety/mod.rs:189`, `Serialize + Deserialize`).

What is missing is any path from it to a consumer. `grep -rn "SafetyDecision" src/` returns
matches **only inside `src/safety/mod.rs`**: never constructed by a caller, never returned by
a command, never serialized. It is a dead type of exactly the kind ADR-020 diagnosed when it
called `HumanGate` "a dead enum variant" — the variant has since acquired a dead struct
around it. And `SafetyAssessmentOutput`, the name ADR-015 told ADR-020 and ADR-023 to block
on, returns **nothing** from `grep -rn ... src/`; `src/cli/scan.rs` does not exist either.

### The failure mode this ADR exists to solve

None of the three researched contracts has a **provenance** field. A verdict of `deny` looks
identical whether a regex, a policy engine, or a language model produced it.

Caro already ships that ambiguity. `blend_smart_decision` (`src/safety/mod.rs:277`) merges an
optional LLM `RiskJudgment` into the deterministic decision under two documented invariants
(`:265–276`) — a `Critical` static match is never relaxed, and a missing or low-confidence
verdict leaves the static decision untouched. The only trace it leaves is
`SmartDecision.note: Option<String>`, built at `:306–312` as
`format!("smart: relaxed to {:?} — {}", …)`. A consumer of a decision made under
`ApprovalMode::Smart` (`src/models/mod.rs:288`) cannot tell from any structured field whether
a model relaxed the verdict, escalated it, or was absent. It can only regex prose.

The 08-25 scan reports three published failures of model-side safety in a single week and
concludes *"LLM-as-safety-judge is now empirically discredited — and still shipping."* Caro's
stated differentiation is that it does not make that bet. Publishing an assessment payload in
which the model's contribution is invisible would make that differentiation unverifiable by
the only audience that matters — the buyer reading the JSON.

### Failure modes observed in the researched contracts

- **F1** — `decision` is *deprecated* at `PreToolUse`, *current* at `PostToolUse`/`Stop`,
  and a third shape applies at `PermissionRequest`. The drift reached Anthropic's own
  onboarding docs (issue [#19944](https://github.com/anthropics/claude-code/issues/19944)).
- **F2** — fail-open vs fail-closed on timeout depends on hook *transport*
  (`command`/`http`/`mcp_tool` fail **open**; SDK callbacks fail **closed**) and changed
  silently across point releases. Nothing in the payload says which regime applied.
- **F3** — HTTP hooks cannot fail closed at all: non-2xx, connection failure and
  2xx-with-plain-text are uniformly non-blocking. A 503 during a deploy degrades to
  allow-everything.
- **F4** — *"When multiple `PreToolUse` hooks return `updatedInput` … the last one to finish
  takes effect. Since hooks run in parallel, the order is non-deterministic."* A documented
  race in the input-rewriting path.
- **F5** — `permissionDecisionReason` is unregistered free-form text. Same as ACS's `reason`
  (ADR-057 F3). Three independent contracts, same mistake.
- **F6** — three vocabularies, no convergence, five unlanded MCP proposals to extend a
  fourth.
- **F7** — no provenance anywhere. *(The one above.)*

---

## Decision

Define **`caro.assessment.v1`**: one canonical, versioned, schema-emitting payload that every
Caro decision surface returns. Nine decisions.

**D1 — One envelope, one decision field, self-identifying.**
`Assessment.schema` is a literal `"caro.assessment.v1"`, serialized first. A field name is
never reused with different semantics across contexts; a new meaning gets a new schema
version. Directly answers F1.

**D2 — Reuse `SuggestedRouting` as the decision axis; do not invent a vocabulary.**
`risk: RiskLevel` says *what this command is*; `decision: SuggestedRouting` says *what to do*.
Two fields, two taxonomies, one payload. The enum already exists and is already correct; this
ADR promotes it from dead type to contract. `SuggestedRouting` gains `JsonSchema` and
`PartialOrd, Ord` with `AutoApprove < AsyncLog < HumanGate < Block`, so Claude Code's
documented *"the most restrictive answer applies"* rule becomes `max()` — a type-level
property rather than logic each adapter re-derives.

**D3 — `evidence` is required, is an enum, and names the model.**
`Evidence` is `Deterministic` | `ModelConsultedNoEffect` | `Blended`. The `Blended` variant
carries the `JudgeAttribution` (backend, model, confidence, threshold), the **deterministic
decision and risk the judge overrode**, and a `BlendDirection` of `Relaxed` or `Escalated`.
Never `Option`, never defaulted, never inferred from prose. This is the F7 fix and the reason
this ADR exists.

**D4 — `fail_mode` is declared in the payload, and silence is unrepresentable.**
Every path emits an `Assessment`, including usage errors and internal errors, which emit
`decision: block`, `fail_mode: fail_closed` and a namespaced `reason_code`. `assess_or_block`
is infallible by signature. Answers F2 and F3: a consumer never has to know which transport
produced a payload to know which way it failed.

**D5 — Caro never emits a rewritten command.**
A safer alternative travels as `suggested_alternative: Option<String>` — advice a host may
re-submit for a **fresh** assessment. It is never a mutation applied behind a decision.
Answers F4, and confirms ADR-057's no-`transform` rule as cross-vendor rather than an ACS
quirk.

**D6 — `reason_code` is namespaced and machine-branchable; `message` is not parseable.**
`reason_code` is `caro.<risk>.<rule_id>` using ADR-057 §3.2's content-addressed `RuleId`.
The two schemes must never diverge. `message` is human text with a documented cap and an
explicit `message_truncated: bool` rather than a silent spill. Answers F5.

**D7 — Host projection is data in the payload, and lossy projections fail toward
restriction.**
`projections: Vec<Projection>` carries, per known host, the exact string that host's
vocabulary expects, a `lossy` flag, a `note`, and the `target_vocabulary_version` the mapping
was built against. Adapters read the table; they do not each re-derive it. **When a host
cannot represent `async_log`, the projected value is that host's `ask` equivalent, not its
`allow`** — losing a recording obligation must cost friction, not safety. An unknown or
unversioned host vocabulary is **omitted** from the array rather than guessed, because a
missing projection fails loudly and a wrong one fails open. Answers F6.

**D8 — One new verb, `caro assess`, pure subprocess, no daemon, no state.**
An assessment is a pure function of (command, cwd, shell, config, optional judgment). It
inherits zero lifecycle obligations. `--json` writes exactly one `Assessment` to stdout;
`--schema` writes the JSON Schema and exits 0. Transports (MCP, hook adapter, proxy) are
**out of scope** and become consumers.

**D9 — This ADR owns the exit-code registry.**
The exit code is derived from `decision` by a single mapping function, so code and payload
cannot disagree — the class of bug Claude Code needed three precedence rules and a
point-release fix to contain.

| Code | Decision / condition |
|---|---|
| `0` | `auto_approve` |
| `18` | `async_log` — permitted, host **must** record |
| `19` | `human_gate` — obtain human approval |
| `20` | `block` — refuse |
| `2` | usage error — fail closed, **payload still written** |
| `1` | internal error — fail closed, **payload still written** |

Monotonic in restrictiveness, so `[ $? -ge 19 ]` is a valid gate without parsing JSON.
`ExitCode` is a single `#[repr(i32)]` enum in `src/safety/assessment.rs` containing **every**
code claimed by any ADR — including those this ADR does not use — with a test asserting no
two variants collide. Codes 18–20 were chosen specifically to leave 0–17 undisturbed for the
ADRs that claimed them, so adoption costs those ADRs nothing. ADR-053 complained the registry
was unowned; ADR-057 repeated the complaint and added a sixth uncoordinated claim. Prose in
seven sibling ADRs is not a registry; an enum is.

### Scope of change

One new file (`src/safety/assessment.rs`), five touched
(`src/safety/mod.rs`, `src/models/mod.rs`, `src/main.rs`, plus the ADR and
`tests/assessment_contract.rs`). **No new module. No dependency delta** — `serde`,
`serde_json` and `schemars 0.8` (`Cargo.toml:46`) are already direct dependencies, so
`.claude/rules/external-sdk-integration.md` does not attach.

Three prerequisite fixes, non-breaking on the wire, land first as one small `chore(models):`
PR: `SuggestedRouting` gains `JsonSchema` (it cannot currently emit a schema) and `Ord` (the
precedence rule cannot currently be written as `max()`), and `RiskLevel`'s `rename_all`
changes `"lowercase"` → `"snake_case"` so one payload does not carry two casing conventions.
Every `RiskLevel` variant is a single word, so the emitted strings are byte-identical; a test
asserts it.

---

## Consequences

### Positive

- **Two ADRs stop waiting on a type that does not exist.** ADR-057 §3.6's `DownMapEntry`
  becomes `Projection`; ADR-020 learns what an approver receives; ADR-015/038 learns what
  `assess_command` returns. The 08-25 scan's #1 build item (`caro-mcp`) becomes a transport
  spike over a settled type instead of a spike that invents one.
- **"We don't rely on LLM safety" becomes a field an auditor can grep** rather than a website
  claim. Caro's judge is already bounded — `Critical` never relaxed, low confidence ignored,
  the judge can never hard-block — so disclosure is a specification, not a confession.
- **The assessed subject is a `command` string**, not a `tool_name` plus `args`. The scan's
  standing negative result (*"still no competitor gating at the shell command string
  layer"*) becomes visible in the schema itself.
- **Reproducibility is assertable.** Same command + config + version ⇒ byte-identical
  payload; no clock, no network, no telemetry.
- **One payload, four transports.** ADR-015/038, ADR-036, ADR-043 and ADR-057 stop being four
  schemas that will drift.

### Negative / risks

- **A required `evidence` field is a disclosure competitors will screenshot.** This is the
  strongest objection and belongs in the devil's-advocate review, not in a rebuttal here.
  The position taken: a bounded, disclosed judge with a `Critical` floor is a stronger claim
  than an undisclosed one, and the differentiation dies the moment disclosure is optional.
- **The projection table will go stale.** Claude Code deprecated `decision`/`reason` and
  added `defer`; MCP has five unlanded annotation proposals. A stale projection forwards a
  string the host no longer accepts, which most hosts treat as a **non-blocking** error — it
  degrades to allow. Mitigated by `target_vocabulary_version`, the omit-rather-than-guess
  rule (D7), and a scheduled CI diff against each host's documented vocabulary.
- **`async_log` has no home in any host vocabulary.** All three project it lossily. D7's
  "project to `ask`, not `allow`" makes that safe at the cost of friction users will notice.
- **`evidence` becomes noise if `ApprovalMode::Smart` ever becomes the default** — most
  payloads would read `ModelConsultedNoEffect` and readers would stop looking, hiding the
  rare `Blended`. `Smart` stays non-default in v1.
- **A golden-payload test is a maintenance tax.** Intentional: field order, casing and
  `reason_code` format are the contract, and a byte-for-byte assertion is the cheapest way to
  make a silent change loud.

### Neutral

- Runtime behaviour does not change. `src/cli/mod.rs` still branches on
  `requires_confirmation` / `blocked_reason`; wiring `decision` into execution remains
  ADR-020's scope and its own argument. This ADR only makes the decision **sayable**.
- `Subject.cwd` is carried but nothing is computed from it. The target axis
  (*"dangerous against **this** target"*) is ADR-047's.

---

## Alternatives considered

**A1 — Define a new decision enum matching the market's `block`/`warn`/`monitor`/`require_approval`
wording (as the market scan's §5 implies).**
Rejected. `SuggestedRouting`'s four values already cover the same space with better names for
Caro's semantics, are already computed correctly by `from_risk_and_safety`, and already
carry helper methods. Inventing a parallel enum would create the exact drift D1 exists to
prevent, and would leave the existing one dead a second time. The scan's complaint is about
*exposure*, not vocabulary.

**A2 — Adopt Claude Code's vocabulary directly as Caro's own (`allow`/`deny`/`ask`).**
Rejected. It has no log-and-continue state, which is most of what a safety layer should say
about moderate-risk work; adopting it would collapse `async_log` into `allow` at the source
rather than at the boundary, and the loss would then be unrecoverable and unmeasured. Betting
Caro's schema on one host also contradicts the agent-agnostic thesis in a week when the scan
counted six new execution environments.

**A3 — Emit provenance as a free-text `note`, as `SmartDecision` does today.**
Rejected. That is the status quo and it is the F7 failure. A consumer that must regex
`"smart: "` out of a prose string to learn a model was involved has no contract, and the
string is exactly the kind of thing that gets reworded in a refactor.

**A4 — Make `evidence` optional so deterministic-only builds can omit it.**
Rejected. An optional disclosure field is absent precisely when it matters most, and a reader
cannot distinguish "no model" from "field not populated." `Deterministic { pattern_count }`
costs nothing to emit and makes absence of a model an assertion rather than an inference.

**A5 — Let each adapter map Caro's decision into its host's vocabulary.**
Rejected. Four adapters means four chances to map `async_log` to `allow` and one of them will.
Putting the table in the payload makes the mapping reviewable in one place, versionable, and
testable (T6), and makes lossiness a published property rather than an adapter's private
decision.

**A6 — Ship the payload together with an MCP transport, per the scan's #1 build item.**
Rejected for this PR. Transports are covered by three existing ADRs and each has its own
failure surface; shipping one inside the contract PR is how the contract acquires a
transport's bugs. The contract is the thing three ADRs are blocked on; it should land alone.

**A7 — Reuse `SafetyAssessmentOutput` as ADR-015 specified.**
Rejected on the evidence: the type has never existed. Two ADRs have been told to block on it
for months. Keeping the name would preserve a dependency that has never been satisfiable.
The name is superseded here, and ADR-015's reference should be corrected in a separate docs
PR so nothing else waits on it.

---

## References

- [Claude Code — Hooks reference](https://code.claude.com/docs/en/hooks) · [Hooks guide](https://code.claude.com/docs/en/hooks-guide) (fetched 2026-08-25)
- [anthropics/claude-code#19944](https://github.com/anthropics/claude-code/issues/19944) — deprecated `decision`/`reason` in the guide (2026-01-22)
- [OpenAI Agents SDK — Human in the loop](https://openai.github.io/openai-agents-python/human_in_the_loop/)
- [MCP 2025-11-25 — Tools / `ToolAnnotations`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) · [Elicitation (draft)](https://modelcontextprotocol.io/specification/draft/client/elicitation) · [Tool annotations blog](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/) (2026-03-16)
- `.hermes/digests/2026-08-25-weekly-agent-market-scan.md` §2.2, §2.4, §3.1, §3.3, §4, §5
- `caro-scope-assessment-contract-2026-08-25.md` — full three-phase scope, type definitions,
  test matrix, demoware-trap analysis
- Caro tree at `1.4.0`: `src/models/mod.rs:152,189,198,288,332,344`,
  `src/safety/mod.rs:175,189,256–263,265–276,277,306–312`, `src/safety/patterns.rs`,
  `src/backends/mod.rs:71`, `src/main.rs:938`, `Cargo.toml:42,46`
