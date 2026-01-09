# Feature Specification: LLM Evaluation Harness

**Feature Branch**: `026-llm-evaluation-harness`
**Created**: 2026-01-09
**Status**: Draft
**Input**: User description: "Build comprehensive LLM evaluation harness for testing shell command generation quality, safety, and correctness. Test categories include: command correctness validation, safety pattern detection accuracy, POSIX compliance verification, and multi-backend consistency testing. Deliverables: test dataset with labeled examples, automated evaluation pipeline, benchmark results across backends, and quality metrics dashboard."

## User Scenarios & Testing

### User Story 1 - Automated CI/CD Quality Gate (Priority: P1)

As a **caro developer**, when I make changes to command generation logic or safety patterns, I need to automatically validate that my changes don't regress quality across all backends before merging to main.

**Why this priority**: This is the foundational use case that enables confident iteration on caro's core functionality. Without automated validation, we risk shipping regressions to users.

**Independent Test**: Can be fully tested by running the evaluation harness in CI on a PR and verifying it produces pass/fail results with benchmark comparisons against baseline.

**Acceptance Scenarios**:

1. **Given** a PR that modifies StaticMatcher patterns, **When** CI runs the evaluation harness, **Then** the harness reports pass rates for correctness, safety, POSIX, and multi-backend tests with comparison to main branch baseline
2. **Given** a PR that changes MLX backend prompts, **When** evaluation harness runs, **Then** the harness detects any regressions in command quality and fails the CI check if pass rate drops below threshold
3. **Given** all tests pass, **When** evaluation completes, **Then** benchmark results are archived for future comparison

---

### User Story 2 - Manual Quality Investigation (Priority: P2)

As a **QA engineer**, when I discover a command generation issue in beta testing, I need to add it to the test dataset and verify fixes across all backends to ensure the issue doesn't occur elsewhere.

**Why this priority**: This enables iterative quality improvement based on real-world failures discovered in beta testing and production usage.

**Independent Test**: Can be fully tested by adding a new test case to the dataset YAML file, running the harness manually, and verifying the new test is evaluated across all backends with clear pass/fail output.

**Acceptance Scenarios**:

1. **Given** a new bug report (e.g., "delete everything" not being blocked), **When** I add a labeled test case to the dataset, **Then** I can run the harness and see which backends fail this specific test
2. **Given** a test case that fails on MLX backend, **When** I fix the MLX prompt and re-run evaluation, **Then** the harness shows the test now passes and updates benchmark results
3. **Given** multiple related test failures, **When** I group them by category (safety, correctness), **Then** the harness provides aggregated metrics per category to guide fixes

---

### User Story 3 - Backend Comparison for Development (Priority: P3)

As a **backend developer**, when implementing a new inference backend (e.g., Claude API, Gemini), I need to benchmark its performance against existing backends to ensure it meets quality standards before release.

**Why this priority**: This enables confident addition of new backends by providing objective quality comparisons against proven implementations.

**Independent Test**: Can be fully tested by running the harness with only the new backend enabled and comparing its results against stored baseline results from MLX/StaticMatcher.

**Acceptance Scenarios**:

1. **Given** a new backend implementation, **When** I run the evaluation harness, **Then** I receive a detailed comparison report showing correctness, safety, POSIX compliance scores vs existing backends
2. **Given** the new backend has lower pass rates, **When** I review per-test results, **Then** I can identify specific failure patterns to guide improvements
3. **Given** the new backend meets quality thresholds, **When** evaluation completes, **Then** its baseline results are stored for future CI comparisons

---

### User Story 4 - Quality Dashboard for Stakeholders (Priority: P4)

As a **project manager**, when planning releases, I need to view historical quality trends and current pass rates across all backends to make informed decisions about release readiness.

**Why this priority**: This provides visibility into caro's quality evolution over time and helps identify areas needing attention before major releases.

