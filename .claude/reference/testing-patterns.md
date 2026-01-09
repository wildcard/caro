# Testing Patterns for Rust CLI Applications

> Reference guide for testing caro and Rust CLI applications.

## Test Organization

### Directory Structure
```
tests/
├── cli_args_test.rs          # CLI argument parsing tests
├── safety_validation_test.rs  # Safety pattern tests
├── backend_contract_test.rs   # Trait compliance tests
├── integration_test.rs        # Full workflow tests
├── regression_issue_*.rs      # Bug regression tests
└── common/
    └── mod.rs                 # Shared test utilities
```

### Test Module Organization
```rust
// In src/safety/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validates_safe_command() {
        // Unit test co-located with implementation
    }
}

// In tests/safety_validation_test.rs
// Integration tests in separate files
#[test]
fn test_end_to_end_validation() {
    // Full integration test
}
```

## Naming Conventions

### Test Function Names
```rust
// Pattern: test_<module>_<behavior>_<scenario>

#[test]
fn test_safety_blocks_rm_rf_root() { }

#[test]
fn test_safety_allows_normal_rm() { }

#[test]
fn test_cli_parses_unquoted_prompt() { }

#[test]
fn test_backend_handles_timeout_gracefully() { }
```

### Async Test Names
```rust
#[tokio::test]
async fn test_backend_generates_command_successfully() { }

#[tokio::test]
async fn test_backend_falls_back_on_failure() { }
```

## Test Categories

### Unit Tests (Fast, Isolated)
```rust
use super::*;

#[test]
fn test_parse_json_response() {
    let json = r#"{"command": "ls -la"}"#;
    let result = parse_response(json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().command, "ls -la");
}

#[test]
fn test_parse_json_handles_malformed() {
    let json = r#"{"broken"#;
    let result = parse_response(json);
    assert!(result.is_err());
}
```

### Integration Tests (Test Module Interactions)
```rust
// tests/integration_test.rs
use caro::{backend::EmbeddedBackend, safety::SafetyValidator};

#[tokio::test]
async fn test_full_generation_pipeline() {
    let backend = EmbeddedBackend::new().await.unwrap();
    let validator = SafetyValidator::new();

    let request = CommandRequest::new("list all files");
    let result = backend.generate_command(&request).await.unwrap();

    let validation = validator.validate(&result.command);
    assert!(matches!(validation, ValidationResult::Safe));
}
```

### Contract Tests (Trait Compliance)
```rust
// tests/backend_contract_test.rs
use caro::backend::CommandGenerator;

async fn verify_backend_contract<B: CommandGenerator>(backend: B) {
    // Test 1: Available check works
    let available = backend.is_available().await;
    assert!(available);

    // Test 2: Backend info is valid
    let info = backend.backend_info();
    assert!(!info.name.is_empty());

    // Test 3: Simple generation works
    let request = CommandRequest::new("echo hello");
    let result = backend.generate_command(&request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_embedded_backend_contract() {
    let backend = EmbeddedBackend::new().await.unwrap();
    verify_backend_contract(backend).await;
}
```

### Regression Tests (Bug Prevention)
```rust
// tests/regression_issue_161.rs
//! Regression tests for Issue #161: Fix list command argument parsing
//!
//! Issue: #161 - Unquoted arguments not being parsed correctly
//! Reporter: @user
//! Date Reported: 2024-12-15
//! QA Tested: 2024-12-20
//! Status: CANNOT_REPRODUCE (feature implemented correctly)

use clap::Parser;
use caro::cli::CliArgs;

#[test]
fn test_issue_161_basic_unquoted_prompt() {
    let args = CliArgs::parse_from(["caro", "list", "files"]);
    assert_eq!(args.prompt.join(" "), "list files");
}

#[test]
fn test_issue_161_unquoted_with_flags() {
    let args = CliArgs::parse_from([
        "caro", "--verbose", "list", "all", "files"
    ]);
    assert!(args.verbose);
    assert_eq!(args.prompt.join(" "), "list all files");
}
```

## Async Testing

### Using tokio::test
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}

// With custom runtime configuration
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_operations() {
    // Test with multiple threads
}
```

### Testing Timeouts
```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_operation_completes_in_time() {
    let result = timeout(
        Duration::from_secs(5),
        slow_operation()
    ).await;

    assert!(result.is_ok(), "Operation timed out");
}
```

### Testing Cancellation
```rust
use tokio::select;

