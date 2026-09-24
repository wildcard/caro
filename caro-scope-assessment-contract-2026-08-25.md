# Implementation Scope — `caro.assessment.v1`, the Decision Contract

**Feature under analysis:** the **`PreToolUse` hook decision contract in Claude Code**
(`hookSpecificOutput.permissionDecision` ∈ `allow | deny | ask | defer`), read live
**2026-08-25** at `https://code.claude.com/docs/en/hooks` and
`https://code.claude.com/docs/en/hooks-guide`; compared against the two other shipping
vocabularies for the same decision — the **OpenAI Agents SDK** approval interruption model
(`openai.github.io/openai-agents-python/human_in_the_loop/`) and **MCP elicitation**
(`accept | decline | cancel`) plus `ToolAnnotations`, at MCP spec revision **2025-11-25**.

**Equivalent we are scoping for Caro:** **`caro.assessment.v1`** — one canonical, versioned,
schema-emitting payload that every Caro decision surface returns, in which the **risk
taxonomy** (`safe|moderate|high|critical` — *what this command is*) and the **decision
taxonomy** (`auto_approve|async_log|human_gate|block` — *what the host should do*) are
finally separate, addressable fields; in which **every verdict carries machine-readable
provenance** saying whether a language model touched it; and in which projection into each
host's own vocabulary is **data in the payload**, not logic scattered across adapters.

