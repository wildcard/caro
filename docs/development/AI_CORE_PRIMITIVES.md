# AI Core Primitives: The 50 Essential Concepts

> **Purpose**: This document defines the core primitives AI systems need to master for effective caro development. These ~50 concepts represent the complete vocabulary for basic operations.

## Category 1: Core Types (8 primitives)

### Data Types
| Type | Purpose | Example |
|------|---------|---------|
| `CommandRequest` | Input from user with context | `CommandRequest { prompt: "list files", context: ctx }` |
| `GeneratedCommand` | LLM output with metadata | `GeneratedCommand { cmd: "ls -la", confidence: 0.95 }` |
| `RiskLevel` | Safety classification | `RiskLevel::Safe`, `Moderate`, `High`, `Critical` |
| `BackendInfo` | Backend metadata | `BackendInfo { name: "mlx", version: "1.0" }` |
| `Config` | Application configuration | `Config { backend: BackendConfig, safety: SafetyConfig }` |
| `ExecutionContext` | System environment | `ExecutionContext { os: MacOS, shell: Zsh }` |
| `SafetyResult` | Validation outcome | `SafetyResult { allowed: true, risk: Safe }` |
| `BackendConfig` | Backend settings | `BackendConfig { timeout: 30s, model: "qwen" }` |

## Category 2: Traits (4 primitives)

### Core Traits
```rust
// 1. CommandGenerator - All backends implement this
#[async_trait]
trait CommandGenerator {
    async fn generate_command(&self, req: &CommandRequest) -> Result<GeneratedCommand>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
}

// 2. Validator - All safety checkers implement this
trait Validator {
    fn validate(&self, cmd: &str) -> SafetyResult;
    fn risk_level(&self, cmd: &str) -> RiskLevel;
}

// 3. Configurable - Types that load from config
trait Configurable {
    fn from_config(config: &Config) -> Result<Self>;
    fn default_config() -> Self;
}

// 4. Executable - Commands that can be run
trait Executable {
    fn execute(&self, shell: &Shell) -> Result<Output>;
    fn dry_run(&self) -> String;
}
```

## Category 3: Operations (12 primitives)

### Backend Operations
| Operation | Signature | Purpose |
|-----------|-----------|---------|
| `generate_command` | `async fn(&CommandRequest) -> Result<GeneratedCommand>` | Create command from prompt |
| `is_available` | `async fn() -> bool` | Check backend status |
| `backend_info` | `fn() -> BackendInfo` | Get backend metadata |

### Safety Operations
| Operation | Signature | Purpose |
|-----------|-----------|---------|
| `validate` | `fn(&str) -> SafetyResult` | Full safety check |
| `is_dangerous` | `fn(&str) -> bool` | Quick danger check |
| `risk_level` | `fn(&str) -> RiskLevel` | Get risk classification |
| `explain_risk` | `fn(&str) -> String` | Human-readable risk explanation |

### Execution Operations
| Operation | Signature | Purpose |
|-----------|-----------|---------|
| `execute` | `fn(&Command, &Shell) -> Result<Output>` | Run command |
| `detect_shell` | `fn() -> Shell` | Identify current shell |
| `detect_context` | `fn() -> ExecutionContext` | Get full system context |
| `confirm` | `fn(&Command, RiskLevel) -> bool` | Get user confirmation |
| `format_output` | `fn(&Output, Format) -> String` | Format for display |

## Category 4: Control Flow (6 primitives)

### Async Patterns
```rust
// 1. Async function
async fn fetch_command() -> Result<GeneratedCommand> { }

// 2. Await expression
let result = backend.generate_command(&req).await?;

// 3. Try expression (? operator)
let config = load_config()?;

// 4. Match on Result
match backend.generate_command(&req).await {
    Ok(cmd) => process(cmd),
    Err(e) => handle_error(e),
}

// 5. Match on enum
match risk_level {
    RiskLevel::Safe => execute(),
    RiskLevel::Critical => block(),
    _ => confirm_and_execute(),
}

// 6. If let for optionals
if let Some(model) = config.model {
    use_model(model);
}
```

## Category 5: Error Handling (6 primitives)

