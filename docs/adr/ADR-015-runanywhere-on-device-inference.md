# ADR-015: RunAnywhere SDK for On-Device Inference and Relay Protocol

**Status**: Proposed

**Date**: 2026-01-27

**Authors**: caro maintainers

**Target**: Community

## Context

Caro currently runs on desktop/laptop systems (macOS, Linux) where shell commands are generated and executed locally. As we plan a Caro iOS app, we need:

1. **On-device inference on mobile** — Generate shell commands directly on iPhone/iPad without cloud connectivity, preserving Caro's privacy-first philosophy.
2. **Relay protocol to personal computer** — Commands generated on mobile need a secure channel to be sent to the user's desktop/laptop for execution, similar to how [Blink Shell's Happy](https://blink.sh) and [Atuin](https://atuin.sh) sync terminal context across devices.
3. **Pre-generation pipeline** — Use the mobile device's idle compute for quick on-device pre-generation before relaying to a more powerful personal computer for refinement or execution.

[RunAnywhere](https://github.com/RunanywhereAI/runanywhere-sdks) provides on-device AI SDKs for iOS (Swift, stable), Android (Kotlin, stable), React Native (beta), and Flutter (beta). It supports GGUF-format LLMs (SmolLM2 360M, Qwen 2.5 0.5B, Llama 3.2 1B, Mistral 7B Q4) with streaming generation, structured JSON output, and speech processing — all running locally with no cloud dependency.

This aligns directly with Caro's existing model catalog (SmolLM, Qwen, Llama, Mistral) and privacy-first design.

## Decision

Integrate the RunAnywhere Swift SDK into a future Caro iOS app as the on-device inference engine, and design a relay protocol for sending generated commands to the user's personal computer.

This is a **research and preparation** decision — we are laying groundwork, not actively developing the iOS app yet.

### Scope of Research

1. **Evaluate RunAnywhere Swift SDK** — Test model loading, inference speed, memory usage, and structured JSON output on iOS 17+ devices.
2. **Design relay protocol** — Specify a secure protocol for mobile-to-desktop command relay, studying prior art from Happy (Blink Shell) and Atuin's sync architecture.
3. **Define pre-generation pipeline** — Architect a flow where mobile generates candidate commands quickly on-device, then optionally relays to desktop for refinement via a more capable model.
4. **Prototype model compatibility** — Validate that Caro's existing GGUF model catalog works with RunAnywhere's inference runtime.

## Rationale

- **Privacy preservation**: On-device inference means no command data leaves the device, consistent with Caro's core value.
- **Model compatibility**: RunAnywhere supports the same GGUF model format and model families (SmolLM, Qwen, Llama, Mistral) that Caro already uses.
- **Platform maturity**: The Swift SDK is stable, targets iOS 17+/macOS 14+, and provides streaming + structured JSON output — both required by Caro's inference backend trait.
- **Relay precedent**: Happy and Atuin demonstrate that terminal-context relay between mobile and desktop is viable and valued by developers.
- **Pre-generation value**: Smaller models (360M–1B) on mobile can produce fast initial suggestions while the user is still context-switching to their computer.

## Consequences

### Benefits

- Enables a Caro iOS companion app with full offline command generation
- Reuses existing model catalog — no need for separate mobile model pipeline
- Secure relay protocol enables "think on phone, execute on computer" workflow
- Pre-generation reduces perceived latency when arriving at the desktop
- Structured JSON output from RunAnywhere maps directly to Caro's command response format

### Trade-offs

- iOS 17+ minimum limits older device support (acceptable for AI workloads)
- Mobile models (360M–1B) produce lower quality output than desktop models (3B–7B) — mitigated by relay refinement
- Relay protocol adds complexity to the system architecture
- RunAnywhere is a third-party dependency — need to monitor project health and licensing

### Risks

- RunAnywhere SDK maturity: Project is relatively new → Mitigation: Research phase allows evaluation before commitment; contribute upstream if needed
- Memory constraints on older iPhones: 2GB minimum for smallest models → Mitigation: Target iPhone 12+ (4GB RAM) as minimum
- Relay security: Command relay must be end-to-end encrypted → Mitigation: Design protocol with E2EE from the start (study Atuin's approach)
- Model quality on mobile: Small models may produce unsafe commands → Mitigation: Caro's safety validator runs on both mobile and desktop sides

## Alternatives Considered

### Alternative 1: MLX Swift (Apple)
- Description: Apple's MLX framework has Swift bindings for on-device ML
- Pros: First-party Apple support, optimized for Apple Neural Engine
- Cons: Tied to Apple ecosystem only, less model format flexibility, more complex setup for GGUF models

### Alternative 2: llama.cpp direct integration
- Description: Compile llama.cpp for iOS and call via C FFI
- Pros: Maximum control, wide model support, battle-tested
- Cons: Complex C/Swift interop, no structured output built-in, must build streaming/progress infrastructure ourselves

### Alternative 3: Cloud relay only (no on-device inference)
- Description: Mobile app sends natural language to desktop, desktop generates command
- Pros: Simpler architecture, full model quality
- Cons: Requires network connectivity, adds latency, breaks privacy-first principle, no offline capability

## Implementation Notes

### Phase 1: Research (Current)
- Evaluate RunAnywhere SDK capabilities and limitations
- Benchmark inference speed/memory on target iOS devices
- Study Happy and Atuin relay protocol designs
- Document findings in a research report

### Phase 2: Protocol Design
- Define relay protocol specification (authentication, encryption, command format)
- Design pre-generation pipeline architecture
- Create protocol ADR based on research findings

### Phase 3: Prototype
- Build minimal iOS app with RunAnywhere inference
- Implement relay protocol proof-of-concept
- Validate end-to-end flow: natural language → mobile inference → relay → desktop execution

### Integration Points
- `InferenceBackend` trait: RunAnywhere would implement a new `RunAnywhereBackend` conforming to Caro's existing trait
- Safety validation: Reuse `SafetyValidator` on both iOS (via Rust cross-compilation or Swift port) and desktop
- Model catalog: Share GGUF model definitions between mobile and desktop

### Testing Approach
- Device matrix testing (iPhone 12–16, iPad Pro)
- Memory pressure testing under constrained conditions
- Relay protocol latency and reliability testing
- Safety validation parity testing (mobile vs desktop)

## Success Metrics

- Research report completed with clear go/no-go recommendation
- RunAnywhere SDK evaluated on 3+ iOS device tiers with benchmark data
- Relay protocol specification drafted with security review
- Pre-generation pipeline architecture documented
- Model compatibility validated for at least 3 GGUF models from Caro's catalog

## References

- [RunAnywhere SDKs](https://github.com/RunanywhereAI/runanywhere-sdks) — On-device AI SDK
- [Blink Shell / Happy](https://blink.sh) — Mobile terminal with desktop relay
- [Atuin](https://atuin.sh) — Shell history sync across devices
- [ADR-006](./ADR-006-olmo3-model-support.md) — OLMo 3 Model Support (related model catalog work)
- [ADR-013](./ADR-013-pre-processing-pipeline.md) — Pre-Processing Pipeline Architecture (related pipeline design)

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-27 | caro maintainers | Initial draft — research phase |
