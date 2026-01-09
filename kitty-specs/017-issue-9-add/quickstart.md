# Quickstart Guide: Running Benchmarks

**Audience**: Developers working on Caro
**Last Updated**: 2026-01-08
**Related**: [spec.md](spec.md), [plan.md](plan.md), [data-model.md](data-model.md)

## Overview

This guide shows you how to run, interpret, and compare performance benchmarks for Caro. The benchmark suite validates performance requirements and detects regressions automatically.

---

## Quick Reference

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark area
cargo bench --bench cache
cargo bench --bench config
cargo bench --bench context
cargo bench --bench logging

# Compare against baseline
cargo bench -- --baseline main

# Save current results as baseline
cargo bench -- --save-baseline my-optimization

# Filter by pattern
cargo bench lru_eviction

# Get help
cargo bench -- --help
```

---

## Prerequisites

### Install Criterion (Already in Dev Dependencies)

```toml
# Cargo.toml - already configured
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "cache"
harness = false
```

### Check Rust Version

```bash
rustc --version  # Should be 1.75.0 or higher
```

---

## Running Benchmarks

### 1. Run All Benchmarks

```bash
cargo bench
```

**What happens**:
- Runs all benchmarks in `benches/` directory
- Takes 5-10 minutes for full suite
- Generates HTML reports in `target/criterion/`
- Saves results for future comparison

**Output**:
```
cache/get_model         time:   [45.1 ns 45.2 ns 45.3 ns]
cache/add_model         time:   [234 ns 236 ns 238 ns]
config/load_small       time:   [2.28 ms 2.30 ms 2.32 ms]
...
```

### 2. Run Specific Benchmark Area

**For cache benchmarks only**:
```bash
cargo bench --bench cache
```

**When to use**:
- Changed code in `src/cache/`
- Faster iteration during development (1-2 minutes)
- Focused validation of specific module

**Other areas**:
```bash
cargo bench --bench config   # Config operations (2 min)
cargo bench --bench context  # Execution context (1 min)
cargo bench --bench logging  # Logging performance (3 min)
```

### 3. Run Single Benchmark

```bash
cargo bench lru_eviction  # Pattern matching
cargo bench get_model     # Runs cache/get_model
```

**When to use**:
- Testing specific optimization (30 seconds)
- Quick validation during TDD cycle

---

## Comparing Against Baseline

### Save Current Results as Baseline

```bash
# Before making changes
cargo bench -- --save-baseline before-optimization
```

### Make Your Changes

```bash
# Edit code, e.g., optimize cache eviction
vim src/cache/lru.rs
```

### Compare New Results

```bash
# After making changes
cargo bench -- --baseline before-optimization
```

**Output with comparison**:
```
cache/lru_eviction
  time:   [890 ns 901 ns 912 ns]
  change: [-32.1% -30.5% -28.9%] (p = 0.00 < 0.05)
  Performance has improved.
```

**Interpreting results**:
- **change**: Performance delta vs baseline
  - Negative = improvement (faster)
  - Positive = regression (slower)
- **p-value**: Statistical significance
  - `p < 0.05` = change is statistically significant
  - `p >= 0.05` = change might be noise
- **Status**:
  - "Performance has improved" = significant improvement
  - "No change in performance" = within noise range
  - "Performance has regressed" = significant regression

### Compare Against Main Branch

```bash
# Save main branch baseline first
git checkout main
cargo bench -- --save-baseline main
git checkout your-feature-branch

# Compare your changes
cargo bench -- --baseline main
```

---

## Viewing Reports

### HTML Reports

After running benchmarks:

```bash
open target/criterion/index.html  # macOS
xdg-open target/criterion/index.html  # Linux
```

**Report Contents**:
- **Violin plots**: Distribution of measurements
- **Line charts**: Performance trends over time
- **Summary table**: Mean, median, std dev, outliers
- **Comparison**: Side-by-side baseline vs current

### Individual Benchmark Reports

```bash
# View specific benchmark
open target/criterion/cache/get_model/report/index.html
```

### JSON Data

Raw data for custom analysis:

```bash
cat target/criterion/cache/get_model/new/estimates.json | jq '.'
```

---

## When to Run Benchmarks

### Use `/benchmark-advisor` Claude Skill

```bash
# In Claude Code
/benchmark-advisor
```

**Automatically suggests benchmarks when**:
- Files in `src/cache/`, `src/config/`, `src/context/`, `src/logging/` changed
- Commit message mentions "optimize", "performance", "faster"
- Large refactoring (> 500 lines changed)

**Example output**:
```
🔍 Benchmark Recommendation

Changed files:
  • src/cache/lru.rs
  • src/cache/eviction.rs

Suggested benchmark:
  cargo bench --bench cache

