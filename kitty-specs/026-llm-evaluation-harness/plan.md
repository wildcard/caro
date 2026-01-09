# Implementation Plan: LLM Evaluation Harness

**Branch**: `026-llm-evaluation-harness` | **Date**: 2026-01-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/kitty-specs/026-llm-evaluation-harness/spec.md`

**Note**: This document captures the complete implementation planning for Issue #135, including research findings, architectural decisions, and work breakdown strategy.

## Summary

Build a comprehensive LLM evaluation harness for systematic quality measurement of shell command generation across all caro backends. The harness tests four categories (correctness, safety, POSIX compliance, multi-backend consistency) using a labeled test dataset (100+ examples), runs in CI/CD with <5 minute execution time, and provides benchmark reports with regression detection.

**Technical Approach**:
- Trait-based evaluator architecture with category-specific implementations
- YAML test dataset for easy contribution
- Parallel execution via tokio for performance
- JSON baselines for regression detection
- cargo test integration for familiar CI/CD workflow
- Optional HTML dashboard for stakeholder visibility

## Technical Context

**Language/Version**: Rust 1.75+ (matches caro codebase requirement)
**Primary Dependencies**: tokio 1.x (async runtime), serde (YAML/JSON serialization), async-trait (trait async methods)
**Storage**: File-based (YAML dataset, JSON results/baselines)
**Testing**: cargo test with custom harness, integration tests for evaluators
**Target Platform**: Cross-platform (Linux, macOS, Windows) - matches caro's supported platforms
**Project Type**: Single project (library + test harness integrated into existing caro repo)
**Performance Goals**: <5 minutes full evaluation (100 tests × 4 backends), <3 seconds per individual test evaluation
**Constraints**:
  - Must use existing backend implementations (no modifications to backends)
  - Must reuse existing safety validation patterns for consistency
  - Must run on standard GitHub Actions runners (no special hardware)
  - Must handle backend unavailability gracefully (e.g., MLX on non-macOS)
**Scale/Scope**:
  - Initial: 100 test cases (25 per category)
  - Growth target: 200+ test cases over 3 months
  - 4 backends evaluated (StaticMatcher, MLX, Ollama, vLLM)
  - Historical trend tracking (baselines stored per evaluation run)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Note**: Constitution file is template-only. Applying standard Rust project best practices for caro project.

**Compliance Status**: ✅ PASS

| Principle | Status | Notes |
|-----------|--------|-------|
| Modularity | ✅ PASS | Clear separation: models, evaluators, harness orchestration |
| Testing | ✅ PASS | cargo test integration, unit tests for evaluators, integration tests for harness |
| Documentation | ✅ PASS | Comprehensive: spec.md, research.md, data-model.md, quickstart.md, API contracts |
| Performance | ✅ PASS | <5min target with parallel execution strategy documented |
| Maintainability | ✅ PASS | YAML dataset for easy contribution, trait-based extensibility |
| Integration | ✅ PASS | Uses existing backend trait, reuses safety validation patterns |

**Re-check after Phase 1**: No new concerns. All design decisions align with Rust best practices and caro's existing architecture patterns.

## Project Structure

### Documentation (this feature)

```
kitty-specs/026-llm-evaluation-harness/
├── spec.md              # ✅ Complete - Feature specification
├── plan.md              # ✅ This file - Implementation plan
├── research.md          # ✅ Complete - Phase 0 research findings
├── data-model.md        # ✅ Complete - Phase 1 entity definitions
├── quickstart.md        # ✅ Complete - Phase 1 developer guide
├── contracts/
│   └── evaluation-api.md # ✅ Complete - Phase 1 API contract
├── research/
│   ├── evidence-log.csv    # ✅ Complete - Evidence audit trail
│   └── source-register.csv # ✅ Complete - Source catalog
├── checklists/
│   └── requirements.md    # ✅ Complete - Spec quality validation
└── tasks.md             # ⏳ Next - Phase 2 output (/spec-kitty.tasks)
```

### Source Code (repository root)

**Structure Decision**: Single project (Option 1) - Integrated into existing caro monorepo

```
src/
├── evaluation/                    # 🆕 New evaluation harness module
│   ├── mod.rs                     # Module exports
│   ├── harness.rs                 # Main EvaluationHarness orchestrator
│   ├── models.rs                  # Core data structures (TestCase, EvaluationResult, etc.)
│   ├── dataset.rs                 # YAML dataset loading and validation
│   ├── baseline.rs                # Baseline comparison logic
│   ├── dashboard.rs               # HTML dashboard generation
│   ├── evaluators/                # Category-specific evaluators
│   │   ├── mod.rs
│   │   ├── correctness.rs         # CorrectnessEvaluator
│   │   ├── safety.rs              # SafetyEvaluator (uses existing safety module)
│   │   ├── posix.rs               # POSIXEvaluator
│   │   └── consistency.rs         # ConsistencyEvaluator (multi-backend)
│   └── utils.rs                   # Helper functions (command equivalence, pattern matching)
│
├── backends/                      # ✅ Existing - No modifications needed
│   ├── mod.rs
│   ├── backend_trait.rs           # Backend trait used by evaluators
│   ├── static_matcher.rs          # StaticMatcher backend
│   ├── mlx.rs                     # MLX backend
│   ├── ollama.rs                  # Ollama backend
│   └── vllm.rs                    # vLLM backend
│
├── safety/                        # ✅ Existing - Reused by SafetyEvaluator
│   ├── mod.rs
│   └── patterns.rs                # Safety pattern definitions
│
└── commands/                      # ✅ Existing - Used for CommandRequest/Result
    ├── mod.rs
    ├── request.rs                 # CommandRequest struct
    └── result.rs                  # CommandResult struct

