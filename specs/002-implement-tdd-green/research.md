# Research Decisions: TDD GREEN Phase

**Feature**: 002-implement-tdd-green | **Date**: 2025-10-01 | **Phase**: 0

This document captures the research findings and technical decisions made for implementing the GREEN phase of caro's TDD cycle.

## Overview

Five critical technical decisions were researched to guide implementation:
1. Async trait implementation patterns
2. Regex compilation strategy for performance
3. Clap v4 CLI structure and patterns
4. Error handling boundaries (library vs binary)
5. Cross-platform path validation approaches

## Decision 1: Async Trait Implementation

### Problem Statement
The `CommandGenerator` trait requires async methods for backend communication (HTTP APIs, local model inference). Rust traits don't natively support async methods without additional setup.

### Research Questions
- How to implement async trait methods in Rust 1.75+?
- Error handling patterns in async trait contexts
- Testing strategies for async trait implementations
- Performance implications and lifetime management

### Options Evaluated

**Option A: Manual Future Implementation**
- Manual implementation of `Future` trait for each method
- Full control over execution and polling
- Rejected: Too complex, error-prone, unnecessary boilerplate

**Option B: Synchronous Trait with Blocking**
- Use blocking trait methods, spawn threads for async work
- Simpler trait definition
- Rejected: Blocks I/O threads, poor async runtime integration

**Option C: async-trait Crate (CHOSEN)**
- Use `#[async_trait]` macro from async-trait crate
- Standard community solution with zero-cost abstraction
- Clean syntax, good error handling support

### Decision: async-trait Crate

**Implementation Pattern**:
```rust
use async_trait::async_trait;

#[async_trait]
pub trait CommandGenerator: Send + Sync {
    async fn generate_command(
        &self,
        request: &CommandRequest
    ) -> Result<GeneratedCommand, Box<dyn std::error::Error>>;

    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
}
```

**Error Handling Pattern**:
- Use `Box<dyn Error>` for trait errors (allows any error type)
- Concrete implementations use specific error types
- Convert to anyhow::Error at binary boundaries

**Testing Pattern**:
```rust
#[tokio::test]
async fn test_async_trait_implementation() {
    let backend = MockBackend::new("test");
    let request = CommandRequest::new("test", ShellType::Bash);
    let result = backend.generate_command(&request).await;
    assert!(result.is_ok());
}
```

**Rationale**:
- Industry standard solution (used by tokio, async-std ecosystems)
- Zero-cost abstraction via macro expansion
- Excellent error handling support
- Compatible with all async runtimes
- Well-documented with extensive community usage

**Trade-offs**:
- Adds dependency (but widely-used, stable crate)
- Slightly obscured trait definition (macro magic)
- Accepted: Benefits far outweigh minimal complexity

## Decision 2: Regex Compilation Strategy

### Problem Statement
Safety validation requires pattern matching against dangerous commands. Performance target is <50ms for validation. Regex compilation is expensive (~1-5ms per pattern), and we have 15+ patterns to check.

### Research Questions
- Static vs dynamic regex compilation approaches
- Performance impact of compilation overhead
- Pattern storage and sharing strategies
- Memory usage considerations

### Options Evaluated

**Option A: Per-Call Compilation**
- Compile regex on each validation call
- Simple, no global state
- Rejected: ~75-150ms total (15 patterns × 5ms), exceeds target

**Option B: Build-Time Code Generation**
- Generate compiled FSM at build time (regex-automata)
- Maximum performance
- Rejected: Overkill for 15 patterns, complex tooling

**Option C: Lazy Static Compilation (CHOSEN)**
- Compile patterns once at first use
- Share compiled Regex across all validations
- Use `once_cell::sync::Lazy` for thread-safe initialization

### Decision: Lazy Static with once_cell

