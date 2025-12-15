# TDD Development Workflow for cmdai

This document describes the Test-Driven Development (TDD) workflow for the cmdai project, including test watch setup, development cycles, agent collaboration, and project-specific standards.

## Table of Contents

1. [Quick Start](#quick-start)
2. [The TDD Cycle](#the-tdd-cycle)
3. [Watch Management](#watch-management)
4. [Agent Collaboration](#agent-collaboration)
5. [Project-Specific Standards](#project-specific-standards)
6. [Common Workflows](#common-workflows)

---

## Quick Start

### Prerequisites

Before starting TDD development, ensure Rust and cargo are properly configured:

```bash
# Verify cargo is in PATH
which cargo

# If not found, load Rust environment
. "$HOME/.cargo/env"
```

### Installing cargo-watch

cargo-watch provides continuous test execution on file changes:

```bash
. "$HOME/.cargo/env" && cargo install cargo-watch
```

### Starting the Test Watch

Launch cargo-watch in the background for continuous feedback:

```bash
. "$HOME/.cargo/env" && cargo watch -x test
```

This command:
- Watches all Rust source files
- Runs `cargo test` on any change
- Provides immediate feedback on test status

### Checking Running Watches

To see all background shells (including test watchers):

```bash
/bashes
```

Or use the `BashOutput` tool with the shell ID to inspect specific output.

---

## The TDD Cycle

cmdai follows strict **Red-Green-Refactor** methodology aligned with spec-driven development.

### Phase 1: RED - Write Failing Contract Tests

**Goal**: Express desired behavior through a failing test before any implementation.

1. **Review the specification** from `specs/[feature-id]/spec.md`
2. **Identify the contract** from `specs/[feature-id]/contracts/`
3. **Write the test** in the appropriate `tests/` subdirectory:
   - `tests/contract/` - Module public API tests
   - `tests/integration/` - Cross-module workflow tests
   - `tests/property/` - Property-based invariant tests

**Example Contract Test** (from Feature 003):

```rust
// tests/cache_contract.rs
#[tokio::test]
async fn test_cache_manager_retrieves_model() {
    let cache = CacheManager::new().expect("cache creation failed");
    let result = cache.get_model("test-model").await;
    assert!(result.is_ok());
}
```

4. **Verify the test fails** by checking cargo-watch output
5. **Confirm failure reason** is correct (not a compilation error)

### Phase 2: GREEN - Implement Minimal Code

**Goal**: Make the test pass with the simplest possible implementation.

1. **Review the failing test** output from cargo-watch
2. **Identify the minimal change** needed
3. **Implement only what's required** to pass the test
4. **Observe cargo-watch** turn green
5. **Verify all tests still pass** (no regressions)

**Example Implementation**:

```rust
// src/cache/mod.rs
impl CacheManager {
    pub async fn get_model(&self, model_id: &str) -> Result<PathBuf, CacheError> {
        // Minimal implementation to pass test
        if self.is_cached(model_id) {
            // Return cached path
        } else {
            self.download_model(model_id).await
        }
    }
}
```

### Phase 3: REFACTOR - Improve Code Quality

**Goal**: Enhance code structure, readability, and performance while keeping tests green.

1. **Identify improvement opportunities**:
   - Extract duplicated code
   - Simplify complex logic
   - Improve naming
   - Add documentation
   - Optimize performance

2. **Make incremental changes** while watching tests
3. **Ensure tests stay green** after each refactor
4. **Document public APIs** with rustdoc comments

**Refactoring Principles**:
- Never refactor on red (always start with green tests)
- Make one change at a time
- Run tests after each change
- Prefer clarity over cleverness

---

## Watch Management

### Inspecting Test Output

**Using BashOutput tool** (when shell ID is known):

```bash
BashOutput with bash_id: 6d7f77
```

**Using /bashes command** (to see all shells):

```bash
/bashes
```

Output shows:
- Shell ID and command
- Current status (running/failed)
- Recent stdout/stderr
- Compilation errors
- Test failures with file:line references

### Understanding Watch Output

**Green (All tests passing)**:
```
[Running 'cargo test']
[Finished running. Exit status: 0]
```

**Red (Tests failing)**:
```
error[E0599]: no function or associated item named `new` found for struct `CacheManager`
  --> tests/cache_contract.rs:10:26
   |
10 |     let cache = CacheManager::new().expect("cache creation failed");
   |                              ^^^ function or associated item not found
```

**Warnings** (should be addressed but don't block tests):
```
warning: unused variable: `manifest`
   --> src/cache/manifest.rs:161:17
```

### Filtering Test Output

Run specific test suites:

```bash
# Contract tests only
cargo test --test cache_contract

# Integration tests only
cargo test --test integration

# Specific test by name
cargo test test_cache_manager_retrieves_model
```

### Stopping the Watch

To stop cargo-watch:

1. Find the shell ID: `/bashes`
2. Kill it: `KillShell` with the shell_id

Or manually:
```bash
kill $(pgrep -f "cargo watch")
```

---

## Agent Collaboration

cmdai uses specialized agents for TDD workflows. Choose the right agent based on your development phase.

### tdd-rust-watcher Agent

**When to use**:
- Active TDD development session
- Working through Red-Green-Refactor cycles
- Debugging failing tests
- Need real-time test feedback

**Key behaviors**:
- Maintains continuous test watch
- Guides through strict Red→Green→Refactor
- Provides minimal, incremental fixes
- Never runs ad-hoc `cargo test` commands

**Example invocation**:
```
User: "I need to implement cache validation"
Assistant: Uses tdd-rust-watcher agent to guide through:
1. Write failing test
2. Observe red output
3. Implement minimal fix
4. Verify green
5. Optional refactor
```

### tdd-rust-engineer Agent

**When to use**:
- Designing new features from scratch
- Implementing complete modules
- Need broader architectural guidance
- Starting new test suites

**Key behaviors**:
- Emphasizes contract-first design
- Focuses on library-first architecture
- Ensures comprehensive test coverage
- Applies Rust best practices

**Example invocation**:
```
User: "Implement the logging module"
Assistant: Uses tdd-rust-engineer agent to:
1. Review logging contract (specs/003/contracts/logging-api.md)
2. Design module structure
3. Write contract tests
4. Implement with TDD cycles
```

### Agent Coordination

For complex features, agents may work together:

1. **Planning phase**: `spec-driven-dev-guide` creates specification
2. **Architecture phase**: `rust-cli-architect` designs module structure
3. **Implementation phase**: `tdd-rust-engineer` writes contracts
4. **Development phase**: `tdd-rust-watcher` guides Red-Green-Refactor
5. **Quality phase**: `qa-testing-expert` validates coverage

---

## Project-Specific Standards

### Contract-First Testing

Based on cmdai's spec-driven development (see `specs/003-implement-core-infrastructure/plan.md`):

1. **Specifications drive contracts**: Each feature has a spec in `specs/[feature-id]/`
2. **Contracts define APIs**: Public APIs documented in `specs/[feature-id]/contracts/`
3. **Tests express contracts**: Contract tests in `tests/contract/` validate APIs
4. **Implementation satisfies tests**: Code in `src/` makes contract tests pass

**Test Organization**:
```
tests/
├── contract/           # Module public API tests (must align with contracts/)
├── integration/        # Cross-module scenarios (from quickstart.md)
└── property/           # Invariant validation (proptest-based)
```

### Library-First Architecture

All modules must be:
- **Exported via lib.rs**: Public API accessible programmatically
- **Self-contained**: Clear single responsibility
- **Reusable**: Usable beyond CLI context
- **Testable**: Contract tests validate public API

**Example lib.rs structure**:
```rust
// src/lib.rs
pub mod cache;
pub mod config;
pub mod execution;
pub mod logging;
pub mod models;
pub mod safety;
```

### Performance Validation

Tests must validate NFR requirements:

- **Startup time**: CLI initialization < 100ms
- **Validation time**: Safety checks < 50ms per command
- **Inference time**: MLX backend < 2s on M1 Mac
- **Cache operations**: Model retrieval < 5s for <1GB models

**Example performance test**:
```rust
#[tokio::test]
async fn test_safety_validation_performance() {
    let validator = SafetyValidator::new();
    let start = Instant::now();

    let result = validator.validate("ls -la").await;

    assert!(start.elapsed() < Duration::from_millis(50));
    assert!(result.is_ok());
}
```

### Cross-Platform Testing

Support required for:
- **Primary**: macOS, Linux
- **Secondary**: Windows

Tests should:
- Use `PathBuf` for cross-platform paths
- Detect shell type (`ShellType::Bash` vs `ShellType::Cmd`)
- Handle platform-specific dangerous patterns
- Use `tempfile` for filesystem tests

### Error Handling Standards

- **No panics in production**: Use `Result` types
- **User-friendly errors**: Clear messages with actionable context
- **Error chains**: Use `thiserror` for library errors, `anyhow` for binary
- **Graceful degradation**: Continue with defaults on non-critical failures

**Example error definition**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Failed to download model: {0}")]
    DownloadFailed(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
```

---

## Common Workflows

### Workflow 1: Fixing Compilation Errors

**Scenario**: cargo-watch shows compilation errors

1. **Read the error message** from watch output
2. **Identify the issue**:
   - Missing import?
   - Type mismatch?
   - Function signature change?
3. **Make minimal fix** to resolve compilation
4. **Verify tests run** (even if failing)
5. **Address test failures** using TDD cycle

**Example**:
```
error[E0061]: this function takes 1 argument but 0 arguments were supplied
   --> src/execution/shell.rs:122:21
    |
122 |         let shell = ShellDetector::detect();
    |                     ^^^^^^^^^^^^^^^^^^^^^-- argument #1 of type `&self` is missing
```

**Fix**: Change to instance method call:
```rust
let detector = ShellDetector::new();
let shell = detector.detect();
```

### Workflow 2: Addressing Failing Contract Tests

**Scenario**: Contract test fails after GREEN phase implementation

1. **Review test expectations** in `tests/contract/`
2. **Check implementation** in corresponding `src/` module
3. **Identify mismatch**:
   - Wrong return type?
   - Missing validation?
   - Incorrect error handling?
4. **Update implementation** to satisfy contract
5. **Verify all contract tests pass**

**Example from Feature 003**:
```rust
// Contract expects builder pattern
#[test]
fn test_config_builder() {
    let config = LogConfig::builder()
        .level(LogLevel::Debug)
        .output(LogOutput::Stdout)
        .build();
    assert_eq!(config.level, LogLevel::Debug);
}

// Implementation must provide builder
impl LogConfig {
    pub fn builder() -> LogConfigBuilder {
        LogConfigBuilder::default()
    }
}
```

### Workflow 3: Running Specific Test Suites

**Scenario**: Need to focus on particular module or feature

**Contract tests only**:
```bash
cargo test --test cache_contract
cargo test --test config_contract
cargo test --test execution_contract
cargo test --test logging_contract
```

**Integration tests only**:
```bash
cargo test --test infrastructure_integration
```

**Property tests only**:
```bash
cargo test --test property_tests
```

**Specific test by name**:
```bash
cargo test test_cache_manager_retrieves_model -- --show-output
```

**All tests with verbose output**:
```bash
cargo test -- --nocapture --test-threads=1
```

### Workflow 4: Integration Test Scenarios

**Scenario**: Validate cross-module workflows from quickstart.md

1. **Review scenario** in `specs/[feature-id]/quickstart.md`
2. **Write integration test** in `tests/integration/`
3. **Use multiple modules** together (cache + config + logging)
4. **Assert end-to-end behavior**
5. **Verify performance requirements**

**Example integration test**:
```rust
#[tokio::test]
async fn test_first_time_user_experience() {
    // Scenario 1 from quickstart.md
    let config = ConfigManager::new().expect("config init");
    let cache = CacheManager::new().expect("cache init");
    let context = ExecutionContext::capture().expect("context capture");

    // User runs cmdai for first time
    assert!(!config.file_exists());
    assert_eq!(cache.stats().total_models, 0);
    assert!(context.shell_type != ShellType::Unknown);
}
```

### Workflow 5: Performance Validation

**Scenario**: Ensure NFR requirements are met

1. **Identify performance target** from spec
2. **Add timing instrumentation** to test
3. **Run test multiple times** for consistency
4. **Profile if needed** with `cargo flamegraph` or `perf`
5. **Optimize hot paths** if below target

**Example**:
```rust
#[tokio::test]
async fn test_cli_startup_performance() {
    let iterations = 10;
    let mut timings = Vec::new();

    for _ in 0..iterations {
        let start = Instant::now();
        let _app = CliApp::new().expect("cli init");
        timings.push(start.elapsed());
    }

    let avg = timings.iter().sum::<Duration>() / iterations as u32;
    assert!(avg < Duration::from_millis(100), "Startup too slow: {:?}", avg);
}
```

---

## Additional Resources

- **Project guidance**: See `CLAUDE.md` for architecture and development commands
- **Repository guidelines**: See `AGENTS.md` for coding standards and commit conventions
- **Feature specifications**: See `specs/[feature-id]/spec.md` for requirements
- **API contracts**: See `specs/[feature-id]/contracts/` for expected behaviors
- **Agent documentation**: See `.claude/agents/` for specialized agent guidance

## Quick Reference

**Start watch**:
```bash
. "$HOME/.cargo/env" && cargo watch -x test
```

**Check watch status**:
```bash
/bashes
```

**Run specific tests**:
```bash
cargo test --test cache_contract
cargo test test_specific_function -- --show-output
```

**Format and lint**:
```bash
cargo fmt --all
cargo clippy -- -D warnings
```

**Full validation** (before PR):
```bash
cargo test && cargo fmt --check && cargo clippy -- -D warnings
```

---

**Last Updated**: 2025-10-03
**Related**: CLAUDE.md, AGENTS.md, specs/002-implement-tdd-green/spec.md, .claude/agents/tdd-rust-watcher.md
