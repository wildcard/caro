# ADR-036 — `caro guard`: Universal Agent-Hook Safety Adapter

**Date**: 2026-07-14
**Status**: Proposed
**Authors**: `caro-research--scoping-process` (automated research/scoping agent)
**Relates to**: ADR-020 (tiered approval), ADR-022 (caro-safety library), ADR-023 (`caro scan`), ADR-024 (headless JSON contract), ADR-032 (execution receipts), ADR-035 (external policy hooks — the *inverse* of this ADR)
**Research**: Claude Code hooks reference (code.claude.com/docs/en/hooks, fetched 2026-07-14); Gemini CLI hooks reference (geminicli.com/docs/hooks/reference, last updated 2026-04-10, fetched 2026-07-14); Hermes 07-10 weekly scan recommendation ("PreToolUse hook adapter emitting Caro assessment JSON") and 07-13 scan (OTel envelope + OWASP Agentic Top 10 mapping)

---

## Context

### The Gap

ADR-035 made caro a *host* of external policy hooks. Nothing makes caro a
*guest* — the guard binary that other agents' hook systems call before they
run a shell command. Hermes has recommended this twice in a row (07-10,
07-13): "the smallest artifact that proves 'Caro as the reviewer engine
behind any agent'."

Two mainstream agent CLIs now ship stable command-hook systems that can
spawn a subprocess before every shell execution:

| Host | Event | Shell tool name | Input | Block signal |
|---|---|---|---|---|
| Claude Code | `PreToolUse` | `Bash` | stdin JSON: `session_id`, `prompt_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`, `tool_name`, `tool_input.command`, `tool_use_id`, optional `agent_id`/`agent_type` | exit 2 + stderr, **or** exit 0 + JSON `hookSpecificOutput.permissionDecision: "deny"` (also `"allow"`, `"ask"`, `updatedInput`) |
| Gemini CLI | `BeforeTool` | `run_shell_command` | stdin JSON: `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `timestamp`, `tool_name`, `tool_input`, optional `mcp_context` | exit 2 + stderr, **or** exit 0 + JSON `decision: "deny"` (alias `"block"`) + `reason`; no `"ask"` tier |

Today a user who wants caro's 52-pattern engine guarding Claude Code must
hand-write a jq wrapper. There is no supported entry point: interactive
`caro` prompts and colours; `caro scan` (ADR-023) speaks shellcheck-JSON/
SARIF, not host hook dialects; the headless envelope (ADR-024) answers
"generate a command", not "judge this one".

### Phase 1 — The Failure Mode Both Hosts Share: Fail-Open Policy

Both host contracts treat everything except the magic value as
non-blocking:

- **Claude Code**: "Claude Code treats exit code 1 as a non-blocking error
  and proceeds with the action, even though 1 is the conventional Unix
  failure code." JSON is parsed **only on exit 0**; on exit 2 stdout is
  ignored. HTTP hook timeouts, connection failures, and non-2xx responses
  "produce non-blocking errors that allow execution to continue."
- **Gemini CLI**: exit codes other than 0/2 are "Warning. A non-fatal
  failure occurred; the CLI continues." A hook that prints any plain text
  before its JSON breaks parsing and degrades to a warning.

So a guard hook that panics, hits a bad allocation, prints a stray log
line to stdout, or is invoked with input it can't parse **silently allows
the command it was supposed to judge**. This is the specific failure mode
to solve by design: a crash in the safety layer must never read as
approval.

A second, softer failure mode: hook processes are spawned per event, so a
guard that loads models, opens caches, or reads large config at startup
adds its full init cost to *every* tool call of the host session.

## Decision

Add one subcommand — **`caro guard --host <claude-code|gemini-cli|generic>`**
— a pure, stateless subprocess that reads one host-native hook payload
from stdin, runs the existing static safety validator, and emits one
host-native verdict. No daemon, no session state, no model load, no
network.

### 1. Host adapters, one core

`guard` normalizes each host's input into one internal type, judges it
once, and serializes the verdict back into that host's dialect:

```rust
/// src/cli/guard.rs (new file inside the existing cli module)
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum HostKind { ClaudeCode, GeminiCli, Generic }

/// Normalized view of any host's pre-execution hook payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardInput {
    pub host: HostKind,          // serialized as kebab-case string
    pub tool_name: String,       // "Bash", "run_shell_command", …
    pub command: Option<String>, // None => not a shell tool
    pub cwd: Option<String>,
    /// Host-side correlation IDs, passed through untouched (never parsed).
    pub correlation: serde_json::Map<String, serde_json::Value>,
}
```

Unknown fields in host input are ignored (`#[serde(deny_unknown_fields)]`
is deliberately **not** used — host schemas grow monthly; ADR-035 finding
that Claude Code added 30 events over time).

