# Data Model: Benchmark Suite Schemas

**Feature**: Criterion Benchmark Suite for Performance Validation
**Date**: 2026-01-08
**Related**: [spec.md](spec.md), [plan.md](plan.md)

## Overview

This document defines the data structures used for benchmark execution, storage, and reporting. All schemas are designed for JSON serialization and integration with Criterion's output format.

---

## 1. BenchmarkResult

Represents a single benchmark execution result from Criterion.

### Schema

```rust
struct BenchmarkResult {
    /// Benchmark identifier (e.g., "cache/get_model")
    id: String,

    /// Mean execution time
    mean: Duration,

    /// Standard deviation
    std_dev: Duration,

    /// Median execution time
    median: Duration,

    /// Percentiles (p50, p75, p95, p99)
    percentiles: Percentiles,

    /// Memory allocations (optional, for context benchmarks)
    memory: Option<MemoryStats>,

    /// Sample size
    sample_size: usize,

    /// Statistical outliers
    outliers: OutlierStats,
}

struct Duration {
    value: f64,
    unit: TimeUnit,  // ns, us, ms, s
}

struct Percentiles {
    p50: Duration,
    p75: Duration,
    p95: Duration,
    p99: Duration,
}

struct MemoryStats {
    total_allocations: u64,
    total_bytes: u64,
    peak_bytes: u64,
}

struct OutlierStats {
    low_severe: usize,
    low_mild: usize,
    high_mild: usize,
    high_severe: usize,
}
```

### JSON Example

```json
{
  "id": "cache/get_model",
  "mean": {
    "value": 45.2,
    "unit": "ns"
  },
  "std_dev": {
    "value": 2.1,
    "unit": "ns"
  },
  "median": {
    "value": 44.8,
    "unit": "ns"
  },
  "percentiles": {
    "p50": {"value": 44.8, "unit": "ns"},
    "p75": {"value": 46.1, "unit": "ns"},
    "p95": {"value": 49.3, "unit": "ns"},
    "p99": {"value": 52.7, "unit": "ns"}
  },
  "memory": null,
  "sample_size": 10000,
  "outliers": {
    "low_severe": 0,
    "low_mild": 12,
    "high_mild": 18,
    "high_severe": 3
  }
}
```

### Validation Rules

- `mean.value` > 0
- `std_dev.value` >= 0
- `sample_size` >= 100 (Criterion default)
- `percentiles.p50` <= `percentiles.p75` <= `percentiles.p95` <= `percentiles.p99`
- `memory` only present for context benchmarks (FR1.3)

---

## 2. BenchmarkSuite

Collection of benchmark results from a single CI run or local execution.

### Schema

```rust
struct BenchmarkSuite {
    /// ISO 8601 timestamp
    timestamp: String,

    /// Git commit SHA
    commit: String,

    /// Git branch name
    branch: String,

    /// Environment metadata
    environment: Environment,

    /// All benchmark results
    benchmarks: Vec<BenchmarkResult>,

    /// Total execution time for suite
    total_duration: Duration,
}

struct Environment {
    os: String,           // "macos", "linux"
    os_version: String,   // "14.2", "22.04"
    cpu: String,          // "Apple M1", "Intel Xeon"
    cpu_cores: u32,
    memory_gb: u32,
    rust_version: String, // "1.75.0"
}
```

### JSON Example

```json
{
  "timestamp": "2026-01-08T15:30:00Z",
  "commit": "abc123def456",
  "branch": "017-issue-9-add",
  "environment": {
    "os": "macos",
    "os_version": "14.2",
    "cpu": "Apple M1",
    "cpu_cores": 8,
    "memory_gb": 16,
    "rust_version": "1.75.0"
  },
  "benchmarks": [
    {
      "id": "cache/get_model",
      "mean": {"value": 45.2, "unit": "ns"},
      ...
    },
    {
      "id": "config/load_small",
      "mean": {"value": 2.3, "unit": "ms"},
      ...
    }
  ],
  "total_duration": {
    "value": 8.5,
    "unit": "min"
  }
}
```

### Validation Rules

- `timestamp` must be valid ISO 8601 format
- `commit` must be valid Git SHA (40 hex chars)
- `benchmarks` array not empty
- `total_duration` should be sum of all benchmark execution times + overhead

