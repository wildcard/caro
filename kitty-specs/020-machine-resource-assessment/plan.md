# Implementation Plan: Machine Resource Assessment

**Branch**: `020-machine-resource-assessment` | **Date**: 2026-01-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/kitty-specs/020-machine-resource-assessment/spec.md`

## Summary

Add a `caro assess` command that detects system resources (CPU, GPU, memory) and provides model recommendations based on hardware capabilities. This helps users understand their system's AI inference capabilities and select appropriate models/backends for optimal performance.

**Technical Approach**: Extend existing Rust CLI with new assessment subcommand using `sysinfo` crate for cross-platform system detection, platform-specific GPU detection, and a recommendation engine that maps hardware profiles to suitable models.

## Technical Context

**Language/Version**: Rust 1.84.0 (existing project)
**Primary Dependencies**:
- `sysinfo` (v0.30+) - Cross-platform system information
- `clap` (existing) - CLI argument parsing
- Platform-specific GPU detection: `metal-rs` (macOS), `nvml-wrapper` (NVIDIA), Windows Management Instrumentation (Windows)
- `serde` + `serde_json` (existing) - JSON export
**Storage**: N/A (read-only system queries, optional file export)
**Testing**: `cargo test` with mock system profiles for unit tests, integration tests on actual hardware
**Target Platform**: macOS, Linux, Windows (cross-platform CLI)
**Project Type**: Single project (existing Rust CLI binary)
**Performance Goals**:
- Assessment completes in < 5 seconds
- Zero allocations during system queries where possible
- Minimal binary size impact (<50KB added to caro binary)
**Constraints**:
- Must not require elevated permissions for basic detection
- Must work offline (no cloud APIs)
- Must handle detection failures gracefully (partial results acceptable)
- Must respect user privacy (no telemetry)
**Scale/Scope**:
- Single CLI subcommand (`caro assess`)
- ~3-5 new Rust modules (~1000-1500 LOC estimated)
- Support for 5-10 hardware profiles (low-end, mid-range, high-end combinations)
- 10-15 model recommendations covering Phi, Llama, Mistral families

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**No formal constitution file found** - Using caro project conventions:

✅ **Rust ecosystem standards**: Feature uses standard crates (`sysinfo`), follows existing code style
✅ **CLI-first design**: Implements as subcommand following existing `caro doctor` pattern
✅ **Cross-platform compatibility**: Uses platform abstraction via `sysinfo`, handles platform-specific GPU detection gracefully
✅ **Privacy-conscious**: No telemetry, no network calls, local-only detection
✅ **Error resilience**: Graceful degradation (partial results on detection failures)
✅ **Testing**: Unit tests with mocked profiles, integration tests on real hardware

**Post-Design Re-check**: Will verify implementation maintains these principles after Phase 1 design.

## Project Structure

### Documentation (this feature)

```
kitty-specs/020-machine-resource-assessment/
├── spec.md              # Feature specification (complete)
├── plan.md              # This file (in progress)
├── research.md          # Phase 0 output (pending)
├── data-model.md        # Phase 1 output (pending)
├── quickstart.md        # Phase 1 output (pending)
└── tasks.md             # Phase 2 output (from /spec-kitty.tasks)
```

### Source Code (repository root)

```
src/
├── assessment/          # NEW: Assessment module
│   ├── mod.rs          # Module exports
│   ├── cpu.rs          # CPU detection logic
│   ├── memory.rs       # Memory detection logic
│   ├── gpu.rs          # GPU detection (platform-specific)
│   ├── profile.rs      # SystemProfile struct
│   └── recommender.rs  # Model recommendation engine
├── cli/
│   ├── commands/
│   │   └── assess.rs   # NEW: 'caro assess' command handler
│   └── mod.rs          # Updated: Register assess command
├── models/             # Existing: May reference for model catalog
└── main.rs             # Updated: Import assessment module

tests/
├── assessment_tests.rs # NEW: Unit tests with mocked system profiles
├── integration/
│   └── assess_command_test.rs # NEW: E2E command tests
└── fixtures/
    └── mock_profiles.rs # NEW: Mock system data for testing
