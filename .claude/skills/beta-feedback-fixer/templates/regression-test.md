# Regression Test Template

Use this template when creating regression tests for fixed issues.

---

## Test File Location

**Primary location**: `tests/beta_regression.rs`

**Alternative locations** (if applicable):
- Integration tests: `tests/integration/{module}_test.rs`
- Unit tests: Within the module being fixed (for pure function bugs)

---

## Test Structure

### Basic Template

```rust
/// Issue #{ID} Regression Test: {Brief Title}
///
/// Root cause: {One-line explanation of what was broken}
/// Fix: {One-line explanation of what was changed}
///
/// This test ensures that {describe what behavior is validated}.
#[tokio::test]
async fn test_issue_{id}_{brief_descriptive_name}() {
    // ============================================
    // Setup: Create test environment
    // ============================================

    // TODO: Create necessary test fixtures
    // Example: temp directories, config files, mock services

    // ============================================
    // Execute: Perform the operation that was broken
    // ============================================

    // TODO: Run the code that previously exhibited the bug

    // ============================================
    // Assert: Verify expected behavior
    // ============================================

    // TODO: Check that the bug is fixed

    // ============================================
    // Cleanup (if needed)
    // ============================================

    // TODO: Clean up resources (temp files, etc.)
}
```

---

## Common Patterns

### Pattern 1: Config Persistence Test

Tests that verify config is properly saved to disk.

```rust
/// Issue #{ID}: Config persistence
#[tokio::test]
async fn test_issue_{id}_config_persists() {
    // Setup: Create temp config file
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Execute: Modify config and save
    let mut config = Config::default();
    config.setting = new_value;

    let manager = ConfigManager::with_path(&config_path);
    manager.save(&config).expect("Failed to save config");

    // Assert: Load config, verify persistence
    let loaded_config = manager.load().expect("Failed to load config");
    assert_eq!(loaded_config.setting, new_value);

    // Cleanup: temp_dir drops automatically
}
```

### Pattern 2: Command Generation Test

Tests that verify correct shell commands are generated.

```rust
/// Issue #{ID}: Command generation accuracy
#[tokio::test]
async fn test_issue_{id}_correct_command_generated() {
    // Setup: Create static matcher
    let matcher = StaticMatcher::new();

    // Execute: Generate command for specific query
    let query = "show disk space by directory";
    let result = matcher.generate(query);

    // Assert: Verify correct command
    assert!(result.is_ok());
    let command = result.unwrap().command;
    assert_eq!(command, "du -h -d 1");

    // Also test that similar queries work
    let alt_query = "disk space per folder";
    let alt_result = matcher.generate(alt_query);
    assert!(alt_result.is_ok());
}
```

### Pattern 3: Output Format Test

Tests that verify no interactive prompts pollute machine-readable output.

```rust
/// Issue #{ID}: JSON output format
#[tokio::test]
async fn test_issue_{id}_json_output_clean() {
    // Setup: Create test CLI args with JSON output
    let args = CliArgs {
        query: "list files".to_string(),
        output: Some("json".to_string()),
        ..Default::default()
    };

    // Execute: Run command (capture stdout)
    let output = run_with_args(args);

    // Assert: Output is valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output)
        .expect("Output should be valid JSON");

    // Verify no prompt text in output
    assert!(!output.contains("Telemetry"));
    assert!(!output.contains("📊"));

    // Verify structure
    assert!(parsed["command"].is_string());
}
```

### Pattern 4: Interactive Prompt Test

Tests that verify prompts appear correctly in interactive mode.

```rust
/// Issue #{ID}: Prompt behavior
#[tokio::test]
async fn test_issue_{id}_prompt_shows_once() {
    // Setup: Fresh config (first run)
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Execute: Simulate first run (with mock prompt)
    let mut config = Config::default();
    config.telemetry.first_run = true;

    // Simulate user accepting prompt
    config.telemetry.enabled = true;
    config.telemetry.first_run = false;

    let manager = ConfigManager::with_path(&config_path);
    manager.save(&config).expect("Save should succeed");

    // Assert: Second run should NOT show prompt
    let loaded = manager.load().expect("Load should succeed");
    assert_eq!(loaded.telemetry.first_run, false);
    assert_eq!(loaded.telemetry.enabled, true);
}
```

