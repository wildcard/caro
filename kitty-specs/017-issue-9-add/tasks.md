---
description: "Work package task list for Criterion benchmark suite implementation"
---

# Work Packages: Criterion Benchmark Suite for Performance Validation

**Inputs**: Design documents from `/kitty-specs/017-issue-9-add/`
**Prerequisites**: plan.md (architecture), spec.md (requirements), research.md (decisions), data-model.md (schemas), contracts/ (CI specs), quickstart.md (usage)

**Tests**: Benchmark validation is built into Criterion (statistical analysis). No separate test suite required - benchmarks self-validate.

**Organization**: Work packages map to functional requirements (FR1-FR4) and acceptance criteria (AC1-AC5). Each package is independently deliverable.

**Prompt Files**: Each work package has a matching prompt file in `/tasks/planned/` with detailed implementation guidance.

## Subtask Format: `[Txxx] [P?] Description`
- **[P]** indicates the subtask can proceed in parallel (different files/components).
- All paths are relative to repository root.

## Path Conventions
- **Benchmarks**: `benches/` (one file per module: cache.rs, config.rs, context.rs, logging.rs)
- **CI**: `.github/workflows/benchmarks.yml`
- **Scripts**: `scripts/` (regression detection, historical aggregation)
- **Claude Skill**: `.claude/skills/benchmark-advisor/`
- **Documentation**: `docs/` (BENCHMARKING.md, PERFORMANCE.md)

---

## Work Package WP01: Setup & Dependencies (Priority: P0) 🚀 Foundation

**Goal**: Configure Criterion benchmarking framework and establish project structure.
**Independent Test**: `cargo bench --no-run` compiles successfully; benchmark harness is ready.
**Prompt**: `/tasks/planned/WP01-setup-dependencies.md`

### Included Subtasks
- [x] T001 Add `criterion` dev-dependency to `Cargo.toml` (version 0.5, features: html_reports)
- [x] T002 Create `benches/` directory structure at repository root
- [x] T003 Configure `[[bench]]` entries in `Cargo.toml` for each benchmark file
- [x] T004 [P] Add `serde_json` dependency for result serialization (if not already present)
- [x] T005 Verify Rust toolchain version (>= 1.75.0)

### Implementation Notes
- Criterion harness requires `harness = false` in `[[bench]]` declarations
- Four benchmark files needed: `cache.rs`, `config.rs`, `context.rs`, `logging.rs`
- Each file needs its own `[[bench]]` entry for independent execution

### Parallel Opportunities
- T004 can proceed concurrently with T001-T003 (different parts of Cargo.toml)

### Dependencies
- None (starting package)

### Risks & Mitigations
- **Risk**: Criterion version conflicts with existing dependencies
- **Mitigation**: Use `cargo tree` to check for conflicts; pin to 0.5.x range

---

## Work Package WP02: Cache Benchmarks (Priority: P1) 📊 FR1.1

**Goal**: Implement comprehensive cache operation benchmarks.
**Independent Test**: `cargo bench --bench cache` runs successfully; HTML reports generated in `target/criterion/`.
**Prompt**: `/tasks/planned/WP02-cache-benchmarks.md`

### Included Subtasks
- [x] T006 Create `benches/cache.rs` with Criterion boilerplate
- [x] T007 Implement `cache_get_model_hit` benchmark (measures lookup performance)
- [x] T008 Implement `cache_add_model` benchmark (insertion + eviction)
- [x] T009 Implement `cache_remove_model` benchmark (deletion)
- [x] T010 Implement `cache_lru_eviction_full` benchmark (eviction with full cache)
- [x] T011 Add benchmark fixtures (pre-populated cache, test models)
- [x] T012 Run benchmarks locally and validate results format
- [x] T013 Document expected performance ranges in code comments

