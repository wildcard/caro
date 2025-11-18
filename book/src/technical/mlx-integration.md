# MLX Integration

Deep dive into Apple Silicon MLX backend integration.

## Overview

The MLX backend provides optimized inference on Apple Silicon using Apple's MLX framework.

## Architecture

```
cmdai (Rust)
    ↓
  cxx FFI
    ↓
MLX (C++)
    ↓
Metal Performance Shaders
    ↓
Apple Silicon GPU
```

## Performance Characteristics

- **Startup**: < 100ms (lazy loading)
- **Inference**: < 2s on M1 Mac
- **Memory**: Unified memory architecture
- **Quantization**: 4-bit quantized models

## Future Work

- Streaming inference
- Multiple model support
- Dynamic model loading
- Performance benchmarks

For backend implementation, see [Backend Development](../dev-guide/backends.md).
