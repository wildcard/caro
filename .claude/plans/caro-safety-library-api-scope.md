# Scope: `caro-safety` — Embeddable Library API for Pre-Action Authorization

**Produced by**: `caro-research--scoping-process` (scheduled agent)
**Date**: 2026-06-11
**Feature trigger**: Hermes Market Scan 2026-06-11 Opportunity D
**Research source**: arXiv 2603.20953 "Before the Tool Call" (OAP spec, Apache 2.0)
**ADR target**: ADR-022
**Depends on**: ADR-020 (tiered approval protocol) — shared types

---

## Executive Summary

Caro already has the best command-intent safety engine in the market. The gap is
that it is only accessible as a CLI subprocess. Agent frameworks that want
deterministic pre-action authorization at sub-10ms latency must spawn a child
process, parse stdout, and manage exit codes — clunky and slow.

This scope defines `caro-safety`: a new workspace crate that extracts Caro's
existing `safety` module behind a **stable synchronous public API** and adds a
**C FFI wrapper** so Python, Node, Go, and any other language can embed it in-process.

This is Caro's direct structural advantage over the Open Agent Passport (OAP):
where OAP proposes a new 53ms HTTP-based interceptor, Caro's approach is
deterministic, offline, sub-10ms, and embeddable with zero network dependency.

---

## Phase 1 — Feature Research

### The Open Agent Passport (OAP) — arXiv 2603.20953

**Paper**: "Before the Tool Call: Deterministic Pre-Action Authorization for
Autonomous AI Agents" (Uchibeke, March 2026). Apache 2.0 spec released on
Zenodo (DOI: 10.5281/zenodo.18901596).

**Problem it solves**: AI agents execute tool calls (shell commands, fund
transfers, database queries, sub-agent delegation) with no standard mechanism
to enforce authorization *before* the action executes. Existing approaches are:

- **Model alignment** — probabilistic, training-time, bypassed by social
  engineering 74.6% of the time in adversarial tests.
- **Post-hoc evaluation** — retrospective, batch, does not prevent execution.

Neither is deterministic pre-action enforcement.

**Core architecture** (from paper abstract + known OAP spec design):

```
tool call intent
    │
    ▼
OAP Interceptor (synchronous, in-path)
    │
    ├── evaluate against declarative policy (YAML/JSON)
    │
    ├── produce ApprovalDecision { allowed: bool, reason, matched_policies }
    │
    └── produce cryptographically signed AuditRecord { command_hash, sha256, ts }
    │
    ▼
execute (if allowed) or reject (with signed record)
```

**Performance**: median 53ms (N=1,000 decisions). This includes policy loading
(YAML parse at startup), pattern matching, and audit record signing (SHA-256 + HMAC).

**Structured output contract**: JSON `AuthorizationDecision`:
```json
{
  "schema_version": 1,
  "decision_id": "<uuid>",
  "allowed": false,
  "risk_level": "high",
  "matched_policies": ["deny-rm-recursive", "deny-sudo-elevation"],
  "reason": "Command matches 2 deny policies",
  "audit_signature": "sha256:...",
  "evaluated_at": "<rfc3339>"
}
```

**Session/context lifecycle**: Stateless. OAP loads its policy at startup
and makes synchronous decisions per tool call. No daemon.

**Why it is experimental/limited**:

1. **General but shallow**: OAP covers any tool call (API calls, DB queries,
   file operations), but has no domain-specific intelligence. Its policy is
   declarative — the operator writes regex rules. There is no built-in awareness
   of shell quoting context, BSD vs GNU flag differences, CVE patterns, or
   platform-specific hazards.

2. **53ms is the floor, not the ceiling**: The 53ms includes HTTP round-trips
   in its reference implementation — OAP is designed as a network interceptor.
   Pure in-process evaluation with pre-compiled patterns is 2–5ms.

3. **No false-positive protection**: OAP's declarative rules match literally.
   A rule `deny: rm -rf` would block `echo 'rm -rf /'` (a safe shell tutorial
   example). Caro's `is_dangerous_in_context()` — with quote-depth analysis —
   prevents this class of false positive.

4. **No MSRV or offline guarantee**: OAP's reference implementation is
   TypeScript-first. Caro's Rust implementation compiles to a single binary
   with zero runtime dependencies.

---

## Phase 2 — Competitive Differentiation

### What OAP gets right that Caro should replicate

