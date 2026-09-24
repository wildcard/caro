# ADR-020 — Tiered Approval Protocol: Wiring `HumanGate` to Runtime Behavior

**Date**: 2026-06-09
**Status**: Proposed
**Authors**: `caro-research--scoping-process` (automated research/scoping agent)
**Relates to**: ADR-015 (MCP Safety Server), Hermes Market Scan 2026-06-09 Opportunity D
**Depends on**: ADR-015 (`SafetyAssessmentOutput` schema must be merged first)

---

## Context

### The Gap: `HumanGate` Is a Dead Enum Variant

Caro's model layer has declared four routing tiers since at least v1.4.0:

```rust
pub enum SuggestedRouting {
    AutoApprove,   // safe — execute immediately
    AsyncLog,      // moderate — execute and log
    HumanGate,     // high/critical — pause for human review
    Block,         // critical/strict — refuse entirely
}
```

`SuggestedRouting::from_risk_and_safety()` correctly computes `HumanGate` for
High-risk commands at Strict/Moderate safety levels, and for Critical commands
at Permissive safety level. But `src/cli/mod.rs` never reads `suggested_routing`
at runtime. The execution path branches only on two derived booleans:

- `requires_confirmation` — calls `risk_level.requires_confirmation(safety_level)`
- `blocked_reason` — calls `risk_level.is_blocked(safety_level)`

`HumanGate` commands currently fall into the "requires confirmation" bucket,
which means they are blocked unless `--confirm` is passed. There is no mechanism
to route them to an external approver and resume.

### Why This Matters Now

Three independent market signals converged in the week of June 2–9, 2026:

**MetaMask Agent Wallet** (launched June 8, 2026) shipped the first
production-grade tiered approval model at the financial-action layer:
- *Guard Mode* (default): daily spend limits + protocol allowlists. Transactions
  outside the policy pause for 2FA approval before executing.
- *Beast Mode* (opt-in): fewer interruptions, but 2FA still triggers on
  confirmed-malicious transactions.

The architectural pattern is identical to what Caro needs at the shell layer:
a pre-configured allowlist of "always safe" patterns, tiered interruption on
policy edges, and a deterministic block for confirmed-dangerous commands.

**OpenAI Agents SDK** (Human-in-the-Loop docs, current) formalizes the
in-process pause-and-resume contract:
- `RunToolApprovalItem` — emitted when a tool call hits an approval rule
- `interruptions` array — all pending approvals returned at end of turn
- `RunState.toString()` / `RunState.fromString()` — serializable state for
  async resumption
- `approve(interruption)` / `reject(interruption)` — decision injection API
- `{ alwaysApprove: true }` — sticky decisions that survive serialization

The failure mode of the OpenAI pattern: it requires **in-process integration**.
The orchestrator must be written in JavaScript and use the OpenAI SDK's state
machine. External tools called as subprocesses have no way to participate.

**Caro's structural advantage**: it is a pure subprocess. The correct model for
a stateless subprocess is not pause-and-resume within a process — it is
**signal on exit and let the pipeline decide**. The `--approval-hook` flag
combines this with synchronous webhook polling for teams that want in-band
approval without restructuring their agent.

---

## Decision

Wire `SuggestedRouting::HumanGate` to actual runtime behavior via two new
mechanisms that work together or independently:

### Mechanism 1 — `--on-human-gate <policy>` (pipeline signal)

New CLI flag controlling what happens when `suggested_routing == HumanGate`
and no `--approval-hook` is configured:

| Policy | Behavior | Exit code |
|--------|----------|-----------|
| `exit2` (default) | Print structured refusal to stderr, exit immediately | 2 |
| `confirm` | Existing behavior — prompt for `--confirm` flag | — |
| `skip` | Silently skip execution, continue pipeline | 0 |

Exit code 2 is already specified in ADR-015's exit code table for
`suggested_routing: human_gate`. This closes the loop.

### Mechanism 2 — `--approval-hook <url>` (synchronous webhook)

When set, instead of exiting, Caro POSTs an `ApprovalRequest` to `<url>` and
polls for an `ApprovalResponse` within `--approval-timeout <seconds>` (default: 30).

**POST payload** (`ApprovalRequest`):
```json
{
  "schema_version": 1,
  "request_id": "<uuid-v4>",
  "command": "sudo rm -rf /var/log",
  "risk_level": "high",
  "suggested_routing": "human_gate",
  "matched_patterns": ["sudo elevation", "recursive deletion"],
  "reason": "Detected 2 dangerous pattern(s) at High risk level",
  "confidence": 0.95,
  "assessed_at": "2026-06-09T10:00:00Z",
  "command_hash": "sha256:...",
  "context": {
    "cwd": "/home/user",
    "shell": "bash",
    "safety_level": "moderate"
  }
}
```

