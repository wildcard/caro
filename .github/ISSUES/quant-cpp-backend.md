# Add quant.cpp inference backend for extended context support

**Labels:** enhancement, backend

## Summary

Add [quant.cpp](https://github.com/quantumaikr/quant.cpp) as a new remote inference backend for caro. quant.cpp is a C-based LLM inference engine (Apache 2.0, v0.5.0) whose key differentiator is **7x longer context windows** via lossless KV cache compression with ~0% perplexity degradation.

## Motivation

| Factor | Details |
|--------|---------|
| **Unique value** | 7x context extension via KV cache compression — no other backend offers this |
| **Compatibility** | GGUF format — same models caro already uses (Qwen 2.5 Coder, Llama, etc.) |
| **Integration cost** | Very low — OpenAI-compatible `/v1/chat/completions` API |
| **Cross-platform** | macOS (Metal), Linux (AVX2), Windows, iOS, Android, WASM |

### Comparison to existing backends

| Backend | Context Extension | KV Compression | Server Required |
|---------|------------------|----------------|-----------------|
| Embedded (llama.cpp) | 1x (baseline) | ~1.6x | No |
| Ollama | 1x | Depends on model | Yes |
| vLLM | 1x | PagedAttention | Yes |
| **quant.cpp** | **7x** | **3.8-8.5x** | Yes |

## Integration Approach

Implement as a **remote backend** via HTTP server mode (same pattern as Ollama/vLLM/Exo):
- Default endpoint: `http://localhost:8080/v1/chat/completions`
- OpenAI-compatible request/response format
- KV compression mode configuration (4-bit, delta, qk-norm)
- Embedded fallback support

## Acceptance Criteria

- [ ] `QuantCppBackend` implements `CommandGenerator` trait
- [ ] `BackendType::QuantCpp` variant added
- [ ] Health check via `/v1/models` endpoint
- [ ] KV compression mode configurable
- [ ] Fallback to embedded backend when unavailable
- [ ] Unit tests for request/response serialization and parsing
- [ ] ADR-015 documenting the decision

## References

- [quant.cpp GitHub](https://github.com/quantumaikr/quant.cpp)
- ADR-015: quant.cpp Backend Integration
