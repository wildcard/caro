# Scope: Structured `CommandAssessmentPayload` Output

**Generated**: 2026-06-03 (automated `caro-research--scoping-process` task)  
**Feature**: Structured Safety Assessment Payload  
**Roadmap item**: Structured Assessment Payloads — v2.0.0 integration prerequisites  
**Proposed ADR**: ADR-017  
**Depends on**: `SafetyDecision` (already exists), `ValidationResult` (already exists),
`--output json` flag (already wired in `CliResult`)

---

## Phase 1 — Feature Research

### What problem it solves and for whom

Caro generates POSIX commands and validates them with a 52-pattern safety
engine. When `--output json` is used today, the emitted `CliResult` payload
contains safety information as a plain string:

```json
{
  "generated_command": "rm -rf /tmp/old",
  "blocked_reason": "Command blocked due to Critical risk: ...",
  "requires_confirmation": false,
  ...
}
```

This is insufficient for three real downstream consumers:

1. **Audit dashboards / SIEM** — need machine-readable risk level, matched
   pattern IDs, and a tamper-evident evidence hash. A string in
   `blocked_reason` cannot be reliably parsed across caro versions.

2. **Approval queues (Cloudflare `waitForApproval()`, n8n, AWS Step Functions)**
   — need a structured routing decision (`allow` / `async_log` / `human_gate`
   / `block`) with metadata to present to the approver. The current output
   requires callers to string-parse `blocked_reason` to derive routing.

3. **CI pipeline scripts** — need `decision`, `risk_level`, and `matched_patterns`
   as JSON keys they can `jq`-query, not as embedded natural-language strings.

### Competitive architecture: what ships today

**Devenex (launched June 2, 2026)**  
Four-artifact model per governed action. Each action produces:

| Artifact | Purpose | Key fields |
|----------|---------|------------|
| Intent Record | What the agent wanted | intent, requester_id, timestamp |
| Execution Plan | What was evaluated | planned_steps[], risk_assessment |
| Governed Execution | The decision + policy eval | decision, policy_refs[], confidence |
| Execution Evidence | Outcome record | exit_code, stdout_hash, evidence_id |

All four artifacts carry a `schema_version` field and an `evidence_id` that
chains them together. The evidence record is SHA-256 hashed and the hash is
logged to an immutable audit trail.

**Microsoft Agent Governance Toolkit (AGT) — `PolicyResult`**  
Emitted by the `AgentOS` package (p99 < 0.1ms). Each action yields:
```json
{
  "allowed": true,
  "policies_evaluated": ["shell-exec-safe-v1"],
  "violations": [],
  "confidence": 0.99,
  "agent_identity": "did:caro:abc123",
  "timestamp": "2026-06-03T..."
}
```
`policies_evaluated` is a list of named policy documents that were consulted —
this enables the OWASP Agentic Top 10 mapping claim (each policy maps to a
taxonomy entry). AGT does NOT emit the original command or intent in this
payload — it assumes the caller retained those.

**Cloudflare `waitForApproval()` (Workflows v2, GA June 2026)**  
The pause primitive expects to receive a structured payload that the approver UI
can render. It does not prescribe a schema, but Cloudflare's own examples use:
```json
{
  "type": "approval_request",
  "command": "...",
  "risk_level": "high",
  "reason": "...",
  "callback_url": "..."
}
```
The approver POSTs `{ "approved": true }` to `callback_url` to resume.

### Why existing implementations are limited / experimental

1. **Devenex** — cloud-dependent; not a subprocess call. Requires an Abacus
   agent runtime. Cannot run in a shell or CI without a network call to the
   Devenex API. The four-artifact model is excellent but over-engineered for
   a lightweight CLI tool.

2. **Microsoft AGT** — framework-specific hooks (`LangChain`, `CrewAI`). The
   `PolicyResult` does not carry the original command or intent — it is
   designed to complement the caller's own context, not stand alone. Also the
   caro `governance` module is Phase 0 only; no wrapping code exists yet.

3. **Cloudflare** — the approval primitive is a Workflows concept; the payload
   schema is informal. Callers must define their own contract.

4. **None** of these targets shell command execution specifically. All assume
   an agent _framework_ is wrapping them. Caro is invoked as a raw subprocess —
   the output must be fully self-contained.

