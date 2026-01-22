# CLAUDE.md - caro Project Guide

> This file provides Claude Code and AI assistants with essential context for working on the caro project.

## Project Overview

**caro** is a Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. It features embedded model inference (MLX for Apple Silicon, Candle for CPU), comprehensive safety validation, and cross-platform support.

**Repository**: https://github.com/wildcard/caro
**Website**: https://caro.sh
**Current Version**: 1.0.4 (published on crates.io)

## Tech Stack

### Backend (Rust)
- **Runtime**: Rust 1.83+, tokio async runtime
- **CLI Framework**: clap with derive macros
- **Inference**: MLX (Apple Silicon), Candle (CPU), llama.cpp bindings
- **HTTP Client**: reqwest (optional, for remote backends)
- **Error Handling**: anyhow + thiserror
- **Logging**: tracing + tracing-subscriber

### Model Integration
- **Primary Model**: Qwen2.5-Coder-1.5B-Instruct (quantized)
- **Model Hub**: Hugging Face Hub (hf-hub crate)
- **Backends**: Embedded MLX, Embedded CPU, Ollama, vLLM

### Testing & Quality
- **Testing**: cargo test, proptest (property-based), serial_test
- **Benchmarking**: criterion
- **Linting**: clippy with strict warnings
- **Formatting**: rustfmt

## Project Structure

```
caro/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                  # CLI interface and argument parsing
│   ├── backends/             # LLM backend implementations
│   │   ├── mod.rs           # CommandGenerator trait
│   │   ├── embedded.rs      # Embedded model backend (MLX/CPU)
│   │   ├── ollama.rs        # Ollama local backend
│   │   └── vllm.rs          # vLLM remote backend
│   ├── safety/              # Command validation (52+ patterns)
│   ├── config/              # TOML configuration management
│   ├── execution/           # Shell command execution
│   └── models/              # Data structures
├── tests/                    # Integration and contract tests
├── benches/                  # Performance benchmarks
├── .claude/                  # Claude Code configuration
│   ├── agents/              # Specialized agent definitions
│   ├── commands/            # Slash commands (/caro.*, /spec-kitty.*)
│   ├── skills/              # Interactive skills
│   ├── memory/              # Project knowledge base
│   ├── reference/           # Best practices documentation
│   └── hooks/               # Session and workflow hooks
├── kitty-specs/             # Spec-Kitty feature specifications
├── website/                 # caro.sh Astro website (if present)
└── landing/                 # Landing page assets
```

## Essential Commands

### Development
```bash
# Build (debug)
cargo build

# Build (release with optimizations)
cargo build --release

# Build with MLX support (Apple Silicon)
cargo build --release --features embedded-mlx

# Run the CLI
cargo run -- "your prompt here"
cargo run --quiet -- list files    # Unquoted prompt support

# Run with debug logging
RUST_LOG=debug cargo run -- "your prompt"
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run property-based tests
cargo test proptest

# Run benchmarks
cargo bench
```

### Quality Checks
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings

# Run all checks (format, lint, test)
make check

# Security audit
cargo deny check
```

### Makefile Shortcuts
```bash
make build          # cargo build
make build-release  # cargo build --release
make test           # cargo test
make fmt            # cargo fmt
make lint           # cargo clippy
make check          # fmt + lint + test
```

## Slash Commands

The project uses Spec-Kitty for feature development:

| Command | Purpose |
|---------|---------|
| `/caro.feature` | Unified feature workflow orchestrator |
| `/caro.qa #123` | Investigate GitHub issues |
| `/caro.roadmap` | Project management and work selection |
| `/caro.release.*` | Release workflow commands |
| `/spec-kitty.specify` | Create feature specification |
| `/spec-kitty.plan` | Generate implementation plan |
| `/spec-kitty.tasks` | Generate work packages |
| `/spec-kitty.implement` | Execute implementation |
| `/spec-kitty.review` | Code review workflow |

