# ADR-027: Headless Permission Resolution — A Granular, Versioned, Machine-Answerable Approval Contract for Subprocess Callers

- **Status**: Proposed
- **Date**: 2026-06-29
- **Authors**: caro-research scoping process (automated scheduled run)
- **Target**: Hybrid (Community CLI + Enterprise CI / agent wrappers)
- **Builds on / relates to**: ADR-024 (Headless JSON/NDJSON output contract),
  ADR-025 (Headless init snapshot cache), ADR-026 (Headless multi-turn agentic
  session), ADR-020 (Tiered Approval Protocol), ADR-022 (caro safety library
  API), ADR-021 (Execution attribution), ADR-015 (MCP safety server — the
  daemon complement)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. The three prior runs
> scoped the headless **output contract** (ADR-024, 2026-06-23), the
> **redundant-init cold-start** dimension (ADR-025, 2026-06-24), and the
> **multi-turn agentic NDJSON conversation** (ADR-026, 2026-06-26). Each
> explicitly defers the *permission / tool-approval* surface to "a separate
> ADR." To add value without duplicating, this run targets exactly that
> deferred dimension. The competitor analog remains Claude Code's headless
> mode — specifically its **permission system** (`permissionMode`, the
> `canUseTool` callback, `allowedTools`/`disallowedTools`,
> `--permission-prompt-tool`, `--dangerously-skip-permissions`), the part of
> the headless surface that is still being actively reworked (see Anthropic's
> own "auto mode" engineering post). Treat the analog choice as a reviewable
> assumption, not a settled decision.

---

## Context

### Phase 1 — what the analog does, and why it is still unfinished

Claude Code's headless mode (`claude -p`) can take *actions* — run shell
commands, edit files, hit git. Whether a given action is allowed is decided by
a **permission layer** with four moving parts:

1. **`permissionMode`** — a global posture: `default`, `acceptEdits`, `plan`,
   `dontAsk`, and the blunt escape hatch `--dangerously-skip-permissions`
   ("bypassPermissions"). `dontAsk` converts any prompt that would otherwise
   reach the runtime callback into an immediate **denial**.
2. **`allowedTools` / `disallowedTools`** — static allow/deny rules evaluated
   before anything interactive.
3. **`canUseTool`** — a runtime callback that "handles everything else": for any
   action not pre-resolved by mode or rules, the SDK calls back to ask. This is
   the interactive-approval mechanism.
4. **`--permission-prompt-tool`** — in non-interactive `-p`, there is no TTY to
   answer `canUseTool`, so the caller must point at an **MCP tool** that
   answers permission prompts programmatically.

The **failure mode** this design produces in a pure subprocess context:

- In headless `-p`, the runtime `canUseTool` callback has **no answerer** unless
  the caller stands up an MCP `--permission-prompt-tool` server. Without it, an
  action that needs approval either **blocks/hangs** or surfaces as an
  ambiguous error — there is no machine-readable "this needs approval, here is
  how to grant it" signal.
- The common workaround is `--dangerously-skip-permissions`: an **all-or-nothing
  bypass** that disables the entire guardrail. Anthropic shipped "auto mode"
  *specifically because* the field was reaching for the dangerous flag for lack
  of a safer middle — an explicit admission that the binary choice (hang vs
  disable-everything) is the wrong contract.
- The shapes are **version-coupled and unversioned**: `permissionMode` values
  and the `canUseTool` payload drift across releases, so a wrapper pinned to one
  release breaks on the next (same root cause as ADR-024 §Context).
- The decision is **not a first-class field of the result**: a caller cannot
  read back *why* an action was allowed (static rule? mode? callback?) from the
  structured output in a stable, typed way.

The net: the part of headless mode that decides *whether a risky action is
allowed* — the part that matters most for unattended automation — is the least
machine-answerable and the most prone to the dangerous bypass.

### Who needs this in caro

ADR-024 made caro a machine-callable subprocess that emits a `CommandEnvelope`,
and it already distinguishes a `blocked` outcome (exit 3) when the safety
validator rejects a command. But "blocked" today is **terminal**: a
non-interactive caller that *intends* to allow a flagged command — a CI job that
knowingly runs `find … -delete`, an agent that has its own approval policy — has
no granular way to say so. Its only lever is to lower the global `SafetyLevel`
(Strict → Permissive), which is caro's `--dangerously-skip-permissions`: it
weakens the guardrail for **every** command in the run, not the one the caller
actually meant to approve.