### Structured output contract of competitor implementations

The market is converging on these fields as minimal requirements:

| Field | Devenex | AGT | Cloudflare |
|-------|---------|-----|-----------|
| schema_version | ✅ | ✅ | ❌ |
| timestamp | ✅ | ✅ | ❌ |
| intent / prompt | ✅ | ❌ | ✅ |
| decision (enum) | ✅ | `allowed: bool` | ✅ |
| risk_level | ✅ | via violations | ✅ |
| matched_patterns / policies | ✅ | ✅ | ❌ |
| confidence | ✅ | ✅ | ❌ |
| evidence_hash | ✅ | ❌ | ❌ |
| routing_suggestion | ❌ | ❌ | ❌ |

**Caro's unique advantage**: it is the only one that can provide all of these
as a standalone subprocess call — no daemon, no framework dependency,
no network.

### Session/context lifecycle

All three competitors handle this the same way: each invocation is stateless.
No session initialization is needed. The output payload IS the audit record
per invocation.

For caro: each `caro <query>` invocation produces exactly one
`CommandAssessmentPayload`. There is no session state to manage.

---

## Phase 2 — Competitive Differentiation

### What they get right that we should replicate

1. **`schema_version` on every payload** — Devenex and AGT both version their
   output. Downstream parsers can gate on `schema_version: 1` and be shielded
   from future field additions.

2. **Decision as an enum, not a bool or string** — AGT uses `allowed: bool`
   (too coarse). Cloudflare uses a string. Caro already has `SuggestedRouting`
   (`AutoApprove / AsyncLog / HumanGate / Block`) — expose it directly.

3. **Evidence hash for tamper-evidence** — Devenex chains artifacts via a
   SHA-256 hash. For caro, `sha256(intent + command + matched_patterns +
   timestamp)` is sufficient. It proves the payload hasn't been post-processed
   to change the risk level.

4. **Matched patterns as objects, not strings** — AGT uses `policies_evaluated`
   as named strings (better for lookups). Caro's `ValidationResult.matched_patterns`
   is `Vec<String>` of plain descriptions. Elevating to `Vec<PatternEvidence>`
   with a `pattern_id` makes SIEM queries exact.

### Bugs / design gaps in competitor implementations we should avoid

1. **Devenex's four-artifact model is too heavy for a CLI tool.** A single
   `CommandAssessmentPayload` with well-chosen fields is better. The
   "Execution Evidence" artifact belongs in a future `caro run --track`
   integration; out of scope for this ADR.

2. **AGT omits the original intent from its payload.** This is a gap — audit
   reviewers need to see what the agent asked for, not just whether the command
   was allowed. We include `intent_summary`.

3. **None of them provide `owasp_categories`** — mapping matched patterns to
   OWASP Agentic Top 10 categories. This is low-cost for caro (a static map
   from pattern name to category) and creates a differentiating claim.

4. **Cloudflare's payload is informal** (no schema). Any caro caller targeting
   Cloudflare `waitForApproval()` would need an adapter. We should make the
   `CommandAssessmentPayload` directly usable as a Cloudflare approval payload
   by naming `decision` and `risk_level` top-level fields.

### Caro's unique positioning

- **Offline and universal**: works in air-gapped CI, SSH sessions, embedded
  systems. Devenex, AGT, and Cloudflare all require network or framework hooks.
- **POSIX-first**: shell-specific pattern matching (BSD vs GNU variants, shell
  type detection). No competitor does shell-specific safety at this granularity.
- **Sub-millisecond deterministic**: pattern-based, no LLM in the safety path.
  p99 < 1ms even on the embedded backend.
- **Single-binary**: one subprocess call returns a complete audit artifact.
  No agent runtime required.

### Existing infrastructure that already covers part of this

