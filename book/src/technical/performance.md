# Performance Optimization

Performance optimization strategies in cmdai.

## Goals

- **Startup time**: < 100ms cold start
- **Inference time**: < 2s on Apple Silicon
- **Binary size**: < 50MB
- **Memory usage**: < 200MB

## Optimization Techniques

### Binary Size

```toml
[profile.release]
opt-level = 'z'      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
strip = true         # Strip debug symbols
panic = 'abort'      # Smaller panic handler
```

### Startup Performance

- Lazy loading of backends
- Minimal dependencies
- Efficient JSON parsing
- No unnecessary allocations

### Inference Performance

- Model quantization (4-bit)
- Metal GPU acceleration
- Batched operations
- Connection pooling for HTTP backends

## Benchmarks

Performance benchmarks coming soon.

See [Architecture](../dev-guide/architecture.md) for implementation details.
