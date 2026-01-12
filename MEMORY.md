# MEMORY.md - AI Learning Reference for Caro

> **Purpose**: This document teaches AI systems (Claude, GPT, Copilot) the patterns, idioms, debugging workflows, and common errors specific to this codebase. Read this first before generating code.

## Teaching AI Systems Caro

**Three-layer learning approach:**

1. **Read MEMORY.md first** - Patterns, idioms, and error fixes (this document)
2. **Reference docs/development/CLAUDE.md** - Architecture and project structure
3. **Study src/ and tests/** - Working examples of idiomatic usage

## Core Principles

### 1. Safety-First Design

All command generation MUST go through safety validation. There is exactly one canonical way to validate commands:

```rust
// CANONICAL: Always use SafetyValidator
use caro::safety::SafetyValidator;

let validator = SafetyValidator::new();
let result = validator.validate(&command)?;

match result.risk_level {
    RiskLevel::Safe => execute(command),
    RiskLevel::Critical => return Err(SafetyError::Blocked),
    _ => prompt_user_confirmation(command, result),
}
```

**NEVER** generate or execute commands without safety validation.

### 2. Async-First Architecture

All I/O operations use async/await with tokio. No blocking calls in async contexts:

```rust
// CORRECT: Async HTTP call
let response = client.post(&url).json(&request).send().await?;

// WRONG: Blocking in async context
let response = std::thread::spawn(|| blocking_call()).join(); // NEVER
```

### 3. Error Handling with Context

Use `anyhow` for application errors and `thiserror` for library errors. Always provide context:

```rust
// CORRECT: Context for debugging
let config = load_config()
    .context("Failed to load configuration from ~/.config/caro/config.toml")?;

// WRONG: Bare error propagation
let config = load_config()?; // Missing context
```

### 4. Trait-Based Extensibility

All backends implement the `CommandGenerator` trait. This is the ONLY way to add new backends:

```rust
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
}
```

## Quick Reference

### Module Structure

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `backends/` | LLM inference | `CommandGenerator`, `BackendInfo` |
| `safety/` | Command validation | `SafetyValidator`, `RiskLevel` |
| `config/` | Configuration | `Config`, `BackendConfig` |
| `cli/` | User interface | Argument parsing, output formatting |
| `execution/` | Command execution | Shell detection, safe execution |

### Common Patterns

#### 1. Backend Fallback Chain

```rust
// CANONICAL: Try backends in order, fallback gracefully
async fn generate_with_fallback(request: &CommandRequest) -> Result<GeneratedCommand> {
    let backends = vec![
        Box::new(EmbeddedBackend::new()) as Box<dyn CommandGenerator>,
        Box::new(OllamaBackend::new()),
        Box::new(VllmBackend::new()),
    ];

    for backend in backends {
        if backend.is_available().await {
            match backend.generate_command(request).await {
                Ok(cmd) => return Ok(cmd),
                Err(e) => log::warn!("Backend failed: {}", e),
            }
        }
    }

    Err(anyhow!("All backends unavailable"))
}
```

#### 2. JSON Response Parsing

LLM responses may contain markdown or extra text. Use the canonical extraction pattern:

```rust
// CANONICAL: Extract JSON from potentially wrapped responses
fn extract_command_json(response: &str) -> Result<GeneratedCommand> {
    // Try direct parse first
    if let Ok(cmd) = serde_json::from_str(response) {
        return Ok(cmd);
    }

    // Try extracting from code blocks
    if let Some(json_str) = extract_json_block(response) {
        return serde_json::from_str(&json_str)
            .context("Failed to parse extracted JSON");
    }

    // Last resort: regex extraction for {"cmd": "..."} pattern
    extract_command_regex(response)
        .context("No valid command JSON found in response")
}
```

#### 3. Platform-Specific Logic

Always use runtime detection, not compile-time conditionals for command generation:

```rust
// CORRECT: Runtime platform detection
let platform = ExecutionContext::detect();
match platform.os {
    Os::MacOS => generate_bsd_compatible(request),
    Os::Linux => generate_gnu_compatible(request),
    Os::Windows => generate_powershell(request),
}

// WRONG: Compile-time flags for runtime behavior
#[cfg(target_os = "macos")]  // Only use for actual compilation differences
fn generate_command() { }
```

#### 4. Safety Pattern Matching

Use pre-compiled regex patterns for performance:

```rust
// CANONICAL: Pre-compiled patterns in lazy_static
lazy_static! {
    static ref DANGEROUS_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"rm\s+-rf\s+/").unwrap(),
        Regex::new(r"mkfs\.").unwrap(),
        Regex::new(r"dd\s+if=.*/dev/zero").unwrap(),
        // ... 52+ patterns
    ];
}

fn is_dangerous(cmd: &str) -> bool {
    DANGEROUS_PATTERNS.iter().any(|p| p.is_match(cmd))
}
```

#### 5. Configuration Loading

```rust
// CANONICAL: Layered config with defaults
fn load_config() -> Result<Config> {
    let default = Config::default();
    let user_config = load_user_config()?;
    let env_overrides = load_env_overrides()?;

    // Merge in order: default < user < env
    default.merge(user_config).merge(env_overrides)
}
```

## Error Taxonomy

### Compilation Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `trait bound CommandGenerator is not satisfied` | Missing `#[async_trait]` macro | Add `use async_trait::async_trait;` and `#[async_trait]` |
| `cannot borrow as mutable` | Trying to mutate through shared reference | Use `Arc<Mutex<T>>` or `RwLock<T>` |
| `future cannot be sent between threads safely` | Non-Send type in async | Ensure all types implement `Send + Sync` |
| `lifetime may not live long enough` | Reference outlives scope | Use owned types or add lifetime bounds |
| `unresolved import` | Missing dependency | Check Cargo.toml and feature flags |

