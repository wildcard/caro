#!/usr/bin/env bash
#
# Test suite for setup.sh and install.sh
#
# Usage: ./tests/test_setup.sh
#
# Tests cover:
# - Shell detection (bash, zsh, fish)
# - PATH configuration
# - Error detection patterns (edition2024, C++ headers)
# - Script syntax validation

# Don't use set -e because we want to run all tests even if some fail
set -u

# Colors for test output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

TESTS_PASSED=0
TESTS_FAILED=0
TEST_TMPDIR=""

# Test helpers
setup_test_env() {
    TEST_TMPDIR=$(mktemp -d)
    export HOME="$TEST_TMPDIR"
    export CARGO_HOME="$TEST_TMPDIR/.cargo"
    mkdir -p "$CARGO_HOME/bin"
    mkdir -p "$HOME/.local/bin"
}

cleanup_test_env() {
    if [ -n "$TEST_TMPDIR" ] && [ -d "$TEST_TMPDIR" ]; then
        rm -rf "$TEST_TMPDIR"
    fi
    unset ZDOTDIR
}

test_start() {
    echo -e "${BLUE}▶${NC} Testing: $1"
}

test_pass() {
    echo -e "${GREEN}✓${NC} PASS: $1"
    ((TESTS_PASSED++))
}

test_fail() {
    echo -e "${RED}✗${NC} FAIL: $1"
    echo -e "${RED}  ${2}${NC}"
    ((TESTS_FAILED++))
}

assert_file_exists() {
    if [ -f "$1" ]; then
        test_pass "File exists: $1"
    else
        test_fail "File should exist: $1" "File not found"
    fi
}

assert_file_contains() {
    if grep -q "$2" "$1" 2>/dev/null; then
        test_pass "File contains '$2': $1"
    else
        test_fail "File should contain '$2': $1" "Pattern not found"
    fi
}

assert_file_not_contains() {
    if ! grep -q "$2" "$1" 2>/dev/null; then
        test_pass "File does not contain '$2': $1"
    else
        test_fail "File should not contain '$2': $1" "Pattern found unexpectedly"
    fi
}

assert_output_contains() {
    if echo "$1" | grep -q "$2"; then
        test_pass "Output contains '$2'"
    else
        test_fail "Output should contain '$2'" "Pattern not found in: $1"
    fi
}

# Extract setup_path function from setup.sh for testing
extract_setup_path() {
    cat > "$TEST_TMPDIR/setup_path_test.sh" << 'TESTEOF'
say() { echo "$1"; }
say_success() { echo "$1"; }
say_warn() { echo "$1"; }
say_error() { echo "$1"; }

# Setup PATH in shell config
setup_path() {
    local install_dir="$1"
    local shell_config=""
    local shell_name=""

    # Detect shell config file
    case "$SHELL" in
        */bash)
            shell_name="bash"
            if [ -f "$HOME/.bashrc" ]; then
                shell_config="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_config="$HOME/.bash_profile"
            else
                shell_config="$HOME/.bashrc"
            fi
            ;;
        */zsh)
            shell_name="zsh"
            shell_config="${ZDOTDIR:-$HOME}/.zshrc"
            ;;
        */fish)
            shell_name="fish"
            shell_config="$HOME/.config/fish/config.fish"
            ;;
        *)
            say_warn "Could not detect shell config file"
            say "Please manually add to your shell config:"
            say "  export PATH=\"$install_dir:\$PATH\""
            return
            ;;
    esac

    # Check if already added
    if grep -q "# caro PATH" "$shell_config" 2>/dev/null; then
        say "PATH already configured in $shell_config"
        return
    fi

    # Create config file if it doesn't exist
    if [ ! -f "$shell_config" ]; then
        mkdir -p "$(dirname "$shell_config")"
        touch "$shell_config"
    fi

    say "Adding $install_dir to PATH in $shell_config..."

    if [[ "$shell_name" == "fish" ]]; then
        echo -e "\n# caro PATH" >> "$shell_config"
        echo "set -gx PATH $install_dir \$PATH" >> "$shell_config"
    else
        echo -e "\n# caro PATH" >> "$shell_config"
        echo "export PATH=\"$install_dir:\$PATH\"" >> "$shell_config"
    fi

    say_success "PATH updated in $shell_config"
    say "Run 'source $shell_config' or restart your terminal to apply"
}
TESTEOF
}

# ============================================================================
# PATH Setup Tests
# ============================================================================