## Reference Documentation

See `.claude/reference/` for best practices:
- `rust-cli-best-practices.md` - CLI patterns, clap usage, error handling
- `safety-patterns.md` - Command validation patterns
- `testing-patterns.md` - Rust testing conventions

## Development Standards

### Code Conventions

**Rust Style:**
- Use `#[derive(...)]` for common traits
- Prefer `Result<T, E>` over panics
- Use `thiserror` for custom error types
- Document public APIs with `///` comments
- Follow clippy suggestions strictly

**Error Handling:**
```rust
// Good: Specific error types with context
Err(BackendError::ModelNotFound { path: model_path.clone() })

// Avoid: Generic errors without context
Err(anyhow!("failed"))
```

**Async Patterns:**
```rust
#[async_trait]
trait CommandGenerator {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand, GeneratorError>;
}
```

### Safety Validation

The `SafetyValidator` module checks commands against 52+ dangerous patterns:
- System destruction (`rm -rf /`, `rm -rf ~`)
- Fork bombs (`:(){:|:&};:`)
- Disk operations (`mkfs`, `dd if=/dev/zero`)
- Privilege escalation (`sudo su`, `chmod 777 /`)
- Critical path access (`/bin`, `/usr`, `/etc`)

**Risk Levels:**
- `Safe` (green) - Normal operations
- `Moderate` (yellow) - Requires confirmation in strict mode
- `High` (orange) - Requires confirmation in moderate mode
- `Critical` (red) - Blocked in strict mode

### Testing Approach

**Test Organization:**
```
tests/
├── cli_args_test.rs        # CLI argument parsing
├── safety_validation_test.rs # Safety pattern tests
├── backend_contract_test.rs  # Trait compliance
└── regression_issue_*.rs    # Bug regression tests
```

**Test Naming:**
```rust
#[test]
fn test_<module>_<behavior>_<scenario>() { }

// Example:
#[test]
fn test_safety_blocks_rm_rf_root() { }
```

### Git Workflow

**Branch Naming:**
- Features: `claude/<feature-name>-<session-id>`
- Releases: `release/v1.x.x`

**Commit Messages:**
```
feat: Add new safety pattern for rm -rf *
fix: Handle empty model response gracefully
docs: Update installation instructions
test: Add regression test for issue #123
refactor: Extract validation logic to separate module
```

## Configuration

### User Config Location
`~/.config/caro/config.toml`

```toml
[backend]
primary = "embedded"
enable_fallback = true

[safety]
enabled = true
level = "moderate"

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"
```

### Project Config Files
| File | Purpose |
|------|---------|
| `Cargo.toml` | Rust dependencies and build config |
| `clippy.toml` | Linter rules |
| `rustfmt.toml` | Code formatting |
| `deny.toml` | Dependency audit |

## Canonical Rules

### Installation Command
**ALWAYS use:**
```bash
bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
```

**NEVER use pipe-to-shell patterns** (`curl | bash`)

### Links
See `.claude/memory/consolidated-knowledge-rules.md` for full constitution.

## Agent System

Specialized agents in `.claude/agents/`:
- `rust-cli-expert.md` - Core Rust CLI development
- `qa-testing-expert.md` - Testing and quality assurance
- `llm-integration-expert.md` - Model backend development
- `safety-validation-expert.md` - Safety pattern development
- `docs-release-manager.md` - Documentation and releases

## Current Focus

Check `ROADMAP.md` for current priorities. Key areas:
- Performance optimization
- Extended test coverage
- Model caching improvements

## Quick Reference

| Action | Command |
|--------|---------|
| Run caro | `cargo run -- "your prompt"` |
| Run tests | `cargo test` |
| Format code | `cargo fmt` |
| Check lint | `cargo clippy -- -D warnings` |
| Build release | `cargo build --release` |
| See feature status | `/caro.feature status` |
| Investigate issue | `/caro.qa #123` |