| Needed | Existing | Gap |
|--------|----------|-----|
| Risk level enum | `RiskLevel` in `models/mod.rs` | none — reuse directly |
| Routing decision | `SuggestedRouting` in `models/mod.rs` | not included in `CliResult` JSON |
| Matched patterns | `ValidationResult.matched_patterns: Vec<String>` | needs elevation to `Vec<PatternEvidence>` |
| Confidence | `ValidationResult.confidence_score: f32` | not included in `CliResult` JSON |
| Timestamp | `chrono::Utc::now()` used elsewhere | not included in `CliResult` JSON |
| JSON output path | `--output json` → `serde_json::to_string_pretty(&result)` | payload shape insufficient |
| Governance module | `src/governance/mod.rs` (Phase 0 spike) | no audit event emission yet |
| Evidence hash | — | new: `sha256(intent+cmd+patterns+timestamp)` |
| OWASP categories | — | new: static map from pattern description keywords |
| Request ID | — | new: `uuid::Uuid::new_v4()` per invocation |

---

## Phase 3 — Scope Definition

### ADR-017: `CommandAssessmentPayload` — Governed Execution Output Contract

**Status**: Proposed  
**Date**: 2026-06-03  
**Authors**: `caro-research--scoping-process`  
**Target**: Community + Enterprise  

#### Context

Caro's `--output json` today serializes `CliResult`, which represents the
command-generation UX layer. When external systems (audit dashboards, approval
queues, CI scripts) consume caro output, they need a structured _safety
decision record_ — not a UX result. The two concerns should separate.

The agent governance market (Devenex, Microsoft AGT, Cloudflare Workflows) is
converging on structured per-action audit records as the standard output
contract for any safety/governance layer. Caro's pattern engine produces all
the inputs for such a record; they are currently buried in `CliResult` strings.

#### Decision

Add `CommandAssessmentPayload` as a new serializable type in `src/models/`.
Include it as `assessment: CommandAssessmentPayload` in every `CliResult` when
`--output json` or `--output yaml` is requested. The plain-text output path is
unaffected.

No new module. No daemon. No network. Pure subprocess call.

#### Consequences

- **Positive**: Every `--output json` invocation becomes a self-contained audit
  record. Downstream integrations (Cloudflare, n8n, SIEM, CI) can `jq`-query
  the payload directly.
- **Positive**: Unblocks the AGT `governance` module Phase 1 — the
  `CommandAssessmentPayload` becomes the payload emitted to AGT's audit sink.
- **Positive**: Enables OWASP mapping without a separate document — the
  `owasp_categories` field in the payload is a live claim.
- **Negative**: `CliResult` JSON grows by ~200 bytes per invocation. Scripts
  that parse the old JSON shape will need updating (breaking change — handled
  with `schema_version`).
- **Risk**: `evidence_hash` computation requires `sha2` crate. Check MSRV
  compatibility before landing (add to Cargo.toml as optional if needed).

#### Alternatives considered

1. **Keep `CliResult` as-is, add a separate `--output assessment-json` flag** —
   rejected because it creates two JSON shapes for the same invocation, and
   callers would need to know which flag to use. A single `--output json`
   with `schema_version: 1` is cleaner.

2. **Emit to a webhook instead of stdout** — deferred to a follow-on ADR
   (`--on-critical=webhook`). This ADR only concerns the payload _shape_, not
   the delivery mechanism.

3. **Use the Devenex four-artifact model directly** — rejected as too heavy.
   A single self-contained payload is sufficient for a subprocess-call tool.

---

### New Types

