# Execution Module API Contract

**Module**: `caro::execution`
**Purpose**: Execution context with environment capture and shell detection

## Public API

### ExecutionContext

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    // Private fields exposed via getters
}

impl ExecutionContext {
    /// Captures current execution environment
    /// Returns error if critical info unavailable (current_dir, platform)
    pub fn capture() -> Result<Self, ExecutionError>;

    /// Creates context with custom values (for testing)
    pub fn new(
        current_dir: PathBuf,
        shell_type: ShellType,
        platform: Platform,
    ) -> Result<Self, ExecutionError>;

    /// Gets current working directory
    pub fn current_dir(&self) -> &Path;

    /// Gets detected or configured shell type
    pub fn shell_type(&self) -> ShellType;

    /// Gets platform (Linux, MacOS, Windows)
    pub fn platform(&self) -> Platform;

    /// Gets filtered environment variables
    pub fn environment_vars(&self) -> &HashMap<String, String>;

    /// Gets username
    pub fn username(&self) -> &str;

    /// Gets hostname
    pub fn hostname(&self) -> &str;

    /// Gets timestamp when context was captured
    pub fn captured_at(&self) -> DateTime<Utc>;

    /// Serializes context for LLM prompt inclusion
    pub fn to_prompt_context(&self) -> String;

    /// Checks if a specific environment variable exists
    pub fn has_env_var(&self, key: &str) -> bool;

    /// Gets a specific environment variable value
    pub fn get_env_var(&self, key: &str) -> Option<&str>;
}
```

### ShellDetector

```rust
pub struct ShellDetector {
    // Private fields
}

impl ShellDetector {
    /// Creates new shell detector
    pub fn new() -> Self;

    /// Detects shell type using multiple strategies
    /// Falls back to ShellType::Sh if detection fails
    pub fn detect(&self) -> ShellType;

    /// Detects shell from SHELL environment variable
    pub fn detect_from_env(&self) -> Option<ShellType>;

    /// Detects shell from parent process (platform-specific)
    pub fn detect_from_process(&self) -> Option<ShellType>;

    /// Applies user configuration override
    pub fn with_override(&self, override_shell: Option<ShellType>) -> ShellType;
}
```

### PlatformDetector

```rust
pub struct PlatformDetector;

impl PlatformDetector {
    /// Detects current platform (Linux, MacOS, Windows)
    pub fn detect() -> Platform;

    /// Checks if running on POSIX-compliant system
    pub fn is_posix() -> bool;
}
```

### ExecutionError

```rust
#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Current directory not accessible: {0}")]
    CurrentDirNotAccessible(String),

    #[error("Unable to determine platform")]
    PlatformDetectionFailed,

    #[error("Username not available")]
    UsernameNotAvailable,

    #[error("Hostname not available")]
    HostnameNotAvailable,

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Behavioral Contracts

### Contract: capture Returns Valid Context

**Given**: Standard user environment with accessible current directory
**When**: `ExecutionContext::capture()` is called
**Then**:
- Returns `Ok(ExecutionContext)`
- `current_dir()` returns absolute path to pwd
- `shell_type()` returns detected shell (or Sh fallback)
- `platform()` returns correct OS platform
- `username()` and `hostname()` are non-empty
- `captured_at()` is within 1 second of current time

### Contract: capture Filters Sensitive Environment Variables

**Given**: Environment has `API_KEY=secret123` and `HOME=/home/user`
**When**: `ExecutionContext::capture()` is called
**Then**:
- `environment_vars()` includes `HOME=/home/user`
- `environment_vars()` does NOT include `API_KEY`
- `has_env_var("HOME")` returns `true`
- `has_env_var("API_KEY")` returns `false`

### Contract: capture Includes Essential Environment Variables

**Given**: Standard POSIX environment
**When**: `ExecutionContext::capture()` is called
**Then**:
- `environment_vars()` includes: PATH, HOME, USER, SHELL, LANG, PWD
- All values are non-empty strings
- Keys are case-sensitive

### Contract: ShellDetector Uses SHELL Environment Variable

**Given**: `SHELL=/bin/bash` environment variable is set
**When**: `ShellDetector::new().detect()` is called
**Then**:
- Returns `ShellType::Bash`
- Detection completes in <10ms

### Contract: ShellDetector Handles Multiple Shell Variants

**Given**: Various SHELL values
**When**: Detection is performed for each
**Then**:
- `/bin/bash`, `/usr/bin/bash`, `bash` → `ShellType::Bash`
- `/bin/zsh`, `/usr/bin/zsh`, `zsh` → `ShellType::Zsh`
- `/usr/bin/fish`, `fish` → `ShellType::Fish`
- `powershell`, `pwsh` → `ShellType::PowerShell`
- `cmd`, `cmd.exe` → `ShellType::Cmd`

