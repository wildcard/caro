# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cmdai` is a single-binary Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. The tool prioritizes safety, performance, and developer experience with Apple Silicon optimization via MLX framework.

**Core Goals:**
- Single binary under 50MB (without embedded model)
- Startup time < 100ms, first inference < 2s on M1 Mac
- Safety-first approach with comprehensive command validation
- Extensible backend system (MLX, vLLM, Ollama)
- Hugging Face model caching with offline capability

## Project Structure

```
cmdai/
├── src/
│   ├── main.rs              # CLI entry point with clap configuration
│   ├── backends/            # Inference backend implementations
│   │   ├── mod.rs          # Backend trait system
│   │   ├── mlx.rs          # Apple Silicon MLX backend (FFI)
│   │   ├── vllm.rs         # vLLM HTTP API backend
│   │   └── ollama.rs       # Ollama local backend
│   ├── cache/              # Hugging Face model caching
│   ├── safety/             # Command validation and safety checks
│   └── config/             # Configuration management
├── tests/
│   ├── integration/        # End-to-end workflow tests
│   └── unit/              # Component-specific tests
└── .devcontainer/         # Development environment setup
```

## Architecture Overview

### Backend Trait System
All model backends implement `ModelBackend` trait:
- Async inference with `Result<String>` responses
- Availability checking with graceful fallbacks
- Unified configuration through `BackendConfig`
- JSON-only response parsing with multiple fallback strategies

### Safety-First Design
Safety module provides:
- Pattern matching for dangerous commands (`rm -rf /`, `mkfs`, fork bombs)
- POSIX compliance validation
- Path quoting and validation
- Risk level assessment (Safe, Moderate, High, Critical)
- User confirmation workflows

### Platform Optimization
- MLX backend uses FFI with cxx crate for Apple Silicon
- Conditional compilation with feature flags
- Cross-platform cache directory management
- Shell-specific optimizations and detection

## Development Commands

> !IMPORTANT: 
> Before running `cargo` or any rust development command in the shell, check the the command is installed with `which` and inspect the `$PATH` for the relevant bin. 

> If it doesn't run `. "$HOME/.cargo/env"` in your shell before command execution

### Building & Testing
```bash
# Build the project
cargo build --release

# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo run -- "list all files"

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### Development Environment
```bash
# Start development container
devcontainer open .

