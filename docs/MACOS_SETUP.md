# macOS Setup Guide for caro

This guide covers setup for caro on macOS, with special attention to Apple Silicon (M1/M2/M3/M4) for GPU acceleration.

## Prerequisites

### Required
- **macOS**: 10.15 (Catalina) or later
- **Rust**: 1.83 or later
- **Homebrew**: Package manager for macOS

### Optional (for GPU acceleration)
- **CMake**: Required for MLX builds on Apple Silicon
- **Xcode**: Full Xcode installation for Metal compiler (Apple Silicon only)

## Quick Start (All Macs)

### 1. Install Rust

```bash
# Install rustup (Rust toolchain installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Load Rust environment
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version
```

### 2. Install Homebrew (if not already installed)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### 3. Install Optional Native Dependencies

```bash
# Only needed for MLX or knowledge-related feature work
brew install protobuf

# Only needed for MLX GPU builds
brew install cmake

# Verify optional tools
protoc --version
cmake --version
```

### 4. Clone and Build

```bash
git clone https://github.com/wildcard/caro.git
cd caro

# Fast local build that works without full Xcode
cargo build --release --no-default-features --features embedded-cpu

# Install locally
cargo install --path . --no-default-features --features embedded-cpu
```

### 5. Test Installation

```bash
# Run a test command
CARO_MODEL=smollm-135m-q4 caro "list all files"

# Or using cargo
CARO_MODEL=smollm-135m-q4 cargo run --release --no-default-features --features embedded-cpu -- "find text files"
```

## Apple Silicon GPU Acceleration

Apple Silicon (M1/M2/M3/M4) chips support GPU-accelerated inference via the MLX framework, providing ~4x faster inference compared to CPU-only mode.

### Current Status

The recommended development path on Apple Silicon is now an explicit **CPU-only build** that:
- ✅ Works with Rust alone and does not require full Xcode
- ✅ Builds and runs from source on `macos/arm64`
- ✅ Lets you use a small model for fast first-run setup
- ✅ Keeps MLX as an upgrade path when you need GPU inference

**For real GPU acceleration**, you need both CMake and the Metal compiler from full Xcode.

### Option 1: CPU-Only Development Build (Recommended for Bring-Up)

**No full Xcode required.** Build and run with the CPU feature set:

```bash
# Build for CPU-only local development
cargo build --release --no-default-features --features embedded-cpu

# Run with a smaller first-download model
CARO_MODEL=smollm-135m-q4 cargo run --release --no-default-features --features embedded-cpu -- "list files"
```

**When to use:**
- Quick testing and development
- You want to avoid installing full Xcode
- You're developing non-inference features
- You want a smaller initial model download during bring-up

**Performance:**
- Model load: depends on the selected model
- Response time: slower than MLX GPU mode
- Memory: depends on the selected model

### Option 2: Full GPU Acceleration with Xcode

**For production use with real GPU-accelerated inference:**

#### Step 1: Install Xcode

Choose one of these methods:

**Method A: App Store (Recommended)**
1. Open App Store
2. Search for "Xcode"
3. Click "Get" or "Install"
4. Wait for download (~15GB) and installation
5. Open Xcode once to accept license

**Method B: Command Line**
```bash
# Check if Xcode is already installed
xcode-select -p

# If not installed, install Command Line Tools first
xcode-select --install

# Then download Xcode from Apple Developer
open https://developer.apple.com/xcode/
```

#### Step 2: Configure Xcode

```bash
# Accept Xcode license
sudo xcodebuild -license accept

# Set Xcode as active developer directory
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer

# Verify Metal compiler is available
xcrun --find metal
# Should output: /usr/bin/metal or similar

# Check Metal version
metal --version
```

#### Step 3: Build with MLX Feature

```bash
cd caro

# Clean previous builds
cargo clean

# Build with MLX GPU acceleration
cargo build --release --features embedded-mlx,embedded-cpu

# This will:
# - Compile mlx-rs (may take 5-10 minutes first time)
# - Link against Metal framework
# - Enable GPU acceleration
```

#### Step 4: Verify GPU Acceleration

```bash
# Run with logging to see MLX initialization
RUST_LOG=info cargo run --release --features embedded-mlx,embedded-cpu -- "list all files"

# You should see:
# INFO caro::backends::embedded::mlx: MLX GPU initialized
# INFO caro::backends::embedded::mlx: Using Metal device
```

**Expected Performance (M4 Pro):**
- Model load: < 2s (MLX optimization)
- First inference: < 2s
- Subsequent inference: < 500ms
- First token latency: < 200ms
- Memory: ~1.2GB (unified memory)

## Troubleshooting

### "metal: command not found"

**Problem**: Metal compiler not found when building with `embedded-mlx` feature.

**Solution**: Install full Xcode (not just Command Line Tools):
```bash
# Check current developer directory
xcode-select -p

# If it shows /Library/Developer/CommandLineTools, you need full Xcode
# Download from App Store or https://developer.apple.com/xcode/

# After installing, switch to Xcode
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

### "xcrun: error: unable to find utility 'metal'"

**Problem**: Xcode is installed but not configured as active developer directory.

**Solution**:
```bash
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
xcrun --find metal  # Should now work
```

### "mlx-sys build failed"

**Problem**: CMake or Metal compiler issues during mlx-rs compilation.

**Solution**:
```bash
# Verify all dependencies
cmake --version          # Should show 3.x or higher
xcrun --find metal       # Should show Metal compiler path