| OAP design choice | Why to replicate | Caro equivalent today |
|---|---|---|
| Stable `schema_version` field in output | Callers pin to a version; breaking changes are signaled | Partial: `SafetyDecision` has no `schema_version` |
| Cryptographically signed audit record | EU AI Act Article 12 compliance; tamper evidence | Missing — `command_hash` in ADR-020 `ApprovalRequest` only |
| `decision_id` UUID for correlation | Correlates decision to approval request, audit log | `request_id` in ADR-020, not in `SafetyDecision` itself |
| Synchronous, pure-function interface | No async runtime requirement for callers | Missing: `validate_command` is `async fn` with zero I/O — trivially synchronizable |

### OAP bugs / design gaps Caro avoids

| OAP failure mode | Root cause | Caro's solution |
|---|---|---|
| No quote-context analysis | Literal regex matching | `is_dangerous_in_context()` counts unescaped quote depth before each match |
| No platform awareness | Language-agnostic spec | `ShellType` + BSD flavor detection in `src/safety/` |
| No CVE-derived patterns | Generic policy framework | `src/safety/cve_patterns.rs` with auto-ingested CVE YAML rules |
| 53ms latency | HTTP-based interceptor design | Pure in-process regex: <2ms for 52 patterns (benchmarked) |
| Policy requires operator authorship | No built-in dangerous-command knowledge | 52 curated patterns, zero false positives, validated test suite |
| TypeScript-only reference impl | Ecosystem choice | Rust with C FFI → bindings for any language |

### What Caro can do that OAP cannot

1. **Offline-first**: No network, no policy server, no sidecar process.
2. **Sub-10ms**: Pre-compiled `once_cell::Lazy` patterns; zero parse cost after startup.
3. **Quote-context analysis**: `echo 'rm -rf /'` is safe; OAP's literal regex flags it.
4. **BSD/GNU awareness**: `find -delete` is safe on BSD, dangerous on Linux without `-exec`. OAP has no platform context.
5. **Community pattern library**: 52+ patterns with provenance (CVE IDs, descriptions, risk levels). OAP ships empty.
6. **C FFI**: embeddable in Python (ctypes/cffi), Node (node-ffi/napi), Go (cgo) — not just Rust callers.

### What existing infrastructure already covers

The safety module (`src/safety/mod.rs`) already has:
- `SafetyValidator::validate_command()` — the core engine (async but pure CPU)
- `SafetyDecision` — structured output with `risk_level`, `suggested_routing`, `matched_patterns`, `confidence`
- `DangerPattern` / `SafetyConfig` — extensible configuration
- `validate_user_pattern()` — input validation for operator-supplied rules
- `ValidationError` — typed error enum via `thiserror`

**What's missing for an embeddable library**:
1. **No synchronous API**: `async fn validate_command` requires a Tokio runtime.
2. **No stable public re-exports**: `safety::SafetyValidator` is not in the top-level `pub use` surface of `lib.rs`.
3. **No `schema_version` on `SafetyDecision`**: Output format can change without callers knowing.
4. **No `decision_id`**: `SafetyDecision` cannot be correlated to an audit record.
5. **No C FFI header / `cbindgen` config**: Cannot call from Python or Node without spawning a subprocess.

---

## Phase 3 — Scope Definition

### New Types (all in `caro-safety/src/lib.rs`)

#### `SafetyAssessment` — versioned, stable output type

```rust
/// Versioned safety assessment returned by `caro_safety::validate`.
///
/// `schema_version` is stable within a major version of `caro-safety`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SafetyAssessment {
    pub schema_version: u8,           // always 1
    pub decision_id: String,          // UUID v4
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

`SafetyAssessment` is produced from the existing `SafetyDecision` via:
`SafetyAssessment::from_decision(decision, command, uuid)` — a pure conversion
with no new validation logic.

#### `ValidateOptions` — caller-controlled policy

```rust
#[derive(Debug, Clone)]
pub struct ValidateOptions {
    pub shell: ShellType,
    pub safety_level: SafetyLevel,
    pub custom_patterns: Vec<DangerPattern>,
    pub allowlist_patterns: Vec<String>,
}

