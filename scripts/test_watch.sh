#!/bin/bash

# TDD Test Watcher for cmdai
# Comprehensive test watching with nextest, clippy, and check
# Usage: ./scripts/test_watch.sh [OPTIONS]

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Configuration
readonly CARGO_ENV="$HOME/.cargo/env"
readonly WATCH_DIRS="src tests Cargo.toml"
readonly RUST_LOG_LEVEL="${RUST_LOG:-warn}"

# Ensure cargo is in PATH
if [[ -f "$CARGO_ENV" ]]; then
    source "$CARGO_ENV"
fi

# Utility functions
log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_section() { echo -e "${PURPLE}=== $* ===${NC}"; }

# Check if required tools are installed
check_dependencies() {
    local missing_tools=()
    
    if ! command -v cargo-watch &> /dev/null; then
        missing_tools+=("cargo-watch")
    fi
    
    if ! command -v cargo-nextest &> /dev/null; then
        log_warn "cargo-nextest not found - will use standard 'cargo test'"
    fi
    
    if ! command -v cargo-clippy &> /dev/null; then
        missing_tools+=("clippy")
    fi
    
    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Install with: cargo install ${missing_tools[*]}"
        exit 1
    fi
}

# Display help
show_help() {
    cat << EOF
TDD Test Watcher for cmdai - Comprehensive Rust TDD Environment

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose logging (RUST_LOG=debug)
    -q, --quiet         Minimal output (RUST_LOG=error)
    -f, --fast          Use nextest for faster test execution
    -c, --check-only    Run only cargo check (no tests)
    -t, --tests-only    Run only tests (skip check and clippy)
    -l, --lint-only     Run only clippy linting
    --no-clear          Don't clear screen between runs
    --features FEATURES Specify cargo features (e.g., --features "mock-backend")

EXAMPLES:
    $0                              # Standard TDD mode with nextest
    $0 --verbose                    # Verbose logging
    $0 --tests-only                 # Only run tests, skip checks
    $0 --features "mock-backend"    # Test with specific features

TDD WORKFLOW:
    1. Write a failing test (RED)
    2. Make minimal changes to pass (GREEN)  
    3. Refactor while keeping tests green (REFACTOR)
    4. Repeat

The watcher monitors: ${WATCH_DIRS}
EOF
}

# Parse command line arguments
parse_args() {
    VERBOSE=false
    QUIET=false
    USE_NEXTEST=true
    CHECK_ONLY=false
    TESTS_ONLY=false
    LINT_ONLY=false
    CLEAR_SCREEN=true
    FEATURES=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                VERBOSE=true
                export RUST_LOG=debug
                shift
                ;;
            -q|--quiet)
                QUIET=true
                export RUST_LOG=error
                shift
                ;;
            -f|--fast)
                USE_NEXTEST=true
                shift
                ;;
            -c|--check-only)
                CHECK_ONLY=true
                shift
                ;;
            -t|--tests-only)
                TESTS_ONLY=true
                shift
                ;;
            -l|--lint-only)
                LINT_ONLY=true
                shift
                ;;
            --no-clear)
                CLEAR_SCREEN=false
                shift
                ;;
            --features)
                FEATURES="--features $2"
                shift 2
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Build the watch command based on options
build_watch_command() {
    local commands=()
    
    if [[ "$CHECK_ONLY" == "true" ]]; then
        commands+=("check $FEATURES")
    elif [[ "$TESTS_ONLY" == "true" ]]; then
        if command -v cargo-nextest &> /dev/null && [[ "$USE_NEXTEST" == "true" ]]; then
            commands+=("nextest run $FEATURES --quiet")
        else
            commands+=("test $FEATURES -q")
        fi
    elif [[ "$LINT_ONLY" == "true" ]]; then
        commands+=("clippy $FEATURES --all-targets -- -D warnings")
    else
        # Full TDD mode: check -> test -> clippy
        commands+=("check $FEATURES")
        
        if command -v cargo-nextest &> /dev/null && [[ "$USE_NEXTEST" == "true" ]]; then
            commands+=("nextest run $FEATURES --quiet")
        else
            commands+=("test $FEATURES -q")
        fi
        
        commands+=("clippy $FEATURES --all-targets -- -D warnings")
    fi
    
    # Join commands with " && " and add clear flag
    local watch_cmd=""
    if [[ "$CLEAR_SCREEN" == "true" ]]; then
        watch_cmd="cargo watch -q -c"
    else
        watch_cmd="cargo watch -q"
    fi
    
    for cmd in "${commands[@]}"; do
        watch_cmd+=" -x \"$cmd\""
    done
    
    echo "$watch_cmd"
}

# Run initial status check
initial_status() {
    log_section "Initial Project Status"
    
    log_info "Checking compilation..."
    if RUST_LOG=error cargo check $FEATURES &> /dev/null; then
        log_success "✓ Project compiles successfully"
    else
        log_error "✗ Compilation errors detected"
        log_info "Running 'cargo check' to see errors:"
        cargo check $FEATURES
        return 1
    fi
    
    log_info "Running initial test suite..."
    local test_cmd
    if command -v cargo-nextest &> /dev/null && [[ "$USE_NEXTEST" == "true" ]]; then
        test_cmd="cargo nextest run $FEATURES --quiet"
    else
        test_cmd="cargo test $FEATURES -q"
    fi
    
    if RUST_LOG=error $test_cmd &> /dev/null; then
        log_success "✓ All tests passing"
    else
        log_warn "⚠ Some tests failing - this is normal for TDD!"
        log_info "Running tests to see failures:"
        $test_cmd
    fi
    
    echo ""
}

# Main function
main() {
    parse_args "$@"
    
    log_section "TDD Test Watcher for cmdai"
    log_info "Rust TDD Environment with cargo-watch, nextest, and clippy"
    log_info "Working directory: $(pwd)"
    log_info "Watching: $WATCH_DIRS"
    echo ""
    
    check_dependencies
    
    # Show configuration
    log_info "Configuration:"
    log_info "  - RUST_LOG: ${RUST_LOG:-warn}"
    log_info "  - Use nextest: $USE_NEXTEST"
    log_info "  - Features: ${FEATURES:-none}"
    log_info "  - Clear screen: $CLEAR_SCREEN"
    echo ""
    
    # Run initial status check
    if ! initial_status; then
        log_error "Fix compilation errors before starting TDD watcher"
        exit 1
    fi
    
    # Build and run the watch command
    local watch_cmd
    watch_cmd=$(build_watch_command)
    
    log_section "Starting TDD Watcher"
    log_info "TDD Cycle: RED (failing test) → GREEN (minimal fix) → REFACTOR"
    log_info "Press Ctrl+C to stop"
    log_info "Command: $watch_cmd"
    echo ""
    
    # Execute the watch command
    eval "$watch_cmd"
}

# Handle signals gracefully
trap 'log_info "TDD watcher stopped"; exit 0' INT TERM

# Run the main function
main "$@"