Three audiences hit this immediately:

1. **Agent frameworks** wrapping caro as a tool, which carry their *own* policy
   ("auto-approve anything ≤ Moderate, escalate Critical to a human") and need
   to project that policy onto caro deterministically, per-invocation.
2. **CI / scripted flows** that must run a specific known-risky remediation
   without globally disarming safety for the rest of the pipeline.
3. **The multi-turn NDJSON stream (ADR-026)**, where a flagged command should
   produce a typed `permission_request` event the driving program can answer
   inline with a `permission_decision` — instead of the conversation dead-ending.

### What caro already has (and what is missing)

caro's safety layer is, in fact, **already designed for tiered approval** — it
just has no headless resolution path:

| Already present (reuse, do not duplicate) | Location |
|---|---|
| `ValidationResult { allowed, risk_level, explanation, warnings, matched_patterns, confidence_score }` (serde) | `src/safety/mod.rs:175` |
| `SafetyDecision { risk_level, reason, suggested_routing, matched_patterns, confidence }` (serde) — "richer context … for tiered approval integration" | `src/safety/mod.rs` |
| `SafetyDecision::{is_safe, requires_human_approval, is_blocked, from_validation_result}` | `src/safety/mod.rs` |
| `SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }` + `requires_human()` / `is_executable()` | `src/models/mod.rs:189` |
| `SafetyLevel { Strict, Moderate, Permissive }` (serde, `Default = Moderate`) | `src/models/mod.rs:247` |
| `RiskLevel` (serde, ordered) | `src/models/mod.rs:152` |
| `CommandEnvelope` + exit-code table + `blocked`/exit 3 | ADR-024 / `src/models/mod.rs` |
| NDJSON event enum + bidirectional message contract | ADR-026 / `src/ai/runner.rs` |
| `AiOutcome { …, risk, allowed, warnings }` | `src/ai/runner.rs:54` |

**The gap:** `SafetyDecision::requires_human_approval()` returns a routing of
"a human must decide," but in a headless subprocess **there is no human and no
machine-answerable substitute.** The decision falls through to `blocked`
(exit 3) with the only override being a global `SafetyLevel` downgrade. caro has
the *taxonomy* of a tiered approval protocol (ADR-020) and none of the
*headless resolution* of one.

---

## Decision

Add a **Headless Permission Resolution Contract**: a versioned, granular,
exit-coded mechanism that lets a non-interactive caller deterministically
resolve a `SafetyDecision` whose routing is "requires human" — **without** a
TTY, **without** an MCP daemon, and **without** the all-or-nothing
`SafetyLevel` downgrade.

The decision is supplied **up front as policy** (CLI flags) and/or **inline as a
typed NDJSON response** (in the ADR-026 stream). It is recorded as a
first-class, typed field on the `CommandEnvelope`. No new safety logic is
introduced; the contract is a thin, serializable resolution layer over the
existing `SafetyDecision` / `SuggestedRouting` / `SafetyLevel`.

Concretely:

1. A `--permission-mode {strict|ask|allow|deny}` flag plus a granular policy
   (`--allow-risk <=LEVEL>`, `--allow-pattern <regex>`, `--deny-pattern
   <regex>`) that resolves every `SafetyDecision::requires_human_approval()`
   deterministically.
2. For the bidirectional stream, two typed messages: an outbound
   `permission_request` event and an inbound `permission_decision`.
3. A `permission` record added to `CommandEnvelope` recording *what* was decided
   and *by which rule* — so the contract is auditable from the wire output.

### Resolution semantics (the heart of the contract)

A `SafetyDecision` is computed exactly as today. The new `PermissionPolicy`
**only** acts when `decision.requires_human_approval()` is true (auto-approve
and hard-`Block` are untouched — `Block` always denies, mirroring caro's
existing invariant that Critical patterns are never silently run):

