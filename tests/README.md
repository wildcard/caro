# Caro Test Suite

This directory contains tests for the Caro installation and setup scripts.

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
    assert_file_contains "$HOME/.zshrc" "alias caro='cmdai'"

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

These tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run setup script tests
  run: ./tests/test_setup.sh
```

The test script exits with code 0 on success and 1 on failure, making it suitable for CI systems.

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
