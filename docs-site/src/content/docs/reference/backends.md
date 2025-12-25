---
title: Backend Reference
description: Complete reference for caro inference backends
---

caro supports multiple inference backends for flexibility across different platforms and use cases.

## Backend Overview

| Backend | Platform | GPU Support | Best For |
|---------|----------|-------------|----------|
| **MLX** | Apple Silicon | Yes | Macs with M1/M2/M3/M4 |
| **Ollama** | All | Varies | Cross-platform, easy setup |
| **vLLM** | Linux/Server | Yes (CUDA) | High-performance serving |

## MLX Backend

The MLX backend provides GPU-accelerated inference on Apple Silicon.

### Requirements

- Apple Silicon Mac (M1/M2/M3/M4)
- macOS 12.0+
- Xcode with Metal compiler

### Configuration

```toml
[backends.mlx]
enabled = true
threads = 4
gpu = true
```

### Performance

| Metric | M1 | M1 Pro | M2 Pro | M4 Pro |
|--------|-----|--------|--------|--------|
| First inference | 2.5s | 2.0s | 1.8s | 1.5s |
| Subsequent | 800ms | 600ms | 500ms | 400ms |
| Memory | 1.2GB | 1.2GB | 1.2GB | 1.2GB |

### Troubleshooting

**Metal compiler not found:**
```bash
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
xcrun --find metal
```

**Build failure:**
```bash
cargo clean
cargo build --release --features embedded-mlx
```

## Ollama Backend

Ollama provides easy local model serving across all platforms.

### Setup

1. Install Ollama:
```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.ai/install.sh | sh
```

2. Start Ollama:
```bash
ollama serve
```

3. Pull a model:
```bash
ollama pull qwen2.5-coder:latest
```

### Configuration

```toml
[backends.ollama]
enabled = true
host = "http://localhost:11434"
model = "qwen2.5-coder:latest"
timeout = 30
```

### Available Models

| Model | Size | Speed | Quality |
|-------|------|-------|---------|
| `qwen2.5-coder:0.5b` | 0.5GB | Fast | Good |
| `qwen2.5-coder:1.5b` | 1.1GB | Medium | Better |
| `qwen2.5-coder:7b` | 4.5GB | Slower | Best |
| `codellama:7b` | 4GB | Medium | Good |

### Performance Tips

- Keep Ollama running in background
- Use smaller models for faster responses
- Increase timeout for larger models

## vLLM Backend

vLLM provides high-performance serving for production deployments.

### Setup

1. Install vLLM:
```bash
pip install vllm
```

2. Start server:
```bash
vllm serve Qwen/Qwen2.5-Coder-1.5B-Instruct \
  --port 8000 \
  --max-model-len 4096
```

### Configuration

```toml
[backends.vllm]
enabled = true
url = "http://localhost:8000"
timeout = 30
```

### Docker Deployment

```yaml
# docker-compose.yml
services:
  vllm:
    image: vllm/vllm-openai:latest
    ports:
      - "8000:8000"
    volumes:
      - ./models:/models
    command: >
      --model Qwen/Qwen2.5-Coder-1.5B-Instruct
      --max-model-len 4096
```

### Performance

| GPUs | Throughput | Latency |
|------|------------|---------|
| 1x A100 | 100+ req/s | <500ms |
| 1x RTX 4090 | 50+ req/s | <800ms |
| 1x RTX 3090 | 30+ req/s | <1200ms |

## Backend Selection

### Automatic Selection

caro automatically selects the best available backend:

1. **MLX** - If on Apple Silicon with MLX support
2. **Ollama** - If Ollama is running locally
3. **vLLM** - If vLLM server is configured

### Manual Selection

Override via command line:
```bash
caro --backend ollama "list files"
caro --backend vllm "list files"
```

Or via environment:
```bash
export CARO_BACKEND=ollama
caro "list files"
```

## Custom Backend

Implement the `ModelBackend` trait for custom backends:

```rust
pub trait ModelBackend: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;
    fn is_available(&self) -> bool;
    fn name(&self) -> &str;
}
```

See the source code for implementation examples.
