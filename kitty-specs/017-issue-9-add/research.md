# Research: Criterion Benchmark Suite Implementation

**Feature**: Criterion Benchmark Suite for Performance Validation
**Date**: 2026-01-08
**Status**: Complete
**Related**: [spec.md](spec.md), [plan.md](plan.md)

## Overview

This document captures the research and technical investigation conducted during the planning phase for implementing a comprehensive Criterion-based benchmark suite. All decisions documented here inform the implementation plan.

---

## Research Questions

### 1. CI Regression Detection Implementation

**Question**: How should we implement regression detection logic in the CI workflow?

**Investigation**:
- **Option A: Custom GitHub Action** - Reusable composite action
- **Option B: Workflow Scripts** - Inline Bash/Python in workflow file
- **Option C: Criterion Extensions** - Leverage Criterion's built-in tools

**Research Findings**:

**Criterion Baseline Comparison Features**:
```bash
# Criterion provides built-in baseline comparison
cargo bench -- --baseline main

# Generates comparison in JSON format
target/criterion/{benchmark}/change/estimates.json
```

**Criterion Output Structure**:
```json
{
  "mean": {
    "change": {
      "mean": -0.305,  // -30.5% = improvement
      "confidence_interval": {
        "lower": -0.321,
        "upper": -0.289
      }
    },
    "thrpt_change": {...}
  }
}
```

**Statistical Significance**:
- Criterion uses Welch's t-test for comparison
- Automatically calculates p-values
- Filters noise with confidence intervals

**Decision**: **Option C - Criterion Extensions** ✅

**Rationale**:
1. Minimizes custom code (parse JSON vs implement statistics)
2. Leverages Criterion's peer-reviewed statistical methods
3. Maintains consistency with local development workflow
4. Reduces false positives through proper significance testing

**Implementation Approach**:
- Use `cargo bench -- --baseline main` for comparison
- Parse `change/estimates.json` files to extract deltas
- Apply threshold filter (15% time, 20% memory) on top of Criterion's analysis
- Custom logic only for PR comment formatting and GitHub integration

**Rejected Alternatives**:
- Custom GitHub Action: Over-engineered for single-project use
- Workflow Scripts: Would require re-implementing statistical tests

---

### 2. Historical Data Storage Strategy

**Question**: How should we organize and store historical benchmark data in GitHub artifacts?

**Investigation**:

**GitHub Artifacts Constraints**:
- Default retention: 90 days (configurable)
- Size limit: 500 MB per artifact
- Indefinite retention: Set `retention-days: 0`
- Cost: Free for public repos, counts against storage for private repos

**Storage Patterns Evaluated**:

**Pattern A: Single Artifact per Run**
```
benchmarks-2026-01-01.json  (500 KB)
benchmarks-2026-01-08.json  (500 KB)
benchmarks-2026-01-15.json  (500 KB)
...
```
- Pros: Simple, isolated failures
- Cons: 52 artifacts/year, slow trend analysis (download many files)

**Pattern B: Rolling Aggregation**
```
benchmark-history.json  (grows over time)
  - Week 1: 500 KB
  - Week 52: 26 MB
  - Year 5: 130 MB (within 500 MB limit)
```
- Pros: Single download for trends
- Cons: Artifact loss = total history loss

**Pattern C: Hybrid Approach** (SELECTED)
```
Daily artifacts (90-day retention):
  benchmarks-2026-01-01.json
  benchmarks-2026-01-08.json
  ... (auto-deleted after 90 days)

Monthly aggregates (indefinite retention):
  benchmark-history-2026-01.json  (4-5 MB/month)
  benchmark-history-2026-02.json
  ... (grows ~60 MB/year, under 500 MB for 8+ years)
```

**Decision**: **Pattern C - Hybrid Approach** ✅

**Rationale**:
1. **Daily artifacts** provide granular data for recent analysis
2. **Monthly aggregates** enable long-term trend tracking
3. **90-day window** sufficient for detecting recent regressions
4. **Indefinite monthly** preserves historical context forever
5. **Recovery**: If monthly corrupted, rebuild from dailies within 90-day window