**Independent Test**: Can be fully tested by generating static HTML/JSON reports from stored evaluation results and verifying they show trends over multiple evaluation runs.

**Acceptance Scenarios**:

1. **Given** multiple evaluation runs stored as JSON, **When** I generate the dashboard, **Then** I see line charts showing pass rate trends for each test category over time
2. **Given** current evaluation results, **When** viewing the dashboard, **Then** I see a matrix comparing all backends across all test categories with color-coded pass/fail indicators
3. **Given** historical benchmark data, **When** planning v1.2.0, **Then** I can identify which test categories have improved since v1.1.0 and which need focus

---

### Edge Cases

- What happens when a backend times out or crashes during evaluation? (System must record timeout/crash as test failure and continue with remaining tests)
- How does the system handle test cases that are ambiguous or have multiple valid outputs? (Test dataset must include expected output patterns or validation rules to handle variability)
- What happens when a new test case is added mid-evaluation run? (System must use the dataset snapshot from run start to ensure consistency)
- How does the harness handle backends that are not available (e.g., MLX on non-Apple hardware)? (System must skip unavailable backends gracefully and report which backends were evaluated)
- What happens when test dataset grows beyond 1000 examples? (Evaluation must remain <5min total, potentially requiring parallel execution or sampling strategies)

## Requirements

### Functional Requirements

- **FR-001**: System MUST evaluate command generation across four categories: correctness, safety, POSIX compliance, and multi-backend consistency
- **FR-002**: System MUST maintain a labeled test dataset with minimum 100 examples: 25 correctness, 25 safety, 25 POSIX, 25 multi-backend
- **FR-003**: System MUST support evaluation of all caro backends: StaticMatcher, MLX, Ollama, vLLM
- **FR-004**: System MUST generate structured results in JSON format including: per-test pass/fail, per-category pass rates, per-backend pass rates, and overall quality score
- **FR-005**: System MUST execute full evaluation suite in under 5 minutes to enable practical CI/CD integration
- **FR-006**: System MUST provide detailed failure output for each failed test including: input request, expected result, actual result, backend used, and failure reason
- **FR-007**: System MUST support incremental test dataset addition without requiring harness code changes
- **FR-008**: System MUST store baseline results for comparison in future evaluation runs
- **FR-009**: System MUST integrate with cargo test framework to enable `cargo test --test evaluation` workflow
- **FR-010**: System MUST provide CLI interface for running evaluation with options: backend filter, category filter, output format
- **FR-011**: System MUST generate benchmark reports comparing current results to stored baselines with statistical significance indicators
- **FR-012**: System MUST validate test dataset integrity on startup (schema validation, required fields, duplicate detection)
- **FR-013**: System MUST gracefully skip backends that are unavailable on current platform (e.g., MLX on Linux)
- **FR-014**: System MUST support parallel test execution to meet 5-minute runtime constraint
- **FR-015**: System MUST generate optional HTML dashboard from JSON results showing historical trends and cross-backend comparisons

### Key Entities

- **TestCase**: Represents a single evaluation example with fields: id (unique identifier), category (correctness/safety/POSIX/multi-backend), input_request (natural language command description), expected_command (expected shell command output), validation_rules (pattern matching or exact match), metadata (tags, difficulty, source)

- **EvaluationResult**: Represents outcome of running one test case on one backend with fields: test_id, backend_name, passed (boolean), actual_command (generated output), failure_reason (if failed), execution_time_ms, timestamp

- **BackendProfile**: Configuration for a specific inference backend including: name, enabled status, timeout_ms, required_features (e.g., macos-only for MLX), evaluation_priority (determines which backends get deep vs basic testing)

- **BenchmarkReport**: Aggregated results from an evaluation run including: run_id, timestamp, overall_pass_rate, category_pass_rates (map of category to percentage), backend_pass_rates (map of backend to percentage), regression_detected (boolean), baseline_comparison (delta from previous run)

## Success Criteria

### Measurable Outcomes

