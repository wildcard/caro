# Feature Specification: Machine Resource Assessment

**Feature Branch**: `020-machine-resource-assessment`
**Created**: 2026-01-08
**Status**: Draft
**Input**: User description: "Issue #147: Add machine resource assessment and model recommendations"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View System Resource Assessment (Priority: P1)

Users run a diagnostic command to see their system's hardware capabilities (CPU, GPU, memory) to understand what resources are available for running AI models.

**Why this priority**: Core capability that provides immediate value. Users need to know their system specs before making configuration decisions.

**Independent Test**: Can be fully tested by running the assessment command on different systems and verifying accurate hardware detection. Delivers value even without recommendations.

**Acceptance Scenarios**:

1. **Given** caro is installed, **When** user runs `caro assess`, **Then** system displays CPU information (cores, model, architecture)
2. **Given** caro is installed, **When** user runs `caro assess`, **Then** system displays memory information (total RAM, available RAM)
3. **Given** system has GPU, **When** user runs `caro assess`, **Then** system detects and displays GPU information (vendor, model, VRAM)
4. **Given** system has no GPU, **When** user runs `caro assess`, **Then** system reports "No GPU detected" without error
5. **Given** assessment completes, **When** results are displayed, **Then** output is human-readable and clearly formatted

---

### User Story 2 - Get Model Recommendations (Priority: P2)

Based on detected system resources, users receive recommendations for optimal model configurations (model size, backend selection) suited to their hardware.

**Why this priority**: Builds on P1 foundation. Transforms raw data into actionable guidance, helping users avoid misconfiguration.

**Independent Test**: Can be tested by running on various hardware profiles (low-end, mid-range, high-end) and verifying appropriate recommendations for each tier.

**Acceptance Scenarios**:

1. **Given** system has < 8GB RAM, **When** assessment runs, **Then** recommends lightweight models (e.g., Phi-2, TinyLlama)
2. **Given** system has >= 8GB RAM and no GPU, **When** assessment runs, **Then** recommends CPU-optimized models (e.g., Mistral 7B with quantization)
3. **Given** system has Apple Silicon GPU, **When** assessment runs, **Then** recommends MLX backend and appropriate models
4. **Given** system has NVIDIA GPU with sufficient VRAM, **When** assessment runs, **Then** recommends CUDA backend and larger models
5. **Given** recommendations are provided, **When** displayed, **Then** includes reasoning (e.g., "Based on 16GB RAM and M1 GPU...")

---

### User Story 3 - Export Assessment for Troubleshooting (Priority: P3)

Users can export assessment results to share with support or for documentation purposes, aiding in troubleshooting and configuration discussions.

**Why this priority**: Nice-to-have utility feature. Helpful for support scenarios but not essential for core functionality.

**Independent Test**: Can be tested by exporting assessment on various systems and verifying the export format is complete and shareable.

**Acceptance Scenarios**:

1. **Given** assessment completes, **When** user runs `caro assess --export json`, **Then** results are saved to JSON file
2. **Given** assessment completes, **When** user runs `caro assess --export markdown`, **Then** results are saved to formatted markdown file
3. **Given** export file created, **When** file is opened, **Then** contains all detection results and recommendations

---

### Edge Cases

- What happens when hardware detection tools are unavailable or fail? (Gracefully degrade, show partial results)
- How does system handle virtualized environments where hardware info may be limited? (Detect virtualization, warn about potential inaccuracies)
- What if user's system specs fall between recommendation tiers? (Provide multiple options with tradeoffs)
- How does the tool behave on unsupported platforms? (Error gracefully with clear message about supported platforms)
- What if GPU detection requires elevated permissions? (Provide clear instructions for granting access)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect CPU information (architecture, core count, model name)
- **FR-002**: System MUST detect total and available system memory (RAM)
- **FR-003**: System MUST detect GPU presence and gather GPU information (vendor, model, VRAM) when available
- **FR-004**: System MUST provide model recommendations based on detected resources
- **FR-005**: System MUST recommend appropriate backend (MLX for Apple Silicon, CUDA for NVIDIA, CPU-only otherwise)
- **FR-006**: System MUST display assessment results in human-readable format
- **FR-007**: System MUST handle detection failures gracefully without crashing
- **FR-008**: System MUST support export to JSON format
- **FR-009**: System MUST support export to Markdown format
- **FR-010**: Recommendations MUST include reasoning based on detected specs
- **FR-011**: System MUST detect virtualized environments and provide appropriate warnings
- **FR-012**: System MUST work on macOS, Linux, and Windows

