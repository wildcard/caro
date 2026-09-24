# ADR-053: Portable Session Chain Ledger — Order-Aware Multi-Command Risk Detection Without a Daemon

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-18
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: **AgentTrust** — `RiskChain` / `SessionTracker`
  (arXiv:2605.04785, May 2026; open source under **AGPL-3.0** with a
  commercial option; ~5,000 LOC core, 192 unit tests; Python library + CLI +
  MCP server + dashboard). RiskChain is the sub-system that detects
  *multi-step attack chains whose individual steps are each benign*, using
  order-aware greedy matching over a session's action history. Read live
  2026-08-18 against the paper's §3 (threat model), §4.1 (architecture),
  §4.7 (RiskChain), §6.7 (ablation), §7.1 (limitations L1–L7), and
  Appendix E (the seven shipped chain patterns).
- **Depends on**: ADR-024 (headless JSON envelope, exit-code registry,
  additive-field rule), ADR-020 (`SuggestedRouting` vocabulary and the
  risk→routing mapping this ADR escalates *into*, never around)
- **Relates to**: ADR-031 (session circuit breaker — counts bad turns in a
  session; this ADR reads their *order and kind*, and both want the same
  session identity), ADR-051 (intent–effects gate — same "deterministic
  subset of a thing LlamaFirewall/AgentTrust does with an LLM" posture),
  ADR-052 (denial payload — `ChainAlert` is designed to serialize into its
  `violations[]` shape once that lands), ADR-007 lowercase *and*
  ADR-007-ast-parser-shell-validation (the AgentTrust `ShellNormalizer`
  analog is already claimed there and is explicitly **not** re-scoped here),
  ADR-040 (policy file — future home for user-authored chains), ADR-041
  (lifecycle events — future sink for `ChainAlert`), ADR-049 (evidence
  packet — future signer of the ledger)
- **Companion scope document**: `caro-scope-session-chain-2026-08-18.md`
- **Numbering note**: highest existing is ADR-052; per
  `.claude/rules/adr-numbering.md`, renumber on merge if another 053 lands
  first.
- **Exit-code note**: this ADR claims **14**. Codes 10/11/12 are claimed by
  ADR-039/040/044. **13 is double-claimed** by ADR-049 (`EvidenceMismatch`)
  and ADR-051 (`DivergenceBlocked`) — a pre-existing collision found while
  scoping this ADR, filed as a boy-scout note in the companion scope doc,
  not resolved here.

> **Provenance note (autonomous run).** Produced with no user present; the
> task template's `[FEATURE NAME]` was unbound. Target selection: the most
> recent market scan (`market-scans/2026-08-17-ai-agent-strategy-memo.md`)
> lists five opportunities. A→ADR-052 (yesterday's run). B is an eval run
> plus a docs page — no ADR, no types, no exit code — and is **not**
> superseded by this document; it should still be run. C→ADR-037/040/041.
> E is a build spike under `.claude/rules/external-sdk-integration.md`.
> **D — session-level pattern telemetry, "the ExploitGym gap" — is named the
> most defensible long-term moat in the memo and has no ADR behind it.**
> This run scopes D's architecture. The analog (AgentTrust RiskChain) was
> selected because it is the only open-source system found that ships an
> order-aware multi-command detector, and because its published ablation
> quantifies exactly how such a component fails. Treat the `StepKind`
> vocabulary and the seven seed chains as reviewable assumptions.
>
> **Validation-discipline note.** `.claude/rules/validation-discipline.md`
> applies to the *feature spec* for a new user-facing capability class, and
> the memo attaches the gate to D explicitly. This document is an
> architecture scope and makes no PMF claim; per the ADR-040 precedent, the
> five gates (20 transcripts, no-surveys, demoware-trap section,
> devil's-advocate review, defended cohort) attach to the implementing spec
> and PR, **which must not open until they are cleared**. Decision D7 below
> is deliberately constructed so that the evidence gate and the engineering
> gate are the same gate.

---

## 1. Context

### 1.1 The problem, and who has it

Caro assesses **one command at a time**. `SafetyValidator::validate`
(`src/safety/mod.rs`) matches a candidate against 67 `DangerPattern`
literals in `src/safety/patterns.rs` and returns a `ValidationResult`, which
`SafetyDecision::from_validation_result` (`src/safety/mod.rs:225`) maps into
a `RiskLevel` and a `SuggestedRouting`. Nothing in that path can see a
second command, let alone the order of a dozen.

That is structurally blind to the attack shape that matters most for
long-running autonomous sessions: a sequence in which **every individual
step is genuinely benign** and only the composition is dangerous. Read a
config file (fine). Base64 it (fine). POST it somewhere (fine). Caro
approves all three, correctly, one at a time, and the aggregate is
exfiltration. The August 2026 incident that made this concrete involved
roughly 17,600 actions in ~6,280 clusters over four days, where the
successful path hid inside thousands of individually unremarkable failures.

A per-command validator cannot get this right by adding more patterns. It
needs *memory of order*.

### 1.2 The analog: AgentTrust RiskChain

AgentTrust (§4.7) formalizes a chain pattern as
`P = (id, n, [s₁…s_k], r_P, m)`: k ordered steps, each
`sᵢ = (τᵢ, regexᵢ, descrᵢ)` where `τ` is an action type; a combined risk
`r_P`; and a minimum match count `m`. Matching walks the steps in order,
scanning forward through the session history for the earliest unused action
matching `τᵢ` and `regexᵢ`, greedily assigning a matching action to the
**lowest-indexed unmatched step**. When ≥ m steps match, a `ChainAlert` is
emitted and `r_P` is merged into the candidate action's risk by maximum
severity (their Algorithm 1, lines 10–13). Seven chains ship by default:
data exfiltration, credential harvesting, persistence installation,
privilege escalation, supply-chain attack, reverse shell, data destruction.

Two properties of that design are simply correct and we should replicate
them:

- **Deterministic tie-breaking.** Lowest-indexed unmatched step, no action
  reused across two steps. The paper's own justification — "making detection
  results bit-for-bit reproducible" — is the same reason Caro rejects
  LLM-judged safety.
- **Monotone escalation.** A chain hit raises risk by `max`; it never lowers
  it. A `BLOCK` cannot be argued down by chain evidence.

### 1.3 The failure modes we must not inherit

**F1 — The history is in-process memory, so it cannot cross a subprocess
boundary.** §4.1: *"All components are stateless except the SessionTracker,
which holds an in-memory action history per session."* Limitation **L6**
goes further: *"The interceptor must be in-process… out-of-process
enforcement requires a different deployment model."* Caro is a CLI. Every
invocation is a fresh process. Ported naively, RiskChain would detect
nothing at all — every session would be one action long. **This is the
single design problem this ADR exists to solve.**

**F2 — The component ships unmeasured, and the published ablation says so.**
§6.7: removing `SessionTracker` leaves verdict accuracy **identical** —
95.0% internal, 95.4% independent, a 0.0 pp delta on both — while dropping
end-to-end median latency from **1.77 ms to 0.22 ms**. The chain tracker
accounts for roughly **87% of end-to-end latency and 0% of measured
accuracy**. That is not evidence the idea is wrong; it is evidence that
*both benchmarks are corpora of single actions*, so the one component whose
entire purpose is multi-action reasoning has nothing to be scored against.
A capability with no corpus is a capability with no error rate, and a safety
control with no measured false-positive rate is one users eventually
disable. Caro must not ship a chain matcher on the same terms.

**F3 — Unbounded session history.** The paper bounds the LLM-judge cache
explicitly (`MAX_ENTRIES`, `TTL_SECONDS`) and documents no bound at all on
the SessionTracker's per-session action history. In a four-day, 17,600-action
run that is the difference between a working control and an OOM.

**F4 — Persisting a history means persisting the commands.** AgentTrust's
`Action` (Definition 1) carries `ρ`, the raw payload — command text, URL,
query. Keeping that in memory in the agent's own process is one privacy
posture; writing it to disk so a *different* process can read it is a much
worse one. Any port to a CLI turns an in-memory structure into an artifact
on a filesystem, and the naive port writes the user's full shell history,
credentials-in-argv included, into a new file nobody asked for.

**F5 — Ambiguity is resolved by an LLM.** Their Algorithm 1 line 14 routes
ambiguous rule outcomes to a hosted LLM judge (~200–600 ms, API key, cost).
Explicitly out of scope for Caro: it is against strategy, it destroys the
offline/zero-latency/reproducible property, and — per L1 — it is the layer
that exists because static analysis has a ceiling, not because chains need it.

---

## 2. Decision

Add an **order-aware, deterministic chain matcher to `src/safety/`, whose
session history is a caller-held, serializable, bounded, fingerprint-only
artifact passed in and out of each invocation.** Caro remains a pure
function of `(command, ledger) → (verdict, ledger′)`. No daemon, no ambient
state, no background process, no network.

Eight decisions define it.

### D1 — The ledger is an argument, not a service

A new opt-in flag `--session-ledger <PATH|->` names a JSON file (or `-`, for
JSON on stdin with the updated ledger returned inside the envelope). Caro
reads it, matches, appends one entry, writes it back, exits. If the flag is
absent the feature is entirely inert and output is byte-identical to today.

This is the direct answer to **F1**. AgentTrust's tracker cannot survive a
process boundary because its history is a live object; Caro's survives
because its history is a *value*. It is also the answer to the template's
"how does it avoid redundant initialization?" — the ledger is capped at 256
entries (~40 KB), matching is pure regex plus enum comparison, and **no
model or backend is loaded on the chain path**. The marginal cost of chain
evaluation is a small file read and a linear scan.

### D2 — The ledger stores fingerprints, never payloads

A `LedgerEntry` records a `StepKind` (a closed, coarse vocabulary), a
`sha256:` digest of the *normalized* payload salted with a per-ledger random
salt, the `RiskLevel`, whether it was blocked, and a timestamp. It never
records command text, arguments, paths, hosts, or environment.

Matching does not need reversibility — a chain step is matched by kind plus
the *candidate's* own regex evaluation, not by re-reading history. The salt
prevents the ledger from becoming a rainbow table of "did this machine ever
run X". This is the answer to **F4**, and it is what makes the ledger safe
to hand to another process, ship in a bug report, or mount in CI.

### D3 — Chain patterns are static data with the same discipline as `DangerPattern`

`ChainPattern { id, steps, combined_risk, min_match }` lives in a new
`src/safety/chain_patterns.rs`, mirroring the existing
`src/safety/patterns.rs` split between data and matcher. Seven seed chains,
each with a stable `id` of the form `chain/<slug>`, validated at startup by
the same `validate_patterns()` discipline that already guards the 67 danger
patterns.

### D4 — Escalation is monotone and routes through the existing mapping

`SafetyDecision::escalate_with_chains(&mut self, &[ChainAlert])` sets
`risk_level = max(risk_level, max alert.combined_risk)` and then recomputes
routing through the **existing** `SuggestedRouting::from_risk_and_safety`
(`src/models/mod.rs:189`). A chain can raise a verdict; it can never lower
one, and it never invents a routing value. Reuse over duplication, per the
task's standing constraint.

### D5 — A chain hit is a first-class field, shaped for ADR-052

`chain_alerts: Vec<ChainAlert>` is added to `CliResult` (`src/cli/mod.rs:76`)
and to the headless envelope as an **additive** field under ADR-024's rule —
no `schema_version` bump, absent-and-empty are the same thing. `ChainAlert`
carries `rule_id`, `combined_risk`, `matched_steps` (the ledger sequence
numbers, in order), `total_steps`, and `description`. Those field names are
chosen so that when ADR-052's `violations[]` lands, a `ChainAlert` maps onto
a `Violation` with `rule_id = "chain/data-exfiltration"` and a remediation
class, rather than becoming a second, parallel denial channel.

### D6 — The ledger is bounded, and eviction is deterministic

`MAX_ENTRIES = 256` (ring; oldest evicted first) and `MAX_AGE = 8h`
(entries older than the window are pruned on load). `seq` is monotonic and
never reused, so evicted history is visibly absent rather than silently
renumbered. This is the answer to **F3**, and it bounds the worst case at a
known file size for a session of any length.

### D7 — The corpus gates the default, and the gate is the same gate

A new `EvalCategory::ChainSequence` and an ordered-scenario harness land
**before** the matcher is enabled by default. Concretely: `--session-ledger`
ships opt-in-only until a session-level corpus exists with (a) positive
chains, (b) *benign lookalike negatives* for each — the same discipline
AgentTrust used for single actions, where `cat README.md` sits next to
`cat .env` — and (c) published recall and false-positive rates.

This is the answer to **F2**, and it is deliberately the same artifact the
validation-discipline gate needs: the negatives cannot be written without
knowing what real long sessions actually look like, which is what the 20
transcripts are for. One piece of work clears both gates.

### D8 — Deterministic only; no judge, no network, no daemon

No LLM in the chain path, ever. Ambiguity resolves *upward* (toward
`HumanGate`), never toward a hosted opinion. This preserves the property
that makes Caro worth layering under a classifier at all: reproducible,
offline, zero-latency, prompt-injection-proof.

### Fail-closed on a malformed ledger

A ledger that fails to parse, fails schema validation, or has non-monotonic
`seq` exits **2** with `error.kind = "ledger_invalid"`. It is *not* treated
as an empty ledger: silently starting from zero is precisely the behaviour
an attacker would want from a file they can corrupt. No new exit code is
claimed for this — it is a usage error, and ADR-024's code 2 already means
that.

---

## 3. Consequences

### Positive

- Caro gains the one capability that per-command validators structurally
  cannot have, in the form that fits a CLI: a value, not a service.
- The chain path adds no dependency, no network, no model load, and no
  daemon. `cargo build` gains two files.
- Opt-in by construction: with no `--session-ledger`, output is byte-identical
  to today, so the change cannot regress any existing consumer.
- The ledger is portable — an agent harness, a CI job, and a hook adapter can
  all pass the same artifact, which is what makes ADR-036/043/048 hosts able
  to inherit chain detection without their own state.
- It gives ADR-031's circuit breaker the session identity it also needs, and
  gives ADR-049's evidence packet something ordered to attest.

### Negative / risks

- **False positives are the real cost.** "Read a file, encode it, make a
  network call" is also the shape of a legitimate deploy script. D7 exists
  because the FP rate, not the recall, decides whether this ships.
- **The caller can drop the ledger.** A partially-trusted agent that forgets
  (or declines) to pass `--session-ledger` gets no chain detection. This is
  the same boundary as AgentTrust's L6 and is inherent to any control the
  callee cannot enforce; the threat model is *partially-trusted agent*, not
  *malicious runtime*. Say so plainly in the docs rather than implying
  coverage we do not have.
- **`StepKind` is a new closed vocabulary**, and closed vocabularies are
  expensive to change once serialized. Mitigated by `Other` as the
  catch-all and by the ADR-024 additive rule, but it is a real commitment.
- **Fingerprint-only history limits future features.** Anything that later
  wants to *explain* a chain in terms of the actual commands cannot, by
  construction. That is the intended trade (D2); it should be a conscious
  one.
- Two contested exit codes already exist at 13; this ADR claims 14 and adds
  one more number to a registry that now needs an owner.

### Neutral

- Latency: AgentTrust measures the analogous path at ~1.5 ms end-to-end for
  a tracker over an in-memory history. Caro's adds a file read. It is not
  free and it is not interesting; it should still be measured, not assumed.

---

## 4. Alternatives considered

**A. Port RiskChain as-is (in-process, in-memory).** Rejected: Caro has no
process to be in. This is F1, and there is no version of it that works for
a subprocess CLI.

**B. Run a background daemon that holds session state.** Rejected: violates
the standing constraint ("pure subprocess call, no daemon, no state"), adds
an attack surface, an IPC contract, a lifecycle, and a platform matrix, and
makes `caro` un-runnable in the CI and container contexts where it is most
useful.

**C. Reuse `~/.caro/state/<intent_hash>/journal.jsonl` (`src/caroml/history.rs`).**
Rejected for v1, and the reasoning is worth recording: the journal is real,
append-only, serializable, and already privacy-conscious (it stores a
`stderr_digest`, not stderr — the same instinct as D2). But it is keyed by
**intent hash**, deliberately: "the local history follows the *intent*, not
the file path." Chain detection needs the opposite key — one agent session
across many different intents. Overloading the journal would break its
documented contract and couple a safety control to the `caroml` run
subsystem. Its *design*, however, is the model D2 copies.

**D. Reuse `AiSession` / `SessionStore` (`src/ai/session.rs`, `src/ai/store.rs`).**
Rejected as the transport, adopted as prior art. `SessionStore` is a
single JSON file of ordered `AiSession`s with `Turn { role, content,
command, confidence, risk, ts }` — genuinely close. But it stores **raw
command text** (F4), is keyed by a machine-local monotonic `u64` rather than
a caller-supplied session identity, lives at a fixed
`$XDG_DATA_HOME` path rather than being passed in (F1), and is scoped to the
interactive `caro ai` loop. A headless agent must be able to hand Caro *its*
session, from any directory, without inheriting the interactive feature's
store. The chain ledger is the headless-safe sibling, and the two should
converge later, not now.

**E. Infer the session from the parent process / TTY / env var.** Rejected:
ambient state by another name. It is unreproducible in tests, breaks under
process supervisors and containers, and makes the verdict depend on
something not present in the arguments — the opposite of the contract
ADR-024 sells.

**F. Do the composition analysis with an LLM judge, as AgentTrust does for
ambiguity.** Rejected per D8 and the standing strategy line about fragile
LLM-only safety dependencies. Noted as the honest limit: some chains are
only visible with data-flow reasoning (their L1), and Caro will miss those.
Claim the deterministic floor; do not claim the ceiling.

**G. Ship the matcher on by default and gather the corpus afterward.**
Rejected — this is exactly F2, and it is how a safety control acquires an
unmeasured false-positive rate and a reputation for being the thing you turn
off.
