# Copilot Instructions for caro

This file provides instructions for GitHub Copilot when reviewing code and generating suggestions for this repository.

## Project Overview

`caro` (formerly `caro`) is a single-binary Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. The tool prioritizes safety, performance, and developer experience with Apple Silicon optimization via MLX framework.

> **Important**: The project was renamed from `caro` to `caro` in December 2025. Flag any remaining references to `caro` in code, documentation, or configuration files.

## Core Principles

### Safety-First Development
- All generated shell commands MUST be validated for dangerous patterns before execution
- NEVER approve code that bypasses safety validation in the `src/safety/` module
- Flag any command patterns that could cause data loss, privilege escalation, or system damage
- The safety module uses 52+ predefined regex patterns to detect Critical, High, and Moderate risks

### Error Handling Standards
- Use `Result<T, Error>` types throughout; NEVER use `unwrap()` or `expect()` in production code
- All errors must use `thiserror` derive macros with meaningful error messages
- Propagate errors with context using `?` operator and appropriate error types

### Performance Requirements
- Startup time target: < 100ms
- First inference target: < 2s on M1 Mac
- Binary size target: < 50MB (release build)
- Flag any code that introduces unnecessary allocations or synchronous blocking

## Code Review Focus Areas

### High Priority
1. **Security vulnerabilities**: Command injection, unsafe shell operations, privilege escalation
2. **Safety validation bypasses**: Any code that could execute commands without proper validation
3. **Error handling**: Missing error propagation, panics in library code
4. **Async correctness**: Blocking calls in async contexts, missing `.await`
5. **Environment variable exposure**: Commands inheriting sensitive env vars without filtering

### Medium Priority
1. **API consistency**: Public APIs must have rustdoc comments
2. **Test coverage**: New features should include unit and integration tests
3. **POSIX compliance**: Shell commands must use standard utilities
4. **Cross-platform compatibility**: Code should work on macOS, Linux, and Windows
5. **CLI flag semantics**: Flag names must match actual behavior

### Lower Priority
1. **Code style**: Follow Rust conventions (use `cargo fmt` and `cargo clippy`)
2. **Documentation accuracy**: Claims in docs must match implementation
3. **Path/URL consistency**: Use consistent formats throughout files
4. **Terminology precision**: Use accurate terms (e.g., "predefined" not "pre-compiled" for patterns)

## Common Issues from Past Reviews

### Documentation Accuracy
Flag when documentation claims don't match implementation:
```markdown
<!-- BAD: Claims setup script handles prerequisites -->
"The setup script handles all prerequisites including Rust"
<!-- But actual install method is `cargo install caro` which requires Rust -->

<!-- GOOD: Accurate description -->
"Install with cargo (requires Rust toolchain)"
```

### CLI Flag Behavior
Ensure flag names match actual behavior:
```rust
// BAD: Flag named --interactive but auto-executes like --execute
#[arg(long, help = "Step-by-step confirmation")]
interactive: bool,  // Actually doesn't prompt

// GOOD: Name reflects behavior, or implement the claimed behavior
#[arg(long, help = "Execute command directly")]
execute: bool,
```

### Conflicting CLI Flags
Validate mutually exclusive flags:
```rust
// BAD: No validation for conflicting flags
if args.execute && args.dry_run {
    // Ambiguous behavior
}

// GOOD: Explicit validation
if args.execute && args.dry_run {
    return Err(anyhow!("Cannot use --execute and --dry-run together"));
}
```

### Timeout Implementation
Timeouts must actually interrupt operations:
```rust
// BAD: Timeout checked after blocking call completes
let output = cmd.output()?;  // Blocks until done
if elapsed > timeout { return Err(...); }

// GOOD: Async timeout that interrupts
tokio::time::timeout(duration, async {
    let child = cmd.spawn()?;
    child.wait_with_output().await
}).await??
```

### Process Cleanup on Timeout
When timing out, terminate child processes:
```rust
// BAD: Only stops waiting, process keeps running
if timeout_elapsed {
    return Err(TimeoutError);  // Child process still running!
}

// GOOD: Kill process on timeout
match tokio::time::timeout(duration, child.wait()).await {
    Err(_) => {
        child.kill().await?;  // Clean up
        Err(TimeoutError)
    }
    Ok(status) => Ok(status?)
}
```