### Pattern 5: Edge Case Test

Tests that verify edge cases are handled correctly.

```rust
/// Issue #{ID}: Edge case handling
#[tokio::test]
async fn test_issue_{id}_handles_edge_case() {
    // Setup: Create edge case conditions
    let edge_case_input = /* specific edge case */;

    // Execute: Run with edge case
    let result = function_under_test(edge_case_input);

    // Assert: Proper handling (not panic, correct result)
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, expected_for_edge_case);
}
```

---

## Test Checklist

Before considering test complete:

- [ ] Test has descriptive name: `test_issue_{id}_{what_it_tests}`
- [ ] Test has doc comment explaining root cause and fix
- [ ] Setup section clearly creates test environment
- [ ] Execute section performs the operation that was broken
- [ ] Assert section verifies the fix works
- [ ] Cleanup section releases resources (if needed)
- [ ] Test runs successfully: `cargo test test_issue_{id}`
- [ ] Test fails when fix is reverted (validates test catches bug)
- [ ] Test covers the main bug and at least 1 edge case
- [ ] Test is isolated (doesn't depend on other tests)
- [ ] Test is deterministic (always same result)

---

## Running Tests

### Run single test
```bash
cargo test test_issue_{id}_{name}
```

### Run all regression tests
```bash
cargo test --test beta_regression
```

### Run with output
```bash
cargo test test_issue_{id} -- --nocapture
```

### Run with logging
```bash
RUST_LOG=debug cargo test test_issue_{id} -- --nocapture
```

---

## Test Naming Convention

Pattern: `test_issue_{issue_id}_{brief_description}`

**Good names**:
- `test_issue_402_telemetry_persistence`
- `test_issue_404_json_output_clean`
- `test_issue_406_disk_space_command`

**Bad names** (too vague):
- `test_telemetry`
- `test_bug_402`
- `test_fix`

---

## Documentation in Tests

Always include doc comments:

```rust
/// Issue #{ID} Regression Test: {Title}
///
/// **Root cause**: {What was broken}
///
/// **Fix**: {What was changed}
///
/// **Validates**: {What this test ensures}
///
/// **Reproduction**:
/// 1. {Step to reproduce original bug}
/// 2. {Another step}
///
/// **Expected behavior**: {What should happen now}
```

---

## Common Mistakes

### ❌ Don't: Test too much

```rust
// Bad: Tests multiple unrelated things
#[test]
fn test_everything() {
    test_config();
    test_commands();
    test_output();
    // Too broad!
}
```

### ✅ Do: Test one thing

```rust
// Good: Focused on one issue
#[test]
fn test_issue_402_config_persistence() {
    // Only tests config persistence
}
```

### ❌ Don't: Depend on external state

```rust
// Bad: Depends on ~/.config/caro existing
#[test]
fn test_config() {
    let config = load_config(); // Uses real config!
}
```

### ✅ Do: Create isolated test environment

```rust
// Good: Creates temp directory
#[test]
fn test_config() {
    let temp = TempDir::new().unwrap();
    let config = load_config_from(temp.path());
}
```

### ❌ Don't: Skip cleanup

```rust
// Bad: Leaves files around
#[test]
fn test_file_creation() {
    create_file("/tmp/test.txt");
    // Never deleted!
}
```

### ✅ Do: Use RAII or explicit cleanup

```rust
// Good: TempDir cleans up automatically
#[test]
fn test_file_creation() {
    let temp = TempDir::new().unwrap();
    create_file(temp.path().join("test.txt"));
    // Automatically cleaned up when temp drops
}
```

---

## Integration with CI

Tests should:
- Run in CI on every PR
- Be fast (< 1 second per test ideally)
- Be deterministic (no flaky failures)
- Have no external dependencies (no network calls)

---

## References

- **Test Examples**: `tests/beta_regression.rs` (existing tests)
- **Testing Guide**: Rust Book Chapter 11
- **TempDir**: `tempfile` crate documentation
- **Async Tests**: `tokio::test` macro documentation