**Expected response** (`ApprovalResponse`):
```json
{
  "schema_version": 1,
  "request_id": "<uuid-v4 — must match>",
  "approved": true,
  "reviewer_id": "kobi@example.com",
  "note": "Expected maintenance task",
  "expires_at": "2026-06-09T10:05:00Z"
}
```

- If `approved: true` — Caro proceeds to execute the command; exit code 0.
- If `approved: false` — Caro exits with code 2 and prints the `note` to stderr.
- If the hook times out — Caro exits with code 2 (treated as rejected).
- If the hook returns a non-2xx status — Caro exits with code 11 (hook error).
- `request_id` mismatch in the response → code 11.

### Mechanism 3 — `--allow-pattern <glob>` (static allowlist bypass)

Analogous to MetaMask's protocol allowlist. A command that matches an
`--allow-pattern` entry is **exempt from `HumanGate`** — it falls back to
`AutoApprove` regardless of `risk_level`, provided `suggested_routing` is not
`Block`.

- Multiple `--allow-pattern` flags are ANDed as a set.
- Patterns use standard glob syntax (via the `glob` crate, already in tree
  for CaroML).
- `Block` is not bypassable by allowlist — allowlist only lifts `HumanGate`.
- Allowlist entries are logged in the `SafetyAssessmentOutput.matched_patterns`
  field with an `[allowlist]` prefix for audit purposes.

Config file equivalent (`.caro.toml`):
```toml
[approval]
hook_url = "https://approval.example.com/hook"
timeout_secs = 30
allow_patterns = ["git commit *", "cargo build *", "npm run *"]
on_human_gate = "exit2"
```

---

## Architecture & Data Flow

```
caro "delete old logs"
        │
        ▼
   CommandGenerator (LLM/static)
        │ generated: "sudo find /var/log -mtime +30 -delete"
        ▼
   SafetyValidator::validate_command()
        │
        ▼
   SafetyDecision { risk_level: High, suggested_routing: HumanGate, ... }
        │
        ├─ command matches --allow-pattern? ──── YES ──▶ execute (exit 0)
        │
        ├─ --approval-hook set?
        │         YES ──▶ POST ApprovalRequest ──▶ poll response
        │                      │ approved        │ rejected/timeout
        │                      ▼                 ▼
        │                  execute (0)        exit 2
        │
        └─ --on-human-gate policy
                 exit2 ──▶ stderr + exit 2
                 confirm ──▶ existing --confirm prompt
                 skip   ──▶ exit 0 silently
```

No daemon, no persistent state. `request_id` is a fresh UUID per invocation.
The approval hook server is entirely out-of-scope — Caro is the client only.

---

## New Types

All new types in `src/approval/mod.rs`.

### `ApprovalRequest`
```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRequest {
    pub schema_version: u8,           // always 1
    pub request_id: String,           // UUID v4
    pub command: String,
    pub risk_level: RiskLevel,
    pub suggested_routing: SuggestedRouting,
    pub matched_patterns: Vec<String>,
    pub reason: String,
    pub confidence: f64,
    pub assessed_at: String,          // RFC 3339
    pub command_hash: String,         // SHA-256 hex
    pub context: ApprovalRequestContext,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRequestContext {
    pub cwd: Option<String>,
    pub shell: ShellType,
    pub safety_level: SafetyLevel,
}
```

### `ApprovalResponse`
```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalResponse {
    pub schema_version: u8,
    pub request_id: String,           // must match ApprovalRequest::request_id
    pub approved: bool,
    pub reviewer_id: Option<String>,
    pub note: Option<String>,
    pub expires_at: Option<String>,   // RFC 3339 — hook may set expiry for caching
}
```

