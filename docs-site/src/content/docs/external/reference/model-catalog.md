---
title: Model Catalog
description: "Documentation: Model Catalog"
editUrl: false
---
caro supports multiple language models to suit different use cases, from ultra-tiny models for CI/CD to larger models for production use.

## Available Models

| Model | Size | Best For | CI-Suitable | MLX-Optimized |
|-------|------|----------|-------------|---------------|
| **SmolLM 135M** | 82 MB | Ultra-fast testing, extreme resource constraints | ✅ | ❌ |
| **Qwen 0.5B** | 352 MB | CI/CD, fast inference | ✅ | ✅ |
| **TinyLlama 1.1B** | 669 MB | Balanced speed/quality for CI | ✅ | ❌ |
| **StarCoder 1B** | 700 MB | Code-specialized, shell commands | ✅ | ❌ |
| **Qwen 1.5B** (default) | 1117 MB | Best balance for local use | ❌ | ✅ |
| **Phi-2 2.7B** | 1560 MB | Excellent code understanding | ❌ | ❌ |
| **Mistral 7B Instruct** | 3520 MB | Highest quality, production | ❌ | ❌ |

## Model Selection

### For Local Development
**Default**: Qwen 1.5B (1.1GB)
- Best balance of size and quality
- MLX-optimized for Apple Silicon
- Good code understanding

### For GitHub Actions CI
**Recommended**: Qwen 0.5B (352MB) or SmolLM 135M (82MB)
- Fast downloads (~30s vs 2min for default)
- Lower memory usage
- Still produces reasonable results

### For Testing
**Recommended**: SmolLM 135M (82MB)
- Fastest download and inference
- Minimal memory footprint
- Sufficient for unit/integration tests

## Usage

### Rust API

```rust
use caro::{ModelLoader, ModelCatalog};

// Use default model (Qwen 1.5B)
let loader = ModelLoader::new()?;

// Use smallest model for CI
let loader = ModelLoader::with_smallest_model()?;

// Use specific model by ID
let loader = ModelLoader::with_model("qwen-0.5b-q4")?;

// List all available models
let models = ModelLoader::list_models();
for model in models {
    println!("{} - {} MB", model.name, model.size_mb);
}

// List CI-suitable models only
let ci_models = ModelLoader::list_ci_models();
```

### Environment Variable

Set `CARO_MODEL` to use a specific model:

```bash
# Use smallest model
export CARO_MODEL=smollm-135m-q4

# Use Qwen 0.5B for CI
export CARO_MODEL=qwen-0.5b-q4

# Use default
unset CARO_MODEL
```

### GitHub Actions Configuration

For CI workflows, use a smaller model to reduce test time:

```yaml
- name: Run E2E tests with small model
  run: cargo test --test e2e_cli_tests
  env:
    CARO_MODEL: qwen-0.5b-q4  # Fast download, good quality
```

Or for fastest tests:

```yaml
- name: Run unit tests with tiny model
  run: cargo test --lib
  env:
    CARO_MODEL: smollm-135m-q4  # Ultra-fast
```

## Model Details

### SmolLM 135M
- **Repository**: HuggingFaceTB/SmolLM-135M-Instruct-GGUF
- **Quantization**: Q4_K_M
- **Size**: 82 MB
- **Use Case**: Testing, extreme resource constraints
- **Pros**: Fastest download, minimal memory
- **Cons**: Lower quality outputs

### Qwen2.5-Coder 0.5B
- **Repository**: Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF
- **Quantization**: Q4_K_M
- **Size**: 352 MB
- **Use Case**: CI/CD with quality requirements
- **Pros**: Good balance, MLX-optimized, code-specialized
- **Cons**: Slightly larger than SmolLM

### TinyLlama 1.1B
- **Repository**: TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF
- **Quantization**: Q4_K_M
- **Size**: 669 MB
- **Use Case**: Local development, testing
- **Pros**: Widely tested, good quality
- **Cons**: Larger CI download time

### StarCoder 1B
- **Repository**: TheBloke/starcoderbase-1b-GGUF
- **Quantization**: Q4_K_M
- **Size**: 700 MB
- **Use Case**: Code generation, shell commands
- **Pros**: Code-specialized, good for command generation
- **Cons**: Not chat-optimized

### Qwen2.5-Coder 1.5B (Default)
- **Repository**: Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF
- **Quantization**: Q4_K_M
- **Size**: 1117 MB
- **Use Case**: Local development, production
- **Pros**: Best balance, MLX-optimized, excellent code understanding
- **Cons**: Too large for CI

### Phi-2 2.7B
- **Repository**: TheBloke/phi-2-GGUF
- **Quantization**: Q4_K_M
- **Size**: 1560 MB
- **Use Case**: High-quality code understanding
- **Pros**: Excellent reasoning, Microsoft-trained
- **Cons**: Large size, slower inference

### Mistral 7B Instruct
- **Repository**: TheBloke/Mistral-7B-Instruct-v0.2-GGUF
- **Quantization**: Q3_K_M
- **Size**: 3520 MB
- **Use Case**: Production, highest quality needed
- **Pros**: Best quality, instruction following
- **Cons**: Large size, slower inference, not CI-suitable

## Performance Comparison

Approximate inference times on Apple M4 Pro:

| Model | First Inference | Subsequent | Memory |
|-------|----------------|------------|--------|
| SmolLM 135M | ~0.5s | ~0.1s | ~200MB |
| Qwen 0.5B | ~1.0s | ~0.3s | ~500MB |
| TinyLlama 1.1B | ~1.5s | ~0.4s | ~800MB |
| Qwen 1.5B | ~2.5s | ~0.6s | ~1.2GB |
| Phi-2 2.7B | ~3.5s | ~1.0s | ~2GB |
| Mistral 7B | ~8s | ~2.5s | ~4GB |

*Times may vary based on hardware and MLX optimization availability*

## CI Recommendations

### Fast Tests (< 1 minute)
```yaml
env:
  CARO_MODEL: smollm-135m-q4
```

### Balanced Tests (< 2 minutes)
```yaml
env:
  CARO_MODEL: qwen-0.5b-q4
```

### Quality Tests (< 5 minutes)
```yaml
env:
  CARO_MODEL: tinyllama-1.1b-q4
```

## Model Selection Guide

```
Need fastest CI?          → SmolLM 135M (82MB)
Need quality + speed?     → Qwen 0.5B (352MB)
Need code specialization? → StarCoder 1B (700MB)
Local development?        → Qwen 1.5B (1.1GB, default)
Need best quality?        → Mistral 7B (3.5GB)
```

## Programmatic Model Selection

```rust
use caro::{ModelCatalog, ModelSize};

// Get all CI-suitable models
let ci_models = ModelCatalog::ci_models();

// Get models by size category
let tiny_models = ModelCatalog::by_size(ModelSize::Tiny);
let small_models = ModelCatalog::by_size(ModelSize::Small);

// Get MLX-optimized models (Apple Silicon)
let mlx_models = ModelCatalog::mlx_models();

// Get model by ID
if let Some(model) = ModelCatalog::by_id("qwen-0.5b-q4") {
    println!("Found: {}", model);
}
```

## Adding Custom Models

To add your own model, edit `src/model_catalog.rs` and add a new `ModelInfo` entry. Ensure the model is:
1. GGUF format
2. Available on Hugging Face
3. Suitable for command generation tasks

See the existing model definitions for examples.
