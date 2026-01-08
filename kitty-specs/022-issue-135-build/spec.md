# Feature Specification: LLM Evaluation Harness for Shell Commands

**Feature Branch**: `022-issue-135-build`
**Created**: 2026-01-08
**Status**: Draft
**Input**: User description: "Issue #135: Build LLM evaluation harness for shell commands. Create comprehensive evaluation framework to test LLM-generated shell command quality, safety, and correctness."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Command Correctness Validation (Priority: P1)

As a caro developer, I need to validate that LLM-generated commands correctly fulfill the user's natural language intent, so that users receive commands that actually accomplish their stated goals.

**Why this priority**: This is the core value proposition of caro - converting natural language to correct shell commands. Without correctness validation, we cannot measure or improve the fundamental product quality.

**Independent Test**: Can be fully tested by running a curated dataset of (natural language input → expected command) pairs through the current caro backend and measuring the match rate. Delivers immediate value by quantifying current accuracy.

**Acceptance Scenarios**:

1. **Given** a test dataset with 100 labeled examples of natural language prompts and their correct shell commands, **When** the harness runs all prompts through the LLM backend, **Then** it reports the percentage of generated commands that match the expected output
2. **Given** a failing test case where the LLM generates an incorrect command, **When** the harness processes the result, **Then** it captures both the expected and actual commands with a diff showing the discrepancy
3. **Given** test cases with varying complexity levels (simple file operations, complex pipes, multi-command sequences), **When** the harness categorizes results by complexity, **Then** it reports accuracy metrics per complexity tier
4. **Given** a command that is semantically equivalent but syntactically different (e.g., `ls -l -a` vs `ls -la`), **When** the harness evaluates it, **Then** it correctly identifies it as a match using equivalence rules

---

### User Story 2 - Safety Pattern Detection Accuracy (Priority: P1)

As a caro developer, I need to measure how accurately the LLM identifies and blocks dangerous commands, so that I can ensure users are protected from destructive operations.

**Why this priority**: Safety is a core tenet of caro's design (as documented in CLAUDE.md). We must validate that safety validation works correctly before v1.1.0 GA release to production users.

**Independent Test**: Can be tested independently by running a dataset of known dangerous commands (from src/safety/ patterns) through the harness and verifying they are correctly flagged. Delivers immediate safety assurance.

**Acceptance Scenarios**:

1. **Given** a test dataset with 50 known dangerous command patterns (rm -rf /, mkfs, fork bombs), **When** the harness evaluates LLM responses containing these commands, **Then** it reports the percentage that were correctly identified as dangerous
2. **Given** a safe command that superficially resembles a dangerous one (e.g., `rm -rf ./temp/` vs `rm -rf /`), **When** the harness evaluates it, **Then** it correctly distinguishes between safe and unsafe variants
3. **Given** test cases across different risk levels (Safe, Moderate, High, Critical), **When** the harness processes results, **Then** it reports false positive and false negative rates per risk tier
4. **Given** a command that bypasses safety checks through obfuscation (e.g., variable expansion, command substitution), **When** the harness evaluates it, **Then** it identifies the detection gap and flags it for pattern enhancement

---

### User Story 3 - POSIX Compliance Verification (Priority: P2)

As a caro developer, I need to verify that generated commands use only POSIX-compliant utilities and syntax, so that commands work across different Unix-like systems without bash-specific dependencies.

**Why this priority**: POSIX compliance is a documented requirement (CLAUDE.md lines 159-162), essential for cross-platform compatibility. While important, it's lower priority than correctness and safety as it affects portability rather than core functionality.

**Independent Test**: Can be tested by running generated commands through a POSIX shell validator and checking for bash-specific features. Delivers value by ensuring portability claims are verified.

**Acceptance Scenarios**:

1. **Given** a test dataset generating commands with various shell features, **When** the harness validates POSIX compliance, **Then** it identifies and reports any bash-specific syntax (arrays, process substitution, [[ ]])
2. **Given** a command using non-standard utilities (e.g., GNU-specific flags), **When** the harness checks portability, **Then** it flags utilities not in the POSIX standard
3. **Given** test cases requiring portable solutions (file operations, text processing), **When** the harness validates results, **Then** it confirms commands use only standard utilities (ls, find, grep, awk, sed, sort)
4. **Given** a command that could use either portable or non-portable syntax, **When** the harness evaluates it, **Then** it reports compliance score and suggests portable alternatives for violations

---

### User Story 4 - Multi-Backend Consistency Testing (Priority: P2)

As a caro developer, I need to compare command generation quality across different backends (MLX, Ollama, vLLM), so that users get consistent quality regardless of their chosen backend.

**Why this priority**: Caro supports multiple backends (CLAUDE.md lines 88-99). Ensuring consistency validates the backend abstraction and helps identify backend-specific issues. Lower priority because it's about optimization rather than core functionality.

**Independent Test**: Can be tested by running the same test dataset through all available backends and computing variance in correctness/safety metrics. Delivers value by identifying backend-specific quality gaps.

**Acceptance Scenarios**:

1. **Given** a standardized test dataset of 100 prompts, **When** the harness runs all prompts through MLX, Ollama, and vLLM backends, **Then** it reports correctness scores for each backend and highlights statistically significant differences
2. **Given** test results showing one backend performs worse on a specific category (e.g., safety detection), **When** the harness analyzes the failures, **Then** it produces a detailed breakdown of failure patterns per backend
3. **Given** backends with different response formats or latencies, **When** the harness processes results, **Then** it normalizes and compares output quality independently of performance characteristics
4. **Given** a prompt that produces different commands from different backends, **When** the harness evaluates both, **Then** it determines which (if any) is correct and reports backend-specific correctness rates

---

### User Story 5 - Quality Metrics Dashboard (Priority: P3)

As a caro maintainer, I need a visual dashboard showing evaluation metrics over time, so that I can track quality improvements and catch regressions as the project evolves.

**Why this priority**: Dashboards enhance visibility but are not required for initial validation. Can be added after core evaluation infrastructure is working. Lower priority as manual analysis of test results can suffice initially.

**Independent Test**: Can be tested by generating test reports, feeding them to the dashboard, and verifying all metrics display correctly. Delivers value by improving developer experience in monitoring quality.

**Acceptance Scenarios**:

1. **Given** evaluation results from multiple test runs, **When** the dashboard processes the data, **Then** it displays trend lines for correctness, safety, and POSIX compliance metrics over time
2. **Given** a regression where safety detection drops from 95% to 85%, **When** the dashboard updates, **Then** it highlights the regression with a visual indicator and links to the failing test cases
3. **Given** test results segmented by category (file operations, network commands, text processing), **When** the dashboard renders them, **Then** it shows per-category breakdowns allowing drill-down into specific command types
4. **Given** benchmark results comparing backend performance, **When** the dashboard displays them, **Then** it shows side-by-side comparisons with statistical confidence intervals

---

### Edge Cases

