# Feature Specification: Criterion Benchmark Suite for Performance Validation

**Feature ID**: 017-issue-9-add
**Created**: 2026-01-08
**Status**: Draft

---

## Overview

### Problem Statement

Caro has specific performance requirements (startup < 100ms, first inference < 2s on M1 Mac) but lacks systematic performance measurement and regression detection. Without benchmarks, performance degradation goes unnoticed until users complain, and optimization efforts lack baseline data for impact validation.

### Proposed Solution

Implement a comprehensive Criterion-based benchmark suite that:
- Establishes performance baselines for critical operations
- Automatically detects regressions in release PRs and periodically on main
- Provides manual invocation for local development
- Includes Claude skill guidance for when to run benchmarks locally

### Value Proposition

**For developers**: Catch performance regressions early, validate optimizations with data
**For users**: Consistent performance that meets documented requirements
**For project**: Data-driven performance decisions, historical performance tracking

---

## User Scenarios & Testing

### Primary Scenarios

**Scenario 1: Developer validates optimization locally**
1. Developer optimizes cache eviction algorithm
2. Claude skill suggests running benchmarks before committing
3. Developer runs `cargo bench` to see performance improvement
4. Benchmark report shows 30% improvement in LRU eviction time
5. Developer commits with confidence, referencing benchmark data

**Scenario 2: CI detects performance regression in release PR**
1. Developer submits PR for release/v1.2.0 branch
2. CI runs full benchmark suite automatically
3. Benchmarks show config loading regressed by 40%
4. CI fails the build with regression report
5. Developer investigates, finds accidental O(n²) operation
6. Developer fixes issue, re-runs benchmarks, sees performance restored

**Scenario 3: Periodic performance monitoring on main**
1. Weekly scheduled CI job runs benchmarks on main
2. Historical data shows gradual startup time increase over 4 weeks (75ms → 98ms)
3. Team reviews trend before it crosses 100ms threshold
4. Proactive optimization work is scheduled

**Scenario 4: Manual benchmark investigation**
1. Developer suspects memory leak in execution context capture
2. Runs `cargo bench --bench context` for targeted benchmarks
3. Observes high memory allocation in large environment test
4. Uses flamegraph integration to identify culprit
5. Fixes leak, validates with re-run showing 60% memory reduction

---

## Functional Requirements

### FR1: Benchmark Coverage

**FR1.1 Cache Operations**
- Benchmark `get_model()` for cache hits (measures lookup performance)
- Benchmark `add_model()` (measures insertion + eviction)
- Benchmark `remove_model()` (measures deletion)
- Benchmark LRU eviction with full cache (measures eviction algorithm)

**FR1.2 Configuration Operations**
- Benchmark config loading from small files (< 1KB)
- Benchmark config loading from large files (> 100KB)
- Benchmark `merge_with_cli()` (CLI arg overlay)
- Benchmark `merge_with_env()` (environment variable overlay)

**FR1.3 Execution Context**
- Benchmark baseline `capture()` (minimal environment)
- Benchmark `capture()` with large environment (100+ variables)
- Measure memory allocations, not just time

**FR1.4 Logging Operations**
- Benchmark throughput (messages/second)
- Benchmark latency percentiles (p50, p95, p99)
- Test under concurrent load

### FR2: CI Integration

**FR2.1 Release PR Benchmarks**
- Run full benchmark suite on all PRs targeting `release/*` branches
- Compare results against main branch baseline
- Fail build if any benchmark regresses beyond threshold (15% slower or 20% more memory)
- Post regression report as PR comment with:
  - Affected benchmarks
  - Performance delta (absolute and percentage)
  - Suggestion to investigate or justify

**FR2.2 Periodic Main Branch Monitoring**
- Run full benchmark suite weekly on main branch (Sunday 00:00 UTC)
- Store historical results in GitHub artifacts
- Generate trend report showing performance over time
- Alert team if any metric approaches threshold (within 10% of requirement)

**FR2.3 CI Performance**
- Benchmark suite completes in under 10 minutes
- Uses GitHub Actions caching for Criterion target/ artifacts
- Runs on ubuntu-latest and macos-latest (M1 runners when available)

### FR3: Manual Invocation

**FR3.1 Local Development**
- Standard Cargo commands work: `cargo bench`
- Targeted runs: `cargo bench --bench cache`, `cargo bench --bench config`
- Filter by pattern: `cargo bench lru_eviction`
- Generate HTML reports: `cargo bench -- --save-baseline <name>`

