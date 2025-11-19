# Feature Flags Quick Reference

## Available Features

### `embedded-cpu` (Default)
**Description**: Cross-platform CPU-based model inference using Candle
**Platforms**: Linux, macOS (Intel/ARM), Windows
**Dependencies**: `candle-core`, `candle-transformers`, `tokenizers`
**Use Case**: Baseline inference on any platform

```bash
cargo build --features embedded-cpu
```

---

### `embedded-metal`
**Description**: GPU-accelerated inference using Apple Metal framework
**Platforms**: macOS Apple Silicon (aarch64) **ONLY**
**Dependencies**: `candle-core/metal`, `candle-nn`, `metal`, `accelerate-src`
**Use Case**: High-performance inference on M1/M2/M3/M4 Macs

```bash
# macOS Apple Silicon only
cargo build --features embedded-metal
```

---

### `remote-backends`
**Description**: HTTP API clients for remote inference (vLLM, Ollama)
**Platforms**: All
**Dependencies**: `reqwest`, `tokio/net`
**Use Case**: Connect to remote inference servers

```bash
cargo build --features remote-backends
```

---

### `mock-backend`
**Description**: Mock backend for testing without real models
**Platforms**: All
**Dependencies**: None
**Use Case**: Testing, development without model downloads

```bash
cargo test --features mock-backend
```

---

### `embedded`
**Description**: Alias for `embedded-cpu` (convenience)
**Platforms**: All
**Use Case**: Simple embedded inference flag

```bash
cargo build --features embedded
```

---

### `full`
**Description**: All features enabled
**Platforms**: Depends on hardware
**Dependencies**: All of the above
**Use Case**: Maximum capability build

```bash
# macOS Apple Silicon - includes Metal
cargo build --features full

# Other platforms - excludes Metal (will fail if Metal dependencies can't resolve)
cargo build --features remote-backends,embedded-cpu
```

---

## Common Build Scenarios

### Development on Apple Silicon Mac
```bash
# Full feature set with Metal GPU acceleration
cargo build --release --features full

# Or explicitly:
cargo build --release --features embedded-metal,embedded-cpu,remote-backends
```

### Development on Intel Mac
```bash
# CPU + remote backends (no Metal)
cargo build --release --features embedded-cpu,remote-backends
```

### Development on Linux/Windows
```bash
# CPU + remote backends (no Metal)
cargo build --release --features embedded-cpu,remote-backends
```

### CI/CD Cross-Platform Build
```bash
# macOS aarch64
cargo build --release --target aarch64-apple-darwin --features embedded-metal,embedded-cpu

# macOS x86_64
cargo build --release --target x86_64-apple-darwin --features embedded-cpu

# Linux
cargo build --release --target x86_64-unknown-linux-gnu --features embedded-cpu

# Windows
cargo build --release --target x86_64-pc-windows-msvc --features embedded-cpu
```

---

## Testing with Features

### Run all tests with CPU backend
```bash
cargo test --features embedded-cpu
```

### Run all tests with Metal backend (macOS aarch64 only)
```bash
cargo test --features embedded-metal
```

### Run specific test suite
```bash
# CPU backend tests
cargo test --features embedded-cpu cpu

# Metal backend tests (macOS aarch64 only)
cargo test --features embedded-metal metal

# Remote backend tests
cargo test --features remote-backends remote
```

### Run contract tests
```bash
# Metal backend contract tests (macOS aarch64 only)
cargo test --features embedded-metal --test metal_backend_contract
```

---

## Feature Combinations

| Combination | Valid? | Platforms | Notes |
|-------------|--------|-----------|-------|
| `embedded-cpu` | ✅ | All | Default, always safe |
| `embedded-metal` | ✅ | macOS ARM64 | Apple Silicon only |
| `embedded-metal` | ❌ | macOS Intel, Linux, Windows | Will fail to build |
| `embedded-cpu,embedded-metal` | ✅ | macOS ARM64 | Both backends available |
| `embedded-cpu,remote-backends` | ✅ | All | Common configuration |
| `full` | ✅ | macOS ARM64 | All features |
| `full` | ⚠️ | Other platforms | Metal dependency may cause issues |
| `default` (no flags) | ✅ | All | Defaults to `embedded-cpu` |

---

## Conditional Compilation

In source code, use feature gates:

```rust
#[cfg(feature = "embedded-cpu")]
use crate::backends::embedded::cpu::CpuBackend;

#[cfg(all(feature = "embedded-metal", target_os = "macos", target_arch = "aarch64"))]
use crate::backends::embedded::metal::MetalBackend;

#[cfg(feature = "remote-backends")]
use crate::backends::remote::{VllmBackend, OllamaBackend};
```

---

## Binary Size Impact

| Features | Approximate Size | Notes |
|----------|------------------|-------|
| `embedded-cpu` | ~30-40 MB | Without model |
| `embedded-metal` | ~35-45 MB | Metal framework overhead |
| `embedded-cpu,embedded-metal` | ~40-50 MB | Both backends |
| `full` | ~45-50 MB | All features |
| `remote-backends` only | ~15-20 MB | Minimal, no local inference |

*Note: Sizes exclude embedded models. With model: +100-500 MB depending on model size.*

---

## Migration from `embedded-mlx`

**Old feature flag** (removed):
```toml
embedded-mlx = ["cxx", "mlx-rs"]
```

**New feature flag**:
```toml
embedded-metal = ["candle-core/metal", "candle-transformers", "candle-nn", "metal", "accelerate-src", "tokenizers"]
```

**What changed**:
- ❌ Removed: `mlx-rs` dependency (FFI binding to MLX)
- ❌ Removed: `cxx` dependency (C++ FFI bridge)
- ✅ Added: Candle with native Metal support
- ✅ Added: Direct Metal framework bindings
- ✅ Added: Accelerate framework integration

**Migration**:
1. Replace `embedded-mlx` with `embedded-metal` in build commands
2. Update CI/CD pipelines to use `embedded-metal`
3. No source code changes needed (backends abstracted)

---

## Quick Checklist

Before committing changes:
- [ ] Code builds with `embedded-cpu` on your platform
- [ ] Code builds with `embedded-metal` on macOS Apple Silicon (if applicable)
- [ ] Tests pass: `cargo test --features embedded-cpu`
- [ ] Clippy clean: `cargo clippy --features full -- -D warnings`
- [ ] Format check: `cargo fmt --check`
- [ ] Binary size < 50MB (without model): `ls -lh target/release/cmdai`

---

**Last Updated**: 2025-11-19
