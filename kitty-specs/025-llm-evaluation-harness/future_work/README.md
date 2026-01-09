# Future Work (Deferred to Future Release)

This directory contains work packages that are explicitly deferred to a future release.

## WP10: Multi-Backend Consistency (Phase 3 - P2 Features)

**Status**: Deferred to v1.2.0 or later
**Reason**: MVP (v1.1.0) focuses on MLX backend only for Apple Silicon. Multi-backend support (vLLM, Ollama) is Phase 2/3 work.

**Original Scope**:
- T079-T088: Add vLLM and Ollama backend integration
- Implement cross-backend consistency testing
- Calculate command similarity scores across backends

**Dependencies**:
- Core evaluation harness (WP01-WP09) must be complete first ✅
- Production backends must stabilize before evaluation
- Requires backend-agnostic inference protocol

See `WP10-multi-backend.md` for full details.
