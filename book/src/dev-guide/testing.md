# Testing Strategy

cmdai uses a comprehensive testing approach with multiple test layers to ensure reliability and safety.

## Test Pyramid

```
       ┌──────────────┐
       │  E2E Tests   │ ← Full workflow, few tests
       └──────────────┘
      ┌────────────────┐
      │  Integration   │ ← Component interaction
      └────────────────┘
    ┌──────────────────┐
    │   Unit Tests     │ ← Individual functions, many tests
    └──────────────────┘
```

## Test Organization

```
tests/
├── unit/                  # Unit tests (alongside source)
├── integration/           # Integration tests
│   ├── backend_tests.rs
│   ├── safety_tests.rs
│   └── cli_tests.rs
└── e2e/                  # End-to-end tests
    ├── command_generation.rs
    └── error_handling.rs
```

## Unit Tests

Unit tests are co-located with source code using `#[cfg(test)]` modules.

### Example: Safety Validator Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rm_rf_root() {
        let validator = SafetyValidator::new(SafetyConfig::strict());

        let result = validator.validate("rm -rf /");

        assert!(matches!(result.risk_level, RiskLevel::Critical));
        assert!(result.is_dangerous());
    }

    #[test]
    fn test_allow_safe_command() {
        let validator = SafetyValidator::new(SafetyConfig::strict());

        let result = validator.validate("ls -la");

        assert!(matches!(result.risk_level, RiskLevel::Safe));
        assert!(!result.is_dangerous());
    }

    #[test]
    fn test_detect_fork_bomb() {
        let validator = SafetyValidator::new(SafetyConfig::strict());

        let result = validator.validate(":(){ :|:& };:");

        assert!(matches!(result.risk_level, RiskLevel::Critical));
        assert_eq!(result.matched_patterns.len(), 1);
    }
}
```

### Running Unit Tests

```bash
# Run all unit tests
cargo test

# Run specific test
cargo test test_detect_rm_rf_root

# Run with output
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test
```

## Integration Tests

Integration tests verify component interactions in the `tests/` directory.

### Example: Backend Contract Tests

```rust
// tests/integration/backend_contract.rs
use cmdai::backends::*;

#[tokio::test]
async fn test_ollama_backend_contract() {
    test_backend_contract(create_ollama_backend()).await;
}

#[tokio::test]
async fn test_vllm_backend_contract() {
    test_backend_contract(create_vllm_backend()).await;
}

async fn test_backend_contract<T: CommandGenerator>(backend: T) {
    // Test 1: Basic generation
    let request = CommandRequest {
        prompt: "list files".to_string(),
        shell: Shell::Bash,
        ..Default::default()
    };

    let result = backend.generate_command(&request).await;
    assert!(result.is_ok());

    let cmd = result.unwrap();
    assert!(!cmd.command.is_empty());
    assert!(cmd.confidence >= 0.0 && cmd.confidence <= 1.0);

    // Test 2: Availability check
    let available = backend.is_available().await;
    assert!(available || !available); // Should not panic

    // Test 3: Backend info
    let info = backend.backend_info();
    assert!(!info.name.is_empty());
    assert!(!info.model.is_empty());
}
```

### Example: Safety Integration Tests

```rust
// tests/integration/safety_tests.rs

#[test]
fn test_safety_levels() {
    let dangerous_cmd = "rm -rf /tmp/*";

    // Strict mode should block
    let strict = SafetyValidator::new(SafetyConfig::strict());
    let result = strict.validate(dangerous_cmd);
    assert!(result.requires_confirmation());

    // Permissive mode should allow with warning
    let permissive = SafetyValidator::new(SafetyConfig::permissive());
    let result = permissive.validate(dangerous_cmd);
    assert!(result.is_allowed());
}

#[test]
fn test_custom_patterns() {
    let mut config = SafetyConfig::default();
    config.add_pattern(DangerousPattern {
        name: "docker_force",
        pattern: r"docker\s+rm\s+-f",
        severity: RiskLevel::High,
        message: "Force removing Docker containers",
    });

    let validator = SafetyValidator::new(config);
    let result = validator.validate("docker rm -f container");

    assert!(matches!(result.risk_level, RiskLevel::High));
}
```

### Running Integration Tests

```bash
# Run all integration tests
cargo test --test integration

# Run specific integration test
cargo test --test backend_contract

# Run with backend servers running
docker-compose up -d && cargo test --test integration
```

## End-to-End Tests

E2E tests verify complete workflows from CLI to execution.

### Example: Command Generation Flow

```rust
// tests/e2e/command_generation.rs

#[tokio::test]
async fn test_full_command_generation() {
    // Setup
    let config = Config::for_testing();
    let app = CmdAI::new(config);

    // Execute
    let result = app.generate("list all PDF files").await;

    // Verify
    assert!(result.is_ok());
    let cmd = result.unwrap();
    assert!(cmd.command.contains("find"));
    assert!(cmd.command.contains(".pdf"));
}

