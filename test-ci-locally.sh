#!/bin/bash
#
# test-ci-locally.sh
# 
# Comprehensive local CI testing script that mirrors all GitHub Actions workflows
# Ensures 100% confidence that CI will pass before pushing to GitHub
#
# Workflows tested:
#   1. CI (ci.yml) - Test suite, formatting, linting, builds, benchmarks, coverage
#   2. Release (release.yml) - Release testing workflow (triggered after publish)
#   3. Publish (publish.yml) - Publish workflow tests
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
PASSED_TESTS=()
FAILED_TESTS=()
SKIPPED_TESTS=()

# Environment setup
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

# Helper functions
log_section() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

log_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

log_error() {
    echo -e "${RED}✗ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

log_info() {
    echo -e "${NC}  $1${NC}"
}

mark_passed() {
    PASSED_TESTS+=("$1")
    log_success "$1"
}

mark_failed() {
    FAILED_TESTS+=("$1")
    log_error "$1"
}

mark_skipped() {
    SKIPPED_TESTS+=("$1")
    log_warning "$1 (skipped)"
}

# Test execution wrapper
run_test() {
    local test_name="$1"
    shift
    local test_command="$@"
    
    log_info "Running: $test_name"
    if eval "$test_command"; then
        mark_passed "$test_name"
        return 0
    else
        mark_failed "$test_name"
        return 1
    fi
}

# ============================================================================
# 1. CI WORKFLOW (.github/workflows/ci.yml)
# ============================================================================

test_formatting() {
    log_section "CI: Check Formatting"
    run_test "cargo fmt --all -- --check" \
        "cargo fmt --all -- --check"
}

test_clippy() {
    log_section "CI: Run Clippy"
    run_test "cargo clippy --all-targets --all-features" \
        "cargo clippy --all-targets --all-features -- -D warnings"
}

test_comprehensive() {
    log_section "CI: Run Comprehensive Tests (single-threaded)"
    run_test "cargo test --workspace --all-features" \
        "cargo test --workspace --all-features --verbose -- --test-threads=1"
}

test_mlx_macos() {
    log_section "CI: Run MLX Backend Tests (macOS Apple Silicon only)"
    
    # Check if macOS Apple Silicon
    if [[ "$(uname)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
        run_test "cargo test mlx_backend_contract" \
            "cargo test --test mlx_backend_contract --verbose" && \
        run_test "cargo test mlx_integration_test" \
            "cargo test --test mlx_integration_test --verbose"
    else
        mark_skipped "MLX tests (not macOS Apple Silicon)"
    fi
}

test_e2e_per_platform() {
    log_section "CI: Run E2E Tests per Platform"
    
    run_test "cargo test e2e_cli_tests" \
        "cargo test --test e2e_cli_tests --verbose" && \
    run_test "cargo test e2e_interactive_execution" \
        "cargo test --test e2e_interactive_execution --verbose" && \
    run_test "cargo test embedded_integration" \
        "cargo test --test embedded_integration --verbose" && \
    run_test "cargo test system_integration" \
        "cargo test --test system_integration --verbose"
}

test_integration_per_platform() {
    log_section "CI: Run Integration Tests per Platform"
    
    run_test "cargo test integration_tests" \
        "cargo test --test integration_tests --verbose" && \
    run_test "cargo test infrastructure_integration" \
        "cargo test --test infrastructure_integration --verbose" && \
    run_test "cargo test error_handling_tests" \
        "cargo test --test error_handling_tests --verbose"
}

test_contract_per_platform() {
    log_section "CI: Run Contract Tests per Platform"
    
    run_test "cargo test backend_trait_contract" \
        "cargo test --test backend_trait_contract --verbose" && \
    run_test "cargo test safety_validator_contract" \
        "cargo test --test safety_validator_contract --verbose" && \
    run_test "cargo test platform_detection_contract" \
        "cargo test --test platform_detection_contract --verbose" && \
    run_test "cargo test config_contract" \
        "cargo test --test config_contract --verbose" && \
    run_test "cargo test execution_contract" \
        "cargo test --test execution_contract --verbose"
}

test_security_audit() {
    log_section "CI: Security Audit"
    
    # Check if cargo-audit is installed
    if ! command -v cargo-audit &> /dev/null; then
        log_warning "cargo-audit not installed, installing..."
        cargo install cargo-audit
    fi
    
    run_test "cargo audit" \
        "cargo audit"
}

test_build_release() {
    log_section "CI: Build Release Binary"
    
    # Determine platform-specific features
    local features=""
    if [[ "$(uname)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
        features="embedded-mlx,embedded-cpu,remote-backends"
    else
        features="embedded-cpu,remote-backends"
    fi
    
    run_test "cargo build --release (features: $features)" \
        "cargo build --release --features \"$features\""
}

test_benchmarks() {
    log_section "CI: Performance Benchmarks"
    
    # Build benchmarks (may fail due to known issues - allowed)
    log_info "Building benchmarks (may fail - allowed)..."
    if cargo bench --no-run --verbose 2>&1; then
        log_info "Benchmark build succeeded"
        
        # Try running benchmarks (also allowed to fail)
        log_info "Running benchmarks (may fail - allowed)..."
        if cargo bench --verbose 2>&1; then
            mark_passed "cargo bench (build and run)"
        else
            mark_skipped "cargo bench (run failed - allowed in TDD)"
        fi
    else
        mark_skipped "cargo bench (build failed - allowed in TDD)"
    fi
    
    # Check binary size
    run_test "Binary size check" \
        "bash -c 'cargo build --release && \
            SIZE=\$(stat -f%z target/release/caro 2>/dev/null || stat -c%s target/release/caro) && \
            echo \"Binary size: \$SIZE bytes\" && \
            if [ \$SIZE -gt 52428800 ]; then \
                echo \"❌ Binary size (\$SIZE bytes) exceeds 50MB limit\" && exit 1; \
            else \
                echo \"✅ Binary size (\$SIZE bytes) is within 50MB limit\" && exit 0; \
            fi'"
}

test_code_coverage() {
    log_section "CI: Code Coverage (Optional)"
    
    # Check if cargo-llvm-cov is installed
    if ! command -v cargo-llvm-cov &> /dev/null; then
        mark_skipped "cargo-llvm-cov (not installed)"
        return 0
    fi
    
    run_test "cargo llvm-cov" \
        "cargo llvm-cov --all-features --lcov --output-path lcov.info"
}

# ============================================================================
# 2. PUBLISH WORKFLOW (.github/workflows/publish.yml)
# ============================================================================

test_publish_workflow() {
    log_section "PUBLISH: Verify Version Consistency"
    
    # Get version from Cargo.toml
    CARGO_VERSION=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version')
    log_info "Cargo.toml version: $CARGO_VERSION"
    
    log_section "PUBLISH: Verify MLX Tests Compile (Ubuntu)"
    run_test "cargo test mlx_backend_contract --no-run" \
        "cargo test --test mlx_backend_contract --no-run --verbose" && \
    run_test "cargo test mlx_integration_test --no-run" \
        "cargo test --test mlx_integration_test --no-run --verbose"
    
    log_section "PUBLISH: Verify Package Can Be Created"
    run_test "cargo package --allow-dirty" \
        "cargo package --allow-dirty"
}

# ============================================================================
# 3. RELEASE WORKFLOW (.github/workflows/release.yml)
# ============================================================================

test_release_workflow() {
    log_section "RELEASE: Run Comprehensive Tests per Platform"
    
    # This is identical to CI comprehensive tests
    run_test "cargo test --workspace --all-features" \
        "cargo test --workspace --all-features --verbose"
    
    # macOS-specific MLX tests
    if [[ "$(uname)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
        log_section "RELEASE: Run MLX Backend Tests (macOS Apple Silicon)"
        run_test "cargo test mlx_backend_contract" \
            "cargo test --test mlx_backend_contract --verbose" && \
        run_test "cargo test mlx_integration_test" \
            "cargo test --test mlx_integration_test --verbose"
    fi
    
    log_section "RELEASE: Run E2E Tests per Platform"
    test_e2e_per_platform
    
    log_section "RELEASE: Run Contract Tests per Platform"
    run_test "cargo test backend_trait_contract" \
        "cargo test --test backend_trait_contract --verbose" && \
    run_test "cargo test safety_validator_contract" \
        "cargo test --test safety_validator_contract --verbose" && \
    run_test "cargo test platform_detection_contract" \
        "cargo test --test platform_detection_contract --verbose"
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

main() {
    log_section "Starting Comprehensive CI Test Suite"
    log_info "Platform: $(uname -s) $(uname -m)"
    log_info "Rust version: $(rustc --version)"
    log_info "Cargo version: $(cargo --version)"
    echo ""
    
    # Ensure cargo is in PATH
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found in PATH"
        log_info "Sourcing cargo environment..."
        source "$HOME/.cargo/env"
    fi
    
    # CI Workflow Tests
    log_section "WORKFLOW 1: CI (.github/workflows/ci.yml)"
    test_formatting || true
    test_clippy || true
    test_comprehensive || true
    test_mlx_macos || true
    test_e2e_per_platform || true
    test_integration_per_platform || true
    test_contract_per_platform || true
    test_security_audit || true
    test_build_release || true
    test_benchmarks || true
    test_code_coverage || true
    
    # Publish Workflow Tests
    log_section "WORKFLOW 2: PUBLISH (.github/workflows/publish.yml)"
    test_publish_workflow || true
    
    # Release Workflow Tests
    log_section "WORKFLOW 3: RELEASE (.github/workflows/release.yml)"
    test_release_workflow || true
    
    # Final Report
    log_section "Test Execution Summary"
    echo ""
    
    if [ ${#PASSED_TESTS[@]} -gt 0 ]; then
        log_success "Passed tests (${#PASSED_TESTS[@]}):"
        for test in "${PASSED_TESTS[@]}"; do
            echo -e "${GREEN}  ✓ $test${NC}"
        done
        echo ""
    fi
    
    if [ ${#SKIPPED_TESTS[@]} -gt 0 ]; then
        log_warning "Skipped tests (${#SKIPPED_TESTS[@]}):"
        for test in "${SKIPPED_TESTS[@]}"; do
            echo -e "${YELLOW}  ⚠ $test${NC}"
        done
        echo ""
    fi
    
    if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
        log_error "Failed tests (${#FAILED_TESTS[@]}):"
        for test in "${FAILED_TESTS[@]}"; do
            echo -e "${RED}  ✗ $test${NC}"
        done
        echo ""
        
        log_section "CI WILL FAIL - Fix failures before pushing"
        exit 1
    else
        log_section "ALL TESTS PASSED - CI will pass ✓"
        exit 0
    fi
}

# Run main function
main "$@"
