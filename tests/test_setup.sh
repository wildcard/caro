#!/usr/bin/env bash
#
# Test suite for setup.sh
#
# Usage: ./tests/test_setup.sh

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
    # Mock cmdai binary
    echo '#!/bin/bash' > "$CARGO_HOME/bin/cmdai"
    echo 'echo "cmdai mock"' >> "$CARGO_HOME/bin/cmdai"
    chmod +x "$CARGO_HOME/bin/cmdai"
}

cleanup_test_env() {
    if [ -n "$TEST_TMPDIR" ] && [ -d "$TEST_TMPDIR" ]; then
        rm -rf "$TEST_TMPDIR"
    fi
    # Clean up environment variables that tests might have set
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

# Source the setup_alias function from setup.sh
# We'll create a wrapper that isolates just the setup_alias function
extract_setup_alias() {
    # Extract just the setup_alias function and its dependencies
    cat > "$TEST_TMPDIR/setup_alias_test.sh" << 'EOF'
say() { echo "$1"; }
say_success() { echo "$1"; }
say_warn() { echo "$1"; }
say_error() { echo "$1"; }

# Setup shell alias
setup_alias() {
    local shell_config=""
    local shell_name=""

    # Detect shell - prioritize $SHELL env var over subprocess shell version vars
    # This is important when script is run via 'bash <(curl ...)' where BASH_VERSION
    # would be set even if user's actual shell is zsh/fish
    case "$SHELL" in
        */bash)
            shell_name="bash"
            if [ -f "$HOME/.bashrc" ]; then
                shell_config="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_config="$HOME/.bash_profile"
            else
                shell_config="$HOME/.bashrc"  # default to .bashrc
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
            # Fallback to checking version variables if $SHELL is not set or unknown
            if [ -n "${ZSH_VERSION:-}" ]; then
                shell_name="zsh"
                shell_config="${ZDOTDIR:-$HOME}/.zshrc"
            elif [ -n "${BASH_VERSION:-}" ]; then
                shell_name="bash"
                if [ -f "$HOME/.bashrc" ]; then
                    shell_config="$HOME/.bashrc"
                elif [ -f "$HOME/.bash_profile" ]; then
                    shell_config="$HOME/.bash_profile"
                else
                    shell_config="$HOME/.bashrc"
                fi
            elif [ -n "${FISH_VERSION:-}" ]; then
                shell_name="fish"
                shell_config="$HOME/.config/fish/config.fish"
            else
                say_warn "Could not detect shell. Please manually add alias:"
                echo "  alias caro='cmdai'"
                return
            fi
            ;;
    esac

    if [ -z "$shell_config" ]; then
        say_warn "Could not detect shell config file. Please manually add alias:"
        echo "  alias caro='cmdai'"
        return
    fi

    if [ ! -f "$shell_config" ]; then
        say_warn "Shell config file not found. Creating $shell_config"
        # Create parent directory if needed (e.g., for fish config)
        mkdir -p "$(dirname "$shell_config")"
        touch "$shell_config"
    fi

    # Check if alias already exists
    if grep -q "alias caro=" "$shell_config" 2>/dev/null; then
        say "Alias 'caro' already exists in $shell_config"
        return
    fi

    # Add alias
    say "Adding alias 'caro' to $shell_config..."
    echo "" >> "$shell_config"
    echo "# Caro alias" >> "$shell_config"
    echo "alias caro='cmdai'" >> "$shell_config"

    say_success "Alias added successfully"
    echo ""
    say "Run 'source $shell_config' or restart your shell to use the alias"
}
EOF
}

# Test: Zsh detection and alias setup
test_zsh_detection() {
    test_start "Zsh detection and alias setup"
    setup_test_env
    extract_setup_alias

    export SHELL="/bin/zsh"
    unset ZSH_VERSION BASH_VERSION FISH_VERSION

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"
    setup_alias > /dev/null 2>&1

    assert_file_exists "$HOME/.zshrc"
    assert_file_contains "$HOME/.zshrc" "alias caro='cmdai'"

    cleanup_test_env
}

# Test: Bash detection with existing .bashrc
test_bash_detection_with_bashrc() {
    test_start "Bash detection with existing .bashrc"
    setup_test_env
    extract_setup_alias

    touch "$HOME/.bashrc"
    export SHELL="/bin/bash"
    unset ZSH_VERSION BASH_VERSION FISH_VERSION

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"
    setup_alias > /dev/null 2>&1

    assert_file_contains "$HOME/.bashrc" "alias caro='cmdai'"

    cleanup_test_env
}

# Test: Bash detection with .bash_profile only
test_bash_detection_with_bash_profile() {
    test_start "Bash detection with .bash_profile only"
    setup_test_env
    extract_setup_alias

    touch "$HOME/.bash_profile"
    export SHELL="/bin/bash"
    unset ZSH_VERSION BASH_VERSION FISH_VERSION

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"
    setup_alias > /dev/null 2>&1

    assert_file_contains "$HOME/.bash_profile" "alias caro='cmdai'"

    cleanup_test_env
}