```

**Structure Decision**: Single project structure (Option 1) - This feature extends the existing caro CLI binary with a new assessment module. No separate services or frontend needed.

## Complexity Tracking

*No complexity violations identified - feature follows existing patterns*

## Parallel Work Analysis

*Not applicable - single developer/agent implementation recommended due to tightly coupled modules*

**Rationale**: Assessment components (CPU, GPU, memory detection, recommendation engine) share the SystemProfile struct and are interdependent. Sequential implementation ensures consistent data model and avoids merge conflicts.

## Phase 0: Outline & Research

### Research Tasks

No unresolved NEEDS CLARIFICATION markers in spec - proceeding with identified research areas:

1. **Rust system detection crates comparison**
   - Primary: `sysinfo` (most mature, cross-platform)
   - Alternatives: `sys-info`, `systemstat`
   - Decision criteria: Cross-platform coverage, maintenance status, binary size impact

2. **GPU detection approaches per platform**
   - macOS: Metal framework (`metal-rs` crate or `system_profiler SPDisplaysDataType`)
   - Linux: `/proc/driver/nvidia/version`, `lspci`, or `nvml-wrapper` for NVIDIA
   - Windows: WMI queries (`wmic path win32_VideoController`)
   - Decision: Platform-specific implementations with graceful fallback

3. **Model recommendation algorithm**
   - Input: RAM, GPU (yes/no), VRAM, CPU cores
   - Output: Model family (Phi/Llama/Mistral), size (2B/7B/13B), backend (MLX/CUDA/CPU), quantization level
   - Decision: Rule-based mapping initially, extensible for future ML-based recommendations

4. **Integration with Issue #274 platform detection**
   - PR #54 provides PlatformContext (OS, arch, shell, utilities)
   - Reuse if merged, otherwise duplicate minimal detection
   - Decision: Check PR #54 status, use shared platform detection if available

### Research Output Location

Results will be consolidated in `research.md` with format:
- **Decision**: [Technology/approach chosen]
- **Rationale**: [Why this option]
- **Alternatives considered**: [What else was evaluated]

## Phase 1: Design & Contracts

### Data Model (data-model.md)

**Core Entities**:

1. **SystemProfile**
   - CPU: Architecture, cores, model_name, frequency_mhz (optional)
   - Memory: total_mb, available_mb
   - GPU: Option<GPUInfo>
   - Platform: OS, arch (from std::env::consts)

2. **GPUInfo**
   - vendor: enum (NVIDIA, AMD, Intel, Apple, Unknown)
   - model: String
   - vram_mb: Option<u64>
   - compute_capability: Option<String> (NVIDIA-specific)

3. **ModelRecommendation**
   - model_name: String (e.g., "Phi-2", "Mistral 7B")
   - model_size: String (e.g., "2.7B", "7B")
   - backend: enum (MLX, CUDA, CPU)
   - quantization: Option<String> (e.g., "Q4_K_M", "Q8_0")
   - reasoning: String (why this recommendation)
   - estimated_memory_usage_mb: u64

4. **AssessmentResult**
   - system_profile: SystemProfile
   - recommendations: Vec<ModelRecommendation>
   - warnings: Vec<String> (e.g., "GPU detection failed")
   - timestamp: DateTime<Utc>

### API Contracts (contracts/)

**CLI Interface** (contracts/cli.md):
```bash
# Basic assessment
caro assess

# JSON export
caro assess --export json --output assessment.json

# Markdown export
caro assess --export markdown --output assessment.md

# Show only recommendations
caro assess --recommendations-only
```

**Output Format** (contracts/output-format.md):
```
Human-readable (default):
═══════════════════════════════════════════════════
Caro System Assessment
═══════════════════════════════════════════════════

System Information:
  CPU: Apple M1 (8 cores, arm64)
  Memory: 16384 MB total, 8192 MB available
  GPU: Apple M1 GPU (integrated, ~10GB unified memory)
  Platform: macOS 14.0 (arm64)

Model Recommendations:
  ✓ Phi-2 (2.7B) via MLX backend
    Reasoning: Optimal for Apple Silicon with unified memory
    Memory: ~2GB, Fast inference on Metal GPU

  ✓ Mistral 7B (Q4_K_M) via MLX backend
    Reasoning: Good balance of quality and performance
    Memory: ~4GB quantized, Runs well on 16GB systems

Warnings:
  (none)

