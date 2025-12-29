---
title: Test-Driven Development
description: How caro uses TDD for reliable, well-tested code
---

This guide covers the Test-Driven Development (TDD) workflow used in caro development.

## TDD Principles

caro follows strict TDD principles:

1. **Red**: Write a failing test first
2. **Green**: Write minimal code to pass the test
3. **Refactor**: Clean up while keeping tests green

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in watch mode
cargo watch -x test
```

## Test Categories

### Unit Tests

Located alongside source code in `src/`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_validation() {
        let validator = SafetyValidator::new();
        assert!(validator.is_dangerous("rm -rf /"));
        assert!(!validator.is_dangerous("ls -la"));
    }
}
```

### Integration Tests

Located in `tests/`:

```bash
tests/
├── integration/
│   ├── cli_tests.rs      # CLI workflow tests
│   ├── backend_tests.rs  # Backend integration
│   └── safety_tests.rs   # Safety validation
└── common/
    └── mod.rs            # Shared test utilities
```

### Property Tests

Using `proptest` for fuzzing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_safety_with_random_input(s in ".*") {
        let validator = SafetyValidator::new();
        // Should not panic on any input
        let _ = validator.validate(&s);
    }
}
```

## Test Coverage

Run coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

## Writing Good Tests

### Test Naming

Use descriptive names:

```rust
#[test]
fn validates_rm_rf_root_as_critical_risk() { ... }

#[test]
fn allows_safe_file_listing_commands() { ... }

#[test]
fn rejects_fork_bomb_patterns() { ... }
```

### Arrange-Act-Assert

Structure tests clearly:

```rust
#[test]
fn test_command_generation() {
    // Arrange
    let backend = MockBackend::new();
    let prompt = "list files";

    // Act
    let result = backend.generate(prompt);

    // Assert
    assert!(result.is_ok());
    assert!(result.unwrap().contains("ls"));
}
```

## Mocking

Use trait objects for testability:

```rust
pub trait ModelBackend {
    fn generate(&self, prompt: &str) -> Result<String>;
}

struct MockBackend {
    response: String,
}

impl ModelBackend for MockBackend {
    fn generate(&self, _prompt: &str) -> Result<String> {
        Ok(self.response.clone())
    }
}
```

## CI Integration

Tests run automatically on every PR:

```yaml
# .github/workflows/test.yml
- name: Run tests
  run: cargo test --all-features

- name: Run clippy
  run: cargo clippy -- -D warnings

- name: Check formatting
  run: cargo fmt --check
```