# Test: Bash detection without existing config (should create .bashrc)
test_bash_detection_no_config() {
    test_start "Bash detection without existing config files"
    setup_test_env
    extract_setup_alias

    export SHELL="/bin/bash"
    unset ZSH_VERSION BASH_VERSION FISH_VERSION

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"
    setup_alias > /dev/null 2>&1

    assert_file_exists "$HOME/.bashrc"
    assert_file_contains "$HOME/.bashrc" "alias caro='cmdai'"

    cleanup_test_env
}

# Test: Fish detection and directory creation
test_fish_detection() {
    test_start "Fish detection and config directory creation"
    setup_test_env
    extract_setup_alias

    export SHELL="/usr/bin/fish"
    unset ZSH_VERSION BASH_VERSION FISH_VERSION

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"
    setup_alias > /dev/null 2>&1

    assert_file_exists "$HOME/.config/fish/config.fish"
    assert_file_contains "$HOME/.config/fish/config.fish" "alias caro='cmdai'"

    cleanup_test_env
}

# Test: Subprocess bash detection (simulates bash <(curl ...) with zsh user)
test_subprocess_bash_with_zsh_user() {
    test_start "Subprocess bash detection with zsh user shell"
    setup_test_env
    extract_setup_alias

    export SHELL="/bin/zsh"  # User's actual shell
    export BASH_VERSION="5.0.0"  # Subprocess has BASH_VERSION set

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"
    setup_alias > /dev/null 2>&1

    # Should detect zsh from $SHELL, not bash from BASH_VERSION
    assert_file_exists "$HOME/.zshrc"
    assert_file_contains "$HOME/.zshrc" "alias caro='cmdai'"

    cleanup_test_env
}

# Test: Duplicate alias prevention
test_duplicate_alias_prevention() {
    test_start "Duplicate alias prevention"
    setup_test_env
    extract_setup_alias

    export SHELL="/bin/zsh"

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"

    # Run setup_alias twice
    setup_alias > /dev/null 2>&1
    setup_alias > /dev/null 2>&1

    # Count occurrences of the alias
    local count
    count=$(grep -c "alias caro='cmdai'" "$HOME/.zshrc" || true)

    if [ "$count" -eq 1 ]; then
        test_pass "Alias appears exactly once"
    else
        test_fail "Alias should appear exactly once" "Found $count occurrences"
    fi

    cleanup_test_env
}

# Test: ZDOTDIR support for zsh
test_zdotdir_support() {
    test_start "ZDOTDIR support for zsh"
    setup_test_env
    extract_setup_alias

    mkdir -p "$HOME/custom_zsh"
    export SHELL="/bin/zsh"
    export ZDOTDIR="$HOME/custom_zsh"

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"
    setup_alias > /dev/null 2>&1

    assert_file_exists "$HOME/custom_zsh/.zshrc"
    assert_file_contains "$HOME/custom_zsh/.zshrc" "alias caro='cmdai'"

    cleanup_test_env
}

# Test: Unknown shell fallback
test_unknown_shell_fallback() {
    test_start "Unknown shell fallback to version variables"
    setup_test_env
    extract_setup_alias

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"

    # Set these AFTER sourcing to avoid bash resetting BASH_VERSION
    export SHELL="/bin/unknown_shell"
    export ZSH_VERSION="5.8"
    unset BASH_VERSION

    setup_alias > /dev/null 2>&1

    # Should fallback to ZSH_VERSION detection
    assert_file_exists "$HOME/.zshrc"
    assert_file_contains "$HOME/.zshrc" "alias caro='cmdai'"

    cleanup_test_env
}

# Test: Completely unknown shell
test_completely_unknown_shell() {
    test_start "Completely unknown shell handling"
    setup_test_env
    extract_setup_alias

    # shellcheck source=/dev/null
    source "$TEST_TMPDIR/setup_alias_test.sh"

    # Set these AFTER sourcing to avoid bash resetting BASH_VERSION
    export SHELL="/bin/unknown_shell"
    unset ZSH_VERSION BASH_VERSION FISH_VERSION

    local output
    output=$(setup_alias 2>&1)

    # Should output manual instructions
    if echo "$output" | grep -q "Please manually add alias"; then
        test_pass "Shows manual instruction for unknown shell"
    else
        test_fail "Should show manual instruction for unknown shell" "Output was: $output"
    fi

    cleanup_test_env
}

# Test: Full script execution (dry run)
test_full_script_dry_run() {
    test_start "Full script syntax validation"

    if bash -n setup.sh; then
        test_pass "Script has valid bash syntax"
    else
        test_fail "Script has syntax errors" "bash -n failed"
    fi
}

