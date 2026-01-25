# Implementation Plan: Bubblewrap Sandbox Execution

## Phase 1: Core Implementation (Linux)

### Task 1.1: Sandbox Module Structure

Create the module structure in `src/execution/sandbox/`:

```
src/execution/sandbox/
├── mod.rs           # Public exports, SandboxExecutor
├── backend.rs       # SandboxBackend trait definition
├── config.rs        # SandboxConfig, SandboxProfile enums
├── error.rs         # SandboxError types
├── bubblewrap.rs    # Linux bubblewrap implementation
├── passthrough.rs   # No-op backend for --no-sandbox
└── macos.rs         # Placeholder for Phase 2
```

**Deliverables:**
- [ ] Module structure with empty stubs
- [ ] Re-exports in `src/execution/mod.rs`
- [ ] Compiles with no warnings

### Task 1.2: Configuration Types

Define core configuration types:

```rust
// config.rs
pub enum SandboxProfile {
    Strict,
    Moderate,
    Permissive,
    Custom,
}

pub struct SandboxConfig {
    pub profile: SandboxProfile,
    pub readonly_paths: Vec<PathBuf>,
    pub writable_paths: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
    pub allow_network: bool,
    pub timeout: Duration,
    pub memory_limit_mb: Option<u32>,
    pub share_pid: bool,
    pub new_session: bool,
}

impl SandboxConfig {
    pub fn strict() -> Self { ... }
    pub fn moderate() -> Self { ... }
    pub fn permissive() -> Self { ... }
}
```

**Deliverables:**
- [ ] SandboxProfile enum
- [ ] SandboxConfig struct with builder pattern
- [ ] Default implementations for each profile
- [ ] Unit tests for config creation

### Task 1.3: Backend Trait

Define the sandbox backend abstraction:

```rust
// backend.rs
#[async_trait]
pub trait SandboxBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;

    async fn execute(
        &self,
        command: &str,
        shell: ShellType,
        config: &SandboxConfig,
        cwd: &Path,
    ) -> Result<ExecutionResult, SandboxError>;
}
```

**Deliverables:**
- [ ] SandboxBackend trait definition
- [ ] ExecutionResult reuse or extension
- [ ] SandboxError enum with variants

### Task 1.4: Bubblewrap Backend

Implement Linux bubblewrap integration:

```rust
// bubblewrap.rs
pub struct BubblewrapBackend {
    bwrap_path: PathBuf,
}

impl BubblewrapBackend {
    pub fn new() -> Result<Self, SandboxError> {
        // Find bwrap in PATH
        // Verify user namespace support
    }

    fn build_command(&self, config: &SandboxConfig) -> Command {
        // Generate bwrap arguments from config
    }
}
```

**Key bwrap arguments:**
- `--ro-bind <src> <dst>` - Read-only bind mount
- `--bind <src> <dst>` - Read-write bind mount
- `--dev /dev` - Mount minimal /dev
- `--proc /proc` - Mount /proc
- `--unshare-net` - Network namespace isolation
- `--unshare-pid` - PID namespace isolation
- `--die-with-parent` - Kill sandbox if parent dies
- `--new-session` - New session ID

**Deliverables:**
- [ ] BubblewrapBackend struct
- [ ] bwrap path detection
- [ ] User namespace availability check
- [ ] Command builder from SandboxConfig
- [ ] Process execution with output capture
- [ ] Timeout handling

### Task 1.5: Passthrough Backend

No-op backend for when sandbox is disabled:

```rust
// passthrough.rs
pub struct PassthroughBackend;

impl SandboxBackend for PassthroughBackend {
    fn name(&self) -> &'static str { "passthrough" }
    fn is_available(&self) -> bool { true }

    async fn execute(...) -> Result<ExecutionResult, SandboxError> {
        // Direct execution without sandbox
    }
}
```

**Deliverables:**
- [ ] PassthroughBackend implementation
- [ ] Delegates to existing CommandExecutor

