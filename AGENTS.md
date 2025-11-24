# Agent Guidelines for cmdai

## Project Overview

**cmdai** is a safety-first Rust CLI tool that converts natural language to POSIX shell commands using local LLMs. Built for blazing-fast performance (<100ms startup, <2s inference on M1 Mac), single-binary distribution (<50MB), and comprehensive safety validation.

**Core Technologies**: Rust 2021 Edition, async with Tokio, clap CLI, MLX for Apple Silicon, Candle for cross-platform CPU inference.

**License**: AGPL-3.0 (network use requires source disclosure).

---

## Essential Commands

### Build & Run
```bash
# Build debug binary
make build
cargo build

# Build optimized release binary
make release
cargo build --release

# Run with debug logging
RUST_LOG=debug cargo run -- "list all files"
make run-debug

# Install locally
make install
cargo install --path .

# Check binary size (must be <50MB)
make size-check
```

### Testing (Critical - TDD Project)
```bash
# Run all tests (default - quiet mode)
make test
RUST_LOG=warn cargo test -q --all-features

# Run specific test suites
make test-contract          # Contract tests only
make test-integration       # Integration tests only
make test-property          # Property-based tests only

# Verbose test output
make test-verbose
RUST_LOG=debug cargo test --verbose --all-features

# Show stdout/stderr from tests
make test-show-output

# Use nextest (if installed)
make test-nextest

# Watch mode for continuous testing
make test-watch
cargo watch -x "test -q --all-features"
```

### Code Quality (Pre-PR Requirements)
```bash
# Format code (MUST run before commit)
make fmt
cargo fmt --all

# Check formatting without changing files
make fmt-check
cargo fmt --all -- --check

# Run Clippy linter (warnings treated as errors)
make lint
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
make audit
cargo audit

# Run all quality checks + tests
make check
```

### Performance & Documentation
```bash
# Run Criterion benchmarks
make bench
cargo bench

# Generate and open documentation
make doc
cargo doc --no-deps --open

# Profile release build
make profile
```

---

## Project Structure

```
cmdai/
├── src/
│   ├── lib.rs              # Library exports (library-first architecture)
│   ├── main.rs             # CLI entry point (orchestration only, no business logic)
│   ├── models/             # Core data types (no dependencies)
│   ├── safety/             # Command validation (depends only on models)
│   │   ├── mod.rs          # SafetyValidator with 52 pre-compiled patterns
│   │   └── patterns.rs     # Dangerous command pattern database
│   ├── backends/           # LLM backend implementations
│   │   ├── mod.rs          # CommandGenerator trait
│   │   ├── embedded/       # MLX (Apple Silicon) and CPU (Candle) backends
│   │   └── remote/         # Ollama and vLLM backends (feature-gated)
│   ├── cache/              # Model caching with integrity validation
│   ├── config/             # Configuration management (TOML)
│   ├── cli/                # CLI interface and argument parsing
│   ├── execution/          # Shell detection and execution context
│   ├── logging/            # Structured logging with tracing
│   ├── platform/           # Platform detection utilities
│   └── model_loader.rs     # Hugging Face model loading
├── tests/                  # Contract-first test structure
│   ├── *_contract.rs       # Contract tests (API boundaries)
│   ├── *_integration.rs    # Integration tests (workflows)
│   ├── property_tests.rs   # Property-based tests
│   ├── e2e_cli_tests.rs    # E2E user scenarios
│   └── quickstart_scenarios.rs  # Scenario walk-throughs
├── benches/                # Criterion performance benchmarks
├── specs/                  # Product & architecture specifications
├── .specify/memory/        # Project constitution and checklists
├── .github/workflows/      # CI/CD (multi-platform, security audit)
└── exports/                # Generated artifacts for reviews
```

---

## Constitution & Core Principles

### I. Simplicity (MANDATORY)
- **Library-first architecture**: All features in `lib.rs`, `main.rs` orchestrates only
- **Direct framework usage**: No wrapper abstractions around clap, tokio, serde
- **Single data flow**: `CommandRequest → GeneratedCommand → ValidationResult`
- **YAGNI**: No organizational-only patterns (repositories, DTOs, UoW)

