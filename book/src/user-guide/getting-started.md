# Getting Started

This guide will help you get cmdai up and running on your system.

> **💡 Quick Start:** If you want to jump right in, see the [Quick Start Guide](./quick-start.md) or start with [Your First Command Tutorial](../tutorial/first-command.md).

## Prerequisites

| Requirement | Details | Notes |
|-------------|---------|-------|
| **Rust** | Version 1.75+ with Cargo | Required for building from source |
| **macOS** | Apple Silicon recommended | MLX backend for optimal performance |
| **Linux** | Any modern distribution | CPU backend used |
| **Windows** | WSL2 recommended | CPU backend used |

### Checking Prerequisites

```bash
# Check Rust version
rustc --version
# Expected: rustc 1.75.0 or higher

# Check Cargo version
cargo --version
# Expected: cargo 1.75.0 or higher

# Update Rust if needed
rustup update
```

## Building from Source

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Build the project
cargo build --release

# Run the CLI
./target/release/cmdai --version
```

## Your First Command

Once built, try generating your first command:

```bash
./target/release/cmdai "list all files in the current directory"
```

cmdai will:
1. Generate a safe command based on your prompt
2. Show you the proposed command
3. Ask for confirmation before execution
4. Execute the command (if confirmed)

## Development Commands

```bash
# Run tests
make test

# Format code
make fmt

# Run linter
make lint

# Build optimized binary
make build-release

# Run with debug logging
RUST_LOG=debug cargo run -- "your command"
```

## Verify Installation

Check that cmdai is working correctly:

```bash
# Show version
./target/release/cmdai --version

# Show current configuration
./target/release/cmdai --show-config

# Run in verbose mode
./target/release/cmdai --verbose "echo hello world"
```

## Next Steps

**Start Using cmdai:**
- [Tutorial: Your First Command](../tutorial/first-command.md) - Interactive tutorial
- [Quick Start](./quick-start.md) - Common usage patterns
- [Safety & Security](./safety.md) - Understand safety features

**Advanced Configuration:**
- [Configuration](./configuration.md) - Customize cmdai
- [Installation Options](./installation.md) - Alternative installation methods

**For Developers:**
- [Architecture](../dev-guide/architecture.md) - System design
- [Contributing](../dev-guide/contributing.md) - Join the project

---

## See Also

**Tutorials:**
- [Tutorial: Your First Command](../tutorial/first-command.md) - Step-by-step introduction
- [Tutorial: Working with Files](../tutorial/working-with-files.md) - File operations
- [Tutorial: System Operations](../tutorial/system-operations.md) - System monitoring

**User Guides:**
- [Installation](./installation.md) - Detailed installation instructions
- [Quick Start](./quick-start.md) - Common patterns and examples
- [Safety & Security](./safety.md) - Understanding safety validation

**Developer Guides:**
- [Architecture](../dev-guide/architecture.md) - How cmdai works
- [Backend Development](../dev-guide/backends.md) - Implementing backends
- [TDD Workflow](../dev-guide/tdd-workflow.md) - Development methodology