### Task 1.6: SandboxExecutor

High-level executor that selects and uses backends:

```rust
// mod.rs
pub struct SandboxExecutor {
    backend: Box<dyn SandboxBackend>,
    default_config: SandboxConfig,
}

impl SandboxExecutor {
    pub fn new(enabled: bool) -> Result<Self, SandboxError> {
        // Select appropriate backend for platform
    }

    pub async fn execute(
        &self,
        command: &str,
        shell: ShellType,
        config: Option<SandboxConfig>,
    ) -> Result<ExecutionResult, SandboxError> {
        // Use provided config or default
        // Delegate to backend
    }
}
```

**Deliverables:**
- [ ] SandboxExecutor struct
- [ ] Backend selection logic
- [ ] Config merging (CLI overrides)
- [ ] Integration with existing execution pipeline

### Task 1.7: Error Types

Define comprehensive error handling:

```rust
// error.rs
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Sandbox backend not available: {reason}")]
    Unavailable { reason: String },

    #[error("Failed to initialize sandbox: {0}")]
    InitializationFailed(String),

    #[error("Command blocked by sandbox: {action} on {resource}")]
    Blocked { action: String, resource: String },

    #[error("Command timed out after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Sandbox execution failed: {0}")]
    ExecutionFailed(#[from] std::io::Error),
}
```

**Deliverables:**
- [ ] SandboxError enum
- [ ] Error to exit code mapping
- [ ] User-friendly error messages

## Phase 2: CLI Integration

### Task 2.1: CLI Flags

Add sandbox-related flags to CLI:

```rust
// cli/mod.rs additions
#[derive(Parser)]
struct Cli {
    /// Enable sandbox execution (default: true on Linux)
    #[arg(long)]
    sandbox: bool,

    /// Disable sandbox execution
    #[arg(long)]
    no_sandbox: bool,

    /// Sandbox profile: strict, moderate, permissive
    #[arg(long, default_value = "moderate")]
    sandbox_profile: String,

    /// Allow network access in sandbox
    #[arg(long)]
    sandbox_allow_network: bool,

    /// Allow write access to path
    #[arg(long, action = ArgAction::Append)]
    sandbox_allow_write: Vec<PathBuf>,
}
```

**Deliverables:**
- [ ] CLI argument definitions
- [ ] Argument parsing and validation
- [ ] Help text for sandbox options

### Task 2.2: Configuration File

Add sandbox config to TOML schema:

```rust
// config/mod.rs additions
#[derive(Deserialize)]
pub struct ExecutionConfig {
    pub sandbox: SandboxTomlConfig,
}

#[derive(Deserialize)]
pub struct SandboxTomlConfig {
    pub enabled: bool,
    pub default_profile: String,
    pub fallback_on_unavailable: String,
    pub profiles: HashMap<String, ProfileConfig>,
}
```

**Deliverables:**
- [ ] TOML schema for sandbox config
- [ ] Config loading and validation
- [ ] Profile definition support
- [ ] Default config values

### Task 2.3: Integration with Execution Flow

Modify command execution to use sandbox:

```rust
// Modify CliApp::execute_command()
async fn execute_command(&self, command: &str) -> Result<()> {
    let sandbox_config = self.build_sandbox_config()?;

    let result = if self.args.no_sandbox {
        self.executor.execute(command).await?
    } else {
        self.sandbox_executor.execute(command, sandbox_config).await?
    };

    // ... handle result
}
```

**Deliverables:**
- [ ] SandboxExecutor instantiation in CliApp
- [ ] Config merging from file + CLI
- [ ] Conditional sandbox/direct execution
- [ ] Error handling for sandbox failures

## Phase 3: Testing

