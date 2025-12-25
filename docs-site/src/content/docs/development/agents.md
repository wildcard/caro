---
title: AI Agent Guidelines
description: Guidelines for AI agents working on the caro codebase
---

This document provides guidelines for AI agents (Claude Code, Cursor, Copilot, etc.) working on the caro codebase.

## Code Style

### Rust Conventions

- Follow Rust 2021 edition idioms
- Use `rustfmt` formatting
- Pass all `clippy` lints
- Prefer `Result<T>` over panics
- Use meaningful variable names

### Documentation

- Add doc comments to all public APIs
- Use `///` for function documentation
- Include examples in doc comments
- Keep comments up-to-date with code

## Safety Requirements

### Critical Rules

1. **Never bypass safety validation** in production code
2. **Test dangerous patterns** only in controlled test environments
3. **Document all security-relevant changes** thoroughly
4. **Review generated commands** before suggesting execution

### Dangerous Patterns to Block

```rust
// These patterns MUST be detected and blocked:
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf ~",
    "mkfs",
    "dd if=/dev/zero",
    ":(){ :|:& };:",  // Fork bomb
    "chmod 777 /",
    "> /dev/sda",
];
```

## Testing Requirements

### Test Coverage

- All new features require tests
- Bug fixes must include regression tests
- Safety validation requires property tests
- Integration tests for CLI workflows

### Test Quality

- Tests should be deterministic
- Avoid flaky tests
- Mock external dependencies
- Use descriptive test names

## Git Workflow

### Commit Messages

```
feat: add new backend support for Ollama
fix: resolve memory leak in MLX initialization
docs: update installation guide for Windows
test: add integration tests for safety module
refactor: simplify backend trait implementation
```

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code improvements

## Performance Considerations

### Startup Time

- Target: < 100ms cold start
- Lazy load dependencies
- Avoid blocking I/O on startup

### Inference Time

- Target: < 2s on Apple Silicon
- Cache model weights
- Use streaming where beneficial

## Error Handling

### Do
```rust
fn process_command(cmd: &str) -> Result<String, Error> {
    let validated = validate(cmd)?;
    execute(validated)
}
```

### Don't
```rust
fn process_command(cmd: &str) -> String {
    let validated = validate(cmd).unwrap(); // NO!
    execute(validated).expect("should work")  // NO!
}
```

## Encoding Rules

### UTF-8 Compliance

- All source files must be UTF-8
- Use ASCII for identifiers
- Avoid smart quotes in strings
- Use standard ASCII punctuation

### Avoid

- Em-dashes (use `--` instead)
- Smart quotes (use `"` and `'`)
- Non-breaking spaces
- Invisible Unicode characters
