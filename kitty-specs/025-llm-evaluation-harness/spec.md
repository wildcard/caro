# Feature Specification: LLM Evaluation Harness

**Feature Branch**: `025-llm-evaluation-harness`
**Created**: 2026-01-09
**Status**: Draft
**Input**: User description: "Issue #135: Build LLM evaluation harness for shell commands"

## User Scenarios & Testing

### User Story 1 - Automated Command Quality Validation (Priority: P1)

As a caro developer, I need to automatically validate that generated shell commands are correct, safe, and POSIX-compliant so I can regression-test LLM output quality before each release.

**Why this priority**: This is the MVP - automated validation of existing command generation is critical for maintaining the 94.8% CSR baseline and preventing regressions in v1.1.0 GA release.

**Independent Test**: Can be fully tested by running `cargo test --test evaluation` against a test dataset of 50 labeled examples and verifying that correctness metrics match expected values.

**Acceptance Scenarios**:

1. **Given** a test dataset with 50 labeled prompt→command pairs, **When** I run the evaluation harness, **Then** it reports correctness percentage (expected 90%+)
2. **Given** a known-good command example, **When** the harness evaluates it, **Then** it marks the command as correct with detailed reasoning
3. **Given** a command with a safety violation, **When** the harness evaluates it, **Then** it detects the violation and reports the specific dangerous pattern
4. **Given** a non-POSIX command, **When** the harness evaluates it, **Then** it flags the POSIX compliance issue with explanation

---

### User Story 2 - Multi-Backend Consistency Testing (Priority: P2)

As a caro maintainer, I need to verify that MLX, vLLM, and Ollama backends produce consistent command outputs for the same prompts so I can ensure backend parity before release.

**Why this priority**: Backend consistency is important but secondary to basic correctness validation. This can be built after P1 is working.

**Independent Test**: Can be tested by running the same 10 test prompts through all 3 backends and comparing command similarity scores (should be 95%+ similar).

**Acceptance Scenarios**:

1. **Given** the same prompt "list all files", **When** evaluated across MLX, vLLM, and Ollama, **Then** all three backends produce semantically equivalent commands
2. **Given** a backend generates a different command variant, **When** the harness compares outputs, **Then** it reports the divergence with semantic analysis
3. **Given** one backend is unavailable, **When** running consistency tests, **Then** the harness skips that backend and reports which backends were tested

---

### User Story 3 - Benchmark Reporting and Metrics Dashboard (Priority: P3)

As a project stakeholder, I need to see evaluation metrics in a structured format so I can track quality trends over time and make data-driven decisions.

**Why this priority**: Reporting is valuable but not essential for initial validation. Console output is sufficient for MVP; structured output enables future dashboard integration.

**Independent Test**: Can be tested by running evaluation and verifying JSON output contains all required metrics (CSR, safety accuracy, POSIX compliance rate).

**Acceptance Scenarios**:

1. **Given** an evaluation run completes, **When** I check the output, **Then** I see JSON with timestamp, version, CSR, safety_accuracy, posix_compliance_rate
2. **Given** multiple evaluation runs over time, **When** I parse the JSON outputs, **Then** I can track CSR trends across releases
3. **Given** a failing test case, **When** the report is generated, **Then** it includes the failed prompt, expected command, actual command, and failure reason

---

### Edge Cases

- What happens when the test dataset file is missing or malformed?
  - System should fail gracefully with clear error message pointing to expected file location
- What happens when a backend timeout occurs during evaluation?
  - Individual test case should be marked as "error" (not "fail") and overall evaluation should continue
- What happens when the expected command in the dataset uses shell-specific syntax?
  - Evaluation should check for semantic equivalence (e.g., `ls -la` vs `ls -l -a`) rather than exact string matching
- How does the system handle commands that are correct but use different quoting styles?
  - POSIX validation should accept both single and double quotes when semantically equivalent
- What happens when a prompt generates multiple valid command alternatives?
  - Dataset should support marking multiple expected commands as correct for a single prompt

