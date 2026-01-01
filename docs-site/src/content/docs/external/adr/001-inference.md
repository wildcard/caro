---
title: "ADR-001: LLM Inference"
description: "Documentation: ADR-001: LLM Inference"
editUrl: false
---
| **Status**     | Accepted                           |
|----------------|-------------------------------------|
| **Date**       | December 2025                       |
| **Authors**    | Caro Maintainers                    |
| **Supersedes** | N/A                                 |

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [Decision Drivers](#decision-drivers)
4. [Design Philosophy](#design-philosophy)
5. [Architecture Overview](#architecture-overview)
6. [Backend System Design](#backend-system-design)
7. [Crate Selection Rationale](#crate-selection-rationale)
8. [Apple Silicon Strategy](#apple-silicon-strategy)
9. [Hugging Face Integration](#hugging-face-integration)
10. [Security Considerations](#security-considerations)
11. [Future Direction](#future-direction)
12. [Consequences](#consequences)

---

## Executive Summary

This document records the architectural decisions governing LLM inference in Caro, a single-binary CLI tool that converts natural language to safe POSIX shell commands. The architecture embraces a multi-backend approach with first-class support for local inference on Apple Silicon, reflecting our belief that the future of AI inference is distributed—running on the machines closest to the data and the humans who control them.

**Core Tenets:**
- **Local-first inference** with remote fallback
- **Apple Silicon as first-class citizen** (MPS/MLX/Metal)
- **Privacy through local control** of models and data
- **Unified trait system** enabling seamless backend switching
- **Hugging Face ecosystem** for model distribution and caching

---

## Context and Problem Statement

### The Challenge

We needed to design an inference system that could:

1. Convert natural language to shell commands with sub-second response times
2. Run entirely offline on consumer hardware (laptops, home offices)
3. Support multiple inference backends with different performance profiles
4. Maintain a single-binary distribution under 50MB
5. Prioritize user privacy and security

### The Landscape

The LLM inference ecosystem presents several paths:

| Approach | Pros | Cons |
|----------|------|------|
| **Cloud APIs** (OpenAI, Anthropic) | Easy integration, powerful models | Data leaves machine, network dependency, cost |
| **Local CPU inference** (llama.cpp, Candle) | Works everywhere | Slow on non-GPU hardware |
| **Local GPU inference** (MLX, CUDA) | Fast, private | Platform-specific, complex setup |
| **Hybrid** | Best of both worlds | Complex architecture |

We chose **local-first with multi-backend support** as the foundational architecture.

---

## Decision Drivers

### Primary Drivers

1. **Privacy and Security**: The model should only see what you choose to share
2. **Offline Capability**: Must work without network connectivity
3. **Performance**: First inference under 2 seconds on Apple Silicon
4. **Portability**: Single binary, minimal external dependencies
5. **Extensibility**: Easy to add new backends as hardware evolves

### Secondary Drivers

- Developer experience (simple configuration, sensible defaults)
- Binary size constraints (< 50MB without embedded model)
- Memory efficiency on consumer hardware
- Cross-platform support (macOS, Linux, Windows)

---

## Design Philosophy

### The Belief: Distributed Inference is the Future

We hold a strong conviction about where inference is heading:

> **The future of AI is not centralized datacenters—it's distributed inference running everywhere: in home offices, on laptops, inside edge devices, and yes, on the powerful machines already sitting on developers' desks.**

This belief stems from several observations:

#### 1. Hardware Democratization

Apple Silicon has proven that consumer hardware can run serious AI workloads:
- **M1/M2/M3/M4 chips**: Unified memory architecture, powerful Neural Engine
- **Metal Performance Shaders (MPS)**: GPU acceleration without CUDA
- **MLX Framework**: Apple's native ML framework optimized for their silicon

NVIDIA's **DGX Spark** and similar products signal that high-performance inference is moving from datacenters to under-desk machines. We expect to see significant growth in companies deploying inference hardware in home offices and small teams.

#### 2. The Privacy Imperative

There's a harsh limitation to reality that must be acknowledged:

> **A model has access to whatever context you provide. The only way to truly control what a model sees is to control the model and the machine running it.**

This doesn't mean cloud APIs (Anthropic, OpenAI, etc.) are *bad*—they're excellent for many use cases. The question is: **what data are you willing to send through that transport?**

For a CLI tool that sees:
- Your shell commands
- Your file paths and directory structures
- Your natural language descriptions of tasks

...local inference provides the most privacy-respecting default.

#### 3. Network Independence

Remote inference has reached a plateau of convenience but introduces:
- Latency variability
- Availability dependencies
- Rate limits and costs
- Network security considerations

Local inference eliminates these concerns entirely for the price of one-time model download.

### The Strategy: Local-First, Remote-Optional

```
┌─────────────────────────────────────────────────────────┐
│                     User Request                        │
└───────────────────────────┬─────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                  Backend Selection                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │  1. User preference (config/CLI flag)           │   │
│  │  2. Platform detection (Apple Silicon? GPU?)    │   │
│  │  3. Availability check (model cached? API up?)  │   │
│  └─────────────────────────────────────────────────┘   │
└───────────────────────────┬─────────────────────────────┘
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
    ┌───────────┐   ┌───────────┐   ┌───────────────┐
    │    MLX    │   │    CPU    │   │    Remote     │
    │  Backend  │   │  Backend  │   │   Backends    │
    │ (macOS    │   │ (Candle)  │   │ (Ollama/vLLM) │
    │  aarch64) │   │           │   │               │
    └───────────┘   └───────────┘   └───────────────┘
            │               │               │
            └───────────────┼───────────────┘
                            ▼
                    ┌───────────────┐
                    │   Generated   │
                    │    Command    │
                    └───────────────┘
```

---

## Architecture Overview

### Trait-Based Backend System

All backends implement a common `CommandGenerator` trait:

```rust
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    /// Generate a shell command from natural language input
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError>;

    /// Check if this backend is currently available for use
    async fn is_available(&self) -> bool;

    /// Get information about this backend's capabilities and performance
    fn backend_info(&self) -> BackendInfo;

    /// Perform any necessary cleanup when shutting down
    async fn shutdown(&self) -> Result<(), GeneratorError>;
}
```

This design enables:
- **Runtime backend switching** without code changes
- **Graceful fallback** when primary backend unavailable
- **Uniform error handling** across all backends
- **Performance introspection** for backend selection decisions

### Backend Hierarchy

```
src/backends/
├── mod.rs                    # CommandGenerator trait, BackendInfo, GeneratorError
├── embedded/
│   ├── mod.rs               # Embedded backend exports
│   ├── common.rs            # InferenceBackend trait (internal)
│   ├── embedded_backend.rs  # EmbeddedModelBackend orchestrator
│   ├── mlx.rs               # Apple Silicon MLX backend
│   └── cpu.rs               # Cross-platform CPU backend
└── remote/
    ├── mod.rs               # Remote backend exports
    ├── ollama.rs            # Ollama API backend
    └── vllm.rs              # vLLM API backend (OpenAI-compatible)
```

### Feature Gating

```toml
[features]
default = ["embedded-mlx", "embedded-cpu"]
embedded-mlx = ["cxx", "llama_cpp"]
embedded-cpu = ["candle-core", "candle-transformers"]
remote-backends = ["reqwest", "tokio/net"]
full = ["remote-backends", "embedded-mlx", "embedded-cpu"]
```

This allows:
- Minimal builds without remote backends
- Platform-specific optimizations
- Reduced binary size for embedded use cases

---

## Backend System Design

### Embedded Backends

#### MLX Backend (Apple Silicon)

**Decision**: Use `llama_cpp` crate with Metal feature for Apple Silicon inference.

**Rationale**:
1. **Mature ecosystem**: llama.cpp has extensive model support and optimization
2. **Metal integration**: Native GPU acceleration on Apple Silicon
3. **GGUF format**: Efficient quantized model format, widely adopted
4. **Memory efficiency**: Uses mmap for model loading, unified memory aware

**Configuration**:
```rust
// GPU optimization for Apple Silicon
n_gpu_layers: 99,        // All layers on GPU
use_mmap: true,          // Memory-mapped loading
use_mlock: false,        // Don't pin to RAM (let unified memory manage)
context_size: 2048,      // Sufficient for command generation
batch_size: 512,         // Prompt processing batch

// Sampling for command generation
temperature: 0.7,        // Balanced creativity/determinism
top_k: 40,               // Limit candidate tokens
top_p: 0.95,             // Nucleus sampling
repetition_penalty: 1.1, // Avoid repetition
```

**Why not pure mlx-rs?**

We evaluated `mlx-rs` (pure Rust MLX bindings) but encountered significant friction:

| Factor | llama_cpp | mlx-rs |
|--------|-----------|--------|
| **Build deps** | CMake | CMake + Xcode (Metal compiler) |
| **Model format** | GGUF (universal) | MLX format (Apple-specific) |
| **Ecosystem** | Huge model zoo | Smaller selection |
| **Maturity** | Battle-tested | Newer, evolving |
| **Binary size** | ~5MB overhead | ~3MB overhead |

The `mlx-rs` crate requires **full Xcode installation** (15GB) to compile Metal shaders, while `llama_cpp` with Metal works with pre-compiled kernels. For a CLI tool targeting developers who may not have Xcode, this was a significant barrier.

#### CPU Backend (Cross-Platform)

**Decision**: Use Candle for cross-platform CPU inference.

**Rationale**:
1. **Pure Rust**: No C++ dependencies, easier cross-compilation
2. **Hugging Face maintained**: Well-supported, frequent updates
3. **Transformer support**: Native support for Qwen architecture
4. **Fallback role**: Ensures Caro works everywhere, even without GPU

**Current Status**: Stub implementation pending full Candle integration. The stub provides API compatibility while MLX is prioritized for Apple Silicon users.

### Remote Backends

#### Ollama Backend

**Decision**: Support Ollama as a local-remote hybrid option.

**Rationale**:
1. **Zero config**: Works with models already managed by Ollama
2. **Model flexibility**: Users can run any Ollama-supported model
3. **Resource isolation**: Inference runs in Ollama process
4. **Familiar workflow**: Many developers already use Ollama

**Implementation**:
```rust
// Ollama REST API integration
base_url: "http://localhost:11434",
endpoint: "/api/generate",
timeout: 30s,
temperature: 0.1,  // More deterministic for commands
top_k: 10,
top_p: 0.3,
```

#### vLLM Backend

**Decision**: Support vLLM for high-performance server deployments.

**Rationale**:
1. **Production-grade**: Designed for high-throughput inference
2. **OpenAI-compatible**: Standard API, easy integration
3. **Team scenarios**: Shared inference server for organizations
4. **GPU utilization**: Better batching and scheduling

**Implementation**:
```rust
// OpenAI-compatible API
endpoint: "/v1/chat/completions",
authentication: Bearer token (optional),
timeout: 30s,
temperature: 0.1,
max_tokens: 100,
```

---

## Crate Selection Rationale

### Core Dependencies

| Crate | Purpose | Why This Crate? |
|-------|---------|-----------------|
| **llama_cpp** | MLX inference | Metal support, GGUF format, mature, low overhead |
| **candle-core/transformers** | CPU inference | Pure Rust, HF maintained, transformer architectures |
| **hf-hub** | Model download | Official HF crate, async support, caching built-in |
| **tokio** | Async runtime | Industry standard, multi-threaded, excellent ecosystem |
| **reqwest** | HTTP client | Async, rustls (no OpenSSL), feature-gated |
| **serde/serde_json** | Serialization | Universal standard, excellent derive macros |

### Inference-Specific Choices

#### llama_cpp over alternatives

| Alternative | Why Not? |
|-------------|----------|
| **rust-bert** | Heavy deps, ONNX focus, larger binary |
| **ort (ONNX Runtime)** | Requires ONNX conversion, extra step |
| **ctranslate2-rs** | Smaller ecosystem, conversion needed |
| **mlx-rs** | Requires full Xcode for Metal compiler |
| **tract** | Limited model support, inference-only |

**Decision**: `llama_cpp` with Metal feature provides the best balance of:
- GGUF format support (no conversion)
- Metal acceleration (GPU inference)
- Build simplicity (no Xcode requirement)
- Model availability (huge GGUF ecosystem)

#### Candle over alternatives for CPU

| Alternative | Why Not? |
|-------------|----------|
| **llama.cpp (CPU mode)** | Larger binary, C++ deps |
| **tract** | Limited transformer support |
| **ggml-rs** | Less maintained than candle |

**Decision**: Candle provides pure Rust CPU inference with direct HF integration.

### Supporting Crates

| Crate | Purpose | Why This Crate? |
|-------|---------|-----------------|
| **tokenizers** | Text tokenization | HF official, fast, model-compatible |
| **sha2** | Checksum validation | Standard, no deps, WASM-friendly |
| **regex** | Safety patterns | Fast, feature-rich, pre-compilation |
| **once_cell** | Lazy statics | Thread-safe, zero-cost after init |
| **directories** | Cache paths | XDG-compliant, cross-platform |

---

## Apple Silicon Strategy

### Why First-Class Apple Silicon Support?

1. **Market reality**: Significant developer population on Mac
2. **Performance**: M-series chips excel at ML workloads
3. **Unified memory**: Eliminates GPU↔CPU transfer overhead
4. **MLX ecosystem**: Apple is investing heavily in local ML
5. **DX priority**: Caro targets developers, many on Mac

### Metal/MPS/MLX Clarification

These terms are often confused:

| Term | What It Is | Caro Usage |
|------|------------|------------|
| **Metal** | Apple's low-level GPU API (like Vulkan) | Used by llama.cpp for GPU kernels |
| **MPS** | Metal Performance Shaders, optimized GPU ops | Used by llama.cpp's Metal backend |
| **MLX** | Apple's high-level ML framework | Future integration via mlx-rs |
| **Neural Engine** | Dedicated NPU on Apple Silicon | Not currently used (via MLX in future) |

### Current Implementation

```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod mlx {
    // llama_cpp with Metal feature
    // - GPU-accelerated inference
    // - GGUF model format
    // - Unified memory optimized
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
mod mlx {
    // Stub implementation
    // Falls back to CPU backend
}
```

### Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Startup time | < 100ms | ~50ms |
| First inference | < 2s | ~1.8s (M4 Pro) |
| Subsequent inference | < 500ms | ~400ms |
| Memory usage | ~1.2GB | ~1.1GB |

### Future MLX Integration

When `mlx-rs` matures and provides pre-compiled Metal shaders (eliminating Xcode requirement), we plan to:

1. Add native MLX backend alongside llama.cpp
2. Support MLX model format for Apple-optimized models
3. Leverage Neural Engine where beneficial
4. Potentially use MLX as primary, llama.cpp as fallback

---

## Hugging Face Integration

### Model Distribution Strategy

**Decision**: Use Hugging Face Hub as primary model distribution mechanism.

**Rationale**:
1. **Standard infrastructure**: Where the models already live
2. **Versioning**: Model versions tracked automatically
3. **Caching**: Built-in caching with `hf-hub` crate
4. **Offline support**: Works offline once model cached
5. **Community**: Access to thousands of quantized models

### Current Model

```
Repository: Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF
File: qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
Size: ~1.1GB
Quantization: Q4_K_M (4-bit, excellent quality/size)
```

### Cache Architecture

```
~/.cache/caro/
├── models/
│   └── qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
├── manifest.json  # Model metadata, checksums, timestamps
└── config/        # User preferences cache
```

**Features**:
- LRU eviction for multiple models
- SHA256 checksum validation
- Last-accessed timestamp tracking
- Configurable max cache size

### Model Loading Flow

```
1. Check bundled model (if any)
      ↓ not found
2. Check cache directory
      ↓ not found or invalid
3. Download from Hugging Face Hub
      ↓ downloaded
4. Validate checksum
      ↓ valid
5. Load into backend
```

**Lazy Loading**: Models load on first inference, not startup, for fast CLI launch.

---

## Security Considerations

### The Privacy Spectrum

```
┌─────────────────────────────────────────────────────────────┐
│                     DATA EXPOSURE SPECTRUM                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  LOCAL INFERENCE              REMOTE INFERENCE              │
│  (Full Privacy)               (Shared with Provider)        │
│                                                             │
│  ◄─────────────────────────────────────────────────────────►│
│                                                             │
│  • Your prompts stay local    • Prompts sent over network  │
│  • No API keys needed         • API keys required           │
│  • Works offline              • Network required            │
│  • You control the model      • Provider controls model     │
│  • Full auditability          • Trust required              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Our Position

> Using cloud APIs (Anthropic, OpenAI, etc.) is not inherently bad. The question is always: **what data are you willing to share through that transport?**

For Caro, local inference is the default because:

1. **Shell commands reveal intent**: "delete all backup files older than 30 days" reveals infrastructure details
2. **File paths reveal structure**: `/home/user/company-secrets/financial/` is metadata
3. **Context accumulates**: A session of commands builds a picture of your work
4. **Defaults matter**: Most users won't change defaults, so defaults should be private

### Remote Backend Security

When remote backends are used:

```rust
// vLLM: Bearer token authentication
Authorization: Bearer <api_key>

// Ollama: Local by default, but supports remote
// Warning: Ensure Ollama is not exposed publicly

// All remote: TLS via rustls (no OpenSSL)
```

**Security measures**:
- TLS-only connections (HTTP upgraded to HTTPS)
- No credential storage (environment variables preferred)
- Clear error messages for auth failures
- Separate handling for 401/403 (no fallback on auth errors)

### Model Security

**Model validation**:
- SHA256 checksum verification on download
- File size validation (prevents truncated downloads)
- Manifest tracking for integrity
- No automatic model updates (user-initiated only)

---

## Future Direction

### Short-Term (6 months)

1. **Complete Candle CPU backend**: Full cross-platform support
2. **Streaming responses**: Progressive output for longer generations
3. **Model selection**: Allow users to choose from multiple models
4. **Performance profiling**: Identify optimization opportunities

### Medium-Term (12 months)

1. **Native MLX integration**: When `mlx-rs` provides pre-compiled shaders
2. **Neural Engine support**: Via MLX for applicable workloads
3. **Windows GPU support**: DirectML or CUDA backends
4. **Model fine-tuning**: Domain-specific command generation

### Long-Term Vision

1. **Embedded model distribution**: Single binary with model included
2. **Federated learning**: Improve models from anonymized usage patterns
3. **Hardware acceleration detection**: Automatic optimal backend selection
4. **Edge deployment**: ARM/embedded Linux support for IoT

### Hardware Trends We're Tracking

| Hardware | Timeline | Implications |
|----------|----------|--------------|
| **DGX Spark** | Now | Desktop inference stations becoming viable |
| **Apple Silicon** | Ongoing | Unified memory, NPU integration |
| **Intel NPU (Meteor Lake)** | 2024+ | x86 laptops gain inference acceleration |
| **Qualcomm AI PC** | 2024+ | ARM Windows with on-device AI |
| **AMD XDNA (Ryzen AI)** | 2024+ | CPU-integrated NPU |

Our architecture is designed to accommodate these developments through the backend trait system.

---

## Consequences

### Positive

1. **Privacy by default**: Users' data stays on their machine
2. **Works offline**: No network dependency for core functionality
3. **Fast iteration**: Sub-second inference on Apple Silicon
4. **Extensible**: New backends can be added without refactoring
5. **Small footprint**: Single binary under 50MB
6. **Cross-platform**: Works on macOS, Linux, Windows

### Negative

1. **Initial download**: ~1.1GB model download on first use
2. **Disk space**: Model consumes cache space
3. **Memory usage**: ~1.2GB during inference
4. **Build complexity**: Feature flags add conditional compilation
5. **Apple Silicon priority**: CPU backends less optimized

### Risks

1. **llama.cpp churn**: Rapid development may require updates
2. **Model updates**: Newer models may require architecture changes
3. **mlx-rs maturity**: Unknown when Xcode requirement will be lifted
4. **Quantization quality**: Q4 quantization has accuracy limits

### Mitigations

1. **Version pinning**: Lock llama_cpp to tested versions
2. **Trait abstraction**: Backend system isolates model changes
3. **Dual path**: llama.cpp + future mlx-rs as fallbacks for each other
4. **Model selection**: Allow users to choose larger models if needed

---

## Appendix A: Backend Comparison Matrix

| Feature | MLX (llama.cpp) | CPU (Candle) | Ollama | vLLM |
|---------|-----------------|--------------|--------|------|
| **Platform** | macOS aarch64 | All | All | All |
| **Network** | No | No | Local | Yes |
| **Model format** | GGUF | SafeTensors | Varies | Varies |
| **First inference** | ~1.8s | ~4s | ~2s | ~3s |
| **Memory** | ~1.2GB | ~1.5GB | External | External |
| **Offline** | Yes | Yes | Yes* | No |
| **GPU accel** | Metal | No | Depends | Yes |
| **Feature flag** | `embedded-mlx` | `embedded-cpu` | `remote-backends` | `remote-backends` |

*Ollama works offline if model already downloaded

## Appendix B: Model Selection Rationale

**Qwen 2.5 Coder 1.5B** was chosen for:

1. **Size**: 1.5B parameters fits in unified memory comfortably
2. **Coding focus**: Trained specifically for code-related tasks
3. **Instruction tuning**: Follows JSON output format reliably
4. **Quantization**: Q4_K_M provides good quality at 1.1GB
5. **License**: Apache 2.0, permissive for CLI distribution
6. **Availability**: On Hugging Face in GGUF format

## Appendix C: Related Documents

- [MLX Implementation Status](../implementation/MLX_IMPLEMENTATION_STATUS.md)
- [MLX Rust vs Python Investigation](../MLX_RUST_VS_PYTHON.md)
- [macOS Setup Guide](/guides/macos-advanced/)

---

*This ADR was authored in December 2025 and reflects the state of the Caro project at that time. Architectural decisions may evolve as the LLM ecosystem matures.*
