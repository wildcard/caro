# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cmdai` is a single-binary Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. The tool prioritizes safety, performance, and developer experience with Apple Silicon optimization via Candle Metal backend.

**Core Goals:**
- Single binary under 50MB (without embedded model)
- Startup time < 100ms, first inference < 2s on M1 Mac
- Safety-first approach with comprehensive command validation
- Extensible backend system (Candle Metal, vLLM, Ollama)
- Hugging Face model caching with offline capability

## Project Structure

```
cmdai/
├── src/
│   ├── main.rs              # CLI entry point with clap configuration
│   ├── backends/            # Inference backend implementations
│   │   ├── mod.rs          # Backend trait system
│   │   ├── embedded/       # Embedded model backends
│   │   │   ├── cpu.rs      # Candle CPU backend (cross-platform)
│   │   │   └── metal.rs    # Candle Metal backend (Apple Silicon)
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
- Candle Metal backend uses native Rust bindings for Apple Silicon GPU acceleration
- Unified codebase: same backend code for CPU and GPU, just different Device
- Built-in GGUF support via candle-transformers
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

### Phase 4: Candle Metal Integration
- Metal GPU acceleration via candle-core Metal backend
- GGUF quantized model loading
- Device auto-detection with CPU fallback
- Apple Silicon performance optimization

## Key Dependencies

**Core:**
- `clap` - Command-line argument parsing
- `serde` + `serde_json` - JSON serialization
- `tokio` - Async runtime
- `anyhow` - Error handling
- `reqwest` - HTTP client for remote backends

**Inference:**
- `candle-core` - PyTorch-like tensor operations (with Metal support)
- `candle-transformers` - Pre-built LLM architectures and GGUF support
- `tokenizers` - HuggingFace tokenizer library

**Platform-Specific:**
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
- Candle Metal backend: < 2s on Apple Silicon (M4 Max target)
- Candle CPU backend: < 5s on modern hardware
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

## Apple Silicon Optimization

cmdai uses **Candle with Metal backend** for GPU-accelerated inference on Apple Silicon Macs.

### Why Candle over MLX?

Based on extensive research and benchmarks (see MLX_BACKEND_IMPLEMENTATION_PLAN.md):

**Performance**: Benchmarks on M1 MacBook (16GB RAM) show:
- Candle: FASTEST for Mistral-7B Q4 inference
- llama.cpp: Second
- mlx-rs: Third (slowest)

**Maturity**:
- Candle: Production-ready, 2500+ commits, used by HuggingFace
- mlx-rs: v0.25.1, active development, "may cause segfaults" warning

**Technical Advantages**:
- ✅ Simpler API (PyTorch-like)
- ✅ Built-in GGUF support
- ✅ Unified CPU/GPU codebase
- ✅ Already integrated as project dependency

### Building with Metal Support

```bash
# Apple Silicon (M1/M2/M3/M4)
cargo build --release --features embedded-metal

# Cross-platform CPU fallback
cargo build --release --features embedded-cpu

# Both backends
cargo build --release --features embedded-metal,embedded-cpu
```

### Performance Targets (M4 Max)

| Metric | Target | Implementation |
|--------|--------|----------------|
| Model loading | < 100ms | Lazy loading, memory-mapped files |
| First inference | < 2s | Metal GPU acceleration |
| Throughput | > 15 tok/s | Optimized Candle kernels |
| Memory usage | < 2GB | Quantized Q4_K_M model |
| Binary size | < 50MB | Optimized release profile |

### Device Selection

Candle automatically selects the best available device:

```rust
// On Apple Silicon with Metal support
let device = Device::new_metal(0)?;  // GPU acceleration

// Fallback to CPU if Metal unavailable
let device = Device::Cpu;  // Cross-platform compatibility
```

The same inference code works on both devices - just swap the Device instance.

## Specialized Agent Usage

When working on specific components:

- **Complex architecture changes**: Use `rust-cli-architect` agent
- **LLM integration & backends**: Use `llm-integration-expert` agent
- **Candle/Apple Silicon features**: Use `macos-unix-systems-expert` agent
- **Test-driven development**: Use `tdd-rust-engineer` agent
- **Documentation updates**: Use `technical-writer` agent

## Quality Standards

- All public APIs must have documentation
- Comprehensive error handling with helpful messages
- No panics in production code - use `Result` types
- Memory safety without unnecessary allocations
- Security-first approach for system-level operations
- POSIX compliance for maximum portability

## Multi-Agent Development Process

This project follows spec-driven development with coordinated multi-agent teams:
1. Specification phase with clear requirements
2. Architecture and design review
3. Phased implementation with safety validation
4. Quality assurance and documentation

Each phase includes specific agent coordination for optimal development flow and maintains alignment with project constitution and safety standards.
