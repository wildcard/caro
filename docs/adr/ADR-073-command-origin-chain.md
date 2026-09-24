# ADR-073: `caro.origin.v1` — Judge the Command by How It Got Here, Not Only by What It Says

- **Status**: **Proposed (implementation ADR — buildable on the tree that ships in 1.4.0; no
  unlanded ADR is a prerequisite).** Gate 1 unmet, Gate 4 pending. See *Validation discipline*.
- **Date**: 2026-09-17
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: the **Claude Code auto mode permission classifier**, and specifically the
  **v2.1.271 (2026-09-14)** pair of changes that moved two categories of command text out of
  implicit, pre-authorised lanes and into reviewed tool calls. All sources read 2026-09-17:
  - [`code.claude.com/docs/en/permission-modes`](https://code.claude.com/docs/en/permission-modes) — gate ordering, circuit-breaker thresholds
  - [`code.claude.com/docs/en/auto-mode-config`](https://code.claude.com/docs/en/auto-mode-config) — `autoMode.*` rule lists, `classifyAllShell`
  - [`code.claude.com/docs/en/skills`](https://code.claude.com/docs/en/skills) — the inline `` !`command` `` injection lane and its permission checks
  - [`code.claude.com/docs/en/changelog`](https://code.claude.com/docs/en/changelog) — v2.1.69, v2.1.74, v2.1.91, v2.1.178, v2.1.271
  - [`anthropic.com/engineering/claude-code-auto-mode`](https://www.anthropic.com/engineering/claude-code-auto-mode) (2026-03-25) — classifier architecture and its stated blind spot
  - [`claude.com/blog/auto-mode-default-in-claude-code`](https://claude.com/blog/auto-mode-default-in-claude-code) (2026-08-07) — vendor's own limitation statement
  - [embracethered.com — *Breaking Claude Code Opus 5 Auto Mode*](https://embracethered.com/blog/posts/2026/breaking-claude-code-opus-5-and-automode/) (Johann Rehberger, 2026-08-26), corroborated by [simonwillison.net](https://simonwillison.net) 2026-08-27
  - [manifold.security — GitSpawn](https://manifold.security/blog/ai-coding-agents-git-hijack) (2026-09-01)
- **Depends on**: nothing. No new module, no new dependency, no new subcommand, no new exit code.
- **Relates to**: ADR-021 (execution attribution — *retrospective* origin, in shell history; this ADR
  is *prospective* origin, as a validation input), ADR-042 (principal-aware policy — *who* is calling;
  this ADR is *through what path the text arrived*), ADR-058/059 (`caro.assessment.v1` — the payload
  this field will eventually ride in, still unwritten in code), ADR-067 (`caro.decompose.v1` —
  execution sites *inside* a command; this ADR is authoring hops *before* it), ADR-072
  (`caro.annotate.v1` — the same untrusted-self-declaration problem, solved with the same
  restrict-only invariant), ADR-069 (`caro.preflight.v1` — ambient config, the one origin class this
  ADR deliberately does not model)
- **Full scope document**: [`caro-scope-command-origin-2026-09-17.md`](../../caro-scope-command-origin-2026-09-17.md)

> **Provenance note (autonomous run).** The scheduled task's template leaves `[FEATURE NAME]`
> unbound and ran with no user present, so target selection is the agent's own and is a reviewable
> decision. Rationale: a duplication sweep of ADR-001…072 finds the **authoring path of the command
> string** covered zero times — ADR-021 tags commands *after* they run, ADR-042 reserves schema for
> *who* invokes, ADR-070 classifies which *argument* is a program, ADR-067 enumerates execution sites
> *within* one string. Nothing asks where the string came from before Caro saw it. Meanwhile the
> researched vendor has shipped **five separate fixes to exactly that axis in six months**
> (v2.1.69, v2.1.74, v2.1.91, v2.1.178, v2.1.271), which is the strongest available signal that the
> axis is real and not yet settled by anyone.
>
> **Source correction carried forward.** ADR-072 cites "Black Hat USA 2026 (Elad Meged / Novee)" and
> CVE-2026-12537 / CVE-2026-54316. A dedicated search on 2026-09-17 could locate **neither the
> conference item nor the vendor named "Novee"**. Those citations should be re-verified before
> ADR-072 leaves Proposed. This ADR cites nothing from that cluster.
>
> **Discarded framing.** Several third-party write-ups describe the classifier as scoring five
> dimensions (Reversibility / Blast Radius / Security Impact / Performance / Stealth-Persistence).
> No Anthropic-authored source uses that vocabulary; it appears only in reverse-engineered
> system-prompt reconstructions. It is **not** used anywhere in this ADR.

---

## Context

### 1. What the researched feature does

Claude Code's auto mode replaces the human approve/deny prompt with a second model that reviews each
tool call. The gate order is fixed and documented: `permissions.deny` blocks first and *"neither the
classifier nor user intent can override it"*; content-scoped `permissions.ask` prompts next; a narrow
built-in allowlist passes reads and searches; and everything else — shell commands, fetches, MCP
calls, subagent spawns — reaches the classifier. On entering auto mode the host **drops the user's own
permission rules that grant arbitrary code execution** (`Bash(*)`, wildcarded interpreters,
package-manager run commands) precisely so those rules cannot be used to skip the classifier.

### 2. The design choice this ADR is about

The classifier is **deliberately blind to where the text came from**. From the 2026-03-25 engineering
post: *"We strip assistant text so the agent can't talk the classifier into making a bad call… the
classifier is reasoning-blind by design… sees only user messages and the agent's tool calls."* Tool
outputs are stripped too. The vendor states the cost of that choice in the same document:

> *"If the user never named 'job abc123,' the classifier can't tell whether the agent pulled it from
> a query or fabricated it. We accept that cost for injection robustness."*

That is the trade this ADR takes the other side of. Blinding the judge to provenance is the correct
defence against *injection into the judge*. It is not a defence against *indirection*, and the vendor
does not claim it is.

### 3. The failure mode, stated as the changelog states it

Because the classifier cannot see provenance, trust class is instead decided by **which host-internal
lane the text arrives in**. Those lanes are private to the host, unserialised, and have been re-cut
five times in six months:

| Version | Date | Change |
|---|---|---|
| v2.1.69 | 2026-03-05 | interactive tools *"silently auto-allowed when listed in a skill's allowed-tools, bypassing the permission prompt"* — fixed |
| v2.1.74 | 2026-03-12 | managed policy `ask` rules *"bypassed by user allow rules or skill allowed-tools"* — fixed |
| v2.1.91 | 2026-04-02 | `disableSkillShellExecution` added — a blunt kill-switch for the whole inline-shell lane |
| v2.1.178 | 2026-06-15 | subagent spawns evaluated by the classifier *before* launch, *"closing a gap where a subagent could request a blocked action without review"* |
| **v2.1.271** | **2026-09-14** | inline `` !`…` `` shell in skills/slash commands now follows default-mode permission rules, and *"a command no rule decides runs as a reviewed tool call"*; separately, a subagent now reports back *"through a dedicated hand-back call that the safety classifier reviews, instead of its last message being reviewed after the fact"* |

Read together, the last entry is a single idea applied twice: **convert an implicit, pre-authorised
lane into an explicit, in-path, reviewed call.** That is the right correction. It is also a
correction only the host can make, and only for its own lanes, one release at a time.

**The consequence for Caro is concrete.** Caro receives a string. `rm -rf ./build` typed by a human,
`rm -rf ./build` interpolated from a checked-in skill file, and `rm -rf ./build` returned by a
subagent that read it off a fetched web page are byte-identical. All three produce the same verdict
today, because origin is not an input. The 52+ pattern corpus, `is_dangerous_in_context`, the CVE
rules — every one of them is a function of the string alone.

### 4. Why this is not solved by the adjacent work already scoped

- **ADR-069 (`caro.preflight.v1`)** covers ambient configuration — the GitSpawn class, where the
  payload is the *state a benign command reads*, not the command text. Disjoint.
- **ADR-072 (`caro.annotate.v1`)** answers *what does this command do*, projected into MCP's four
  hints. Origin is not a property of what the command does.
- **ADR-042** reserves schema for *which principal* invoked Caro. A single principal can hand Caro
  text from five different origins in one session.
- **ADR-021** tags commands in shell history *after* execution, for a learning loop. Prospective
  validation never sees it.

### 5. What Caro can do here that the researched host cannot

Three things, and they follow from Caro being an external subprocess rather than the host:

1. **Caro can afford to be origin-aware, because Caro is not the thing being injected.** The host's
   classifier must stay blind because it reads the transcript, which an attacker can write into.
   Caro reads a command string and a declared chain from its *caller*, not from content. The reason
   for the blindness does not apply.
2. **Caro can hold the vocabulary still while hosts move.** When a host re-cuts a lane — as v2.1.271
   just did — an origin-aware validator sees a different *value in a field*, not a different
   *meaning of the contract*. Five host releases, one schema.
3. **Caro is agent-agnostic.** A Codex CLI wrapper, a Goose hook, and a CI script can each declare
   an origin chain in the same vocabulary. No host can define a cross-host origin vocabulary; an
   external validator is the only place one can live.

### 6. Ground truth in the tree (verified 2026-09-17, not assumed from other ADRs)

This ADR is written against the code, and several things other scope documents assume are **not
there**:

- `SafetyValidator` is a **concrete struct**, not a trait (`src/safety/mod.rs:155`). There is no
  trait boundary to implement against.
- **There is no exit-code registry.** Only `EXIT_CODE_EDIT = 201` (`src/main.rs:938`) plus ad-hoc
  `0`/`1`. ADR-024's proposed 0/1/2/3/4/6 table is unimplemented. Any ADR claiming codes are "frozen"
  is describing a table that does not exist.
- **No `schema_version` anywhere.** `caro.assessment.v1` and `caro.headless.v1` appear only in
  proposal documents; neither string is in `src/`.
- `caro scan`, `caro assess`, `caro guard`, `caro replay`, `caro bench` **do not exist**; `assess` is
  commented out at `src/main.rs:422` with its tests `#[ignore]`d.
- **No shell AST parser.** `is_dangerous_in_context` (`src/safety/mod.rs:432`) is regex plus a quote-parity
  heuristic. ADR-007's `yash-syntax` was never adopted — the crate is not in `Cargo.toml`.
- **No provenance of any kind.** `Turn { role: Role, content, command, confidence, risk, ts }`
  (`src/ai/session.rs:18`) with `Role ∈ {User, Assistant}` is the entire model. No `--agent-id`, no
  `CARO_AGENT_ID` in `src/`.

What *does* exist and is directly reusable: `RiskLevel {Safe, Moderate, High, Critical}` (`Ord`),
`SuggestedRouting {AutoApprove, AsyncLog, HumanGate, Block}`, `SafetyDecision`, the `SafetySection`
TOML surface with its `#[serde(default)]`-everything convention, `-o/--output json|yaml|plain` with the
`CliResult` envelope (`src/cli/mod.rs:76`), and — most importantly — **`blend_smart_decision`**
(`src/safety/mod.rs:277`), a bounded blend function with two written invariants. This ADR's core
function is deliberately its twin.

---

## Decision

Add **`caro.origin.v1`**: an optional, caller-declared, serialisable **origin chain** that is an
input to routing, and a small deterministic function that turns it into a routing escalation.

### D1 — Origin escalates *routing*, never *risk*

`RiskLevel` stays a property of the command text. `SuggestedRouting` becomes a function of
`(RiskLevel, SafetyLevel, OriginChain)`. Nothing in the pattern corpus, the CVE rules, or any future
benchmark (ADR-063) changes meaning, because the risk they measure is untouched.

### D2 — Monotonic upward only

`routing_with_origin >= routing_without_origin`, always. An origin chain can move a verdict
`AutoApprove → AsyncLog → HumanGate`. It can never move one down. This is the same restrict-only
invariant ADR-042 adopted for identity and ADR-072 for annotations, for the same reason: the
declaration comes from the caller and cannot be verified.

### D3 — Origin never produces `Block`

Escalation saturates at `HumanGate`. Only a pattern match blocks. This is a direct port of
`blend_smart_decision`'s stated bound — *"the judge can never add a hard block — only a static
`Critical` blocks"* — and it makes the worst case of a **wrong** chain a spurious prompt rather than a
broken workflow.

### D4 — Absent chain means today's behaviour, exactly

No `--origin`, or an empty chain, produces a byte-identical verdict and a byte-identical JSON envelope
(the field is `skip_serializing_if = "Option::is_none"`). Backward compatibility is total, and the
feature is opt-in per invocation. A caller who declares nothing is not punished — which is the honest
position, since Caro's threat model here is an *honest host that wants to be stricter than its own
lanes allow*, not a malicious host, which simply would not call Caro at all.

### D5 — The chain is a chain, not a label

The unit of declaration is an ordered `Vec<OriginHop>`, because the v2.1.271 fix is about *depth*, not
*category*. Two derived scalars drive the decision, and only two:

- **`review_distance`** — hops since the last hop a human actually read.
- **`untrusted_ingress`** — whether any unreviewed hop carried content from outside the session
  (fetched content, tool output, subagent return, a file in the repo).

### D6 — The escalation table is fixed, total, and in the ADR

| `review_distance` | `untrusted_ingress` | tiers raised |
|---|---|---|
| 0 | false | 0 |
| 0 | true | 1 |
| ≥ 1 | false | 1 |
| ≥ 1 | true | 2 |

Saturating at `HumanGate` (D3). No configuration, no weights, no scoring model, no LLM. A reviewer
can compute any verdict by hand — which is the property the researched classifier, by construction,
cannot offer.

### D7 — No new subcommand, no new exit code

Two flags on the existing path (`--origin`, `--origin-hop`) and one optional field on `CliResult`.
Exit codes are untouched: because origin never blocks (D3), the existing
`process::exit(if was_blocked {1} else {0})` at `src/main.rs:3531` is correct unchanged. The
machine-readable contract is the JSON field. **This ADR therefore does not block on ADR-024**, and
should not be allowed to acquire that dependency during review.

### D8 — Default off in config, on when a chain is passed

`[origin] enabled` defaults to `false` in `config.toml` for the escalation *effect*; the computed
`OriginVerdict` is still emitted in JSON when a chain is supplied, so integrators can measure the
escalation rate before they enable it. This is the ADR-056 observe-mode pattern applied at feature
granularity, and it is the mitigation for the demoware trap below.

---

## New types

All in `src/safety/mod.rs`, beside `blend_smart_decision`. All derive
`Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema`; all string enums are
`#[serde(rename_all = "snake_case")]`; every struct field is `#[serde(default)]`.

```rust
pub enum OriginActor {
    Human, Model, SkillFile, SlashCommand, Subagent,
    ToolOutput, FetchedContent, ConfigFile, Unknown,   // Unknown is the deserialization default
}

pub struct OriginHop {
    pub actor: OriginActor,
    pub reviewed: bool,            // a human read this literal text at this hop
    pub label: Option<String>,     // skill name, subagent name, url host — never parsed, only echoed
}

pub struct OriginChain { pub hops: Vec<OriginHop> }   // oldest first

pub struct OriginVerdict {
    pub schema: String,            // "caro.origin.v1" — the tree's first schema string
    pub review_distance: u8,
    pub untrusted_ingress: bool,
    pub tiers_raised: u8,
    pub routing_before: SuggestedRouting,
    pub routing_after: SuggestedRouting,
    pub reason: Option<String>,
}
```

Contract: `OriginChain::review_distance()`, `::untrusted_ingress()`, and

```rust
pub fn apply_origin_escalation(
    base: SuggestedRouting,
    chain: Option<&OriginChain>,
    enabled: bool,
) -> OriginVerdict;
```

with the two invariants asserted in unit tests over the full 4×4 cross-product of
`(SuggestedRouting, tiers_raised)`.

---

## Files changed

| File | Change |
|---|---|
| `src/safety/mod.rs` | five types + `apply_origin_escalation` + unit tests |
| `src/cli/mod.rs` | `CliResult.origin: Option<OriginVerdict>`, `skip_serializing_if` |
| `src/main.rs` | `--origin <json\|@file\|->`, repeatable `--origin-hop <actor[:label][:reviewed]>`; one call site |
| `src/models/mod.rs` | `[origin] enabled` in `UserConfiguration`; add the key to `ConfigSchema` (which today lists no `safety.*` keys either — a pre-existing gap worth closing in the same PR) |
| `src/bin/generate-schema.rs` | register the new `JsonSchema` types |
| `tests/origin_chain_test.rs` | new integration tests |

No new module. No new dependency.

---

## Integration tests — fixed inputs, deterministic JSON, asserted exit code

Using the existing `CliTestRunner` convention (`tests/e2e_cli_tests.rs:73`).

| # | Input | Asserted output |
|---|---|---|
| 1 | no `--origin` | `origin` key **absent**; envelope byte-identical to pre-change golden; exit `0` |
| 2 | `--origin-hop human:typed:reviewed` | `tiers_raised: 0`, `routing_after == routing_before`; exit `0` |
| 3 | `--origin-hop fetched_content --origin-hop model` | `untrusted_ingress: true`, `review_distance: 2`, `tiers_raised: 2`; exit `0` |
| 4 | chain that would raise past `HumanGate` | `routing_after: "human_gate"` (saturation); exit `0` |
| 5 | a `Critical`-pattern command + any chain | `routing_after: "block"` from the pattern; `tiers_raised` reported but inert; exit `1` |
| 6 | `[origin] enabled = false` + a deep chain | verdict emitted, `tiers_raised: 0`, routing unchanged; exit `0` |
| 7 | malformed `--origin` JSON | usage error on stderr, no partial envelope; exit `1` (today's error code — **not** a new one) |
| 8 | property test | for 10 000 random `(RiskLevel, SafetyLevel, chain)` triples, `routing_after >= routing_before` and `routing_after != Block` unless `routing_before == Block` |

---

## Consequences

**Good.** Caro gains an axis no competitor's runtime gate has, and gains it as a *contract* rather
than a heuristic. The vocabulary is cross-host, which is only possible from outside a host. The
verdict stays hand-computable. Zero behaviour change for every existing caller.

**Bad.** The chain is a self-declaration and cannot be verified — the same limitation MCP states about
its own annotations. D2/D3/D4 bound the damage to "a caller who lies gets the verdict they already
get today," but they do not eliminate it, and the ADR should not be reviewed as if they did.

**Cost.** One flag, one field, one function, roughly 250 LOC including tests. The real cost is
integrator work: every host wrapper must learn to populate the chain, and a half-populated chain is
worse than none because it reports `Unknown` hops as real ones.

---

## Alternatives considered

1. **Infer origin from the command text.** Rejected: the whole finding is that the three cases are
   byte-identical. Inference here is guessing.
2. **Copy the host: a second model judges provenance.** Rejected: non-deterministic, needs a model on
   the hot path, and contradicts the offline/subprocess constraint. `--approval smart` already exists
   for callers who want a judge.
3. **A single `origin: enum` label instead of a chain.** Rejected: loses depth, which is precisely what
   v2.1.271 was about. A skill that runs a subagent that fetched a page is not a "skill."
4. **Let origin lower risk for `Human` hops** (a trust bonus). Rejected: turns the field into a bypass.
   One forged hop and every gate opens. D2 exists to make this unavailable.
5. **Put the chain in `caro.assessment.v1` and wait.** Rejected: ADR-058/059 are unwritten in code,
   `assess` is a commented-out subcommand, and this ADR is deliberately buildable today. The field is
   designed to move into that payload unchanged when it lands.
6. **Model ambient origin (GitSpawn) here too.** Rejected: that is ADR-069, and the mechanism is
   different — ambient state read by a benign command, not text authored through hops.

---

## Validation discipline (`.claude/rules/validation-discipline.md`)

- **Gate 1 — 20 transcripts: UNMET.** Zero first-hand interviews exist for origin-aware validation.
  `docs/discovery/transcripts/` carries nothing on this axis. **No implementation PR may open on this
  ADR until Gate 1 is met.** The changelog evidence in Context §3 is vendor behaviour, not user
  demand, and the rule is explicit that scanning artefacts is not a substitute for talking to people.
- **Gate 2 — no surveys:** n/a, none used.
- **Gate 3 — demoware trap ("what breaks at 100 real users"):** the load-bearing assumption is that
  hosts populate the chain honestly *and consistently*. At demo scale one wrapper populates it. At
  100 users across six hosts, chains arrive half-populated and hop vocabularies diverge, so `Unknown`
  dominates — and the feature either never fires (useless) or fires on everything (prompt fatigue →
  users disable it, which is strictly worse than never shipping it). **Instrumentation:** emit
  `hops_declared`, `unknown_hop_ratio`, and `escalations_applied` per invocation in the verdict, so
  the ratio is measurable before anyone trusts it. **Fallback:** D8 ships the escalation off by
  default and the verdict on, so integrators measure first; D3 caps the worst case at a prompt.
- **Gate 4 — devil's-advocate review: REQUIRED and not yet run.** The two arguments to put in front of
  it: (a) D4 makes the honest caller strictly worse off than the silent one, which is an incentive
  structure worth attacking; (b) `Unknown` is both the deserialisation default *and* an untrusted
  actor, so a partially-populated chain escalates — defensible, but it is the knob that will generate
  every false positive.
- **Gate 5 — Sean Ellis:** no PMF claim is made anywhere in this document.

---

## Recommended next steps

1. Run Gate 4 (`devils-advocate`) against this ADR before anything else. It is cheap and it gates the
   rest.
2. Fold the origin question into the next discovery interviews rather than opening a dedicated round —
   *"when your agent runs a command, do you know whether you wrote it?"* — toward Gate 1.
3. Re-verify ADR-072's Black Hat / Novee / CVE citations (Provenance note above) before that ADR
   leaves Proposed.
4. Only then: implement D1–D8 as a single PR on a feature branch, per
   `.claude/rules/git-workflow.md`.

---

## Out of scope for v1

| Item | Why it waits |
|---|---|
| Verifying a chain (signatures, attestation) | needs ADR-049's evidence packet; v1 is explicitly a declaration |
| `--origin-strict` — non-zero exit when origin raises to `HumanGate` in a non-interactive run | needs the ADR-024 exit-code table, which does not exist |
| Carrying the chain into `caro.assessment.v1` | ADR-058/059 unwritten in code |
| Per-actor configurable weights | D6 is fixed on purpose; configurability is how a hand-computable rule stops being one |
| Recording origin in shell history / receipts | ADR-021, ADR-032 |
| Ambient-configuration origin (GitSpawn class) | ADR-069 |
| An `OriginActor` for MCP servers specifically | `ToolOutput` covers it; splitting it needs ADR-043 to land |
| Session-level accumulation of origin across turns | needs state; violates the no-daemon constraint |