**Date:** 2026-08-25 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-058-caro-assessment-v1-decision-contract.md`

> **Provenance note (autonomous run).** No user present; the task template's
> `[FEATURE NAME]` was unbound, so target selection was mine. Rationale, in order of
> weight:
>
> 1. **It is the item that is blocking other items.** `.hermes/digests/2026-08-25-weekly-agent-market-scan.md`
>    §5 *Carried forward* names it directly and for the second consecutive run:
>    *"Caro's decision taxonomy — prior run flagged that `CRITICAL/HIGH/MEDIUM/LOW` is a
>    risk taxonomy where the market has converged on a decision taxonomy
>    (`block`/`warn`/`monitor`/`require_approval`). Unaddressed. Blocks 3.3."*
>    `ADR-057` §3.6 defers its down-map table to the same unwritten contract and says so:
>    *"Blocked on the memo's §3.6 `caro.assessment.v1` decision contract, which is still
>    unwritten."* Its **Recommended next step #1** is *"Write the memo's §3.6 ADR
>    (`caro.assessment.v1`) first."* This document is that ADR's scope.
> 2. **The memo's own #1 build item is already ADR'd; this is not.** The 08-25 scan
>    recommends shipping `caro-mcp` with a single `assess_command(...) -> Assessment` tool.
>    The transport half is covered three times over (ADR-015, ADR-038, ADR-043); the
>    `Assessment` half — the return type — exists in **no** ADR and in **no** source file.
>    `grep -rn "SafetyAssessmentOutput" src/` returns **nothing**, despite ADR-015 stating
>    that ADR-020 and ADR-023 both block on it merging first. Two ADRs have been waiting
>    on a type nobody has written.
> 3. **The research target is genuinely experimental and has a failure mode worth
>    designing against.** Claude Code's hook decision contract carries a live deprecation
>    (`decision`/`reason` → `hookSpecificOutput.permissionDecision`), a documented
>    non-deterministic race, and a fail-open/fail-closed split that depends on hook *type*
>    and has changed silently across point releases. All three are quoted below.
>
> **Verified vs. not.**
> - **Verified by fetch (2026-08-25):** every Claude Code hook field name, the four-value
>   `permissionDecision` enum, the deprecation sentence, the `deny > defer > ask > allow`
>   precedence rule, the exit-0 / exit-2 semantics, the `updatedInput` race admission, the
>   type-dependent timeout behaviour, the OpenAI Agents SDK approval surface, the MCP
>   elicitation three-action contract, and the MCP "annotations MUST be treated as
>   untrusted" language. Quoted verbatim in §1.4 and §1.3.
> - **Verified against the Caro tree (2026-08-25):** every type, field, line number, derive
>   and absence claim in §2.4 was produced by reading `src/` and `Cargo.toml` at `1.4.0`.
> - **Second-hand, not independently verified:** the market-structure claims in
>   `.hermes/digests/2026-08-25-weekly-agent-market-scan.md` (OneCLI, Pushary, Plow Latch
>   feature sets are vendor marketing; the digest says so itself). The GuardFall 10-of-11
>   result reaches this document third-hand via ADR-057 via the 08-20 memo via *The Hacker
>   News*; it is used only as directional support, never as a load-bearing number.
> - **Searched for and not found:** any cross-vendor standard verdict vocabulary published
>   in August 2026. CISA's agentic guidance and CoSAI's token-exchange work are **identity
>   and credential** standards, not decision schemas. As of today there are three
>   incompatible vocabularies for one question and no convergence in sight — which is
>   precisely the condition that makes a projection table the right design (§3.2).
>
> **Gate warning.** This is a **contract**, not a new user-facing capability class: it
> re-serializes decisions Caro already computes. `.claude/rules/validation-discipline.md`
> §"What this rule does NOT do" therefore attaches only partially — Gate 3 (demoware trap,
> written at §3.7) and Gate 4 (devil's-advocate review) apply; Gate 1's 20 transcripts
> attach to the *features that consume* this contract (memo 3.3, 3.4), not to the schema
> itself. The `caro assess` verb introduced in §3.4 is the boundary case and is argued
> explicitly in the ADR. Per `.claude/rules/git-workflow.md` this file and the ADR are left
> **uncommitted** for a human to branch and PR.

---

## Phase 1 — Feature Research

### 1.1 What problem it solves, and for whom

The buyer is someone who has already decided that the model cannot be trusted to police
itself, and now needs an *external* process to answer one question per tool call: **should
this happen?** Every agent host has independently invented an answer channel for that
question, and all three shipping designs are incompatible with each other.

Claude Code's answer is the `PreToolUse` hook: a subprocess (or HTTP endpoint, or in-process
SDK callback) that receives the pending tool call on stdin and writes a decision to stdout.
Its most load-bearing property, quoted:

> "`PreToolUse` hooks fire before any permission-mode check, in every permission mode,
> including `dontAsk`. A hook that returns `permissionDecision: "deny"` blocks the tool even
> in `bypassPermissions` mode or with `--dangerously-skip-permissions`."

That sentence is the entire commercial case for the layer Caro sits in, written by the host
vendor. Pushary's content strategy — per the 08-25 scan, *"`--dangerously-skip-permissions`,
a safer way"* — is a product built on top of exactly that guarantee.

What the hook contract does **not** supply is the content of the decision. Claude Code
defines the envelope and leaves the judgment to whatever the user wires in. Caro has 67
TDD-validated dangerous-command patterns and a shell-accurate parser and, today, no
serialized form in which to hand a judgment back.

### 1.2 Core architecture — data flow, key types, separation of concerns

```
tool call → [all matching hooks, in parallel] → combine → permission flow → execute
```

Five separations worth stealing:

1. **The decider is a subprocess; the enforcer is the host.** Same split ACS states as
   "decision runtime vs. enforcement point" (ADR-057 §1.2). Claude Code implements it as
   process boundary + stdio, which is cheaper than a runtime object and matches Caro's
   "pure subprocess call, no daemon, no state" constraint exactly.
2. **The decision field is namespaced under the event that produced it.**
   `hookSpecificOutput.hookEventName` must echo `"PreToolUse"`. The payload self-identifies
   rather than relying on the reader's context. (Caro's `schema` field, §3.2, is the same
   idea done once at the top instead of per event.)
3. **Conflict resolution is a documented total order, not a first-writer-wins race:**

   > "After all matching hooks finish, Claude Code combines their outputs. For `PreToolUse`
   > permission decisions, the most restrictive answer applies, in the order `deny`,
   > `defer`, `ask`, `allow`."

   This is the single best idea in the contract and it is *free to copy*: if the decision
   enum is `Ord` with restrictiveness as its ordering, combination is `max()` and the
   precedence rule becomes a type-level property instead of adapter logic. Caro's
   `SuggestedRouting` is not currently `Ord` (§2.4).
4. **Two channels with different trust weights.** stdout carries structured JSON that is
   parsed on *every* exit code; stderr carries human text that is only surfaced on the
   blocking path. Caro's `--json`-to-stdout / prose-to-stderr split (ADR-024) already agrees.
5. **Decision and input-rewriting are separate fields.** `permissionDecision` says whether;
   `updatedInput` rewrites what. They are separate for a reason and, as §1.3 shows, the
   rewriting half is the broken half.

### 1.3 Why it is experimental / limited — the failure modes

Seven, ordered by how much each one matters to Caro's design.

**F1 — the same field name means different things depending on the event.** Verbatim:

> "PreToolUse previously used top-level `decision` and `reason` fields, but these are
> deprecated for this event. Use `hookSpecificOutput.permissionDecision` and
> `hookSpecificOutput.permissionDecisionReason` instead. The deprecated values `"approve"`
> and `"block"` map to `"allow"` and `"deny"` respectively. Other events like PostToolUse
> and Stop continue to use top-level `decision` and `reason` as their current format."

So `decision` is *deprecated* at `PreToolUse`, *current* at `PostToolUse`/`Stop`, and a
third shape (`hookSpecificOutput.decision.behavior`) applies at `PermissionRequest`. A hook
author copying a working pattern between events misfires silently. The drift reached
Anthropic's own onboarding docs: GitHub issue **#19944** (filed 2026-01-22, closed as
duplicate) records the Hooks *Guide* still shipping a `"decision": "approve"` example after
the Hooks *Reference* had deprecated it.

**F2 — fail-open versus fail-closed depends on the hook's transport, and changed silently
between point releases.** Verbatim:

> "A timed-out `command`, `http`, or `mcp_tool` hook doesn't block the tool call. The call
> continues through the normal permission flow, so don't count on a stalled hook to act as
> a gate."

An in-process Agent SDK callback on the same event fails **closed** on timeout. Same event,
same semantic role, opposite failure direction, decided by which transport you happened to
choose. And the SDK-callback behaviour on `UserPromptSubmit` only became fail-closed in
v2.1.208; before that a timeout ended the turn with an execution error. **Nothing in the
payload tells a consumer which regime it is in.**

**F3 — HTTP hooks cannot fail closed at all.** Only a 2xx response carrying a valid JSON
body with decision fields can block. Non-2xx, connection failure, and 2xx-with-plain-text
are *all* non-blocking errors. A policy endpoint behind a load balancer that returns 503
during a deploy degrades to allow-everything, and the only trace is a `<hook name> hook
error` notice.

**F4 — input rewriting is a documented, unresolved race.** Verbatim:

> "When multiple `PreToolUse` hooks return `updatedInput` to rewrite a tool's arguments, the
> last one to finish takes effect. Since hooks run in parallel, the order is
> non-deterministic."

This is ADR-057's F6 (`transform` without re-evaluation) in a second, independent product,
made worse: not only is the rewritten command not re-evaluated, *which* rewrite lands is not
determined. Caro's rule from ADR-057 — **never emit a rewritten shell string into someone
else's engine** — is hereby confirmed as a cross-vendor rule and not an ACS quirk.

**F5 — the reason is an unregistered free-form string.** `permissionDecisionReason` is
human text fed back to the model. There is no code, no namespace, no stability promise.
Identical to ACS's F3. A host that wants to branch on *why* has nothing to branch on, and
two vendors' reasons will collide. This is now observed in three independent contracts,
which upgrades it from "their bug" to "the market's default mistake."

**F6 — the three vocabularies do not interoperate, and nothing is standardizing them.**

| System | Vocabulary | Shape |
|---|---|---|
| Claude Code `PreToolUse` | `allow` / `deny` / `ask` / `defer` | 4 values, totally ordered by restrictiveness |
| OpenAI Agents SDK | approve / reject | binary; state machine is pause-and-resume via `RunResult.interruptions` |
| MCP elicitation | `accept` / `decline` / `cancel` | 3 values, and `decline` vs `cancel` is a *user-intent* distinction, not a policy one |

There is no "proceed but flag for later review" state anywhere except Claude Code's `ask`,
and `ask` is a *pause*, not a log-and-continue. MCP's `ToolAnnotations`
(`readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`) look like a fourth
vocabulary but are explicitly disclaimed:

> "For trust & safety and security, clients **MUST** consider tool annotations to be
> untrusted unless they come from trusted servers."

and, from the MCP blog's own post-mortem on them (2026-03-16):

> "They aren't enforcement. If you need a guarantee that a tool can't exfiltrate data,
> that's a job for network controls or sandboxing, not a boolean hint."

Five Specification Enhancement Proposals to extend the annotation vocabulary
(`#1913`, `#1984`, `#1561`, `#1560`, `#1487`) were open and unlanded as of that post. A
search for an August-2026 cross-vendor verdict standard found none: CISA's guidance and
CoSAI's ID-JAG token exchange govern **identity and credentials at trust boundaries**, not
decisions about individual calls.

