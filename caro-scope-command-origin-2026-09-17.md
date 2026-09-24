# Scope — `caro.origin.v1`: Command Origin Chain as a Validation Input

**Companion to** [`docs/adr/ADR-073-command-origin-chain.md`](docs/adr/ADR-073-command-origin-chain.md)
**Date**: 2026-09-17 · **Produced by**: `caro-research--scoping-process` (autonomous, no user present)
**Feature researched**: Claude Code auto mode permission classifier + the v2.1.271 (2026-09-14) lane changes
**All sources read**: 2026-09-17

---

## Phase 1 — Feature research

### 1.1 What it is and who it is for

Auto mode is permission-fatigue relief for developers running an agent that issues dozens of tool
calls per task. Instead of prompting a human on each one, a **second model reads the session and
decides**. It became the default in Claude Code on 2026-08-07.

### 1.2 Architecture and data flow

```
tool call
  ├─ permissions.deny ............... hard block. "Neither the classifier nor user intent can override it."
  ├─ permissions.ask (content-scoped)  always prompts, even in auto mode
  ├─ built-in allowlist ............. reads, searches, todo/plan transitions
  │    └─ on entering auto mode the host DROPS user rules granting arbitrary code
  │       execution (Bash(*), python:*, node:*, package-manager run) so they cannot
  │       be used to skip the classifier
  ├─ in-project file writes ......... auto-allowed, no classifier call
  └─ THE CLASSIFIER ................. shell, fetch, MCP, subagent spawn, out-of-project FS,
                                      SendMessage, rm/rmdir on critical paths
         ├─ stage 1: single-token yes/no, recall-tuned, ~8.5% FPR
         └─ stage 2: full chain-of-thought on survivors, ~0.4% FPR, ~17% FNR
```

Internal precedence inside the classifier: `hard_deny` > `soft_deny` > `allow` > explicit stated user
intent (only where the user's message *"directly and specifically describes the exact action"*).

**Model**: Sonnet 4.6 at launch (2026-03-25); changelog v2.1.210 (2026-07-14) moved external sessions
to Sonnet 5, *"validated on the session's first request and pinned for the session"* — a deliberate
per-session pin, i.e. the lifecycle answer to "how do we avoid re-deciding this every call."

**Key separation of concerns — and the crux of this scope.** The classifier's input is *deliberately
truncated*: user messages plus the agent's tool calls. Assistant prose is stripped; **all tool outputs
are stripped**. That is the anti-injection design. Content arriving from outside the session (web
pages, files, MCP results) never reaches the classifier at all — it is handled by a separate
first-layer injection probe that *annotates* suspicious content with a warning for the agent to read,
rather than blocking anything.

### 1.3 Why it is limited — in the vendor's own words

- 2026-03-25: *"the classifier finds approval-shaped evidence and stops short of checking whether it's
  consent for the blast radius of the action. We've tried prompt-engineering around this and haven't
  found a fix that doesn't cost more than it gains."*
- 2026-03-25, on origin blindness: *"If the user never named 'job abc123,' the classifier can't tell
  whether the agent pulled it from a query or fabricated it. We accept that cost for injection
  robustness."*
- 2026-08-07: *"it relies on classification systems and therefore does not eliminate risk. For
  high-stakes changes to production infrastructure, we still recommend reviewing Claude's actions
  yourself."*
- docs: *"Auto mode reduces permission prompts but does not guarantee safety."*

### 1.4 The demonstrated failure

Johann Rehberger, 2026-08-26, *Breaking Claude Code Opus 5 Auto Mode* — **60–80% success** on a chain
that never issues an instruction: a page drives a `curl`, the archive contains a `struct.py` that
shadows Python's stdlib module, and the RCE fires when the agent writes and runs *its own* decoder
script inside that directory. Every individual step reads as benign. Rehberger notes the classifier
*denied the cleanup command* when the agent tried to kill the malware it had noticed. Anthropic closed
the report **Informative / working as designed**, on the stated position that auto mode is a
best-effort convenience classifier and *"the real boundary is OS isolation and network egress
control."*

This is the origin problem in its purest form: the dangerous *text* was authored by the model, and
the *danger* came from content two hops upstream that the classifier is designed never to see.

### 1.5 Output contract and lifecycle

- Decision surface: `allow` / prompt / block, expressed as host behaviour, not a serialised payload.
  There is **no machine-readable verdict document** an external tool can consume.
- Circuit breaker: *"if the classifier blocks an action 3 times in a row or 20 times total, auto mode
  pauses… These thresholds are not configurable. Any allowed action resets the consecutive counter,
  while the total counter persists for the session."* (Note: the 2026-03-25 blog said headless mode
  *terminates the process*; current docs say the action does not run and Claude keeps working —
  a behaviour change between March and now, in neither case configurable.)