---

## 3. HistoricalData

Time-series data for trend analysis. Two storage formats: daily and monthly.

### Daily Artifact Schema

```rust
struct DailyBenchmarkData {
    /// Date in YYYY-MM-DD format
    date: String,

    /// Full benchmark suite results
    suite: BenchmarkSuite,
}
```

### Monthly Aggregate Schema

```rust
struct MonthlyBenchmarkHistory {
    /// Month in YYYY-MM format
    month: String,

    /// Ordered list of daily runs
    runs: Vec<DailyBenchmarkData>,

    /// Summary statistics for the month
    summary: MonthlySummary,
}

struct MonthlySummary {
    total_runs: usize,
    benchmarks: Vec<BenchmarkTrend>,
}

struct BenchmarkTrend {
    id: String,
    mean_of_means: Duration,
    trend_direction: TrendDirection,  // improving, stable, degrading
    max_regression: f64,              // percentage
    max_improvement: f64,             // percentage
}

enum TrendDirection {
    Improving,   // Performance getting better
    Stable,      // Within +/- 5% variance
    Degrading,   // Performance getting worse
}
```

### JSON Example (Monthly Aggregate)

```json
{
  "month": "2026-01",
  "runs": [
    {
      "date": "2026-01-01",
      "suite": { ... }
    },
    {
      "date": "2026-01-08",
      "suite": { ... }
    }
  ],
  "summary": {
    "total_runs": 5,
    "benchmarks": [
      {
        "id": "cache/get_model",
        "mean_of_means": {"value": 46.3, "unit": "ns"},
        "trend_direction": "stable",
        "max_regression": 3.2,
        "max_improvement": 2.1
      }
    ]
  }
}
```

### Validation Rules

- Daily: `date` format YYYY-MM-DD
- Monthly: `month` format YYYY-MM
- Monthly: `runs` sorted by date ascending
- `total_runs` = `runs.length`

---

## 4. RegressionReport

CI-generated report for PR comments when regression detected.

### Schema

```rust
struct RegressionReport {
    /// PR number (if applicable)
    pr_number: Option<u32>,

    /// Baseline commit SHA
    baseline_commit: String,

    /// Current commit SHA
    current_commit: String,

    /// Detected regressions
    regressions: Vec<Regression>,

    /// Improvements (for context)
    improvements: Vec<Improvement>,

    /// Threshold used for detection
    threshold: RegressionThreshold,

    /// Overall status
    status: ReportStatus,
}

struct Regression {
    benchmark_id: String,
    baseline_mean: Duration,
    current_mean: Duration,
    delta_percent: f64,      // Positive = regression
    statistical_significance: f64,  // p-value
    severity: Severity,
}

struct Improvement {
    benchmark_id: String,
    baseline_mean: Duration,
    current_mean: Duration,
    delta_percent: f64,      // Negative = improvement
}

struct RegressionThreshold {
    time_percent: f64,       // 15% by default
    memory_percent: f64,     // 20% by default
}

enum Severity {
    Critical,  // > 30% regression
    High,      // 15-30% regression
    Medium,    // 10-15% regression
}

enum ReportStatus {
    Pass,          // No regressions
    Fail,          // Regressions detected
    Warning,       // Regressions but below threshold
}
```

### JSON Example

```json
{
  "pr_number": 123,
  "baseline_commit": "abc123",
  "current_commit": "def456",
  "regressions": [
    {
      "benchmark_id": "config/load_large",
      "baseline_mean": {"value": 12.5, "unit": "ms"},
      "current_mean": {"value": 18.3, "unit": "ms"},
      "delta_percent": 46.4,
      "statistical_significance": 0.001,
      "severity": "critical"
    }
  ],
  "improvements": [
    {
      "benchmark_id": "cache/get_model",
      "baseline_mean": {"value": 50.2, "unit": "ns"},
      "current_mean": {"value": 45.2, "unit": "ns"},
      "delta_percent": -9.96
    }
  ],
  "threshold": {
    "time_percent": 15.0,
    "memory_percent": 20.0
  },
  "status": "fail"
}
```

### Markdown Template (for PR comments)