All new types go in `src/models/mod.rs` alongside `RiskLevel`, `SuggestedRouting`, etc.

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Per-invocation structured safety audit record.
///
/// Emitted as `result.assessment` in `--output json` / `--output yaml` responses.
/// Schema-versioned from day one; downstream parsers should gate on
/// `schema_version == 1`.
///
/// # Example (JSON)
/// ```json
/// {
///   "schema_version": 1,
///   "request_id": "01HZ...",
///   "timestamp": "2026-06-03T10:00:00Z",
///   "intent_summary": "delete temp files older than 7 days",
///   "generated_command": "find /tmp -mtime +7 -delete",
///   "shell": "Bash",
///   "safety": {
///     "risk_level": "Moderate",
///     "reason": "Detected 1 dangerous pattern(s) at Moderate risk level",
///     "suggested_routing": "AsyncLog",
///     "matched_patterns": [...],
///     "confidence": 0.99
///   },
///   "decision": "async_log",
///   "matched_patterns": [...],
///   "owasp_categories": ["tool_misuse"],
///   "evidence_hash": "sha256:abc123...",
///   "backend_used": "static",
///   "latency_ms": 2
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CommandAssessmentPayload {
    /// Payload schema version. Increment on breaking field changes.
    pub schema_version: u8,
    /// UUID v4 for correlating multi-step pipelines.
    pub request_id: String,
    /// UTC timestamp of the safety decision.
    pub timestamp: DateTime<Utc>,
    /// The user's original natural-language prompt (truncated to 512 chars
    /// to bound payload size).
    pub intent_summary: String,
    /// The command caro generated and evaluated.
    pub generated_command: String,
    /// Shell type used for pattern matching.
    pub shell: ShellType,
    /// Full structured safety decision from the validator.
    pub safety: SafetyDecision,
    /// Matched patterns with OWASP evidence linkage.
    pub matched_patterns: Vec<PatternEvidence>,
    /// Final routing decision (derived from `safety.suggested_routing`).
    pub decision: RoutingDecision,
    /// OWASP Agentic Top 10 categories implicated by matched patterns.
    pub owasp_categories: Vec<OwaspCategory>,
    /// SHA-256 hash of `(intent_summary + generated_command + pattern_ids + timestamp)`.
    /// Allows downstream consumers to verify the payload was not altered.
    pub evidence_hash: String,
    /// Backend that generated the command (static / embedded / ollama / vllm).
    pub backend_used: String,
    /// Total wall-clock time from request to decision, in milliseconds.
    pub latency_ms: u64,
}

/// A single matched safety pattern with provenance and taxonomy linkage.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PatternEvidence {
    /// Machine-readable pattern identifier (snake_case).
    /// Derived from the pattern description, e.g. `rm_rf_root`.
    pub pattern_id: String,
    /// Human-readable description of what the pattern detects.
    pub description: String,
    /// Risk level assigned to this pattern.
    pub risk_level: RiskLevel,
    /// OWASP Agentic Top 10 category this pattern maps to, if known.
    pub owasp_category: Option<OwaspCategory>,
}

/// Final routing decision for this command. Mirrors `SuggestedRouting`
/// but uses `snake_case` JSON values for wire compatibility with
/// Cloudflare Workflows and n8n approval nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoutingDecision {
    /// Command is safe — proceed without logging.
    Allow,
    /// Command has moderate risk — log and proceed asynchronously.
    AsyncLog,
    /// Command has high risk — pause for human approval before execution.
    HumanGate,
    /// Command is critically dangerous — block unconditionally.
    Block,
}

impl From<SuggestedRouting> for RoutingDecision {
    fn from(r: SuggestedRouting) -> Self {
        match r {
            SuggestedRouting::AutoApprove => Self::Allow,
            SuggestedRouting::AsyncLog => Self::AsyncLog,
            SuggestedRouting::HumanGate => Self::HumanGate,
            SuggestedRouting::Block => Self::Block,
        }
    }
}

