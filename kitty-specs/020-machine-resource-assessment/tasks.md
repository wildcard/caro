# Work Packages: Machine Resource Assessment

**Feature**: Issue #147 - Machine Resource Assessment
**Branch**: `020-machine-resource-assessment`
**Created**: 2026-01-08
**Status**: In Planning

---

## Overview

This feature adds a `caro assess` command that detects system resources (CPU, GPU, memory) and provides model recommendations based on hardware capabilities.

**Total Work Packages**: 5
**Estimated Scope**: ~1000-1500 LOC across 5 new modules

---

## Work Package Summary

| ID | Title | Priority | Status | Subtasks |
|----|-------|----------|--------|----------|
| WP01 | Foundation - Core Detection | P1 | planned | 8 |
| WP02 | Platform-Specific GPU Detection | P1 | planned | 6 |
| WP03 | Recommendation Engine | P2 | planned | 5 |
| WP04 | Output Formatting & Export | P2 | planned | 7 |
| WP05 | Testing & Integration | P3 | planned | 6 |

---

## Setup Phase

### Work Package 01: Foundation - Core Detection

**Priority**: P1 (Critical path - required for all other work)
**Goal**: Implement basic system profiling with CPU and memory detection
**Independent Test**: Can verify CPU/memory detection works independently on all platforms

**Subtasks**:
- [ ] T001: Add `sysinfo` dependency to Cargo.toml (v0.30+)
- [ ] T002 [P]: Create `src/assessment/mod.rs` module structure
- [ ] T003: Implement `SystemProfile` struct in `src/assessment/profile.rs`
- [ ] T004 [P]: Implement `CPUInfo` struct and detection in `src/assessment/cpu.rs`
- [ ] T005 [P]: Implement `MemoryInfo` struct and detection in `src/assessment/memory.rs`
- [ ] T006: Create `src/cli/commands/assess.rs` CLI command handler
- [ ] T007: Register `assess` command in `src/cli/mod.rs`
- [ ] T008: Implement basic human-readable output formatter

**Implementation Sketch**:
1. Add dependencies (T001)
2. Create module structure (T002, T003)
3. Implement detection (T004, T005 can run in parallel)
4. Wire up CLI command (T006, T007)
5. Add basic output (T008)

**Success Criteria**:
- `cargo assess` runs without error
- Displays CPU architecture, cores, model name
- Displays total and available RAM
- Works on macOS, Linux, Windows

**Dependencies**: None (foundation work)

**Risks**:
- `sysinfo` crate may not detect all CPU features consistently across platforms
- Mitigation: Focus on core fields (arch, cores), gracefully handle missing data

**Prompt File**: `tasks/planned/WP01-foundation-core-detection.md`

---

## Core Implementation Phase

### Work Package 02: Platform-Specific GPU Detection

**Priority**: P1 (Required for complete assessment)
**Goal**: Detect GPU hardware across macOS, Linux, and Windows platforms
**Independent Test**: Can verify GPU detection works on systems with and without GPUs

**Subtasks**:
- [ ] T009: Implement `GPUInfo` struct in `src/assessment/gpu.rs`
- [ ] T010: Add macOS GPU detection (Metal framework or system_profiler)
- [ ] T011: Add Linux GPU detection (NVIDIA focus with nvml-wrapper)
- [ ] T012: Add Windows GPU detection (WMI queries)
- [ ] T013: Implement graceful fallback for missing/unsupported GPUs
- [ ] T014: Update `SystemProfile` to include GPU info

**Implementation Sketch**:
1. Define GPU data structures (T009, T014)
2. Implement platform-specific detection (T010, T011, T012 - sequential by platform)
3. Add error handling and fallback (T013)

**Success Criteria**:
- Detects Apple Silicon GPUs on macOS
- Detects NVIDIA GPUs on Linux
- Detects GPUs via WMI on Windows
- Reports "No GPU detected" gracefully when missing
- Displays vendor, model, VRAM when available

**Dependencies**: WP01 (requires SystemProfile)

**Parallel Opportunities**: Platform-specific implementations (T010-T012) can be done in parallel if multiple agents available

**Risks**:
- GPU detection may require elevated permissions on some platforms
- Detection accuracy varies by driver/OS version
- Mitigation: Comprehensive error handling, clear user warnings

**Prompt File**: `tasks/planned/WP02-gpu-detection.md`

---

### Work Package 03: Recommendation Engine

**Priority**: P2 (Builds on detection to provide value)
**Goal**: Generate model recommendations based on detected hardware profiles
**Independent Test**: Can verify recommendations match expected models for hardware tiers

**Subtasks**:
- [ ] T015: Define hardware profile tiers (low/mid/high-end) with thresholds
- [ ] T016: Create `ModelRecommendation` struct in `src/assessment/recommender.rs`
- [ ] T017: Implement recommendation algorithm with rule-based logic
- [ ] T018: Add model catalog data (Phi, Llama, Mistral families)
- [ ] T019: Generate reasoning text for each recommendation