- **SC-001**: Evaluation harness completes full test suite (100+ tests across all available backends) in under 5 minutes on CI infrastructure
- **SC-002**: CI integration detects command generation regressions with 95%+ accuracy (true positives on actual quality degradations, minimal false positives)
- **SC-003**: Developers can add new test cases to the dataset via YAML file edits and see them evaluated in next run without code changes
- **SC-004**: Benchmark reports provide actionable insights: developers can identify specific backend/category combinations needing improvement within 2 minutes of reviewing results
- **SC-005**: System maintains 99%+ uptime in CI (handles backend crashes, timeouts, and platform differences gracefully without failing entire evaluation)
- **SC-006**: Quality dashboard enables non-technical stakeholders to understand caro's quality trends without reviewing raw test output
- **SC-007**: Evaluation framework supports adding new backends (e.g., Anthropic Claude) with <50 lines of adapter code
- **SC-008**: Test dataset grows from 100 to 200+ examples over 3 months through contributions from beta testers and developers, demonstrating practical usability

## Assumptions

1. **Test dataset storage**: Using YAML format for test cases (easy to edit, git-friendly, supports comments for documentation)
2. **Baseline storage**: JSON format in dedicated `tests/evaluation/baselines/` directory with timestamped filenames
3. **CI integration**: GitHub Actions workflow with matrix strategy for parallel backend testing
4. **Performance targets**: Based on current CI infrastructure (GitHub Actions standard runners)
5. **Backend prioritization**: MLX and StaticMatcher receive deepest testing (all 100+ examples), Ollama/vLLM receive basic coverage (subset of examples) per discovery answers
6. **Validation approach**: Correctness uses command equivalence checking, safety uses pattern matching against known dangerous patterns, POSIX uses shellcheck-style validation, multi-backend uses output consistency comparison
7. **Dashboard implementation**: Static HTML generation with Chart.js for visualization, hosted in GitHub Pages or as CI artifacts
8. **Parallel execution**: Using tokio async runtime for concurrent backend requests to meet 5-minute target

## Out of Scope

- Real-time streaming evaluation UI (dashboard is static HTML generated post-evaluation)
- Integration with external monitoring services (SaaS dashboards like Datadog)
- Machine learning-based test case generation (test cases are manually curated)
- Automated fix suggestions for failing tests (harness reports failures, developers implement fixes)
- Multi-language command generation testing (focus is shell commands only)
- Performance profiling of backend latency (focus is correctness, not performance optimization)
- A/B testing framework for comparing prompt variations (separate concern from quality validation)

## Dependencies

- **cargo test framework**: Integration requires test harness to be compatible with Rust's built-in test infrastructure
- **Existing backend implementations**: MLX, Ollama, vLLM, StaticMatcher must have stable APIs for harness to invoke
- **Safety validation module**: Harness must use caro's existing safety pattern detection for consistency
- **Test infrastructure**: Requires CI/CD environment with adequate compute for 5-minute evaluation runs
- **Beta testing data**: Real-world test cases should be derived from issues discovered during v1.1.0 beta testing

## Security & Privacy Considerations

- Test dataset must NOT contain real user commands or personally identifiable information
- Evaluation results stored in git must NOT leak API keys or sensitive configuration
- Backend API calls during evaluation must respect rate limits and not expose credentials
- Dashboard generated from results must be safe to publish publicly (no sensitive data in visualizations)

## Performance & Scalability

- **Evaluation runtime**: Must complete in <5 minutes with 100 tests across 4 backends (target: ~3 seconds per test including overhead)
- **Test dataset growth**: System should support up to 500 test cases without architectural changes
- **Parallel execution**: Enable concurrent backend evaluation to maximize throughput on multi-core CI runners
- **Result storage**: Baseline results must be efficiently diffable in git (use pretty-printed JSON, avoid large binary artifacts)

## Open Questions

None - discovery process and GitHub issue #135 provided comprehensive requirements.
