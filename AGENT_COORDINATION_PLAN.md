# Multi-Agent Implementation Coordination Plan

**Objective**: Execute the MLX Backend Implementation Plan using specialized sub-agents working in parallel under master coordination.

**Coordinator**: Main orchestration agent
**Strategy**: Parallel execution with context-efficient specialization
**Timeline**: 9-16 hours of work parallelized across 5 agents

---

## Executive Summary

This plan decomposes the MLX backend implementation into 5 parallel workstreams, each handled by a specialized agent:

1. **Workstream 1: Candle Proof of Concept** → rust-production-architect
2. **Workstream 2: Backend Implementation** → llm-integration-expert
3. **Workstream 3: Testing Infrastructure** → qa-testing-expert
4. **Workstream 4: Configuration & Dependencies** → rust-cli-expert
5. **Workstream 5: Documentation** → technical-writer

**Key Principle**: Each agent works independently but can consult with coordinator. All work is integrated at defined checkpoints.

---

## Workstream Dependencies

```
┌─────────────────────────────────────────────────────────────┐
│                    PHASE 0: PREPARATION                     │
│              (Coordinator sets up structure)                │
└──────────────┬──────────────────────────────────────────────┘
               │
    ┌──────────┼──────────┬──────────┬──────────┐
    │          │          │          │          │
    ▼          ▼          ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│  WS1   │ │  WS2   │ │  WS3   │ │  WS4   │ │  WS5   │
│  PoC   │ │Backend │ │Testing │ │Config  │ │  Docs  │
│        │ │        │ │        │ │        │ │        │
│ 2-4h   │ │ 3-5h   │ │ 2-3h   │ │ 1-2h   │ │ 1-2h   │
└────┬───┘ └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘
     │         │          │          │          │
     └─────────┴──────────┴──────────┴──────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  INTEGRATION PHASE  │
              │   (Coordinator)     │
              └─────────────────────┘
```

**Critical Path**: WS1 (PoC) must validate approach before WS2 proceeds to full implementation