### Contract: ShellDetector Falls Back to Sh

**Given**: SHELL environment variable is missing or unrecognized value
**When**: `ShellDetector::new().detect()` is called
**Then**:
- Returns `ShellType::Sh` (POSIX baseline)
- No error is raised

### Contract: ShellDetector Applies User Override

**Given**: Auto-detection would return `Bash` but user config specifies `Zsh`
**When**: `detector.with_override(Some(ShellType::Zsh))` is called
**Then**:
- Returns `ShellType::Zsh`
- User preference takes precedence over detection

### Contract: PlatformDetector Returns Correct Platform

**Given**: Running on Linux
**When**: `PlatformDetector::detect()` is called
**Then**:
- Returns `Platform::Linux`

**Given**: Running on macOS
**When**: `PlatformDetector::detect()` is called
**Then**:
- Returns `Platform::MacOS`

**Given**: Running on Windows
**When**: `PlatformDetector::detect()` is called
**Then**:
- Returns `Platform::Windows`

### Contract: is_posix Returns Correct Boolean

**Given**: Platform is Linux or MacOS
**When**: `PlatformDetector::is_posix()` is called
**Then**: Returns `true`

**Given**: Platform is Windows
**When**: `PlatformDetector::is_posix()` is called
**Then**: Returns `false`

### Contract: to_prompt_context Serializes for LLM

**Given**: ExecutionContext with various fields populated
**When**: `context.to_prompt_context()` is called
**Then**:
- Returns formatted string with context information
- Includes: current_dir, shell_type, platform, username
- Format suitable for LLM prompt injection
- Example:
```
Current directory: /home/user/projects/myapp
Shell: bash
Platform: Linux
User: user@hostname
```

### Contract: Context Capture Performance

**Given**: Standard environment
**When**: `ExecutionContext::capture()` is called
**Then**:
- Operation completes in <50ms (NFR-003)
- No network requests made
- All data captured from local system only

### Contract: Context Is Immutable After Capture

**Given**: ExecutionContext created
**When**: Attempting to access mutable methods
**Then**:
- All public methods return references or copies
- No `&mut self` methods exposed
- Ensures consistent context throughout request

### Contract: Sensitive Data Filtering Patterns

**Given**: Environment variables with various names
**When**: ExecutionContext filters variables
**Then**:
- Excluded patterns (case-insensitive):
  - `*_KEY`, `*_TOKEN`, `*_PASSWORD`, `*_SECRET`, `*_CREDENTIAL`
  - `API_KEY`, `AUTH_TOKEN`, `AWS_SECRET_ACCESS_KEY`
  - `GITHUB_TOKEN`, `GITLAB_TOKEN`
- Included patterns:
  - `PATH`, `HOME`, `USER`, `SHELL`, `LANG`, `PWD`, `TERM`
  - `LC_*` (locale variables)

### Contract: Invalid Current Directory Handling

**Given**: Process current directory has been deleted
**When**: `ExecutionContext::capture()` is called
**Then**:
- Returns `Err(ExecutionError::CurrentDirNotAccessible(...))`
- Error message includes recovery suggestion
- No partial context returned

## Test Coverage Requirements

1. **Unit Tests**:
   - Shell detection for each ShellType variant
   - Platform detection on different OS
   - Environment variable filtering logic
   - Sensitive pattern matching

2. **Integration Tests**:
   - Full context capture end-to-end
   - User override merging with auto-detection
   - Context serialization for LLM prompts

3. **Property Tests**:
   - Captured context always deserializes successfully
   - `to_prompt_context()` output never contains sensitive patterns
   - Environment variable filtering is deterministic

## Performance Requirements

| Operation | Target Latency | Constraint |
|-----------|----------------|------------|
| `capture()` | <50ms | NFR-003 requirement |
| `detect()` (shell) | <10ms | No process inspection needed |
| `to_prompt_context()` | <1ms | String formatting only |
| `has_env_var()` | <1μs | HashMap lookup |

## Thread Safety

- `ExecutionContext` is `Clone + Send + Sync` (immutable after creation)
- `ShellDetector` is `Send + Sync` (no mutable state)
- `PlatformDetector` is stateless (all static methods)

## Error Handling

Errors include:
- Specific component that failed (current_dir, username, etc.)
- System error context via `#[from] std::io::Error`
- Suggestions for recovery where applicable
- Never panics in production code
