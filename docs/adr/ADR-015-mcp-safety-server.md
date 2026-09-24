# ADR-015 — caro-mcp-server: MCP Safety Tool

**Date**: 2026-05-29  
**Last reviewed**: 2026-07-21 (automated freshness pass by `caro-research--scoping-process`; the MCP 2026-07-28 RC removed the initialize handshake/session this ADR assumed — see **ADR-038** for the full staleness table, RC alignment, and the MRTR/Tasks approval-gate scope. Tool surface, `SafetyAssessmentOutput`, and exit codes carry forward unchanged. Prior review 2026-06-12: ADR-020 and ADR-023 depend on this — `SafetyAssessmentOutput` schema must merge first; ADR-023 defines a local copy in `src/cli/scan.rs` to unblock `caro scan` without blocking on the rmcp build spike)  
**Status**: Proposed  
**Author**: Automated research/scoping agent (`caro-research--scoping-process`)  
**Supersedes**: —  
**Relates to**: Hermes Market Scan 2026-05-29, ROADMAP.md v2.0.0 items

---

## Context

As of May 2026, MCP (Model Context Protocol) has become the de-facto transport
standard for agent-to-tool communication. AWS MCP Server GA (May 6), Cloudflare
Agents Week (May 2026), OpenAI Agents SDK sandbox launch (April 15), and
Microsoft Agent Governance Toolkit (April 2026) all ship MCP-native tooling.
The ecosystem signal is unambiguous: agents expect to call safety validators
via MCP, not via bespoke SDK integrations.

The OpenAI Agents SDK's sandbox launch explicitly left "pluggable safety
backend" open. RAMPART (Microsoft, May 20) established safety-in-CI as a
developer expectation. Two CVE-grade events — the Microsoft Semantic Kernel
RCE disclosure and the Google Antigravity sandbox escape — validated that a
deterministic, out-of-process safety layer is a sound defense-in-depth
primitive, not over-engineering.

Caro today has:
- 52+ compiled regex patterns across Critical/High/Moderate/Low
- `SafetyValidator` callable as a library (`caro::safety`)
- `SafetyDecision` (structured, Serde-serializable, carries `SuggestedRouting`)
- Tokio async runtime already in-tree
- `serde_json = "1"` already in `Cargo.toml`
- MSRV 1.85

Caro today does **not** have:
- An MCP transport layer
- A `--output json` flag for the primary `caro` binary
- A documented, machine-readable structured output contract

This ADR scopes the minimum work to ship `caro-mcp-server` as a standalone
binary exposing Caro's safety validator as an MCP tool, plus `--output json`
on the existing `caro` CLI for RAMPART/CI integration.

---

## Decision

Ship two independently useful artifacts in a single PR:

### Artifact 1 — `caro-mcp` binary (MCP Safety Server)

A new Cargo binary (`[[bin]] name = "caro-mcp"`) that starts an MCP server
over stdio transport, exposing two tools:

1. `validate_command` — validate one shell command, return `SafetyDecision` as JSON
2. `validate_batch` — validate a list of commands, return `Vec<SafetyDecision>`

The binary depends on `rmcp` under a new optional feature flag `mcp-server`.
No architectural wrapping yet — the MCP handler is a thin shim over the
existing `SafetyValidator::new(config).validate_command()` call path.

> **Version note (updated 2026-06-02):** rmcp was at `0.16` when this ADR was
> drafted. The current release is `1.7.0` — a major version bump with breaking
> API changes. The build-spike checklist below uses `"1"` (semver major) to
> pin to the 1.x line without locking to a specific minor. Verify MSRV
> compatibility at spike time; rmcp 1.x requires Rust ≥ 1.85 which matches
> caro's current MSRV.

### Artifact 2 — `--output json` on existing `caro --dry-run`

Extend the existing `--dry-run` flag to accept `--output json`, emitting a
`SafetyAssessmentOutput` JSON object to stdout. This is the RAMPART/CI
integration path for teams that call caro as a subprocess rather than via MCP.

Both artifacts share the same `SafetyAssessmentOutput` JSON schema (see
Output Contract below).

---

## Architecture & Data Flow

```
Agent / RAMPART test / CI script
          │
          ├─ MCP tool call ──────────▶  caro-mcp (stdio server)
          │                                  │
          └─ subprocess ─────────────▶  caro --dry-run --output json
                                            │
                                Both paths ▼
                         SafetyValidator::new(config)
                               .validate_command(cmd, shell)
                                        │
                             ValidationResult + SafetyDecision
                                        │
                             SafetyAssessmentOutput (JSON)
                                        │
                          ◀─────────── returned to caller
```

No daemon, no persistent state. Each call is a pure, stateless subprocess or
MCP request. Patterns are compiled once per process via `once_cell::Lazy`
(existing behavior); startup overhead is ~0 for the compiled pattern check.

---

## New Types

All new types live in `src/mcp/mod.rs` (new file). They are thin wrappers
over existing validator types — no duplication of validation logic.

