---
applyTo: "**/*.rs"
---

# Rust Code Review Instructions

## Error Handling

### Required Patterns
- Use `Result<T, E>` for fallible operations; never panic in library code
- Define domain-specific errors with `thiserror` derive macros
- Propagate errors with `?` operator; add context when crossing module boundaries
- Use `anyhow::Result` only in binary/CLI code, not in library modules

### Anti-Patterns to Flag
```rust
// BAD: unwrap() in production code
let value = option.unwrap();

// GOOD: Proper error handling
let value = option.ok_or(MyError::MissingValue)?;

// BAD: expect() without context
let config = parse_config().expect("failed");

// GOOD: Descriptive error
let config = parse_config()
    .map_err(|e| ConfigError::ParseFailed { source: e })?;
```

## Async/Await Patterns

### Required Patterns
- Use `#[tokio::main]` for async entry points
- Use `#[async_trait]` for async trait methods
- Prefer `tokio::spawn` for concurrent operations
- Use `tokio::time::timeout` for operations with time limits

### Anti-Patterns to Flag
```rust
// BAD: Blocking in async context
async fn bad() {
    std::thread::sleep(Duration::from_secs(1)); // blocks executor
}

// GOOD: Async sleep
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// BAD: Sync file I/O in async
async fn bad_io() {
    std::fs::read_to_string("file.txt"); // blocks
}

// GOOD: Async file I/O
async fn good_io() {
    tokio::fs::read_to_string("file.txt").await;
}
```

### Timeout and Process Management
```rust
// BAD: Timeout checked after blocking call
let output = cmd.output()?;  // Blocks until complete
if start.elapsed() > timeout {
    return Err(TimeoutError);  // Too late!
}

// GOOD: Async timeout with process cleanup
match tokio::time::timeout(duration, child.wait()).await {
    Ok(status) => Ok(status?),
    Err(_) => {
        child.kill().await?;  // CRITICAL: Kill orphaned process
        Err(TimeoutError)
    }
}
```

## CLI Design Patterns

### Flag Validation
```rust
// BAD: Conflicting flags allowed
#[derive(Parser)]
struct Args {
    #[arg(long)]
    execute: bool,
    #[arg(long)]
    dry_run: bool,
}
// User can pass both --execute and --dry-run

// GOOD: Validate at parse time
impl Args {
    fn validate(&self) -> Result<()> {
        if self.execute && self.dry_run {
            return Err(anyhow!("Cannot use --execute and --dry-run together"));
        }
        Ok(())
    }
}
```

### Flag Naming
```rust
// BAD: Name doesn't match behavior
#[arg(long, help = "Step-by-step confirmation")]
interactive: bool,  // But actually auto-executes

// GOOD: Accurate naming and implementation
#[arg(long, help = "Execute command without confirmation")]
execute: bool,
```

### Input Validation
```rust
// BAD: No validation before processing
pub fn execute_command(cmd: &str) -> Result<()> {
    Command::new("sh").arg("-c").arg(cmd).output()?;
}

// GOOD: Validate input first
pub fn execute_command(cmd: &str) -> Result<()> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return Err(anyhow!("Command cannot be empty"));
    }
    if cmd.len() > MAX_COMMAND_LENGTH {
        return Err(anyhow!("Command exceeds maximum length"));
    }
    Command::new("sh").arg("-c").arg(cmd).output()?;
}
```

## Memory and Performance

### Required Patterns
- Use `&str` over `String` when ownership isn't needed
- Prefer `Vec::with_capacity` when size is known
- Use `once_cell::Lazy` for expensive static initializations
- Clone explicitly when needed; avoid hidden allocations

### Flag These Issues
- Unnecessary `.clone()` calls
- `to_string()` in hot paths when `&str` would work
- Unbounded collections without size limits
- Missing `#[inline]` on small, frequently called functions

## Trait Design

### Required Patterns
```rust
// Backend traits must be Send + Sync for async contexts
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand, GeneratorError>;

    async fn is_available(&self) -> bool;

    fn backend_info(&self) -> BackendInfo;
}
```

### Guidelines
- Traits in `src/backends/` must include `Send + Sync` bounds
- Provide default implementations for optional methods
- Document trait invariants and expected behavior

## Testing Requirements

### Async Test Syntax
```rust
// BAD: .await in non-async test (won't compile)
#[test]
fn test_async_function() {
    let result = async_fn().await;  // ERROR!
}

// GOOD: Use tokio::test for async
#[tokio::test]
async fn test_async_function() {
    let result = async_fn().await;
    assert!(result.is_ok());
}
```