/// OWASP Top 10 for Agentic Applications (2025/2026 edition).
/// See: https://owasp.org/www-project-top-10-for-large-language-model-applications/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OwaspCategory {
    /// OWASP AA1 — Prompt injection / goal hijacking
    GoalHijacking,
    /// OWASP AA2 — Unauthorized tool use or misuse
    ToolMisuse,
    /// OWASP AA3 — Identity abuse / agent impersonation
    IdentityAbuse,
    /// OWASP AA4 — Memory or context poisoning
    MemoryPoisoning,
    /// OWASP AA5 — Cascading agent failures
    CascadingFailures,
    /// OWASP AA6 — Rogue/uncontrolled agent execution
    RogueAgentExecution,
    /// OWASP AA7 — Over-privileged actions (privilege escalation)
    OverPrivilegedActions,
    /// OWASP AA8 — Supply chain abuse
    SupplyChainAbuse,
    /// OWASP AA9 — Sensitive data exfiltration
    DataExfiltration,
    /// OWASP AA10 — Insecure output / command injection
    InsecureOutput,
}
```

---

### Minimal File Changes

No new modules. Changes touch four files:

#### 1. `src/models/mod.rs` (+~120 lines)

Add `CommandAssessmentPayload`, `PatternEvidence`, `RoutingDecision`,
`OwaspCategory` structs/enums as shown above.

Add a free function:
```rust
/// Build a `CommandAssessmentPayload` from a `ValidationResult`.
///
/// Called from `cli/mod.rs` after safety validation completes.
pub fn build_assessment_payload(
    intent: &str,
    command: &str,
    shell: ShellType,
    validation: &crate::safety::ValidationResult,
    safety_level: SafetyLevel,
    backend_used: &str,
    latency_ms: u64,
) -> CommandAssessmentPayload { ... }
```

This function:
- Derives `SafetyDecision` via `SafetyDecision::from_validation_result`
- Maps `validation.matched_patterns: Vec<String>` → `Vec<PatternEvidence>`
  using a static `PATTERN_OWASP_MAP: &[(&str, OwaspCategory)]` lookup table
  (see `src/safety/owasp_map.rs` below — one new small file, but avoids
  polluting `models`)
- Computes `evidence_hash` as `sha256(intent + command + pattern_ids.join(",")
  + timestamp.to_rfc3339())`
- Sets `schema_version: 1`, generates `request_id` via `uuid::Uuid::new_v4()`

#### 2. `src/safety/owasp_map.rs` (new file, ~60 lines)

Static mapping from pattern description keywords to `OwaspCategory`. Example:

```rust
pub fn owasp_category_for_pattern(description: &str) -> Option<OwaspCategory> {
    let lower = description.to_lowercase();
    if lower.contains("rm -rf") || lower.contains("disk wipe") || lower.contains("dd if=")
        || lower.contains("fork bomb") {
        Some(OwaspCategory::RogueAgentExecution)
    } else if lower.contains("chmod") || lower.contains("sudo") || lower.contains("privilege") {
        Some(OwaspCategory::OverPrivilegedActions)
    } else if lower.contains("network") || lower.contains("curl") || lower.contains("wget")
        || lower.contains("backdoor") {
        Some(OwaspCategory::DataExfiltration)
    } else if lower.contains("mkfs") || lower.contains("fdisk") || lower.contains("format") {
        Some(OwaspCategory::ToolMisuse)
    } else {
        None
    }
}
```

This is the smallest justification for a new file — OWASP mapping logic
should not live in `models` (no domain knowledge) or `safety` (patterns.rs
is already dense). Adding it to `safety/` as a sibling is appropriate.

#### 3. `src/cli/mod.rs` (~30 lines changed)

In `run_with_args`, after safety validation, call `build_assessment_payload`:

```rust
let assessment = crate::models::build_assessment_payload(
    &prompt,
    &generated.command,
    shell,
    &validation,
    safety_level,
    &generated.backend_used,
    generation_time.as_millis() as u64,
);
```

Add `assessment: CommandAssessmentPayload` to `CliResult`:

```rust
pub struct CliResult {
    // ... existing fields unchanged ...
    /// Structured safety audit record. Present in --output json/yaml responses.
    /// Omitted from plain-text output.
    pub assessment: CommandAssessmentPayload,
}
```

#### 4. `Cargo.toml` (~3 lines)

Add `uuid` and `sha2` dependencies:

```toml
uuid = { version = "1", features = ["v4"] }
sha2 = "0.10"
```

Both have MSRV ≤ 1.83, MIT/Apache-2.0 license. No MSRV bump required.
Confirm with `cargo check --no-default-features` before landing.

---

### Exit Code / Output Contract

Exit codes are **unchanged** by this ADR. The `assessment` field enriches the
existing JSON payload; it does not change when caro exits non-zero.

| Exit code | Meaning | `assessment.decision` |
|-----------|---------|----------------------|
| 0 | Command allowed or dry-run | `allow` or `async_log` |
| 2 | Command requires confirmation | `human_gate` |
| 3 | Command blocked | `block` |

Machines and scripts that currently `jq '.blocked_reason'` on caro's JSON
output should migrate to `jq '.assessment.decision'`. The old `blocked_reason`
field remains in `CliResult` (no removal in this ADR) so scripts continue to
work; the migration is additive.

The canonical machine-readable field to key on going forward is:
```
.assessment.decision  ∈ { "allow", "async_log", "human_gate", "block" }
```

---

### Integration Tests

Add to `tests/assessment_payload.rs` (or nearest existing integration test file):

```rust
// Test 1: Safe command → allow decision, empty patterns
// Input: caro --output json --dry-run "list files in current directory"
// Expected: exit 0, .assessment.decision == "allow",
//           .assessment.matched_patterns == [], schema_version == 1

