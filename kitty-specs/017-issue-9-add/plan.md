# Implementation Plan: Criterion Benchmark Suite for Performance Validation

**Branch**: `017-issue-9-add` | **Date**: 2026-01-08 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/kitty-specs/017-issue-9-add/spec.md`

## Summary

Implement a comprehensive Criterion-based benchmark suite to establish performance baselines, detect regressions automatically in CI, and provide manual invocation for local development. The suite will benchmark critical operations (cache, config, execution context, logging) with statistical analysis to minimize false positives. CI integration will run on release PRs and periodically on main branch, with historical data tracking for trend analysis. A Claude skill will guide developers on when to run benchmarks locally based on file changes.

## Technical Context

**Language/Version**: Rust 1.75+ (matches existing codebase)
**Primary Dependencies**:
- `criterion` (dev-dependency) - benchmarking framework with statistical analysis
- `serde_json` - for benchmark result serialization
- GitHub Actions - CI/CD platform

**Storage**:
- Local: Criterion's `target/criterion/` directory (HTML reports + JSON data)
- CI: GitHub artifacts (hybrid storage - daily artifacts for 90 days + monthly aggregates indefinitely)

**Testing**:
- Benchmark validation via `cargo bench`
- CI regression detection via Criterion baseline comparison (`--baseline main`)
- Statistical significance testing (built into Criterion)

**Target Platform**:
- Primary: macOS (M1/M2) for development and performance baselines
- CI: ubuntu-latest and macos-latest GitHub Actions runners

**Project Type**: Single Rust project (CLI tool)

**Performance Goals**:
- Startup time: < 100ms (existing requirement from CLAUDE.md)
- First inference: < 2s on M1 Mac (existing requirement)
- Cache operations: sub-millisecond for hits
- Config loading: < 10ms for typical files
- Benchmark suite execution: < 10 minutes on CI

**Constraints**:
- Deterministic benchmarks (no network calls, no random data)
- CI runner resources: 2-4 cores, 7GB RAM (GitHub Actions standard)
- Artifact storage: max 500MB per workflow run
- False positive rate: < 5% (via Criterion's statistical analysis)

**Scale/Scope**:
- 4 benchmark areas (cache, config, context, logging)
- 10-15 individual benchmarks across all areas
- Historical tracking: 12+ weeks of performance data
- CI runs: ~50/month (release PRs + weekly main monitoring)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Constitution Status**: No project-specific constitution found (template only). Proceeding with Rust best practices and existing codebase patterns.

**Default Principles Applied**:
- ✓ Library-first architecture: Benchmarks are dev-dependencies, don't affect production code
- ✓ Test-driven: Benchmarks validate performance requirements (startup < 100ms, inference < 2s)
- ✓ CI integration: Automated regression detection on release PRs
- ✓ Observability: Criterion provides detailed statistical reports and trend analysis

**No violations identified** - benchmarking is a testing concern, follows standard Rust practices.

## Project Structure

### Documentation (this feature)

```
kitty-specs/017-issue-9-add/
├── spec.md              # Feature specification (COMPLETE)
├── plan.md              # This file (IN PROGRESS)
├── research.md          # Phase 0: Technology research and decisions
├── data-model.md        # Phase 1: Benchmark configuration schema
├── quickstart.md        # Phase 1: Developer guide for running benchmarks
├── contracts/           # Phase 1: CI workflow contracts and outputs
│   ├── ci-workflow.yml  # GitHub Actions workflow structure
│   ├── regression-report-schema.json  # PR comment format
│   └── benchmark-history-schema.json  # Historical data format
└── tasks.md             # Phase 2: Work packages (created by /spec-kitty.tasks)
```

### Source Code (repository root)

```
benches/
├── cache.rs             # FR1.1: Cache operations benchmarks
│                        # - get_model (cache hit)
│                        # - add_model (insertion + eviction)
│                        # - remove_model (deletion)
│                        # - lru_eviction (full cache eviction)
├── config.rs            # FR1.2: Configuration operations benchmarks
│                        # - load_small (< 1KB files)
│                        # - load_large (> 100KB files)
│                        # - merge_with_cli (CLI arg overlay)
│                        # - merge_with_env (env var overlay)
├── context.rs           # FR1.3: Execution context benchmarks
│                        # - capture_baseline (minimal environment)
│                        # - capture_large_env (100+ variables)
│                        # - memory allocation tracking
└── logging.rs           # FR1.4: Logging operations benchmarks
                         # - throughput (messages/second)
                         # - latency (p50, p95, p99)
                         # - concurrent_load (multi-threaded)

