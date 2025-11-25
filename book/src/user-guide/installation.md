# Installation

cmdai can be installed in several ways depending on your platform and preferences.

## Build from Source (Recommended)

The recommended way to install cmdai is to build from source:

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Build release binary
cargo build --release

# Optional: Install to system
cargo install --path .
```

## Platform-Specific Notes

### macOS (Apple Silicon)

On Apple Silicon Macs (M1/M2/M3), cmdai will automatically use the optimized MLX backend:

```bash
# Build with MLX support (automatic on Apple Silicon)
cargo build --release --features mlx
```

### macOS (Intel)

Intel Macs will use the CPU backend:

```bash
# Standard build
cargo build --release
```

### Linux

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install build-essential pkg-config

# Build cmdai
cargo build --release
```

### Windows

```bash
# Build with PowerShell support
cargo build --release
```

## Verify Installation

After installation, verify everything is working:

```bash
# Check version
cmdai --version

# Show configuration
cmdai --show-config

# Test basic functionality
cmdai "echo test"
```

## Backend Configuration

After installation, you may want to configure your preferred backend. See the [Configuration](./configuration.md) guide for details.

## Troubleshooting

### Command not found

If `cmdai` is not found after installation:

```bash
# Add cargo bin to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Make it permanent (bash)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# Make it permanent (zsh)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
```

### Build Errors

If you encounter build errors:

1. Ensure Rust is up to date: `rustup update`
2. Check Rust version: `rustc --version` (should be 1.75+)
3. Clean and rebuild: `cargo clean && cargo build --release`

### MLX Backend Not Available

If MLX backend is not detected on Apple Silicon:

1. Verify you're on Apple Silicon: `uname -m` (should show `arm64`)
2. Ensure MLX feature is enabled: `cargo build --release --features mlx`
3. Check build output for MLX-related messages

## Next Steps

- [Quick Start](./quick-start.md) - Learn common usage patterns
- [Configuration](./configuration.md) - Customize cmdai settings
- [Safety & Security](./safety.md) - Understand safety features
