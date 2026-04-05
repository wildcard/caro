# ADR-015: quant.cpp Backend Integration

**Status**: Proposed

**Date**: 2026-04-05

**Authors**: caro maintainers

**Target**: Community

## Context

Caro currently supports multiple inference backends: embedded (llama.cpp/Candle), Ollama, vLLM, Exo, and Claude. All backends provide standard context windows without specialized KV cache optimization.

[quant.cpp](https://github.com/quantumaikr/quant.cpp) is a C-based LLM inference engine (Apache 2.0, v0.5.0, 72K LOC) from QuantumAI that achieves **7x longer context windows** through aggressive KV cache compression with ~0% perplexity degradation. It supports the same GGUF model format caro already uses and provides an OpenAI-compatible HTTP server.

- Users working with complex multi-step commands and agentic loops benefit from extended context
- quant.cpp's KV compression (3.8-8.5x) significantly outperforms llama.cpp's KV quantization (~1.6x)
- The project has 253 commits, a 34-pass test suite, and cross-platform support (Metal, AVX2)
- No official Rust bindings exist, but an OpenAI-compatible HTTP server is available

## Decision

Add quant.cpp as a **remote backend** via its OpenAI-compatible HTTP server API (`/v1/chat/completions`), following the same integration pattern as Ollama, vLLM, and Exo backends.

The backend will:
1. Connect to a user-managed quant.cpp server (default `http://localhost:8080`)
2. Use the OpenAI-compatible chat completions API
3. Expose KV compression mode as a configuration option
4. Support fallback to the embedded backend when unavailable

## Rationale

- **Unique value**: 7x context extension is a genuinely novel capability not available from any other backend. This is especially valuable for caro's agentic context loop and complex multi-step command generation.
- **Low integration cost**: The OpenAI-compatible API means we follow the exact same pattern as 3 existing backends, requiring ~350 lines of new code with no new dependencies.
- **Model compatibility**: GGUF format support means users can run the same Qwen 2.5 Coder models they already use with caro, just with extended context.
- **Risk mitigation**: As a remote backend (not FFI/embedded), there is zero impact on build complexity, binary size, or cross-compilation. If quant.cpp becomes unmaintained, the backend can simply be removed.

## Consequences

### Benefits

- Users gain access to 7x longer context windows for complex command generation
- Same GGUF models work across embedded and quant.cpp backends
- No new dependencies added to caro (uses existing `reqwest`)
- Follows established remote backend pattern for maintainability

### Trade-offs

- Users must install and run quant.cpp server separately
- quant.cpp is pre-1.0 (v0.5.0) — API may change
- Another backend to document and maintain
- KV compression settings add configuration complexity

### Risks

- **API stability**: quant.cpp is v0.5.0, pre-1.0 → Mitigation: OpenAI-compatible API is a stable interface standard; changes unlikely
- **Project longevity**: Smaller community than llama.cpp → Mitigation: Remote backend pattern means easy removal if abandoned
- **User confusion**: Many backend choices → Mitigation: Clear documentation of when to use quant.cpp (extended context needs)

## Alternatives Considered

### Alternative 1: C FFI via rust-bindgen
- Description: Link directly against `libturboquant.a` for embedded inference
- Pros: No server required, native performance, single-binary experience
- Cons: Requires C compiler at build time, cmake dependency, cross-compilation complexity, contradicts single-binary philosophy. Build matrix would grow significantly.

### Alternative 2: Skip integration entirely
- Description: Don't add quant.cpp support
- Pros: No maintenance burden, no additional complexity
- Cons: Miss unique 7x context extension capability; users wanting extended context have no option within caro

### Alternative 3: Wait for Rust bindings
- Description: Wait for official Rust crate from QuantumAI
- Pros: Cleaner integration, potentially embeddable
- Cons: No timeline for Rust bindings; HTTP server mode is available now and sufficient

## Implementation Notes

- New file: `src/backends/remote/quantcpp.rs`
- Add `BackendType::QuantCpp` variant to enum
- Follow `ExoBackend` as primary template (closest pattern match)
- Default port: 8080 (quant.cpp server default)
- Health check: `GET /v1/models`
- KV compression mode: configurable via `KvCompressionMode` enum

### Testing Approach

- Unit tests for JSON parsing (4-stage fallback)
- Unit tests for request/response serialization
- Backend creation and configuration tests
- Integration tests require running quant.cpp server (manual/CI)

## Success Metrics

- **Metric 1**: Backend passes all existing command generation tests when quant.cpp server available
- **Metric 2**: No regression in build time or binary size
- **Metric 3**: Extended context enables successful generation of complex multi-step commands

## References

- [quant.cpp GitHub Repository](https://github.com/quantumaikr/quant.cpp)
- ADR-001: LLM Inference Architecture (embedded backend decision)
- Related tech debt: Duplicated `parse_command_response` across remote backends

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-04-05 | caro maintainers | Initial draft |