### Implementation Notes
- Each benchmark should use `criterion::Criterion::bench_function()`
- Use `black_box()` to prevent compiler optimization
- Cache hit benchmark needs pre-warmed cache state
- LRU eviction test should fill cache to capacity first

### Parallel Opportunities
- All benchmark implementations (T007-T010) can be written concurrently if developers coordinate on shared fixtures (T011)

### Dependencies
- Depends on WP01 (Criterion setup complete)

### Risks & Mitigations
- **Risk**: Cache state bleeding between benchmarks
- **Mitigation**: Each benchmark creates isolated cache instance

---

## Work Package WP03: Config Benchmarks (Priority: P1) 📊 FR1.2

**Goal**: Implement configuration loading and merging benchmarks.
**Independent Test**: `cargo bench --bench config` runs successfully; config operations measured at ms precision.
**Prompt**: `/tasks/planned/WP03-config-benchmarks.md`

### Included Subtasks
- [x] T014 Create `benches/config.rs` with Criterion boilerplate
- [x] T015 [P] Implement `config_load_small` benchmark (< 1KB files)
- [x] T016 [P] Implement `config_load_large` benchmark (> 100KB files)
- [x] T017 Create benchmark fixture config files (small and large)
- [x] T018 Implement `config_merge_with_cli` benchmark (CLI arg overlay)
- [x] T019 Implement `config_merge_with_env` benchmark (environment variable overlay)
- [x] T020 Run benchmarks locally and validate results
- [x] T021 Document expected latencies in code comments

### Implementation Notes
- Use temporary files for config fixtures to ensure determinism
- Small config: ~500 bytes typical user config
- Large config: ~150KB stress test
- Merge benchmarks measure overlay performance (not parsing)

### Parallel Opportunities
- T015 and T016 can proceed concurrently (different fixtures)
- T018 and T019 can proceed concurrently (different merge strategies)

### Dependencies
- Depends on WP01 (Criterion setup complete)

### Risks & Mitigations
- **Risk**: File I/O noise affecting measurements
- **Mitigation**: Criterion's statistical analysis filters outliers

---

## Work Package WP04: Context & Logging Benchmarks (Priority: P1) 📊 FR1.3, FR1.4

**Goal**: Implement execution context capture and logging performance benchmarks.
**Independent Test**: `cargo bench --bench context` and `cargo bench --bench logging` both run successfully.
**Prompt**: `/tasks/planned/WP04-context-logging-benchmarks.md`

### Included Subtasks
- [x] T022 Create `benches/context.rs` with Criterion boilerplate
- [x] T023 Implement `context_capture_baseline` benchmark (minimal environment)
- [x] T024 Implement `context_capture_large_env` benchmark (100+ variables)
- [x] T025 Add memory allocation tracking for context benchmarks
- [x] T026 Create `benches/logging.rs` with Criterion boilerplate
- [x] T027 Implement `logging_throughput` benchmark (messages/second)
- [x] T028 Implement `logging_latency` benchmark (p50, p95, p99 percentiles)
- [x] T029 Implement `logging_concurrent_load` benchmark (multi-threaded)
- [x] T030 Run benchmarks locally and validate results
- [x] T031 Document expected performance targets

### Implementation Notes
- Large env benchmark: populate with 100+ dummy environment variables
- Logging throughput: use `iter_custom()` to measure batched operations
- Latency percentiles: Criterion provides these automatically
- Concurrent benchmark: spawn threads with shared logger

### Parallel Opportunities
- Context benchmarks (T022-T025) and logging benchmarks (T026-T029) can proceed fully in parallel (separate files)

### Dependencies
- Depends on WP01 (Criterion setup complete)

### Risks & Mitigations
- **Risk**: Concurrent logging benchmark affected by thread scheduling
- **Mitigation**: Use sufficient iterations for statistical significance

---

## Work Package WP05: CI Workflow & Regression Detection (Priority: P1) 🔧 FR2