## Requirements

### Functional Requirements

- **FR-001**: System MUST load test dataset from TOML file containing prompt→command pairs with correctness/safety labels
- **FR-002**: System MUST execute evaluation against MLX backend (primary) with optional vLLM/Ollama support
- **FR-003**: System MUST validate command correctness by comparing generated output against expected command from dataset
- **FR-004**: System MUST detect safety violations using existing safety module pattern matching
- **FR-005**: System MUST validate POSIX compliance by checking for shell-specific features (bash/zsh syntax)
- **FR-006**: System MUST calculate Command Success Rate (CSR) as percentage of correct commands
- **FR-007**: System MUST integrate with `cargo test` framework as standard integration test
- **FR-008**: System MUST output evaluation results in JSON format with timestamp, version, metrics
- **FR-009**: System MUST support running evaluation on subset of test cases via test filter arguments
- **FR-010**: System MUST report per-test-case results including prompt, expected, actual, pass/fail status

### Key Entities

- **TestCase**: A single evaluation example containing prompt text, expected shell command, safety label (safe/unsafe), POSIX compliance label (compliant/non-compliant), and optional notes
- **EvaluationResult**: Output of running harness containing overall metrics (CSR, safety accuracy, POSIX compliance rate), per-test-case results, backend information, and timestamp
- **TestDataset**: Collection of TestCase entries loaded from TOML file, organized by category (correctness, safety, posix)

## Success Criteria

### Measurable Outcomes

- **SC-001**: Developers can run `cargo test --test evaluation` and see CSR metric for current codebase (target: maintain 94.8% baseline from ROADMAP.md)
- **SC-002**: Evaluation harness processes 50 test cases in under 2 minutes on M1 Mac with MLX backend
- **SC-003**: Safety detection accuracy is measured and reported (target: 100% detection of known dangerous patterns from safety module)
- **SC-004**: POSIX compliance validation catches known shell-specific syntax with 95%+ accuracy
- **SC-005**: JSON output contains all required fields (timestamp, caro_version, csr, safety_accuracy, posix_compliance_rate, per_case_results)
- **SC-006**: Multi-backend consistency testing reports divergence when backends produce different commands (P2 feature)
- **SC-007**: Test dataset is version-controlled and contains minimum 50 diverse examples covering common use cases (list files, find text, system info, etc.)

## Assumptions

*These assumptions were made programmatically based on ROADMAP.md research, Issue #135 requirements, and existing codebase knowledge:*

1. **Test Dataset Format**: Using TOML format for human readability and Rust tooling compatibility. Structure will be:
   ```toml
   [[test_cases]]
   prompt = "list all files"
   expected_command = "ls -la"
   category = "correctness"
   safe = true
   posix_compliant = true
   ```

2. **Integration with cargo test**: Evaluation harness will be implemented as `tests/evaluation/mod.rs` integration test, not a standalone binary. This leverages existing test infrastructure.

3. **Initial Test Set Size**: Starting with 50-100 labeled examples is sufficient for MVP. Can be expanded iteratively based on gap analysis.

4. **Metrics Output**: JSON format chosen for future dashboard integration. Console output will also display human-readable summary for immediate feedback.

5. **Backend Priority**: MLX is primary backend for evaluation (matches production usage on macOS). vLLM and Ollama support is P2 feature for multi-backend consistency testing.

6. **CSR Target**: Maintaining 94.8% baseline from ROADMAP.md is the v1.1.0 success target. Regression below 90% should block release.

7. **Test Execution Environment**: Evaluation tests will run in CI/CD pipeline as part of `cargo test` suite, ensuring quality gates before merge.

## Out of Scope

- Real-time dashboard UI (JSON output enables future integration)
- Automated test case generation from production logs (manual curation for MVP)
- Performance benchmarking (separate from correctness evaluation - covered by Issue #9)
- Command execution validation (harness tests generation, not runtime execution safety)
- Multi-language support (English prompts only for v1.1.0)