═══════════════════════════════════════════════════
```

JSON format (--export json):
```json
{
  "timestamp": "2026-01-08T10:30:00Z",
  "system_profile": {
    "cpu": {"architecture": "arm64", "cores": 8, "model": "Apple M1"},
    "memory": {"total_mb": 16384, "available_mb": 8192},
    "gpu": {"vendor": "Apple", "model": "M1 GPU", "vram_mb": null},
    "platform": {"os": "macos", "version": "14.0"}
  },
  "recommendations": [
    {
      "model_name": "Phi-2",
      "model_size": "2.7B",
      "backend": "MLX",
      "quantization": null,
      "reasoning": "Optimal for Apple Silicon...",
      "estimated_memory_mb": 2048
    }
  ],
  "warnings": []
}
```

### Quickstart Scenarios (quickstart.md)

**Scenario 1: User with Apple Silicon Mac**
```bash
$ caro assess
# Expects: MLX backend recommendations, M1/M2 GPU detected, Phi-2 and Mistral 7B suggested
```

**Scenario 2: User with Linux + NVIDIA GPU**
```bash
$ caro assess
# Expects: CUDA backend recommendations, NVIDIA GPU VRAM detected, larger models if sufficient VRAM
```

**Scenario 3: User with low-end system (4GB RAM, no GPU)**
```bash
$ caro assess
# Expects: CPU-only backend, lightweight models only (TinyLlama, Phi-2), warnings about memory constraints
```

**Scenario 4: Export for support ticket**
```bash
$ caro assess --export json --output my-system.json
# Expects: JSON file created with full assessment, no errors
```

### Integration Points

1. **Platform detection** (if PR #54 merged): Reuse PlatformContext from `src/platform/mod.rs`
2. **Model catalog** (if exists): Reference existing model definitions for recommendations
3. **Config system** (existing): Potentially compare detected capabilities vs configured settings

## Phase 2: Implementation Phases (tasks.md will expand)

*High-level implementation sequence - detailed tasks generated by /spec-kitty.tasks*

### Phase 2.1: Foundation (P1 - Core Detection)
- Implement SystemProfile struct
- Add CPU detection (sysinfo integration)
- Add memory detection (sysinfo integration)
- Basic CLI command scaffolding

### Phase 2.2: GPU Detection (P1 - Platform-Specific)
- macOS GPU detection (Metal or system_profiler)
- Linux GPU detection (NVIDIA focus)
- Windows GPU detection (WMI)
- Graceful fallback for unsupported/missing GPUs

### Phase 2.3: Recommendation Engine (P2 - Intelligence)
- Define hardware profiles (low/mid/high-end tiers)
- Implement recommendation algorithm
- Map profiles to model suggestions
- Generate reasoning text

### Phase 2.4: Output Formatting (P2 - UX)
- Human-readable formatter
- JSON export
- Markdown export
- CLI argument handling

### Phase 2.5: Testing & Polish (P3 - Quality)
- Unit tests with mocked profiles
- Integration tests on real hardware
- Error handling refinement
- Documentation

## Agent Context Update

*To be executed after Phase 1 design completion*

```bash
../../.kittify/scripts/bash/update-agent-context.sh claude
```

This will update `.claude/` context files with new technologies:
- `sysinfo` crate usage patterns
- Platform-specific GPU detection approaches
- Recommendation engine patterns

## Acceptance Criteria Mapping

| Spec Requirement | Implementation Component | Validation Method |
|------------------|-------------------------|-------------------|
| FR-001: CPU detection | `src/assessment/cpu.rs` | Unit test with mock data |
| FR-002: Memory detection | `src/assessment/memory.rs` | Unit test with mock data |
| FR-003: GPU detection | `src/assessment/gpu.rs` | Integration test on real hardware |
| FR-004: Model recommendations | `src/assessment/recommender.rs` | Unit test with fixtures |
| FR-006: Human-readable output | `src/cli/commands/assess.rs` | Integration test output validation |
| FR-008: JSON export | `src/cli/commands/assess.rs` | Integration test file parsing |
| SC-001: < 5 second completion | End-to-end timing | Integration test with timeout |
| SC-002: 100% detection on supported platforms | Cross-platform CI | CI tests on macOS, Linux, Windows |

## Risk Mitigation

1. **GPU detection accuracy risk**
   - Mitigation: Extensive platform testing, graceful fallback, clear warnings
   - Acceptance: Partial results (CPU+memory only) still provide value

2. **Recommendation staleness risk**
   - Mitigation: Separate model catalog data from recommendation logic
   - Future: Support external/updatable model database

3. **Platform-specific code complexity**
   - Mitigation: Isolate platform-specific code in separate modules with clear abstractions
   - Pattern: Trait-based approach with per-platform implementations

## Next Steps

1. ✅ Specification complete (spec.md)
2. ✅ Implementation plan complete (this file)
3. 🔜 **Run /spec-kitty.research** to create research.md (Phase 0)
4. 🔜 **Complete Phase 1 artifacts** (data-model.md, contracts/, quickstart.md)
5. 🔜 **Run /spec-kitty.tasks** to generate detailed work packages (tasks.md)
6. 🔜 **Run /spec-kitty.implement** to begin implementation

---

**Plan Status**: ✅ Complete (ready for Phase 0 research)
**Last Updated**: 2026-01-08
**Next Command**: `/spec-kitty.research` or `spec-kitty research`
