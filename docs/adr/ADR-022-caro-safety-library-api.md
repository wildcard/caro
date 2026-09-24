# ADR-022 — `caro-safety`: Synchronous Embeddable Safety Library

**Date**: 2026-06-11
**Status**: Proposed
**Authors**: `caro-research--scoping-process` (automated research/scoping agent)
**Relates to**: ADR-020 (tiered approval protocol), ADR-015 (MCP safety server)
**Research**: arXiv 2603.20953 (OAP), Hermes 2026-06-11 Opportunity D

---

## Context

Caro's safety validation engine (`src/safety/mod.rs`) is the core competitive
asset: 52+ pre-compiled regex patterns, quote-context analysis, BSD/GNU
platform awareness, CVE-derived rules, and a zero-false-positive record. It
validates commands in <2ms.

The engine is only accessible via the CLI subprocess model. Agent frameworks
that want deterministic command-intent validation at sub-10ms latency —
without spawning a process per command — have no path in.

The **Open Agent Passport** (arXiv 2603.20953, Apache 2.0, March 2026) has
formalized this gap as a research primitive gaining adoption in agent
frameworks. OAP's reference implementation measures median 53ms and has no
shell-context intelligence. Caro's pattern engine is demonstrably superior
but invisible to in-process callers.

Three structural gaps prevent embedding:

1. `validate_command` is `async fn` with zero I/O — forces a Tokio runtime
   on callers unnecessarily.
2. `SafetyDecision` has no `schema_version`, `decision_id`, or `command_hash`
   — callers cannot detect format changes or anchor audit records.
3. No C FFI surface — Python/Node/Go agent frameworks cannot call in-process.

---

## Decision

Extract `caro-safety` as a new **workspace crate** with:

1. A synchronous `validate(command, options) -> SafetyAssessment` entry point
2. A versioned `SafetyAssessment` output type (`schema_version`, `decision_id`, `command_hash`)
3. A C FFI surface (`#[no_mangle]` exports + `cbindgen`-generated header)
4. Remove vestigial `async` from `validate_command` in `src/safety/mod.rs`

This is **not a rewrite**. The pattern database, matching logic, and type
definitions stay in `src/safety/`. `caro-safety` is a thin re-export +
sync wrapper + FFI surface.

### New types

**`SafetyAssessment`** — versioned, stable output type:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SafetyAssessment {
    pub schema_version: u8,           // always 1 in this version
    pub decision_id: String,          // UUID v4 — correlates with ApprovalRequest (ADR-020)
    pub allowed: bool,
    pub risk_level: RiskLevel,
    pub suggested_routing: SuggestedRouting,
    pub matched_patterns: Vec<String>,
    pub reason: String,
    pub confidence: f64,
    pub command_hash: String,         // SHA-256 hex of command string
    pub assessed_at: String,          // RFC 3339
}
```

**`ValidateOptions`** — caller-controlled policy:
```rust
#[derive(Debug, Clone, Default)]
pub struct ValidateOptions {
    pub shell: ShellType,             // default: ShellType::default()
    pub safety_level: SafetyLevel,    // default: Moderate
    pub custom_patterns: Vec<DangerPattern>,
    pub allowlist_patterns: Vec<String>,
}
```

**`validate()`** — primary entry point (synchronous, infallible):
```rust
pub fn validate(command: &str, options: &ValidateOptions) -> SafetyAssessment
```

**C FFI** (generated header `include/caro_safety.h`):
- `caro_safety_validate(command, shell_type, safety_level) -> *CaroSafetyResult`
- `caro_safety_free(result)`
- `caro_safety_version() -> *const c_char`

### Files that change

| File | Change |
|---|---|
| `Cargo.toml` (workspace) | Add `caro-safety` to `[workspace.members]` |
| `caro-safety/Cargo.toml` | **New** workspace crate |
| `caro-safety/src/lib.rs` | **New** — `validate()`, `SafetyAssessment`, `ValidateOptions` |
| `caro-safety/src/ffi.rs` | **New** — C FFI exports |
| `caro-safety/cbindgen.toml` + `build.rs` | **New** — header generation |
| `src/safety/mod.rs` | Remove `async` from `validate_command`; add `decision_id`, `command_hash`, `schema_version` to `SafetyDecision` |
| `src/lib.rs` | Add `SafetyValidator`, `SafetyConfig`, `SafetyDecision`, `DangerPattern` to stable `pub use` surface |
| `caro-safety/tests/integration.rs` | **New** — 12 deterministic tests |

No changes to backends, CLI execution path, or any inference code.

### Exit code contract (`caro validate <command>` subcommand)

| Code | Meaning |
|---|---|
| 0 | `allowed: true` |
| 1 | `allowed: false`, routing = Block |
| 2 | `allowed: false`, routing = HumanGate |
| 10 | Input error |

### Explicit out-of-scope (v1)

- Python/Node/Go wrapper packages (FFI header is sufficient for ctypes/cffi/cgo)
- `caro-safety` published to crates.io independently (after API stabilization in v1.5)
- HMAC-signed audit records (requires key management; SHA-256 hash satisfies Article 12)
- Async Rust API (callers use `spawn_blocking`)
- WASM target (requires `no_std` audit)

---

## Consequences

**Positive:**
- Caro's safety engine becomes embeddable in any language via C FFI at <10ms.
- `schema_version` enables callers to detect breaking changes without parsing
  error strings.
- `command_hash` (SHA-256) satisfies EU AI Act Article 12 audit logging
  requirements without HMAC complexity.
- `decision_id` correlates directly with `ApprovalRequest::request_id` in
  ADR-020's approval webhook — the two ADRs compose without coupling.

**Negative / risks:**
- C FFI introduces memory safety surface. All FFI functions are wrapped in
  `std::panic::catch_unwind`.
- AGPL-3.0 applies to `caro-safety`. Proprietary agent frameworks embedding
  it must obtain a commercial license. This is intentional.
- Removing `async` from `validate_command` is a semver-minor breaking change
  for any caller using `.await` — all internal callers drop the `.await`,
  compile-time catch.

---

## Alternatives Considered

**A — Expose a `block_on` wrapper in `src/safety/`**: Keep `async` on
`validate_command`, add a `validate_sync` wrapper in the same file. Rejected:
hides the vestigial async and clutters the core module. The `async` has no
semantic purpose.

**B — Publish as a separate top-level crate (not a workspace member)**:
Rejected for v1: workspace membership gives access to internal types without
a public dependency. After API stabilization, a separate publish is the plan.

**C — Implement OAP spec exactly (schema-compatible output)**:
OAP schema is Apache 2.0 — we can produce OAP-compatible JSON. Rejected for
v1: OAP's `audit_signature` requires HMAC, which adds key management scope.
The `command_hash` field covers the same audit trail use case without a shared
secret. OAP compatibility can be added as a `--oap-compat` flag later.

---

## References

- arXiv 2603.20953 — "Before the Tool Call" paper
- ADR-020 — Tiered Approval Protocol (`HumanGate` webhook)
- ADR-015 — MCP Safety Server
- `.claude/rules/external-sdk-integration.md` — build spike checklist
- `.claude/plans/caro-safety-library-api-scope.md` — full research and scoping document

---

*Generated by `caro-research--scoping-process` scheduled agent · 2026-06-11*
*Source signals: arXiv 2603.20953; Hermes Market Scan 2026-06-11 Opportunity D*