#[tokio::test]
async fn test_safety_blocking() {
    let config = Config::for_testing_strict();
    let app = CmdAI::new(config);

    let result = app.generate("delete everything").await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::Safety(SafetyError::CommandBlocked(_))
    ));
}
```

### Running E2E Tests

```bash
# Run E2E tests
cargo test --test e2e

# Run with real backends (requires setup)
CMDAI_USE_REAL_BACKENDS=1 cargo test --test e2e
```

## Property-Based Testing

Use `proptest` for property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_safety_never_allows_rm_rf_root(
        prefix in ".*",
        suffix in ".*"
    ) {
        let cmd = format!("{}rm -rf /{}", prefix, suffix);
        let validator = SafetyValidator::new(SafetyConfig::strict());
        let result = validator.validate(&cmd);

        prop_assert!(result.is_dangerous());
    }

    #[test]
    fn test_safe_commands_always_pass(
        cmd in prop::collection::vec("[a-z]+", 1..5)
    ) {
        let cmd_str = cmd.join(" ");
        let validator = SafetyValidator::new(SafetyConfig::strict());
        let result = validator.validate(&cmd_str);

        // Should not crash
        prop_assert!(result.risk_level >= RiskLevel::Safe);
    }
}
```

## Benchmark Tests

Use `criterion` for performance benchmarks:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_safety_validation(c: &mut Criterion) {
    let validator = SafetyValidator::new(SafetyConfig::strict());

    c.bench_function("validate_safe_command", |b| {
        b.iter(|| {
            validator.validate(black_box("ls -la"))
        })
    });

    c.bench_function("validate_dangerous_command", |b| {
        b.iter(|| {
            validator.validate(black_box("rm -rf /"))
        })
    });
}

criterion_group!(benches, bench_safety_validation);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench
```

## Test Utilities

### Test Fixtures

Create reusable test data:

```rust
// tests/common/mod.rs

pub fn sample_command_request() -> CommandRequest {
    CommandRequest {
        prompt: "list files".to_string(),
        shell: Shell::Bash,
        context: None,
        parameters: Default::default(),
    }
}

pub fn test_config() -> Config {
    Config {
        backend: BackendConfig::embedded(),
        safety: SafetyConfig::strict(),
        ..Default::default()
    }
}
```

### Mock Backends

```rust
pub struct MockBackend {
    responses: Vec<GeneratedCommand>,
}

#[async_trait]
impl CommandGenerator for MockBackend {
    async fn generate_command(&self, _: &CommandRequest)
        -> Result<GeneratedCommand>
    {
        Ok(self.responses[0].clone())
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            name: "mock".to_string(),
            version: "test".to_string(),
            model: "mock-model".to_string(),
            capabilities: Default::default(),
        }
    }
}
```

## Test Coverage

### Generating Coverage Reports

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html
```

### Coverage Goals

- **Unit tests**: > 80% line coverage
- **Integration tests**: All critical paths
- **E2E tests**: All user workflows

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check
```

## Testing Best Practices

### 1. Arrange-Act-Assert Pattern

```rust
#[test]
fn test_example() {
    // Arrange
    let validator = SafetyValidator::new(config);
    let command = "rm -rf /";

    // Act
    let result = validator.validate(command);

    // Assert
    assert!(result.is_dangerous());
}
```

### 2. Test Naming Convention

```rust
#[test]
fn test_{module}_{scenario}_{expected_result}() {
    // Test implementation
}

// Examples:
#[test]
fn test_safety_validator_rm_rf_root_blocks() { }

#[test]
fn test_ollama_backend_unavailable_returns_error() { }
```

### 3. Use Helper Functions

```rust
fn assert_command_is_safe(cmd: &str) {
    let result = validate_command(cmd);
    assert!(!result.is_dangerous());
    assert!(matches!(result.risk_level, RiskLevel::Safe));
}

#[test]
fn test_ls_command() {
    assert_command_is_safe("ls -la");
}
```

### 4. Test Error Cases

```rust
#[test]
fn test_invalid_backend_url() {
    let config = BackendConfig {
        base_url: "invalid://url".to_string(),
        ..Default::default()
    };

    let result = OllamaBackend::new(config);
    assert!(result.is_err());
}
```

### 5. Async Test Setup

```rust
#[tokio::test]
async fn test_async_operation() {
    let backend = setup_backend().await;
    let result = backend.generate_command(&request).await;
    assert!(result.is_ok());
}
```

## Test-Driven Development

See [TDD Workflow](./tdd-workflow.md) for our test-driven development process.

## Next Steps

- [TDD Workflow](./tdd-workflow.md) - Follow TDD practices
- [Contributing](./contributing.md) - Submit tests with your code
- [Architecture](./architecture.md) - Understand what to test
