# Qwen2.5-Coder-1.5B-Instruct Model Metadata

This directory contains metadata files for the Qwen2.5-Coder-1.5B-Instruct model used by caro's embedded inference backends.

## Model Information

- **Model**: Qwen2.5-Coder-1.5B-Instruct
- **Quantization**: Q4_K_M (recommended) or Q8_0 (higher quality)
- **Size**: ~1.1GB (Q4_K_M) or ~1.9GB (Q8_0)
- **Architecture**: Qwen2ForCausalLM
- **Parameters**: 1.5 Billion
- **Context Length**: 32,768 tokens
- **Vocabulary Size**: 151,665 tokens

## Source

- **Original Model**: [Qwen/Qwen2.5-Coder-1.5B-Instruct](https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct)
- **GGUF Quantized**: [Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF](https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF)
- **License**: Apache 2.0
- **Developer**: Qwen Team (Alibaba Cloud)

## Files

- `config.json`: Model architecture configuration
- `tokenizer.json`: Tokenizer vocabulary and configuration (7MB)

## Model Download

The actual model weights (GGUF files) are **not included** in this repository due to size constraints (~1.1GB-1.9GB). Users must download the model on first use via one of these methods:

### Automatic Download (Recommended)
```bash
caro "list files"  # Downloads model automatically on first run
```

### Manual Download
```bash
# Q4_K_M (recommended, ~1.1GB)
mkdir -p ~/.cache/caro/models/
curl -L -o ~/.cache/caro/models/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf \
  "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"

# Q8_0 (higher quality, ~1.9GB)
curl -L -o ~/.cache/caro/models/qwen2.5-coder-1.5b-instruct-q8_0.gguf \
  "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q8_0.gguf"
```

## Performance

### MLX GPU (Apple Silicon)
- **Initialization**: <100ms
- **First Token**: <200ms
- **Inference**: <2s for typical shell command
- **Throughput**: ~8-10 tokens/sec (M1 Mac)
- **Memory**: ~1.2GB (model + runtime)

### Candle CPU (Cross-platform)
- **Initialization**: <500ms
- **Inference**: <5s for typical shell command
- **Throughput**: ~2-4 tokens/sec
- **Memory**: ~1.5GB (model + runtime)

## Usage in caro

This model is used by the embedded inference backends:

- **MLX Backend** (`src/backends/embedded/mlx.rs`): Apple Silicon GPU acceleration
- **CPU Backend** (`src/backends/embedded/cpu.rs`): Cross-platform CPU inference

The model is loaded lazily on first inference request to minimize startup time.

## Citation

```bibtex
@article{qwen2.5-coder,
  title={Qwen2.5-Coder Technical Report},
  author={Qwen Team},
  journal={arXiv preprint arXiv:2409.12186},
  year={2024}
}
```

## Links

- **Model Card**: https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct
- **GGUF Variants**: https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF
- **Paper**: https://arxiv.org/abs/2409.12186
- **License**: https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct/blob/main/LICENSE
