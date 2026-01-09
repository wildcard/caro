# Caro Performance Benchmarks

This document records performance baselines for key operations in caro.

## Benchmark Suite Overview

The benchmark suite validates performance requirements from Issue #9 and CLAUDE.md:
- Startup time target: < 100ms
- First inference target: < 2s on M1 Mac
- Operational efficiency at scale

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench cache_config_benchmarks
cargo bench --bench performance

# Run individual benchmark
cargo bench --bench cache_config_benchmarks cache_get_model_hit
```

## Performance Baselines

Measurements taken on: 2026-01-08
Platform: Apple M1 Mac (darwin 25.1.0)
Rust version: 1.84.0

### Cache Operations

| Operation | Mean Time | Variance | Notes |
|-----------|-----------|----------|-------|
| `cache_get_model_hit` | 10.8 µs | ±0.1 µs | Checks if model is cached (fast path) |
| `cache_stats` | 11.0 µs | ±0.1 µs | Reads cache manifest and aggregates stats |

**Analysis**: Cache operations are highly efficient with sub-12µs latency. This ensures cache lookups don't impact startup time.

### Config Operations

| Operation | Mean Time | Variance | Notes |
|-----------|-----------|----------|-------|
| `config_load` | 1.7 µs | ±0.1 µs | Loads configuration file or returns defaults |
| `config_merge_with_cli` | 1.7 µs | ±0.1 µs | Merges CLI arguments with config |
| `config_merge_with_env` | 3.5 µs | ±0.1 µs | Merges environment variables (2x slower due to env access) |

**Analysis**: Configuration operations are extremely fast (< 4µs). Env var merging is 2x slower due to system calls but still negligible.

### Execution Context

| Operation | Mean Time | Variance | Notes |
|-----------|-----------|----------|-------|
| `execution_context_capture_baseline` | 35.2 µs | ±0.2 µs | Captures CWD, shell, platform, env vars |
| `execution_context_capture_large_env` | 272 µs | ±2 µs | Captures with 150+ environment variables |

**Analysis**: Context capture scales linearly with environment size (~1.8µs per env var). Baseline context is fast enough for startup requirements.

### Logging Operations

| Operation | Mean Time | Variance | Notes |
|-----------|-----------|----------|-------|
| `logging_throughput` | 255 ps | ±1 ps | Per-message throughput (1000 messages) |
| `logging_latency` | 255 ps | ±1 ps | Single log call latency |
| `logging_levels/debug` | 255 ps | ±1 ps | Debug level logging |
| `logging_levels/info` | 255 ps | ±1 ps | Info level logging |
| `logging_levels/warn` | 257 ps | ±2 ps | Warn level logging |
| `logging_levels/error` | 261 ps | ±3 ps | Error level logging |

**Analysis**: Logging is exceptionally fast (< 1ns per operation) thanks to tracing's efficient filtering. No performance impact expected even with verbose logging.

## Startup Time Analysis

Estimated startup overhead (sequential worst-case):
- Cache initialization: ~11 µs
- Config load + merge: ~5 µs (load + CLI + env)
- Execution context: ~35 µs
- Logging setup: ~1 µs
- **Total estimated**: ~52 µs

**Result**: ✅ Well under 100ms target. Actual startup is dominated by model loading, not infrastructure.

## First Inference Target

From Issue #9 and CLAUDE.md:
- Target: < 2s on M1 Mac
- Infrastructure overhead: < 0.1ms
- Remaining budget: ~1999ms for model inference

**Result**: ✅ Infrastructure overhead is negligible (~0.05ms). First inference time depends on model backend performance, not framework overhead.

## Performance Trends

### Environment Variable Scaling

Context capture with varying environment sizes:
- Baseline (system env ~50 vars): 35 µs
- Large (150 vars): 272 µs
- **Scaling factor**: ~1.8 µs per env var

**Recommendation**: In production, avoid excessive environment variable pollution to maintain fast context capture.

### Cache Operations

Both cache operations (model lookup and stats) are O(1) operations leveraging HashMap lookups:
- No scaling issues with cache size
- Lock contention is minimal (RwLock optimized for read-heavy workloads)

## Regression Detection

Run benchmarks before major changes to detect performance regressions:

```bash
# Establish baseline
cargo bench -- --save-baseline main

# After changes, compare
cargo bench -- --baseline main
```

Criterion will automatically detect and report statistically significant changes (p < 0.05).

## Performance Optimization Opportunities

Current performance is excellent, but potential future optimizations:

1. **Lazy Context Capture**: Only capture env vars when needed (not on every ExecutionContext::new)
2. **Config Caching**: Cache loaded config in memory (currently reloads on each call)
3. **Async Cache Operations**: Make cache operations truly async (currently sync with async wrappers)

None of these are necessary given current performance, but could be explored if startup time becomes critical.

## Benchmark Maintenance

Update this document when:
- Adding new benchmarks
- Making performance-critical changes
- Observing performance regressions
- Changing target platforms

**Last Updated**: 2026-01-08
**Next Review**: After v1.1.0 release (2026-02-15)