.github/workflows/
└── benchmarks.yml       # FR2: CI integration workflow
                         # - Triggers: release/* PRs, weekly cron on main
                         # - Regression detection logic
                         # - PR comment generation
                         # - Historical data management

.claude/skills/benchmark-advisor/
└── SKILL.md             # FR4: Claude skill for local guidance
                         # - Git diff analysis
                         # - File-to-benchmark mapping
                         # - Specific command suggestions

scripts/
├── benchmark-regression.sh      # Criterion output parsing for CI
└── benchmark-compare.py         # Regression detection and reporting

docs/
├── BENCHMARKING.md              # AC3: Developer documentation
└── PERFORMANCE.md               # Baseline documentation

Cargo.toml               # Add criterion dev-dependency
```

**Structure Decision**: Single project layout with modular benchmark organization. Each benchmark area (`benches/*.rs`) is independently runnable via `cargo bench --bench <area>`, matching FR1 structure and standard Criterion patterns. CI automation scripts kept in `scripts/` for clarity and reusability.

## Complexity Tracking

*No constitution violations to justify*

**Simplicity Assessment**:
- ✓ Leverages Criterion's built-in features (no custom benchmarking framework)
- ✓ Standard Rust benchmark structure (`benches/` directory)
- ✓ Minimal custom CI logic (parse Criterion output, post comments)
- ✓ File-based historical storage (GitHub artifacts, no database)

## Phase 0: Research & Decisions

### Decision Log

#### Decision 1: CI Regression Detection Implementation
**Chosen**: Option C - Criterion Extensions (leverage existing tools)
**Rationale**:
- Minimizes custom code by using Criterion's built-in baseline comparison (`--baseline main`)
- Criterion provides statistical significance testing out of the box (reduces false positives)
- Custom parsing only needed for PR comment formatting, not core regression logic
- Aligns with "simple by default" principle

**Alternatives Considered**:
- Custom GitHub Action: More reusable but over-engineered for single project
- Workflow Scripts: Too much inline logic, harder to test
- **Selected approach balances simplicity with reliability**

**Implementation Details**:
- Baseline comparison: `cargo bench -- --baseline main`
- Parse Criterion's JSON output from `target/criterion/*/base/estimates.json`
- Regression threshold: 15% time increase or 20% memory increase (from spec)
- Statistical significance: Use Criterion's p-value analysis (target: p < 0.05)

#### Decision 2: Historical Data Storage Strategy
**Chosen**: Option C - Hybrid Approach (reliability + efficiency)
**Rationale**:
- Daily artifacts (`benchmarks-YYYY-MM-DD.json`) provide granular data with 90-day retention
- Monthly aggregates (`benchmark-history-YYYY-MM.json`) enable long-term trend analysis
- Balance between GitHub artifact storage costs and data availability
- Recoverable from individual runs if monthly aggregate corrupted

**Storage Schema**:
```json
// Daily artifact: benchmarks-2026-01-08.json
{
  "date": "2026-01-08",
  "commit": "abc123",
  "benchmarks": {
    "cache/get_model": {"mean": 45.2, "stddev": 2.1, "unit": "ns"},
    "config/load_small": {"mean": 2.3, "stddev": 0.4, "unit": "ms"}
  }
}

