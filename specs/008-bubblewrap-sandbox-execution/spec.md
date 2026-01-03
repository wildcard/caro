# Bubblewrap Sandbox Execution - Product Requirements Document

## Executive Summary

This specification defines the implementation of a sandbox execution layer for Caro using bubblewrap (bwrap) on Linux, with platform-specific alternatives for macOS. The sandbox provides defense-in-depth protection by isolating generated shell commands in a restricted environment, preventing dangerous commands from affecting the host system even if pattern-based validation fails.

## Problem Statement

### Current State

Caro generates shell commands from natural language using LLM inference. The safety pipeline currently consists of:

1. **Pattern-based validation** (52+ regex patterns)
2. **User confirmation** for high-risk commands
3. **Direct execution** in the user's shell

### Gap Analysis

| Threat | Current Mitigation | Gap |
|--------|-------------------|-----|
| Known dangerous patterns | Regex matching | Evasion via encoding/obfuscation |
| Novel attack patterns | None | Zero-day dangerous commands |
| LLM hallucination | User confirmation | Users may not recognize danger |
| Environment-specific attacks | None | BSD vs GNU, PATH manipulation |
| Supply chain (malicious models) | None | Model generates attack payload |

### Impact

- **Security**: Single point of failure (pattern matching)
- **Trust**: Users hesitant to run generated commands
- **Adoption**: Enterprise users require execution isolation

## Goals

### Primary Goals

1. **Isolation**: Commands execute in a sandbox with restricted filesystem, network, and process access
2. **Safety**: Even if a dangerous command bypasses validation, it cannot harm the host
3. **Transparency**: Users understand what restrictions are applied
4. **Performance**: Sandbox overhead <500ms per command execution
5. **Usability**: Legitimate commands work without user intervention

### Secondary Goals

1. **Auditability**: Log sandbox events for security review
2. **Configurability**: Power users can customize restrictions
3. **Cross-platform**: Consistent behavior across Linux and macOS
4. **Graceful degradation**: Clear behavior when sandbox unavailable

### Non-Goals

1. **Windows support in v1**: Defer to future release
2. **GUI configuration**: CLI and config file only
3. **Container image management**: Not a container runtime
4. **Network filtering rules**: Block/allow only, no fine-grained firewall

## User Stories

### US-1: Safe Command Execution

> As a Caro user, I want generated commands to run in a sandbox by default, so that even if the LLM generates something dangerous, my system is protected.

**Acceptance Criteria:**
- Commands execute in bubblewrap sandbox on Linux
- Sandbox prevents writes outside current directory
- Sandbox blocks network access by default
- Command output is captured and displayed normally

### US-2: Sandbox Visibility

> As a security-conscious user, I want to see when commands run in a sandbox and what restrictions apply, so I can trust the execution environment.

**Acceptance Criteria:**
- CLI shows sandbox status indicator
- Verbose mode displays active restrictions
- Sandbox failures produce clear error messages

### US-3: Profile Selection

> As a power user, I want to choose different sandbox profiles for different use cases, so I can balance security and functionality.

**Acceptance Criteria:**
- Three built-in profiles: strict, moderate, permissive
- Profile can be set via CLI flag or config file
- Custom profiles can be defined in config

### US-4: Sandbox Bypass

> As a user running trusted commands, I want to disable the sandbox when needed, so I can run commands that require full system access.

**Acceptance Criteria:**
- `--no-sandbox` flag disables sandbox for single command
- Config option to disable globally
- Warning shown when sandbox disabled for high-risk commands

### US-5: macOS Parity

> As a macOS user, I want sandbox protection equivalent to Linux users, so I have the same security guarantees.

**Acceptance Criteria:**
- macOS uses `sandbox-exec` with equivalent restrictions
- Same CLI flags and config options work
- Clear documentation of platform differences

### US-6: Unavailable Sandbox Handling

> As a user on a system without sandbox support, I want clear feedback about the limitation, so I can make informed decisions.

**Acceptance Criteria:**
- Startup check detects sandbox availability
- Warning shown if sandbox unavailable
- Configurable behavior: block, warn, or allow execution

## Functional Requirements

### FR-1: Sandbox Backend Abstraction

```rust
pub trait SandboxBackend: Send + Sync {
    /// Returns true if this backend is available on the current system
    fn is_available(&self) -> bool;

    /// Returns the backend name for logging
    fn name(&self) -> &'static str;

    /// Execute a command in the sandbox
    async fn execute(
        &self,
        command: &str,
        shell: ShellType,
        config: &SandboxConfig,
        cwd: &Path,
    ) -> Result<ExecutionResult, SandboxError>;
}
```