```markdown
## 🔍 Benchmark Regression Report

**Status**: {status} | **Baseline**: {baseline_commit} | **Current**: {current_commit}

### ⚠️ Regressions Detected

| Benchmark | Baseline | Current | Change | Severity | p-value |
|-----------|----------|---------|--------|----------|---------|
| {benchmark_id} | {baseline_mean} | {current_mean} | +{delta_percent}% | {severity} | {p_value} |

### ✅ Improvements

| Benchmark | Baseline | Current | Change |
|-----------|----------|---------|--------|
| {benchmark_id} | {baseline_mean} | {current_mean} | {delta_percent}% |

### 📊 Threshold Configuration

- Time regression threshold: {time_percent}%
- Memory regression threshold: {memory_percent}%

### 🔧 Suggested Actions

- **Critical/High severity**: Investigate regression before merging
- **Medium severity**: Consider if performance impact is acceptable
- Review changes in affected modules: {affected_files}

---
Generated by Criterion Benchmark Suite • [View full report](link)
```

---

## 5. BenchmarkMapping

Configuration for Claude skill file-to-benchmark mapping.

### Schema (TOML)

```toml
[performance_critical]
# Map file patterns to benchmark commands
"src/cache/**/*.rs" = "cargo bench --bench cache"
"src/config/**/*.rs" = "cargo bench --bench config"
"src/context/**/*.rs" = "cargo bench --bench context"
"src/logging/**/*.rs" = "cargo bench --bench logging"
"src/main.rs" = "cargo bench"  # Full suite for startup time

[thresholds]
# Override default thresholds if needed
time_percent = 15.0
memory_percent = 20.0

[skill_config]
# Claude skill behavior
auto_suggest = true           # Automatically suggest on file changes
show_command = true           # Show full cargo command
explain_why = true            # Explain why benchmark is recommended
```

### Rust Structure (for parsing)

```rust
struct BenchmarkMapping {
    performance_critical: HashMap<String, String>,
    thresholds: Option<ThresholdConfig>,
    skill_config: Option<SkillConfig>,
}

struct ThresholdConfig {
    time_percent: f64,
    memory_percent: f64,
}

struct SkillConfig {
    auto_suggest: bool,
    show_command: bool,
    explain_why: bool,
}
```

---

## Entity Relationships

```
BenchmarkSuite
├── contains many BenchmarkResult
└── stored in HistoricalData (daily)

HistoricalData (monthly)
├── aggregates many DailyBenchmarkData
└── generates MonthlySummary

RegressionReport
├── compares two BenchmarkSuite (baseline vs current)
└── generates list of Regression + Improvement

BenchmarkMapping
└── used by Claude skill to suggest BenchmarkResult runs
```

---

## Storage Locations

| Entity | Local | CI Artifact | Notes |
|--------|-------|-------------|-------|
| BenchmarkResult | `target/criterion/*/new/estimates.json` | - | Criterion native format |
| BenchmarkSuite | `target/benchmark-suite.json` | `benchmarks-{date}.json` | Custom aggregation |
| HistoricalData (daily) | - | `benchmarks-{YYYY-MM-DD}.json` | 90-day retention |
| HistoricalData (monthly) | - | `benchmark-history-{YYYY-MM}.json` | Indefinite retention |
| RegressionReport | - | `regression-report-{pr}.json` | Per-PR, 30-day retention |
| BenchmarkMapping | `.claude/skills/benchmark-advisor/mapping.toml` | - | Version controlled |

---

## Criterion Integration

### Parsing Criterion Output

Criterion stores results in `target/criterion/{benchmark_id}/new/estimates.json`:

```json
{
  "mean": {
    "point_estimate": 45.2,
    "standard_error": 0.8,
    "confidence_interval": {
      "lower_bound": 43.6,
      "upper_bound": 46.8,
      "confidence_level": 0.95
    }
  },
  "median": {
    "point_estimate": 44.8,
    ...
  },
  "std_dev": {
    "point_estimate": 2.1,
    ...
  }
}
```

**Mapping to BenchmarkResult**:
- `mean.point_estimate` → `BenchmarkResult.mean.value`
- `std_dev.point_estimate` → `BenchmarkResult.std_dev.value`
- `median.point_estimate` → `BenchmarkResult.median.value`
- Parse `{benchmark_id}` from directory path

---

## Version History

- **v1.0** (2026-01-08): Initial schema design for benchmark suite implementation