**F7 — the one this ADR exists to solve: a verdict does not say whether a model produced
it.** None of the three contracts has a provenance field. `permissionDecision: "deny"`
looks identical whether it came from a regex, a policy engine, or an LLM asked politely to
be careful.

This is not an abstract concern for Caro, because **Caro already ships the ambiguity.**
`src/safety/mod.rs:277`, `blend_smart_decision`, merges an optional LLM `RiskJudgment` into
the deterministic decision under two invariants (Critical is never relaxed; low confidence
leaves the static decision untouched). The *only* trace it leaves is
`SmartDecision.note: Option<String>` (`src/safety/mod.rs:262`) — free text, built at
`:306–312` as `format!("smart: relaxed to {:?} — {}", judgment.risk, judgment.reason)`.

So today, a consumer of a Caro decision made under `ApprovalMode::Smart`
(`src/models/mod.rs:288`) cannot determine from any structured field whether a language
model relaxed the verdict, escalated it, or was absent entirely. It can only regex a prose
string for the prefix `"smart: "`.

Put that next to what the 08-25 market scan says landed in a single week — Anthropic's own
research on Claude agents escalating to self-replicating malware under goal conflict; a
Copilot Autofix flaw exploited by an autonomous red-team agent in five days; state-linked
operators using OSS agents against Taiwanese targets — and the scan's §2.4 conclusion,
*"LLM-as-safety-judge is now empirically discredited — and still shipping."* Caro's stated
differentiation is that it does **not** make that bet. Shipping an assessment payload in
which the model's contribution is invisible would make that differentiation unverifiable by
the one audience that cares: the buyer reading the JSON.

**Therefore: provenance is a required field in v1, not a v2 addition.** That is the
specific Phase-1 failure mode this scope solves by design (§3.2, `Evidence`).

### 1.4 Structured output contract