### Key Entities

- **System Profile**: Represents the complete hardware assessment (CPU, memory, GPU, platform info)
- **CPU Info**: Architecture, core count, model name, frequency
- **Memory Info**: Total RAM, available RAM, swap status
- **GPU Info**: Vendor, model, VRAM, compute capability (optional)
- **Model Recommendation**: Suggested model name, size, quantization level, backend, reasoning

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Assessment command completes in under 5 seconds on typical hardware
- **SC-002**: CPU and memory detection succeeds on 100% of supported platforms (macOS, Linux, Windows)
- **SC-003**: GPU detection succeeds on systems with dedicated GPUs (90%+ success rate)
- **SC-004**: Recommendations align with hardware capabilities (no 70B model suggestions for 8GB RAM systems)
- **SC-005**: Users can successfully export results in both JSON and Markdown formats
- **SC-006**: Assessment runs without requiring elevated permissions on standard installations
- **SC-007**: Output is clear enough that 90% of users understand their system capabilities without additional documentation

### Qualitative Measures

- Users report the recommendations help them select appropriate models
- Support requests related to model configuration decrease after feature launch
- Assessment output is referenced in troubleshooting documentation

## Assumptions *(mandatory)*

1. **Hardware detection libraries available**: Assume standard system APIs and tools (e.g., `/proc/cpuinfo`, `sysctl`, `wmic`) are accessible for detection
2. **Read-only access sufficient**: Assume assessment doesn't require write permissions or elevated privileges for basic detection
3. **Manual configuration**: Assume users will manually apply recommendations; assessment is informational, not auto-configuring
4. **Static assessment**: Assume assessment is run on-demand, not continuously monitoring system state
5. **English language output**: Assume assessment output is in English (i18n out of scope)
6. **Command-line interface**: Assume assessment is accessed via CLI command, not GUI

## Out of Scope *(mandatory)*

- **Automatic configuration**: Feature does NOT automatically apply recommended settings
- **Runtime monitoring**: Feature does NOT continuously monitor resource usage during inference
- **Benchmark execution**: Feature does NOT run performance benchmarks (shows capabilities only)
- **Network diagnostics**: Feature does NOT assess network connectivity or bandwidth
- **Storage analysis**: Feature does NOT check disk space or storage performance
- **Historical tracking**: Feature does NOT track assessment results over time
- **Comparison mode**: Feature does NOT compare current system to other configurations
- **Remote assessment**: Feature does NOT support assessing remote systems

## Dependencies *(optional)*

- **Platform detection**: Requires platform-specific APIs (macOS `sysctl`, Linux `/proc`, Windows `wmic`)
- **GPU detection libraries**: May require platform-specific GPU detection tools (optional enhancement)
- **Existing caro config**: Should integrate with existing configuration system for showing current vs recommended settings

## Open Questions *(optional)*

None identified. Specification is complete with assumptions documented above.

## Technical Constraints *(optional)*

- Must work within Rust ecosystem (platform detection crates available)
- Must not add heavy dependencies (avoid large ML libraries just for detection)
- Must respect user privacy (no telemetry or data collection)
- Must work offline (no cloud-based detection)

## Risks *(optional)*

- **Hardware detection accuracy**: May struggle with virtualized or unusual hardware configurations
- **GPU detection complexity**: GPU detection varies significantly across platforms and may require elevated permissions
- **Recommendation validity**: Model recommendations may become outdated as new models are released (requires maintenance)
- **User interpretation**: Users may misinterpret recommendations without sufficient context

## References *(optional)*

- Existing Issue: #147
- Related to platform detection work in Issue #274 (command validation pipeline)
- Similar pattern: `caro doctor` command for health checks