### `McpValidateCommandRequest`
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct McpValidateCommandRequest {
    /// The shell command to validate. Required.
    pub command: String,
    /// Target shell. Defaults to "bash" if omitted.
    #[serde(default)]
    pub shell: ShellType,
    /// Safety policy level. Defaults to "moderate" if omitted.
    #[serde(default)]
    pub safety_level: SafetyLevel,
}
```

### `McpValidateBatchRequest`
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct McpValidateBatchRequest {
    pub commands: Vec<String>,
    #[serde(default)]
    pub shell: ShellType,
    #[serde(default)]
    pub safety_level: SafetyLevel,
}
```

### `SafetyAssessmentOutput`  ← **the unified JSON contract**
```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SafetyAssessmentOutput {
    /// Schema version for forward-compatibility.
    pub schema_version: u8,                     // always 1 for this release
    pub command: String,
    pub risk_level: RiskLevel,
    pub allowed: bool,
    pub suggested_routing: SuggestedRouting,
    pub matched_patterns: Vec<String>,
    pub reason: String,
    pub confidence: f64,
    /// RFC 3339 timestamp (UTC). Enables audit log correlation.
    pub assessed_at: String,
    /// SHA-256 hex of command bytes. Enables dedup without storing the command.
    pub command_hash: String,
}
```

`SafetyAssessmentOutput` is convertible from `(ValidationResult, SafetyDecision)`
via a `From` impl. No new validation logic — just projection.

`schema_version = 1` is pinned now so downstream consumers can gate on it.

---

## Files That Change

| File | Change |
|------|--------|
| `Cargo.toml` | Add `rmcp = { version = "1", optional = true }`; add `[[bin]] name = "caro-mcp"`; add `mcp-server = ["dep:rmcp"]` feature |
| `Cargo.lock` | Regenerate (committed) |
| `src/mcp/mod.rs` | **NEW** — MCP server handler, `SafetyAssessmentOutput`, `McpValidate*Request` |
| `src/lib.rs` | `#[cfg(feature = "mcp-server")] pub mod mcp;` |
| `src/bin/caro-mcp.rs` | **NEW** — binary entry point (`main` + rmcp server setup) |
| `src/cli/mod.rs` | Add `--output <text|json>` flag; emit `SafetyAssessmentOutput` on `--dry-run --output json` |
| `tests/mcp_integration.rs` | **NEW** — integration tests (see below) |

No new modules beyond `src/mcp/`. No changes to `src/safety/`, `src/models/`,
or any existing validation logic. The safety infrastructure is consumed as-is.

---

## Output Contract (JSON Schema)

The canonical JSON payload for a single command assessment:

```json
{
  "schema_version": 1,
  "command": "rm -rf /",
  "risk_level": "critical",
  "allowed": false,
  "suggested_routing": "block",
  "matched_patterns": ["recursive deletion of root filesystem"],
  "reason": "Detected 1 dangerous pattern(s) at Critical risk level (deletion, recursive)",
  "confidence": 1.0,
  "assessed_at": "2026-05-29T14:00:00Z",
  "command_hash": "e3b0c44298fc1c149afb..."
}
```

Exit codes for the `caro --dry-run --output json` path:

| Exit code | Meaning |
|-----------|---------|
| 0 | Safe — `allowed: true`, `suggested_routing: auto_approve` |
| 1 | Blocked — `allowed: false`, `suggested_routing: block` |
| 2 | Human gate required — `suggested_routing: human_gate` |
| 3 | Async log (allowed but flagged) — `suggested_routing: async_log` |
| 10 | Input error (empty command, unknown shell) |
| 11 | Internal validator error |

This contract is the interface that RAMPART tests, CI pipelines, and agent
frameworks will depend on. It must not change within a major version.

---

## Integration Tests

`tests/mcp_integration.rs` — known inputs → deterministic JSON + exit code:

| Test | Input | Expected `risk_level` | Expected exit |
|------|-------|-----------------------|---------------|
| `test_safe_command` | `ls -la` | `safe` | 0 |
| `test_critical_blocked` | `rm -rf /` | `critical` | 1 |
| `test_moderate_async_log` | `chmod 777 /tmp/test` | `moderate` | 3 |
| `test_high_human_gate` | `sudo rm -rf /var` | `high` | 2 (at `moderate` safety) |
| `test_json_schema_stable` | any | `schema_version == 1` | — |
| `test_batch_mixed` | `["ls", "rm -rf /"]` | both correct | — |
| `test_empty_command` | `""` | — | 10 |

All tests are deterministic — no LLM inference, no network. They call
`SafetyValidator` directly or invoke the binary as a subprocess.

---

## Competitive Differentiation

### What competitors get right (to replicate)

- **RAMPART**: structured test assertions against agent behavior. We solve
  this by making every Caro decision machine-readable via `SafetyAssessmentOutput`.
- **OpenAI Agents SDK guardrails**: fail-fast on policy breach. We solve this
  with `suggested_routing: block` + non-zero exit codes that scripts can gate on.
