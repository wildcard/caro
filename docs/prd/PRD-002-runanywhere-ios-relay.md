# PRD-002: Caro iOS App with RunAnywhere On-Device Inference and Desktop Relay

**Status**: Research

**Date**: 2026-01-27

**Author**: caro maintainers

**Related ADR**: [ADR-015-runanywhere-on-device-inference](../adr/ADR-015-runanywhere-on-device-inference.md)

---

## Executive Summary

Research and prepare the ground for a Caro iOS companion app that generates shell commands on-device using the [RunAnywhere Swift SDK](https://github.com/RunanywhereAI/runanywhere-sdks), and relays them to the user's personal computer for execution via a secure protocol inspired by Happy (Blink Shell) and Atuin.

This PRD covers the **research and preparation phase only** — no active development yet.

## Problem Statement

### Current State
Caro is a desktop-only CLI tool. Users must be at their computer to generate and execute commands. There is no mobile companion or remote generation capability.

### User Pain Points
1. **Away from computer**: Users think of commands they need to run but aren't at their desk
2. **Context switching delay**: Moving from phone to computer loses the thought/intent
3. **No offline mobile AI**: Existing mobile solutions require cloud connectivity

### Opportunity
Combine on-device inference (RunAnywhere) with a relay protocol to enable:
- Generate commands on iPhone while away from desk
- Queue commands for execution when back at computer
- Pre-generate using small fast models, refine on desktop with larger models
- Maintain Caro's privacy-first principle — everything stays on-device

## Goals & Success Metrics

### Primary Goals (Research Phase)

| Goal | Metric | Target |
|------|--------|--------|
| SDK evaluation | Research report completeness | All key areas evaluated |
| Model compatibility | GGUF models tested | 3+ from Caro catalog |
| Device benchmarks | iOS devices benchmarked | 3+ device tiers |
| Protocol design | Relay spec drafted | Security-reviewed draft |
| Architecture | Pre-generation pipeline designed | Documented with diagrams |

### Future Goals (Development Phase — not yet scoped)

| Goal | Metric | Target |
|------|--------|--------|
| Mobile inference speed | Time to first token | <500ms on iPhone 14+ |
| Relay latency | Command delivery time | <1s on local network |
| Offline capability | Works without connectivity | 100% for generation |
| Safety parity | Mobile safety validation | Match desktop accuracy |

## User Scenarios

### Scenario 1: Quick Command While Away
> Sarah is in a meeting and remembers she needs to restart a service on her server. She opens Caro on her iPhone, types "restart nginx", gets `sudo systemctl restart nginx`, and queues it to her laptop. When she returns to her desk, the command is waiting for confirmation and execution.

### Scenario 2: Pre-Generation Pipeline
> Alex is walking to their desk. On the train, they open Caro iOS and type a complex query. The phone's SmolLM2 360M generates a quick draft command. When Alex reaches their desk, Caro desktop receives the intent and refines it using Mistral 7B for a more accurate result.

### Scenario 3: Offline Field Work
> Jordan is a sysadmin at a data center with no internet. They use Caro iOS to generate diagnostic commands entirely on-device, then execute them by typing on the server's physical terminal.

## Technical Architecture (Research Target)

### Components

```
┌─────────────────────┐         ┌─────────────────────┐
│   Caro iOS App      │         │  Caro Desktop CLI    │
│                     │  Relay  │                      │
│ ┌─────────────────┐ │ Protocol│ ┌──────────────────┐ │
│ │ RunAnywhere SDK │ │◄───────►│ │ Relay Listener   │ │
│ │ (Swift, GGUF)   │ │  E2EE  │ │                  │ │
│ └─────────────────┘ │         │ └──────────────────┘ │
│ ┌─────────────────┐ │         │ ┌──────────────────┐ │
│ │ Safety Validator│ │         │ │ Safety Validator │ │
│ │ (mobile)        │ │         │ │ (full)           │ │
│ └─────────────────┘ │         │ └──────────────────┘ │
│ ┌─────────────────┐ │         │ ┌──────────────────┐ │
│ │ Command Queue   │ │         │ │ Inference Backend│ │
│ │ (pending relay) │ │         │ │ (refinement)     │ │
│ └─────────────────┘ │         │ └──────────────────┘ │
└─────────────────────┘         └─────────────────────┘
```

### Relay Protocol (To Research)
- **Discovery**: Local network (Bonjour/mDNS) + optional WAN relay
- **Authentication**: Device pairing with shared secret / public key exchange
- **Encryption**: End-to-end encryption (NaCl/libsodium or Noise protocol)
- **Transport**: WebSocket or QUIC for low-latency delivery
- **Prior art to study**: Atuin sync protocol, Happy relay, KDE Connect

### Pre-Generation Pipeline
1. User types natural language on iOS
2. RunAnywhere SDK runs small model (360M–1B) locally → fast draft command
3. Draft shown to user immediately on mobile
4. If relay is connected, intent + draft sent to desktop
5. Desktop optionally refines with larger model (3B–7B)
6. User confirms execution on either device

### RunAnywhere SDK Integration Points
- **Model loading**: Download GGUF models on-device, share model catalog with desktop
- **Streaming**: RunAnywhere supports streaming generation — map to Caro's streaming UI
- **Structured output**: JSON output mode maps to Caro's `CommandResponse` format
- **Speech-to-text**: Whisper support enables voice-to-command on mobile (future)

## Research Deliverables

### Phase 1: SDK Evaluation (Q2 2026)
- [ ] Benchmark RunAnywhere inference on iPhone 12, 14, 16
- [ ] Test SmolLM2 360M, Qwen 2.5 0.5B, Llama 3.2 1B with Caro prompts
- [ ] Evaluate memory usage and thermal behavior under sustained inference
- [ ] Test structured JSON output with Caro's command response format
- [ ] Document SDK API surface and integration effort estimate

### Phase 2: Protocol Research (Q2 2026)
- [ ] Study Atuin sync architecture and protocol design
- [ ] Study Happy (Blink Shell) relay mechanism
- [ ] Study KDE Connect protocol for local device communication
- [ ] Draft relay protocol specification
- [ ] Security review of proposed protocol

### Phase 3: Architecture Design (Q3 2026)
- [ ] Design pre-generation pipeline with mobile/desktop split
- [ ] Design safety validation strategy for mobile (Rust cross-compile vs Swift port)
- [ ] Design model sharing/download strategy for mobile
- [ ] Create architecture decision documents for iOS app structure
- [ ] Go/no-go recommendation for development phase

## Dependencies

- RunAnywhere Swift SDK (stable, iOS 17+)
- iOS development environment (Xcode, Swift)
- GGUF models from Caro's existing catalog
- Research into relay protocol prior art

## Out of Scope (This Phase)

- Active iOS app development
- Android app (RunAnywhere Kotlin SDK exists but deprioritized)
- App Store submission
- Desktop relay listener implementation
- Voice-to-command via Whisper (future enhancement)

## References

- [RunAnywhere SDKs](https://github.com/RunanywhereAI/runanywhere-sdks)
- [Blink Shell / Happy](https://blink.sh)
- [Atuin](https://atuin.sh)
- [KDE Connect](https://kdeconnect.kde.org/)
- [ADR-015](../adr/ADR-015-runanywhere-on-device-inference.md) — RunAnywhere On-Device Inference
- [ADR-013](../adr/ADR-013-pre-processing-pipeline.md) — Pre-Processing Pipeline Architecture
- [PRD-001](./PRD-001-olmo3-model-support.md) — OLMo 3 Model Support (model catalog precedent)

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-27 | caro maintainers | Initial research phase draft |
