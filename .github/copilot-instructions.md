# Copilot Instructions for caro

This file provides instructions for GitHub Copilot when reviewing code and generating suggestions for this repository.

## Project Overview

`caro` is a single-binary Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. The tool prioritizes safety, performance, and developer experience with Apple Silicon optimization via MLX framework.

## Core Principles

### Safety-First Development
- All generated shell commands MUST be validated for dangerous patterns before execution
- NEVER approve code that bypasses safety validation in the `src/safety/` module
- Flag any command patterns that could cause data loss, privilege escalation, or system damage
- The safety module uses 52+ pre-compiled regex patterns to detect Critical, High, and Moderate risks

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
4. **Memory safety**: Unnecessary allocations, lifetime issues

### Medium Priority
1. **API consistency**: Public APIs must have rustdoc comments
2. **Test coverage**: New features should include unit and integration tests
3. **POSIX compliance**: Shell commands must use standard utilities
4. **Cross-platform compatibility**: Code should work on macOS, Linux, and Windows

### Lower Priority
1. **Code style**: Follow Rust conventions (use `cargo fmt` and `cargo clippy`)
2. **Documentation**: Update relevant docs when behavior changes
3. **Performance optimizations**: Only suggest if there's a measurable impact

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
```

## Testing Standards

### Required Checks
- `cargo fmt --all --check` - Code formatting
- `cargo clippy -- -D warnings` - Linter with warnings as errors
- `cargo test` - All tests must pass
- `cargo audit` - No known security vulnerabilities

### Test Patterns
- Use `#[tokio::test]` for async tests
- Safety-critical code requires property-based tests with `proptest`
- Integration tests should use mock backends when testing non-LLM code

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

## Pull Request Guidelines

When reviewing PRs:
1. Verify safety-critical changes have comprehensive tests
2. Check that error handling follows project standards
3. Ensure documentation is updated for API changes
4. Validate that new dependencies are justified and audited
5. Confirm CI passes before approval

## Language and Communication

- Provide specific, actionable feedback
- Reference line numbers and file paths when commenting
- Prioritize security and correctness over style
- Be concise - avoid lengthy explanations for obvious issues