# Watch for changes during development
cargo watch -x check -x test -x run
```

## Implementation Phases

### Phase 1: Core CLI Structure
- Command-line argument parsing with clap
- Mock inference backend for initial testing
- Basic safety validation implementation
- Configuration and cache directory setup

### Phase 2: Safety & Validation
- Comprehensive dangerous command patterns
- POSIX compliance checking
- User confirmation workflows
- Risk assessment and color-coded output

### Phase 3: Remote Backends
- vLLM HTTP API integration
- Ollama local API support
- Error handling and retry mechanisms
- Response format standardization

### Phase 4: MLX Integration
- FFI bindings using cxx crate
- Metal Performance Shaders integration
- Unified memory architecture handling
- Apple Silicon performance optimization

## Key Dependencies

**Core:**
- `clap` - Command-line argument parsing
- `serde` + `serde_json` - JSON serialization
- `tokio` - Async runtime
- `anyhow` - Error handling
- `reqwest` - HTTP client for remote backends

**Platform-Specific:**
- `cxx` - Safe C++ FFI for MLX integration
- `directories` - Cross-platform directory management
- `colored` - Terminal color output

**Development:**
- `tokio-test` - Async testing utilities
- `tempfile` - Temporary file creation for tests

## Safety Validation Patterns

### Dangerous Commands to Block
- Filesystem destruction: `rm -rf /`, `rm -rf ~`
- Disk operations: `mkfs`, `dd if=/dev/zero`
- Fork bombs: `:(){ :|:& };:`
- System path modification: Operations on `/bin`, `/usr`, `/etc`
- Privilege escalation: `sudo su`, `chmod 777 /`

### POSIX Compliance Requirements
- Use standard utilities (ls, find, grep, awk, sed, sort)
- Proper path quoting for spaces and special characters
- Avoid bash-specific features for maximum portability
- Validate command syntax before execution

## System Prompt Template

The tool uses a strict system prompt for JSON-only responses:
- Single command generation with safety constraints
- POSIX-compliant utilities only
- Proper file path quoting
- Destructive operation avoidance
- Clear JSON format: `{"cmd": "command_here"}`

## Performance Requirements

### Startup Optimization
- Lazy loading of all dependencies
- Efficient JSON parsing with fallback strategies
- Minimal memory allocations during initialization
- Cached model loading when available

### Inference Performance
- MLX backend: < 2s on Apple Silicon
- Remote backends: < 5s with network latency
- Streaming support where beneficial
- Memory-conscious resource management

## Testing Strategy

### Unit Tests
- Safety pattern validation
- Command parsing and validation
- Configuration management
- Cache directory operations

### Integration Tests
- End-to-end command generation workflows
- Backend communication and error handling
- Cross-platform compatibility
- Performance benchmarks

### Property Tests
- Safety validation with random inputs
- POSIX compliance checking
- Error recovery mechanisms

## Specialized Agent Usage

When working on specific components:

- **Complex architecture changes**: Use `rust-cli-architect` agent
- **LLM integration & backends**: Use `llm-integration-expert` agent  
- **MLX/Apple Silicon features**: Use `macos-unix-systems-expert` agent
- **Test-driven development**: Use `tdd-rust-engineer` agent
- **Documentation updates**: Use `technical-writer` agent

## Quality Standards

- All public APIs must have documentation
- Comprehensive error handling with helpful messages
- No panics in production code - use `Result` types
- Memory safety without unnecessary allocations
- Security-first approach for system-level operations
- POSIX compliance for maximum portability

## Spec-Driven Development Workflows

This project uses **dual spec-driven workflows** optimized for different feature sizes:

### Spec-Kitty Workflow (Rapid Development)

**Use for**: Small/medium features (< 2 weeks), bug fixes, enhancements, parallel development

**Location**: `kitty-specs/` (git worktrees)

**Commands**: `/spec-kitty.*` slash commands in `.claude/commands/`

**Workflow**:
1. `bin/sk-new-feature "description"` - Creates isolated worktree
2. `/spec-kitty.specify` - Create spec.md
3. `/spec-kitty.plan` - Create plan.md
4. `/spec-kitty.tasks` - Generate work packages
5. `/spec-kitty.implement` - Execute tasks
6. `/spec-kitty.accept` - Run acceptance checks
7. `/spec-kitty.merge` - Merge and cleanup worktree

**Benefits**:
- ✅ Parallel development (multiple features simultaneously)
- ✅ No branch switching overhead
- ✅ Real-time dashboard (http://127.0.0.1:9237)
- ✅ Automated task management
- ✅ Perfect for rapid iteration with tools like Charm.land Crush

**Dashboard**: `bin/sk-dashboard` to monitor all features

### Spec-Kit Workflow (Large Features)

**Use for**: Large features (> 2 weeks), major architecture changes, extensive research

**Location**: `specs/` (traditional directories)

**Commands**: Custom slash commands in `.codex/prompts/`

**Workflow**:
1. Manual directory creation in `specs/NNN-feature-name/`
2. Create spec.md, plan.md, tasks.md manually
3. Use `.specify/templates/` for structure
4. Follow `.specify/memory/constitution.md` principles
5. Standard git workflow on feature branches

**Benefits**:
- ✅ Better for complex, long-running features
- ✅ Explicit constitution-based governance
- ✅ Flexible structure for research-heavy work

### Decision Matrix: Which Workflow?

| Criteria | Spec-Kitty | Spec-Kit |
|----------|------------|----------|
| **Feature size** | < 2 weeks | > 2 weeks |
| **Complexity** | Low-Medium | High |
| **Parallel dev** | Multiple features at once | One at a time |
| **Research phase** | Light research | Extensive research |
| **Architecture** | Incremental changes | Major refactoring |
| **Examples** | Add caching, Fix bug, New API endpoint | MLX backend, Safety system, Multi-backend |

### When to Use Spec-Kitty

✅ **DO use spec-kitty when:**
- Adding a new feature that takes < 2 weeks
- Fixing a bug that requires changes across multiple files
- Building an enhancement to existing functionality
- Working on multiple features in parallel (e.g., with Charm.land Crush)
- You want visual tracking via the dashboard
- The feature has clear, well-defined scope

**Example scenarios**:
- "Add Redis caching with TTL support"
- "Fix memory leak in MLX initialization"
- "Add Prometheus metrics endpoint"
- "Implement command history feature"
- "Add JSON output format option"

### When to Use Spec-Kit

✅ **DO use spec-kit when:**
- Building a major new system (> 2 weeks)
- Extensive research or prototyping needed
- Architectural decisions require deep investigation
- Multiple competing approaches need evaluation
- Long-running feature with many unknowns

**Example scenarios**:
- "Implement complete MLX backend with C++ FFI"
- "Design and build multi-backend inference system"
- "Create comprehensive safety validation framework"
- "Research and implement model quantization pipeline"
- "Architect distributed caching system"

### Both Workflows Coexist

The project supports **both workflows simultaneously**:
- `kitty-specs/` for rapid, parallel development
- `specs/` for large, complex features

**Example**: You can work on a large MLX backend feature in `specs/004-implement-ollama-and/` while simultaneously using spec-kitty for quick bug fixes in `kitty-specs/001-fix-memory-leak/`.

### Integration Points

**Shared resources**:
- Both follow the same constitution principles (`.specify/memory/constitution.md`)
- Both use the same testing standards
- Both require security-first approach
- Both commit to the same git repository

**Different tools**:
- Spec-kitty: Automated task management, worktrees, dashboard
- Spec-kit: Manual planning, traditional branches, constitution-driven

### Quick Reference

```bash
# Spec-Kitty workflow
bin/sk-new-feature "Add caching"      # Create feature
cd kitty-specs/001-add-caching/       # Enter worktree
/spec-kitty.specify                   # Generate spec
/spec-kitty.implement                 # Execute tasks
bin/sk-dashboard                      # Monitor progress

# Spec-Kit workflow
mkdir -p specs/005-new-feature/       # Create directory
# Manually create spec.md, plan.md
# Use .specify/templates/ as reference
# Follow constitution-based development
```

See `docs/SPEC_KITTY_GUIDE.md` for comprehensive spec-kitty documentation.

## Multi-Agent Development Process

This project follows spec-driven development with coordinated multi-agent teams:
1. Specification phase with clear requirements
2. Architecture and design review
3. Phased implementation with safety validation
4. Quality assurance and documentation

Each phase includes specific agent coordination for optimal development flow and maintains alignment with project constitution and safety standards.