Why: Changes affect performance-critical cache operations.
Look for: LRU eviction time, cache hit latency
```

### Manual Decision Guidelines

**Always run benchmarks when**:
- ✅ Optimizing performance-critical code
- ✅ Changing data structures (HashMap, Vec, etc.)
- ✅ Adding/removing allocations
- ✅ Modifying hot paths (cache, config loading)
- ✅ Before submitting optimization PR

**Optional for**:
- ❓ Bug fixes (unless performance-related)
- ❓ Refactoring without algorithmic changes
- ❓ Documentation updates
- ❓ Adding new features (unless performance-sensitive)

---

## CI Integration

### Automatic Runs

Benchmarks run automatically on CI:

1. **Release PRs**: All PRs to `release/*` branches
   - Full benchmark suite vs main baseline
   - Regression detection (15% threshold)
   - PR comment with results

2. **Periodic Monitoring**: Weekly on main branch (Sunday 00:00 UTC)
   - Full benchmark suite
   - Historical data storage
   - Trend analysis

### Viewing CI Results

**In PR comments**:
```markdown
## 🔍 Benchmark Regression Report

**Status**: ⚠️ FAIL | **Baseline**: abc123 | **Current**: def456

### ⚠️ Regressions Detected

| Benchmark | Baseline | Current | Change | Severity |
|-----------|----------|---------|--------|----------|
| config/load_large | 12.5 ms | 18.3 ms | +46.4% | critical |

### 🔧 Suggested Actions
- Investigate config loading regression
- Review changes in src/config/loader.rs
```

**GitHub Actions artifacts**:
1. Go to PR → "Checks" tab → "Benchmarks" workflow
2. Scroll to "Artifacts" section
3. Download: `benchmark-results.json`, `regression-report.json`

---

## Performance Requirements

**From CLAUDE.md**:
- Startup time: < 100ms ✅ Validated by benchmarks
- First inference: < 2s on M1 Mac ✅ Validated by integration tests

**Additional targets** (from benchmarks):
- Cache hit: < 100 ns
- Config load (small): < 10 ms
- Context capture: < 100 μs
- Logging throughput: > 100k msg/s

---

## Troubleshooting

### Benchmarks Taking Too Long

**Problem**: `cargo bench` takes > 15 minutes

**Solutions**:
```bash
# Run subset
cargo bench --bench cache  # Just one area

# Reduce sample size (less accurate)
cargo bench -- --sample-size 10  # Default: 100

# Quick mode (faster but less precise)
cargo bench -- --quick
```

### Noisy Results

**Problem**: Large variance, inconsistent results

**Causes**:
- CPU throttling (laptop power settings)
- Background processes
- Insufficient warm-up

**Solutions**:
```bash
# Increase sample size
cargo bench -- --sample-size 500

# Run on CI (more consistent environment)
git push  # Let CI run benchmarks

# Check system load
top  # Close resource-intensive apps
```

### Baseline Not Found

**Problem**: `Error: baseline 'main' not found`

**Solution**:
```bash
# Create baseline first
git checkout main
cargo bench -- --save-baseline main
git checkout your-branch
cargo bench -- --baseline main
```

### Memory Allocation Errors

**Problem**: Context benchmarks fail with allocation errors

**Solution**:
```bash
# Context benchmarks track allocations
# Ensure test fixtures are deterministic
cargo bench --bench context -- --nocapture
```

---

## Best Practices

### Before Optimizing

1. **Establish baseline**:
   ```bash
   cargo bench -- --save-baseline before
   ```

2. **Identify bottleneck** (use profiling first):
   ```bash
   # Optional: flamegraph for hotspots
   cargo flamegraph --bench cache
   ```

3. **Write optimization hypothesis**:
   - "Replacing Vec with SmallVec should reduce allocations"
   - "Using ahash instead of std HashMap should be faster"

### After Optimizing

1. **Compare results**:
   ```bash
   cargo bench -- --baseline before
   ```

2. **Verify statistical significance**:
   - Look for `p < 0.05`
   - Ignore changes < 5% unless highly consistent

3. **Document in PR**:
   ```markdown
   ## Performance Impact

   Optimized LRU eviction by using intrusive linked list.

   **Benchmark Results**:
   - cache/lru_eviction: 1.2 μs → 890 ns (-30.5%, p < 0.001)

   See benchmark report: [link to CI artifacts]
   ```

### Commit Messages

Reference benchmark data:

```
optimize: Improve cache LRU eviction performance

Replaced std::collections::LinkedList with custom intrusive list
to reduce allocations during eviction.

Benchmark: cache/lru_eviction improved by 30.5% (1.2μs → 890ns)
```

---

## Advanced Usage

### Custom Benchmark Parameters

```rust
// benches/cache.rs
fn bench_cache_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_scaling");
    for size in [10, 100, 1000, 10_000] {
        group.bench_with_input(
            BenchmarkId::new("get_model", size),
            &size,
            |b, &s| {
                let cache = Cache::new(s);
                b.iter(|| cache.get("model"));
            },
        );
    }
    group.finish();
}
```

### Memory Profiling

```rust
// benches/context.rs
use criterion::black_box;

fn bench_allocations(c: &mut Criterion) {
    c.bench_function("context/capture_large_env", |b| {
        b.iter(|| {
            let ctx = black_box(ExecutionContext::capture());
            // Criterion will track allocations
            ctx
        });
    });
}
```

### Throughput Benchmarks

```rust
// benches/logging.rs
fn bench_logging_throughput(c: &mut Criterion) {
    c.bench_function("logging/throughput", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                log::info!("benchmark message");
            }
            start.elapsed()
        });
    });
}
```

---

## Getting Help

- **Documentation**: `docs/BENCHMARKING.md` (comprehensive guide)
- **Criterion docs**: https://bheisler.github.io/criterion.rs/
- **Claude skill**: `/benchmark-advisor` (context-aware suggestions)
- **Performance requirements**: `CLAUDE.md` (project standards)

---

## Appendix: Benchmark Areas

| Area | Benchmarks | Estimated Time | Run Command |
|------|-----------|----------------|-------------|
| Cache | get_model, add_model, remove_model, lru_eviction | 2 min | `cargo bench --bench cache` |
| Config | load_small, load_large, merge_with_cli, merge_with_env | 2 min | `cargo bench --bench config` |
| Context | capture_baseline, capture_large_env | 1 min | `cargo bench --bench context` |
| Logging | throughput, latency, concurrent_load | 3 min | `cargo bench --bench logging` |
| **Full Suite** | All above | 8-10 min | `cargo bench` |