**Implementation Pattern**:
```rust
use once_cell::sync::Lazy;
use regex::Regex;

static DANGEROUS_PATTERNS: Lazy<Vec<(Regex, &'static str, RiskLevel)>> = Lazy::new(|| {
    vec![
        (
            Regex::new(r"rm\s+-rf\s+/").unwrap(),
            "Recursive deletion of root filesystem",
            RiskLevel::Critical
        ),
        (
            Regex::new(r":\(\)\{.*:\|:&\s*\};:").unwrap(),
            "Fork bomb detected",
            RiskLevel::Critical
        ),
        // ... 13 more patterns
    ]
});

pub fn validate_command(cmd: &str) -> ValidationResult {
    for (pattern, desc, level) in DANGEROUS_PATTERNS.iter() {
        if pattern.is_match(cmd) {
            return ValidationResult::dangerous(level.clone(), desc);
        }
    }
    ValidationResult::safe()
}
```

**Performance Characteristics**:
- First call: ~75ms (compile all patterns once)
- Subsequent calls: ~5-10ms (match only, no compilation)
- Target <50ms: Met after first call (amortized over lifetime)

**Memory Usage**:
- ~15KB for compiled pattern FSMs (static lifetime)
- Shared across all threads and validation calls
- Acceptable overhead for performance gain

**Rationale**:
- 5-15x performance improvement vs per-call compilation
- Thread-safe with zero runtime locking after initialization
- Clean API with no global state management needed
- Pattern compilation errors caught at first use (panics with clear message)

**Trade-offs**:
- First validation call slower (one-time cost)
- Static lifetime patterns (cannot reload without restart)
- Accepted: Performance benefits justify constraints

## Decision 3: Clap CLI Structure

### Problem Statement
CLI needs complex argument handling: nested commands, multiple output formats, environment variables, shell type selection. Need compile-time validation and user-friendly help text.

### Research Questions
- Derive macros vs builder API patterns
- Subcommand organization strategies
- Custom value parser implementation
- Environment variable integration

### Options Evaluated

**Option A: Builder API**
- Programmatic command construction
- Maximum flexibility, runtime configuration
- Rejected: Verbose, error-prone, loses compile-time validation

**Option B: Manual Argument Parsing**
- Parse std::env::args() manually
- Full control over parsing logic
- Rejected: Reinvents wheel, poor error messages, no help generation

**Option C: Derive Macros (CHOSEN)**
- Use `#[command()]` and `#[arg()]` attributes
- Compile-time validation with type safety
- Auto-generated help text and error messages

### Decision: Clap Derive Macros

**Implementation Pattern**:
```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "caro")]
#[command(about = "AI-powered POSIX command generation")]
#[command(version)]
pub struct CliApp {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, short, global = true, value_enum)]
    pub format: Option<OutputFormat>,

    #[arg(long, global = true, env = "CARO_LOG_LEVEL")]
    pub log_level: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a shell command from natural language
    Generate {
        /// Natural language description
        input: String,

        #[arg(long, short)]
        shell: Option<ShellType>,

        #[arg(long, short)]
        safety: Option<SafetyLevel>,
    },

    /// Validate a command for safety issues
    Validate {
        /// Command to validate
        command: String,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Json,
    Yaml,
    Text,
}
```

**Subcommand Structure**:
- `caro generate <input>` - Primary command generation
- `caro validate <command>` - Safety validation only
- Global flags: `--format`, `--log-level`, `--version`, `--help`

**Custom Value Parser**:
```rust
impl FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            "fish" => Ok(ShellType::Fish),
            _ => Err(format!("Unknown shell type: {}", s))
        }
    }
}
```

**Rationale**:
- Type-safe argument parsing at compile time
- Auto-generated help text matches code structure
- Clear error messages for invalid arguments
- Environment variable support built-in
- Excellent documentation and ecosystem support

**Trade-offs**:
- Macro expansion obscures generated code
- Less flexibility for runtime command changes
- Accepted: CLI structure is static, compile-time safety preferred

## Decision 4: Error Handling Boundaries

### Problem Statement
Library code needs typed errors for caller pattern matching. Binary code needs context chains and user-friendly messages. How to balance type safety with ergonomics?

