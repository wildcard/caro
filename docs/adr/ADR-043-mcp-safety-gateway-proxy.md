# ADR-043: MCP Safety Gateway (Proxy Mode) — Transparent stdio Interception of Exec-Tool Calls with Structured, Self-Correctable Denials

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-30
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: MCP tool-call interception gateways — **McpVanguard**
  (provnai/McpVanguard, MIT, Python; tracking the MCP 2026-07-28 final spec)
  and **mcp-guardian** (eqtylab/mcp-guardian, Apache-2.0, Rust; GUI-gated
  approvals) — read against the **MCP 2026-07-28 final spec** (stateless
  core, Tasks + Apps extensions; audited in ADR-038)
- **Depends on**: ADR-020 (tiered approval — `SuggestedRouting` verdict
  vocabulary), ADR-041 (caro-events Phase 0 — the audit sink this proxy
  emits into)
- **Relates to**: ADR-015/038 (caro as a *voluntarily called* MCP validator
  tool — this ADR is the involuntary complement), ADR-036 (`caro guard` —
  caro as a *guest* of another agent's hook system; this ADR needs no host
  cooperation at all), ADR-040 (policy file — composes later), ADR-031
  (session circuit breaker — deferred), Hermes market scan 2026-07-30
  (opportunity A, #1 "build/test next")

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Target selection
> rationale: the 2026-07-30 Hermes scan names the MCP safety gateway its #1
> build item ("purest expression of the universal standalone safety layer"),
> and a duplication sweep of all 48 ADRs finds no coverage: ADR-015/038 make
> caro a validator *tool the agent must choose to call*; ADR-036 makes caro
> a *hook the host must be configured to spawn*; no ADR places caro in the
> tool-call path itself. The Jul 27 unauthorized-actions audit (≈1.2
> out-of-scope actions per agent run across popular frameworks) is the
> evidence that voluntary integration points are insufficient. Treat the
> no-SDK framing decision (D1) as a reviewable assumption.

---

## Context

### Phase 1 — What the analogs are and how they work

**Problem and audience.** Agents call tools over MCP; some of those tools
execute shell commands (`run_command`, `bash`, `terminal`, …). Tool
*permissions* are declared in MCP host config, but nothing validates the
*command strings* flowing through approved tools. A gateway that sits
between agent and server can inspect every `tools/call` before the server
sees it — enforcement that does not depend on the agent cooperating.
Audience: developers wrapping local stdio servers, and platform/security
teams fronting shared servers.

**McpVanguard architecture** (README + docs, fetched 2026-07-30). Pipeline
per call: L0 preflight (NFKC/URL-decode/zero-width-strip normalization,
size gates) → auth policy → L1 deterministic rules (signatures, recursive
argument inspection, path safe-zones) → L1.5 camouflage detection → L2
optional LLM semantic scorer (OpenAI/Groq/Ollama; can escalate, cannot
downgrade a deterministic block) → L3 behavioral state (optionally
Redis-backed) → policy composer producing
`ALLOW / WARN / REVIEW / SHADOW-BLOCK / BLOCK`. Deployment: local stdio
wrap (`vanguard start --profile balanced --server "npx …"`) or hosted
SSE/Streamable-HTTP gateway with JWT/JWKS auth. On block, the agent
receives **a standard JSON-RPC error** and the upstream server never sees
the call. Audit: JSONL with `policy_explanation` (primary layer, rule
family, profile effect, upstream-call status, tuning hint), optional
signed `receipt_v1`. Adoption path: `monitor` → `balanced` → `strict`
profiles. It already ships additive `Mcp-Method`/`Mcp-Name` consistency
checks against the 2026-07-28 spec.

**mcp-guardian architecture.** Rust workspace (`mcp-guardian-proxy` +
core + Tauri GUI): proxies stdio servers, logs all traffic, and gates
individual tool-call messages behind **real-time human approval in the
desktop GUI**. Automated scans are "Coming Soon"; last release v0.6.0
(Apr 2025) — effectively dormant.

**Why these are experimental / their failure modes:**

1. **Shell-blindness.** Both inspect tool calls *generically*. McpVanguard's
   deterministic layer is normalization + signatures + path boundaries — it
   has no shell-grammar-aware command engine, and its benchmarks explicitly
   disclaim zero-false-positive behavior. The dangerous-command problem is
   delegated to L2, an **optional network LLM call** — nondeterministic,
   latent, offline-hostile. mcp-guardian ships no scanner at all.
2. **Opaque denials.** A blocked call returns a generic JSON-RPC error. The
   calling model gets no machine-readable assessment — no risk tier, no
   matched pattern, no reason it can use to self-correct — so the observed
   behavior is retry storms or silent task failure. (JSON-RPC errors also
   conflate "protocol broke" with "policy said no".)
3. **Daemon and state creep.** Hosted gateway modes, per-session budgets,
   Redis-backed behavioral state, JWT plumbing — the local-first story
   erodes as the useful features accrete to the daemon.
4. **GUI-bound approval.** mcp-guardian's human gate requires its desktop
   app; headless hosts (CI, servers, other agents) cannot use it.
5. **Exec-tool identification drift.** Which tools carry command strings is
   per-config guesswork; a server renaming `run_command` → `shell_exec`
   silently bypasses interception.

**Structured output contract (theirs).** McpVanguard: JSON-RPC error on
block; JSONL audit lines with SIEM-friendly decision fields; verdict enum
of five values; `receipt_v1` export. No published JSON Schema for the
deny payload itself — the agent-facing surface is unversioned prose.

**Session/context lifecycle.** The 2026-07-28 spec's stateless core (no
handshake session, no `Mcp-Session-Id` — ADR-038's audit) makes stdio
proxying nearly stateless: a gateway holds only process-lifetime state.
McpVanguard compiles rules once per process; redundant initialization is
avoided by construction because the proxy lives exactly as long as the
host app keeps the stdio pipe open.

### Phase 2 — What caro already has (verified against source)

| Need | Exists now | Where |
|---|---|---|
| Shell-specialized deterministic engine, 52+ patterns, zero-FP discipline | yes | `src/safety/` (`SafetyValidator`) |
| Assessment payload (Serialize) | yes | `ValidationResult`, `src/safety/mod.rs:175` |
| Verdict + routing payload | yes | `SafetyDecision` (`risk_level`, `reason`, `suggested_routing`, `matched_patterns`, `confidence`) + `SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }`, `src/safety/mod.rs:189`, `src/models/mod.rs:189` |
| Tier semantics | yes | `RiskLevel { Safe, Moderate, High, Critical }`, ADR-020 mapping |
| Audit event schema + sink | scoped, implementable today | ADR-041 (`EventSink`, `caro-events/1`, `--events-file`) |
| Secret redaction | yes | `src/logging/redaction.rs` |
| Feature-flag precedent (off-by-default) | yes | `Cargo.toml [features]` (`candidate-ranking` pattern) |
| Schema generation + CI diff | yes | `src/bin/generate-schema.rs`, schemars 0.8 |
| Subcommand-in-`cli`-module precedent | yes | `src/cli/scan.rs` planned by ADR-023; `edit_prompt.rs`, `telemetry.rs` shipped |
| JSON-RPC passthrough loop, exec-tool matching, deny envelope | **no** | — this ADR |

Note deliberately absent: there is **no `src/mcp/`, no rmcp dependency**
in-tree today. ADR-015/038's server waits on its build spike per
`.claude/rules/external-sdk-integration.md`.

### Differentiation (what caro can do that they cannot)

- **Deterministic, offline, shell-aware hot path.** The 52-pattern engine
  is exactly the specialized L1 the generic gateways lack; no LLM, no
  network, no Redis in the enforcement path. Single static Rust binary.
- **One brain, many mounts.** The same `SafetyValidator` already backs the
  CLI, the library (ADR-022), `caro scan` (ADR-023), the hook guest
  (ADR-036), and the MCP tool (ADR-015). Proxy mode is a fourth mount, not
  a second implementation — verdicts are identical across all of them.
- **Published, versioned schemas.** ADR-041's events plus this ADR's
  `caro-assessment/1` deny payload give SIEMs and agents a stable contract;
  every analog's agent-facing deny surface is unversioned.
- **Self-correctable denials** (Decision D3) — no analog does this.

---

## Decision

Ship **`caro mcp-proxy`**: a stdio JSON-RPC pass-through that wraps any
downstream stdio MCP server, intercepts only `tools/call` requests whose
tool matches an intercept rule, validates the extracted command string(s)
with the existing `SafetyValidator`, and forwards, denies, or (in monitor
mode) logs — as a pure subprocess with no daemon and no cross-invocation
state.

```
host app (Claude Desktop / any MCP client)
   │ stdio                      spawns
   ▼                              │
caro mcp-proxy ── stdio ──▶ downstream MCP server ──▶ shell/files/network
   │
   └──▶ SafetyValidator verdict per intercepted call
        └──▶ caro-events JSONL (ADR-041 sink)
```

Host-side installation is a config edit only:
`"command": "caro", "args": ["mcp-proxy", "--", "npx", "-y", "@modelcontextprotocol/server-shell"]`.

### D1 — No MCP SDK in v1 (framing-only passthrough)

The proxy does **not** take the rmcp dependency. stdio transport is
newline-delimited JSON-RPC; passthrough needs line framing plus the shape
of exactly one method (`tools/call`). Implementation: read a line, try
`serde_json::from_str::<serde_json::Value>`; if `method == "tools/call"`
and the tool matches an intercept rule, act; **otherwise forward the
original raw line byte-identically** (never re-serialize — downstream
servers must see exactly what the client sent, and vice versa). Malformed
lines are forwarded verbatim in both directions with a warning event: the
proxy must never be a new failure point.

This keeps v1 at zero new dependencies (serde_json and tokio are in-tree),
and — because no external SDK is added — the
`external-sdk-integration.md` build-spike gate does not bind this ADR.
rmcp remains ADR-015/038's concern; when their spike lands, the proxy MAY
migrate to typed structs behind the same tests.

### D2 — Intercept rules (and the drift failure mode)

```
McpProxyConfig { mode, intercept: Vec<InterceptRule>, events_file? }
InterceptRule  { tool_name: String /* glob */, command_args: Vec<String> }
```

Built-in defaults (used when no `--policy` file given): tool-name globs
`{bash,shell,exec,execute_command,run_command,run_shell_command,terminal,*_shell,shell_*}`
with argument keys `{command, cmd, script, args, input}`. Each matching
argument that is a string (or array of strings, joined) is assessed;
multiple commands in one call take the **worst** verdict.

Failure mode 5 (rename drift) is answered by design, not by guessing:
for every *non-intercepted* `tools/call`, the proxy runs a cheap
name-and-shape heuristic (tool name contains an exec-ish stem, or any
configured `command_args` key is present); on a hit it emits an
`intercept_missed` warning event (monitor and enforce modes alike) so
drift is visible in the audit stream on day one. `--strict-tools`
escalates the heuristic to deny-unmatched for hardened deployments.
Assessing every string argument of every tool was rejected (Alternative
A4): it breaks the zero-false-positive discipline on non-shell strings.

### D3 — Deny is an in-band tool result, not a protocol error

A denied call is answered by the proxy (downstream never sees it) with a
**successful JSON-RPC response** whose `result` is a spec-legal tool
result: `isError: true`, `content[0]` a one-line human summary, and
`structuredContent` carrying the versioned payload:

```rust
/// src/safety/mod.rs — next to SafetyDecision
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DenyPayload {
    pub schema: String,          // "caro-assessment/1"
    pub decision: SafetyDecision, // verbatim, already Serialize
    pub command_hash: String,     // SHA-256; raw command never echoed
    pub mode: ProxyMode,          // enforce | monitor
    pub proxy_version: String,    // caro semver
}
```

This solves analog failure mode 2 by construction: the model sees *why*
(risk tier, matched pattern names, reason) inside the tool result and can
self-correct in its next attempt — a JSON-RPC error would be swallowed by
most agent frameworks as a transport fault. `HumanGate` routing maps to
deny-with-`reason: "human approval required"` in v1 (protocol-native
approval gates are ADR-038's MRTR scope); `AutoApprove` and `AsyncLog`
forward; `Block` denies. Verdict vocabulary is unchanged from ADR-020 —
no fifth verdict is invented (McpVanguard's SHADOW-BLOCK is subsumed by
monitor mode).

### D4 — Modes and events

- `--mode enforce` (default): behavior above.
- `--mode monitor`: every call forwards untouched; verdicts are only
  emitted as events (`decision_made` with `actor: "policy"`, decision
  values as if enforcing). Adoption path mirrors the analogs'
  monitor→strict rollout without content mutation (mutating forwarded
  results is an injection surface — never done in any mode).
- Events reuse ADR-041 verbatim: per intercepted call the proxy emits
  `assessment_completed` and `decision_made` into `--events-file`;
  `run_started` once at proxy start with the additive mode value
  `"mcp_proxy"` (legal under caro-events/1 additive-only rules). No
  `execution_*` events — the proxy does not execute; the downstream
  server's execution is attributed by correlating `command_hash`.
  Events remain hash-only per ADR-041.

### D5 — Lifecycle (redundant-initialization answer)

One proxy process per host-config entry, living exactly as long as the
host keeps stdin open. `SafetyValidator` patterns compile once at startup
(existing behavior); config and policy are read once; there is no
per-call setup. Child server is spawned at start, `wait()`ed on stdin
EOF/child exit, and its stderr is passed through. No state survives the
process; nothing is shared between proxies. This matches the finalized
spec's stateless core — the proxy never needs to track a session.

### New types (all Serialize + Deserialize + JsonSchema from day one)

| Type | Module | Fields / contract |
|---|---|---|
| `ProxyMode` | `src/models/mod.rs` | `Enforce \| Monitor`; `#[serde(rename_all = "snake_case")]`; `Default = Enforce` |
| `InterceptRule` | `src/models/mod.rs` | `tool_name: String` (glob), `command_args: Vec<String>`; `Default` = built-ins above |
| `McpProxyConfig` | `src/models/mod.rs` | `mode: ProxyMode`, `intercept: Vec<InterceptRule>`, `events_file: Option<PathBuf>`; `#[serde(default)]` on `UserConfiguration` as `mcp_proxy`, mirrored in the builder's three sites |
| `DenyPayload` | `src/safety/mod.rs` | as in D3; construction only via `DenyPayload::from_decision(&SafetyDecision, &str_cmd, ProxyMode)` which hashes and never stores the raw command |

### Minimal file set (one new file, no new modules)

| # | File | Change |
|---|------|--------|
| 1 | `Cargo.toml` | feature `mcp-proxy = []` (no new deps), **not** in `default` for v1 |
| 2 | `src/models/mod.rs` | `ProxyMode`, `InterceptRule`, `McpProxyConfig`; `UserConfiguration` + Default + builder ×3 |
| 3 | `src/safety/mod.rs` | `DenyPayload` + constructor; `JsonSchema` derive on `SafetyDecision`/`SuggestedRouting` (one line each) |
| 4 | `src/main.rs` | `McpProxy` clap subcommand (feature-gated): `--mode`, `--policy`, `--events-file`, `--strict-tools`, trailing `-- <cmd>…` |
| 5 | `src/cli/mcp_proxy.rs` (new file, existing module — `scan.rs` precedent) | ~250-line passthrough loop: spawn child, two pump directions, intercept match, assess, deny/forward, event emission |
| 6 | `src/bin/generate-schema.rs` + `docs/schemas/caro-assessment-1.schema.json` | emit and commit the deny-payload schema |
| 7 | `tests/mcp_proxy.rs` | integration tests below |

### Exit code / output contract (what machines depend on)

- **Process exit codes unchanged**: `0` — clean shutdown (stdin EOF or
  child clean exit); `1` — startup/config failure, with one greppable
  stderr line `caro: mcp-proxy: <detail>`; a child that exits nonzero
  propagates its code. No new codes minted (ADR-024's registry, when it
  lands, assigns dedicated ones; ADR-041 precedent).
- **Wire contract**: every non-intercepted message crosses the proxy
  byte-identically, both directions. Intercepted-and-allowed calls are
  forwarded as the original raw line. Only denied calls are answered by
  the proxy, and only with the D3 shape.
- **Schema stability**: within `caro-assessment/1`, fields are never
  removed or renamed; consumers must ignore unknown fields. Machine
  schema at `docs/schemas/caro-assessment-1.schema.json`, CI-diffed.
- **Events**: exactly the ADR-041 contract; this ADR adds only the
  additive `mode: "mcp_proxy"` value and the `intercept_missed` warning
  event (additive variant, same envelope).

### Integration tests (known input → deterministic JSON + exit code)

Downstream stand-in: a ~40-line test helper binary (`tests/bin/`
fixture) that answers `initialize`/`tools/list`/`tools/call` with canned
JSON and records everything it receives to a temp file — no LLM, no
network; static patterns keep verdicts deterministic.

1. **Byte-identical passthrough**: `initialize` → `tools/list` round
   trip; recorder file byte-equal to sent lines; proxy exits `0` on EOF.
2. **Allowed exec call**: `tools/call run_command {"command":"echo ok"}`
   → helper receives it; client receives helper's canned result; events
   contain `assessment_completed` + `decision_made(allow)`.
3. **Denied exec call**: `{"command":"sudo rm -rf /"}` (built-in Critical
   pattern) → recorder proves downstream never saw it; response has
   `isError: true` and `structuredContent` that validates against
   `caro-assessment-1.schema.json`; `decision.suggested_routing ==
   "block"`; proxy stays alive and exits `0` at EOF.
4. **Monitor mode**: same dangerous input with `--mode monitor` →
   forwarded; events record the deny verdict; response is the helper's.
5. **Zero-FP guarantee**: `tools/call write_note {"text":"how to use rm
   -rf safely"}` (non-intercepted tool) → forwarded untouched, no
   assessment events.
6. **Drift heuristic**: call to unmatched tool `shell_exec_v2` with a
   `command` arg → forwarded, `intercept_missed` event present; with
   `--strict-tools` → denied.
7. **Malformed line**: non-JSON line from client → forwarded verbatim;
   proxy neither crashes nor responds; warning event emitted.
8. **Hash-only**: sentinel secret in a denied command never appears in
   events file or deny payload; `command_hash` equals SHA-256 of the
   command.

All via `assert_cmd` + `tempfile` (existing dev-deps) + the `jsonschema`
dev-dep ADR-041 already introduces.

## Out of scope (v2+ / other ADRs)

- HTTP / SSE / Streamable-HTTP transports and any hosted-gateway mode,
  auth, JWT/JWKS — stdio only; a daemon is against the constraint.
- Protocol-native human-approval gates (MRTR) and Tasks-extension
  special-casing — ADR-038; Tasks messages pass through opaquely.
- `tools/list` / `initialize` metadata inspection (server-side prompt
  injection, tool poisoning) — real, but a different threat model;
  candidate for v2 alongside ADR-038's `clientInfo` work.
- Response-direction filtering / DLP on results returning to the agent.
- Rate budgets, session anomaly detection, circuit breaking — ADR-031.
- Policy-file layering and per-tier remapping — ADR-040 composes here
  once implemented (`--policy` v1 accepts only `McpProxyConfig`).
- LLM semantic scoring layer — deliberately never in the deny hot path.
- Multi-server federation, registries, tool namespacing — gateway
  platforms' territory; explicitly avoided per Hermes 07-30 ("do not
  build orchestration").

## Alternatives considered

1. **Build on the rmcp SDK now** — rejected for v1: passthrough needs
   framing plus one method shape; the SDK triggers the build-spike gate,
   MSRV/license checks, and transitive-dep risk for no v1 gain. Migration
   path stays open behind the same tests.
2. **Validator-tool only (ADR-015/038), no proxy** — rejected as the sole
   integration: the Jul 27 audit's ~1.2 unauthorized actions per run is
   direct evidence that voluntary call-outs are bypassed in practice.
3. **JSON-RPC protocol error on deny** (McpVanguard's choice) — rejected:
   conflates policy with transport failure and hides the reason from the
   model; in-band `isError` tool results keep the agent self-correcting.
4. **Assess every string argument of every tool** — rejected: guarantees
   false positives on non-shell strings (prose mentioning `rm -rf`),
   violating the zero-FP discipline that differentiates caro.
5. **Fork mcp-guardian** (Apache-2.0, Rust, compatible direction) —
   rejected: dormant since Apr 2025, approval flow is GUI-coupled, and
   caro's value is the validator, not the proxy plumbing (~250 lines).

## Consequences

**Positive.** Caro becomes installable in front of *any* agent that
speaks MCP — Claude Desktop, LangGraph, llama.cpp — via one config line,
with the deterministic engine in the path rather than beside it; denials
teach the model instead of stalling it; every intercepted call lands in
the ADR-041 audit stream, giving the assess→decide chain for commands
caro never executed; zero new dependencies keeps the v1 PR small enough
to review in one sitting.

**Negative / risks.** A framing-level proxy must track spec evolution by
hand until rmcp adoption (mitigated: only `tools/call` is interpreted;
everything else is opaque). Interception rules are a curated list —
genuinely novel exec-tool names bypass silently in default mode
(mitigated: `intercept_missed` heuristic event + `--strict-tools`).
stdout-based stdio framing quirks in exotic servers (non-newline-delimited
batches) would fall back to verbatim forwarding — safe, but unassessed;
documented limitation.

**Neutral.** Validation-discipline gates: this is a scope ADR, not an
implementation PR; the five gates (incl. devil's-advocate review) bind
the implementation PR that graduates it. Exit codes and all existing CLI
behavior untouched; feature is off-by-default (`mcp-proxy` flag) until a
release ADR flips it.

## Landing instructions

Produced on an unattended run; **not committed** (git-workflow rule). To
land: `bin/sk-new-feature "mcp safety gateway proxy"`, move this file
onto the branch, open a PR titled `docs(adr): ADR-043 mcp safety gateway
proxy scope`. Renumber per `adr-numbering.md` if another 043 merges
first.