impl Default for ValidateOptions {
    fn default() -> Self {
        Self {
            shell: ShellType::default(),
            safety_level: SafetyLevel::Moderate,
            custom_patterns: Vec::new(),
            allowlist_patterns: Vec::new(),
        }
    }
}
```

#### `validate()` — primary entry point

```rust
/// Synchronously validate a shell command against Caro's pattern database.
///
/// - Synchronous (no async runtime required)
/// - Deterministic (same input → same output)
/// - Sub-10ms for any command against the full built-in pattern set
/// - Zero network dependencies
/// - Infallible: always returns SafetyAssessment, never panics on valid UTF-8 input
pub fn validate(command: &str, options: &ValidateOptions) -> SafetyAssessment {
    // Uses tokio::runtime::Builder::new_current_thread().build().block_on()
    // OR (preferred) removes async from validate_command — see note below.
}
```

**Async-to-sync strategy**: `validate_command` is `async fn` but has zero
`.await` points on I/O — it is purely CPU work. **Remove `async`** from
`validate_command` in `src/safety/mod.rs`. This is the cleanest fix and there
is no semantic reason for it to be async. Impact: any caller using `.await` on
`validate_command` drops the `.await` — a compile-time change, not a behavior
change.

#### C FFI surface (`caro-safety/src/ffi.rs`)

```c
/* caro_safety.h — generated by cbindgen */
typedef struct CaroSafetyResult {
  int allowed;                // 1 = allowed, 0 = blocked
  int risk_level;             // 0=Safe,1=Low,2=Moderate,3=High,4=Critical
  int suggested_routing;      // 0=AutoApprove,1=AsyncLog,2=HumanGate,3=Block
  const char* reason;         // NUL-terminated UTF-8
  const char* decision_id;    // UUID v4 string
  const char* command_hash;   // SHA-256 hex
  double confidence;
} CaroSafetyResult;