### Error Types
| Type | Use Case | Example |
|------|----------|---------|
| `anyhow::Error` | Application errors | `anyhow!("Config missing")` |
| `anyhow::Context` | Error context | `.context("Loading config")?` |
| `Result<T>` | Fallible operations | `Result<Config>` |
| `Option<T>` | Optional values | `Option<String>` |
| `thiserror` | Library error types | `#[derive(Error)]` |
| `bail!` | Early return with error | `bail!("Invalid input")` |

## Category 6: Configuration (6 primitives)

### Config Operations
```rust
// 1. Load default config
let config = Config::default();

// 2. Load from file
let config = Config::from_file("~/.config/caro/config.toml")?;

// 3. Merge configs
let config = default.merge(user_config);

// 4. Get nested value
let timeout = config.backend.timeout;

// 5. Override from environment
let config = config.with_env_overrides();

// 6. Validate config
config.validate()?;
```

## Category 7: I/O Operations (4 primitives)

### Console I/O
```rust
// 1. Print output
println!("Generated: {}", command);

// 2. Print error
eprintln!("Error: {}", error);

// 3. Read user input
let input = std::io::stdin().read_line(&mut buffer)?;

// 4. Colored output
use colored::Colorize;
println!("{}", "Safe".green());
```

## Category 8: Logging (4 primitives)

### Log Levels
```rust
// 1. Debug - Detailed debugging
log::debug!("Backend response: {:?}", response);

// 2. Info - General information
log::info!("Using backend: {}", backend.name());

// 3. Warn - Potential issues
log::warn!("Backend unavailable, falling back");

// 4. Error - Errors that don't crash
log::error!("Failed to connect: {}", e);
```

## Category 9: Testing (6 primitives)

### Test Patterns
```rust
// 1. Unit test
#[test]
fn test_safety_pattern() {
    assert!(is_dangerous("rm -rf /"));
}

// 2. Async test
#[tokio::test]
async fn test_backend() {
    let result = backend.generate(&req).await;
    assert!(result.is_ok());
}

// 3. Assertion
assert_eq!(result, expected);

// 4. Error assertion
assert!(result.is_err());

// 5. Setup/teardown
let _guard = setup_test_environment();

// 6. Property test
proptest! {
    #[test]
    fn fuzz_safety(cmd in any::<String>()) {
        let _ = validate(&cmd); // Should never panic
    }
}
```

## Category 10: Common Imports (6 primitives)

### Essential Imports
```rust
// 1. Async trait
use async_trait::async_trait;

// 2. Error handling
use anyhow::{anyhow, bail, Context, Result};

// 3. Serialization
use serde::{Deserialize, Serialize};

// 4. Async runtime
use tokio;

// 5. Regex for patterns
use regex::Regex;

// 6. CLI parsing
use clap::Parser;
```

## Summary: The Complete Primitive Set

| Category | Count | Core Concepts |
|----------|-------|---------------|
| Types | 8 | CommandRequest, GeneratedCommand, RiskLevel, etc. |
| Traits | 4 | CommandGenerator, Validator, Configurable, Executable |
| Operations | 12 | generate, validate, execute, detect, etc. |
| Control Flow | 6 | async/await, match, if let, ? operator |
| Error Handling | 6 | Result, Option, Context, bail |
| Configuration | 6 | load, merge, validate, override |
| I/O | 4 | print, read, colored output |
| Logging | 4 | debug, info, warn, error |
| Testing | 6 | test, async_test, assert, proptest |
| Imports | 6 | async_trait, anyhow, serde, tokio |

**Total: 56 primitives**

## Learning Path

### Beginner (First 20)
1. Core types: `CommandRequest`, `GeneratedCommand`, `RiskLevel`, `Config`
2. Essential operations: `generate_command`, `validate`, `execute`
3. Basic control: `async/await`, `match`, `?` operator
4. Simple I/O: `println!`, `log::info!`

### Intermediate (Next 20)
1. All traits: `CommandGenerator`, `Validator`
2. Error handling: `Context`, `bail!`, custom errors
3. Configuration: Loading, merging, validation
4. Testing: Unit tests, async tests

### Advanced (Remaining 16)
1. Property testing with `proptest`
2. Complex async patterns
3. Custom derive macros
4. Performance optimization patterns

---

*Master these primitives and you can implement any caro feature.*
*Last updated: 2026-01-12*
