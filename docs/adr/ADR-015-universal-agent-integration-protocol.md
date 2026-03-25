# ADR-015: Universal Agent Integration Protocol

**Status**: Proposed

**Date**: 2026-03-25

**Authors**: @wildcard

**Target**: Community + Enterprise

**Related**:
- [ADR-010](./ADR-010-bubblewrap-sandbox-execution.md) — Sandbox execution
- [ADR-003](./ADR-003-monitoring-audit-trail.md) — Monitoring and audit trail
- [ADR-016](./ADR-016-guardian-mode-llm-review.md) — Guardian Mode (LLM-assisted review)

## Context

### Inspiration: Codex Guardian

OpenAI's Codex CLI (open source, Rust) ships an experimental feature called "Guardian"
([PR #13860](https://github.com/openai/codex/pull/13860)) that automates approval decisions
for agent actions using a risk-based AI subagent reviewer. It cleanly separates:

- **approval_policy** — *when* a command requires review (untrusted, on-request, never)
- **approvals_reviewer** — *who* reviews it (user or `guardian_subagent`)

Guardian returns a structured **assessment payload**: `risk_score`, `rationale`, `action_summary`,
`decision`. This payload is what drives both automated decisions and audit trails.

Guardian is still **experimental** because:
1. Assessment payload generation can fail silently (Issue #15341)
2. The protocol is intentionally temporary while the design finalizes
3. Risk scoring is not yet well-tuned (over-protective in some cases, permissive in others)
4. It only works inside Codex CLI — not usable by other coder agents

### The Problem Caro Must Solve

Caro currently has no machine-readable output mode. Its safety validation returns human-readable
text to the terminal. This means:

- Coder agents (Claude Code, Cursor, GitHub Copilot, Aider) cannot call Caro as a safety
  middleware layer without parsing terminal output
- No structured exit-code contract for automation
- No drop-in alternative to Codex Guardian for agents not using Codex
- No path to integration with shell hooks, CI/CD pipelines, or approval workflows

Today's agents implement safety in isolation (or not at all). Caro should be the reusable
safety layer that any agent, script, or workflow can call.

### Stakeholders

- **Agent developers**: Need a simple, reliable subprocess API to invoke safety validation
- **End users**: Want coder agents to automatically invoke Caro without manual configuration
- **Enterprise users**: Need structured output for SIEM/audit log ingestion
- **Shell scripters**: Need exit codes and JSON for composable pipelines

## Decision

We will add a **machine-readable validation protocol** to Caro, making it a universal
safety middleware that any coder agent or script can call via subprocess.

### Core Addition: `caro validate` Subcommand

```bash
# New subcommand (non-interactive, no execution)
caro validate --json "rm -rf ./dist"
```

Exit codes (POSIX-compatible):
- `0` — Safe, allowed
- `1` — Blocked (dangerous pattern detected)
- `2` — Warning (moderate risk, human review recommended)
- `3` — Error (validator internal error)

JSON output on stdout:
```json
{
  "decision": "block",
  "risk_score": 95,
  "risk_level": "critical",
  "rationale": "Recursive deletion of a relative path — could wipe project files if CWD is wrong",
  "pattern_matched": "recursive_filesystem_deletion",
  "suggested_alternative": "rm -rf ./dist/* (keeps directory, slightly safer) or trash ./dist",
  "confidence_score": 1.0,
  "execution_time_ms": 8,
  "caro_version": "1.3.0"
}
```

For allowed commands:
```json
{
  "decision": "allow",
  "risk_score": 2,
  "risk_level": "safe",

  "rationale": "No dangerous patterns detected",
  "pattern_matched": null,
  "suggested_alternative": null,
  "confidence_score": 0.95,
  "execution_time_ms": 4,
  "caro_version": "1.3.0"
}
```

For warnings:
```json
{
  "decision": "warn",
  "risk_score": 45,
  "risk_level": "moderate",
  "rationale": "Command modifies file permissions — verify target path is intentional",
  "pattern_matched": "permission_modification",
  "suggested_alternative": null,
  "confidence_score": 0.9,
  "execution_time_ms": 5,
  "caro_version": "1.3.0"
}
```

### Pre-built Agent Integration Hooks

#### Claude Code (via hooks)

`~/.claude/hooks/pre-tool-use/caro-validate.sh`:
```bash
#!/bin/bash
# Caro safety validation hook for Claude Code
# Blocks dangerous Bash commands before Claude executes them

COMMAND="$CLAUDE_TOOL_INPUT_COMMAND"
if [ -z "$COMMAND" ]; then exit 0; fi

RESULT=$(caro validate --json "$COMMAND" 2>/dev/null)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 1 ]; then
  RATIONALE=$(echo "$RESULT" | jq -r '.rationale')
  echo "BLOCKED by Caro: $RATIONALE" >&2
  exit 1
fi

exit 0
```

#### Shell preexec Hook (bash/zsh)

```bash
# Add to ~/.bashrc or ~/.zshrc
caro_preexec() {
    local cmd="$1"
    local result exit_code
    result=$(caro validate --json "$cmd" 2>/dev/null)
    exit_code=$?
    if [ $exit_code -eq 1 ]; then
        echo "⚠️  Caro blocked: $(echo "$result" | jq -r '.rationale')" >&2
        return 1
    fi
}
[[ -n "$ZSH_VERSION" ]] && preexec_functions+=(caro_preexec)
[[ -n "$BASH_VERSION" ]] && trap 'caro_preexec "$BASH_COMMAND"' DEBUG
```

### Architecture

The `validate` subcommand maps directly onto the existing `SafetyValidator` but adds:

1. **`AssessmentPayload`** — Structured output type (see ADR implementation notes)
2. **JSON serialization** — `serde_json` output to stdout
3. **Exit code contract** — Deterministic process exit codes
4. **No-execution guarantee** — `caro validate` NEVER executes the command

```
caro validate --json "rm -rf /"
       │
       ▼
SafetyValidator::validate_command()
       │
       ▼
ValidationResult → AssessmentPayload::from(result)
       │
       ▼
serde_json::to_string_pretty(&payload) → stdout
       │
       ▼
process::exit(exit_code)
```

### Difference from Default `caro` Behavior

| Mode | Generates command | Validates | Executes | Output format |
|------|------------------|-----------|----------|---------------|
| `caro "query"` | Yes | Yes | Yes (with confirm) | Human-readable |
| `caro validate "cmd"` | No | Yes | **Never** | JSON (--json flag) or human |
| `caro validate --dry-run "query"` | Yes | Yes | **Never** | JSON |

## Rationale

### Why a Separate Subcommand?

The `validate` subcommand keeps the separation clean:
- No ambiguity about whether a command will execute
- Simple subprocess interface — no state, no sessions, pure function
- Works identically in interactive and CI/CD contexts
- Easy to test: known input → deterministic JSON output

### Why Match Codex Guardian's Assessment Payload Format?

Teams already building with Codex Guardian should be able to drop in Caro with minimal
changes to their tooling. Matching the conceptual payload fields (`risk_score`, `rationale`,
`decision`) creates a de-facto community standard for safety assessment output.

### Why Not a Daemon/Server?

Caro's philosophy is stateless, single-binary operation. A daemon would:
- Add installation complexity
- Introduce state management bugs
- Prevent use in sandboxed/containerized CI environments
- Conflict with the "batteries included" single-binary design

Subprocess calls are fast enough (<50ms) for agent use cases.

### Why These Three Exit Codes?

- `0/1` are universal (success/failure) — works in every shell and CI system
- `2` for warnings distinguishes "probably fine, human should check" from "definitely blocked"
- `3` for errors (not the command's fault) prevents silently allowing commands when the
  validator itself fails

## Consequences

### Benefits

1. **Universal adoption**: Any coder agent, script, or CI pipeline can call Caro
2. **Drop-in Guardian replacement**: Projects using Codex Guardian can adopt Caro
3. **Auditable output**: Structured JSON feeds directly into ADR-003 monitoring systems
4. **Composable**: Unix-philosophy tool — validate | log | decide
5. **Community standard**: Establishes assessment payload format others can build on

### Trade-offs

1. **Two surfaces to maintain**: Both the interactive CLI and the validate API
2. **Version coupling**: Callers depend on JSON field names → breaking changes are painful
3. **No streaming**: Batch commands must call the subprocess multiple times

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| JSON schema changes breaking callers | High | Semantic versioning + `caro_version` in payload |
| Silent exit code misinterpretation | Medium | Document exit codes clearly, add `--exit-code-info` flag |
| Performance impact on hot loops | Low | Benchmark; subprocess fork <20ms on modern hardware |

## Alternatives Considered

### Alternative 1: Library Crate

Expose `caro-safety` as a separate crate that agents link against.

- **Pros**: Zero subprocess overhead, native Rust integration
- **Cons**: Version coupling, Rust-only, binary size impact for callers, complex distribution
- **Why not chosen**: Subprocess is simpler, language-agnostic, and works for all agent types

### Alternative 2: MCP Tool

Expose Caro as an MCP server with a `validate_command` tool.

- **Pros**: Native Claude Code integration, structured I/O
- **Cons**: Requires running MCP server daemon, more complex than a simple CLI call, MCP-specific
- **Why not chosen**: Subprocess approach works universally; MCP can be a future addition layered on top

### Alternative 3: Extend Existing Flags

Add `--validate-only --json` to the main `caro` command.

- **Pros**: Fewer subcommands
- **Cons**: Ambiguous — user might think it could still execute, flag combinations are harder to document
- **Why not chosen**: Explicit `validate` subcommand is unambiguous and easier to hook safely

## Implementation Notes

### New Module: `src/cli/validate.rs`

```rust
pub struct ValidateArgs {
    pub command: String,
    pub json: bool,
    pub shell: Option<ShellType>,
}

pub async fn run(args: ValidateArgs, config: &Config) -> Result<()> {
    let validator = SafetyValidator::new(SafetyConfig::from_level(config.safety_level))?;
    let result = validator.validate_command(&args.command, shell).await?;
    let payload = AssessmentPayload::from(result);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        // Human-readable output
        println!("{}: {} (risk: {})", payload.decision, payload.rationale, payload.risk_level);
    }

    std::process::exit(payload.exit_code());
}
```

### New Type: `AssessmentPayload` in `src/safety/mod.rs`

See companion ADR implementation. The `AssessmentPayload` type wraps `ValidationResult`
with agent-friendly field names and exit code semantics.

### CLI Integration (`src/main.rs` / `src/cli/mod.rs`)

```rust
// Add to Args enum
#[derive(Subcommand)]
enum Commands {
    // ... existing commands
    Validate(ValidateArgs),
}
```

### Testing Strategy

1. **Unit**: `AssessmentPayload::from(ValidationResult)` field mapping
2. **Integration**: `caro validate --json "rm -rf /"` → assert JSON fields + exit code 1
3. **Contract**: JSON schema snapshot test (prevents silent breaking changes)
4. **Hook**: Shell script test calling `caro validate` as a preexec hook

## Success Metrics

| Metric | Target |
|--------|--------|
| JSON output latency | <50ms p99 |
| Exit code reliability | 100% correct for known patterns |
| Claude Code hook integration | Works out-of-box with documented hook |
| Zero breaking changes | Additive-only JSON fields in patch versions |

## References

- [Codex Guardian PR #13860](https://github.com/openai/codex/pull/13860) — Smart approvals implementation
- [Codex Guardian Issue #15341](https://github.com/openai/codex/issues/15341) — Assessment payload bug (what to avoid)
- [ADR-010](./ADR-010-bubblewrap-sandbox-execution.md) — Sandbox execution (complementary layer)
- [ADR-016](./ADR-016-guardian-mode-llm-review.md) — Guardian Mode (LLM-assisted review layer)
- [CLAUDE.md](../../CLAUDE.md) — Project architecture overview

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-03-25 | @wildcard | Initial draft, inspired by Codex Guardian research |