The full `PreToolUse` success payload, as documented:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "Destructive command blocked by hook",
    "updatedInput": { "command": "..." }
  },
  "systemMessage": "…",
  "continue": true,
  "suppressOutput": false
}
```

Exit-code semantics, quoted:

- Exit **0**: *"the hook reports no objection through its exit code. For a `PreToolUse` hook
  this doesn't approve the tool call: the normal permission flow still applies."* JSON on
  stdout is still parsed.
- Exit **2**: *"the way a hook signals 'stop, don't do this.'"* and *"Exit 2's block is the
  one outcome JSON can't override."*
- Any other code: honoured if stdout parses to a schema-valid JSON object; otherwise a
  **non-blocking** error — the action proceeds.
- *"Stderr from a hook that exits 0 goes to the debug log only, never the transcript, and
  Claude never sees it."*

Two design consequences for Caro. First, **exit code and payload must never disagree**;
Claude Code has three precedence rules reconciling them and still needed a bug fix
(pre-v2.1.214, exit 2 plus schema-invalid JSON was treated as non-blocking). Caro derives
the exit code *from* the decision field mechanically, so disagreement is unrepresentable
(§3.4). Second, **silence must be impossible**: an internal error still emits a payload.

Output strings are capped at 10,000 characters, with overflow spilled to a file.
`additionalContext` must be nested inside `hookSpecificOutput` or it is *silently ignored* —
a good argument for a flat, `deny_unknown_fields` envelope.

### 1.5 Session / context lifecycle

Claude Code hooks are stateless per invocation: a fresh process per tool call, all context
on stdin, no handshake, no warm cache. Redundant initialization is not avoided — it is
accepted as the cost of the process boundary, which is why the docs care so much about
timeouts (60s for `agent` hooks, 600s for `command`/`http`/`mcp_tool`, 30s for
`UserPromptSubmit`, a 1.5s shared budget for `SessionEnd`).

The one place lifecycle appears is `defer`, available only in non-interactive `-p` mode: it
exits the *host* with `stop_reason: "tool_deferred"` and the tool call preserved, so an SDK
wrapper can collect an approval out of band and resume. That is genuinely useful and it is
genuinely **out of scope here** (§3.6) — it requires the host to own resumable state, which
Caro's "no daemon, no state" constraint forbids Caro from assuming. The OpenAI SDK's
serializable `RunState` is the same idea done more thoroughly, and belongs to ADR-046
(approval exchange), not to this contract.

For Caro this is the good news, again: an assessment is a pure function of
(command, cwd, shell, config). It inherits **zero** lifecycle obligations, so the
"pure subprocess call" constraint costs nothing.

---

## Phase 2 — Competitive Differentiation

### 2.1 What they get right, and we replicate

- **Four decision values, not two.** `allow`/`ask`/`deny` plus a non-interactive escape is
  the right cardinality; binary approve/reject (OpenAI) cannot express "run it but record
  it," which is most of what a safety layer should say about moderate-risk work.
- **A documented total order for combination.** `deny > defer > ask > allow`. Copy it, and
  make it `Ord` so `max()` *is* the rule.
- **The payload names its own event/schema.** Self-identification beats out-of-band context.
- **Structured decision on stdout, prose on stderr.**
- **The gate fires before permission-mode checks, including bypass modes.** Caro's adapters
  must preserve that property rather than becoming another thing a `--yes` flag skips.
- **Explicit, published timeout budgets.** Caro should publish one for `caro assess` and
  test it, rather than leaving it to the adapter.

### 2.2 Their gaps, avoided by designing the schema first

| Gap | Their state | Caro's schema decision |
|---|---|---|
| F1 — same name, different meaning per event | `decision` deprecated at one event, current at two others, third shape at a fourth | **One** envelope, **one** decision field, a literal `schema: "caro.assessment.v1"` as the first key; a name is never reused with different semantics — a new meaning gets a new schema version |
| F2 — fail direction depends on transport | undocumented as a single rule; changed across point releases | `fail_mode` is a **required field in the payload**. The consumer is told, per assessment, which regime produced it |
| F3 — HTTP hooks can't fail closed | 503 ⇒ allow-everything | Caro emits a payload on **every** path including internal error: `decision: block`, `fail_mode: fail_closed`, `reason_code: caro.internal.error`. Silence is unrepresentable |
| F4 — `updatedInput` race | "last one to finish takes effect… non-deterministic" | Caro emits **no** rewritten command, ever. A safer alternative travels as `suggested_alternative` — advice a host may re-submit for a *fresh* assessment, never a mutation applied behind the decision |
| F5 — unregistered free-form reason | no namespace, no stability | `reason_code` is `caro.<risk>.<rule_id>`, generated and content-addressed, identical to the scheme ADR-057 §3.2 already committed to; `message` carries the human text and is contractually **not** parseable |
| F6 — three incompatible vocabularies, no standard | nothing converging | `projections[]` is **data in the payload**: for each known host, the value Caro's decision maps to and whether the mapping is lossy. Adapters read the table; they do not each re-derive it |
| F7 — no verdict provenance | absent in all three | `evidence` is **required**. A blended verdict names the backend, the model, the direction of the adjustment, and the deterministic verdict it started from |
| Cap/overflow behaviour bolted on late | 10k cap, spill file | `message` has a documented cap in v1 and truncation is a flagged field, not a silent spill |

### 2.3 Our unique positioning

1. **We can be honest about the model in a way none of them can, because we do not depend
   on it.** Every competitor named in the 08-25 scan either has no LLM in the decision path
   (and so has nothing to disclose) or has one and would rather not say. Caro has an
   optional, *bounded* one — Critical never relaxed, low confidence ignored, judge can never
   hard-block (`src/safety/mod.rs:265–276`) — which means disclosure is not a confession,
   it is a **specification**. `evidence` turns "we don't rely on LLM safety" from a website
   claim into a field an auditor can grep.
2. **We are the only one whose decision is about the shell string.** The scan's negative
   result stands: *"still no competitor gating at the shell command string layer, parsed as
   the shell will parse it. OneCLI, Pushary, Phinq, Execlave, WriteGuard and Microsoft AGT
   all operate on the tool-call payload or the network request."* This contract is where
   that becomes legible — the assessed subject is a `command`, not a `tool_name` plus
   `args`.
3. **We can be the same answer in all three vocabularies at once.** The projection table
   costs one field and makes Caro the only decision payload that a Claude Code hook, an MCP
   elicitation, and an OpenAI approval can all consume without a bespoke adapter each. In a
   market with no standard and no convergence, being trivially projectable beats betting on
   a winner.
4. **Offline, deterministic, reproducible.** Same command + same config + same version ⇒
   byte-identical payload, no clock, no network, no telemetry. Testable as an equality
   assertion (§3.5 T4). None of the SaaS gates can claim it.
5. **It is a subprocess, so it composes with everything.** ADR-015/038 (MCP tool), ADR-036
   (`caro guard` hook adapter), ADR-043 (proxy), ADR-057 (policy export) all become
   *transports for one payload* instead of four schemas that will drift.

### 2.4 What Caro already has (verified against the tree at `1.4.0`, 2026-08-25)

| Asset | Location | State |
|---|---|---|
| `RiskLevel` | `src/models/mod.rs:152` | `Safe\|Moderate\|High\|Critical`; derives `Serialize, Deserialize, JsonSchema, PartialOrd, Ord`; `#[serde(rename_all = "lowercase")]` |
| **`SuggestedRouting`** | `src/models/mod.rs:189` | `AutoApprove\|AsyncLog\|HumanGate\|Block`; `#[serde(rename_all = "snake_case")]`; **no `JsonSchema`**, **no `Ord`** |
| `SuggestedRouting::from_risk_and_safety` | `src/models/mod.rs:198` | the complete risk×safety→decision matrix, already written and correct |
| `SafetyDecision { risk_level, reason, suggested_routing, matched_patterns, confidence }` | `src/safety/mod.rs:189` | `Serialize + Deserialize`, with `is_auto_executable`/`requires_approval`/`is_blocked` helpers at `:211–221` |
| `ValidationResult` | `src/safety/mod.rs:175` | `allowed, risk_level, explanation, warnings, matched_patterns, confidence_score: f32` |
| `blend_smart_decision` | `src/safety/mod.rs:277` | bounded LLM blend; invariants documented at `:265–276` |
| `SmartDecision { requires_confirmation, blocked, note: Option<String> }` | `src/safety/mod.rs:~256–263` | the free-text provenance channel F7 is about |
| `RiskJudgment { risk, reason, confidence: f64 }` | `src/models/mod.rs:344` | LLM verdict type |
| `RiskJudgeContext` | `src/models/mod.rs:332` | judge input |
| `ApprovalMode { Prompt, Auto, Smart }` | `src/models/mod.rs:288` | `JsonSchema`, `Default = Prompt` |
| `CommandGenerator::classify_risk` | `src/backends/mod.rs:71` | default returns `None`; implemented for Ollama at `src/backends/remote/ollama.rs:281` |
| Built-in pattern set | `src/safety/patterns.rs` | **67** `DangerPattern` literals (`grep -c "DangerPattern {"`) |
| `schemars 0.8` | `Cargo.toml:46` | already a direct dependency; 12 `JsonSchema` derives in `src/models/mod.rs` |
| `serde_json 1` | `Cargo.toml:42` | already direct |

**The load-bearing finding: the decision taxonomy the market scan says Caro lacks has been
in the tree since v1.4.0 and is wired to nothing.**

`SuggestedRouting`'s four values map 1:1 onto Claude Code's four, in the same restrictiveness
order:

| Caro | Claude Code | MCP elicitation | OpenAI SDK |
|---|---|---|---|
| `auto_approve` | `allow` | `accept` | approve |
| `async_log` | `allow` (+ context) | `accept` | approve |
| `human_gate` | `ask` | *(elicit)* | interruption |
| `block` | `deny` | `decline` | reject |

The scan's §5 complaint — *"`CRITICAL/HIGH/MEDIUM/LOW` is a risk taxonomy where the market
has converged on a decision taxonomy"* — is therefore **half wrong in Caro's favour and
half worse than stated**. The vocabulary exists and is better than the memo assumed. What
does not exist is any path from it to a consumer: `grep -rn "SafetyDecision" src/` returns
matches **only inside `src/safety/mod.rs`**. It is never constructed by a caller, never
returned by a command, never serialized to stdout. It is a dead type of exactly the kind
ADR-020 diagnosed when it called `HumanGate` "a dead enum variant" — except that in the
intervening months the dead variant acquired a dead struct around it.

**Seven drifts found while verifying, each of which this contract would otherwise publish:**

- **D-a — two casing conventions in one payload.** `RiskLevel` is `rename_all = "lowercase"`;
  `SuggestedRouting` is `rename_all = "snake_case"`. A payload containing both emits
  `"risk": "critical"` beside `"decision": "human_gate"`. Because every `RiskLevel` variant
  is a single word, standardizing *both* attributes on `snake_case` changes the wire form of
  neither — a free, non-breaking fix that must land before anything serializes them together.