### 2. Verdict mapping (reuses ADR-020 routing verbatim)

`SafetyDecision::from_validation_result` already yields a
`SuggestedRouting`. The mapping is fixed:

| `SuggestedRouting` | claude-code output | gemini-cli output | generic output |
|---|---|---|---|
| `Block` | exit 0 + `permissionDecision: "deny"` + reason | exit 0 + `decision: "deny"` + reason | envelope `decision: "deny"` |
| `HumanGate` | exit 0 + `permissionDecision: "ask"` + reason | exit 0 + `decision: "deny"` + reason prefixed `needs-human-approval:` (host has no ask tier) | envelope `decision: "ask"` |
| `AsyncLog` | **silent pass** (exit 0, no stdout) | silent pass | envelope `decision: "allow"` + `log: true` |
| `AutoApprove` | **silent pass** (exit 0, no stdout) | silent pass | envelope `decision: "allow"` |

**Silent pass is deliberate.** Emitting `permissionDecision: "allow"`
would *override the host's own permission prompts* — caro widening the
host's policy. Per the ADR-035 principle "hooks can tighten, never
loosen", the default emits an opinion only to restrict. An explicit
`--emit-allow` flag opts into allow-signaling for hosts used in
`bypassPermissions`-style modes where the caller wants caro to be the only
gate.

Non-shell tools (`tool_name` not in the host's known shell-tool set and no
`tool_input.command` string): silent pass. The docs ship the recommended
matcher (`"matcher": "Bash"` / `"run_shell_command"`) so hosts don't even
spawn caro for file edits.

### 3. Fail-closed by emission (the Phase-1 fix)

`guard` never lets an internal failure reach the host as a fail-open
signal. The entire run is wrapped so that **every** error path — stdin
unreadable, JSON unparseable, unknown host field shapes, validator panic
(caught via `catch_unwind`), output serialization failure — converges to
the host's *strongest* deny signal:

- claude-code / gemini-cli: **exit 2**, one-line reason on stderr
  (`caro-guard: fail-closed: <cause>`). Exit 2 is chosen for the error
  path (not JSON-deny) because it works even when stdout is already
  poisoned or the output encoder itself failed.
- generic: deny envelope with `origin: "error"` on stdout, exit 3.

`--on-error allow` opts out (observe-only rollouts), mirroring ADR-035's
`on_failure` knob. The default is closed. `caro guard` itself never exits
1 — the code both hosts ignore — under any circumstance.

### 4. `generic` host: the published assessment envelope

`--host generic` is the Hermes "publish the schema" deliverable — a
versioned, host-agnostic assessment payload other frameworks (LangChain
callbacks, OpenAI Agents SDK guardrails, CI glue) can consume:

```rust
/// src/safety/mod.rs — new type next to SafetyDecision
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AssessmentEnvelope {
    pub schema_version: u32,          // 1
    pub decision_id: uuid::Uuid,      // v4, per ADR-022's planned field
    pub command_hash: String,         // "sha256:<hex>" of the exact command bytes
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub decision: GuardDecision,      // Allow | Ask | Deny  (kebab-case)
    pub origin: AssessmentOrigin,     // Patterns | Error    (kebab-case)
    pub risk_level: RiskLevel,
    pub reason: String,
    pub matched_patterns: Vec<String>,
    pub confidence: f64,
    /// OWASP Agentic Top 10 category IDs implicated by matched patterns
    /// (e.g. "AAI-05"), empty when none apply. Hermes 07-13 item A.
    pub owasp_agentic: Vec<String>,
    /// Flat OTel-compatible attributes (gen_ai.tool.name, caro.risk_level, …)
    /// for direct log-record embedding. Hermes 07-13 item A.
    pub otel_attributes: std::collections::BTreeMap<String, String>,
    /// Host correlation IDs echoed back (session_id, tool_use_id, …).
    pub correlation: serde_json::Map<String, serde_json::Value>,
}
```

`BTreeMap` keeps attribute order deterministic. All new types derive
`Serialize`/`Deserialize`/`JsonSchema` from day one; the schema is emitted
by the existing `generate-schema` bin and published under `docs/spec/`
alongside the ADR-032 receipt schema.

### 5. Exit-code / output contract (what machines depend on)