### Hardcoded String Constants
Extract repeated values to constants:
```rust
// BAD: Status strings scattered throughout code
if status == "implemented" { ... }
if status == "in-progress" { ... }

// GOOD: Define constants
const STATUS_IMPLEMENTED: &str = "implemented";
const STATUS_IN_PROGRESS: &str = "in-progress";
```

### Field Naming Semantics
Names should reflect actual meaning:
```rust
// BAD: `executed` but actually means "passed safety checks"
pub struct CommandResult {
    pub executed: bool,  // Misleading name
}

// GOOD: Clear semantic meaning
pub struct CommandResult {
    pub passed_safety_checks: bool,
}
```

## Architecture Guidelines

### Backend Trait System
All model backends implement `CommandGenerator` trait with:
- Async inference with `Result<GeneratedCommand, GeneratorError>` responses
- Availability checking via `is_available()` method
- Proper shutdown handling

### Module Organization
```
src/
  backends/     - LLM inference backends (MLX, vLLM, Ollama)
  safety/       - Command validation (CRITICAL - security-sensitive)
  cache/        - Model caching with Hugging Face Hub
  config/       - Configuration management
  cli/          - Command-line interface
  models/       - Data structures and types
  execution/    - Command execution with safety checks
  platform/     - Cross-platform utilities
```

## Testing Standards

### Required Checks
- `cargo fmt --all --check` - Code formatting
- `cargo clippy -- -D warnings` - Linter with warnings as errors
- `cargo test` - All tests must pass
- `cargo audit` - No known security vulnerabilities

### Async Test Patterns
```rust
// BAD: .await in non-async test
#[test]
fn test_async_function() {
    let result = async_fn().await;  // Won't compile!
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
```

### Test Types Required
- **Unit tests**: Safety pattern validation, command parsing, configuration
- **Integration tests**: End-to-end workflows, backend communication
- **Property tests**: Safety validation with random inputs (use `proptest`)

## Dangerous Patterns to Flag

### Always Block
- `rm -rf /` or `rm -rf ~` - Filesystem destruction
- `mkfs`, `dd if=/dev/zero` - Disk operations
- `:(){ :|:& };:` - Fork bombs
- `chmod 777 /` or system path modifications
- `sudo su`, `curl | bash` - Privilege escalation

### Require Confirmation
- Any command with `--force` flags
- Commands modifying system directories (`/etc`, `/usr`, `/bin`)
- Network operations with sensitive data

## Input Validation

### Empty/Invalid Input
```rust
// BAD: No validation before processing
pub fn execute_command(cmd: &str) -> Result<()> {
    Command::new("sh").arg("-c").arg(cmd).output()?;
}

// GOOD: Validate input first
pub fn execute_command(cmd: &str) -> Result<()> {
    if cmd.trim().is_empty() {
        return Err(anyhow!("Command cannot be empty"));
    }
    Command::new("sh").arg("-c").arg(cmd).output()?;
}
```

## Pull Request Guidelines

When reviewing PRs:
1. Verify safety-critical changes have comprehensive tests
2. Check that error handling follows project standards
3. Ensure documentation is updated for API changes
4. Validate that new dependencies are justified and audited
5. Confirm CI passes before approval
6. Check for unreachable code paths
7. Verify claims in documentation match implementation
8. Ensure path/URL formats are consistent within files
9. Flag code duplication that should be extracted to helpers

## POSIX Compliance

### Required Practices
- Use standard utilities: `ls`, `find`, `grep`, `awk`, `sed`, `sort`
- Proper path quoting for spaces and special characters
- Avoid bash-specific features for maximum portability
- Validate command syntax before execution

## Language and Communication

- Provide specific, actionable feedback
- Reference line numbers and file paths when commenting
- Prioritize security and correctness over style
- Be concise - avoid lengthy explanations for obvious issues
- Use accurate terminology matching the codebase