// Test 2: Critical command → block decision
// Input: caro --output json --dry-run "delete everything"
//   (generates: rm -rf /)
// Expected: exit 3, .assessment.decision == "block",
//           .assessment.owasp_categories contains "rogue_agent_execution",
//           .assessment.evidence_hash starts with "sha256:"

// Test 3: High-risk command, moderate safety level → human_gate
// Input: caro --output json --safety moderate --dry-run "drop production database"
// Expected: exit 2, .assessment.decision == "human_gate",
//           .assessment.safety.suggested_routing == "HumanGate"

// Test 4: Evidence hash is deterministic for same inputs
// Compute hash for the same (intent, command, patterns, timestamp) twice.
// Must be identical. (Use a fixed timestamp in test helper.)

// Test 5: --output plain does NOT emit JSON (regression guard)
// Ensure plain-text path is unchanged.
```

All tests must use `--dry-run` so no command executes. All tests assert on
`serde_json::Value` parsed from stdout so they are backend-agnostic.

---

### Explicit Out-of-Scope

The following are **not** in this ADR and belong in follow-on issues:

| Item | Rationale |
|------|-----------|
| `--on-critical=webhook` delivery | Payload shape first, delivery mechanism second. ADR-018 candidate. |
| Cloudflare Workflows reference integration | Requires the webhook delivery mechanism above. |
| OWASP coverage mapping document (`docs/security/`) | Positioning work; requires manual review of all 52 patterns. |
| AGT Phase 1 wrapping (`src/governance`) | The `assessment` payload becomes the event body. Depends on this ADR landing first. |
| `caro audit` batch command | Separate ADR candidate; reuses the same `CommandAssessmentPayload` type. |
| `execution_evidence` (post-run outcome record) | Deferred — requires `caro run` integration and exit-code capture. |
| Evidence hash verification tooling | Nice-to-have but not needed for audit integration. |
| SIEM/OpenTelemetry export | Depends on webhook delivery (above). |

---

### Constraints Satisfied

- ✅ **Reuses existing validator/safety/config infrastructure** — `SafetyDecision` and
  `ValidationResult` are passed in, not re-derived.
- ✅ **All new types serializable from day one** — `Serialize + Deserialize + JsonSchema`
  on every new type.
- ✅ **Pure subprocess call** — no daemon, no state, no network. One caro invocation →
  one payload on stdout.
- ✅ **Solves the failure mode found in Phase 1** — the `decision` enum, `evidence_hash`,
  and `schema_version` fields eliminate every string-parsing fragility in
  current downstream consumers by design, not workaround.
- ✅ **No new modules** except the small `src/safety/owasp_map.rs` (justified by
  domain separation; all alternatives were messier).

---

## Recommended Next Steps

1. **Open feature branch**: `bin/sk-new-feature "feat(models): CommandAssessmentPayload structured output ADR-017"`
2. **Add ADR-017** to `docs/adr/` using `ADR-TEMPLATE.md` and the text above.
3. **Add `uuid` and `sha2`** to `Cargo.toml` and verify MSRV with
   `cargo check --no-default-features --features embedded-cpu`.
4. **Implement types** in `src/models/mod.rs` + `src/safety/owasp_map.rs`.
5. **Wire into `CliResult`** in `src/cli/mod.rs` — one call to `build_assessment_payload`.
6. **Write integration tests** (5 cases above) before wiring to keep TDD discipline.
7. **Update README**: add a `## Machine-readable output` section showing the
   `--output json` payload shape with the new `assessment` key.

Estimated complexity: **M** (2–3 days). No new inference, no new backend, no
new module beyond the OWASP map file. This is a serialization + type design
task.

---

*Generated by the `caro-research--scoping-process` scheduled task.*  
*Feature name resolved autonomously from today's Hermes digest (June 3, 2026).*  
*`[FEATURE NAME]` placeholder in the skill template was not populated — resolved*
*to "Structured Assessment Payload" based on Hermes Opportunity 2 (top-priority item).*