### `OnHumanGatePolicy`
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnHumanGatePolicy {
    #[default]
    Exit2,
    Confirm,
    Skip,
}
```

### `AllowlistEntry`
```rust
#[derive(Debug, Clone)]
pub struct AllowlistEntry {
    pub pattern: glob::Pattern,
    pub raw: String,   // preserved for audit logging
}
```

`ApprovalRequest` is a superset of `SafetyAssessmentOutput` (ADR-015) — it
adds `context` and `request_id`. Both share `schema_version`, `command`,
`risk_level`, `matched_patterns`, `reason`, `confidence`, `assessed_at`,
`command_hash`. The two types remain separate because their consumers and
lifecycles differ.

---

## Files That Change

| File | Change |
|------|--------|
| `Cargo.toml` | Add `uuid = { version = "1", features = ["v4"] }` (optional under `approval-hook` feature); verify `reqwest` or `ureq` availability for HTTP POST |
| `src/approval/mod.rs` | **NEW** — `ApprovalRequest`, `ApprovalResponse`, `OnHumanGatePolicy`, `AllowlistEntry`, `ApprovalClient` |
| `src/lib.rs` | `pub mod approval;` |
| `src/cli/mod.rs` | Wire `suggested_routing` → `approval` module; add `--approval-hook`, `--on-human-gate`, `--allow-pattern`, `--approval-timeout` flags; `ParsedArgs` + `IntoCliArgs` trait extensions |
| `src/main.rs` | Add new `Cli` flags (clap) for the four new args |
| `src/config/mod.rs` | `[approval]` section in `.caro.toml` config struct |
| `tests/approval_integration.rs` | **NEW** — deterministic integration tests (see below) |

No changes to `src/safety/`, `src/models/`, or any backend. All new behavior
is in the `approval` module which sits between the validator output and the
execution decision.

**HTTP client note**: The `reqwest` crate is likely already in-tree (check
`Cargo.toml`). If not, the build spike must verify MSRV compatibility before
the feature PR. `ureq` is a lighter alternative (no async, blocking only) and
sufficient for a synchronous webhook poll.

---

## Exit Code Contract

These codes extend the ADR-015 table and must not change within a major version:

| Exit code | Meaning |
|-----------|---------|
| 0 | Executed (safe, or approved via hook, or allowlist bypass) |
| 1 | Blocked — `suggested_routing: block` |
| 2 | HumanGate hit — no approval obtained (exit2 policy, rejected, or timeout) |
| 3 | AsyncLog — executed and logged |
| 10 | Input error |
| 11 | Hook error (non-2xx, request_id mismatch, parse failure) |

---

## Integration Tests

`tests/approval_integration.rs` — all deterministic, no LLM, no network:

| Test | Input | Flags | Expected behavior | Exit |
|------|-------|-------|-------------------|------|
| `test_human_gate_exit2_default` | `sudo rm -rf /var` | none | stderr message, no exec | 2 |
| `test_human_gate_allowlist_bypass` | `sudo rm -rf /var` | `--allow-pattern "sudo rm *"` | command executes | 0 |
| `test_block_not_bypassable_by_allowlist` | `rm -rf /` | `--allow-pattern "rm *"` | still blocked | 1 |
| `test_human_gate_skip_policy` | `sudo rm -rf /var` | `--on-human-gate skip` | silent skip | 0 |
| `test_approval_hook_approved` | `sudo rm -rf /var` | `--approval-hook <mock-url>` | mock returns `approved: true`, executes | 0 |
| `test_approval_hook_rejected` | `sudo rm -rf /var` | `--approval-hook <mock-url>` | mock returns `approved: false` | 2 |
| `test_approval_hook_timeout` | `sudo rm -rf /var` | `--approval-hook <mock-url> --approval-timeout 1` | mock hangs, times out | 2 |
| `test_approval_hook_bad_request_id` | `sudo rm -rf /var` | `--approval-hook <mock-url>` | mock returns wrong `request_id` | 11 |
| `test_approval_request_schema_stable` | any HumanGate | hook | `schema_version == 1` in POST body | — |
| `test_allowlist_audit_prefix` | `git commit -m "fix"` | `--allow-pattern "git commit *"` | `matched_patterns` contains `[allowlist] git commit *` | 0 |

Mock HTTP server for hook tests uses `wiremock` or `httpmock` (both compile-time,
no network). Add to `[dev-dependencies]` only.

---

## Competitive Differentiation

### What to replicate

- **MetaMask Guard Mode**: static allowlist bypass. Implemented as `--allow-pattern`.
  Teams pre-configure known-safe commands; those never trigger the hook.
- **OpenAI SDK sticky decisions**: `{ alwaysApprove: true }` for a tool.
  Equivalent: allowlist entry. The pattern file in `.caro.toml` is the sticky
  decision store.
- **OpenAI SDK request_id correlation**: every approval request has a stable ID
  that the response must echo. Implemented in `ApprovalResponse::request_id`.

### Their gaps we design around

- **OpenAI SDK**: in-process only. Our design works as a subprocess via exit
  codes — compatible with any shell, CI system, or agent framework that can
  call a subprocess.
- **MetaMask**: browser-native, cloud-resident approval. Our hook is a generic
  HTTP endpoint — works with Slack bots, PagerDuty, custom approval UIs, or a
  `caro-approver` CLI yet to be written.
- **Both**: their approvals are ephemeral (per-session). Our `expires_at` field
  in `ApprovalResponse` lets the hook server implement caching for repeated
  identical commands, reducing approval fatigue without baking caching into
  Caro itself.

### What Caro can do that others cannot

- **Offline allowlist**: `--allow-pattern` requires zero network. Pre-approved
  commands never hit the hook.
- **Exit-code native**: pipelines that use `set -e` or `|| exit` get
  `HumanGate` semantics for free via exit code 2, no SDK required.
- **Universal**: any agent framework, any language, any CI system that can call
  a subprocess benefits from the same approval contract.
- **Composable**: `caro-mcp` (ADR-015) can surface `suggested_routing:
  human_gate` to an MCP orchestrator, which then invokes the approval hook via
  its own mechanism. The two ADRs compose without coupling.

---

## Explicit Out-of-Scope (v1 of this feature)

| Not in scope | Rationale |
|--------------|-----------|
| Approval hook server / UI | Caro is the client. Server implementations are ecosystem work. |
| `alwaysApprove` session-level sticky decisions | Allowlist file covers the same use case durably. Session stickiness adds state. |
| Async (fire-and-forget) hook mode | Out-of-process async requires a daemon or queue. Stateless subprocess principle. |
| Per-pattern approval expiry cache | `expires_at` is in the response contract so future versions can use it; not read in v1. |
| HMAC signature verification on hook response | Security hardening for v2; v1 trusts TLS + `request_id` correlation. |
| `caro-approver` CLI (approval server reference impl) | Separate tool, separate repo, out of scope for this ADR. |
| MCP tool call approval integration | ADR-015 surfaces `suggested_routing`; the MCP client decides what to do with it. |

---

## Consequences

**Positive:**
- `SuggestedRouting::HumanGate` becomes a first-class runtime concept, not dead
  code. The four-tier model is complete.
- Exit code 2 creates a language-agnostic, subprocess-native approval signal.
  Any orchestrator that calls `caro` can gate on it without SDK integration.
- `--allow-pattern` closes the "approval fatigue" failure mode before it
  opens. Teams configure common commands once; the hook only sees genuine edge
  cases.
- `ApprovalRequest` extends `SafetyAssessmentOutput` (ADR-015) without
  breaking it. Both schemas are stable and versioned.

**Negative / risks:**
- HTTP client dependency (`reqwest` or `ureq`) adds binary size. Gated behind
  `approval-hook` feature flag — main `caro` binary unaffected if feature not
  enabled.
- The approval hook introduces a network call in the execution path. `--approval-timeout`
  must be set defensively in CI environments. Default 30s may be too long for
  latency-sensitive pipelines — document recommended `--approval-timeout 5` for CI.
- `request_id` requires UUID generation. `uuid = "1"` with `v4` feature is a
  well-understood dependency, but must pass the external-sdk-integration spike
  checklist per `~/.claude/rules/external-sdk-integration.md`.

---

## Alternatives Considered

### A: Extend `--confirm` flag to accept a URL
Rejected. `--confirm` is a boolean; extending it to accept a URL conflates
two semantics. The new `--approval-hook` is clearer and composable.

### B: Implement a blocking STDIN approval prompt
Rejected for agentic use cases. A blocking TTY prompt requires an interactive
terminal; agents calling `caro` as a subprocess have no TTY. The webhook
model works headlessly.

### C: Emit `HumanGate` and let the caller deal with it (exit 2 only)
Valid and already covered by `--on-human-gate exit2`. This ADR adds the hook
on top — the policy flag makes the exit-only behavior the explicit default, not
an undocumented side effect.

### D: Store allowlist in the OS keychain
Over-engineered. `.caro.toml` is the right config surface for allowlist patterns.

---

## Implementation Checklist (for the engineer picking this up)

**Build spike (PR N, merge before feature PR):**
- [ ] Verify `reqwest` or `ureq` in `Cargo.toml`; if absent, add under
  `approval-hook` optional feature and run the external-sdk-integration spike
  checklist from `.claude/rules/external-sdk-integration.md`
- [ ] Verify `uuid = { version = "1", features = ["v4"] }` resolves against MSRV 1.85
- [ ] Add `wiremock` or `httpmock` to `[dev-dependencies]`
- [ ] `cargo check --no-default-features` passes
- [ ] `cargo check --features approval-hook` passes

**Feature PR (after spike merges):**
- [ ] `src/approval/mod.rs` — all new types + `ApprovalClient::post_and_poll()`
- [ ] `src/config/mod.rs` — `[approval]` section
- [ ] `src/cli/mod.rs` — wire `suggested_routing == HumanGate` → approval module
- [ ] `src/main.rs` — four new clap args
- [ ] `tests/approval_integration.rs` — 10 deterministic tests (table above)
- [ ] `cargo test --features approval-hook` passes
- [ ] Publish JSON schema to `docs/schema/approval-request-v1.json` and `docs/schema/approval-response-v1.json`
- [ ] `CHANGELOG.md` entry under `## [Unreleased]`
- [ ] Verify ADR-015 is merged first (depends on `SafetyAssessmentOutput`)

---

*Generated by `caro-research--scoping-process` scheduled agent · 2026-06-09*
*Source signals: Hermes Market Scan 2026-06-09 Opportunity D; MetaMask Agent Wallet (June 8 launch); OpenAI Agents SDK HITL docs*