### FR-2: Sandbox Configuration

```rust
pub struct SandboxConfig {
    /// Base profile to use
    pub profile: SandboxProfile,

    /// Paths mounted read-only
    pub readonly_paths: Vec<PathBuf>,

    /// Paths mounted read-write
    pub writable_paths: Vec<PathBuf>,

    /// Paths completely hidden
    pub blocked_paths: Vec<PathBuf>,

    /// Allow network access
    pub allow_network: bool,

    /// Execution timeout
    pub timeout: Duration,

    /// Memory limit in MB (None = unlimited)
    pub memory_limit_mb: Option<u32>,

    /// Share PID namespace with host
    pub share_pid: bool,

    /// New session (detach from controlling terminal)
    pub new_session: bool,
}
```

### FR-3: Predefined Profiles

| Profile | `readonly_paths` | `writable_paths` | `network` | `blocked_paths` |
|---------|------------------|------------------|-----------|-----------------|
| Strict | `/usr`, `/lib*`, `/bin`, `/etc/alternatives` | None | No | `/home/*`, `/root`, `/var` |
| Moderate | `/usr`, `/lib*`, `/bin`, `/sbin`, `/etc` | `$CWD` | No | `/root`, sensitive dirs |
| Permissive | All | `$CWD`, `$HOME/.cache` | Yes | None |

### FR-4: CLI Interface

```bash
# Default: sandbox enabled with moderate profile
caro "find large files"

# Explicit sandbox control
caro "find files" --sandbox              # Enable (default)
caro "find files" --no-sandbox           # Disable
caro "find files" --sandbox-profile strict

# Custom restrictions
caro "build project" \
  --sandbox-allow-write ./build \
  --sandbox-allow-write ./dist \
  --sandbox-allow-network

# Dry-run sandbox (show what would happen)
caro "rm -rf /" --sandbox-dry-run
```

### FR-5: Configuration File

```toml
[execution.sandbox]
# Enable sandbox by default
enabled = true

# Default profile when not specified
default_profile = "moderate"

# Behavior when sandbox unavailable
# Options: "block" | "warn" | "allow"
fallback_on_unavailable = "warn"

# Per-profile configurations
[execution.sandbox.profiles.strict]
allow_network = false
readonly_paths = ["/usr", "/lib", "/lib64", "/bin", "/sbin"]
blocked_paths = ["/home", "/root", "/var/log", "/var/run"]
timeout_seconds = 30

[execution.sandbox.profiles.moderate]
allow_network = false
readonly_paths = ["/usr", "/lib", "/lib64", "/bin", "/sbin", "/etc"]
writable_paths = ["$CWD"]
timeout_seconds = 60

[execution.sandbox.profiles.permissive]
allow_network = true
readonly_paths = ["/"]
writable_paths = ["$CWD", "$HOME/.cache", "$HOME/.local"]
timeout_seconds = 120
```

### FR-6: Bubblewrap Integration (Linux)

Generate bwrap command from SandboxConfig:

```bash
bwrap \
  --ro-bind /usr /usr \
  --ro-bind /lib /lib \
  --ro-bind /lib64 /lib64 \
  --ro-bind /bin /bin \
  --ro-bind /sbin /sbin \
  --ro-bind /etc /etc \
  --bind $CWD $CWD \
  --proc /proc \
  --dev /dev \
  --tmpfs /tmp \
  --unshare-net \
  --unshare-pid \
  --unshare-ipc \
  --new-session \
  --die-with-parent \
  --chdir $CWD \
  -- bash -c "user_command_here"
```

### FR-7: macOS Sandbox Integration

Generate sandbox-exec profile from SandboxConfig:

```scheme
(version 1)
(deny default)
(allow process-fork)
(allow process-exec)
(allow file-read* (subpath "/usr"))
(allow file-read* (subpath "/bin"))
(allow file-read* (subpath "/Library"))
(allow file-read* (subpath "/System"))
(allow file-read-data (subpath "$CWD"))
(allow file-write-data (subpath "$CWD"))
(deny network*)
```

### FR-8: Error Handling

