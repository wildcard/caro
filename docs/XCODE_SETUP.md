# Xcode and Metal Compiler Setup for caro (macOS)

## Why is Xcode Needed?

The `mlx-rs` crate requires Apple's **Metal compiler** to build GPU-accelerated machine learning code for Apple Silicon. The Metal compiler is only included in full Xcode, not in the Command Line Tools.

## Current Status

Typical Apple Silicon development machines have:
- ✅ Command Line Tools installed
- ✅ Enough tooling for CPU-only source builds
- ❌ Metal compiler (requires full Xcode)

## Installation Options

### Option 1: Use the CPU-Only Development Path (No Xcode Needed)

**Status:** ✅ **WORKING NOW**

The recommended bring-up path on Apple Silicon is a CPU-only build that:
- Avoids full Xcode during initial setup
- Builds and runs from source cleanly
- Lets you start with a small model download
- Keeps MLX as a separate upgrade step

```bash
# This works RIGHT NOW without Xcode:
cd caro
CARO_MODEL=smollm-135m-q4 cargo run --release --no-default-features --features embedded-cpu -- "list files"

# Or build/install first:
cargo build --release --no-default-features --features embedded-cpu
cargo install --path . --no-default-features --features embedded-cpu
```

**Pros:**
- ✅ Works immediately
- ✅ No multi-GB Xcode download
- ✅ Clear, reproducible feature set
- ✅ Good default for local development

**Cons:**
- ⚠️ Slower than MLX GPU mode
- ⚠️ Not the highest-quality Apple Silicon path

### Option 2: Install Xcode for GPU Acceleration

**Enables:** Real GPU-accelerated inference with MLX framework

#### Step 1: Install Xcode

**Method A: App Store (Easiest)**
1. Open App Store
2. Search for "Xcode"
3. Click "Get" (or "Install" if previously installed)
4. Wait for ~15GB download
5. Launch Xcode once to accept license

**Method B: Direct Download**
1. Go to https://developer.apple.com/xcode/
2. Download Xcode 15.x or later
3. Open the .xip file
4. Move Xcode.app to /Applications/
5. Open Xcode and accept license

#### Step 2: Configure Xcode

```bash
# Accept license (if not done via GUI)
sudo xcodebuild -license accept

# Set Xcode as active developer directory
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer

# Verify Metal compiler is available
xcrun --find metal
# Should output: /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/metal

# Test Metal version
metal --version
```

#### Step 3: Build with MLX

```bash
cd caro

# Clean previous build
cargo clean

# Build with MLX feature (this will take 5-10 minutes first time)
cargo build --release --features embedded-mlx,embedded-cpu

# If successful, you'll see:
# Compiling mlx-sys...
# Compiling mlx-rs...
# Compiling caro...
# Finished `release` profile
```

#### Step 4: Test GPU Acceleration

```bash
# Run with info logging
RUST_LOG=info cargo run --release --features embedded-mlx,embedded-cpu -- "list all files recursively"

# You should see different output indicating real inference:
# INFO caro::backends::embedded::mlx: MLX GPU initialized
# INFO caro::backends::embedded::mlx: Using Metal device
# Command: find . -type f  (actual inference result)
```

## Verification Commands

### Check Current Setup

```bash
# Check if Xcode is installed
xcode-select -p
# /Library/Developer/CommandLineTools = CLI tools only (CPU-only mode)
# /Applications/Xcode.app/... = Full Xcode (GPU mode available)

# Check if Metal is available
xcrun --find metal
# Error = No full Xcode (CPU-only mode)
# /Applications/... = Full Xcode (GPU mode available)

# Check Xcode version (if installed)
xcodebuild -version
# Will show version if full Xcode installed

# Test Metal compilation
echo 'kernel void test() {}' | metal -o /dev/null -
# Success = Metal compiler working
# Error = Need full Xcode
```

### Check Build Features

```bash
# See what features are active
cargo build --release --verbose 2>&1 | grep features

# CPU-only build:
# --no-default-features --features embedded-cpu

# GPU build:
# --features embedded-mlx,embedded-cpu
```

## Performance Comparison

### CPU-Only Bring-Up
```
First run:        Depends on selected model download
Response time:    Slower than MLX GPU mode
Memory:           Depends on selected model
Accuracy:         Suitable for development bring-up
```

### With Xcode + MLX GPU
```
First run:        ~2s (MLX initialization + model load)
First inference:  < 2s (real GPU inference)
Subsequent:       < 500ms (model cached)
Memory:           ~1.2GB (unified GPU/CPU memory)
Accuracy:         Full LLM capabilities
```

## Decision Guide

### Use the CPU-Only Path If:
- ✅ You want to start developing immediately
- ✅ You're testing non-inference features
- ✅ You don't want to install 15GB Xcode
- ✅ You want the smallest first-run setup
- ✅ You're developing integration tests

### Install Xcode If:
- 🚀 You need real AI-powered command generation
- 🚀 You want production-quality inference
- 🚀 You're benchmarking performance
- 🚀 You need the full capabilities of the LLM
- 🚀 You plan to deploy this for actual use

## Current Project Status

```
Platform:         ✅ Apple Silicon detected
CPU bring-up:     ✅ Available without full Xcode
MLX GPU:          ⏳ Requires CMake + full Xcode
```

## Quick Commands Reference

```bash
# Build with the CPU-only path (works now)
cargo build --release --no-default-features --features embedded-cpu

# Try to build with GPU (will fail without Xcode)
cargo build --release --features embedded-mlx,embedded-cpu

# Run with CPU-only features
CARO_MODEL=smollm-135m-q4 cargo run --release --no-default-features --features embedded-cpu -- "list files"

# Check what's blocking GPU mode
xcrun --find metal  # If error, need Xcode

# After installing Xcode, rebuild
cargo clean
cargo build --release --features embedded-mlx,embedded-cpu
```

## Support

If you encounter issues:

1. **"metal: not found"** → Install full Xcode from App Store
2. **"mlx-sys build failed"** → Run `xcode-select --switch /Applications/Xcode.app/...`
3. **CPU backend selected** → Build with `--features embedded-mlx,embedded-cpu` after installing Xcode
4. **CMake errors** → Update CMake: `brew upgrade cmake`

## Summary

**Current state:** CPU-only source builds work without full Xcode, so you can start local development immediately.

**To unlock GPU:** Install Xcode, ensure `xcrun --find metal` works, and rebuild with `--features embedded-mlx,embedded-cpu`.

**Recommendation:** Use the CPU-only path for bring-up and day-one development, then add Xcode when you need MLX performance.