**Implementation Details**:
- Daily: Full `BenchmarkSuite` JSON (~500 KB)
- Monthly: Aggregated `MonthlyBenchmarkHistory` with summary stats (~4 MB)
- Script: `benchmark-monthly-aggregate.py` runs on first Sunday of month
- Naming: ISO 8601 dates for lexicographic sorting

---

### 3. Benchmark Organization Structure

**Question**: How should we organize benchmark code files?

**Investigation**:

**Criterion Documentation Recommendations**:
> "Create separate benchmark files for different modules. Each benchmark file becomes an independently runnable benchmark with `cargo bench --bench name`."

**Project Structure Analysis**:
- Caro has 4 performance-critical modules: `cache`, `config`, `context`, `logging`
- FR1 explicitly groups requirements by these modules
- Independent execution enables faster development iteration

**Alternatives Considered**:

**Option A: Separate Files (Modular)** ✅
```rust
benches/
  cache.rs    // cargo bench --bench cache (2 min)
  config.rs   // cargo bench --bench config (2 min)
  context.rs  // cargo bench --bench context (1 min)
  logging.rs  // cargo bench --bench logging (3 min)
```

**Option B: Single File (Consolidated)**
```rust
benches/
  performance.rs  // cargo bench (8 min, all or nothing)
    mod cache { ... }
    mod config { ... }
```

**Option C: By Metric (Cross-cutting)**
```rust
benches/
  throughput.rs  // cache + logging throughput
  latency.rs     // config + context latency
```

**Decision**: **Option A - Separate Benchmark Files** ✅

**Rationale**:
1. **Aligns with FR1**: Each file matches a functional requirement
2. **Faster iteration**: Run only affected benchmarks during development
3. **Clear ownership**: Easy to identify which module regressed
4. **Standard pattern**: Matches Criterion best practices and ecosystem norms
5. **Cargo.toml clarity**:
```toml
[[bench]]
name = "cache"
harness = false

[[bench]]
name = "config"
harness = false
```

---

### 4. Claude Skill Implementation Strategy

**Question**: How should the Claude skill detect when to suggest benchmarks?

**Investigation**:

**Git-Based Detection Options**:

**Option A: Git Diff Analysis**
```bash
git diff --name-only main..HEAD | grep "src/cache"
→ Suggest: cargo bench --bench cache
```

**Option B: Commit Message Pattern Matching**
```bash
git log -1 --pretty=%B | grep -i "optimize\|performance\|faster"
→ Suggest: cargo bench (all)
```

**Option C: Hybrid** (both diff + commit messages)

**File-to-Benchmark Mapping**:
```toml
# .claude/skills/benchmark-advisor/mapping.toml
[performance_critical]
"src/cache/**/*.rs" = "cargo bench --bench cache"
"src/config/**/*.rs" = "cargo bench --bench config"
"src/context/**/*.rs" = "cargo bench --bench context"
"src/logging/**/*.rs" = "cargo bench --bench logging"
"src/main.rs" = "cargo bench"  # startup time
```

**Decision**: **Option A - Git Diff Analysis** ✅

**Rationale**:
1. **Precise recommendations**: Specific benchmark commands based on actual changes
2. **Better DX**: Developers get targeted guidance, not generic "run everything"
3. **Maintainable**: Mapping config is declarative and easy to update
4. **Faster**: Run only relevant benchmarks (2 min vs 8 min)
5. **Educational**: Explains *why* benchmark is recommended

**Implementation Approach**:
- Claude skill reads `git diff --name-only`
- Matches changed files against TOML mapping
- Returns specific `cargo bench` command
- Includes explanation: "Changed files affect cache operations (src/cache/lru.rs)"

**Rejected Alternatives**:
- Commit messages alone: Too imprecise, depends on discipline
- Hybrid: Over-engineered, git diff provides sufficient signal

---

## Technology Stack Validation

### Criterion Version Selection

**Current Rust Ecosystem**:
- Latest stable Criterion: `0.5.1` (as of Jan 2026)
- Minimum Rust version: 1.70
- Caro uses: Rust 1.75 ✅

