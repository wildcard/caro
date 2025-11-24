# Building cmdai

This guide explains how to build and test cmdai locally. If you're having trouble, skip to the [Troubleshooting](#troubleshooting) section.

## Quick Start (TL;DR)

```bash
# 1. Install Rust (if you haven't)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Clone and build
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build

# 3. Run tests
cargo test --lib

# 4. Try the TUI
cargo run -- --tui
```

## Prerequisites

### Required

- **Rust 1.75 or later**
  - Install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Verify: `cargo --version` should show `1.75.0` or higher

### Optional

- **Git** - For version control
- **Make** - For convenience commands (optional, can use `cargo` directly)

## Building

### 1. Clone the Repository

```bash
git clone https://github.com/wildcard/cmdai.git
cd cmdai
```

### 2. Build the Project

```bash
# Development build (faster, includes debug info)
cargo build

# Release build (optimized, slower to compile)
cargo build --release
```

The binary will be in:
- Development: `target/debug/cmdai`
- Release: `target/release/cmdai`

### 3. Run the Application

```bash
# Run directly with cargo
cargo run -- --help

# Or run the binary
./target/debug/cmdai --help

# Try the TUI mode
cargo run -- --tui
```

## Testing

### Run All Library Tests

```bash
cargo test --lib
```

This runs **85 unit tests** covering:
- TUI components and state management
- Safety validation patterns
- Configuration management
- Model implementations

### Run Specific Test Module

```bash
# Test only TUI components
cargo test --lib tui::

# Test only safety validation
cargo test --lib safety::

# Test a specific function
cargo test test_app_state_default
```

### Run Tests with Output

```bash
# See println! statements in tests
cargo test --lib -- --nocapture

# Show test names as they run
cargo test --lib -- --show-output
```

## Code Quality Checks

These are the same checks that run in CI. **Run these before submitting a PR!**

### Format Check

```bash
# Check formatting (don't modify files)
cargo fmt --all -- --check

# Auto-fix formatting
cargo fmt --all
```

### Linting with Clippy

```bash
# Run all lints (same as CI)
cargo clippy --lib --bins --tests --all-features -- -D warnings

# Auto-fix some issues
cargo clippy --lib --bins --tests --all-features --fix
```

### Full CI Check Locally

Run the same checks as GitHub Actions:

```bash
# 1. Format
cargo fmt --all -- --check

# 2. Clippy
cargo clippy --lib --bins --tests --all-features -- -D warnings

# 3. Build
cargo build --verbose

# 4. Test
cargo test --lib --verbose

# 5. Doc check
cargo doc --all-features --no-deps
```

## Feature Flags

cmdai uses Cargo features to enable different backends:

```bash
# No default features (minimal build)
cargo build --no-default-features

# Remote backends only (Ollama, vLLM)
cargo build --no-default-features --features remote-backends

# Embedded CPU backend
cargo build --no-default-features --features embedded-cpu

# All features
cargo build --all-features
```

Current features:
- `remote-backends` - Ollama and vLLM HTTP backends
- `embedded-cpu` - Local CPU inference with Candle
- `mlx` - Apple Silicon acceleration (macOS only, in development)

## Development Workflow

### Making Changes

1. **Create a branch**
   ```bash
   git checkout -b feature/my-awesome-feature
   ```

2. **Make your changes**
   - Edit code in `src/`
   - Add tests for new functionality
   - Update documentation if needed

3. **Check your work**
   ```bash
   # Format code
   cargo fmt --all

   # Run lints
   cargo clippy --lib --bins --tests --all-features --fix

   # Run tests
   cargo test --lib
   ```

4. **Commit and push**
   ```bash
   git add .
   git commit -m "feat: add my awesome feature"
   git push origin feature/my-awesome-feature
   ```

5. **Open a Pull Request** on GitHub

### Watch Mode (Auto-rebuild)

Install `cargo-watch` for automatic rebuilds on file changes:

```bash
cargo install cargo-watch

# Watch and run tests
cargo watch -x "test --lib"

# Watch and run clippy
cargo watch -x "clippy --lib --bins --tests"

# Watch and run the app
cargo watch -x "run -- --tui"
```

## Building for Different Platforms

### Cross-Platform Build

```bash
# Install cross-compilation tool
cargo install cross

# Build for Linux
cross build --target x86_64-unknown-linux-gnu

# Build for macOS (requires macOS host)
cargo build --target x86_64-apple-darwin

# Build for Windows (requires Windows host or cross)
cross build --target x86_64-pc-windows-msvc
```

### Platform-Specific Features

Some features are platform-specific:

```bash
# MLX backend (macOS Apple Silicon only)
cargo build --features mlx  # Only works on Apple Silicon Macs
```

## Troubleshooting

### "cargo: command not found"

**Problem**: Rust is not installed or not in PATH.

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to current shell
source $HOME/.cargo/env

# Verify
cargo --version
```

### "error: package ... requires rustc 1.75"

**Problem**: Your Rust version is too old.

**Solution**:
```bash
# Update Rust
rustup update stable

# Verify
cargo --version
```

### Compilation Errors After `git pull`

**Problem**: Dependencies or code structure changed.

**Solution**:
```bash
# Clean build artifacts
cargo clean