| Error Condition | Behavior | User Message |
|-----------------|----------|--------------|
| Sandbox backend unavailable | Respect `fallback_on_unavailable` config | "Sandbox unavailable: [reason]. Execution [blocked/proceeding without sandbox]." |
| Command fails in sandbox | Return non-zero exit, capture stderr | Normal error output |
| Command blocked by sandbox | Return specific error code | "Command blocked by sandbox: attempted [action] on [resource]" |
| Timeout exceeded | Kill process, return timeout error | "Command timed out after [N]s in sandbox" |
| Sandbox setup fails | Block execution | "Failed to initialize sandbox: [reason]" |

## Non-Functional Requirements

### NFR-1: Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Sandbox setup overhead | <100ms | Time from execute call to process start |
| Total command overhead | <500ms p99 | End-to-end with sandbox vs without |
| Memory overhead | <10MB | Additional memory for sandbox wrapper |

### NFR-2: Security

- Sandbox must prevent filesystem writes outside allowed paths
- Sandbox must prevent network access when disabled
- Sandbox must isolate PID namespace (processes can't see host processes)
- Sandbox must prevent privilege escalation (no setuid execution)
- Sandbox escape vulnerabilities are P0 security issues

### NFR-3: Reliability

- Sandbox failures must not crash caro
- Commands must not hang indefinitely (timeout enforced)
- Sandbox state must not persist between commands

### NFR-4: Compatibility

- Linux: Kernel 3.8+ with user namespaces enabled
- macOS: 10.15+ (Catalina) for sandbox-exec
- Shells: bash, zsh, fish, sh in sandbox

## Technical Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Caro CLI                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      CliApp                                  │   │
│  │  --sandbox, --no-sandbox, --sandbox-profile                 │   │
│  └──────────────────────────┬──────────────────────────────────┘   │
└─────────────────────────────┼───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Execution Layer                                 │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                  SandboxExecutor                             │   │
│  │  ┌─────────────────────────────────────────────────────┐    │   │
│  │  │              SandboxConfig                           │    │   │
│  │  │  • profile: SandboxProfile                          │    │   │
│  │  │  • readonly_paths, writable_paths, blocked_paths    │    │   │
│  │  │  • allow_network, timeout, memory_limit             │    │   │
│  │  └─────────────────────────────────────────────────────┘    │   │
│  │                          │                                   │   │
│  │         ┌────────────────┼────────────────┐                 │   │
│  │         ▼                ▼                ▼                 │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────────┐        │   │
│  │  │ Bubblewrap │  │ macOS      │  │ Passthrough    │        │   │
│  │  │ Backend    │  │ Sandbox    │  │ (no sandbox)   │        │   │
│  │  │ (Linux)    │  │ Backend    │  │                │        │   │
│  │  └────────────┘  └────────────┘  └────────────────┘        │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Execution Flow

```
1. User runs: caro "find large files"

2. CLI parses sandbox options:
   - Check --sandbox / --no-sandbox flags
   - Load default_profile from config
   - Merge CLI overrides

3. Command generation (existing pipeline):
   - LLM generates: find . -size +100M
   - Safety validation passes

4. Sandbox execution:
   a. Select backend (bubblewrap on Linux)
   b. Check backend availability
   c. Build SandboxConfig from profile + overrides
   d. Generate bwrap command line
   e. Spawn bwrap with command
   f. Capture stdout/stderr
   g. Wait for completion or timeout
   h. Return ExecutionResult

5. Display output to user
```

### File Structure

```
src/execution/
├── mod.rs                 # Module exports
├── executor.rs            # Existing CommandExecutor (unchanged)
├── shell.rs               # Shell detection (unchanged)
└── sandbox/
    ├── mod.rs             # SandboxExecutor, public interface
    ├── backend.rs         # SandboxBackend trait
    ├── config.rs          # SandboxConfig, SandboxProfile
    ├── bubblewrap.rs      # Linux bubblewrap implementation
    ├── macos.rs           # macOS sandbox-exec implementation
    ├── passthrough.rs     # No-op backend for --no-sandbox
    └── error.rs           # SandboxError types
```

## Testing Strategy

### Unit Tests

| Test Case | Description |
|-----------|-------------|
| `config_parsing` | Parse SandboxConfig from TOML |
| `profile_defaults` | Verify default values for each profile |
| `bwrap_command_generation` | Generate correct bwrap arguments |
| `macos_profile_generation` | Generate correct sandbox-exec profile |
| `config_merging` | CLI overrides merge with config file |

### Integration Tests

| Test Case | Description |
|-----------|-------------|
| `sandbox_blocks_write` | Write to /etc fails in sandbox |
| `sandbox_allows_cwd_write` | Write to CWD succeeds in moderate profile |
| `sandbox_blocks_network` | Network connection fails when disabled |
| `sandbox_captures_output` | stdout/stderr captured correctly |
| `sandbox_timeout` | Long-running command times out |
| `sandbox_exit_code` | Exit codes passed through correctly |

### Security Tests

| Test Case | Description |
|-----------|-------------|
| `escape_attempt_symlink` | Symlink to blocked path fails |
| `escape_attempt_proc` | Reading /proc/1 fails |
| `escape_attempt_device` | Writing to /dev/sda fails |
| `privilege_escalation` | setuid execution blocked |
| `namespace_isolation` | Can't see host processes |

### Performance Tests

| Test Case | Target |
|-----------|--------|
| `sandbox_overhead_simple` | <200ms for `echo hello` |
| `sandbox_overhead_complex` | <500ms for `find . -name "*.rs"` |
| `sandbox_memory` | <10MB additional memory |

## Rollout Plan

### Phase 1: Linux MVP (Target: 2 weeks)

- [ ] Implement SandboxBackend trait
- [ ] Implement BubblewrapBackend
- [ ] Implement SandboxConfig and profiles
- [ ] Add CLI flags: --sandbox, --no-sandbox, --sandbox-profile
- [ ] Add config file support
- [ ] Unit and integration tests
- [ ] Documentation

### Phase 2: macOS Support (Target: 1 week)

- [ ] Implement MacOSSandboxBackend
- [ ] Cross-platform testing
- [ ] Platform-specific documentation

### Phase 3: Production Hardening (Target: 1 week)

- [ ] Performance optimization
- [ ] Security audit
- [ ] Error message polish
- [ ] Enable by default with "moderate" profile

### Phase 4: Future (Backlog)

- [ ] Windows investigation (Job Objects, WSL2)
- [ ] Custom profile wizard
- [ ] Sandbox event logging/audit trail
- [ ] Resource limits (CPU, memory, I/O)

## Success Metrics

| Metric | Target | Tracking |
|--------|--------|----------|
| Adoption rate | >60% of commands run in sandbox | Opt-in telemetry |
| Overhead | <500ms p99 | CI benchmarks |
| Security | 0 sandbox escapes | Bug bounty, security reviews |
| False positives | <5% legitimate commands fail | User feedback |
| Platform coverage | Linux + macOS GA | Release checklist |

## Open Questions

1. **Q: Should we vendor bubblewrap or require system installation?**
   - Leaning: Require system install, provide installation instructions
   - Rationale: Avoid binary bloat, easier updates, trust system package

2. **Q: How to handle commands that legitimately need full access?**
   - Leaning: --no-sandbox with warning for high-risk commands
   - Alternative: Prompt user to confirm disabling sandbox

3. **Q: Should sandbox be enabled by default?**
   - Leaning: Yes, with "moderate" profile
   - Risk: Some commands may unexpectedly fail

4. **Q: How to handle ~/.bashrc, ~/.zshrc sourcing in sandbox?**
   - Leaning: Mount home read-only, don't source rc files
   - Alternative: Source with restrictions

## Appendix

### A: Bubblewrap Installation

```bash
# Debian/Ubuntu
sudo apt install bubblewrap

# Fedora
sudo dnf install bubblewrap

# Arch
sudo pacman -S bubblewrap

# macOS (not needed, uses built-in sandbox-exec)
```

### B: Checking User Namespace Support

```bash
# Check if user namespaces are enabled
cat /proc/sys/kernel/unprivileged_userns_clone
# Should return 1

# If not, enable (requires root):
echo 1 | sudo tee /proc/sys/kernel/unprivileged_userns_clone
```

### C: Example bwrap Invocations

```bash
# Minimal sandbox (read-only root)
bwrap --ro-bind / / --dev /dev --proc /proc -- ls

# Network isolated
bwrap --ro-bind / / --unshare-net -- curl https://example.com
# (fails: network unreachable)

# Write-protected /etc
bwrap --ro-bind / / --ro-bind /etc /etc -- touch /etc/test
# (fails: read-only filesystem)
```

### D: Related Documents

- [ADR-004: Bubblewrap Sandbox Execution](../../docs/adr/ADR-004-bubblewrap-sandbox-execution.md)
- [SECURITY.md](../../SECURITY.md)
- [Spec 003: Core Infrastructure](../003-implement-core-infrastructure/spec.md)