- What happens when a test dataset includes malformed natural language prompts that don't clearly specify intent?
- How does the harness handle LLM responses that return errors, timeouts, or invalid JSON instead of commands?
- What happens when the "correct" command for a prompt has multiple valid alternatives (e.g., different but equivalent regex patterns)?
- How does the system validate commands that require specific shell environments or installed tools that may not be present?
- What happens when safety patterns produce false positives, blocking legitimate safe commands?
- How does the harness handle backend-specific failures (network errors for remote backends, model loading failures for MLX)?
- What happens when POSIX compliance checks run on systems that don't have a strict POSIX shell available for validation?
- How does the system measure semantic equivalence for commands with different syntax but identical behavior?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST load test datasets from standardized JSON files containing (prompt, expected_command, category, risk_level, posix_compliant) tuples
- **FR-002**: System MUST execute natural language prompts through configured caro backends (MLX, Ollama, vLLM) and capture generated commands
- **FR-003**: System MUST compare generated commands against expected commands using exact match, semantic equivalence, and configurable similarity metrics
- **FR-004**: System MUST validate generated commands against safety patterns from src/safety/ module and report detection accuracy (true positives, false positives, false negatives, true negatives)
- **FR-005**: System MUST verify POSIX compliance by checking for bash-specific syntax, non-standard utilities, and GNU-specific flags
- **FR-006**: System MUST generate evaluation reports in JSON and Markdown formats showing overall metrics and per-category breakdowns
- **FR-007**: System MUST support parallel test execution to minimize total evaluation time (target: < 5 minutes for 100-prompt dataset)
- **FR-008**: System MUST collect and report inference performance metrics (latency, throughput) alongside quality metrics for each backend
- **FR-009**: System MUST allow filtering test results by category (file operations, network, text processing), risk level, and backend
- **FR-010**: System MUST persist evaluation results with timestamps and git commit hashes to enable regression tracking over time

### Key Entities *(include if feature involves data)*

- **TestCase**: Represents a single evaluation item containing natural language prompt, expected command, category tag, risk level, POSIX compliance flag, and optional metadata for validation rules
- **EvaluationResult**: Represents the outcome of running a test case through a backend, including generated command, correctness score (0.0-1.0), safety validation result (blocked/allowed/flagged), POSIX compliance violations, and performance metrics
- **TestDataset**: Collection of test cases grouped by theme (correctness, safety, POSIX, backend-comparison) with versioning and provenance tracking
- **BackendMetrics**: Aggregated statistics for a specific backend across all test cases, including average correctness, safety accuracy (precision/recall), POSIX compliance rate, and performance percentiles
- **EvaluationRun**: A complete test execution session linking test dataset version, backend configuration, timestamp, git commit, and collection of evaluation results

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can run a complete evaluation suite against any backend in under 5 minutes and receive detailed results
- **SC-002**: The harness achieves 100% coverage of documented safety patterns from src/safety/ when validating test cases
- **SC-003**: The harness correctly identifies semantic equivalence in at least 90% of test cases where multiple valid command forms exist
- **SC-004**: POSIX compliance validation catches 95% of bash-specific syntax in generated commands when tested against a curated non-compliance dataset
- **SC-005**: Backend comparison tests identify statistically significant quality differences (p < 0.05) between backends when differences exceed 10 percentage points
- **SC-006**: The evaluation framework reduces manual testing effort by 80% compared to current ad-hoc validation approach
- **SC-007**: Regression detection identifies quality drops of 5% or more within one test run after code changes
- **SC-008**: Test result reports are understandable to non-technical stakeholders (product managers can interpret correctness and safety metrics without developer assistance)

## Assumptions *(optional)*

- Test datasets will be manually curated initially; automated dataset generation is out of scope for v1.1.0
- The harness will use the existing caro CLI as a black-box interface rather than directly calling backend APIs (maintains separation of concerns)
- POSIX compliance validation will use shellcheck or similar static analysis tools rather than runtime execution in a POSIX shell
- The dashboard visualization (User Story 5) may be deferred to v1.2.0 if time-constrained; initial focus is on generating machine-readable reports
- Backend performance metrics (latency) are informational only and don't affect correctness scoring
- Semantic equivalence rules will start with a basic set (flag ordering, whitespace normalization) and expand iteratively based on test failures

## Out of Scope *(optional)*

- Automated test case generation from user telemetry or production logs (future enhancement)
- Real-time monitoring or alerting when evaluation metrics drop below thresholds (future CI/CD integration)
- Adversarial testing or fuzzing to discover edge cases in LLM prompt handling
- User-facing test case contribution workflow (developers only initially)
- Integration with external benchmarking platforms or leaderboards
- Automated retraining or fine-tuning recommendations based on evaluation failures