### Runtime Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `Backend unavailable` | Service not running | Start Ollama/vLLM or use embedded backend |
| `JSON parse error` | LLM returned malformed response | Use `extract_command_json()` with fallbacks |
| `Command blocked by safety` | Matched dangerous pattern | Review command, adjust safety level if needed |
| `Config not found` | Missing config file | Use `Config::default()` as fallback |
| `Timeout waiting for response` | Slow inference | Increase timeout or use smaller model |

### Logic Errors

| Symptom | Cause | Fix |
|---------|-------|-----|
| Commands fail on macOS | GNU vs BSD difference | Use `ExecutionContext` for platform detection |
| Safety false positives | Pattern too broad | Add context-aware validation |
| Wrong shell syntax | Shell not detected | Explicitly specify shell in config |
| Commands not executing | Missing execution permission | Check `--confirm` flag handling |

## Debugging Workflow

Follow this 5-step process for any issue:

### 1. Reproduce with Logging

```bash
RUST_LOG=debug cargo run -- "your command"
```

### 2. Check Component in Isolation

```rust
// Unit test the specific component
#[tokio::test]
async fn test_failing_component() {
    let backend = EmbeddedBackend::new();
    let result = backend.generate_command(&request).await;
    assert!(result.is_ok());
}
```

### 3. Verify Safety Validation

```bash
cargo test --package caro --lib safety::tests
```

### 4. Check Platform Detection

```rust
let ctx = ExecutionContext::detect();
println!("OS: {:?}, Shell: {:?}", ctx.os, ctx.shell);
```

### 5. Inspect Generated Command

Add `--verbose` or `--output json` to see full generation details.

## Idioms for AI Code Generation

### DO Use These Patterns

1. **Struct builders for complex config**
```rust
BackendConfig::new()
    .with_timeout(Duration::from_secs(30))
    .with_model("qwen2.5-coder")
    .build()
```

2. **Result types for all fallible operations**
```rust
fn parse_command(input: &str) -> Result<Command> { }
```

3. **Explicit type annotations on complex closures**
```rust
let processor: Box<dyn Fn(&str) -> Result<String>> = Box::new(|s| Ok(s.to_string()));
```

4. **Constants for magic values**
```rust
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const MAX_RETRIES: u32 = 3;
```

5. **Descriptive error messages**
```rust
return Err(anyhow!(
    "Failed to connect to Ollama at {}. Is the service running?",
    base_url
));
```

### DON'T Use These Anti-Patterns

1. **Unwrap in production code**
```rust
// WRONG
let config = load_config().unwrap();

// CORRECT
let config = load_config().context("Loading configuration")?;
```

2. **Panics for recoverable errors**
```rust
// WRONG
panic!("Backend not available");

// CORRECT
return Err(BackendError::Unavailable);
```

3. **Stringly-typed APIs**
```rust
// WRONG
fn set_level(level: &str) { }

// CORRECT
fn set_level(level: RiskLevel) { }
```

4. **Blocking in async**
```rust
// WRONG
async fn fetch() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks executor!
}

// CORRECT
async fn fetch() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

5. **Ignoring errors silently**
```rust
// WRONG
let _ = risky_operation();

// CORRECT
if let Err(e) = risky_operation() {
    log::warn!("Operation failed: {}", e);
}
```

## Testing Patterns

### Unit Tests

Every module has tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let validator = SafetyValidator::new();
        assert!(validator.is_dangerous("rm -rf /"));
        assert!(!validator.is_dangerous("ls -la"));
    }
}
```

### Integration Tests

Located in `tests/` directory:

```rust
// tests/backend_integration.rs
#[tokio::test]
async fn test_backend_fallback() {
    let generator = MultiBackendGenerator::new();
    let result = generator.generate(&request).await;
    assert!(result.is_ok());
}
```

### Property-Based Tests

Use `proptest` for edge cases:

```rust
proptest! {
    #[test]
    fn safety_never_false_negative(cmd in any::<String>()) {
        let validator = SafetyValidator::new();
        // Critical patterns must ALWAYS be detected
        if cmd.contains("rm -rf /") {
            prop_assert!(validator.is_dangerous(&cmd));
        }
    }
}
```

## File Locations

| What You Need | Where To Find It |
|---------------|------------------|
| CLI entry point | `src/main.rs` |
| Backend trait | `src/backends/mod.rs` |
| Safety patterns | `src/safety/mod.rs` |
| Configuration | `src/config/mod.rs` |
| Integration tests | `tests/` |
| Project specs | `specs/` |
| Feature worktrees | `kitty-specs/` |

## Machine-Readable Diagnostics

For AI agents, use these flags for structured output:

```bash
# JSON output for programmatic parsing
caro --output json "list files"

# Verbose mode with timing
caro --verbose "find large files"

# Check safety without executing
caro --dry-run "remove old logs"
```

## Version History

| Version | Key Changes |
|---------|-------------|
| v1.1.0 | Embedded backends, agentic loop, platform detection |
| v1.0.x | Initial release, basic CLI, safety validation |

---

*Last updated: 2026-01-12*
*For project architecture, see `docs/development/CLAUDE.md`*