const CaroSafetyResult* caro_safety_validate(
    const char* command,
    int shell_type,            // 0=Bash,1=Zsh,2=Fish,3=Sh
    int safety_level           // 0=Permissive,1=Moderate,2=Strict
);
void caro_safety_free(const CaroSafetyResult* result);
const char* caro_safety_version(void);
```

All FFI functions are wrapped in `std::panic::catch_unwind`. The header is
generated by `cbindgen` at build time — no manual sync.

---

### Minimal File Changes

| File | Change | Notes |
|---|---|---|
| `Cargo.toml` (workspace) | Edit — add `caro-safety` to `[workspace.members]` | One line |
| `caro-safety/Cargo.toml` | **New** | Deps: `caro` (path), `sha2`, `uuid`, `serde`, `schemars` |
| `caro-safety/src/lib.rs` | **New** | `validate()`, `SafetyAssessment`, `ValidateOptions` |
| `caro-safety/src/ffi.rs` | **New** | C FFI exports |
| `caro-safety/cbindgen.toml` | **New** | Outputs `caro-safety/include/caro_safety.h` |
| `caro-safety/build.rs` | **New** | Invokes cbindgen |
| `src/safety/mod.rs` | Edit — remove `async` from `validate_command`; add `decision_id` + `command_hash` + `schema_version` to `SafetyDecision` | Additive + one desync |
| `src/lib.rs` | Edit — add `pub use safety::{SafetyValidator, SafetyConfig, SafetyDecision, ValidationResult, DangerPattern, SafetySection};` | Stable surface |
| `caro-safety/tests/integration.rs` | **New** | 12 deterministic tests |

**No new modules in `src/`. No changes to any backend, CLI, or inference code.**

---

### Exit Code / Output Contract

New `caro validate <command>` subcommand (separate from `caro <query>`):

| Exit code | Meaning |
|-----------|---------|
| 0 | `allowed: true` — safe or allowlisted |
| 1 | `allowed: false`, routing = Block |
| 2 | `allowed: false`, routing = HumanGate |
| 10 | Input error (empty command, bad flags) |

JSON output on stdout (always, for `caro validate`):
```json
{
  "schema_version": 1,
  "decision_id": "...",
  "allowed": false,
  "risk_level": "Critical",
  "suggested_routing": "Block",
  "matched_patterns": ["rm -rf / (recursive force delete of root)"],
  "reason": "Detected 1 dangerous pattern(s) at Critical risk level (deletion, recursive)",
  "confidence": 1.0,
  "command_hash": "sha256:abc...",
  "assessed_at": "2026-06-11T10:00:00Z"
}
```

---

### Integration Tests (`caro-safety/tests/integration.rs`)

| Test | Input | Options | Expected |
|---|---|---|---|
| `test_rm_rf_root_blocked` | `rm -rf /` | default | `allowed=false, risk=Critical, routing=Block` |
| `test_echo_rm_rf_safe` | `echo 'rm -rf /'` | default | `allowed=true, risk=Safe` (quote-context) |
| `test_sudo_elevation_human_gate` | `sudo rm -rf /var` | strict | `routing=HumanGate` |
| `test_schema_version_is_1` | any | default | `schema_version == 1` |
| `test_decision_id_is_uuid_v4` | any | default | valid UUID v4 format |
| `test_command_hash_sha256` | `ls -la` | default | `command_hash == sha256hex("ls -la")` |
| `test_allowlist_bypasses_human_gate` | `sudo echo test` | allowlist=`["sudo echo *"]` | `allowed=true, "[allowlist]"` in patterns |
| `test_block_not_bypassable_by_allowlist` | `rm -rf /` | allowlist=`["rm *"]` | still `allowed=false, routing=Block` |
| `test_custom_pattern_respected` | `kubectl delete -n prod` | custom High pattern | `risk=High, allowed=false` |
| `test_assessment_json_roundtrip` | `rm -rf /` | default | serialize → deserialize → equal |
| `test_ffi_validate_blocked` | `rm -rf /` via C FFI | Bash, Moderate | `result->allowed == 0, risk_level == 4` |
| `test_ffi_validate_safe` | `ls -la` via C FFI | default | `result->allowed == 1` |

---

### Explicit Out-of-Scope (v1)

| Not in scope | Rationale |
|---|---|
| Python/Node/Go wrapper packages | FFI header is sufficient for ctypes/cffi/cgo. Official bindings in follow-up. |
| `caro-safety` on crates.io as independent publish | Requires API stabilization period first. Target v1.5.0. |
| HMAC-signed audit records | Requires shared secret / key management. `command_hash` (SHA-256) is sufficient for Article 12. Defer to v2. |
| Async Rust API | Callers wanting async use `spawn_blocking(|| validate(…))`. No duplication. |
| WASM target | Valuable but requires `no_std` audit of `regex` crate. File as future beads. |
| Pattern streaming / incremental validation | UX feature, not library feature. |
| MCP tool surface | ADR-015 handles MCP. This ADR is the library under it. |
| `caro validate` as part of CaroML pipeline | CaroML already calls `SafetyValidator` directly. No change needed. |

---

### Build Spike Checklist (before feature PR)

New dependencies introduced:
- `cbindgen` (build-dep only)
- `sha2` (runtime dep in `caro-safety` only)

Per `.claude/rules/external-sdk-integration.md`:

| # | Check | `cbindgen` | `sha2` |
|---|---|---|---|
| 1 | License | MIT/Apache-2.0 ✅ | MIT/Apache-2.0 ✅ |
| 2 | MSRV ≤ 1.83 | needs verify | needs verify |
| 3 | Optional feature flag | build-dep; no feature flag needed | `caro-safety` crate dep only, not in main `caro` |
| 4 | Code reference forces compile | `build.rs` cbindgen invocation | `sha2::Sha256::digest(cmd.as_bytes())` in `SafetyAssessment::from_decision` |
| 5 | Two verification builds | `cargo check -p caro && cargo check -p caro-safety` | same |

`uuid` is already in tree (v1, v4+serde). No spike needed.

---

### Recommended Next Steps

1. **File ADR-022** — copy the ADR text from this document into
   `docs/adr/ADR-022-caro-safety-library-api.md`
2. **Run the build spike PR** — verify `cbindgen` + `sha2` MSRVs; create
   `caro-safety/` workspace crate skeleton
3. **Deasync `validate_command`** — remove `async` in `src/safety/mod.rs`
4. **Add `schema_version`, `decision_id`, `command_hash` to `SafetyDecision`**
5. **Implement `validate()` + C FFI** after spike merges
6. **Write 12 integration tests** before feature PR opens
7. **Open as `/caro.feature`** → spec-kitty plan for "caro-safety embeddable library"

---

*Signals consulted:*
- [arXiv 2603.20953: Before the Tool Call](https://arxiv.org/abs/2603.20953) — OAP paper (March 2026)
- [Hermes Market Scan 2026-06-11](/.hermes/digests/2026-06-11-agent-market-scan.md) — Opportunity D
- `src/safety/mod.rs` — current validator implementation (audited in full)
- `src/lib.rs` — current public API surface gap analysis
- `docs/adr/ADR-020-tiered-approval-protocol.md` — adjacent scope
- `docs/adr/ADR-021-execution-attribution.md` — adjacent scope
- `.claude/rules/external-sdk-integration.md` — build spike checklist