# Rebuild
cargo build
```

### Tests Failing Locally

**Problem**: Tests pass in CI but fail locally (or vice versa).

**Solution**:
```bash
# Clean and rebuild
cargo clean
cargo build

# Run tests with verbose output
cargo test --lib -- --nocapture --show-output

# Check which tests are failing
cargo test --lib 2>&1 | grep "test.*FAILED"
```

### Clippy Warnings

**Problem**: Clippy shows warnings not seen in your editor.

**Solution**:
```bash
# Run clippy exactly as CI does
cargo clippy --lib --bins --tests --all-features -- -D warnings

# Fix automatically where possible
cargo clippy --lib --bins --tests --all-features --fix
```

### Slow Builds

**Problem**: Compilation takes forever.

**Solution**:
```bash
# Use development profile (faster builds, slower runtime)
cargo build  # instead of cargo build --release

# Incremental compilation (should be on by default)
export CARGO_INCREMENTAL=1

# Use mold linker (much faster on Linux)
# Install: cargo install mold
# Add to .cargo/config.toml:
# [target.x86_64-unknown-linux-gnu]
# linker = "clang"
# rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

### "Cannot Find Binary" After Build

**Problem**: Built successfully but can't find the binary.

**Solution**:
```bash
# Development builds go here:
ls -lh target/debug/cmdai

# Release builds go here:
ls -lh target/release/cmdai

# Run with cargo (finds it automatically):
cargo run -- --help
```

### Permission Denied on macOS/Linux

**Problem**: `./cmdai: Permission denied`

**Solution**:
```bash
# Make binary executable
chmod +x target/debug/cmdai

# Or just use cargo run
cargo run -- --help
```

### CI Failing But Local Builds Pass

**Problem**: GitHub Actions CI is red but everything works locally.

**Solution**:

1. **Check which job is failing** in the GitHub Actions tab
2. **Run the exact same command locally**:
   - Format: `cargo fmt --all -- --check`
   - Clippy: `cargo clippy --lib --bins --tests --all-features -- -D warnings`
   - Tests: `cargo test --lib --verbose`

3. **Common causes**:
   - Uncommitted formatting changes (run `cargo fmt --all`)
   - New clippy warnings (run `cargo clippy --fix`)
   - Test failures on different platforms

### Still Stuck?

**Where to get help**:

1. **Check existing issues**: [GitHub Issues](https://github.com/wildcard/cmdai/issues)
2. **Open a new issue**: Include:
   - Your OS and Rust version (`rustc --version`)
   - Full error message
   - Steps you've tried
3. **Join discussions**: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)

## CI/CD Pipeline

cmdai uses GitHub Actions for continuous integration. The pipeline runs on:

- Every push to `main` branch
- Every push to `claude/**` branches
- Every pull request to `main`

### CI Jobs

1. **Format Check** - Ensures code is formatted with `rustfmt`
2. **Clippy Lint** - Catches common mistakes and enforces best practices
3. **Test Matrix** - Runs tests on Ubuntu, macOS, and Windows with different feature combinations
4. **TUI Build** - Specifically tests the TUI module compiles and runs
5. **Documentation** - Ensures docs build without warnings
6. **Security Audit** - Scans dependencies for known vulnerabilities
7. **Release Build** - Creates optimized binaries for all platforms

### Viewing CI Results

Check your PR or commit for a green ✅ or red ❌:

- ✅ **Green** = All checks passed, safe to merge
- ❌ **Red** = Something failed, click "Details" to see what

![CI Status Badge](https://github.com/wildcard/cmdai/actions/workflows/ci.yml/badge.svg)

The badge at the top of the README shows current main branch status.

## Performance Considerations

### Build Times

Typical build times (on modern hardware):

- **Clean build**: 2-5 minutes
- **Incremental build**: 10-30 seconds
- **Test suite**: 5-15 seconds

### Reducing Build Time

```bash
# Use --lib to skip building binaries
cargo test --lib  # Instead of cargo test

# Use fewer features
cargo build --no-default-features

# Parallel builds (usually automatic)
cargo build -j 8  # 8 parallel jobs
```

## Next Steps

- **Read** [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines
- **Check out** [docs/TUI_CONTRIBUTING.md](docs/TUI_CONTRIBUTING.md) for TUI development
- **Review** [docs/TUI_DEVELOPMENT_PLAN.md](docs/TUI_DEVELOPMENT_PLAN.md) for architecture

## Quick Reference

| Command | Purpose |
|---------|---------|
| `cargo build` | Build in debug mode |
| `cargo build --release` | Build optimized binary |
| `cargo test --lib` | Run tests |
| `cargo fmt --all` | Format code |
| `cargo clippy --lib --bins --tests -- -D warnings` | Lint code |
| `cargo run -- --help` | Run with help flag |
| `cargo run -- --tui` | Launch TUI mode |
| `cargo doc --open` | Build and open documentation |
| `cargo clean` | Remove build artifacts |

---

**Happy building! 🦀**

If this guide helped you, consider starring the repo ⭐ and contributing back!