### II. Test-First (NON-NEGOTIABLE)
- **RED-GREEN-REFACTOR cycle strictly enforced**
- **No implementation without failing test first** - violations block code review
- **Test ordering**: Contract tests → Integration tests → Implementation → Unit tests
- **Real dependencies in tests** - no mocking unless testing error conditions
- **Git commit granularity**: Tests must be committed before implementation

### III. Safety-First Development
- **Dangerous command detection mandatory** before execution
- **52 pre-compiled regex patterns** covering Critical/High/Moderate risks
- **Risk levels**: Safe (green) → Moderate (yellow) → High (orange) → Critical (red)
- **User confirmation required** for High/Critical operations
- **No unsafe Rust** without explicit justification
- **Validation performance**: <50ms at P95

### IV. Implementation Order (STRICT)
1. **Models first** (`src/models/mod.rs`) - Foundation, no dependencies
2. **Safety second** (`src/safety/`) - Depends only on models
3. **Backends third** (`src/backends/`) - Depends on models
4. **CLI last** (`src/cli/`) - Orchestrates all modules

### V. Observability
- **Structured logging**: Use `tracing` crate with appropriate levels
- **Error handling**: `anyhow` for binaries, `thiserror` for libraries
- **User-facing messages**: Clear, actionable, distinct from debug logs
- **Performance metrics**: Log startup time, validation latency, inference duration

---

## Coding Conventions

### Formatting (rustfmt.toml)
```
max_width = 100
tab_spaces = 4 (no hard tabs)
newline_style = Unix
edition = 2021
reorder_imports = true
use_try_shorthand = true
```

**Before committing**: Always run `cargo fmt --all` or `make fmt`.

### Naming Conventions
- **Types**: UpperCamelCase (`CommandRequest`, `SafetyValidator`)
- **Functions/modules**: snake_case (`generate_command`, `safety/patterns.rs`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_COMMAND_LENGTH`)
- **Enum variants**: UpperCamelCase with descriptive names (`RiskLevel::Critical`)

### Linting (Clippy)
- **Warnings treated as errors**: `cargo clippy -- -D warnings`
- **Cognitive complexity threshold**: 25
- **Enum variant name threshold**: 3
- **Allow attributes**: Local with explanation comment only

### Error Handling Patterns
```rust
// Libraries: Use thiserror for typed errors
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("Backend unavailable: {reason}")]
    BackendUnavailable { reason: String },
    // ...
}

// Binaries: Use anyhow for context chains
use anyhow::{Context, Result};
let config = ConfigManager::load()
    .context("Failed to load configuration")?;
```

### Async Patterns
```rust
// Use async-trait for trait methods
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    async fn generate_command(&self, request: &CommandRequest) 
        -> Result<GeneratedCommand, GeneratorError>;
}

// Tokio runtime in main.rs
#[tokio::main]
async fn main() { /* ... */ }
```

---

## Testing Strategy

### Test-Driven Development Workflow
1. **RED**: Write test, verify it fails (`cargo test --test <test_file>`)
2. **GREEN**: Add minimal code to make test pass (no extra features)
3. **REFACTOR**: Improve code quality while keeping tests green
4. **COMMIT**: Granular commits after each test passes
5. **REPEAT**: Move to next failing test

### Test Types & Priority
1. **Contract tests** (`*_contract.rs`) - API boundaries, trait compliance
2. **Integration tests** (`*_integration.rs`) - Module interactions, workflows
3. **E2E tests** (`e2e_*.rs`) - User scenarios, CLI interface
4. **Property tests** (`property_tests.rs`) - Safety validation with random inputs
5. **Unit tests** (in modules) - Edge cases, error conditions

### Test Environment
```bash
# Set log levels to reduce noise
RUST_LOG=warn cargo test -q

# For debugging specific tests
RUST_LOG=debug cargo test test_name -- --nocapture

