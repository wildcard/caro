.PHONY: help build release build-cpu build-mlx test test-cpu clean fmt lint audit check bench install install-cpu run-cpu run-mlx

CPU_FEATURES := --no-default-features --features embedded-cpu
MLX_FEATURES := --features embedded-mlx,embedded-cpu
DEV_MODEL ?= smollm-135m-q4
MLX_MODEL ?= qwen-0.5b-q4
ARGS ?=

# Default target
help:
	@echo "Available targets:"
	@echo "Build Commands:"
	@echo "  build          - Build the project in debug mode"
	@echo "  release        - Build optimized release binary"
	@echo "  build-cpu      - Build release binary for CPU-only local development"
	@echo "  build-mlx      - Build release binary with Apple Silicon MLX support"
	@echo "Test Commands (Clean Output):"
	@echo "  test           - Run all tests quietly (default)"
	@echo "  test-cpu       - Run library and integration tests for CPU-only builds"
	@echo "  test-verbose   - Run all tests with verbose output"
	@echo "  test-quiet     - Run tests with minimal output"
	@echo "  test-show-output - Run tests showing stdout/stderr"
	@echo "  test-nextest   - Run tests with nextest (if installed)"
	@echo "  test-watch     - Watch for changes and run tests"
	@echo "Test Suites:"
	@echo "  test-contract  - Run contract tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-property  - Run property-based tests only"
	@echo "Quality Checks:"
	@echo "  fmt       - Format code with rustfmt"
	@echo "  lint      - Run clippy lints"
	@echo "  audit     - Run security audit"
	@echo "  check     - Run all quality checks"
	@echo "Other:"
	@echo "  bench     - Run benchmarks"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install caro locally"
	@echo "  install-cpu - Install caro locally using the CPU-only feature set"
	@echo "  run-cpu   - Run caro from source with the CPU-only feature set"
	@echo "  run-mlx   - Run caro from source with MLX enabled"

# Build commands
build:
	cargo build

release:
	cargo build --release

build-cpu:
	cargo build --release $(CPU_FEATURES)

build-mlx:
	cargo build --release $(MLX_FEATURES)

# Test commands with cleaner output
test:
	RUST_LOG=warn cargo test -q --all-features

test-cpu:
	RUST_LOG=warn cargo test $(CPU_FEATURES)

test-verbose:
	RUST_LOG=debug cargo test --verbose --all-features

test-quiet:
	RUST_LOG=error cargo test -q --all-features -- --quiet

test-show-output:
	RUST_LOG=warn cargo test --all-features -- --nocapture

# Specific test suites
test-contract:
	RUST_LOG=warn cargo test --test "backend_trait_contract" --test "safety_validator_contract" --test "cli_interface_contract" -q

test-integration:
	RUST_LOG=warn cargo test --test "integration_tests" -q

test-property:
	RUST_LOG=warn cargo test --test "property_tests" -q

test-error-handling:
	RUST_LOG=warn cargo test --test "error_handling_tests" -q

test-performance:
	RUST_LOG=warn cargo test --test "performance_tests" -q

# Nextest commands (cleaner output)
test-nextest:
	@command -v cargo-nextest >/dev/null 2>&1 && { \
		RUST_LOG=warn cargo nextest run --all-features; \
	} || { \
		echo "cargo-nextest not found, using standard test"; \
		make test; \
	}

test-nextest-verbose:
	@command -v cargo-nextest >/dev/null 2>&1 && { \
		RUST_LOG=debug cargo nextest run --all-features --profile verbose; \
	} || { \
		echo "cargo-nextest not found, using verbose test"; \
		make test-verbose; \
	}

# Watch mode for continuous testing
test-watch:
	@command -v cargo-watch >/dev/null 2>&1 && { \
		RUST_LOG=warn cargo watch -x "test -q --all-features"; \
	} || { \
		echo "cargo-watch not found. Install with: cargo install cargo-watch"; \
	}

# Code quality
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

audit:
	cargo audit

# Combined quality check
check: fmt-check lint audit test

# Performance
bench:
	cargo bench

# Binary size check (must be < 50MB)
size-check: release
	@SIZE=$$(stat -f%z target/release/cmdai 2>/dev/null || stat -c%s target/release/cmdai); \
	echo "Binary size: $$SIZE bytes"; \
	if [ $$SIZE -gt 52428800 ]; then \
		echo "❌ Binary size ($$SIZE bytes) exceeds 50MB limit"; \
		exit 1; \
	else \
		echo "✅ Binary size ($$SIZE bytes) is within 50MB limit"; \
	fi

# Utility commands
clean:
	cargo clean

install:
	cargo install --path .

install-cpu:
	cargo install --path . $(CPU_FEATURES)

# Development setup
setup:
	rustup component add rustfmt clippy
	cargo install cargo-audit cargo-deny cargo-llvm-cov

# Documentation
doc:
	cargo doc --no-deps --open

# Run with debug logging
run-debug:
	RUST_LOG=debug cargo run --

run-cpu:
	CARO_MODEL=$(DEV_MODEL) cargo run $(CPU_FEATURES) -- $(ARGS)

run-mlx:
	CARO_MODEL=$(MLX_MODEL) cargo run $(MLX_FEATURES) -- $(ARGS)

# Profile release build
profile: release
	@echo "Release binary info:"
	@ls -la target/release/cmdai
	@echo ""
	@echo "Dependencies:"
	@ldd target/release/cmdai 2>/dev/null || otool -L target/release/cmdai 2>/dev/null || echo "Dependencies check not available"
