# ADR-004: Bubblewrap Sandbox for Command Execution

**Status**: Proposed

**Date**: 2026-01-03

**Authors**: @wildcard

**Target**: Community

## Context

Caro generates shell commands from natural language using LLM inference. While we have a pattern-based safety validation system (52+ regex patterns in `src/safety/`), this approach has fundamental limitations:

- **Pattern evasion**: Attackers can encode dangerous commands using hex escapes, variable expansion, or obfuscation
- **Novel attacks**: New dangerous patterns not in our database bypass validation
- **LLM hallucination**: The model may generate syntactically valid but semantically dangerous commands
- **Environment differences**: Commands behave differently across systems (BSD vs GNU, different PATH)

Our security policy (`SECURITY.md`) explicitly calls for "Defense in Depth" with execution sandboxing as a recommended layer. Currently, validated commands run directly in the user's shell with no isolation—if pattern matching fails, there's no safety net.

### Problem Statement

How do we provide a robust execution isolation layer that:
1. Prevents dangerous commands from affecting the host system
2. Allows legitimate commands to execute normally
3. Adds minimal latency (<500ms overhead)
4. Works without root privileges
5. Integrates cleanly with our existing execution pipeline

### Stakeholders

- **End users**: Need protection from dangerous generated commands
- **Security researchers**: Require auditable, defense-in-depth architecture
- **Developers**: Need maintainable, testable sandbox implementation
- **Enterprise users**: Require compliance with security policies

## Decision

We will integrate **bubblewrap (bwrap)** as the primary sandbox backend for Linux, with platform-specific alternatives for macOS and Windows.