# Use nextest for cleaner output
cargo nextest run --all-features
```

### Test Organization
```
tests/
├── backend_trait_contract.rs       # CommandGenerator trait
├── safety_validator_contract.rs    # SafetyValidator API
├── cli_interface_contract.rs       # CLI argument parsing
├── config_contract.rs              # Configuration management
├── cache_contract.rs               # Cache integrity
├── execution_contract.rs           # Shell detection
├── integration_tests.rs            # End-to-end workflows
├── embedded_integration.rs         # Embedded backend tests
├── remote_integration.rs           # Remote backend tests (feature-gated)
├── property_tests.rs               # Property-based testing
├── e2e_cli_tests.rs               # CLI user scenarios
└── quickstart_scenarios.rs        # Scenario walk-throughs
```

### Testing Best Practices
- **Real dependencies**: Don't mock unless testing error conditions
- **Quiet by default**: Use `RUST_LOG=warn` and `-q` flag
- **Document scenarios**: Add new scenarios to `specs/` when extending features
- **Integration required for**: New libraries, contract changes, shared schemas
- **Run full suite before PR**: `make check` (fmt + lint + audit + test)

---

## Feature Flags & Cross-Platform

### Feature Flags (Cargo.toml)
```toml
[features]
default = ["embedded-cpu"]           # CPU inference via Candle
mock-backend = []                    # Mock backend for testing
remote-backends = ["reqwest"]        # Ollama + vLLM support
embedded-mlx = ["cxx", "mlx-rs"]    # Apple Silicon MLX (macOS aarch64)
embedded-cpu = ["candle-core"]       # Cross-platform CPU inference
full = ["remote-backends", "embedded-mlx", "embedded-cpu"]
```

### Build Commands by Feature
```bash
# Default build (CPU inference only)
cargo build --release

# With remote backends
cargo build --release --features remote-backends

# Apple Silicon optimized (macOS only)
cargo build --release --features embedded-mlx,embedded-cpu

# All features
cargo build --release --all-features
```

### Platform-Specific Code
```rust
// Conditional compilation for Apple Silicon
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub use backends::embedded::MlxBackend;

// Feature-gated modules
#[cfg(feature = "remote-backends")]
pub mod remote;
```

### CI/CD Matrix
- **Platforms**: Ubuntu, macOS (Intel + Silicon), Windows
- **Targets**: linux-amd64, linux-arm64, macos-intel, macos-silicon, windows-amd64
- **Tests**: All platforms run lib tests, Ubuntu runs full feature matrix
- **Checks**: Formatting, Clippy, security audit, binary size (<50MB)

---

## Important Gotchas & Patterns

### 1. Rust Environment Setup
**CRITICAL**: Before running any `cargo` command, verify Rust is in PATH:
```bash
which cargo
# If not found, load Rust environment:
. "$HOME/.cargo/env"
```

### 2. Safety Pattern Compilation
- **Patterns compiled once at startup** using `once_cell::Lazy` (30x speedup)
- **52 pre-compiled regex patterns** in `src/safety/patterns.rs`
- **Context-aware matching**: Distinguishes dangerous commands from safe string literals
  - `rm -rf /` → dangerous (executable context)
  - `echo 'rm -rf /' > script.sh` → safe (in quotes)

### 3. Test Failure Messages
- **"old_string not found"** in edit: Check exact whitespace/indentation
- **Async test timeout**: Increase timeout in `.config/nextest.toml`
- **Test output noise**: Use `RUST_LOG=warn` or `RUST_LOG=error`

### 4. Performance Requirements
- **Startup time**: <100ms (target)
- **First inference**: <2s on M1 Mac
- **Safety validation**: <50ms at P95
- **Binary size**: <50MB (use `make size-check`)

### 5. Configuration Files
- **User config**: `~/.config/cmdai/config.toml`
- **Backend config**: Supports embedded (default), Ollama, vLLM
- **Safety config**: Customizable patterns, safety levels (strict/moderate/permissive)

### 6. Library-First Architecture
- **All business logic in `src/lib.rs` exports**
- **`main.rs` orchestrates only** - no business logic
- **Public APIs must have rustdoc comments**
- **Each module has single, well-defined purpose**

### 7. Error Handling
- **Libraries**: Use `thiserror` for typed errors with `#[error("...")]`
- **Binaries**: Use `anyhow` with `.context("...")` chains
- **No panics in production** - use `Result` types
- **User-facing messages**: Helpful, actionable, distinct from debug logs