# Helpers for the install_via_cargo tests (#1340). The mock cargo has
# three modes:
#   stale - metadata lists caro, binary deleted: "already installed", exit 0
#           unless --force, which writes the binary.
#   root  - an `install.root` config: installs into $TEST_TMPDIR/other-root.
#   fail  - the build fails: exit 101.
write_install_harness() {
    local mock_bin="$TEST_TMPDIR/mock-bin"
    mkdir -p "$mock_bin" "$TEST_TMPDIR/other-root/bin"
    cat > "$mock_bin/cargo" << 'EOF'
#!/bin/bash
echo "cargo $*" >> "$CARGO_HOME/cargo-calls.log"
case "$MOCK_CARGO_MODE" in
    fail)
        echo "error: failed to compile caro"
        exit 101
        ;;
    root)
        printf '#!/bin/bash\necho caro\n' > "$TEST_TMPDIR/other-root/bin/caro"
        chmod +x "$TEST_TMPDIR/other-root/bin/caro"
        exit 0
        ;;
esac
for arg in "$@"; do
    if [ "$arg" = "--force" ]; then
        printf '#!/bin/bash\necho caro\n' > "$CARGO_HOME/bin/caro"
        chmod +x "$CARGO_HOME/bin/caro"
        exit 0
    fi
done
echo "Ignored package \`caro v1.4.0\` is already installed, use --force to override"
exit 0
EOF
    chmod +x "$mock_bin/cargo"

    # Use the real function from setup.sh, not a copy.
    {
        echo 'say() { echo "$1"; }'
        echo 'say_success() { echo "$1"; }'
        echo 'say_error() { echo "$1"; }'
        echo 'err() { say_error "$1"; exit 1; }'
        echo 'FORCE_INSTALL="false"'
        sed -n '/^install_via_cargo() {/,/^}/p' setup.sh
        echo 'install_via_cargo'
    } > "$TEST_TMPDIR/install_via_cargo_test.sh"
}

# Run the harness with only the mock and system tools on PATH, so a real
# caro on this machine cannot hide the bug.
run_install_harness() {
    export TEST_TMPDIR
    MOCK_CARGO_MODE="$1" \
        PATH="$TEST_TMPDIR/mock-bin:/usr/bin:/bin" \
        bash "$TEST_TMPDIR/install_via_cargo_test.sh" > "$TEST_TMPDIR/out.log" 2>&1
}

# Test: cargo metadata lists caro but the binary was deleted (#1340).
# setup.sh must reinstall with --force instead of reporting success.
test_cargo_install_missing_binary() {
    test_start "cargo install repairs a missing binary"
    setup_test_env
    write_install_harness

    if run_install_harness stale && [ -x "$CARGO_HOME/bin/caro" ]; then
        test_pass "Missing binary reinstalled with --force"
    else
        test_fail "Missing binary not reinstalled" "$(cat "$TEST_TMPDIR/out.log")"
    fi

    cleanup_test_env
}

# Test: an `install.root` config puts the binary outside ~/.cargo/bin.
# That is a real install: no --force, no error.
test_cargo_install_custom_root() {
    test_start "cargo install with a custom install root"
    setup_test_env
    write_install_harness

    if run_install_harness root \
        && ! grep -q -- "--force" "$CARGO_HOME/cargo-calls.log"; then
        test_pass "Custom install root accepted without --force"
    else
        test_fail "Custom install root rejected or forced" "$(cat "$TEST_TMPDIR/out.log")"
    fi

    cleanup_test_env
}

# Test: a failed cargo build still fails setup.sh (the output goes through tee).
test_cargo_install_failure() {
    test_start "cargo install failure is reported"
    setup_test_env
    write_install_harness

    if ! run_install_harness fail \
        && grep -q "Failed to install via cargo" "$TEST_TMPDIR/out.log"; then
        test_pass "Failed cargo build fails the install"
    else
        test_fail "Failed cargo build was not reported" "$(cat "$TEST_TMPDIR/out.log")"
    fi

    cleanup_test_env
}

# Main test runner
main() {
    echo ""
    echo "═══════════════════════════════════════════════════════"
    echo "  Caro Setup Script Test Suite"
    echo "═══════════════════════════════════════════════════════"
    echo ""

    # Run all tests
    test_full_script_dry_run
    test_zsh_detection
    test_bash_detection_with_bashrc
    test_bash_detection_with_bash_profile
    test_bash_detection_no_config
    test_fish_detection
    test_subprocess_bash_with_zsh_user
    test_duplicate_alias_prevention
    test_zdotdir_support
    test_unknown_shell_fallback
    test_completely_unknown_shell
    test_cargo_install_missing_binary
    test_cargo_install_custom_root
    test_cargo_install_failure

    # Summary
    echo ""
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