**FR3.2 Comparison**
- Compare against baseline: `cargo bench -- --baseline <name>`
- Compare against main: `cargo bench -- --baseline main`
- Show statistical significance (Criterion's built-in analysis)

### FR4: Claude Skill Integration

**FR4.1 Skill: benchmark-advisor**
- Suggests running benchmarks when:
  - Code changes affect performance-critical modules (cache, config, logging)
  - Commit message mentions "optimize", "performance", "faster"
  - Large refactoring (> 500 lines changed in core modules)
- Provides specific benchmark command for changed modules
- Explains why benchmarks are recommended

**FR4.2 Skill Commands**
- `/benchmark-advisor` - Analyze current changes, suggest benchmarks
- Output includes:
  - Why benchmarks are recommended
  - Specific command to run
  - What to look for in results

---

## Success Criteria

1. **Baseline Establishment**: All performance-critical operations have Criterion benchmarks with statistical analysis
2. **Regression Detection Rate**: 100% of performance regressions >15% are caught in release PR CI
3. **False Positive Rate**: < 5% of CI benchmark failures are noise (use Criterion's statistical significance)
4. **Benchmark Execution Time**: Full suite completes in < 10 minutes on CI
5. **Developer Adoption**: 80%+ of optimization commits reference benchmark data in commit message or PR description
6. **Historical Tracking**: 12 weeks of performance trend data available for analysis
7. **Requirement Validation**: Current performance meets documented requirements (startup < 100ms, inference < 2s)

---

## Acceptance Criteria

### AC1: Benchmark Suite Implementation
- [ ] `benches/cache.rs` with 4+ benchmarks for cache operations
- [ ] `benches/config.rs` with 4+ benchmarks for config operations
- [ ] `benches/context.rs` with 2+ benchmarks for execution context
- [ ] `benches/logging.rs` with throughput and latency benchmarks
- [ ] All benchmarks use Criterion with statistical analysis
- [ ] Benchmarks run successfully with `cargo bench`

### AC2: CI Integration
- [ ] GitHub Actions workflow `.github/workflows/benchmarks.yml`
- [ ] Workflow triggers on PRs to `release/*` branches
- [ ] Workflow runs weekly on main (cron schedule)
- [ ] Regression detection logic compares against baseline
- [ ] Build fails if regression exceeds threshold (15% time, 20% memory)
- [ ] Regression report posted as PR comment
- [ ] Historical data stored in GitHub artifacts

### AC3: Documentation
- [ ] `docs/BENCHMARKING.md` with:
  - How to run benchmarks locally
  - How to interpret results
  - How to compare baselines
  - Performance requirements reference
  - CI integration explanation
- [ ] Benchmark results baseline documented (`PERFORMANCE.md`)
- [ ] CI workflow documented in `CONTRIBUTING.md`

### AC4: Claude Skill
- [ ] Skill file `.claude/skills/benchmark-advisor/SKILL.md`
- [ ] Skill detects performance-critical file changes
- [ ] Skill suggests specific benchmark commands
- [ ] Skill explains recommendation reasoning

### AC5: Performance Requirements Validation
- [ ] Benchmarks confirm startup time < 100ms
- [ ] Benchmarks confirm cache operations within acceptable bounds
- [ ] Documentation updated with actual measured baselines

---

## Out of Scope

- Flamegraph integration (use existing tools like `cargo-flamegraph`)
- Memory profiling beyond allocations (use `heaptrack` separately)
- Benchmark visualization dashboard (future enhancement)
- Automated performance tuning (manual optimization only)
- Benchmark results database (GitHub artifacts sufficient for v1)
- Cross-platform comparison analysis (just track separately)
- Micro-benchmarks for every function (only performance-critical paths)

---

## Dependencies

### Internal
- `src/cache/` - Cache module to benchmark
- `src/config/` - Config module to benchmark
- `src/context/` - Execution context module
- `src/logging/` - Logging infrastructure

### External
- `criterion` crate (dev-dependency)
- GitHub Actions (CI platform)
- Rust toolchain (cargo bench support)

### Assumptions
- Criterion is the standard Rust benchmarking framework (assumption: project uses Rust best practices)
- GitHub Actions has sufficient compute for benchmarks (assumption: 10-minute runs are acceptable)
- Main branch is the baseline for comparison (assumption: main represents production state)
- 15% regression threshold is acceptable (assumption: based on typical statistical significance)

---

## Technical Constraints

- Benchmarks must be deterministic (no network calls, no random data)
- Use Criterion's statistical analysis for significance testing
- CI runners have 2-4 cores, 7GB RAM (GitHub Actions standard)
- Benchmark data stored in GitHub artifacts (max 500MB per workflow)
- Weekly runs to manage artifact storage costs

---

## Security Considerations

- Benchmarks run in CI with read-only repository access
- No sensitive data in benchmark fixtures
- Benchmark results are public (in GitHub artifacts)
- Claude skill does not execute benchmarks automatically (suggests only)

---

## Open Questions

None - scope is well-defined and implementation-ready.