### Research Questions
- When to use thiserror vs anyhow
- Error conversion patterns at boundaries
- User-facing message formatting
- Debug vs Display implementations

### Options Evaluated

**Option A: anyhow Everywhere**
- Simple, ergonomic error handling everywhere
- Good context chains with `.context()`
- Rejected: Library callers cannot match on error types

**Option B: thiserror Everywhere**
- Type-safe errors throughout codebase
- Explicit error variants and conversions
- Rejected: Binary code too verbose, loses context ergonomics

**Option C: Boundary-Based Strategy (CHOSEN)**
- thiserror in library code (src/*/mod.rs)
- anyhow in binary code (src/main.rs, src/cli/)
- `impl From<LibError> for anyhow::Error` at boundaries

### Decision: thiserror in Libraries, anyhow in Binary

**Library Error Pattern** (thiserror):
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SafetyError {
    #[error("Invalid command syntax: {0}")]
    InvalidSyntax(String),

    #[error("Dangerous pattern detected: {pattern} - {reason}")]
    DangerousPattern {
        pattern: String,
        reason: String,
    },

    #[error("Configuration error: {0}")]
    ConfigError(#[from] std::io::Error),
}
```

**Binary Error Handling** (anyhow):
```rust
use anyhow::{Context, Result};

fn run_cli() -> Result<()> {
    let request = CommandRequest::new(input, shell)
        .context("Failed to create command request")?;

    let command = backend.generate_command(&request).await
        .context("Backend generation failed")?;

    let result = validator.validate(&command.command, shell)
        .context("Safety validation failed")?;

    Ok(())
}
```

**Boundary Conversion**:
```rust
// Automatic via From trait
impl From<SafetyError> for anyhow::Error {
    fn from(err: SafetyError) -> Self {
        anyhow::Error::new(err)
    }
}
```

**User Message Formatting**:
```rust
// Display for users, Debug for logs
impl Display for SafetyError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SafetyError::DangerousPattern { pattern, reason } => {
                write!(f, "Safety check failed: {}\n  Pattern: {}", reason, pattern)
            }
            _ => write!(f, "{:?}", self)
        }
    }
}
```

**Rationale**:
- Library errors enable caller error handling (match on variants)
- Binary errors optimize for context chains and user messages
- Clear separation of concerns (type safety vs ergonomics)
- Standard Rust ecosystem pattern

**Trade-offs**:
- Two error handling styles to maintain
- Conversion boilerplate at boundaries
- Accepted: Clarity and type safety justify duplication

## Decision 5: Cross-Platform Path Validation

### Problem Statement
Safety validation must handle both POSIX shells (bash, zsh, fish, sh) and Windows shells (PowerShell, cmd.exe). Different quote rules, escaping mechanisms, and special characters per shell.

### Research Questions
- Quote escaping differences (single vs double quotes)
- Special character handling per shell family
- Path separator validation (/ vs \)
- Shell-specific dangerous patterns

### Options Evaluated

**Option A: Unified Validator**
- Single validation function with shell-specific branches
- Centralized logic
- Rejected: Too complex, hard to test shell-specific edge cases

**Option B: Shell-Specific Crates**
- Use platform crates (nix, winapi) for validation
- Platform-native validation
- Rejected: Heavy dependencies, overkill for pattern matching

**Option C: Separate Validator Functions (CHOSEN)**
- `validate_posix()` for bash/zsh/fish/sh
- `validate_windows()` for PowerShell/cmd
- Shell detection via ShellType enum

### Decision: Separate Validator Functions per Shell Family

**Architecture Pattern**:
```rust
pub mod validators {
    use crate::models::{ShellType, ValidationResult};

    pub fn validate_command(cmd: &str, shell: ShellType) -> ValidationResult {
        match shell {
            ShellType::Bash | ShellType::Zsh | ShellType::Fish | ShellType::Sh => {
                validate_posix(cmd, shell)
            }
            ShellType::PowerShell | ShellType::Cmd => {
                validate_windows(cmd, shell)
            }
            ShellType::Unknown => {
                ValidationResult::moderate("Unknown shell type, using conservative validation")
            }
        }
    }

    fn validate_posix(cmd: &str, shell: ShellType) -> ValidationResult {
        // Check for unquoted paths with spaces
        if cmd.contains(" /") && !cmd.contains("\"") && !cmd.contains("'") {
            return ValidationResult::moderate("Unquoted path may cause issues");
        }

        // Check for dangerous expansions
        if cmd.contains("$(") || cmd.contains("`") {
            return ValidationResult::high("Command substitution detected");
        }

        ValidationResult::safe()
    }

    fn validate_windows(cmd: &str, shell: ShellType) -> ValidationResult {
        // Check for unquoted paths with spaces
        if cmd.contains(r" C:\") && !cmd.contains("\"") {
            return ValidationResult::moderate("Unquoted Windows path may cause issues");
        }

        // Check for PowerShell script execution
        if shell == ShellType::PowerShell && cmd.contains("-ExecutionPolicy Bypass") {
            return ValidationResult::high("Execution policy bypass detected");
        }

        ValidationResult::safe()
    }
}
```

**Shell-Specific Pattern Differences**:

| Pattern Type | POSIX | Windows |
|--------------|-------|---------|
| Path separator | `/` | `\` or `/` |
| Quote escaping | `\'` or `"` | `` ` `` or `"` |
| Variable expansion | `$VAR`, `${VAR}` | `$env:VAR`, `%VAR%` |
| Command substitution | `$(cmd)`, `` `cmd` `` | `&(cmd)`, `$(cmd)` |
| Dangerous deletion | `rm -rf /` | `Remove-Item -Recurse C:\` |

**Shell Detection**:
```rust
impl ShellType {
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            if env::var("PSModulePath").is_ok() {
                ShellType::PowerShell
            } else {
                ShellType::Cmd
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            env::var("SHELL")
                .ok()
                .and_then(|s| s.split('/').last())
                .and_then(|s| s.parse().ok())
                .unwrap_or(ShellType::Bash)
        }
    }
}
```

**Rationale**:
- Clear separation enables focused testing per shell family
- Each validator handles family-specific edge cases
- Easy to extend with new shell types
- Pattern matching clarity (no nested conditionals)

**Trade-offs**:
- Code duplication between validators
- Must maintain two validation paths
- Accepted: Clarity and testability justify duplication

## Summary of Decisions

| Decision | Technology | Rationale |
|----------|-----------|-----------|
| **Async Traits** | async-trait crate | Industry standard, zero-cost, excellent error handling |
| **Regex Compilation** | once_cell::Lazy | 5-15x performance gain, thread-safe, minimal memory cost |
| **CLI Structure** | Clap derive macros | Type-safe, compile-time validation, auto-generated help |
| **Error Handling** | thiserror (lib) + anyhow (binary) | Type safety in libs, ergonomics in binary |
| **Path Validation** | Separate POSIX/Windows validators | Clear separation, focused testing, extensible |

## Implementation Dependencies

Based on research decisions, the following crates are required:

```toml
[dependencies]
# Core framework
clap = { version = "4.4", features = ["derive", "env"] }
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Async support
async-trait = "0.1"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Pattern matching
regex = "1.10"
once_cell = "1.19"

# Output formatting
colored = "2.1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
tokio-test = "0.4"
proptest = "1.4"
criterion = "0.5"
```

## Next Steps

1. Create `data-model.md` with entity definitions based on research decisions
2. Create `quickstart.md` with validation workflow examples
3. Update `CLAUDE.md` with implementation guidelines
4. Generate `tasks.md` with concrete implementation tasks (50 tasks estimated)
5. Begin GREEN phase implementation: T001 (Define ShellType enum)

---
*Research complete - Ready for Phase 1 design artifacts*