### Task 3.1: Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_strict_profile_defaults() { ... }

    #[test]
    fn test_bwrap_command_generation() { ... }

    #[test]
    fn test_config_merging() { ... }
}
```

**Test coverage:**
- [ ] SandboxConfig creation and defaults
- [ ] Profile inheritance
- [ ] CLI argument parsing
- [ ] TOML config parsing
- [ ] bwrap command generation
- [ ] Error handling paths

### Task 3.2: Integration Tests

```rust
#[tokio::test]
async fn test_sandbox_blocks_etc_write() {
    let executor = SandboxExecutor::new(true).unwrap();
    let result = executor.execute(
        "touch /etc/test",
        ShellType::Bash,
        Some(SandboxConfig::strict()),
    ).await;

    assert!(result.is_err());
}
```

**Test scenarios:**
- [ ] Write to read-only path blocked
- [ ] Write to writable path succeeds
- [ ] Network blocked when disabled
- [ ] Network succeeds when allowed
- [ ] Timeout enforcement
- [ ] Exit code preservation
- [ ] stdout/stderr capture

### Task 3.3: Security Tests

```rust
#[tokio::test]
async fn test_symlink_escape_blocked() {
    // Create symlink pointing outside sandbox
    // Attempt to read through symlink
    // Verify access denied
}
```

**Security scenarios:**
- [ ] Symlink escape attempts
- [ ] /proc access restrictions
- [ ] Device access blocked
- [ ] Setuid execution prevented
- [ ] Process visibility isolation

### Task 3.4: Performance Benchmarks

```rust
#[bench]
fn bench_sandbox_overhead(b: &mut Bencher) {
    b.iter(|| {
        // Measure sandbox vs direct execution
    });
}
```

**Metrics:**
- [ ] Sandbox setup time
- [ ] Simple command overhead
- [ ] Complex command overhead
- [ ] Memory usage

## Phase 4: macOS Support

### Task 4.1: macOS Sandbox Backend

```rust
// macos.rs
pub struct MacOSSandboxBackend;

impl SandboxBackend for MacOSSandboxBackend {
    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }

    async fn execute(...) -> Result<ExecutionResult, SandboxError> {
        // Generate sandbox-exec profile
        // Execute with sandbox-exec
    }
}
```

**Deliverables:**
- [ ] sandbox-exec profile generation
- [ ] Profile to SandboxConfig mapping
- [ ] macOS-specific path handling
- [ ] Integration tests on macOS CI

### Task 4.2: Cross-Platform Testing

- [ ] Linux CI with bubblewrap
- [ ] macOS CI with sandbox-exec
- [ ] Platform detection tests
- [ ] Graceful fallback tests

## Phase 5: Documentation

### Task 5.1: User Documentation

- [ ] README section on sandbox feature
- [ ] CLI help text
- [ ] Configuration examples
- [ ] Troubleshooting guide

### Task 5.2: Developer Documentation

- [ ] Module-level rustdoc
- [ ] Architecture overview
- [ ] Backend implementation guide
- [ ] Security considerations

## Acceptance Criteria

### Functional

- [ ] Commands execute in bubblewrap sandbox on Linux
- [ ] Commands execute in sandbox-exec on macOS
- [ ] Three profiles work as documented
- [ ] CLI flags override config file
- [ ] Sandbox failures produce clear errors

### Performance

- [ ] Overhead <500ms p99
- [ ] Memory <10MB additional

### Security

- [ ] Blocks write to read-only paths
- [ ] Blocks network when disabled
- [ ] No known sandbox escape vectors

## Dependencies

### External

- `bubblewrap` (Linux, system package)
- `sandbox-exec` (macOS, built-in)

### Internal

- `src/execution/executor.rs` - Existing executor
- `src/execution/shell.rs` - Shell detection
- `src/config/` - Configuration loading
- `src/cli/` - CLI argument parsing

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| User namespaces disabled | Sandbox unavailable | Detect and warn, configurable fallback |
| Command failures in sandbox | User frustration | Clear errors, easy disable |
| Performance regression | Poor UX | Benchmark in CI, optimize |
| macOS sandbox-exec limitations | Reduced functionality | Document differences |