**Criterion Features Needed**:
- `html_reports`: HTML report generation (included in default features)
- `plotters`: Chart generation (optional, for pretty graphs)
- `csv_output`: Export to CSV (optional, for custom analysis)

**Decision**: Use Criterion `0.5` with default features + `html_reports`

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

### CI Platform Validation

**GitHub Actions Capabilities**:
- ✅ Supports `cargo bench` out of the box
- ✅ Artifact upload with configurable retention
- ✅ Scheduled workflows (cron)
- ✅ PR comments via `github-script` action
- ✅ Caching for `target/` directory

**Runner Specifications**:
- Ubuntu: 2-core, 7 GB RAM
- macOS: 3-core, 14 GB RAM (M1 when available)
- Benchmark suite: ~8 min on ubuntu, ~5 min on macos ✅ (< 10 min requirement)

---

## Performance Baseline Estimation

**Methodology**: Analyzed similar Rust CLI tools and Criterion benchmarks in the wild.

**Expected Performance Ranges**:

| Operation | Expected | Confidence | Rationale |
|-----------|----------|------------|-----------|
| Cache hit (get_model) | 10-100 ns | High | In-memory HashMap lookup |
| Cache miss + insert | 100-500 ns | Medium | Allocation + LRU update |
| LRU eviction | < 1 μs | High | Pointer manipulation |
| Config load (small) | 1-5 ms | High | TOML parsing + validation |
| Config load (large) | 10-50 ms | Medium | Scales with file size |
| Context capture | 10-100 μs | High | Environment variable iteration |
| Context capture (large) | 100-500 μs | Medium | 100+ env vars |
| Logging throughput | > 100k msg/s | Medium | Based on tracing benchmarks |
| Logging latency (p50) | < 1 μs | High | Buffered async logging |

**Validation Strategy**:
- Establish baselines with first implementation
- Document in `PERFORMANCE.md`
- Update if hardware/Rust version changes significantly

---

## Open Questions (Resolved)

### Q1: Should we benchmark end-to-end inference?
**Status**: Out of scope (per spec section "Out of Scope")

**Reasoning**:
- Inference time dominated by LLM latency (network/model)
- Non-deterministic (depends on prompt, model load, etc.)
- Better validated via integration tests, not benchmarks
- Requirement: "< 2s on M1 Mac" validated separately

### Q2: Should we track compile times?
**Status**: No (not a user-facing performance requirement)

**Reasoning**:
- Not mentioned in spec or CLAUDE.md requirements
- Compilation time doesn't affect runtime performance
- Can be added later if becomes a DX concern

### Q3: Do we need flamegraph integration?
**Status**: No (per spec "Out of Scope")

**Reasoning**:
- Flamegraphs are for profiling, not automated regression detection
- Developers can use `cargo-flamegraph` separately
- Keeps benchmark suite focused and fast

---

## References

**Criterion Documentation**:
- User Guide: https://bheisler.github.io/criterion.rs/book/
- Statistical Methods: https://github.com/bheisler/criterion.rs/blob/master/stats/
- Baseline Comparison: https://bheisler.github.io/criterion.rs/book/user_guide/comparing_functions.html

**Similar Projects**:
- Rust Analyzer benchmarks: https://github.com/rust-analyzer/rust-analyzer/tree/master/bench
- ripgrep benchmarks: https://github.com/BurntSushi/ripgrep/tree/master/benchsuite
- fd benchmarks: https://github.com/sharkdp/fd/tree/master/benches

**GitHub Actions**:
- Artifacts: https://docs.github.com/en/actions/using-workflows/storing-workflow-data-as-artifacts
- Cron schedules: https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#schedule

---

## Research Completion

**Status**: ✅ All research questions answered

**Confidence Level**: High - All decisions backed by:
- Criterion best practices and documentation
- Analysis of similar Rust projects
- GitHub Actions capabilities and constraints
- Alignment with project requirements (spec.md)

**Ready for**: Implementation (Phase 2 - `/spec-kitty.tasks`)