The sandbox layer will be:
1. **Optional but recommended**: Enabled by default on supported platforms
2. **Configurable**: Users can tune restrictions per safety level
3. **Transparent**: Commands see a restricted but functional environment
4. **Fail-safe**: If sandbox fails to initialize, block execution (don't fall through)

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Existing Caro Pipeline                       │
│  Input → Agent Loop → Safety Validation → User Confirmation    │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    NEW: Sandbox Layer                           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  SandboxExecutor                         │   │
│  │  ┌─────────────┬─────────────┬─────────────────────┐    │   │
│  │  │ Bubblewrap  │ macOS       │ Windows             │    │   │
│  │  │ (Linux)     │ sandbox-exec│ (Job Objects/WSL)   │    │   │
│  │  └─────────────┴─────────────┴─────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────┘   │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Command Execution                            │
│              Isolated environment with restricted access        │
└─────────────────────────────────────────────────────────────────┘
```

### Sandbox Profiles

Three pre-defined profiles mapping to existing `SafetyLevel`:

| Profile | Network | Filesystem | Devices | Use Case |
|---------|---------|------------|---------|----------|
| `Strict` | Blocked | Read-only (whitelist) | None | High-risk commands |
| `Moderate` | Blocked | Read-only + CWD write | Limited | Default for generated commands |
| `Permissive` | Allowed | Full read, CWD write | Standard | User-trusted commands |

## Rationale

### Why Bubblewrap?

1. **Unprivileged operation**: Uses user namespaces—no root/sudo required
2. **Lightweight**: ~500KB binary, minimal overhead (<100ms typical)
3. **Battle-tested**: Powers Flatpak, used by millions of Linux users
4. **Fine-grained control**: Precise filesystem, network, PID isolation
5. **No daemon**: Stateless, per-command invocation
6. **Active maintenance**: Well-maintained by containers project

### Why Not Alternatives?

- **Docker/Podman**: Heavyweight, requires daemon, images, more complexity
- **Firejail**: More features but larger attack surface, more complex
- **systemd-nspawn**: Requires root privileges
- **chroot**: Insufficient isolation, easily escapable
- **SELinux/AppArmor**: Kernel-level, complex policy management

### Alignment with Project Goals

- **Defense in Depth**: Adds isolation layer after pattern matching
- **User Safety**: Protects against LLM hallucination and evasion attacks
- **Performance**: Bubblewrap overhead fits within <500ms budget
- **Binary Size**: No significant impact (external dependency)
- **Cross-platform**: Architecture supports platform-specific implementations

## Consequences

### Benefits

1. **True isolation**: Commands cannot escape sandbox even if validation fails
2. **Auditability**: Sandbox configuration is explicit and auditable
3. **Confidence**: Users can run generated commands with higher trust
4. **Compliance**: Meets enterprise security requirements for execution isolation
5. **Graceful degradation**: Clear error when sandbox unavailable vs silent bypass

### Trade-offs

1. **Linux-first**: Bubblewrap only on Linux; need alternatives for macOS/Windows
2. **Kernel requirements**: Needs user namespaces (kernel 3.8+, enabled on most distros)
3. **Learning curve**: Users need to understand sandbox restrictions
4. **Some commands fail**: Commands requiring full system access won't work in sandbox

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| User namespaces disabled on some systems | Sandbox unavailable | Detect at startup, provide clear guidance |
| Commands fail unexpectedly in sandbox | User frustration | Clear error messages, easy disable option |
| Sandbox escape vulnerabilities | Security compromise | Pin bubblewrap version, monitor CVEs |
| Performance regression | Poor UX | Benchmark in CI, cache sandbox setup |
| macOS/Windows parity | Inconsistent experience | Document platform differences clearly |

## Alternatives Considered

### Alternative 1: Docker-based Sandbox

- **Description**: Run commands in ephemeral Docker containers
- **Pros**: Full isolation, familiar to developers, extensive tooling
- **Cons**: Requires Docker daemon, significant overhead (seconds), complex setup, heavyweight for single commands

### Alternative 2: Firejail

- **Description**: Use Firejail for sandboxing
- **Pros**: Feature-rich, good network filtering, profile system
- **Cons**: Larger attack surface, setuid binary (security concern), more complex than needed

### Alternative 3: eBPF-based Monitoring

- **Description**: Use eBPF to monitor and block dangerous syscalls
- **Pros**: No container overhead, kernel-level visibility
- **Cons**: Requires root/CAP_BPF, complex development, less isolation than namespaces

### Alternative 4: Pure Pattern Matching (Status Quo)

- **Description**: Continue with regex-based validation only
- **Pros**: No additional dependencies, works everywhere
- **Cons**: Fundamentally bypassable, no defense in depth, doesn't meet security goals

## Implementation Notes

### Module Structure

```
src/execution/
├── mod.rs              # Re-exports, ExecutionResult
├── executor.rs         # Existing CommandExecutor
├── shell.rs            # Shell detection
└── sandbox/            # NEW
    ├── mod.rs          # SandboxExecutor, SandboxConfig
    ├── backend.rs      # SandboxBackend trait
    ├── bubblewrap.rs   # Linux implementation
    ├── macos.rs        # macOS sandbox-exec implementation
    ├── windows.rs      # Windows stub/future implementation
    └── profiles.rs     # Predefined sandbox profiles
```

### Key Interfaces

```rust
#[async_trait]
pub trait SandboxBackend: Send + Sync {
    /// Check if this backend is available on the current system
    fn is_available(&self) -> bool;

    /// Execute command in sandbox with given restrictions
    async fn execute(
        &self,
        command: &str,
        shell: ShellType,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult>;
}

pub struct SandboxConfig {
    pub profile: SandboxProfile,
    pub readonly_paths: Vec<PathBuf>,
    pub writable_paths: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
    pub allow_network: bool,
    pub timeout: Duration,
    pub memory_limit_mb: Option<u32>,
}

pub enum SandboxProfile {
    Strict,
    Moderate,
    Permissive,
    Custom(SandboxConfig),
}
```

### CLI Integration

```bash
# Enable sandbox (default on Linux)
caro "find large files" --sandbox

# Disable sandbox
caro "list files" --no-sandbox

# Specify profile
caro "delete temp files" --sandbox-profile strict

# Custom restrictions
caro "build project" --sandbox-allow-write ./build --sandbox-allow-network
```

### Configuration (caro.toml)

```toml
[execution.sandbox]
enabled = true                    # Enable by default
default_profile = "moderate"      # Default restriction level
fallback_on_unavailable = "block" # "block" | "warn" | "allow"

[execution.sandbox.profiles.strict]
allow_network = false
readonly_paths = ["/usr", "/lib", "/bin", "/etc"]
blocked_paths = ["/home", "/root", "/var"]

[execution.sandbox.profiles.moderate]
allow_network = false
readonly_paths = ["/usr", "/lib", "/bin"]
writable_paths = ["$CWD"]
```

### Testing Strategy

1. **Unit tests**: Mock sandbox backend, test configuration parsing
2. **Integration tests**: Real bubblewrap execution on Linux CI
3. **Security tests**: Verify blocked paths are actually blocked
4. **Performance tests**: Measure sandbox overhead, ensure <500ms
5. **Cross-platform tests**: Verify graceful handling on unsupported platforms

### Rollout Strategy

1. **Phase 1**: Linux bubblewrap implementation, opt-in via flag
2. **Phase 2**: macOS sandbox-exec implementation
3. **Phase 3**: Enable by default on supported platforms
4. **Phase 4**: Windows investigation (Job Objects or WSL2)

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Sandbox overhead | <500ms p99 | CI benchmark suite |
| Security coverage | Block 100% of known evasion patterns | Security test suite |
| User adoption | >50% of Linux users enable sandbox | Telemetry (opt-in) |
| False positive rate | <5% commands fail due to sandbox | User feedback, logs |
| Platform coverage | Linux + macOS GA, Windows preview | Release tracking |

## References

- [Bubblewrap GitHub](https://github.com/containers/bubblewrap)
- [Linux user namespaces](https://man7.org/linux/man-pages/man7/user_namespaces.7.html)
- [Flatpak sandbox architecture](https://docs.flatpak.org/en/latest/sandbox-permissions.html)
- [macOS sandbox-exec](https://reverse.put.as/wp-content/uploads/2011/09/Apple-Sandbox-Guide-v1.0.pdf)
- [SECURITY.md](../SECURITY.md) - Defense in depth requirements
- [ADR-001](./ADR-001-enterprise-community-architecture.md) - Architecture principles

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-03 | @wildcard | Initial draft |