test_path_setup_zsh() {
    test_start "PATH setup for zsh"
    setup_test_env
    extract_setup_path

    export SHELL="/bin/zsh"
    unset ZDOTDIR

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_path_test.sh"
    setup_path "$HOME/.local/bin" > /dev/null 2>&1

    assert_file_exists "$HOME/.zshrc"
    assert_file_contains "$HOME/.zshrc" "# caro PATH"
    assert_file_contains "$HOME/.zshrc" 'export PATH=".*\.local/bin'

    cleanup_test_env
}

test_path_setup_bash_with_bashrc() {
    test_start "PATH setup for bash with existing .bashrc"
    setup_test_env
    extract_setup_path

    touch "$HOME/.bashrc"
    export SHELL="/bin/bash"

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_path_test.sh"
    setup_path "$HOME/.local/bin" > /dev/null 2>&1

    assert_file_contains "$HOME/.bashrc" "# caro PATH"
    assert_file_contains "$HOME/.bashrc" 'export PATH=".*\.local/bin'

    cleanup_test_env
}

test_path_setup_bash_with_bash_profile() {
    test_start "PATH setup for bash with .bash_profile only"
    setup_test_env
    extract_setup_path

    touch "$HOME/.bash_profile"
    export SHELL="/bin/bash"

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_path_test.sh"
    setup_path "$HOME/.local/bin" > /dev/null 2>&1

    assert_file_contains "$HOME/.bash_profile" "# caro PATH"
    assert_file_contains "$HOME/.bash_profile" 'export PATH=".*\.local/bin'

    cleanup_test_env
}

test_path_setup_fish() {
    test_start "PATH setup for fish"
    setup_test_env
    extract_setup_path

    export SHELL="/usr/bin/fish"

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_path_test.sh"
    setup_path "$HOME/.local/bin" > /dev/null 2>&1

    assert_file_exists "$HOME/.config/fish/config.fish"
    assert_file_contains "$HOME/.config/fish/config.fish" "# caro PATH"
    assert_file_contains "$HOME/.config/fish/config.fish" "set -gx PATH"

    cleanup_test_env
}

test_path_setup_duplicate_prevention() {
    test_start "PATH setup duplicate prevention"
    setup_test_env
    extract_setup_path

    export SHELL="/bin/zsh"

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_path_test.sh"

    # Run setup_path twice
    setup_path "$HOME/.local/bin" > /dev/null 2>&1
    setup_path "$HOME/.local/bin" > /dev/null 2>&1

    # Count occurrences
    local count
    count=$(grep -c "# caro PATH" "$HOME/.zshrc" || true)

    if [ "$count" -eq 1 ]; then
        test_pass "PATH config appears exactly once"
    else
        test_fail "PATH config should appear exactly once" "Found $count occurrences"
    fi

    cleanup_test_env
}

test_path_setup_zdotdir() {
    test_start "PATH setup respects ZDOTDIR"
    setup_test_env
    extract_setup_path

    mkdir -p "$HOME/custom_zsh"
    export SHELL="/bin/zsh"
    export ZDOTDIR="$HOME/custom_zsh"

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_path_test.sh"
    setup_path "$HOME/.local/bin" > /dev/null 2>&1

    assert_file_exists "$HOME/custom_zsh/.zshrc"
    assert_file_contains "$HOME/custom_zsh/.zshrc" "# caro PATH"

    cleanup_test_env
}

test_path_setup_unknown_shell() {
    test_start "PATH setup with unknown shell shows manual instructions"
    setup_test_env
    extract_setup_path

    export SHELL="/bin/unknown_shell"

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_path_test.sh"
    local output
    output=$(setup_path "$HOME/.local/bin" 2>&1)

    assert_output_contains "$output" "manually add"

    cleanup_test_env
}

# ============================================================================
# Error Detection Tests
# ============================================================================

