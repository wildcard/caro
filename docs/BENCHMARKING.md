# Benchmarking Guide

Comprehensive guide for running and interpreting Criterion benchmarks in Caro.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench cache
cargo bench --bench config
cargo bench --bench context
cargo bench --bench logging

# Run specific benchmark
cargo bench --bench cache -- get_model
```

## Understanding Results

Criterion provides statistical analysis with:
- **Mean**: Average execution time
- **Std Dev**: Variability in measurements
- **Median**: Middle value (resistant to outliers)
- **MAD**: Median Absolute Deviation

### Example Output

```
cache/get_model         time:   [50.2 ns 51.5 ns 52.8 ns]
                        change: [-2.3% +0.5% +3.1%] (p = 0.42 > 0.05)
                        No change in performance detected.
```

**Interpretation**:
- Mean: 51.5 ns
- 95% confidence interval: [50.2ns, 52.8ns]
- Change from baseline: +0.5% (not statistically significant, p=0.42)

### Statistical Significance

- **p < 0.05**: Change is statistically significant
- **p ≥ 0.05**: Change may be noise (not conclusive)

## Baseline Comparison

Save current results as baseline:
```bash
cargo bench -- --save-baseline my-baseline
```

Compare against baseline:
```bash
cargo bench -- --baseline my-baseline
```

## CI Integration

Benchmarks run automatically in CI:
- **On PRs to release/\***: Regression detection with baseline comparison
- **Weekly on main**: Historical data collection
- **Manual**: `workflow_dispatch` for on-demand runs

### Regression Detection

CI fails if benchmarks regress beyond thresholds:
- **Time**: 15% slower than baseline
- **Memory**: 20% more than baseline

Regressions are posted as PR comments with severity classification.

## HTML Reports

Criterion generates detailed HTML reports in `target/criterion/`:
- Violin plots showing distribution
- Iteration times
- Regression analysis
- Comparison charts

Open in browser:
```bash
open target/criterion/cache/get_model/report/index.html
```

## Troubleshooting

### Noisy Results

**Symptom**: Large variance, inconsistent measurements

**Solutions**:
- Close resource-intensive applications
- Run multiple times to verify
- Check system load (`top`, `htop`)
- Increase sample size (edit benchmark file)

### Long Execution Time

**Symptom**: Benchmarks take > 10 minutes

**Solutions**:
- Run specific suites: `cargo bench --bench cache`
- Reduce sample sizes in benchmark code
- Use faster measurement modes (not recommended for CI)

### Baseline Not Found

**Symptom**: "Error: Baseline 'X' not found"

**Solution**: Save baseline first:
```bash
cargo bench -- --save-baseline X
```

## Best Practices

### When to Run Benchmarks

- Before optimization work (establish baseline)
- After optimization (measure improvement)
- Before release PRs (regression check)
- When modifying performance-critical code

### Interpreting Changes

- **< 5%**: Likely noise, not meaningful
- **5-15%**: Minor change, investigate if unexpected
- **15-30%**: Significant change, review code carefully
- **> 30%**: Major change, requires explanation

### Making Changes

1. **Establish baseline**: `cargo bench -- --save-baseline before`
2. **Make changes**: Edit code
3. **Measure impact**: `cargo bench -- --baseline before`
4. **Document**: Note improvements/regressions in commit message

## Benchmark Advisor Skill

Use the Claude skill for intelligent suggestions:

```
User: "What benchmarks should I run?"

Claude: Based on your changes to src/cache/manifest.rs,
        I recommend:

        cargo bench --bench cache
```

The skill analyzes `git diff` and suggests relevant benchmarks.

## Performance Requirements

From CLAUDE.md:
- **Startup**: < 100ms
- **First inference**: < 2s (M1 Mac)
- **Cache operations**: < 1μs
- **Config loading**: < 10ms

See `docs/PERFORMANCE.md` for detailed baselines.
