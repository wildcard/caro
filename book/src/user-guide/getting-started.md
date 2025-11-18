# Getting Started

This guide will help you get cmdai up and running on your system.

## Prerequisites

- **Rust 1.75+** with Cargo
- **macOS with Apple Silicon** (for MLX backend, optional)
- **Linux or Windows** (CPU backend will be used)

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

- [Installation](./installation.md) - Installation options
- [Quick Start](./quick-start.md) - Common usage patterns
- [Configuration](./configuration.md) - Customize cmdai
- [Safety & Security](./safety.md) - Understand safety features
