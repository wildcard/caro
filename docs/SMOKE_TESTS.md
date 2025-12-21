# Running Smoke Tests Locally

This document explains how to run the CI smoke tests locally before pushing to GitHub.

## Overview

The project uses a **two-tier testing strategy**:

- **Tier A (Smoke Tests)**: Fast, deterministic tests that run on every PR using small models
- **Tier B (Extended Tests)**: Comprehensive tests that run nightly with multiple models

## Quick Start

### Run Tier A Smoke Tests

```bash
# Use smallest model (82MB, fastest)
CARO_MODEL=smollm-135m-q4 cargo test --test inference_smoke --verbose -- --test-threads=1

# Or use Qwen 0.5B (352MB, MLX-optimized on macOS)
CARO_MODEL=qwen-0.5b-q4 cargo test --test inference_smoke --verbose -- --test-threads=1
```

### Run with Deterministic Settings

```bash
# Force single-threaded for reproducibility
export OMP_NUM_THREADS=1
export OPENBLAS_NUM_THREADS=1
export MKL_NUM_THREADS=1
export CARO_MODEL=smollm-135m-q4

cargo test --test inference_smoke --verbose -- --test-threads=1
```

## Test Structure

### Smoke Tests (`tests/smoke/inference_smoke.rs`)

Six core tests validate the inference pipeline:

1. **`smoke_test_model_download`** - Verifies model downloads correctly
2. **`smoke_test_model_load`** - Validates model loads without errors
3. **`smoke_test_basic_inference`** - Runs single inference, checks output structure
4. **`smoke_test_inference_determinism`** - Verifies temp=0 produces consistent results
5. **`smoke_test_output_structure`** - Validates `GeneratedCommand` structure
6. **`smoke_test_inference_performance`** - Checks inference completes within timeouts

### Expected Behavior

**Determinism**: With `temperature=0` and single-threading, inference should be **reasonably deterministic** (≥70% token overlap across runs). Perfect determinism is not guaranteed due to:
- CPU scheduling variations
- BLAS library implementation differences
- Floating-point rounding in different environments

**Performance**:
- Model load: < 60s on CI runners
- Cold inference: < 30s
- Warm inference: < 10s

These are generous timeouts for CI; local runs will be faster.

## Recommended Models for Smoke Tests

| Model | Size | Speed | Use Case |
|-------|------|-------|----------|
| `smollm-135m-q4` | 82MB | ⚡⚡⚡ | **Primary CI model**, fastest download |
| `qwen-0.5b-q4` | 352MB | ⚡⚡ | **macOS MLX-optimized**, best quality |
| `tinyllama-1.1b-q4` | 669MB | ⚡ | Extended testing only |

## Adding a New Model

### 1. Add to `src/model_catalog.rs`

```rust
pub static NEW_MODEL_Q4: ModelInfo = ModelInfo {
    id: "new-model-q4",
    name: "New Model Q4",
    hf_repo: "org/model-name-GGUF",
    filename: "model-file.Q4_K_M.gguf",
    size_mb: 250,
    size_category: ModelSize::Small,
    description: "Description here",
    mlx_optimized: false,
    ci_suitable: true,  // If < 500MB
};
```

### 2. Add to `ALL_MODELS` array

```rust
static ALL_MODELS: &[&ModelInfo] = &[
    &SMOLLM_135M_Q4,
    &NEW_MODEL_Q4,  // Add here
    // ...
];
```

### 3. Test locally

```bash
CARO_MODEL=new-model-q4 cargo test --test inference_smoke --verbose
```

### 4. Update CI matrix (for Tier B only)

Edit `.github/workflows/ci.yml`:

```yaml
extended-model-tests:
  matrix:
    model:
      # ... existing models
      - id: new-model-q4
        name: New Model Q4
        size_mb: 250
```

**Do NOT add to Tier A smoke tests** unless the model is:
- < 150MB
- Widely available and stable
- Significantly faster than current options

## Troubleshooting

### Test fails with "Model download failed"

**Solution**: Check internet connection and Hugging Face availability:

```bash
# Test HF access
curl -I https://huggingface.co/

# Clear cache and retry
rm -rf ~/.cache/caro/models  # Linux
rm -rf ~/Library/Caches/caro/models  # macOS

cargo test --test inference_smoke -- smoke_test_model_download
```

### Test fails with "Inference too slow"

**Solution**: Ensure you're not running other heavy processes:

```bash
# Check CPU load
top

# Run with more verbose logging
RUST_LOG=debug cargo test --test inference_smoke --verbose
```

### Tests show low determinism (<70% overlap)

**Possible causes**:
1. Model wasn't actually using `temperature=0`
2. Multi-threading enabled (check `OMP_NUM_THREADS`)
3. Different BLAS library versions
4. Model doesn't support deterministic inference well

**Solution**:
```bash
# Force single-threaded
export OMP_NUM_THREADS=1
export OPENBLAS_NUM_THREADS=1
export MKL_NUM_THREADS=1

# Try a different model
CARO_MODEL=qwen-0.5b-q4 cargo test --test inference_smoke -- smoke_test_inference_determinism
```

### macOS-specific: Metal/MLX errors

**Solution**:
```bash
# Ensure MLX backend is built correctly
cargo build --features embedded-mlx

# Check Metal support
system_profiler SPDisplaysDataType
```

## CI Workflow Behavior

### On Pull Request / Push

**Runs**:
- Lint & format checks (no model)
- Tier A smoke tests (`smollm-135m-q4` on Ubuntu/macOS)
- Unit tests (all platforms)
- Build checks (all platforms)

**Does NOT run**:
- Extended model matrix (Tier B)
- Windows smoke tests (for speed)

### On Schedule (Nightly)

**Runs**:
- All Tier A tests
- Extended model matrix (all 4 models on macOS)
- Comprehensive E2E tests per model

### On Manual Trigger

Use GitHub Actions UI → "Run workflow" to test specific scenarios.

## Best Practices

1. **Always run smoke tests before pushing**:
   ```bash
   make smoke-test  # If makefile exists
   # or
   CARO_MODEL=smollm-135m-q4 cargo test --test inference_smoke
   ```

2. **Check model cache usage**:
   ```bash
   du -sh ~/.cache/caro/models      # Linux
   du -sh ~/Library/Caches/caro/models  # macOS
   ```

3. **Clean cache periodically**:
   ```bash
   # Models can accumulate over time
   rm -rf ~/.cache/caro/models/*
   ```

4. **Use verbose output for debugging**:
   ```bash
   RUST_LOG=debug cargo test --test inference_smoke --verbose -- --nocapture
   ```

## Performance Benchmarks (Reference)

Approximate times on M4 Pro MacBook (your mileage may vary):

| Operation | SmolLM 135M | Qwen 0.5B |
|-----------|-------------|-----------|
| Download (first time) | ~15s | ~45s |
| Model load | ~2s | ~5s |
| Cold inference | ~0.5s | ~1.5s |
| Warm inference | ~0.2s | ~0.6s |

On GitHub Actions Ubuntu runners (CPU-only), expect 2-3x slower.

## Further Reading

- [Model Catalog Documentation](../MODEL_CATALOG.md)
- [Multi-Model CI Implementation](../MULTI_MODEL_CI_IMPLEMENTATION.md)
- [GitHub Actions Workflow](.github/workflows/ci.yml)