// Monthly aggregate: benchmark-history-2026-01.json
{
  "month": "2026-01",
  "runs": [
    {"date": "2026-01-01", "commit": "abc123", "benchmarks": {...}},
    {"date": "2026-01-08", "commit": "def456", "benchmarks": {...}}
  ]
}
```

#### Decision 3: Benchmark Organization Structure
**Chosen**: Option A - Separate Benchmark Files (modular approach)
**Rationale**:
- Each area independently runnable: `cargo bench --bench cache`
- Easier to maintain and test in isolation (single responsibility)
- Maps directly to FR1 structure (cache, config, context, logging)
- Standard Criterion pattern used across Rust ecosystem
- Faster iteration during development (run only affected benchmarks)

**File Organization**:
- `benches/cache.rs` - 4 benchmarks (get, add, remove, lru_eviction)
- `benches/config.rs` - 4 benchmarks (load_small, load_large, merge_with_cli, merge_with_env)
- `benches/context.rs` - 2 benchmarks (capture_baseline, capture_large_env)
- `benches/logging.rs` - 3 benchmarks (throughput, latency_p50_p95_p99, concurrent_load)

#### Decision 4: Claude Skill Implementation
**Chosen**: Option A - Git Diff Analysis (change-based detection)
**Rationale**:
- Provides precise, actionable recommendations (specific benchmark commands)
- File-to-benchmark mapping maintains clarity of what affects what
- Better developer experience than generic "run all benchmarks" suggestions
- Maintainable via configuration file (no code changes for new mappings)

**Mapping Configuration** (`benchmark-advisor-config.toml`):
```toml
[performance_critical]
"src/cache/**/*.rs" = "cargo bench --bench cache"
"src/config/**/*.rs" = "cargo bench --bench config"
"src/context/**/*.rs" = "cargo bench --bench context"
"src/logging/**/*.rs" = "cargo bench --bench logging"
"src/main.rs" = "cargo bench"  # Affects startup time
```

### Technology Stack Validation

| Component | Technology | Justification |
|-----------|-----------|---------------|
| Benchmark Framework | `criterion` 0.5+ | Standard Rust benchmarking, statistical analysis built-in |
| CI Platform | GitHub Actions | Existing project infrastructure |
| Data Serialization | `serde_json` | Already in dependencies, handles Criterion output |
| Historical Storage | GitHub Artifacts | Zero-cost, integrated with CI, sufficient retention |
| Regression Detection | Custom Bash/Python | Lightweight parsing, no external dependencies |
| Claude Skill | Git + TOML config | Native to Claude Code, no runtime dependencies |

### Performance Baselines (Expected)

Based on CLAUDE.md requirements and similar Rust CLI tools:

| Operation | Expected Performance | Measurement Method |
|-----------|---------------------|-------------------|
| Cache hit (get_model) | 10-100 ns | Criterion `iter()` |
| Cache miss + insert | 100-500 ns | Criterion `iter()` |
| LRU eviction | < 1 μs | Criterion `iter()` |
| Config load (small) | 1-5 ms | Criterion `iter()` |
| Config load (large) | 10-50 ms | Criterion `iter()` |
| Context capture (baseline) | 10-100 μs | Criterion `iter()` |
| Context capture (large env) | 100-500 μs | Criterion `iter()` |
| Logging throughput | > 100k msg/s | Custom measurement |
| Logging latency (p50) | < 1 μs | Criterion `iter()` |

## Phase 1: Design & Contracts

### Data Model

See `data-model.md` for complete schemas.

**Core Entities**:
1. **BenchmarkResult** - Individual benchmark execution result
2. **BenchmarkSuite** - Collection of results from one CI run
3. **HistoricalData** - Time-series data for trend analysis
4. **RegressionReport** - CI-generated report for PR comments
5. **BenchmarkMapping** - File-to-benchmark configuration for Claude skill

### API Contracts

See `contracts/` directory for complete specifications.

**Contract 1: CI Workflow Input/Output**
- **Input**: Git ref, baseline ref, threshold percentages
- **Output**: Regression report JSON, historical data artifact
- **Trigger**: PR to `release/*` OR weekly cron on `main`

**Contract 2: Regression Report Schema**
- **Format**: JSON with markdown template for PR comments
- **Fields**: affected benchmarks, performance deltas, statistical significance, suggestion

**Contract 3: Historical Data Schema**
- **Daily**: Single-run results with metadata
- **Monthly**: Aggregated results with trend data

### Developer Documentation

See `quickstart.md` for complete guide.

**Key Workflows**:
1. Running benchmarks locally: `cargo bench`
2. Comparing against baseline: `cargo bench -- --baseline main`
3. Filtering by area: `cargo bench --bench cache`
4. Interpreting results: HTML reports in `target/criterion/`
5. When to run: Use `/benchmark-advisor` Claude skill

## Next Steps

This plan is complete. Proceed to:
1. **Phase 0 Research**: Document any remaining technical unknowns
2. **Phase 1 Artifacts**: Generate `data-model.md`, `contracts/`, `quickstart.md`
3. **Phase 2 Tasks**: Run `/spec-kitty.tasks` to generate work packages for implementation