# Clean and rebuild
cargo clean
cargo build --release --features embedded-mlx,embedded-cpu

# If you only need local development, use the CPU-only path:
cargo build --release --no-default-features --features embedded-cpu
```

### Model Download Issues

**Problem**: Model fails to download from Hugging Face.

**Solution**:
```bash
# Check internet connection
curl -I https://huggingface.co

# Manually download model
mkdir -p ~/.cache/caro/models
cd ~/.cache/caro/models

# Download from Hugging Face (small dev model, ~82MB)
curl -L -o smollm-135m-instruct-q4_k_m.gguf \
  "https://huggingface.co/HuggingFaceTB/SmolLM-135M-Instruct-GGUF/resolve/main/smollm-135m-instruct-q4_k_m.gguf"

# Verify file
ls -lh smollm-135m-instruct-q4_k_m.gguf
```

### "Failed to load model"

**Problem**: Model file corrupted or not found.

**Solution**:
```bash
# Check model location
ls -lh ~/Library/Caches/caro/models/
# or
ls -lh ~/.cache/caro/models/

# Remove corrupted model
rm ~/Library/Caches/caro/models/*.gguf

# Rerun caro to trigger re-download
CARO_MODEL=smollm-135m-q4 cargo run --release --no-default-features --features embedded-cpu -- "test"
```

## Platform Detection

The project automatically detects your platform:

```bash
# Check what backend will be used
cargo test model_variant_detect --lib -- --nocapture

# On Apple Silicon with `embedded-mlx` enabled:
# ✅ ModelVariant::MLX

# On Apple Silicon without `embedded-mlx`, or on other platforms:
# ✅ ModelVariant::CPU
```

## Build Profiles

### Development Build (Fast compilation)
```bash
cargo build
# - Unoptimized
# - Debug symbols included
# - Fast compile times
# - Slower runtime
```

### Release Build (Optimized)
```bash
cargo build --release --no-default-features --features embedded-cpu
# - Full optimizations
# - Stripped debug symbols
# - Binary size optimized
# - Best default for Apple Silicon bring-up without Xcode
```

### Release with Debug Info (Profiling)
```bash
cargo build --profile release-with-debug
# - Full optimizations
# - Debug symbols included
# - For profiling and debugging
```

## Environment Variables

```bash
# Enable debug logging
export RUST_LOG=debug

# Enable info logging (recommended)
export RUST_LOG=info

# Disable network access (test offline operation)
export NO_NETWORK=1

# Select a smaller dev model for first run
export CARO_MODEL=smollm-135m-q4
```

## Uninstallation

```bash
# Remove installed binary
cargo uninstall caro

# Remove cache and models
rm -rf ~/Library/Caches/caro
rm -rf ~/.cache/caro

# Remove project directory
cd .. && rm -rf caro
```

## System Requirements

### Minimum
- macOS 10.15+
- 4GB RAM
- 2GB free disk space (for model cache)
- Internet connection (first run only)

### Recommended for GPU Acceleration
- Apple Silicon Mac (M1/M2/M3/M4)
- 8GB+ RAM
- macOS 12.0+
- Xcode 14+ installed
- 5GB free disk space (includes Xcode)

## Performance Comparison

### Apple Silicon M4 Pro

| Backend | First Inference | Subsequent | Model Load | Memory |
|---------|----------------|------------|------------|--------|
| **Stub** | ~100ms | ~100ms | ~500ms | ~1.1GB |
| **MLX (GPU)** | < 2s | < 500ms | < 2s | ~1.2GB |
| **CPU** | ~4s | ~3s | ~3s | ~1.5GB |

### Intel Mac

| Backend | First Inference | Subsequent | Model Load | Memory |
|---------|----------------|------------|------------|--------|
| **CPU** | ~5s | ~4s | ~4s | ~1.5GB |

## Additional Resources

- [Apple Silicon MLX Framework](https://github.com/ml-explore/mlx)
- [Xcode Download](https://developer.apple.com/xcode/)
- [Homebrew Documentation](https://brew.sh)
- [Rust Installation Guide](https://www.rust-lang.org/tools/install)

## Support

For issues specific to macOS:
- Check Metal is available: `xcrun --find metal`
- Verify Xcode version: `xcodebuild -version`
- Test Metal shader compilation: `xcrun -sdk macosx metal`
- Check system info: `system_profiler SPHardwareDataType | grep Chip`

## Summary

**For quick start**: Install Rust, then use the CPU-only build path. It works immediately without full Xcode.

**For production GPU acceleration**: Install CMake and full Xcode, verify the Metal compiler, and build with `--features embedded-mlx,embedded-cpu`.

Both modes are supported: CPU-only is the easiest development bring-up, while MLX provides real GPU-accelerated inference for Apple Silicon.