- **Cloudflare HITL approval**: tiered decisions. We already have
  `SuggestedRouting` with four tiers — the MCP server surfaces this as a
  first-class return value.

### Their bugs / gaps we design around

- **Microsoft AGT**: covers OWASP 10/10 but is a full governance stack
  requiring identity, cloud, and enterprise integration. **Caro's gap:** zero
  dependencies, single binary, offline.
- **RAMPART**: Python-native, no Rust SDK. Caro's MCP tool is language-agnostic.
- **Existing MCP safety tools**: none. The slot is open — ship first.
- **Design gap to solve explicitly**: most safety tools treat the *decision*
  as opaque (allowed/blocked). We surface `matched_patterns` + `reason` +
  `command_hash` so the calling agent or CI system can log, audit, and
  correlate decisions without storing the raw command.

### What Caro can do that others cannot

- **Offline**: patterns are compiled into the binary. No network, no cloud.
- **Sub-millisecond**: single regex pass, not LLM inference.
- **Universal**: any agent that speaks MCP can call it.
- **Community layer**: AGPL. Forks and improvements flow back.
- **Composable**: `caro-mcp` can be a dependency of larger governance stacks
  (Microsoft AGT, Suprbox) without competing with them.

---

## Explicit Out-of-Scope (v1 of this feature)

| Not in scope | Rationale |
|--------------|-----------|
| HTTP/SSE transport | stdio is sufficient for all known use cases; avoids TLS config overhead |
| `--on-uncertain=suspend` HITL lifecycle hook | Opportunity 3 from market scan; depends on this artifact as a prerequisite |
| OWASP Agentic Top 10 mapping | Documentation-only, tracked separately |
| `caro-check` standalone RAMPART integration | Covered by `--output json` + documented RAMPART sample; full integration in next sprint |
| OAuth / authentication on the MCP server | Not needed for stdio; applicable if HTTP transport is added later |
| Persisting assessment history | Out-of-process responsibility; callers can pipe JSON to any store |

---

## Consequences

**Positive:**
- Closes the "pluggable safety backend" slot that OpenAI SDK and Cloudflare
  opened explicitly. First-mover in the MCP safety tool category.
- `SafetyAssessmentOutput` becomes the stable, documented output contract.
  Every downstream integration (RAMPART, OTel, Datadog, audit log) is unlocked
  by this one schema.
- No changes to existing safety logic — zero regression risk.
- `rmcp` is optional; users who don't install `caro-mcp` see no binary size
  increase in the main `caro` binary.

**Negative / risks:**
- `rmcp = 1.x` (current: 1.7.0 as of 2026-06-02) must satisfy MSRV 1.85.
  The build-spike step (per `external-sdk-integration.md`) must run before any
  wrapping code is written. The API changed between 0.x and 1.x — handler
  patterns from 0.x tutorials no longer apply.
- The MCP protocol version may advance (spec v2025-03-26 / v2025-06-18 both
  supported by rmcp 1.x). Pin the major version (`"1"`) in `Cargo.toml`.

---

## Alternatives Considered

### A: Expose a JSON REST endpoint instead of MCP
Rejected. REST requires a running server process. MCP over stdio is a
subprocess — no port management, no firewall rules, composable via shell pipes.

### B: Add JSON output only (no MCP server)
Valid stepping stone but leaves the MCP slot unfilled. Given the 4–6 week
window to establish Caro as the shell-safety reference before the ecosystem
solidifies, doing both in one PR is the right call. The complexity delta
between JSON-only and JSON+MCP is small.

### C: Use the `mcp` crate instead of `rmcp`
`rmcp` is the official Anthropic-maintained Rust SDK (current: 1.7.0, hosted
at `modelcontextprotocol/rust-sdk`). It tracks the canonical spec. `mcp` is a
community fork. Use `rmcp`.

---

## Implementation Checklist (for the engineer picking this up)

**Build spike (PR N, must merge before feature PR):**
- [ ] Add `rmcp = { version = "1", optional = true }` to Cargo.toml
- [ ] Add `mcp-server = ["dep:rmcp"]` feature
- [ ] Add `pub fn mcp_smoke() {}` in `src/mcp/mod.rs` that touches rmcp's API
- [ ] `cargo check --no-default-features` passes
- [ ] `cargo check --features mcp-server` passes
- [ ] Record both results in commit body

**Feature PR (after spike merges):**
- [ ] `src/mcp/mod.rs` — `SafetyAssessmentOutput` + `McpValidate*Request` + handler
- [ ] `src/bin/caro-mcp.rs` — binary entry point
- [ ] `src/cli/mod.rs` — `--output <text|json>` + exit code contract
- [ ] `tests/mcp_integration.rs` — 7 deterministic tests (table above)
- [ ] `cargo test --features mcp-server` passes
- [ ] Publish JSON schema to `docs/schema/safety-assessment-v1.json`
- [ ] `CHANGELOG.md` entry under `## [Unreleased]`

---

*Generated by `caro-research--scoping-process` scheduled agent · 2026-05-29*
