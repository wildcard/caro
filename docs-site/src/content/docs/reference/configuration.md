---
title: Configuration Reference
description: Complete configuration options for caro
---

This document covers all configuration options available in caro.

## Configuration Location

caro stores configuration in platform-specific locations:

| Platform | Config Path |
|----------|-------------|
| **macOS** | `~/Library/Application Support/caro/config.toml` |
| **Linux** | `~/.config/caro/config.toml` |
| **Windows** | `%APPDATA%\caro\config.toml` |

## Configuration File Format

Configuration uses TOML format:

```toml
# caro configuration file

[general]
# Default backend to use
backend = "mlx"

# Enable colored output
color = true

# Show safety warnings
safety_warnings = true

[model]
# Model to use for inference
name = "qwen2.5-coder-1.5b-instruct"

# Model format
format = "gguf"

# Quantization level
quantization = "q4_k_m"

[backends.mlx]
# MLX-specific settings
enabled = true
threads = 4

[backends.ollama]
# Ollama backend settings
enabled = false
host = "http://localhost:11434"
model = "qwen2.5-coder:latest"

[backends.vllm]
# vLLM backend settings
enabled = false
url = "http://localhost:8000"
```

## Configuration Options

### General Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `backend` | string | `"mlx"` | Default inference backend |
| `color` | bool | `true` | Enable colored terminal output |
| `safety_warnings` | bool | `true` | Show safety level warnings |
| `confirm_execution` | bool | `true` | Require confirmation before execution |

### Model Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `name` | string | `"qwen2.5-coder-1.5b-instruct"` | Model name |
| `format` | string | `"gguf"` | Model file format |
| `quantization` | string | `"q4_k_m"` | Quantization level |
| `cache_dir` | string | (auto) | Custom model cache directory |

### Backend: MLX

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `true` | Enable MLX backend |
| `threads` | int | `4` | Number of CPU threads |
| `gpu` | bool | `true` | Use GPU acceleration |

### Backend: Ollama

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `false` | Enable Ollama backend |
| `host` | string | `"http://localhost:11434"` | Ollama server URL |
| `model` | string | `"qwen2.5-coder:latest"` | Ollama model name |
| `timeout` | int | `30` | Request timeout in seconds |

### Backend: vLLM

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `false` | Enable vLLM backend |
| `url` | string | `"http://localhost:8000"` | vLLM server URL |
| `timeout` | int | `30` | Request timeout in seconds |

## Environment Variables

Configuration can also be set via environment variables:

```bash
# Override default backend
export CARO_BACKEND=ollama

# Custom cache directory
export CARO_CACHE_DIR=~/custom/cache

# Enable debug logging
export RUST_LOG=debug

# Disable color output
export NO_COLOR=1

# Ollama host
export OLLAMA_HOST=http://localhost:11434
```

## Command-Line Overrides

Command-line flags override configuration file settings:

```bash
# Use specific backend
caro --backend ollama "list files"

# Disable color output
caro --no-color "list files"

# Skip confirmation
caro --yes "list files"

# Verbose output
caro --verbose "list files"
```

## Cache Directory

Model cache location:

| Platform | Cache Path |
|----------|------------|
| **macOS** | `~/Library/Caches/caro/models/` |
| **Linux** | `~/.cache/caro/models/` |
| **Windows** | `%LOCALAPPDATA%\caro\cache\` |

### Managing Cache

```bash
# Show cache location and size
caro cache info

# Clear model cache
caro cache clear

# Download specific model
caro cache download qwen2.5-coder-1.5b-instruct
```

## Example Configurations

### Development Setup

```toml
[general]
backend = "mlx"
color = true
safety_warnings = true

[backends.mlx]
enabled = true
```

### Server Deployment

```toml
[general]
backend = "vllm"
color = false
confirm_execution = false

[backends.vllm]
enabled = true
url = "http://inference-server:8000"
timeout = 60
```

### Multi-Backend Setup

```toml
[general]
backend = "ollama"  # Primary

[backends.mlx]
enabled = true

[backends.ollama]
enabled = true
host = "http://localhost:11434"
model = "qwen2.5-coder:latest"

[backends.vllm]
enabled = true
url = "http://backup-server:8000"
```