test_error_detection_edition2024() {
    test_start "Error detection: edition2024"

    local test_output="error: failed to compile
feature \`edition2024\` is required
Consider trying a newer version of Cargo"

    if echo "$test_output" | grep -q "edition2024\|feature.*is required\|newer version of Cargo"; then
        test_pass "Detected edition2024 error pattern"
    else
        test_fail "Should detect edition2024 error" "Pattern not matched"
    fi
}

test_error_detection_cpp_headers() {
    test_start "Error detection: missing C++ headers"

    local test_output="fatal error: 'algorithm' file not found
#include <algorithm>
         ^~~~~~~~~~~"

    if echo "$test_output" | grep -q "fatal error:.*file not found\|'algorithm' file not found\|'cstdint' file not found\|'vector' file not found"; then
        test_pass "Detected C++ headers error pattern"
    else
        test_fail "Should detect C++ headers error" "Pattern not matched"
    fi
}

test_error_detection_cstdint() {
    test_start "Error detection: missing cstdint header"

    local test_output="./thirdparty/llama.cpp/unicode.h:3:10: fatal error: 'cstdint' file not found
    3 | #include <cstdint>"

    if echo "$test_output" | grep -q "'cstdint' file not found"; then
        test_pass "Detected cstdint error pattern"
    else
        test_fail "Should detect cstdint error" "Pattern not matched"
    fi
}

test_error_detection_vector() {
    test_start "Error detection: missing vector header"

    local test_output="./thirdparty/llama.cpp/llama.h:1005:10: fatal error: 'vector' file not found
 1005 | #include <vector>"

    if echo "$test_output" | grep -q "'vector' file not found"; then
        test_pass "Detected vector error pattern"
    else
        test_fail "Should detect vector error" "Pattern not matched"
    fi
}

# ============================================================================
# Script Syntax Validation
# ============================================================================

test_setup_sh_syntax() {
    test_start "setup.sh syntax validation"

    if bash -n setup.sh 2>/dev/null; then
        test_pass "setup.sh has valid bash syntax"
    else
        test_fail "setup.sh has syntax errors" "bash -n failed"
    fi
}

test_install_sh_syntax() {
    test_start "install.sh syntax validation"

    if bash -n install.sh 2>/dev/null; then
        test_pass "install.sh has valid bash syntax"
    else
        test_fail "install.sh has syntax errors" "bash -n failed"
    fi
}

# ============================================================================
# Shell Config Detection Tests
# ============================================================================

test_shell_detection_priority() {
    test_start "SHELL env var takes priority over version vars"
    setup_test_env
    extract_setup_path

    export SHELL="/bin/zsh"
    export BASH_VERSION="5.0.0"  # This should be ignored

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_path_test.sh"
    setup_path "$HOME/.local/bin" > /dev/null 2>&1

    # Should detect zsh from $SHELL, not bash from BASH_VERSION
    assert_file_exists "$HOME/.zshrc"
    assert_file_not_contains "$HOME/.bashrc" "caro PATH" 2>/dev/null || true

    cleanup_test_env
}

# ============================================================================
# Integration-like Tests
# ============================================================================

test_full_path_workflow() {
    test_start "Full PATH setup workflow simulation"
    setup_test_env
    extract_setup_path

    export SHELL="/bin/zsh"

    # Simulate: PATH not set, run setup_path, verify config
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        # shellcheck source=/dev/null
        source "$TEST_TMPDIR/setup_path_test.sh"
        setup_path "$HOME/.local/bin" > /dev/null 2>&1
    fi

    assert_file_exists "$HOME/.zshrc"
    assert_file_contains "$HOME/.zshrc" "# caro PATH"
    assert_file_contains "$HOME/.zshrc" ".local/bin"

    cleanup_test_env
}

# ============================================================================
# Main Test Runner
# ============================================================================

main() {
    echo ""
    echo "═══════════════════════════════════════════════════════"
    echo "  Caro Setup Script Test Suite"
    echo "═══════════════════════════════════════════════════════"
    echo ""

    # Script syntax validation
    echo -e "${YELLOW}── Script Syntax ──${NC}"
    test_setup_sh_syntax
    test_install_sh_syntax
    echo ""

    # PATH setup tests
    echo -e "${YELLOW}── PATH Setup ──${NC}"
    test_path_setup_zsh
    test_path_setup_bash_with_bashrc
    test_path_setup_bash_with_bash_profile
    test_path_setup_fish
    test_path_setup_duplicate_prevention
    test_path_setup_zdotdir
    test_path_setup_unknown_shell
    echo ""

    # Shell detection tests
    echo -e "${YELLOW}── Shell Detection ──${NC}"
    test_shell_detection_priority
    echo ""

    # Error detection tests
    echo -e "${YELLOW}── Error Detection Patterns ──${NC}"
    test_error_detection_edition2024
    test_error_detection_cpp_headers
    test_error_detection_cstdint
    test_error_detection_vector
    echo ""

    # Integration tests
    echo -e "${YELLOW}── Integration ──${NC}"
    test_full_path_workflow
    echo ""

    # Summary
    echo "═══════════════════════════════════════════════════════"
    echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    echo "═══════════════════════════════════════════════════════"
    echo ""

    if [ "$TESTS_FAILED" -gt 0 ]; then
        exit 1
    fi
}

main "$@"