| `--permission-mode` | Behaviour when routing == requires-human |
|---|---|
| `strict` (default) | **Deny.** Identical to today's headless behaviour: exit 3, `blocked`. Safe default — adding the contract changes nothing unless a caller opts in. |
| `ask` | Emit a `permission_request` event and **wait for a `permission_decision`** on the input stream (ADR-026 only). With no stream attached, `ask` degrades to `strict` and exits 7 (`needs_approval_no_channel`) so a wrapper can detect the misconfiguration deterministically. |
| `allow` | Approve **iff** the decision passes the granular policy (`risk_level <= --allow-risk` **and** matches no `--deny-pattern`, **or** matches an `--allow-pattern`). Otherwise deny (exit 3). Never approves a hard-`Block`. |
| `deny` | Deny every requires-human decision outright (caro's `dontAsk`). |

Hard rule: `--permission-mode allow` **cannot** approve a `SuggestedRouting::Block`
(Critical) decision. The blunt global downgrade (`SafetyLevel::Permissive`)
remains the only way to touch those, and remains loudly documented as the
dangerous lever — the equivalent of `--dangerously-skip-permissions`, kept
*separate* from this contract so granular approval never becomes a back door to
disabling Critical safety.

---

## New types (added to existing modules — no new `src` module)

All derive `Debug, Clone, PartialEq, Serialize, Deserialize` — **serializable
from day one** (constraint satisfied). Placed beside the types they extend so no
module is created.

### In `src/cli/mod.rs` (next to `OutputFormat`)

```rust
/// How a non-interactive caller resolves a `requires-human` safety routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    #[default]
    Strict,   // deny requires-human (today's behaviour; safe default)
    Ask,      // emit permission_request, await permission_decision (stream only)
    Allow,    // approve iff granular policy passes; never approves Block
    Deny,     // deny all requires-human (dontAsk analog)
}

impl std::str::FromStr for PermissionMode { /* mirrors OutputFormat::from_str */ }
```

### In `src/safety/mod.rs` (beside `SafetyDecision` / `SuggestedRouting`)

```rust
/// Up-front, machine-supplied policy that resolves requires-human routings
/// in a headless run. Pure data; constructed from CLI flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PermissionPolicy {
    pub mode: PermissionMode,
    /// Highest RiskLevel auto-approvable under `allow` mode. Default: Safe.
    pub max_auto_risk: RiskLevel,
    pub allow_patterns: Vec<String>,   // compiled lazily; regex over the command
    pub deny_patterns: Vec<String>,
}

impl PermissionPolicy {
    /// Resolve a decision deterministically. `stream_attached` distinguishes
    /// the ADR-026 bidirectional case from a single-shot subprocess.
    pub fn resolve(&self, decision: &SafetyDecision, command: &str, stream_attached: bool)
        -> PermissionOutcome;
}

/// Outcome of applying a `PermissionPolicy` to a `SafetyDecision`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum PermissionOutcome {
    Approved   { by: DecidedBy, matched_rule: Option<String> },
    Denied     { by: DecidedBy, reason: String },
    /// `ask` mode with a stream: caller must answer with a permission_decision.
    NeedsResponse { request_id: String },
}

/// Provenance of a permission outcome — auditable from the wire (relates to ADR-021).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecidedBy { Policy, Stream, DefaultStrict, BlockInvariant }
```

### Extension to `CommandEnvelope` (ADR-024, in `src/models/mod.rs`)

```rust
/// Added field — `#[serde(skip_serializing_if = "Option::is_none")]` keeps the
/// ADR-024 envelope wire-compatible for callers that never use permissions.
pub permission: Option<PermissionRecord>,

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionRecord {
    pub mode: PermissionMode,
    pub outcome: PermissionOutcome,   // flattened result/by/reason
    pub risk: RiskLevel,
    pub matched_patterns: Vec<String>, // from SafetyDecision, no recompute
}
```

Adding an `Option` field with `skip_serializing_if` does **not** require a
`schema_version` bump under ADR-024's compatibility rule (additive, optional);
the field appears only when `--permission-mode` is set to a non-default value.

### NDJSON messages (ADR-026 event enum extension, in `src/ai/runner.rs`)

```rust
// Outbound event (server → caller):
PermissionRequest {
    request_id: String,
    command: String,
    decision: SafetyDecision,   // already serde
}
// Inbound message (caller → server), parsed from the input stream:
PermissionDecision { request_id: String, approve: bool, reason: Option<String> }
```

**Method contracts**

- `PermissionPolicy::from_cli(mode, max_auto_risk, allow, deny) -> Self`.
- `PermissionPolicy::resolve(&decision, command, stream_attached) -> PermissionOutcome`
  — total function; never panics; compiles `allow/deny_patterns` once and caches.
  Honours the `Block`-invariant (a `Block` decision can only yield `Denied { by:
  BlockInvariant }`).
- `PermissionRecord::from_outcome(decision, outcome, mode) -> Self`.
- `CommandEnvelope` gains `with_permission(record) -> Self` (builder-style).

---

## Exit-code / output contract (what machines depend on)

Extends the ADR-024 table; existing codes are unchanged.

| Code | `status` / `permission.outcome` | Trigger |
|---|---|---|
| 0 | `ok`, `approved` | generated, and (if flagged) approved by policy/stream |
| 1 | `error` / `internal` | unexpected internal failure (ADR-024) |
| 2 | `error` / `usage` | bad flags — e.g. `--allow-pattern` not valid regex, `--permission-mode allow` with no policy |
| 3 | `blocked` / `denied` | requires-human routing denied by policy/mode, **or** a hard `Block` |
| 4 | `error` / `backend_unavailable` | (ADR-024) |
| 5 | `error` / `config` | (ADR-024) |
| 6 | `error` / `auth` | (ADR-024) |
| **7** | `blocked` / — | **`ask` mode but no input stream to answer the request** (`needs_approval_no_channel`) |
| 201 | (edit mode) | existing `EXIT_CODE_EDIT`, unchanged |

Guarantee: on `-o json`, exactly one `CommandEnvelope` is written to stdout; when
`--permission-mode` is non-default the envelope carries a `permission` record
naming the `outcome` and `DecidedBy`; the process exits with the code above.
This **fixes the Phase-1 failure mode by design**: the hang-or-disable-everything
dichotomy is replaced by (a) a granular per-invocation policy that never touches
the Critical-block invariant, (b) a distinct exit code (7) that makes "needs
approval, no channel" machine-detectable instead of a silent hang, and (c) a
typed, version-stable `permission` field instead of prose a caller must parse.

---

## Minimal set of files to change (no new `src` modules)

1. **`src/cli/mod.rs`** — add `PermissionMode` enum + `FromStr`; add the four
   flags (`--permission-mode`, `--allow-risk`, `--allow-pattern`,
   `--deny-pattern`) to the arg struct; build a `PermissionPolicy`.
2. **`src/safety/mod.rs`** — add `PermissionPolicy`, `PermissionOutcome`,
   `DecidedBy`, and `PermissionPolicy::resolve` beside the existing
   `SafetyDecision`. Pure data + one resolution function; reuses
   `SuggestedRouting::requires_human()` / `is_blocked()`.
3. **`src/models/mod.rs`** — add `PermissionRecord`; add the optional
   `permission` field + `with_permission` builder to `CommandEnvelope`.
4. **`src/ai/runner.rs`** — after `validate_command` produces a `SafetyDecision`,
   call `policy.resolve(...)`; set `AiOutcome.allowed` from the outcome; extend
   the ADR-026 NDJSON event enum with `PermissionRequest` /
   `PermissionDecision`; in `ask` mode, emit the request and read the response
   from the existing input-stream loop.
5. **`src/main.rs`** — add `EXIT_CODE_NEEDS_APPROVAL = 7` beside
   `EXIT_CODE_EDIT`; map `PermissionOutcome` → status + exit code at the single
   existing envelope-emit site (ADR-024); thread the policy from CLI to runner.
6. **`tests/permission_contract.rs`** — new integration test *binary* (a
   `tests/` file is a separate crate, not a `src` module — respects the "no new
   modules" constraint).

Everything else (validator, backends, session store, config, envelope
serialization) is reused unchanged.

---

## Integration tests — known inputs → deterministic JSON + exit code

Use the deterministic `mock`/static backend so output is byte-reproducible. The
mock is seeded to return a fixed flagged command (e.g. `find . -name '*.log'
-delete`, a Moderate/High requires-human routing) and a fixed Critical command
(`rm -rf /`, a hard `Block`).

| # | Invocation | Asserted JSON / exit |
|---|---|---|
| 1 | `caro -o json -b mock "clean logs"` (default mode) | `status=="blocked"`, `permission.outcome.result=="denied"`, `decided_by=="default_strict"` → **exit 3** |
| 2 | `caro -o json -b mock --permission-mode allow --allow-risk high "clean logs"` | `status=="ok"`, `outcome=="approved"`, `by=="policy"` → **exit 0** |
| 3 | `caro -o json -b mock --permission-mode allow --allow-risk moderate "clean logs"` (risk High > cap) | `status=="blocked"`, `outcome=="denied"` → **exit 3** |
| 4 | `caro -o json -b mock --permission-mode allow --deny-pattern 'delete' "clean logs"` | denied by deny-pattern, `matched_rule=="delete"` → **exit 3** |
| 5 | `caro -o json -b mock --permission-mode allow --allow-risk critical "wipe root"` (mock → `rm -rf /`) | **still** `denied`, `by=="block_invariant"` → **exit 3** (allow cannot approve Block) |
| 6 | `caro -o json -b mock --permission-mode ask "clean logs"` (no stream) | `status=="blocked"`, `error.kind` absent, exit-code field == 7 → **exit 7** |
| 7 | stream mode (ADR-026): pipe `clean logs` then `{"type":"permission_decision","approve":true}` | one `permission_request` event then `result` with `outcome=="approved"`, `by=="stream"` → **exit 0** |
| 8 | serde round-trip / snapshot of a fixed `CommandEnvelope` with a `permission` record | byte-stable field order; absent when mode default | n/a |

Test 8 locks the wire contract; test 5 locks the safety invariant (the single
most important assertion in this ADR — granular approval is provably not a
Critical-bypass).

---

## Out of scope (explicitly deferred to a later ADR)

- **On-disk policy files** (`--permission-config policy.toml`, org-wide allow/deny
  rule sets). v1 takes policy via flags + stream only. The serializable
  `PermissionPolicy` is forward-compatible with file-loading.
- **MCP-based permission server** (the `--permission-prompt-tool` analog). That
  is the *daemon* complement and belongs with ADR-015's MCP safety server, not
  in the daemonless subprocess contract.
- **Cryptographic signing / audit log of decisions.** Provenance is captured as
  `DecidedBy` in the envelope; durable signed audit ties into ADR-021 execution
  attribution and is its own scope.
- **Interactive TTY approval UX.** Already exists for the interactive path; this
  ADR is strictly the *headless* resolution contract.
- **Per-tool allowlists beyond shell commands** (file/network capability
  scoping). caro generates one shell command per turn; richer capability scoping
  is a v2 concern.
- **Approving a `Block` (Critical) via granular policy.** Deliberately impossible
  by the `Block`-invariant; the global `SafetyLevel` downgrade remains the only
  (loudly-flagged) lever.

---

## Consequences

### Positive
- Closes the gap between caro's *existing* tiered-approval taxonomy
  (`SafetyDecision` / `SuggestedRouting`) and its headless surface — the
  infrastructure is reused, not rebuilt.
- Eliminates the "hang or disable-everything" dichotomy that drove Claude Code
  users toward `--dangerously-skip-permissions`; caro callers get per-invocation
  granularity with the Critical-block invariant intact.
- Additive and opt-in: default (`strict`) behaviour is byte-identical to ADR-024,
  so no existing wrapper breaks.
- Decision provenance (`DecidedBy`) is on the wire — auditable, version-stable,
  no prose parsing.

### Negative / risks
- A fourth permission concept (`PermissionMode`) sits alongside `SafetyLevel`,
  `SuggestedRouting`, and `RiskLevel`; the ADR must document that
  `PermissionMode` resolves *only* requires-human routings and never overrides
  `Block`, or users may conflate it with `SafetyLevel`. Mitigation: the
  resolution-semantics table above + the test-5 invariant.
- Regex `allow/deny_patterns` are user input over the generated command; must be
  compiled once and length-bounded to avoid ReDoS (reuse the existing pattern
  compilation guards in `src/safety/patterns.rs`).
- `ask` mode only functions with an attached stream; the exit-7 contract makes
  the misconfiguration explicit rather than silent, but documentation must make
  the stream requirement obvious.

### Neutral
- No new dependency, no new module, no daemon, no persisted state — a pure
  subprocess contract, consistent with ADR-024/025/026.

---

## Alternatives considered

1. **Reuse `SafetyLevel` (Strict/Moderate/Permissive) as the only lever.**
   Rejected: it is global per-run, not per-decision — the exact all-or-nothing
   bypass we are trying to avoid. It cannot express "approve this one High
   command but keep blocking everything else."
2. **MCP `--permission-prompt-tool` server (mirror Claude Code 1:1).** Rejected
   for v1: it requires a daemon and violates the "pure subprocess, no daemon, no
   state" constraint. Captured as the deferred ADR-015 complement.
3. **A runtime `canUseTool`-style callback over IPC.** Rejected: implies a
   long-lived bidirectional process and ambient state; the up-front
   `PermissionPolicy` + the ADR-026 stream `permission_decision` cover the same
   need statelessly.
4. **Boolean `--yes` / `--force` flag.** Rejected: it is just
   `--dangerously-skip-permissions` by another name — no risk ceiling, no
   pattern scoping, no Block-invariant, no audit field.
5. **Make `permission` a required envelope field.** Rejected: breaks ADR-024
   wire-compat for callers that never use permissions; an `Option` with
   `skip_serializing_if` is additive.

---

## Constraint-compliance check

- *Reuse existing validator/safety/config infrastructure; do not duplicate* —
  `PermissionPolicy::resolve` is a thin layer over `SafetyDecision`,
  `SuggestedRouting`, `RiskLevel`, `SafetyLevel`; no new safety logic. ✅
- *All new types serializable from day one* — every new type derives full serde;
  the envelope field is additive-optional. ✅
- *Pure subprocess call (no daemon, no state)* — policy supplied up front by
  flags or inline by one NDJSON message; one process in, one envelope out, exit. ✅
- *Solve the Phase-1 failure mode by design, not workaround* — the
  hang-or-disable-everything dichotomy is replaced by granular policy + a
  distinct exit code (7) + a typed, versioned `permission` field, with the
  Critical-block invariant provable by test 5. ✅

---

## Suggested landing sequence (per repo rules)

1. Feature branch via `bin/sk-new-feature "headless permission resolution
   contract"` (`.claude/rules/git-workflow.md` — never commit to `main`).
2. Land types + `resolve` + tests behind the new flags (additive; `strict`
   default keeps ADR-024 output byte-identical).
3. Update `docs/adr/README.md` index with ADR-027; renumber only if a later ADR
   already claimed 027 (`.claude/rules/adr-numbering.md`).
4. This is an output/permission-contract feature, not a new product line, so the
   `validation-discipline.md` 20-transcript gate does not apply; standard
   `dev-process.md` CI (`cargo test`, `cargo clippy -- -D warnings`,
   `cargo test safety`) governs. The Critical-block invariant (test 5) is a
   safety-critical assertion — develop it TDD-first per
   `skill: safety-pattern-developer`.

## Sources

- [Configure permissions — Claude Code / Agent SDK Docs](https://code.claude.com/docs/en/agent-sdk/permissions)
- [Configure permissions — Claude API Docs](https://platform.claude.com/docs/en/agent-sdk/permissions)
- [How we built Claude Code auto mode: a safer way to skip permissions — Anthropic](https://www.anthropic.com/engineering/claude-code-auto-mode)
- [Claude Code --dangerously-skip-permissions: what it does and when not to use it — TrueFoundry](https://www.truefoundry.com/blog/claude-code-dangerously-skip-permissions)
- [Run Claude Code programmatically (headless) — Claude Code Docs](https://code.claude.com/docs/en/headless)
- Caro codebase: `src/safety/mod.rs` (`ValidationResult`, `SafetyDecision`,
  `SuggestedRouting`), `src/models/mod.rs` (`RiskLevel`, `SafetyLevel`,
  `CommandEnvelope`), `src/cli/mod.rs` (`OutputFormat`), `src/ai/runner.rs`
  (`AiOutcome`, `run_once`), `src/main.rs` (`EXIT_CODE_EDIT`).
- Prior runs of this scoping process: ADR-024 (headless JSON/NDJSON contract),
  ADR-025 (init snapshot cache), ADR-026 (multi-turn agentic session),
  `caro-scope-structured-output.md`.
