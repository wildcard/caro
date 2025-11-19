# Quick Start: Metal Backend on Apple Silicon

Get cmdai running with GPU-accelerated inference on your Apple Silicon Mac in under 5 minutes.

## Prerequisites

- **Hardware**: Mac with Apple Silicon (M1, M2, M3, or M4)
- **OS**: macOS 12.0+ (Monterey or later)
- **Rust**: 1.82.0 or later
- **Disk Space**: ~2GB (model + build artifacts)
- **Xcode**: Command Line Tools installed

### Verify Prerequisites

```bash
# Check you have Apple Silicon
uname -m
# Should output: arm64

# Check macOS version
sw_vers -productVersion
# Should be 12.0 or later

# Check Rust version
rustc --version
# Should be 1.82.0 or later

# Install Xcode Command Line Tools if needed
xcode-select --install
```

## Installation

### Step 1: Clone and Build

```bash
git clone https://github.com/wildcard/cmdai
cd cmdai

# Build with Metal backend
cargo build --release --features embedded-metal

# This takes 2-5 minutes on first build
```

### Step 2: First Run

```bash
./target/release/cmdai "list all files"
```

**What happens on first run**:
1. Downloads Qwen2.5-Coder-1.5B-Instruct model (~1.1GB)
2. Loads model to Metal GPU
3. Runs inference
4. Presents command for confirmation

**Expected output**:
```
⚡ Initializing Metal GPU (M4 Max)...
📦 Downloading model... (1.1GB)
✓ Model loaded (89ms)
✓ Inference complete (1.2s)

Generated command:
  ls -la

Risk: Safe ✓
Run this command? [y/N]:
```

### Step 3: Verify Metal Acceleration

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/cmdai "test" 2>&1 | grep -i metal

# Should see:
# [INFO] Initializing Candle on device: Metal(0)
```

## Performance Expectations

On M4 Max hardware:

| Metric | First Run | Subsequent Runs |
|--------|-----------|-----------------|
| Model loading | ~2min (download) | <100ms |
| First inference | ~2s | ~1s |
| Memory usage | ~1.5GB | ~1.1GB |
| Throughput | 25-30 tok/s | 25-30 tok/s |

## Troubleshooting

### Issue: Build fails with "Metal not found"

**Solution**:
```bash
# Verify Metal framework exists
ls /System/Library/Frameworks/Metal.framework

# If missing, update macOS to 12.0+
```

### Issue: Slow performance (>5s per inference)

**Possible causes**:
1. **Debug build**: Use `--release` flag
2. **CPU fallback**: Metal not initialized

**Check**:
```bash
# Verify you're using release build
cargo build --release --features embedded-metal

# Check logs for Metal initialization
RUST_LOG=debug ./target/release/cmdai "test" 2>&1 | grep device
```

### Issue: Model download fails

**Solutions**:
```bash
# Manual download
mkdir -p ~/.cache/huggingface/hub
cd ~/.cache/huggingface/hub
wget https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf

# Point cmdai to local file
cmdai --model-path ~/.cache/huggingface/hub/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf "test"
```

### Issue: "Green CI but my build fails"

See the [macOS Testing Guide](MACOS_TESTING.md) for detailed troubleshooting.

## Next Steps

- Read the [User Guide](../README.md) for usage examples
- Check [macOS Testing Guide](MACOS_TESTING.md) if issues arise
- Review [Implementation Plan](../MLX_BACKEND_IMPLEMENTATION_PLAN.md) for technical details

## Advanced: CPU Fallback

If Metal isn't available, cmdai automatically falls back to CPU:

```bash
# Force CPU mode
cargo build --release --features embedded-cpu

# Expect 3-5x slower inference
```

## Understanding the Technology

### Why Candle?

cmdai uses the **Candle** ML framework from HuggingFace because:

- **Performance**: Benchmarks show it's faster than MLX for LLM inference on Apple Silicon
- **Maturity**: Production-ready with 2500+ commits
- **Simplicity**: PyTorch-like API with Rust error handling
- **Built-in GGUF**: Native support for quantized models

### Metal Backend

The Metal backend provides GPU acceleration using Apple's Metal Performance Shaders:

```rust
// Device selection is automatic
let device = Device::new_metal(0)?;  // GPU on Apple Silicon
```

The same inference code works on both CPU and GPU - just the Device changes!

### Quantization

cmdai uses **Q4_K_M quantization** for the model:

- **Size**: ~1.1GB (vs 3GB for full precision)
- **Quality**: <5% accuracy loss for code generation
- **Speed**: Faster inference due to smaller data movement

## Example Session

```bash
# First command
$ ./target/release/cmdai "find large files"
⚡ Using Metal GPU
✓ Model loaded (42ms)
✓ Inference complete (0.9s)

Generated command:
  find . -type f -size +100M -ls

Risk: Safe ✓
Run this command? [y/N]: y

# Second command (model already loaded)
$ ./target/release/cmdai "show git status"
✓ Inference complete (0.8s)

Generated command:
  git status

Risk: Safe ✓
Run this command? [y/N]: y
```

## Benchmarking Your Hardware

Want to see how fast your Mac is?

```bash
# Run with timing
time ./target/release/cmdai "list files"

# Expected times on M4 Max:
# - Model load: <100ms
# - Inference: <2s
# - Total: <2.5s
```

## Configuration

Create `~/.config/cmdai/config.toml` to customize:

```toml
[backend]
primary = "embedded"
enable_fallback = true

[backend.embedded]
model_variant = "metal"  # or "cpu"
temperature = 0.7
max_tokens = 100
```

## Development Mode

For development with fast iteration:

```bash
# Watch mode (recompile on changes)
cargo watch -x 'run --features embedded-metal -- "test"'

# Debug logging
RUST_LOG=trace cargo run --features embedded-metal -- "test"
```

---

**Need help?** Check the [macOS Testing Guide](MACOS_TESTING.md) or open a [GitHub issue](https://github.com/wildcard/cmdai/issues).