### Platform-Specific Tests
```rust
// BAD: Unix-only test without guard
#[test]
fn test_shell_command() {
    Command::new("bash").arg("-c").arg("sleep 1");  // Fails on Windows
}

// GOOD: Platform guard
#[test]
#[cfg(unix)]
fn test_shell_command() {
    Command::new("bash").arg("-c").arg("sleep 1");
}

// GOOD: Cross-platform alternative
#[test]
fn test_shell_command() {
    #[cfg(unix)]
    let shell = "bash";
    #[cfg(windows)]
    let shell = "cmd";
    // ...
}
```

### Test Helper Consistency
```rust
// BAD: Test helper returns wrong values
impl TestArgs {
    fn execute(&self) -> bool {
        false  // Always returns false, ignores actual field!
    }
}

// GOOD: Return actual field values
impl TestArgs {
    fn execute(&self) -> bool {
        self.execute
    }
}
```

### Required for Safety Code
- Property-based tests with `proptest` for input validation
- Edge case coverage: empty strings, max lengths, unicode
- Integration tests for cross-module workflows

## Naming Semantics

### Field Names Must Reflect Meaning
```rust
// BAD: Misleading field name
pub struct CommandResult {
    pub executed: bool,  // Actually means "passed safety checks"
}

// GOOD: Clear semantic meaning
pub struct CommandResult {
    pub passed_safety_checks: bool,
}

// BAD: Confusing boolean
pub allowed: bool,  // Allowed to do what?

// GOOD: Specific meaning
pub safe_to_execute: bool,
```

### Constants for Repeated Values
```rust
// BAD: Magic strings throughout code
if status == "implemented" { ... }
if status == "in-progress" { ... }

// GOOD: Define constants
pub mod status {
    pub const IMPLEMENTED: &str = "implemented";
    pub const IN_PROGRESS: &str = "in-progress";
    pub const PLANNED: &str = "planned";
}
```

## Environment Variable Security

```rust
// BAD: Command inherits all env vars
Command::new("sh")
    .arg("-c")
    .arg(user_command)
    .output()?;  // Leaks API keys, tokens, etc.

// GOOD: Clear or filter environment
Command::new("sh")
    .arg("-c")
    .arg(user_command)
    .env_clear()  // Start with empty env
    .env("PATH", safe_path)  // Add only what's needed
    .output()?;
```

## Serialization

### Required Patterns
- Use `#[derive(Serialize, Deserialize)]` from `serde`
- Use `#[serde(rename_all = "camelCase")]` for JSON APIs
- Define explicit `#[serde(default)]` values for optional fields

### Anti-Patterns
```rust
// BAD: Magic strings in serialization
#[serde(rename = "my_field")]  // inconsistent naming

// GOOD: Consistent naming convention
#[serde(rename_all = "snake_case")]
pub struct Config { ... }
```

## Documentation Standards

### Required Documentation
- All public functions, structs, and traits need rustdoc comments
- Include `# Example` sections for complex APIs
- Document panics, errors, and safety considerations

```rust
/// Validates a shell command for dangerous patterns.
///
/// # Arguments
/// * `command` - The shell command string to validate
/// * `shell` - The target shell type (Bash, Zsh, etc.)
///
/// # Returns
/// * `Ok(ValidationResult)` - Validation completed successfully
/// * `Err(ValidationError)` - Validation failed (not the same as command being dangerous)
///
/// # Example
/// ```
/// let validator = SafetyValidator::new(SafetyConfig::moderate())?;
/// let result = validator.validate_command("ls -la", ShellType::Bash).await?;
/// assert!(result.allowed);
/// ```
pub async fn validate_command(...) -> Result<ValidationResult, ValidationError>
```

## Feature Flags

### Current Features
- `default = ["embedded-mlx", "embedded-cpu"]`
- `remote-backends` - Enable vLLM and Ollama backends
- `embedded-mlx` - Apple Silicon optimization
- `embedded-cpu` - Cross-platform Candle backend

### Guidelines
- Gate platform-specific code with appropriate `#[cfg(...)]`
- Ensure code compiles with `--no-default-features`
- Test feature combinations in CI

## Clippy Lints

The following must pass: `cargo clippy -- -D warnings`

### Common Issues to Watch
- `clippy::unwrap_used` - Use proper error handling
- `clippy::expect_used` - Provide context or use Result
- `clippy::panic` - Never panic in library code
- `clippy::todo` - Remove before merging
- `clippy::dbg_macro` - Remove debug statements

## Legacy References

> **Important**: The project was renamed from `cmdai` to `caro`. Flag any remaining references to `cmdai` in code, documentation, imports, or configuration.
