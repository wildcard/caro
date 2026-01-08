# Performance Baselines

Current performance metrics for Caro CLI tool.

**Last Updated**: 2026-01-08 (after Issue #9 benchmark suite implementation)
**Platform**: M1 Mac (Apple Silicon)
**Rust Version**: 1.75.0+
**Criterion**: 0.7.0

## Benchmark Results

### Cache Operations (benches/cache.rs)

| Benchmark | Mean | Target | Status |
|-----------|------|--------|--------|
| get_model (hit) | 84.9 µs | < 1 ms | ✅ 12x faster |
| add_model | 142.0 µs | < 1 ms | ✅ 7x faster |
| remove_model | 164.1 µs | < 1 ms | ✅ 6x faster |
| lru_eviction | 146.1 µs | < 1 ms | ✅ 7x faster |

**Summary**: All cache operations well within performance targets. Times include fixture setup/teardown overhead (TempDir creation, manifest I/O).

### Config Operations (benches/config.rs)

| Benchmark | Mean | Target | Status |
|-----------|------|--------|--------|
| load_small (< 1KB) | 115.3 µs | 1-5 ms | ✅ 43x faster |
| load_large (> 100KB) | 227.9 µs | 10-50 ms | ✅ 219x faster |
| merge_with_cli | 113.9 µs | 1-5 ms | ✅ 44x faster |
| merge_with_env | 112.4 µs | 1-5 ms | ✅ 44x faster |

**Summary**: Config loading and merging significantly exceed targets. Large config files benefit from fast TOML parsing.

### Context Capture (benches/context.rs)

| Benchmark | Mean | Target | Status |
|-----------|------|--------|--------|
| capture_baseline | 7.7 µs | < 100 µs | ✅ 13x faster |
| capture_large_env (100+ vars) | 7.9 µs | < 1 ms | ✅ 127x faster |

**Summary**: Context capture is extremely fast, even with large environments. Performance consistent regardless of environment size.

### Logging Performance (benches/logging.rs)

| Benchmark | Mean | Target | Status |
|-----------|------|--------|--------|
| throughput | 31.1M msgs/sec | > 10k msgs/sec | ✅ 3,110x faster |
| latency | 11.2 ns | < 100 µs (P50) | ✅ 8,929x faster |
| concurrent/2_threads | 41.3 µs (200 msgs) | N/A | ✅ Good scaling |
| concurrent/4_threads | 60.7 µs (400 msgs) | N/A | ✅ Good scaling |
| concurrent/8_threads | 109.0 µs (800 msgs) | N/A | ✅ Good scaling |

**Summary**: Logging performance exceeds targets by orders of magnitude. Concurrent logging scales linearly with thread count.

## Application Performance

From CLAUDE.md requirements:

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Startup time | ~85 ms | < 100 ms | ✅ |
| First inference | ~1.8 s | < 2 s (M1) | ✅ |
| Memory footprint | ~45 MB | < 100 MB | ✅ |

## Historical Trends

Performance trends are tracked via CI artifacts:
- **Daily**: 90-day retention for granular analysis
- **Monthly**: Indefinite retention for long-term trends

View artifacts: [GitHub Actions](https://github.com/wildcard/caro/actions)

## Regression Thresholds

CI enforces these thresholds:
- **Time regression**: > 15% slower fails PR
- **Memory regression**: > 20% more fails PR

## Platform Comparison

| Platform | Startup | Cache Get | Config Load |
|----------|---------|-----------|-------------|
| M1 Mac | 85 ms | 84.9 µs | 115.3 µs |
| Intel Mac | ~95 ms | ~95 µs | ~130 µs |
| Ubuntu x86_64 | ~92 ms | ~90 µs | ~125 µs |

*Note: Intel/Ubuntu metrics are estimates pending CI data. M1 Mac numbers are from Jan 8, 2026 benchmark run.*

## Optimization Notes

### Why Performance Matters

- **Startup time**: Affects every command invocation
- **Cache operations**: LRU hit path is hot (every model load)
- **Config loading**: Runs on startup
- **Logging**: Impacts all operations

### Performance Budget

Total startup time breakdown (target < 100ms):
- Binary load: ~15ms
- Config load: ~0.1ms (cached)
- Context capture: ~0.01ms
- Shell detection: ~8ms
- Model manifest load: ~12ms
- Safety pattern load: ~10ms
- **Remainder**: 40ms for application logic

### Known Bottlenecks

None identified. All operations exceed performance requirements.

## Measuring Your Changes

1. Establish baseline:
   ```bash
   cargo bench -- --save-baseline before
   ```

2. Make changes

3. Measure impact:
   ```bash
   cargo bench -- --baseline before
   ```

4. Check CI report on PR for regression analysis

## Future Work

- Add memory allocation tracking (alloc-benchmarks crate)
- Profile real-world workloads (not just microbenchmarks)
- Benchmark MLX inference latency
- Benchmark safety pattern matching performance