Adapter modes speak **the host's contract, not caro's** — that is the
entire point of an adapter:

| Mode | Exit | Meaning | stdout |
|---|---|---|---|
| claude-code, gemini-cli | 0 | verdict delivered (deny/ask JSON, or silent pass) | host-dialect JSON or empty |
| claude-code, gemini-cli | 2 | fail-closed error path (also usable as belt-and-braces block) | ignored by host; reason on stderr |
| generic | 0 | assessment produced, `decision: allow \| ask` | `AssessmentEnvelope` |
| generic | 3 | `decision: deny` (aligns with ADR-024's 3 = blocked) | `AssessmentEnvelope` |
| all | 64 | usage error before reading stdin (unknown `--host`, conflicting flags) | empty |

No other exit codes exist in guard mode. 64 (`EX_USAGE`) cannot collide
with host semantics because it occurs only when a human misconfigures the
hook command line, and both hosts surface it as a visible warning.
Stability promise: same as ADR-024 — fields are add-only within
`schema_version: 1`; exit meanings never change.

### 6. Session/context lifecycle

Hook processes are spawned per tool call, so `guard` is engineered for
cold-start cost ≈ zero rather than for cache reuse:

- **No backend/model initialization** — guard mode never touches
  `backends/`; it constructs only the pattern validator (pre-compiled
  regexes, <2ms, measured in ADR-022).
- **No config-network** — reads `config.toml` if present for safety-level
  and user patterns; missing config falls back to defaults silently.
- **No state** — nothing written to disk. (Receipt emission is the
  ADR-032 integration, out of scope v1.) Repeat invocations are
  byte-for-byte deterministic apart from `decision_id`/`created_at`.

This also answers "how does it avoid redundant initialization": by having
nothing redundant to initialize, instead of a daemon or warm cache — per
the task's pure-subprocess constraint, and unlike the hosts' HTTP hook
variant which exists precisely because their command hooks got too slow
once people put model calls in them.

## Phase 2 — Competitive Differentiation

**What the hosts get right (replicated):** single stdin JSON in / stdout
JSON out shape; regex matchers so the guard only runs for shell tools;
per-hook timeout expectations (guard's <10ms budget sits far under both
hosts' 60s defaults); Gemini's "silence is mandatory" rule (guard writes
nothing to stdout except the final JSON — all diagnostics to stderr).

**Their gaps we avoid by design:** fail-open on exit 1/timeouts/parse
errors (§3); dual signaling channels that discard JSON on exit 2 (guard's
error path uses stderr only, its success path stdout only — never both
carrying decisions); Gemini's missing "ask" tier (guard degrades
`HumanGate` to deny-with-prefixed-reason rather than to allow);
allow-signals that widen host policy (silent-pass default, §2).

**Unique positioning:** deterministic, offline, <10ms, zero-LLM judgment —
both hosts' example guard hooks are literally `if [[ "$command" == rm* ]]`;
caro brings 52+ CVE-derived, quote-context-aware, BSD/GNU-aware patterns.
One binary guards *both* ecosystems plus anything generic. No cloud
dependency means it runs in the air-gapped CI environments where these
agents increasingly operate. OpenAI Codex CLI has no command-hook system
at all today — `--host generic` plus its sandbox-policy config is the only
path there, documented as such.

**Existing infrastructure reused (nothing duplicated):**
`SafetyValidator` + `SafetyDecision` + `SuggestedRouting::from_risk_and_safety`
(ADR-020) unchanged; `Cargo.toml` already carries `uuid`, `sha2`, `chrono`,
`schemars`, `serde_json`; `generate-schema` bin publishes the envelope
schema; ADR-023's stdin-reading discipline; ADR-024's envelope style and
exit-3 alignment.

## Consequences

**Positive:** caro becomes installable as the safety layer of the two
dominant agent CLIs with one settings.json line; the published envelope is
the marketing artifact Hermes has asked for twice; fail-closed emission
makes caro demonstrably safer than the hosts' own example hooks — a
concrete differentiator for the comparison table on caro.sh.

**Negative / risks:** host dialects are theirs to break — mitigated by
ignoring unknown fields, pinning to documented stable fields only, and a
fixture-based contract test per host that CI can re-verify against doc
snapshots; silent-pass default means caro adds no value on safe commands
unless the user reads the docs (accepted: correctness over visibility);
`HumanGate`→deny degradation on Gemini may over-block for `paranoid`
safety levels (documented; users can set safety-level per config).

**Trade-off accepted:** no `updatedInput` command-rewriting in v1 even
though Claude Code supports it — rewriting a command caro judged dangerous
into a "safer" variant is exactly the kind of silent policy-widening §2
forbids, and ADR-016 (self-healing fix) owns the rewrite concept.

## Scope — Files Changed (minimal set)

1. `src/main.rs` — add `Commands::Guard { host, on_error, emit_allow, config }` + dispatch (~40 LOC)
2. `src/cli/guard.rs` — **new file, existing module** — `HostKind`, `GuardInput`, host parse/emit adapters, fail-closed wrapper (~300 LOC)
3. `src/safety/mod.rs` — `AssessmentEnvelope`, `GuardDecision`, `AssessmentOrigin`, `owasp_agentic` mapping table on existing patterns (~120 LOC)
4. `src/bin/generate-schema.rs` — register `AssessmentEnvelope` (~5 LOC)
5. `tests/guard_contract.rs` — **new test file** — integration tests below
6. `docs/spec/assessment-envelope.schema.json` — generated artifact

No new top-level modules, no new dependencies.

## Integration Tests (known input → deterministic output + exit code)

Fixtures under `tests/fixtures/guard/`:

1. claude-code payload, `rm -rf /` → exit 0, `permissionDecision: "deny"`, reason names the matched pattern
2. claude-code payload, `ls -la` → exit 0, empty stdout (silent pass)
3. claude-code payload, `dd if=/dev/zero of=/dev/sda bs=1M` with `--emit-allow` → still deny (allow flag cannot weaken)
4. claude-code payload, HumanGate-tier command (`chmod -R 777 /home`) → `permissionDecision: "ask"`
5. gemini-cli payload, same HumanGate command → `decision: "deny"`, reason starts `needs-human-approval:`
6. gemini-cli payload, non-shell tool (`read_file`) → exit 0, empty stdout
7. malformed JSON on stdin, `--host claude-code` → exit 2, stderr starts `caro-guard: fail-closed:`
8. malformed JSON, `--on-error allow` → exit 0, empty stdout
9. generic mode, dangerous command → exit 3, envelope with `schema_version: 1`, `decision: "deny"`, `command_hash` matching precomputed sha256, non-empty `owasp_agentic`
10. generic mode, safe command → exit 0, `decision: "allow"`; assert all fields except `decision_id`/`created_at` byte-identical across two runs
11. guard mode never exits 1: fuzz-ish table of 20 broken inputs, assert exit ∈ {0, 2, 3, 64}

## Out of Scope (next version)

- `caro guard install <host>` — auto-writes host settings.json (v1.1; needs Design-dialogue-free UX pass)
- PostToolUse/AfterTool result auditing and receipt emission (joins ADR-032)
- `updatedInput` safe-rewrite integration with ADR-016
- HTTP hook variant (daemon-shaped; violates the pure-subprocess constraint by definition)
- `session_context` provenance field (Hermes 07-13 "Later" item; one-page spec first)
- Envelope signing (ADR-032 v2 dependency; crypto crate spike per external-sdk-integration.md)
- Codex CLI adapter (no hook system exists to adapt to)

## Alternatives Considered

1. **Ship only shell-script wrappers around `caro scan`** — rejected: jq
   dependency, fail-open by construction (a wrapper bug is exit 1), and
   scan's shellcheck-JSON contract doesn't carry approval routing.
2. **MCP server as the integration point (extend ADR-015)** — rejected for
   v1: hosts call hooks unconditionally per tool call; MCP tools must be
   *chosen* by the model, which is exactly the wrong trust direction for a
   guard.
3. **One `--format` flag on `caro scan` instead of a new subcommand** —
   rejected: scan is batch/file-oriented with issue-list semantics; guard
   is single-verdict with host-dialect semantics; overloading repeats the
   `caro check` near-miss ADR-023 documented.
4. **Daemon with warm pattern cache** — rejected: violates the
   pure-subprocess constraint, and measured validator init (<2ms) makes it
   unnecessary.

## References

- Claude Code hooks reference — code.claude.com/docs/en/hooks (fetched 2026-07-14)
- Gemini CLI hooks reference — geminicli.com/docs/hooks/reference (fetched 2026-07-14)
- Hermes digests: `.hermes/digests/2026-07-10-weekly-agent-market-scan.md`, `2026-07-13-agent-market-scan.md`
- ADR-020, ADR-022, ADR-023, ADR-024, ADR-032, ADR-035
- OWASP Agentic AI Top 10; OTel GenAI semantic conventions (`gen_ai.*`)