- Session lifecycle / redundant-init avoidance: the classifier model is chosen once and **pinned for
  the session**; rule lists are config-resident, not re-derived per call; the allowlist tiers exist
  precisely so most calls never reach the model.
- Config surface: `autoMode.environment`, `autoMode.allow` / `.soft_deny` / `.hard_deny` (natural-language
  rules, `"$defaults"` splicing), `autoMode.classifyAllShell`, `permissions.defaultMode`,
  `permissions.disableAutoMode`, `disableSkillShellExecution`, plus `claude auto-mode {defaults,config,critique,reset}`.
  Notably `permissions.defaultMode: "auto"` may only be set in `~/.claude/settings.json` or managed
  settings — **not** in project-local settings — *"so a checked-in repo or a build step could
  otherwise inject its own allow rules."* That is the host, in its own config rules, acknowledging
  provenance as a trust axis.

### 1.6 The v2.1.271 change, and the five-fix pattern behind it

See ADR-073 Context §3 for the table. The summary: **five releases in six months have adjusted which
lane a piece of command text travels in**, and the latest one converts two implicit lanes into
explicit reviewed calls. The skills documentation now describes the post-fix behaviour directly:

> *"Injected commands never prompt for permission while the skill renders. Claude Code checks each
> one against your permission rules first… In auto mode, a command that would otherwise need your
> approval doesn't abort the invocation. The skill loads with an instruction telling Claude to run the
> command first, and Claude's own call then goes through auto mode's usual checks."*

### 1.7 Adjacent: the Advisor tool

Real, official, and explicitly experimental: *"The advisor tool is experimental and requires the
Anthropic API… Behavior, pricing, and availability may change."* A second, stronger model consulted at
key moments; receives the **full conversation including every tool call and result** — i.e. the
opposite input policy from the classifier. Configured via `/advisor [model]`, `advisorModel`,
`--advisor`, `CLAUDE_CODE_DISABLE_ADVISOR_TOOL=1`. *"There is no setting to cap or force advisor
calls."* Not a safety gate and not scoped here; recorded because it shows the vendor running two
reviewers with deliberately different visibility, which is the same trade this scope makes.

### 1.8 Unverified claims — explicitly not used

Not located on 2026-09-17: any Anthropic source using a "Reversibility / Blast Radius / Security
Impact / Performance / Stealth-Persistence" five-dimension framing (third-party reconstructions only);
any Black Hat USA 2026 item on this topic; any vendor named "Novee"; any CVE assigned to the auto-mode
classifier itself. **ADR-072 cites the Black Hat / Novee / CVE-2026-12537 / CVE-2026-54316 cluster and
those citations should be re-verified.**

---

## Phase 2 — Competitive differentiation

### 2.1 What they get right, that we should copy

| Their choice | Why it is right | Where it lands in this scope |
|---|---|---|
| Deny rules evaluated **before** the model, unoverridable | the deterministic layer must outrank the probabilistic one | pattern verdict outranks origin; origin never blocks (D3) |
| Dropping `Bash(*)`-class allow rules on entering auto mode | a permission rule must not be a bypass | origin can only restrict (D2) |
| Defaults are pessimistic | unannotated ≠ safe | `Unknown` is the deserialisation default *and* counts as untrusted |
| Two-stage: cheap filter, expensive judge on survivors | latency discipline | ours is cheaper still: a table, no model |
| Model pinned per session | no redundant re-initialisation | ours is stateless per call; nothing to pin |
| `defaultMode: auto` banned from project-local settings | repo content is a distinct trust class | the entire premise of `OriginActor` |

### 2.2 Gaps we avoid by designing the schema first

1. **Lane churn.** Five releases re-cut which text is pre-authorised. A *field* whose value changes is
   stable; a *lane* whose membership changes is not. We define the field.