**Implementation Sketch**:
1. Define tiers and thresholds (T015)
2. Create recommendation data structures (T016)
3. Implement mapping algorithm (T017)
4. Add model catalog (T018)
5. Generate human-readable reasoning (T019)

**Success Criteria**:
- Recommends lightweight models (Phi-2, TinyLlama) for < 8GB RAM
- Recommends MLX backend for Apple Silicon
- Recommends CUDA backend for NVIDIA GPUs
- Recommends CPU-only for systems without GPUs
- Includes reasoning ("Based on 16GB RAM and M1 GPU...")

**Dependencies**: WP01, WP02 (requires complete SystemProfile)

**Risks**:
- Recommendation logic may become outdated as new models release
- Mitigation: Separate model catalog from algorithm for easy updates

**Prompt File**: `tasks/planned/WP03-recommendation-engine.md`

---

## Polish Phase

### Work Package 04: Output Formatting & Export

**Priority**: P2 (User-facing polish)
**Goal**: Provide human-readable output and JSON/Markdown export options
**Independent Test**: Can verify output formats are complete and well-formatted

**Subtasks**:
- [ ] T020: Create `AssessmentResult` wrapper struct
- [ ] T021: Implement human-readable formatter with ASCII borders
- [ ] T022: Add JSON export with `--export json` flag
- [ ] T023: Add Markdown export with `--export markdown` flag
- [ ] T024: Implement `--recommendations-only` flag
- [ ] T025: Add CLI argument parsing for export options
- [ ] T026: Handle file writing with error handling

**Implementation Sketch**:
1. Create result wrapper (T020)
2. Implement formatters (T021, T022, T023 can run in parallel)
3. Add CLI flags (T024, T025)
4. Wire up file export (T026)

**Success Criteria**:
- Default output uses ASCII borders and clear formatting
- `--export json` creates valid JSON file
- `--export markdown` creates formatted markdown file
- `--recommendations-only` shows just recommendations
- Exports include all detected data and recommendations

**Dependencies**: WP03 (requires ModelRecommendation data)

**Parallel Opportunities**: Formatters (T021-T023) can be implemented in parallel

**Risks**:
- Export file paths may not exist or lack write permissions
- Mitigation: Clear error messages, suggest alternative paths

**Prompt File**: `tasks/planned/WP04-output-formatting.md`

---

### Work Package 05: Testing & Integration

**Priority**: P3 (Quality assurance)
**Goal**: Comprehensive test coverage and integration validation
**Independent Test**: Can verify all tests pass on CI/CD pipeline

**Subtasks**:
- [ ] T027: Create mock system profiles in `tests/fixtures/mock_profiles.rs`
- [ ] T028: Write unit tests for CPU detection (`tests/assessment_tests.rs`)
- [ ] T029: Write unit tests for memory detection
- [ ] T030: Write unit tests for recommendation algorithm
- [ ] T031: Write integration test for end-to-end assessment command
- [ ] T032: Add documentation to README.md for `caro assess` command

**Implementation Sketch**:
1. Create test fixtures (T027)
2. Write unit tests (T028, T029, T030 can run in parallel)
3. Write integration test (T031)
4. Document feature (T032)

**Success Criteria**:
- All unit tests pass on macOS, Linux, Windows
- Integration test validates complete workflow
- Mocked profiles cover low/mid/high-end hardware
- Documentation explains all CLI flags and output formats
- `cargo test` completes in < 30 seconds

**Dependencies**: WP01-WP04 (requires complete implementation)

**Parallel Opportunities**: All unit tests (T028-T030) can be written in parallel

**Risks**:
- Integration tests may fail on CI if hardware detection differs
- Mitigation: Use mocked profiles for consistent test results

**Prompt File**: `tasks/planned/WP05-testing-integration.md`

---

## Execution Order

### Sequential Dependencies
```
WP01 (Foundation)
  ↓
WP02 (GPU Detection) ← depends on WP01
  ↓
WP03 (Recommendation Engine) ← depends on WP01 + WP02
  ↓
WP04 (Output Formatting) ← depends on WP03
  ↓
WP05 (Testing) ← depends on WP01-WP04
```

### Parallel Opportunities
- Within WP01: T004 (CPU) and T005 (Memory) can run in parallel
- Within WP02: T010 (macOS), T011 (Linux), T012 (Windows) can run in parallel
- Within WP04: T021 (human-readable), T022 (JSON), T023 (Markdown) can run in parallel
- Within WP05: T028, T029, T030 (unit tests) can run in parallel

---

## MVP Scope

**Minimum viable product**: WP01 + WP02 (basic assessment without recommendations)

This delivers core value:
- CPU/Memory/GPU detection
- Basic human-readable output
- Works across all platforms

Users can verify their system specs even without recommendations.

---

## Notes

- Testing is optional per user directive, but included for completeness
- All subtasks are implementation-focused (no research tasks)
- Platform-specific work isolated to WP02 for clean separation
- Recommendation logic in WP03 is extensible for future enhancements

---

**Tasks Status**: ✅ Complete (ready for `/spec-kitty.implement`)
**Last Updated**: 2026-01-08
**Next Command**: `/spec-kitty.implement`