### 8. Dependency Management
- **`cargo audit`**: Run before merging PRs
- **`deny.toml`**: Gating rules for licenses and vulnerabilities
- **Allowed licenses**: MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-DFS-2016, CC0-1.0
- **Vulnerability policy**: `deny`, unmaintained crates `warn`

### 9. Git Commit Conventions
- **Concise, Title Case subjects** (<72 chars)
- **Imperative mood** ("Add feature" not "Added feature")
- **Optional leading emoji** (🎉, 🐛, 📚, ✨, etc.)
- **PR references**: `(#12)` at end of subject
- **Squash WIP commits** before requesting review

### 10. TDD Enforcement
- **Pre-commit hooks** verify test-first discipline
- **PR reviews validate** TDD workflow
- **Failing to follow TDD** results in rejected changes
- **Git commits must show** tests before implementation

---

## Development Workflow

### Starting a New Feature
1. **Read specs**: Check `specs/` for architecture contracts
2. **Write contract test**: Define API in `tests/*_contract.rs`
3. **Verify RED**: Run `cargo test --test <test_file>` - must fail
4. **Minimal implementation**: Add just enough code to pass
5. **Verify GREEN**: Run tests again - must pass
6. **Refactor**: Improve code quality, keep tests green
7. **Commit**: Granular commit after each test passes
8. **Repeat**: Next failing test

### Pre-PR Checklist
```bash
# 1. Format code
make fmt

# 2. Run linter
make lint

# 3. Security audit
make audit

# 4. Run full test suite
make test

# 5. Check binary size
make size-check

# Or run all at once:
make check
```

### PR Guidelines
- **Describe intent**: What problem does this solve?
- **List touched modules**: Which files/modules changed?
- **Link to specs/issues**: Reference relevant documentation
- **Include transcripts**: Screenshots or command output for user-facing changes
- **Squash WIP commits**: Clean commit history
- **Follow commit conventions**: Title Case, imperative mood, <72 chars

---

## Backend System

### CommandGenerator Trait
All backends implement this async trait:
```rust
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    async fn generate_command(&self, request: &CommandRequest) 
        -> Result<GeneratedCommand, GeneratorError>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
    async fn shutdown(&self) -> Result<(), GeneratorError>;
}
```

### Available Backends
1. **Embedded CPU (default)**: Candle framework, cross-platform
2. **Embedded MLX**: Apple Silicon optimized (macOS aarch64)
3. **Ollama**: Local API backend (feature: `remote-backends`)
4. **vLLM**: Remote HTTP API (feature: `remote-backends`)

### Backend Configuration (`~/.config/cmdai/config.toml`)
```toml
[backend]
primary = "embedded"  # or "ollama", "vllm"
enable_fallback = true

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"

[backend.vllm]
base_url = "http://localhost:8000"
model_name = "codellama/CodeLlama-7b-hf"
```

---

## Safety Validation

### Risk Levels
- **Safe (Green)**: Normal operations, no confirmation
- **Moderate (Yellow)**: Confirmation in strict mode
- **High (Orange)**: Confirmation in moderate mode
- **Critical (Red)**: Blocked in strict mode, explicit confirmation required

### Blocked Operations (Critical)
- Filesystem destruction: `rm -rf /`, `rm -rf ~`, `mkfs`
- Fork bombs: `:(){ :|:& };:`
- Device writes: `dd if=/dev/zero`
- System path modifications: `/bin`, `/usr`, `/etc`
- Privilege escalation: `sudo su`, `chmod 777 /`

### Safety Configuration (`~/.config/cmdai/config.toml`)
```toml
[safety]
enabled = true
level = "moderate"  # strict, moderate, or permissive
require_confirmation = true
custom_patterns = ["additional", "dangerous", "patterns"]
```