**Parallel Work**: WS3, WS4, WS5 can start immediately (don't depend on PoC)

---

## Workstream 1: Candle Proof of Concept

**Agent**: rust-production-architect
**Priority**: CRITICAL (blocks WS2)
**Duration**: 2-4 hours
**Dependencies**: None

### Objective
Create a standalone proof-of-concept that demonstrates Candle Metal backend working on Apple Silicon with real model inference.

### Deliverables
1. ✅ `examples/candle_poc.rs` - Minimal working inference example
2. ✅ Proof that Candle Metal works on M4 Max
3. ✅ Documentation of PoC setup and results
4. ✅ Performance baseline (initialization, inference time)

### Success Criteria
- [ ] Code compiles with `--features metal`
- [ ] Model loads to Metal GPU successfully
- [ ] Inference produces valid JSON command output
- [ ] Total execution time < 5 seconds
- [ ] Clear documentation of what works

### Handoff to WS2
- If PoC succeeds → WS2 proceeds with full integration
- If PoC fails → Coordinator reassesses strategy

### Checkpoint
**Checkpoint 1**: PoC results ready for review (after 2-4 hours)

---

## Workstream 2: Backend Implementation

**Agent**: llm-integration-expert
**Priority**: HIGH (core functionality)
**Duration**: 3-5 hours
**Dependencies**: WS1 PoC validation

### Objective
Replace placeholder code in embedded backends with real Candle inference using learnings from PoC.

### Deliverables
1. ✅ `src/backends/embedded/cpu.rs` - Real Candle CPU inference
2. ✅ `src/backends/embedded/metal.rs` - Metal GPU wrapper (if needed)
3. ✅ Updated `CandleModelState` with real types
4. ✅ Model loading from GGUF files
5. ✅ Tokenization integration
6. ✅ JSON response parsing with fallbacks

### Success Criteria
- [ ] `cargo build --features embedded-cpu` compiles
- [ ] `cargo build --features embedded-metal` compiles on macOS
- [ ] Unit tests pass for both backends
- [ ] Inference returns real model output (not hardcoded)
- [ ] Metal device auto-detection works

### Files Modified
- `src/backends/embedded/cpu.rs` (lines 11-14, 41-81, 87-126)
- `src/backends/embedded/mod.rs` (add metal module)
- `src/backends/embedded/common.rs` (update types if needed)

### Checkpoint
**Checkpoint 2**: Backend implementation complete and compiling (after 3-5 hours)

---

## Workstream 3: Testing Infrastructure

**Agent**: qa-testing-expert
**Priority**: HIGH (validation)
**Duration**: 2-3 hours
**Dependencies**: None (can start immediately with stubs)

### Objective
Create comprehensive test suite for embedded backends, enable ignored tests, add integration tests.

### Deliverables
1. ✅ `tests/embedded_backend_contract.rs` - Rename and update from mlx_backend_contract.rs
2. ✅ Remove `#[ignore]` from 6 performance tests
3. ✅ `tests/integration/embedded_inference.rs` - End-to-end integration tests
4. ✅ `benches/metal_vs_cpu.rs` - Performance benchmark suite
5. ✅ Test utilities for model setup

### Success Criteria
- [ ] All 10 contract tests run (not ignored)
- [ ] Contract tests pass on Apple Silicon
- [ ] Integration tests validate full inference pipeline
- [ ] Benchmarks compare Metal vs CPU performance
- [ ] Test coverage > 80% for embedded backends

### Files Created/Modified
- `tests/mlx_backend_contract.rs` → `tests/embedded_backend_contract.rs`
- `tests/integration/embedded_inference.rs` (new)
- `benches/metal_vs_cpu.rs` (new)

### Checkpoint
**Checkpoint 3**: Test suite ready for backend validation (after 2-3 hours)

---

## Workstream 4: Configuration & Dependencies

**Agent**: rust-cli-expert
**Priority**: MEDIUM (supporting)
**Duration**: 1-2 hours
**Dependencies**: None (can start immediately)

### Objective
Update all configuration files, dependency declarations, and feature flags to support Candle Metal backend.

### Deliverables
1. ✅ Updated `Cargo.toml` with correct features
2. ✅ Updated `.github/workflows/ci.yml` for Metal builds
3. ✅ Updated build scripts if needed
4. ✅ Feature flag validation

### Success Criteria
- [ ] `cargo build --features embedded-metal` works on macOS
- [ ] `cargo build --features embedded-cpu` works cross-platform
- [ ] CI builds pass on all platforms
- [ ] No mlx-rs dependencies remain
- [ ] Feature flags are correctly gated

### Files Modified
- `Cargo.toml` (lines 70-100)
- `.github/workflows/ci.yml` (if needed)
- `.github/workflows/macos-apple-silicon.yml` (update features)

### Checkpoint
**Checkpoint 4**: Configuration validated and CI passing (after 1-2 hours)

---

## Workstream 5: Documentation

**Agent**: technical-writer
**Priority**: MEDIUM (polish)
**Duration**: 1-2 hours
**Dependencies**: None (can start with plan, finalize after implementation)

### Objective
Update all documentation to reflect Candle Metal backend approach and provide clear user/developer guidance.

### Deliverables
1. ✅ Updated `CLAUDE.md` - Reflects Candle strategy
2. ✅ Updated `README.md` - Performance claims, quickstart
3. ✅ `docs/QUICKSTART_METAL.md` - Getting started with Metal backend
4. ✅ Updated contract documentation
5. ✅ API documentation for new types

### Success Criteria
- [ ] Documentation mentions Candle (not mlx-rs)
- [ ] Quickstart guide tested on clean M4 Max
- [ ] Performance claims are accurate
- [ ] Code examples compile and run
- [ ] All TODOs removed or tracked

### Files Modified
- `CLAUDE.md` (Apple Silicon section)
- `README.md` (Features, Quick Start)
- `docs/QUICKSTART_METAL.md` (new)
- `specs/004-implement-ollama-and/contracts/mlx-backend.md` → rename and update

### Checkpoint
**Checkpoint 5**: Documentation complete and validated (after 1-2 hours)

---

## Master Prompts for Each Agent

### Master Prompt: WS1 - Candle PoC (rust-production-architect)

```
ROLE: You are implementing a proof-of-concept for Candle Metal backend on Apple Silicon.

CONTEXT:
- Project: cmdai (Rust CLI for NL to shell commands)
- Current state: MLX backend is placeholder code (no real inference)
- Strategic decision: Pivot from mlx-rs to Candle Metal (faster, more mature)
- Reference: MLX_BACKEND_IMPLEMENTATION_PLAN.md Phase 1

OBJECTIVE:
Create a minimal working example (examples/candle_poc.rs) that proves Candle Metal
can load a GGUF model and run inference on Apple Silicon.

REQUIREMENTS:
1. Single file: examples/candle_poc.rs
2. Uses Candle with Metal backend
3. Loads Qwen2.5-Coder-1.5B-Instruct GGUF model
4. Runs inference on sample prompt
5. Outputs JSON command format
6. Completes in < 5 seconds

DELIVERABLES:
- examples/candle_poc.rs (fully working code)
- README_POC.md (setup instructions and results)
- Performance metrics (load time, inference time)

SUCCESS CRITERIA:
- Compiles with: cargo run --example candle_poc --release --features metal
- Outputs: "✓ INFERENCE WORKING!"
- Produces valid JSON: {"cmd": "..."}

CONSTRAINTS:
- Must use Candle (not mlx-rs)
- Must work on macOS Apple Silicon
- Keep it minimal (< 200 lines)
- Document any failures clearly

START HERE:
1. Read MLX_BACKEND_IMPLEMENTATION_PLAN.md Phase 1
2. Check current Cargo.toml dependencies
3. Create examples/candle_poc.rs following the plan
4. Test on M4 Max (or document assumptions)
5. Report results

CONSULT COORDINATOR IF:
- Candle API has changed significantly
- Model loading fails
- Metal device initialization fails
- Any blocking issues arise
```

### Master Prompt: WS2 - Backend Implementation (llm-integration-expert)

```
ROLE: You are implementing the core Candle backend integration for cmdai.

CONTEXT:
- WS1 PoC has validated that Candle Metal works
- Current code: src/backends/embedded/cpu.rs has placeholder inference
- Need: Replace placeholders with real Candle model loading and inference
- Reference: MLX_BACKEND_IMPLEMENTATION_PLAN.md Phase 2

OBJECTIVE:
Replace placeholder code in embedded backends with production-ready Candle inference.

REQUIREMENTS:
1. Update CandleModelState struct with real Candle types
2. Implement real model loading in load() method
3. Implement real inference in infer() method
4. Add tokenization using tokenizers crate
5. Implement JSON parsing with fallback strategies
6. Auto-detect Metal vs CPU device

DELIVERABLES:
- Updated src/backends/embedded/cpu.rs
- New src/backends/embedded/metal.rs (if needed as wrapper)
- Updated src/backends/embedded/common.rs (if types changed)
- Unit tests passing

SUCCESS CRITERIA:
- cargo build --features embedded-cpu compiles
- cargo build --features embedded-metal compiles (macOS)
- cargo test --features embedded-cpu passes
- Inference returns real model output
- Device auto-detection works (Metal on macOS, CPU elsewhere)

FILES TO MODIFY:
- src/backends/embedded/cpu.rs (lines 11-14, 41-81, 87-126)
- src/backends/embedded/mod.rs (exports)
- Add src/backends/embedded/metal.rs if needed

CONSTRAINTS:
- Follow the code structure from PoC
- Maintain async/await patterns
- Preserve error handling
- Keep Arc<Mutex<>> for thread safety

START HERE:
1. Wait for WS1 PoC validation
2. Read PoC code to understand working patterns
3. Update CandleModelState with real types
4. Implement load() with GGUF loading
5. Implement infer() with real inference
6. Test compilation and basic functionality

CONSULT COORDINATOR IF:
- PoC patterns don't translate to backend structure
- Type system issues with Candle types
- Performance issues arise
- Integration challenges with existing code
```

### Master Prompt: WS3 - Testing Infrastructure (qa-testing-expert)

```
ROLE: You are creating comprehensive test infrastructure for embedded backends.

CONTEXT:
- Project has MLX contract tests with 6 marked #[ignore]
- Need to enable all tests and validate real implementation
- Create integration tests for end-to-end workflows
- Add performance benchmarks
- Reference: MLX_BACKEND_IMPLEMENTATION_PLAN.md Phase 4

OBJECTIVE:
Create production-ready test suite that validates embedded backend functionality.

REQUIREMENTS:
1. Rename tests/mlx_backend_contract.rs → tests/embedded_backend_contract.rs
2. Remove #[ignore] from 6 performance tests
3. Update test expectations for Candle (not MLX)
4. Create integration tests for full inference pipeline
5. Create benchmarks comparing Metal vs CPU
6. Add test utilities for model setup

DELIVERABLES:
- tests/embedded_backend_contract.rs (updated, no #[ignore])
- tests/integration/embedded_inference.rs (new)
- benches/metal_vs_cpu.rs (new)
- Test helper utilities

SUCCESS CRITERIA:
- All 10 contract tests run without #[ignore]
- Tests pass on Apple Silicon (when backend is ready)
- Integration tests cover: model load, inference, JSON parsing
- Benchmarks measure: initialization, inference, throughput
- Test coverage > 80% for embedded modules

FILES TO CREATE/MODIFY:
- tests/mlx_backend_contract.rs → tests/embedded_backend_contract.rs
- tests/integration/embedded_inference.rs (new)
- benches/metal_vs_cpu.rs (new)

CONSTRAINTS:
- Tests should work with stub initially, pass with real implementation
- Use #[cfg(target_os = "macos")] for Metal-specific tests
- Benchmarks should use Criterion
- Integration tests should download model if needed

START HERE:
1. Read current tests/mlx_backend_contract.rs
2. Plan test updates for Candle backend
3. Create test structure that works with stubs
4. Update tests as backend implementation progresses
5. Finalize when backend is complete

CONSULT COORDINATOR IF:
- Unclear what tests should validate
- Test failures indicate design issues
- Need clarification on performance targets
```

### Master Prompt: WS4 - Configuration (rust-cli-expert)

```
ROLE: You are updating configuration and build setup for Candle Metal backend.

CONTEXT:
- Project currently has mlx-rs in dependencies
- Need to remove mlx-rs, use only Candle
- Update feature flags for embedded-metal
- Update CI/CD for proper builds
- Reference: MLX_BACKEND_IMPLEMENTATION_PLAN.md Phase 3

OBJECTIVE:
Update all configuration files to support Candle Metal backend correctly.

REQUIREMENTS:
1. Remove mlx-rs from Cargo.toml
2. Update candle-core dependencies with Metal features
3. Create embedded-metal feature flag
4. Update CI workflows for Metal builds
5. Validate feature flag combinations

DELIVERABLES:
- Updated Cargo.toml
- Updated .github/workflows/ci.yml (if needed)
- Updated .github/workflows/macos-apple-silicon.yml
- Feature flag validation tests

SUCCESS CRITERIA:
- cargo build --features embedded-cpu works (all platforms)
- cargo build --features embedded-metal works (macOS only)
- cargo build --features full compiles
- CI builds pass on all platforms
- No mlx-rs references remain

FILES TO MODIFY:
- Cargo.toml (dependencies, features)
- .github/workflows/macos-apple-silicon.yml (feature flags)
- .github/workflows/ci.yml (if needed)

CONSTRAINTS:
- embedded-metal should only work on macOS aarch64
- embedded-cpu should be cross-platform
- Feature flags should be composable
- CI must validate all combinations

START HERE:
1. Read current Cargo.toml
2. Plan dependency updates
3. Update [dependencies] section
4. Update [features] section
5. Update platform-specific dependencies
6. Test feature flag combinations
7. Update CI workflows

CONSULT COORDINATOR IF:
- Dependency conflicts arise
- Feature flag design questions
- CI configuration issues
```

### Master Prompt: WS5 - Documentation (technical-writer)

```
ROLE: You are updating all documentation for the Candle Metal backend.

CONTEXT:
- Documentation currently mentions MLX/mlx-rs
- Strategic pivot to Candle Metal
- Need to update all docs, guides, and code comments
- Reference: MLX_BACKEND_IMPLEMENTATION_PLAN.md Phase 5

OBJECTIVE:
Create comprehensive documentation for Candle Metal backend implementation.

REQUIREMENTS:
1. Update CLAUDE.md to reflect Candle strategy
2. Update README.md with accurate performance claims
3. Create quickstart guide for Metal backend
4. Update contract documentation
5. Add code comments and API docs

DELIVERABLES:
- Updated CLAUDE.md
- Updated README.md
- docs/QUICKSTART_METAL.md (new)
- Updated contract docs
- Inline code documentation

SUCCESS CRITERIA:
- All MLX references changed to Candle
- Quickstart guide tested on clean M4 Max
- Performance claims are accurate (based on benchmarks)
- Code examples compile and run
- No broken links or outdated information

FILES TO MODIFY:
- CLAUDE.md (Apple Silicon section)
- README.md (Features, Quick Start, Performance)
- specs/004-implement-ollama-and/contracts/mlx-backend.md → rename
- Create docs/QUICKSTART_METAL.md

CONSTRAINTS:
- Must be technically accurate
- Examples must be tested
- Should match project tone and style
- Follow existing documentation structure

START HERE:
1. Read MLX_BACKEND_IMPLEMENTATION_PLAN.md
2. Audit current documentation for MLX references
3. Draft updates (can start immediately)
4. Finalize after implementation is complete
5. Validate all examples work

CONSULT COORDINATOR IF:
- Technical details unclear
- Performance claims need validation
- Documentation structure questions
```

---

## Coordination Checkpoints

### Checkpoint 1: PoC Validation (After 2-4 hours)
**Gate**: WS1 completion
**Decision Point**: Proceed with WS2 or reassess strategy

**Success Criteria**:
- [ ] PoC code compiles and runs
- [ ] Inference produces valid output
- [ ] Performance meets targets (<5s total)

**Actions**:
- If successful: Green-light WS2 to proceed
- If failed: Coordinator reviews issues, adjusts strategy

### Checkpoint 2: Backend Implementation (After 3-5 hours)
**Gate**: WS2 completion
**Affected**: WS3 (tests need backend to validate)

**Success Criteria**:
- [ ] Backend code compiles
- [ ] Unit tests pass
- [ ] Basic inference works

**Actions**:
- WS3 can finalize and run integration tests
- WS4 validates feature flags with real code

### Checkpoint 3: Testing Complete (After 2-3 hours)
**Gate**: WS3 completion
**Validation**: All tests passing

**Success Criteria**:
- [ ] Contract tests pass
- [ ] Integration tests pass
- [ ] Benchmarks run successfully

**Actions**:
- Coordinator reviews test results
- Identifies any issues for fixes

### Checkpoint 4: Configuration Validated (After 1-2 hours)
**Gate**: WS4 completion
**Affected**: CI/CD pipeline

**Success Criteria**:
- [ ] All feature combinations build
- [ ] CI workflows pass
- [ ] No dependency conflicts

**Actions**:
- Trigger CI runs to validate
- Fix any configuration issues

### Checkpoint 5: Documentation Complete (After 1-2 hours)
**Gate**: WS5 completion
**Final**: Ready for release

**Success Criteria**:
- [ ] All docs updated
- [ ] Examples tested
- [ ] No broken links

**Actions**:
- Final review of all documentation
- Prepare for merge/release

---

## Integration Phase

**After all checkpoints pass**, coordinator integrates all work:

### Integration Steps

1. **Code Integration**
   - Merge WS2 backend implementation
   - Apply WS4 configuration changes
   - Validate builds

2. **Test Integration**
   - Run WS3 full test suite
   - Validate all tests pass
   - Review benchmark results

3. **Documentation Integration**
   - Apply WS5 documentation updates
   - Validate all examples
   - Update README

4. **Final Validation**
   - Clean build test
   - Full test suite run
   - CI/CD validation
   - Performance benchmarks

5. **Commit & Push**
   - Squash commits if needed
   - Write comprehensive commit message
   - Push to feature branch
   - Create PR

---

## Communication Protocol

### Agent → Coordinator

**When to consult**:
- Blocking issues encountered
- Design decisions needed
- Dependencies on other workstreams
- Validation of approach

**How to consult**:
- Document the issue clearly
- Provide context and attempted solutions
- Ask specific questions
- Wait for coordinator response

### Coordinator → Agents

**Regular updates**:
- Checkpoint status
- Other workstream progress
- Integration timeline
- Priority adjustments

---

## Risk Management

### Risk 1: PoC Fails
**Impact**: HIGH (blocks WS2)
**Mitigation**:
- WS3, WS4, WS5 continue (not dependent)
- Coordinator reassesses Candle vs alternative
- Fallback: Implement with CPU-only first

### Risk 2: Backend Integration Issues
**Impact**: HIGH (blocks completion)
**Mitigation**:
- WS1 PoC validates approach first
- Incremental implementation
- Coordinator reviews early

### Risk 3: Test Failures
**Impact**: MEDIUM (indicates bugs)
**Mitigation**:
- Tests written before implementation
- Contract tests define expected behavior
- Early test runs identify issues

### Risk 4: Configuration Conflicts
**Impact**: MEDIUM (blocks builds)
**Mitigation**:
- WS4 starts early
- Validate incrementally
- Use existing CI as baseline

### Risk 5: Timeline Overrun
**Impact**: LOW (work is parallelized)
**Mitigation**:
- Independent workstreams
- Each has time buffer
- Can adjust scope if needed

---

## Success Metrics

### Code Quality
- [ ] All code compiles without warnings
- [ ] Clippy passes with `-D warnings`
- [ ] Formatting consistent

### Functionality
- [ ] Model loads successfully
- [ ] Inference produces valid output
- [ ] JSON parsing works
- [ ] Device auto-detection works

### Performance
- [ ] Model loading < 100ms
- [ ] First inference < 2s
- [ ] Binary size < 50MB

### Testing
- [ ] All contract tests pass
- [ ] Integration tests pass
- [ ] Test coverage > 80%
- [ ] Benchmarks complete

### Documentation
- [ ] All docs updated
- [ ] Examples tested
- [ ] No MLX references remain

---

## Timeline Summary

| Workstream | Agent | Duration | Dependencies |
|------------|-------|----------|--------------|
| WS1: PoC | rust-production-architect | 2-4h | None |
| WS2: Backend | llm-integration-expert | 3-5h | WS1 |
| WS3: Testing | qa-testing-expert | 2-3h | None (finalize after WS2) |
| WS4: Config | rust-cli-expert | 1-2h | None |
| WS5: Docs | technical-writer | 1-2h | None (finalize after all) |
| Integration | Coordinator | 1-2h | All WS complete |

**Total Parallel Time**: ~6-9 hours (instead of 9-16 hours sequential)
**Critical Path**: WS1 → WS2 → Integration (6-11 hours)

---

## Launch Sequence

### Phase 1: Launch Independent Workstreams (Parallel)
```bash
# Launch immediately (no dependencies)
- WS3: Testing Infrastructure
- WS4: Configuration
- WS5: Documentation (draft mode)
```

### Phase 2: Launch PoC (Critical Path)
```bash
# Launch after Phase 1
- WS1: Candle PoC
```

### Phase 3: Launch Backend Implementation
```bash
# Launch after WS1 completes successfully
- WS2: Backend Implementation
```

### Phase 4: Integration
```bash
# Coordinator integrates all deliverables
- Merge all changes
- Run final validation
- Commit and push
```

---

## Deliverables Checklist

### WS1 Deliverables
- [ ] examples/candle_poc.rs
- [ ] README_POC.md
- [ ] Performance metrics documented

### WS2 Deliverables
- [ ] Updated src/backends/embedded/cpu.rs
- [ ] New src/backends/embedded/metal.rs (if needed)
- [ ] Unit tests passing

### WS3 Deliverables
- [ ] tests/embedded_backend_contract.rs (no #[ignore])
- [ ] tests/integration/embedded_inference.rs
- [ ] benches/metal_vs_cpu.rs

### WS4 Deliverables
- [ ] Updated Cargo.toml
- [ ] Updated CI workflows
- [ ] Feature flags validated

### WS5 Deliverables
- [ ] Updated CLAUDE.md
- [ ] Updated README.md
- [ ] docs/QUICKSTART_METAL.md
- [ ] Updated contract docs

### Integration Deliverables
- [ ] All code merged
- [ ] All tests passing
- [ ] CI green
- [ ] Documentation complete
- [ ] Committed to feature branch

---

## Status: READY TO LAUNCH

All master prompts are prepared. Coordinator is ready to launch agents in parallel.

**Next Action**: Execute agent launches with master prompts above.
