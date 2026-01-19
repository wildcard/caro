# Caro Test Suite

This directory contains tests for the Caro installation scripts, knowledge integration, and feature-specific backends.

## Running Tests

### Setup Script Tests

Test the `setup.sh` installation script:

```bash
./tests/test_setup.sh
```

This test suite validates:

- ✅ Script syntax validation
- ✅ Zsh shell detection and alias setup
- ✅ Bash shell detection (.bashrc and .bash_profile)
- ✅ Fish shell detection with directory creation
- ✅ Subprocess shell detection (e.g., `bash <(curl ...)` with zsh user)
- ✅ Duplicate alias prevention
- ✅ ZDOTDIR support for zsh
- ✅ Fallback to version variables when $SHELL is unknown
- ✅ Graceful handling of completely unknown shells

### Knowledge Integration Tests

Test the embedded knowledge index using LanceDB (default backend):

```bash
# Run all knowledge integration tests
cargo test --features knowledge --test knowledge_integration

# Run a specific test
cargo test --features knowledge --test knowledge_integration test_lancedb_health
```

These tests validate:
- ✅ LanceDB backend health and connectivity
- ✅ Recording successful commands with context
- ✅ Recording corrections with feedback
- ✅ Finding similar commands via vector search
- ✅ Database statistics and entry counts
- ✅ Clearing and persistence across backend instances
- ✅ Context metadata preservation

**Requirements:**
- `knowledge` feature flag enabled
- No external services required (uses temporary directory)
- Models downloaded automatically on first run

### ChromaDB Integration Tests

Test the ChromaDB backend (requires running server):

```bash
# Start ChromaDB server using Docker
cd tests && docker-compose up -d

# Run all ChromaDB integration tests
cargo test --features chromadb --test chromadb_integration -- --ignored --nocapture --test-threads=1

# Run a specific test
cargo test --features chromadb --test chromadb_integration test_chromadb_connection -- --ignored

# Stop ChromaDB server
cd tests && docker-compose down
```

**IMPORTANT:** ChromaDB tests must run serially (`--test-threads=1`) because they share the same collection name and interfere when run in parallel. See issue #537 for work to enable parallel execution.

These tests validate:
- ✅ ChromaDB server connection and health
- ✅ Recording successful commands with context
- ✅ Recording corrections with feedback
- ✅ Finding similar commands via vector search
- ✅ Collection statistics and entry counts
- ✅ Clearing collections
- ✅ Context metadata preservation

**Requirements:**
- `chromadb` feature flag enabled
- ChromaDB server running on localhost:8000 (or set `CHROMADB_URL`)
- Optional: `CHROMADB_AUTH_TOKEN` environment variable for auth

## Test Coverage

### `test_setup.sh`

**Tests the setup.sh script:**

| Test Case | Description |
|-----------|-------------|
| Full script syntax validation | Validates bash syntax with `bash -n` |
| Zsh detection | Tests zsh detection via $SHELL and creates .zshrc |
| Bash with .bashrc | Tests bash detection when .bashrc exists |
| Bash with .bash_profile | Tests bash detection when only .bash_profile exists |
| Bash without config | Tests bash detection and .bashrc creation |
| Fish detection | Tests fish detection and config directory creation |
| Subprocess bash detection | Tests that script detects user's actual shell (zsh) even when run with `bash <(curl ...)` |
| Duplicate alias prevention | Verifies alias isn't added multiple times |
| ZDOTDIR support | Tests custom zsh config directory via $ZDOTDIR |
| Unknown shell fallback | Tests fallback to version variables |
| Completely unknown shell | Tests graceful failure with manual instructions |

## Test Infrastructure

### Isolated Test Environments

Each test runs in an isolated environment:
- Temporary HOME directory created with `mktemp -d`
- Mock cargo installation
- Clean environment variables
- Automatic cleanup after each test

### Test Helpers

```bash
setup_test_env()      # Create isolated test environment
cleanup_test_env()    # Clean up temporary files and env vars
assert_file_exists()  # Assert a file exists
assert_file_contains()  # Assert a file contains a pattern
test_pass()           # Mark test as passing
test_fail()           # Mark test as failing
```

## Adding New Tests

To add a new test to `test_setup.sh`:

1. Create a test function following the naming convention `test_*`:

```bash
test_my_new_feature() {
    test_start "My new feature description"
    setup_test_env
    extract_setup_alias

    # Test logic here
    export SHELL="/bin/zsh"
    source "$TEST_TMPDIR/setup_alias_test.sh"
    setup_alias > /dev/null 2>&1

    # Assertions
    assert_file_exists "$HOME/.zshrc"
    assert_file_contains "$HOME/.zshrc" "alias caro='caro'"

    cleanup_test_env
}
```

2. Add the test function call to `main()`:

```bash
main() {
    # ... existing tests ...
    test_my_new_feature
    # ...
}
```

## CI Integration

These tests are integrated into the CI/CD pipeline:

### Setup Script Tests
```yaml
# Example GitHub Actions workflow
- name: Run setup script tests
  run: ./tests/test_setup.sh
```

### Knowledge Integration Tests
```yaml
# Runs in main test job with knowledge feature
- name: Run knowledge integration tests
  run: cargo test --features knowledge --test knowledge_integration --verbose
```

### ChromaDB Integration Tests
```yaml
# Runs in dedicated chromadb-integration job with ChromaDB service
services:
  chromadb:
    image: chromadb/chroma:0.5.18
    ports:
      - 8000:8000

steps:
  - name: Run ChromaDB integration tests
    run: cargo test --features chromadb --test chromadb_integration -- --ignored --nocapture --test-threads=1
```

See `.github/workflows/ci.yml` for the complete CI configuration.

All test scripts exit with code 0 on success and 1 on failure, making them suitable for CI systems.

## Troubleshooting

### Tests fail with "unbound variable" errors

Make sure the test script uses `set -u` (undefined variable checking) and all variable references use `${VAR:-}` syntax for optional variables.

### Cleanup issues

The cleanup function removes temporary directories and unsets environment variables. If tests interfere with each other, check that `cleanup_test_env()` is called at the end of each test.

### Shell detection issues

When testing shell detection, remember:
- Set `$SHELL` AFTER sourcing the test script to avoid bash resetting `BASH_VERSION`
- Use `unset BASH_VERSION` when testing non-bash shells
- Use `${ZSH_VERSION:-}` syntax for safe checking of potentially unset variables
