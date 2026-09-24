# ADR-031: Session Circuit Breaker — Deterministic Halt-on-Tier for Agentic Sessions

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-06
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: OpenAI Agents SDK guardrail **tripwires**
  (`InputGuardrailTripwireTriggered` / `OutputGuardrailTripwireTriggered`)
  and Claude Agent SDK **hooks** (`PreToolUse` deny, `Stop`), plus the
  Trust3 × Copilot Studio runtime kill switch (2026-06-30)
- **Depends on**: ADR-020 (tiered approval), ADR-024 (headless JSON
  contract), ADR-026 (multi-turn agentic session)
- **Relates to**: Hermes memos 2026-07-02 (opportunity A: "stop/override
  primitive") and 2026-07-03 (recommendation 1: "ship the session circuit
  breaker")

---

## Context

### The market problem

Caro gates **individual commands**: one invocation, one verdict
(`SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }`,
`src/models/mod.rs:189`). It has no first-class notion of *"this session is
going badly — stop the whole run."* An agent driving caro in headless mode
(ADR-024/026) can be blocked on turn after turn and simply keep trying
variants until something slips through the patterns. Enforcement buyers now
evaluate agent-safety products against a kill-switch checklist (Trust3 ×
Copilot Studio ships discovery → execution history → runtime gate → kill
switch; Hermes 2026-07-02 memo, item 2). Caro satisfies the runtime-gate box
but not the kill-switch box.

### Phase 1 — how the OSS reference implementations work

**OpenAI Agents SDK guardrails** (openai-agents-python). A guardrail is a
user-supplied function returning `GuardrailFunctionOutput { output_info,
tripwire_triggered: bool }`. Input guardrails run on the *first* agent of a
run; output guardrails on the *last*. When `tripwire_triggered == true` the
runner raises `InputGuardrailTripwireTriggered` /
`OutputGuardrailTripwireTriggered` and the run is supposed to halt.
Crucially, the SDK uses **optimistic execution**: the agent proceeds
*concurrently* while guardrails evaluate.

Documented failure modes (their tracker, verified 2026-07-06):

1. **The halt is advisory, not enforced** — issue **#991** (and #889):
   after the tripwire exception is raised, in-flight tool calls *still
   execute*, and handoffs still trigger the next agent. The exception
   unwinds the caller's stack, but the work the guardrail existed to prevent
   happens anyway. This is a direct consequence of running the guardrail
   concurrently with the guarded work.
2. **Streaming leaks ahead of the verdict** — issues #495/#300: output
   guardrails evaluate the final output, so with streaming the user has
   already seen the content before the tripwire fires.
3. **No structured contract for subprocess callers.** The tripwire surfaces
   as a Python exception. There is no exit code, no JSON payload, no event.
   A non-Python orchestrator (or any shell script) cannot observe "the run
   was halted and why" without wrapping the interpreter.
4. **No cumulative session memory.** Guardrails are stateless per run:
   "3 high-risk attempts in this session" is not expressible without the
   caller hand-rolling state.

**Claude Agent SDK hooks.** `PreToolUse` fires **before** the tool executes
and a `permissionDecision: "deny"` deterministically blocks it — even under
`bypassPermissions`. This gets the ordering right (deny happens strictly
before execution). But the session-level policy is imperative: if you want
"halt after N denials", you write a hook that keeps its own counter in
external state and wires a `Stop` yourself. There is no declarative policy
surface, and the counter's persistence/atomicity is the integrator's problem.

**Trust3 kill switch.** Full lifecycle (tamper-evident history → runtime
guardrails → kill agents in real time), but platform-bound to Copilot
Studio — not agent-agnostic, not offline, not a subprocess.

### Phase 2 — what caro already has

| Capability | Where | Status |
|---|---|---|
| Per-command risk verdict | `RiskLevel { Safe, Moderate, High, Critical }` (`src/models/mod.rs:152`) | Shipped |
| Canonical routing | `SuggestedRouting::from_risk_and_safety` (`src/models/mod.rs:198`) | Shipped |
| Policy dial | `SafetyLevel { Strict, Moderate (default), Permissive }` (`src/models/mod.rs:247`) | Shipped |
| Persistent session state | `AiSession { id: u64, turns: Vec<Turn>, … }` (`src/ai/session.rs:60`), `SessionStore::{open, resume_recent, create, upsert, get(u64)}` (`src/ai/store.rs`) | Shipped |
| Headless contract + exit codes | `HeadlessEnvelope`, `ExitCode 0–6` (ADR-024); `7 = NeedsApproval` reserved (ADR-027); `30/31/32` reserved (ADR-030) | Scoped |
| Multi-turn agentic loop | ADR-026 (`Blocked` turn status → exit 3 halts the *conversation loop*) | Scoped |
| Per-turn blocking | Safety validator (`src/safety/`) | Shipped |
| Config surface | `config.toml` (`src/config/mod.rs:75`); ADR-020 adds `[approval]` | Shipped/Scoped |

What is missing is exactly the delta between "block this turn" and "trip
this session": cumulative counters, a persisted tripped state, a declarative
trip policy, and a wire/exit-code contract for the halt.

### Our unique positioning

- **Deterministic and synchronous by construction.** Caro's pipeline is
  sequential; there is no optimistic concurrency to race against (the exact
  failure of openai-agents #991).
- **Pure subprocess.** The breaker state lives in the session file, not a
  daemon. Any orchestrator in any language gets the halt as an exit code +
  JSON payload.
- **Agent-agnostic.** Works identically for a Claude agent, an OpenAI agent,
  a cron job, or a human — the counter-position to platform-bound kill
  switches ("someone deterministic has to check the checker", Hermes
  2026-07-03).
- **Offline.** No control plane required.

---

## Decision

Add a **declarative session circuit breaker**: a `[circuit_breaker]` policy
in `config.toml`, cumulative per-session counters persisted in `AiSession`,
and a **synchronous pre-dispatch evaluation** in the turn pipeline. When the
breaker trips, the session is durably marked halted; the current and all
subsequent invocations against that session id refuse to generate or execute
anything until the halt is explicitly acknowledged.

### Design principle (solves failure mode #1 by construction)

The breaker check is a **blocking step in the single-threaded turn
sequence**:

```
resolve session → breaker.check (tripped? → refuse, exit 8)
→ generate → safety-validate → record verdict counters
→ breaker.evaluate (tripped now? → persist trip, emit event, exit 8, DO NOT dispatch)
→ route/execute
```

Nothing is dispatched concurrently with evaluation, and the tripped flag is
persisted via the existing `SessionStore::upsert` **before** the process
exits — so a parallel or subsequent invocation of the same session observes
the trip. There is no code path in which execution continues after a trip,
because the trip decision *precedes* dispatch rather than racing it.

### New types (all in existing modules; serde from day one)

`src/models/mod.rs`:

```rust
/// Declarative trip policy. Loaded from [circuit_breaker] in config.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "snake_case")]
pub struct BreakerPolicy {
    /// Master switch. Default true in headless/session mode; ignored for
    /// one-shot interactive invocations (no session ⇒ no breaker).
    pub enabled: bool,            // default: true
    /// Trip immediately on the first Critical verdict.
    pub trip_on_critical: bool,   // default: true
    /// Trip when High-risk verdicts in the session reach this count. 0 = off.
    pub max_high: u32,            // default: 3
    /// Trip when Block routings in the session reach this count. 0 = off.
    pub max_blocked: u32,         // default: 3
}

/// Cumulative per-session verdict counters. Monotonic; never reset except
/// by explicit acknowledgment.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BreakerCounts {
    pub turns: u32,
    pub high: u32,
    pub critical: u32,
    pub blocked: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BreakerReason {
    CriticalVerdict,
    HighRiskBudgetExhausted,
    BlockedBudgetExhausted,
    ManualHalt, // reserved for `caro session halt <id>` (out of scope v1 CLI)
}

/// Persisted trip record. Presence on a session == the session is halted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BreakerTrip {
    pub reason: BreakerReason,
    pub tripped_at: chrono::DateTime<chrono::Utc>,
    pub counts: BreakerCounts,
    /// The command text that tripped the breaker (never executed).
    pub tripping_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BreakerDecision {
    Proceed,
    Tripped(BreakerTrip),
}
```

Method contracts:

- `BreakerPolicy::evaluate(&self, counts: &BreakerCounts, last_risk: RiskLevel, last_routing: SuggestedRouting, command: Option<&str>) -> BreakerDecision`
  — **pure function**, no IO, no clock reads besides the timestamp inside
  the constructed `BreakerTrip`. Precedence when multiple conditions fire on
  the same turn: `CriticalVerdict` > `BlockedBudgetExhausted` >
  `HighRiskBudgetExhausted` (fixed, documented, tested).
- `Default for BreakerPolicy` — the values above; a missing
  `[circuit_breaker]` section behaves identically to defaults.

`src/ai/session.rs` — extend `AiSession` (backward compatible; old session
files deserialize with `serde(default)`):

```rust
#[serde(default, skip_serializing_if = "is_default")]
pub breaker_counts: BreakerCounts,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub breaker: Option<BreakerTrip>,
```

- `AiSession::record_verdict(&mut self, risk: RiskLevel, routing: SuggestedRouting)`
  — increments `breaker_counts` from the **typed** `RiskLevel`, not from
  `Turn.risk` (which is `Option<String>`, `src/ai/session.rs:28-30` — a
  stringly-typed display field the breaker must never parse; see
  "Findings" below).
- `AiSession::is_halted(&self) -> bool` — `self.breaker.is_some()`.
- `AiSession::acknowledge_halt(&mut self)` — clears `breaker` **and**
  resets `breaker_counts`; only called on the explicit acknowledgment path.

### Wire contract

- **Exit code**: `SessionHalted = 8` added to ADR-024's `ExitCode` registry
  (`0–6` taken by ADR-024, `7` by ADR-027, `30/31/32` by ADR-030, `201`
  legacy `EXIT_CODE_EDIT`). Returned both when a turn trips the breaker and
  when an already-halted session is invoked again.
- **Envelope**: `HeadlessStatus` gains `"halted"`; the envelope carries
  `breaker: Option<BreakerTrip>` (present iff halted), `command` = the
  tripping command (never executed), and the usual `safety`/`risk_level`
  fields of the tripping verdict. `schema_version` bumps per ADR-024's rule.
- **NDJSON event** (ADR-026 stream taxonomy, `#[serde(tag = "type",
  rename_all = "snake_case")]`): a `session_halted` event —
  `{"type":"session_halted","session_id":<u64>,"reason":"critical_verdict","counts":{…},"tripped_at":"…"}`
  — emitted once at trip time, before the final `result` event. Session id
  is **`u64`** on the wire (per the v2 headless scope correction;
  `AiSession.id: u64`, `src/ai/session.rs:62`).
- **Acknowledgment**: `--acknowledge-halt` flag valid only together with
  `--session-id <u64>`; clears the trip, resets counters, logs an
  acknowledgment turn into the session, then proceeds with the prompt (or
  exits 0 if no prompt). Re-arming is deliberate and auditable — mirrors
  the Claude SDK property that a deny survives permissive modes: **no
  safety level, flag, or env var other than explicit acknowledgment clears
  a trip** (`SafetyLevel::Permissive` does not bypass the breaker).

### Config surface (`config.toml`, next to ADR-020's `[approval]`)

```toml
[circuit_breaker]
enabled = true
trip_on_critical = true
max_high = 3
max_blocked = 3
```

### Minimal file set (no new modules)

| # | File | Change |
|---|------|--------|
| 1 | `src/models/mod.rs` | `BreakerPolicy`, `BreakerCounts`, `BreakerReason`, `BreakerTrip`, `BreakerDecision` |
| 2 | `src/ai/session.rs` | `AiSession.breaker_counts` / `.breaker`; `record_verdict`, `is_halted`, `acknowledge_halt` |
| 3 | `src/config/mod.rs` | `[circuit_breaker]` section → `BreakerPolicy` |
| 4 | `src/cli/mod.rs` | `HeadlessStatus::Halted`, envelope `breaker` field, `session_halted` event variant, `--acknowledge-halt` flag |
| 5 | `src/agent/mod.rs` | `AgentLoop`: pre-dispatch `breaker.check` + post-verdict `evaluate` + `upsert` before exit |
| 6 | `src/main.rs` | exit-code wiring for `8` |
| 7 | `tests/session_breaker.rs` | new integration test file (tests/ already hosts flat integration files) |

### Integration tests (deterministic input → fixed JSON + exit code)

All use the static-matcher/mock backend (no LLM), a temp session store path,
and assert **both** the parsed envelope and the process exit code:

1. **Critical trips immediately.** One prompt matching a Critical pattern →
   exit `8`, `status:"halted"`, `breaker.reason:"critical_verdict"`,
   `breaker.counts.critical == 1`; session file contains the persisted trip.
2. **High-risk budget.** Three prompts matching High patterns in one
   session → turns 1–2 exit `3`/`4` per existing contract, turn 3 exits `8`
   with `reason:"high_risk_budget_exhausted"`.
3. **Halted session refuses fast.** Invoke a tripped session id → exit `8`
   with `breaker` echoed, **no backend initialization** (assert via mock
   backend call-count = 0; this is the warm-path guarantee — refusal reads
   only the session file).
4. **Acknowledgment re-arms.** `--session-id <id> --acknowledge-halt "list files"`
   → exit `0`, `status:"ok"`, counters reset, acknowledgment recorded.
5. **Disabled policy.** `enabled = false` → the Critical prompt from test 1
   exits `3` (per-turn block still applies); breaker never trips.
6. **Old session files load.** Fixture session JSON without breaker fields
   deserializes; counts default to zero.
7. **Trip precedes dispatch.** A High pattern whose count trips the budget
   is *also* executable under `--execute`; assert the executor was never
   invoked on the tripping turn (the anti-#991 test).

### Out of scope (v2+)

- Sliding time windows / decay for counters (v1 counters are whole-session).
- Cross-session or global breaker ("halt everything this agent does").
- `caro session halt <id>` manual-halt CLI verb (`BreakerReason::ManualHalt`
  is reserved so the schema doesn't break when it lands).
- `agent_id` identity field on events/audit rows (Hermes 2026-07-03 item D —
  separate scope; the `session_halted` event schema should leave room).
- Remote kill-switch API / fleet control plane.
- LLM-judged trip conditions (e.g. "intent drift") — the breaker is
  deterministic by principle; a probabilistic judge may *feed* a verdict but
  never *be* the breaker.
- TUI/interactive-mode breaker UX beyond a plain refusal message.

---

## Consequences

**Positive.** Caro ticks the kill-switch checklist while staying a pure
subprocess; the halt is observable by any language via exit 8 + JSON; the
#991 class of bug is structurally impossible (evaluation precedes dispatch,
single-threaded); old session files remain valid; `Permissive` cannot bypass
a trip, mirroring the strongest property of Claude SDK's `PreToolUse` deny.

**Negative / accepted costs.** One more exit code for integrators to handle
(mitigated: exit-code registry + schema bump are documented in one place per
ADR-024). Sessions accumulate counters forever until acknowledged — a
long-lived benign session with occasional High verdicts will eventually trip
(accepted for v1; sliding windows are the v2 answer). The breaker only
protects *session-scoped* usage; one-shot invocations without `--session-id`
get no cumulative protection (documented; agents wanting the breaker must
pass a session id — which ADR-025/026 already push them toward).

**Neutral.** `BreakerReason::ManualHalt` and the event schema reserve room
for the manual verb and `agent_id` without breaking changes.

## Alternatives considered

1. **Exception/abort-style halt (OpenAI model).** Rejected: no structured
   contract for subprocess callers, and their own tracker (#991) shows the
   race between advisory halt and real work.
2. **Imperative hook (Claude SDK model).** A `--breaker-hook <cmd>` that
   keeps its own counters. Rejected for v1: pushes state, atomicity, and
   policy onto every integrator; caro already owns durable session state, so
   declarative policy + owned counters is strictly less integration burden.
   (ADR-020's `--approval-hook` remains the right shape for *approval*,
   which genuinely needs external judgment; *counting* does not.)
3. **Daemon/control-plane kill switch (Trust3 model).** Rejected: violates
   the pure-subprocess constraint and caro's offline positioning.
4. **Reuse exit 3 (`Blocked`) for trips.** Rejected: orchestrators must
   distinguish "this turn was blocked, keep going" from "stop driving this
   session"; conflating them reproduces the ambiguity this feature removes.
5. **Store breaker state outside the session file** (separate halt ledger).
   Rejected: two sources of truth for one session's lifecycle; `AiSession` +
   `SessionStore::upsert` already provide atomic-enough persistence for the
   single-writer CLI model.

## Findings logged for other sessions

- **`Turn.risk` is `Option<String>`** ("LOW/MEDIUM/HIGH/CRITICAL",
  `src/ai/session.rs:28-30`) while the canonical type is `RiskLevel
  { Safe, Moderate, High, Critical }` — the labels don't even match
  (`LOW/MEDIUM` vs `Safe/Moderate`). The breaker deliberately counts from
  the typed verdict, never by parsing this string. A follow-up should
  migrate `Turn.risk` to `Option<RiskLevel>` with a serde alias shim.
- Validation-discipline gates (`.claude/rules/validation-discipline.md`):
  this scope extends the shipped safety pipeline (infrastructure, not a new
  product line), matching the "extends caro-core, exempt" precedent of
  ADR-018/019/030 — but Gate 4 (devil's-advocate review) is cheap and
  recommended on the spec PR since this ADR is AI-drafted.

## References

- OpenAI Agents SDK guardrails: https://openai.github.io/openai-agents-python/guardrails/
- openai-agents-python #991 (tools execute after tripwire), #889, #495, #300
- Claude Agent SDK hooks: https://code.claude.com/docs/en/agent-sdk/hooks
- Hermes memos: `.hermes/digests/2026-07-02-agent-market-scan.md` (opportunity A),
  `2026-07-03-agent-market-scan.md` (recommendation 1)
- ADR-020, ADR-024 (exit-code registry), ADR-026 (session loop), ADR-027 (exit 7), ADR-030 (exit 30–32)