#[tokio::test]
async fn test_cancellation_cleanup() {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let handle = tokio::spawn(async move {
        cancellable_operation(rx).await
    });

    // Cancel after brief delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    tx.send(()).unwrap();

    let result = handle.await.unwrap();
    assert!(result.is_cancelled());
}
```

## Property-Based Testing

### Using proptest
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_never_panics(s in ".*") {
        let _ = parse_command(&s);  // Should never panic
    }

    #[test]
    fn test_safe_commands_accepted(
        cmd in "[a-z]+",
        file in "[a-z]+\\.(txt|md|rs)"
    ) {
        let validator = SafetyValidator::new();
        let command = format!("{} {}", cmd, file);
        let result = validator.validate(&command);

        // Simple commands should generally be safe
        prop_assert!(!matches!(result, ValidationResult::Blocked { .. }));
    }
}
```

### Custom Strategies
```rust
use proptest::prelude::*;

fn safe_command_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("ls -la".to_string()),
        Just("pwd".to_string()),
        "[a-z]+ [a-z]+\\.txt".prop_map(|s| s),
    ]
}

proptest! {
    #[test]
    fn test_safe_commands(cmd in safe_command_strategy()) {
        let validator = SafetyValidator::new();
        assert!(matches!(validator.validate(&cmd), ValidationResult::Safe));
    }
}
```

## Test Fixtures and Utilities

### Shared Test Utilities
```rust
// tests/common/mod.rs
use caro::{config::Config, backend::MockBackend};
use tempfile::TempDir;

pub fn test_config() -> Config {
    Config {
        backend: BackendConfig {
            primary: "mock".into(),
            ..Default::default()
        },
        safety: SafetyConfig {
            level: SafetyLevel::Strict,
            ..Default::default()
        },
    }
}

pub fn temp_config_dir() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("caro")).unwrap();
    dir
}

pub fn mock_backend() -> MockBackend {
    MockBackend::new()
        .with_response("test", "echo hello")
}
```

### Using Fixtures
```rust
// tests/integration_test.rs
mod common;

#[test]
fn test_with_fixtures() {
    let config = common::test_config();
    let backend = common::mock_backend();

    // Use fixtures in test
}
```

## Serial Test Execution

### When Tests Need Isolation
```rust
use serial_test::serial;

#[test]
#[serial]
fn test_modifies_global_state() {
    // This test modifies environment variables
    std::env::set_var("CARO_SHELL", "zsh");

    // Test logic...

    std::env::remove_var("CARO_SHELL");
}

#[test]
#[serial]
fn test_also_uses_env() {
    // This test won't run concurrently with the above
}
```

### File System Tests
```rust
use tempfile::TempDir;

#[test]
fn test_config_file_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    std::fs::write(&config_path, r#"
        [backend]
        primary = "ollama"
    "#).unwrap();

    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.backend.primary, "ollama");
}
```

## Mocking and Stubs

### Mock Backend
```rust
pub struct MockBackend {
    responses: HashMap<String, String>,
    fail_after: Option<usize>,
    call_count: AtomicUsize,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            fail_after: None,
            call_count: AtomicUsize::new(0),
        }
    }

    pub fn with_response(mut self, prompt_contains: &str, response: &str) -> Self {
        self.responses.insert(prompt_contains.into(), response.into());
        self
    }

    pub fn fail_after(mut self, n: usize) -> Self {
        self.fail_after = Some(n);
        self
    }
}

#[async_trait]
impl CommandGenerator for MockBackend {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand, GeneratorError>
    {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);

        if let Some(n) = self.fail_after {
            if count >= n {
                return Err(GeneratorError::InferenceFailed {
                    message: "Mock failure".into(),
                });
            }
        }

        for (key, response) in &self.responses {
            if request.prompt.contains(key) {
                return Ok(GeneratedCommand {
                    command: response.clone(),
                    confidence: 0.95,
                });
            }
        }

        Ok(GeneratedCommand {
            command: "echo 'default'".into(),
            confidence: 0.5,
        })
    }
}
```

## Test Coverage

### Running Coverage
```bash
# Using cargo-tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html

# Using llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

### Coverage Targets
- **Unit tests**: > 80% line coverage
- **Safety patterns**: 100% pattern coverage
- **Public API**: 100% function coverage
- **Error paths**: All error variants tested

## CI Integration

### GitHub Actions Test Workflow
```yaml
# .github/workflows/test.yml
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable

    - name: Run tests
      run: cargo test --all-features

    - name: Run clippy
      run: cargo clippy -- -D warnings

    - name: Check formatting
      run: cargo fmt -- --check
```

## Best Practices Summary

1. **Name tests descriptively**: `test_<module>_<behavior>_<scenario>`
2. **Test edge cases**: Empty input, invalid input, boundary conditions
3. **Use property testing**: For input validation and parsing
4. **Isolate integration tests**: Use tempdir for file system tests
5. **Mock external dependencies**: Backends, network, file system
6. **Document regression tests**: Include issue reference and context
7. **Run tests in CI**: Every PR should pass all tests
8. **Measure coverage**: Track and maintain coverage targets