tests/
├── evaluation/                    # 🆕 New evaluation test suite
│   ├── mod.rs
│   ├── main.rs                    # Custom test harness entry point
│   ├── dataset.yaml               # 🆕 Test case dataset (100+ examples)
│   ├── baselines/                 # 🆕 Baseline results for regression detection
│   │   ├── main-latest.json       # Symlink to most recent baseline
│   │   └── main-YYYY-MM-DD.json   # Timestamped baseline snapshots
│   ├── results/                   # 🆕 Individual evaluation run results
│   │   └── {run_id}.json          # Per-run benchmark reports
│   └── dashboard/                 # 🆕 Generated HTML dashboard
│       ├── index.html             # Main dashboard page
│       └── assets/                # Chart.js and styling
│
├── integration/                   # ✅ Existing - Add evaluator integration tests
│   └── evaluation_tests.rs        # 🆕 Integration tests for harness
│
└── unit/                          # ✅ Existing - Add evaluator unit tests
    └── evaluation/                # 🆕 Unit tests for evaluators
        ├── correctness_tests.rs
        ├── safety_tests.rs
        ├── posix_tests.rs
        └── consistency_tests.rs

.github/
└── workflows/
    └── evaluation.yml             # 🆕 CI workflow for evaluation harness
```

**Key Design Decisions**:
1. **Integrated Module**: Evaluation harness lives in `src/evaluation/` as a first-class module
2. **Reuse Existing Traits**: Uses `Backend` trait from `src/backends/`, no modifications needed
3. **Safety Integration**: `SafetyEvaluator` wraps existing `safety::patterns` for consistency
4. **Test Isolation**: All evaluation artifacts in `tests/evaluation/` directory
5. **No Runtime Dependencies**: Dashboard generation is static HTML, no server required

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

## Parallel Work Analysis

*Include this section if multiple developers/agents will implement this feature*

### Dependency Graph

```
[Identify what must be built sequentially vs what can be done in parallel]
Example:
Foundation (Day 1) → Wave 1 (Days 2-3, parallel) → Wave 2 (Days 4-5, parallel) → Integration (Day 6)
```

### Work Distribution

- **Sequential work**: [What must be done first before parallel work can begin]
- **Parallel streams**: [Independent work that can be done simultaneously]
- **Agent assignments**: [Who owns which files/modules to avoid conflicts]

### Coordination Points

- **Sync schedule**: [When parallel workers merge their changes]
- **Integration tests**: [How to verify parallel work integrates correctly]

## Implementation Summary

**Planning Phase Complete**: ✅ 2026-01-09

### Artifacts Generated

| Artifact | Status | Purpose |
|----------|--------|---------|
| spec.md | ✅ Complete | Feature requirements and success criteria |
| plan.md | ✅ Complete | This document - technical architecture and design |
| research.md | ✅ Complete | Architectural decisions with evidence |
| data-model.md | ✅ Complete | Entity definitions and relationships |
| quickstart.md | ✅ Complete | Developer onboarding guide |
| contracts/evaluation-api.md | ✅ Complete | API contract and interface definitions |
| research/evidence-log.csv | ✅ Complete | Research audit trail |
| research/source-register.csv | ✅ Complete | Source catalog |
| checklists/requirements.md | ✅ Complete | Specification quality validation |

### Key Decisions Summary

1. **Architecture**: Trait-based evaluators with parallel execution via tokio
2. **Integration**: Uses existing Backend trait, reuses safety validation patterns
3. **Dataset**: YAML format for ease of contribution (100+ test cases)
4. **Baseline**: JSON format for regression detection
5. **CI/CD**: cargo test integration with GitHub Actions matrix strategy
6. **Performance**: Parallel execution achieves <5min target (vs 20min sequential)
7. **Prioritization**: Deep testing on MLX + StaticMatcher, basic coverage on Ollama/vLLM

### Next Steps

1. **Task Breakdown**: Run `/spec-kitty.tasks` to generate work packages
2. **Implementation**: Begin with core infrastructure (models, dataset loading)
3. **Incremental Delivery**: Implement evaluators one category at a time
4. **Testing**: TDD approach - write evaluator tests before implementation
5. **Integration**: Add CI workflow after core functionality is working

### Success Metrics (from spec.md)

- ✅ **SC-001**: Evaluation completes in <5 minutes ← Parallel execution strategy
- ✅ **SC-002**: 95%+ regression detection accuracy ← Baseline comparison logic
- ✅ **SC-003**: Zero-code test case addition ← YAML dataset format
- ✅ **SC-004**: 2min insight extraction ← JSON reports + HTML dashboard
- ✅ **SC-005**: 99%+ CI uptime ← Graceful backend failure handling
- ✅ **SC-006**: Stakeholder-friendly dashboard ← Static HTML with Chart.js
- ✅ **SC-007**: <50 LOC per new backend ← Trait-based extensibility
- ✅ **SC-008**: Dataset growth to 200+ ← Easy YAML contribution

All success criteria have clear implementation paths defined in this plan.

### Risk Mitigation

| Risk | Mitigation Strategy |
|------|-------------------|
| Performance <5min | Parallel execution + backend sampling for Ollama/vLLM |
| LLM non-determinism | Pattern matching + equivalence checking in validation rules |
| Backend unavailability | Platform detection + graceful skipping |
| Test dataset staleness | Easy YAML format + bug report integration workflow |
| False positive regressions | 5% threshold + statistical significance in comparison |

### Dependencies

**External**:
- GitHub Issue #135 ✅ (feature request)
- Beta testing data ✅ (from .claude/releases)
- Existing backend implementations ✅ (no modifications needed)

**Internal** (within caro codebase):
- Backend trait (src/backends/) ✅
- Safety patterns (src/safety/) ✅
- CommandRequest/Result (src/commands/) ✅
- Tokio runtime ✅

All dependencies are satisfied. Ready for implementation.

---

**Planning Phase Status**: COMPLETE ✅

**Ready for**: `/spec-kitty.tasks` to generate work package breakdown

**Estimated Complexity**: Medium-High (new subsystem, but well-defined scope and clear architecture)

**Estimated Timeline**: 2-3 weeks for full implementation with comprehensive test coverage