2. **No serialised verdict.** Their decision is host behaviour. Ours is a JSON object with a schema
   string, so a CI job can assert on it.
3. **Origin blindness inherited from an input policy we do not share.** They strip provenance because
   their judge reads attacker-writable transcript. Our input is a caller-supplied parameter. Copying
   the blindness would be cargo-culting the mitigation without the threat.
4. **A judge that can deny cleanup.** Rehberger's run had the classifier block the agent's attempt to
   kill malware. A non-deterministic judge with block authority has that failure available to it;
   D3 removes it from ours by construction.
5. **Non-configurable circuit-breaker thresholds** (3 / 20). Not copied — we have no session state.

### 2.3 Our unique positioning

- **Offline and deterministic.** No model on the path; the verdict table is in the ADR and computable
  by hand.
- **External, therefore cross-host.** A vocabulary spanning Claude Code, Codex CLI, Goose and plain CI
  can only be defined outside all of them.
- **Not the injection target.** Caro does not read the transcript, so it can afford the visibility the
  host must refuse.
- **Honest threat model.** Caro serves an honest host that wants to be stricter than its own lanes
  permit. A malicious host does not invoke Caro. Stating that plainly is the differentiation — the
  researched vendor states the equivalent boundary, and MCP states it about annotations.

### 2.4 Existing infrastructure that already covers part of this

| Need | Already in the tree |
|---|---|
| Risk tiers with ordering | `RiskLevel {Safe,Moderate,High,Critical}`, `Ord`, `src/models/mod.rs:152` |
| Routing tiers | `SuggestedRouting {AutoApprove,AsyncLog,HumanGate,Block}`, `src/models/mod.rs:189` |
| A bounded blend with written invariants | `blend_smart_decision`, `src/safety/mod.rs:277` — the direct template |
| Structured payload | `SafetyDecision`, `src/safety/mod.rs:189` |
| JSON output | `-o/--output json`, `CliResult`, `src/cli/mod.rs:76` |
| Schema emission | `src/bin/generate-schema.rs` + `schemars` |
| TOML config with defaulted sections | `SafetySection`, `src/safety/mod.rs:134` |
| Restrict-only precedent | `validate_user_pattern` caps user patterns at `High` (`src/safety/mod.rs:74`) |

**What is not there and must not be assumed**: no `SafetyValidator` trait, no exit-code registry, no
`schema_version` string anywhere in `src/`, no `caro scan/assess/guard/replay/bench`, no shell AST
parser, no `src/preprocessing/`, no provenance of any kind (`Turn.role ∈ {User, Assistant}` is the
whole model), no `--agent-id` / `CARO_AGENT_ID`.

---

## Phase 3 — Scope

Decisions D1–D8, the type definitions, the file list, the test matrix and the out-of-scope list live
in [ADR-073](docs/adr/ADR-073-command-origin-chain.md) and are not duplicated here. Three
implementation notes that belong in the scope rather than the record:

### 3.1 Flag surface

```
--origin <JSON|@path|->        full chain, for programmatic callers
--origin-hop <SPEC>            repeatable, shell-friendly; SPEC = actor[:label][:reviewed]
```
`--origin-hop` exists so a five-line bash wrapper can declare a chain without emitting JSON. Both
flags may not be combined (usage error, exit `1`). Ordering of repeated `--origin-hop` is oldest-first
and is load-bearing.

### 3.2 The parsing rule that keeps this deterministic

`label` is **never parsed** — it is echoed into the verdict and never matched, so no future pattern can
come to depend on its contents. This is the single rule that stops `caro.origin.v1` from quietly
growing into a second pattern corpus.

### 3.3 Sequencing

`apply_origin_escalation` runs **after** the pattern verdict and after `blend_smart_decision`, taking
whatever routing those produced as its `base`. Composition is therefore: patterns set risk → safety
level sets routing → smart judge may move it both ways within its own bounds → origin may only raise
it, never past `HumanGate`. Each layer's invariant is independently testable and the order is fixed.

### 3.4 Definition of done

- `cargo test` green including the 8-case matrix and the 10 000-triple property test
- `cargo clippy -- -D warnings` clean
- a pre-change golden envelope for the no-`--origin` path, asserted byte-identical
- Gate 4 review comment in the PR
- **Gate 1 met** — without it the PR does not open (`.claude/rules/validation-discipline.md`)