**Goal**: Implement automated benchmark execution and regression detection in GitHub Actions.
**Independent Test**: Push to test branch triggers workflow; regression report generated for simulated change.
**Prompt**: `/tasks/planned/WP05-ci-workflow-regression-detection.md`

### Included Subtasks
- [x] T032 Create `.github/workflows/benchmarks.yml` based on contract spec
- [x] T033 Configure workflow triggers (pull_request to release/*, schedule cron, workflow_dispatch)
- [x] T034 Implement baseline comparison step (checkout main, run benchmarks, save baseline)
- [x] T035 Create `scripts/benchmark-compare.py` for regression detection
- [x] T036 Implement Criterion JSON output parsing in compare script
- [x] T037 Implement threshold checking (15% time, 20% memory)
- [x] T038 Implement PR comment generation from regression report schema
- [x] T039 Create `scripts/benchmark-aggregate.py` for daily artifact generation
- [x] T040 Create `scripts/benchmark-monthly-aggregate.py` for monthly rollup
- [x] T041 Configure artifact upload with retention policies (90-day daily, indefinite monthly)
- [x] T042 Implement workflow failure on regression detection
- [x] T043 Test workflow locally with `act` or on feature branch
- [x] T044 Validate artifact generation and PR comment format

### Implementation Notes
- Use `github-script` action for PR comment posting
- Parse Criterion's `change/estimates.json` files for baseline comparison
- Statistical significance: check p-value from Criterion output
- Monthly aggregate runs on first Sunday of each month

### Parallel Opportunities
- Scripts (T035-T040) can be developed concurrently by different developers
- Workflow YAML (T032-T034) proceeds independently until integration (T042-T044)

### Dependencies
- Depends on WP01-WP04 (all benchmarks must exist for CI to run)
- Logically should follow benchmark implementation

### Risks & Mitigations
- **Risk**: CI takes > 10 minutes (violates FR2.3)
- **Mitigation**: Monitor first runs; optimize sample sizes if needed
- **Risk**: GitHub artifact storage costs
- **Mitigation**: 90-day retention for dailies, weekly schedule not daily

---

## Work Package WP06: Claude Skill Integration (Priority: P2) 🤖 FR4

**Goal**: Implement Claude skill to suggest benchmark runs based on code changes.
**Independent Test**: Skill detects changes in `src/cache/`, suggests `cargo bench --bench cache`.
**Prompt**: `/tasks/planned/WP06-claude-skill-integration.md`

### Included Subtasks
- [x] T045 Create `.claude/skills/benchmark-advisor/` directory
- [x] T046 Create `SKILL.md` with skill metadata and invocation logic
- [x] T047 Implement git diff analysis logic (detect changed files)
- [x] T048 Create `mapping.toml` config for file-to-benchmark mappings
- [x] T049 Implement mapping lookup (pattern matching on changed file paths)
- [x] T050 Implement recommendation generation (specific cargo bench command)
- [x] T051 Add explanation text ("why this benchmark is recommended")
- [x] T052 Test skill with simulated file changes
- [x] T053 Update skill with commit message pattern detection (optional enhancement)

### Implementation Notes
- Skill should use `git diff --name-only` for change detection
- Mapping config uses glob patterns: `"src/cache/**/*.rs" = "cargo bench --bench cache"`
- Should detect changes in `src/main.rs` and suggest full suite (affects startup time)
- Explanation should reference which specific files triggered the recommendation

### Parallel Opportunities
- SKILL.md (T046-T051) can be developed independently from mapping config (T048)

### Dependencies
- No strict dependencies, but logically follows WP01-WP04 (benchmarks exist to recommend)

### Risks & Mitigations
- **Risk**: Glob pattern matching complexity
- **Mitigation**: Start with simple prefix matching, enhance later if needed

---

## Work Package WP07: Documentation (Priority: P2) 📚 AC3

**Goal**: Create comprehensive developer documentation for benchmark usage and CI integration.
**Independent Test**: Documentation covers all quickstart scenarios; new developer can run benchmarks successfully.
**Prompt**: `/tasks/planned/WP07-documentation.md`

### Included Subtasks
- [x] T054 Create `docs/BENCHMARKING.md` with comprehensive guide
- [x] T055 Document: How to run benchmarks locally (all commands from quickstart.md)
- [x] T056 Document: How to interpret results (Criterion reports, statistical significance)
- [x] T057 Document: How to compare baselines (save/load baseline workflow)
- [x] T058 Document: CI integration explanation (when benchmarks run, how to read reports)
- [x] T059 Create `docs/PERFORMANCE.md` with baseline documentation
- [x] T060 Document current performance baselines (startup, cache, config, context, logging)
- [x] T061 Document performance requirements from CLAUDE.md
- [x] T062 Update `CONTRIBUTING.md` with benchmark workflow section
- [x] T063 Document: When to run benchmarks (before optimization PRs, after large refactors)
- [x] T064 Add troubleshooting section (noisy results, long execution times, baseline not found)
- [x] T065 Validate documentation with quickstart.md scenarios

### Implementation Notes
- BENCHMARKING.md should mirror quickstart.md structure but with more detail
- Include code examples for common patterns (parameterized benchmarks, custom fixtures)
- PERFORMANCE.md serves as living document - update after baseline changes
- CONTRIBUTING.md should link to BENCHMARKING.md for details

### Parallel Opportunities
- BENCHMARKING.md (T054-T058, T062-T064) and PERFORMANCE.md (T059-T061) can be written concurrently

### Dependencies
- Depends on WP01-WP04 (benchmarks exist to document)
- Depends on WP05 (CI workflow exists to document)
- Should be last major package before polish

### Risks & Mitigations
- **Risk**: Documentation becomes stale as code evolves
- **Mitigation**: Link to code examples; include note about checking cargo bench --help

---

## Work Package WP08: Validation & Polish (Priority: P3) ✨ AC5

**Goal**: Validate performance requirements, polish implementation, and prepare for merge.
**Independent Test**: All acceptance criteria from spec.md verified; ready for PR.
**Prompt**: `/tasks/planned/WP08-validation-polish.md`

### Included Subtasks
- [x] T066 Run full benchmark suite and collect baseline data
- [x] T067 Validate: Startup time < 100ms (from CLAUDE.md requirement)
- [x] T068 Validate: Cache operations within expected ranges
- [x] T069 Validate: Full suite completes in < 10 minutes on CI
- [x] T070 Update PERFORMANCE.md with actual measured baselines
- [x] T071 Code cleanup: remove debug prints, unused imports
- [x] T072 Code cleanup: ensure consistent formatting (cargo fmt)
- [x] T073 Code cleanup: address clippy warnings
- [x] T074 Verify all acceptance criteria from spec.md (AC1-AC5)
- [x] T075 Test full CI workflow end-to-end (trigger on test branch)
- [x] T076 Verify regression detection works (introduce artificial regression)
- [x] T077 Verify historical data storage (check artifacts after scheduled run)
- [x] T078 Test Claude skill with real code changes
- [x] T079 Final documentation review and corrections

### Implementation Notes
- Startup time validation may require integration test, not benchmark
- Use actual CI runs to validate 10-minute requirement
- Artificial regression: temporarily slow down a benchmark, verify CI catches it

### Parallel Opportunities
- Validation tasks (T066-T069) can run concurrently if using separate benchmark files
- Code cleanup (T071-T073) proceeds independently from validation

### Dependencies
- Depends on ALL previous work packages (WP01-WP07)
- This is the final integration and validation package

### Risks & Mitigations
- **Risk**: Performance requirements not met
- **Mitigation**: If baselines miss requirements, optimize before merge (may need additional WP)

---

## Dependency & Execution Summary

- **Critical Path**: WP01 → (WP02 || WP03 || WP04) → WP05 → WP07 → WP08
- **Parallel Streams**:
  - Stream A: WP02 (cache) + WP03 (config) + WP04 (context/logging) can all proceed in parallel after WP01
  - Stream B: WP06 (Claude skill) can proceed independently, any time after WP01
- **MVP Scope**: WP01 + WP02 + WP03 + WP04 delivers functional benchmarks (manual invocation works)
- **Full Feature**: Requires all packages through WP08
- **Quality Gates**: Do not proceed to WP08 until all WP01-WP07 acceptance criteria pass

### Parallelization Highlights

**After WP01 (Setup) completes**, three independent streams can proceed:
1. **Benchmark Implementation**: WP02, WP03, WP04 (different files, no conflicts)
2. **CI Infrastructure**: WP05 (can develop scripts independently, integrate later)
3. **Developer Tools**: WP06 (Claude skill, independent of benchmarks)
4. **Documentation**: WP07 (can draft early based on plan, finalize after implementation)

**Agent Assignment Strategy** (if using parallel agents):
- Agent A: WP02 (cache.rs)
- Agent B: WP03 (config.rs)
- Agent C: WP04 (context.rs + logging.rs)
- Agent D: WP05 (CI workflow + scripts)
- Agent E: WP06 (Claude skill) + WP07 (documentation)
- Convergence: WP08 (validation, single agent)

---

## Subtask Index (Reference)

| Subtask ID | Summary | Work Package | Priority | Parallel? |
|------------|---------|--------------|----------|-----------|
| T001 | Add criterion to Cargo.toml | WP01 | P0 | No |
| T002 | Create benches/ directory | WP01 | P0 | No |
| T003 | Configure [[bench]] entries | WP01 | P0 | No |
| T004 | Add serde_json dependency | WP01 | P0 | [P] |
| T005 | Verify Rust toolchain >= 1.75 | WP01 | P0 | No |
| T006 | Create benches/cache.rs | WP02 | P1 | No |
| T007 | Impl cache_get_model benchmark | WP02 | P1 | [P] |
| T008 | Impl cache_add_model benchmark | WP02 | P1 | [P] |
| T009 | Impl cache_remove_model benchmark | WP02 | P1 | [P] |
| T010 | Impl cache_lru_eviction benchmark | WP02 | P1 | [P] |
| T011 | Add cache benchmark fixtures | WP02 | P1 | No |
| T012 | Run cache benchmarks locally | WP02 | P1 | No |
| T013 | Document cache perf ranges | WP02 | P1 | No |
| T014 | Create benches/config.rs | WP03 | P1 | No |
| T015 | Impl config_load_small benchmark | WP03 | P1 | [P] |
| T016 | Impl config_load_large benchmark | WP03 | P1 | [P] |
| T017 | Create config fixture files | WP03 | P1 | No |
| T018 | Impl config_merge_with_cli benchmark | WP03 | P1 | [P] |
| T019 | Impl config_merge_with_env benchmark | WP03 | P1 | [P] |
| T020 | Run config benchmarks locally | WP03 | P1 | No |
| T021 | Document config perf ranges | WP03 | P1 | No |
| T022 | Create benches/context.rs | WP04 | P1 | No |
| T023 | Impl context_capture_baseline | WP04 | P1 | [P] |
| T024 | Impl context_capture_large_env | WP04 | P1 | [P] |
| T025 | Add memory tracking for context | WP04 | P1 | No |
| T026 | Create benches/logging.rs | WP04 | P1 | No |
| T027 | Impl logging_throughput benchmark | WP04 | P1 | [P] |
| T028 | Impl logging_latency benchmark | WP04 | P1 | [P] |
| T029 | Impl logging_concurrent benchmark | WP04 | P1 | [P] |
| T030 | Run context/logging benchmarks | WP04 | P1 | No |
| T031 | Document context/logging perf | WP04 | P1 | No |
| T032 | Create .github/workflows/benchmarks.yml | WP05 | P1 | No |
| T033 | Configure workflow triggers | WP05 | P1 | No |
| T034 | Impl baseline comparison step | WP05 | P1 | No |
| T035 | Create scripts/benchmark-compare.py | WP05 | P1 | [P] |
| T036 | Parse Criterion JSON output | WP05 | P1 | [P] |
| T037 | Impl threshold checking | WP05 | P1 | [P] |
| T038 | Impl PR comment generation | WP05 | P1 | [P] |
| T039 | Create benchmark-aggregate.py | WP05 | P1 | [P] |
| T040 | Create benchmark-monthly-aggregate.py | WP05 | P1 | [P] |
| T041 | Configure artifact upload | WP05 | P1 | No |
| T042 | Impl workflow failure on regression | WP05 | P1 | No |
| T043 | Test workflow locally | WP05 | P1 | No |
| T044 | Validate artifacts and PR comments | WP05 | P1 | No |
| T045 | Create .claude/skills/benchmark-advisor/ | WP06 | P2 | No |
| T046 | Create SKILL.md | WP06 | P2 | No |
| T047 | Impl git diff analysis | WP06 | P2 | [P] |
| T048 | Create mapping.toml config | WP06 | P2 | [P] |
| T049 | Impl mapping lookup logic | WP06 | P2 | [P] |
| T050 | Impl recommendation generation | WP06 | P2 | [P] |
| T051 | Add explanation text | WP06 | P2 | [P] |
| T052 | Test skill with file changes | WP06 | P2 | No |
| T053 | Add commit message detection | WP06 | P2 | [P] |
| T054 | Create docs/BENCHMARKING.md | WP07 | P2 | No |
| T055 | Document: How to run locally | WP07 | P2 | [P] |
| T056 | Document: Interpret results | WP07 | P2 | [P] |
| T057 | Document: Compare baselines | WP07 | P2 | [P] |
| T058 | Document: CI integration | WP07 | P2 | [P] |
| T059 | Create docs/PERFORMANCE.md | WP07 | P2 | [P] |
| T060 | Document current baselines | WP07 | P2 | [P] |
| T061 | Document performance requirements | WP07 | P2 | [P] |
| T062 | Update CONTRIBUTING.md | WP07 | P2 | [P] |
| T063 | Document: When to run benchmarks | WP07 | P2 | [P] |
| T064 | Add troubleshooting section | WP07 | P2 | [P] |
| T065 | Validate docs with quickstart | WP07 | P2 | No |
| T066 | Run full benchmark suite | WP08 | P3 | No |
| T067 | Validate startup time < 100ms | WP08 | P3 | [P] |
| T068 | Validate cache perf ranges | WP08 | P3 | [P] |
| T069 | Validate CI < 10 min | WP08 | P3 | [P] |
| T070 | Update PERFORMANCE.md with actuals | WP08 | P3 | No |
| T071 | Code cleanup: debug prints | WP08 | P3 | [P] |
| T072 | Code cleanup: cargo fmt | WP08 | P3 | [P] |
| T073 | Code cleanup: clippy warnings | WP08 | P3 | [P] |
| T074 | Verify all acceptance criteria | WP08 | P3 | No |
| T075 | Test full CI end-to-end | WP08 | P3 | No |
| T076 | Verify regression detection | WP08 | P3 | No |
| T077 | Verify historical data storage | WP08 | P3 | No |
| T078 | Test Claude skill with real changes | WP08 | P3 | No |
| T079 | Final documentation review | WP08 | P3 | No |

---

**Total**: 79 subtasks across 8 work packages
**MVP Scope**: WP01-WP04 (36 subtasks, core benchmarking functionality)
**Full Feature**: WP01-WP08 (79 subtasks, includes CI, skill, docs, validation)