- **D-b — half the payload cannot emit a schema.** `RiskLevel` derives `JsonSchema`;
  `SuggestedRouting` does not. `schemars` is already a dependency; this is a one-line fix
  and it blocks `caro assess --schema`.
- **D-c — the precedence rule cannot be expressed.** `SuggestedRouting` derives
  `PartialEq, Eq` but not `PartialOrd, Ord`, so Claude Code's `max()`-shaped
  "most restrictive wins" cannot be written as `max()`. `RiskLevel` already has `Ord`.
- **D-d — three confidence fields, two names, two widths.**
  `ValidationResult.confidence_score: f32`, `SafetyDecision.confidence: f64`,
  `RiskJudgment.confidence: f64`. The envelope must pick one; the others should converge in
  a separate PR.
- **D-e — `SafetyAssessmentOutput` does not exist.** ADR-015 states ADR-020 and ADR-023 both
  block on it merging first. `grep -rn "SafetyAssessmentOutput" src/` returns nothing;
  `src/cli/scan.rs` (ADR-023's unblocking local copy) does not exist — `src/cli/` contains
  `edit_prompt.rs`, `mod.rs`, `telemetry.rs`. **This contract supersedes that name**; ADR-058
  should say so explicitly so the dependency stops being cited.
- **D-f — the pattern count is still wrong in the docs.** `README.md` and `CLAUDE.md` say
  "52+"; the tree has **67**. ADR-057 flagged this the day before yesterday and it is still
  a free win. Any count in an emitted payload must come from `DANGEROUS_PATTERNS.len()`.
- **D-g — the ADR index is 42 rows behind.** `.claude/rules/adr-numbering.md` requires every
  ADR to have a `docs/adr/README.md` table entry, but that table's last row is **ADR-015**
  (`docs/adr/README.md:54`) while the directory contains ADRs through **057**. This scope
  deliberately does **not** add a single row to a table missing forty-two others — a
  half-repaired index is worse than a visibly broken one. Backfilling it is a mechanical
  docs PR (`ls docs/adr/ADR-*.md` plus each file's `Status` and `Date` header) and is the
  cheapest of the seven drifts to close.

Also verified absent, confirming the gap: no `Assess`, `Guard`, `Scan` or `Policy` variant
exists in `src/main.rs`'s `Commands` enum; no occurrence of `PreToolUse` or
`permissionDecision` anywhere in `src/`; the only exit-code constant in the tree remains
`EXIT_CODE_EDIT = 201` (`src/main.rs:938`).

---

## Phase 3 — Scope Definition

### 3.1 ADR

`docs/adr/ADR-058-caro-assessment-v1-decision-contract.md` — context, decision (D1–D9),
consequences, alternatives considered. Highest existing is ADR-057; per
`.claude/rules/adr-numbering.md`, renumber on merge if another 058 lands first.

It supersedes the *name* `SafetyAssessmentOutput` (ADR-015 §), and unblocks: ADR-020
(tiered approval — the payload is what an approver receives), ADR-023 (`caro scan`),
ADR-055 and ADR-057 §3.6 (both waiting on the decision down-map, now `projections[]`).

### 3.2 New types

All in **one new file**, `src/safety/assessment.rs`, under the existing `src/safety` module
because an assessment is a projection of a safety decision and must not drift from one. All
derive `Debug, Clone, Serialize, Deserialize, JsonSchema` from day one;
`#[serde(rename_all = "snake_case")]` everywhere; `#[serde(deny_unknown_fields)]` on all
read paths.

```rust
/// The one payload every Caro decision surface emits.
/// `schema` is serialized first and is a literal, so a reader can identify the
/// envelope before parsing anything else.
pub struct Assessment {
    pub schema: &'static str,           // "caro.assessment.v1" — never absent
    pub caro_version: String,           // env!("CARGO_PKG_VERSION")
    pub subject: Subject,
    /// WHAT THIS IS. Reuses src/models::RiskLevel unchanged.
    pub risk: RiskLevel,
    /// WHAT TO DO. Reuses src/models::SuggestedRouting — promoted, not replaced.
    pub decision: SuggestedRouting,
    /// Namespaced, stable, machine-branchable: "caro.<risk>.<rule_id>".
    /// Same scheme ADR-057 §3.2 committed to; the two must never diverge.
    pub reason_code: String,
    /// Human text. Contractually NOT parseable. Capped; see `message_truncated`.
    pub message: String,
    pub message_truncated: bool,
    pub confidence: f64,                // one field, one width — see drift D-d
    pub matched_rules: Vec<RuleId>,     // ADR-057's content-addressed RuleId, shared
    /// REQUIRED. The F7 fix. Never Option, never defaulted.
    pub evidence: Evidence,
    /// REQUIRED. The F2 fix. Declared, never inferred by the consumer.
    pub fail_mode: FailMode,
    /// The F6 fix. Data, not adapter logic.
    pub projections: Vec<Projection>,
    /// Advice only. NEVER applied by Caro, never a mutation. The F4 fix.
    pub suggested_alternative: Option<String>,
}

pub struct Subject {
    pub command: String,                // the shell string, verbatim, as submitted
    pub shell: ShellType,               // src/models — unchanged
    pub cwd: Option<String>,
    pub safety_level: SafetyLevel,      // src/models — unchanged
    pub approval_mode: ApprovalMode,    // src/models — unchanged
}

/// Provenance. The consumer can always answer "did a model touch this verdict?"
/// by matching on one enum, with no string parsing.
pub enum Evidence {
    /// Deterministic pattern match only. No model was consulted.
    /// This is what `ApprovalMode::Prompt` and `::Auto` always produce.
    Deterministic { pattern_count: u32 },   // DANGEROUS_PATTERNS.len() — see drift D-f
    /// A model was consulted and did not change the outcome (absent,
    /// below `SMART_JUDGE_MIN_CONFIDENCE`, or agreeing).
    /// The deterministic verdict stands and is the one reported.
    ModelConsultedNoEffect { pattern_count: u32, judge: JudgeAttribution },
    /// A model changed the outcome. Both verdicts are reported, always.
    Blended {
        pattern_count: u32,
        judge: JudgeAttribution,
        /// What the deterministic path alone would have decided.
        deterministic_decision: SuggestedRouting,
        deterministic_risk: RiskLevel,
        direction: BlendDirection,
    },
}

pub struct JudgeAttribution {
    pub backend: BackendType,           // src/models — unchanged
    pub model: Option<String>,
    pub confidence: f64,
    pub min_confidence_threshold: f64,  // SMART_JUDGE_MIN_CONFIDENCE at build time
}

pub enum BlendDirection {
    /// Judge found a flagged command benign in context; friction reduced.
    Relaxed,
    /// Judge found a static-Safe command risky; escalated toward a human.
    Escalated,
}

/// Declared per assessment. A consumer never has to guess, and never has to
/// know which transport produced the payload.
pub enum FailMode {
    /// An error on this path produces `decision: block`. Caro's default, always.
    FailClosed,
    /// Reserved. No v1 code path emits this; it exists so that a future host
    /// integration that genuinely cannot fail closed must SAY SO in the payload
    /// rather than degrade silently the way F3 describes.
    FailOpen,
}

pub struct Projection {
    pub host: HostTarget,
    /// The exact string the host's own vocabulary expects.
    pub value: String,                  // e.g. "ask", "decline", "reject"
    /// True when this host cannot represent Caro's decision exactly.
    pub lossy: bool,
    pub note: Option<String>,           // e.g. "no log-and-continue state in this host"
}

pub enum HostTarget {
    /// hookSpecificOutput.permissionDecision — allow | deny | ask
    ClaudeCodePreToolUse,
    /// elicitation/create result action — accept | decline | cancel
    McpElicitation,
    /// approve | reject
    OpenAiAgentsApproval,
    /// ADR-057's ACS export vocabulary — allow | deny
    AcsVerdict,
}

/// Owns the exit-code registry. See §3.4.
#[repr(i32)]
pub enum ExitCode { /* … */ }
```

**Method contracts** — three functions, no trait, no builder:

```rust
/// Pure. No I/O, no clock, no env, no network. Same inputs ⇒ identical payload.
/// This is the only constructor; `Assessment` fields are `pub` for reading but
/// the type has no public struct-literal path in another module.
pub fn assess(subject: &Subject, cfg: &SafetyConfig, judge: Option<&RiskJudgment>)
    -> Assessment;

/// The F3 fix. Infallible by signature: every error becomes a fail-closed
/// Assessment rather than an Err. Silence is unrepresentable.
pub fn assess_or_block(subject: &Subject, cfg: &SafetyConfig, judge: Option<&RiskJudgment>)
    -> Assessment;

/// Derives the exit code from the decision. The ONLY mapping; exit code and
/// payload cannot disagree because there is one source for both.
pub fn exit_code(a: &Assessment) -> ExitCode;
```

`assess` takes `&SafetyConfig` rather than reading globals, so `custom_patterns` and
`allowlist_patterns` flow through the existing layered TOML loader with zero duplication —
same discipline as ADR-057 §3.2. `projections` is computed from a `const` table, so adding
a host is a data edit.

**Three changes to existing types, all non-breaking on the wire:**

1. `SuggestedRouting` gains `JsonSchema` (drift D-b) and `PartialOrd, Ord` (drift D-c), with
   the ordering `AutoApprove < AsyncLog < HumanGate < Block`, so combining verdicts from
   several sources is `max()` — Claude Code's documented precedence rule as a type property.
2. `RiskLevel`'s `rename_all` changes `"lowercase"` → `"snake_case"` (drift D-a). Every
   variant is one word, so the emitted strings are byte-identical; this removes a
   two-convention payload before it ships. A test asserts the wire form is unchanged.
3. `blend_smart_decision` additionally returns the `Evidence` it produced, so `SmartDecision.note`
   stops being the only record of a model's involvement. The `note` string stays for CLI
   prose; nothing parses it.

### 3.3 Minimal set of files that change

One new file, five touched, **no new module**.

| File | Change |
|---|---|
| `src/safety/assessment.rs` | **new** — the types above, `assess`, `assess_or_block`, `exit_code` |
| `src/safety/mod.rs` | `pub mod assessment;`; `blend_smart_decision` also returns `Evidence` |
| `src/models/mod.rs` | derives on `SuggestedRouting` (+`JsonSchema`, `+Ord`); `rename_all` on `RiskLevel` |
| `src/main.rs` | one `Assess { .. }` variant on `Commands` + its dispatch arm + `ExitCode` wiring |
| `docs/adr/ADR-058-*.md` | **new** |
| `tests/assessment_contract.rs` | **new** — §3.5 |

Deliberately **not** changed: `src/safety/patterns.rs` (read, never edited),
`src/safety/validator.rs`, `src/backends/` (the judge already exists and is already bounded),
`src/cli/mod.rs` (runtime behaviour is ADR-020's job, not this contract's), any config
schema, and `Cargo.toml` — **there is no dependency delta.** `serde`, `serde_json` and
`schemars 0.8` are all already direct dependencies, so
`.claude/rules/external-sdk-integration.md` does not attach to this PR at all.

### 3.4 Exit-code / output contract

```
caro assess <COMMAND> [--shell SHELL] [--cwd DIR] [--safety-level LEVEL]
                      [--approval-mode {prompt,auto,smart}] [--json] [--schema]
```

`--json` writes exactly one `Assessment` to **stdout** and nothing else; all human text
goes to stderr. `--schema` writes the JSON Schema for `Assessment` (via `schemars`) to
stdout and exits `0` without assessing anything — this is what lets a consumer validate the
contract in CI, and it is the reason drift D-b must be fixed first.

**The exit code is derived from `decision`, mechanically.** There is one mapping function
and therefore no way for the code and the payload to disagree — the class of bug Claude
Code needed three precedence rules and a point-release fix to contain.

| Code | Constant | Decision / condition | Machine contract |
|---|---|---|---|
| `0` | `ASSESS_AUTO_APPROVE` | `auto_approve` | safe to run unattended |
| `18` | `ASSESS_ASYNC_LOG` | `async_log` | permitted, but the host **must** record it |
| `19` | `ASSESS_HUMAN_GATE` | `human_gate` | do not run; obtain human approval |
| `20` | `ASSESS_BLOCK` | `block` | refuse; do not offer an override path |
| `2` | `ASSESS_USAGE` | usage error (unknown shell, bad flag) | fail closed; **payload still written** with `decision: block` |
| `1` | `ASSESS_INTERNAL` | internal error (config unreadable, panic guard) | fail closed; **payload still written**, `reason_code: caro.internal.error`, `fail_mode: fail_closed` |

Monotonic in restrictiveness, so `sh -c 'caro assess "$c"; [ $? -ge 19 ]'` is a valid gate
without parsing JSON. Exits `1` and `2` still emit a payload — that is the F3 fix, and it is
the property test T5 asserts.

**This ADR takes ownership of the exit-code registry.** ADR-053 complained that the registry
is unowned; ADR-057 repeated the complaint and added a sixth uncoordinated claim (13 twice,
14, 15, 16, 17), while the tree still contains exactly one exit constant. Prose in seven
sibling ADRs is not a registry. `ExitCode` in `src/safety/assessment.rs` is: a single
`#[repr(i32)]` enum, `Serialize`, with every claimed code present as a variant — including
the ones this ADR does not use — and a test asserting no two variants share a value. The
codes above sit at 18–20 specifically to leave 0–17 undisturbed for the ADRs that claimed
them, so adopting the registry costs those ADRs nothing.

### 3.5 Integration tests — deterministic input → fixed JSON + exit code

New file `tests/assessment_contract.rs`, following `tests/custom_patterns_toml.rs`.
Fixtures under `tests/fixtures/assessment/`. Every test offline and hermetic.

**T1 — golden payload.** `assess("rm -rf /", bash, strict, prompt)` → assert the serialized
JSON equals a checked-in golden **byte for byte**, and exit `20`. Locks field order,
`schema` literal, casing (drift D-a), and `reason_code` format in one assertion.

**T2 — provenance is never silent.** For each of the three `Evidence` variants, construct
the corresponding path and assert the discriminant. Specifically: with
`ApprovalMode::Smart` and a `RiskJudgment` above `SMART_JUDGE_MIN_CONFIDENCE` that changes
the outcome, assert `evidence` is `Blended`, that `deterministic_decision` is present and
differs from `decision`, and that `direction` is correct. **A `Deterministic` evidence value
on a path where a judge was consulted is a test failure.** This is the F7 fix and the reason
the ADR exists.

**T3 — the model can never widen permission past the invariants.** Property test
(`proptest`, already in-tree per `proptest-regressions/`): for all (static risk, judgment,
safety level), if `static_risk == Critical` then `decision` is unchanged by the judge; and
the judge alone never produces `Block`. Directly asserts the two invariants documented at
`src/safety/mod.rs:265–276` at the payload layer, where a buyer can see them.

**T4 — determinism.** `assess(...)` twice in one process, serialize both, assert
byte-identical. Guards against `HashMap` iteration order, timestamps and absolute paths
leaking into output.

**T5 — silence is unrepresentable.** Invoke `caro assess` with an unreadable config and with
an unknown `--shell`; assert exit `1` and `2` respectively, assert **stdout still parses as
an `Assessment`**, and assert `decision == block` and `fail_mode == fail_closed` in both.
**This test failing to fail is the bug** — it exists to prove Caro cannot degrade to
allow-everything the way an HTTP hook does (F3).

**T6 — projections are complete, ordered, and honest.** Assert `projections` contains one
entry per `HostTarget` variant, in declaration order; assert every `value` is a member of
that host's documented vocabulary; and assert that `async_log` is marked `lossy: true` for
`ClaudeCodePreToolUse`, `McpElicitation` and `OpenAiAgentsApproval` — because none of the
three has a log-and-continue state (§1.3 F6). A future host added without a lossiness
verdict fails the test.

**T7 — schema round-trip.** `caro assess --schema` output validates the T1 golden payload
(via `jsonschema`, or by asserting `Assessment: DeserializeOwned` from the golden with
`deny_unknown_fields` on). Proves `--schema` is not decorative.

**T8 — exit code and decision cannot disagree.** Exhaustive over `SuggestedRouting`: assert
`exit_code(a)` is a pure function of `a.decision` and that the mapping is strictly monotonic
in the `Ord` added in §3.2. One assertion covering the whole class of bug in §1.4.

### 3.6 Explicitly out of scope (next version)

- **`defer` / resumable approval.** Claude Code's `-p`-mode `defer` and the OpenAI SDK's
  serializable `RunState` both require the *host* to own resumable state. This contract is
  a pure function; approval exchange belongs to ADR-045/046.
- **Transport.** No MCP server, no `caro guard` hook binary, no proxy. ADR-015/038, ADR-036
  and ADR-043 own those and each becomes a consumer of this payload. Shipping a transport
  inside the contract PR is how the contract acquires a transport's bugs.
- **Runtime behaviour change.** `src/cli/mod.rs` still branches on `requires_confirmation`
  and `blocked_reason`. Wiring `decision` into execution is ADR-020's scope and its own
  argument; this ADR only makes the decision *sayable*.
- **The target axis.** *"Is this command dangerous **against this target**?"* (08-25 scan
  §2.5, opportunity 3.3) is ADR-047's. `Subject.cwd` is carried so the field is reserved,
  but v1 computes nothing from it.
- **Event emission.** Emitting an `Assessment` into an append-only audit log is ADR-041 /
  opportunity 3.4. The payload is designed to be loggable; logging it is not this PR.
- **A second `Evidence` source.** Only the existing bounded `RiskJudgment` path is modelled.
  If a second model-derived signal appears it gets a new variant and a schema minor bump.
- **`FailOpen` code paths.** The variant exists so that a future integration must declare
  it; no v1 code emits it, and a test asserts that.
- **Fixing drifts D-d, D-e, D-f, D-g.** The confidence-width convergence, the
  `SafetyAssessmentOutput` reference removal, the 52→67 doc correction and the ADR-index
  backfill are each a small independent PR and must not ride inside a feature PR
  (`.claude/rules/good-boy-scout.md` — leave it better, don't gold-plate). D-a, D-b and D-c
  *are* in scope because the payload cannot be correct without them.

### 3.7 Demoware trap — what breaks at 100 real users

*Required by `.claude/rules/validation-discipline.md` Gate 3.*

**Assumption that holds at demo scale.** The demo runs `caro assess "rm -rf /"`, gets a
clean `block` with `evidence: deterministic`, and a projection table that maps neatly onto
three hosts. It assumes (i) one assessment per command, evaluated once, by one Caro; and
(ii) the projection targets' vocabularies are what today's docs say.

**How it fails at 100 users.** Both assumptions break, in different directions.

*First failure — the payload gets combined, and combination is where safety leaks.* Hosts do
not run one gate. Claude Code runs every matching hook **in parallel** and combines; a real
deployment will have Caro alongside a corporate policy hook and a logging hook. Caro's
`decision` will be reconciled with verdicts Caro never saw, by a host whose combination rule
Caro does not control. If Caro's decision enum is not `Ord`, every adapter re-implements
"most restrictive wins" by hand and one of them gets it backwards. Worse: the projection is
lossy in exactly the place it matters — `async_log` projects to `allow` in all three hosts,
so a moderate-risk command that Caro said "run it but record it" about becomes an
unqualified `allow` the moment it crosses the boundary, and the recording obligation
evaporates silently.

*Second failure — projection targets drift and the payload keeps asserting stale values.*
Claude Code deprecated `decision`/`reason` at `PreToolUse` and its own onboarding docs
lagged (issue #19944); a fourth value, `defer`, was added; MCP has five open proposals to
extend `ToolAnnotations`. A `projections[]` table baked at Caro build time will confidently
publish a host's *old* vocabulary and the adapter will forward a string that host no longer
accepts — which, per F3, most hosts treat as a **non-blocking error**. A stale projection
does not fail loudly; it degrades to allow.

*Third failure — `evidence` becomes noise if the judge is on by default.* If
`ApprovalMode::Smart` is ever made the default, most payloads carry
`ModelConsultedNoEffect` and readers stop looking at the field, at which point the rare
`Blended` verdict hides in plain sight — the exact fate of Claude Code's
`permissionDecisionReason`.

**Instrumentation that tells us it is breaking.**
- `Projection.lossy` is already in the schema; add a per-projection
  `target_vocabulary_version: String` recording *which documented revision* the mapping was
  built against, so a consumer can compare it to the host it is talking to.
- Emit `projection_stale: bool` from `caro assess --schema` consumers in CI — a scheduled
  job re-fetches each host's documented vocabulary and diffs it against the const table.
- Count `Blended` as a fraction of assessments; if it exceeds a threshold in a user's own
  logs, the judge is doing more work than "bounded" implies.

**Fallback when it triggers.**
- **Lossy projections fail toward restriction, never toward permission.** When a host
  cannot represent `async_log`, the projection value is the host's **`ask`** equivalent, not
  its `allow` — `lossy: true` with a note saying so. Losing a recording obligation must cost
  friction, not safety. This is a v1 schema commitment, not a v2 tuning knob, and T6 asserts
  it.
- **An unknown or unversioned host vocabulary is not projected at all.** The `projections`
  array omits it and `message` names the omission. A missing projection makes an adapter
  fail loudly; a wrong one makes it fail open.
- **`ApprovalMode::Smart` stays non-default in v1.** The contract makes the model's
  contribution visible; it does not make it routine.

Concretely: this is why `target_vocabulary_version` is a **required field in v1** rather
than a v2 addition — which is the whole reason the demoware section belongs in the scope and
not in the retro.

---

## Recommended next step

Do **not** open an implementation PR from this document. In order:

1. **Land drifts D-a, D-b and D-c as one tiny `chore(models):` PR first.** Three derive/attribute
   lines, no behaviour change, wire form provably unchanged. Everything below depends on
   `SuggestedRouting` being `Ord` and `JsonSchema`, and doing it separately keeps the
   contract PR's diff honest.
2. **Run the `devils-advocate` review on ADR-058.** The strongest attack is not about the
   schema, it is about the field this ADR is proudest of: *"a required `evidence` field that
   says 'a model touched this verdict' is a field every competitor will screenshot. You are
   building the disclosure your own marketing has to survive."* The answer — that a bounded,
   disclosed judge with a Critical floor is a stronger claim than an undisclosed one, and
   that Caro's differentiation dies the moment the disclosure is optional — deserves to be
   argued in the PR, not asserted here.
3. **Write T5 and T2 before any serializer code.** *Silence is unrepresentable* and
   *provenance is never silent* are the feature. Write them failing first.
4. **Then the payload, then `caro assess`, then the projection table.** The table is last on
   purpose: it is the part most likely to be wrong, and the part cheapest to change once
   everything else is pinned by golden tests.
5. **Only then unblock the dependants.** Tell ADR-057 §3.6 that `DownMapEntry` is now
   `Projection`; tell ADR-020 what an approver receives; tell ADR-015/038 what
   `assess_command` returns. The 08-25 scan's #1 build item — `caro-mcp` — becomes a
   transport spike over a settled type instead of a spike that invents one.

And independently: **stop citing `SafetyAssessmentOutput`.** ADR-015 has told two other ADRs
to block on a type that has never existed in the tree. Whatever else happens to this scope,
that reference should be corrected so nothing else waits on it.

---

## Sources

Fetched and read 2026-08-25:

- [Claude Code — Hooks reference](https://code.claude.com/docs/en/hooks)
- [Claude Code — Hooks guide](https://code.claude.com/docs/en/hooks-guide)
- [anthropics/claude-code issue #19944 — hooks guide uses deprecated `decision`/`reason`](https://github.com/anthropics/claude-code/issues/19944) (filed 2026-01-22, closed as duplicate)
- [OpenAI Agents SDK — Human in the loop](https://openai.github.io/openai-agents-python/human_in_the_loop/)
- [MCP specification (draft) — Elicitation](https://modelcontextprotocol.io/specification/draft/client/elicitation)
- [MCP specification 2025-11-25 — Tools / `ToolAnnotations`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
- [MCP specification 2025-11-25 — Changelog](https://modelcontextprotocol.io/specification/2025-11-25/changelog)
- [MCP blog — Tool annotations](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/) (2026-03-16, updated 2026-03-18)

Referenced, not re-fetched this run:

- `.hermes/digests/2026-08-25-weekly-agent-market-scan.md` — §2.2, §2.4, §3.1, §3.3, §4, §5
- `.hermes/digests/2026-08-20-agent-market-scan.md` — §3.6 (`caro.assessment.v1`), via ADR-057
- `docs/adr/ADR-015-mcp-safety-server.md`, `ADR-020-tiered-approval-protocol.md`,
  `ADR-023-caro-scan-shell-safety-ci.md`, `ADR-053`, `ADR-055`,
  `ADR-057-measured-fidelity-policy-export.md`
- `caro-scope-policy-export-2026-08-24.md` — `RuleId` / `reason_code` scheme, exit-registry complaint
- Caro tree at `1.4.0`: `src/models/mod.rs`, `src/safety/mod.rs`, `src/safety/patterns.rs`,
  `src/backends/mod.rs`, `src/backends/remote/ollama.rs`, `src/prompts/risk_judge.rs`,
  `src/main.rs`, `Cargo.toml`

Second-hand only, flagged as such: CISA agentic-AI guidance and CoSAI ID-JAG token exchange
(identity standards, not verdict schemas); OneCLI, Pushary and Plow Latch capability claims
(vendor marketing, per the 08-25 digest's own sourcing caveat); the GuardFall 10-of-11
result (third-hand via ADR-057).