### Validation Pipeline
1. **Pattern Matching**: Check against 52 pre-compiled regex patterns
2. **POSIX Compliance**: Validate shell syntax and quoting
3. **Path Validation**: Prevent injection, verify quote escaping
4. **Risk Assessment**: Assign Safe/Moderate/High/Critical level
5. **User Confirmation**: Require explicit approval for High/Critical

---

## Common Tasks

### Adding a New Safety Pattern
1. Add pattern to `src/safety/patterns.rs`
2. Add test to `tests/safety_validator_contract.rs`
3. Verify pattern compiles: `cargo test test_pattern_compilation`
4. Test dangerous command detection
5. Document in specs and update CHANGELOG.md

### Implementing a New Backend
1. Add contract test to `tests/backend_trait_contract.rs`
2. Implement `CommandGenerator` trait in `src/backends/<name>.rs`
3. Add backend-specific configuration to `src/config/`
4. Test availability checking and error handling
5. Add integration test in `tests/<name>_integration.rs`
6. Document configuration in README.md

### Adding a CLI Flag
1. Add field to `Cli` struct in `src/main.rs`
2. Implement `IntoCliArgs` method
3. Add contract test to `tests/cli_interface_contract.rs`
4. Add E2E test to `tests/e2e_cli_tests.rs`
5. Update README.md usage section

### Debugging Test Failures
```bash
# Run single test with output
RUST_LOG=debug cargo test test_name -- --nocapture

# Run with backtraces
RUST_BACKTRACE=1 cargo test test_name

# Use nextest with verbose profile
cargo nextest run -P verbose test_name

# Check test without running
cargo test --no-run --verbose
```

---

## Resources

### Key Files
- **Constitution**: `.specify/memory/constitution.md` - Core principles (MUST READ)
- **Contributing**: `CONTRIBUTING.md` - Development guidelines
- **Claude Guide**: `CLAUDE.md` - AI assistant context
- **Specifications**: `specs/` - Architecture contracts and design docs
- **Changelog**: `CHANGELOG.md` - Version history and breaking changes

### Documentation
- **API Docs**: `cargo doc --no-deps --open`
- **README**: Project overview, quick start, usage examples
- **Code of Conduct**: `CODE_OF_CONDUCT.md`

### CI/CD
- **CI Workflow**: `.github/workflows/ci.yml` - Multi-platform tests, security audit
- **Release Workflow**: `.github/workflows/release.yml` - Binary distribution
- **Issue Templates**: `.github/ISSUE_TEMPLATE/` - Bug reports, feature requests

### External Resources
- [MLX Framework](https://github.com/ml-explore/mlx) - Apple Silicon ML
- [vLLM](https://github.com/vllm-project/vllm) - High-performance LLM serving
- [Ollama](https://ollama.ai) - Local LLM runtime
- [Hugging Face](https://huggingface.co) - Model hosting

---

## Quick Reference Card

### Most Common Commands
```bash
# Development cycle
make build          # Build debug
make test           # Run all tests quietly
make fmt            # Format code
make lint           # Run Clippy

# Pre-commit
make check          # Format + lint + audit + test

# Release
make release        # Build optimized binary
make size-check     # Verify <50MB
make install        # Install locally

# Debugging
RUST_LOG=debug cargo run -- "prompt"
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Test Quick Reference
```bash
make test           # All tests (quiet)
make test-contract  # Contract tests
make test-integration  # Integration tests
make test-verbose   # Verbose output
make test-nextest   # Use nextest if available
```

### File Locations
- **User config**: `~/.config/cmdai/config.toml`
- **Cargo cache**: `~/.cargo/`
- **Build output**: `target/release/cmdai`
- **Test artifacts**: `target/nextest/`
- **Benchmark reports**: `target/criterion/`

---

**Last Updated**: 2025-01-24  
**Project Stage**: Active early development (core implementation in progress)  
**Constitution Version**: 1.0.0  
**Rust Edition**: 2021  
**MSRV**: 1.75+
