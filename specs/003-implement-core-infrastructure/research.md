# Research: Core Infrastructure Modules

**Feature**: 003-implement-core-infrastructure
**Date**: 2025-10-02
**Status**: Complete

## Research Questions & Decisions

### 1. Cache Directory Management

**Decision**: Use `directories` crate with XDG Base Directory specification

**Rationale**:
- Industry-standard for cross-platform directory management in Rust
- Automatic platform-specific paths: `~/.cache/caro` (Linux), `~/Library/Caches/caro` (macOS), `%LOCALAPPDATA%\caro\cache` (Windows)
- Zero-cost abstraction over platform APIs
- Well-maintained crate with 15M+ downloads

**Alternatives Considered**:
- Manual path construction: Rejected due to platform differences and edge cases
- `dirs` crate: Rejected in favor of `directories` which has better XDG compliance
- Environment variables only: Insufficient for handling all platform conventions

**Implementation Notes**:
- Use `BaseDirs::new()` for cache directory resolution
- Fallback to temp directory if XDG directories unavailable
- Create directories with mode 0700 (user-only access) for security

---

### 2. Configuration File Format

**Decision**: TOML with `toml` crate for parsing

**Rationale**:
- Human-readable and editable (NFR-007 requirement)
- Native Rust support via mature `toml` crate
- Strong typing with serde integration for validation
- Better error messages than JSON for user-edited files
- Widely adopted in Rust ecosystem (Cargo.toml precedent)

**Alternatives Considered**:
- JSON: Rejected due to poor human-editability (no comments, strict syntax)
- YAML: Rejected due to complexity and parsing ambiguities
- RON (Rusty Object Notation): Rejected as less familiar to users

**Implementation Notes**:
- Configuration file location: `~/.config/caro/config.toml` (XDG-compliant)
- Schema validation using serde with custom deserializers for enums
- Graceful handling of unknown keys (forward compatibility)

---

### 3. Async Runtime for Non-Blocking Operations

**Decision**: Use existing `tokio` runtime with spawn_blocking for file I/O

**Rationale**:
- Already a project dependency for backends module
- Supports async file operations via `tokio::fs`
- `spawn_blocking` for CPU-bound operations (checksum validation)
- Non-blocking logging requirement (NFR-004) satisfied with async channels

**Alternatives Considered**:
- `async-std`: Rejected to avoid dual runtime dependency
- Synchronous I/O: Rejected due to NFR-004 (non-blocking logging requirement)
- Thread pools only: Rejected in favor of structured async/await

**Implementation Notes**:
- Use `tokio::fs` for file operations in cache and config modules
- Logging uses `tracing-appender` with async non-blocking writer
- Performance: File I/O offloaded to thread pool via `spawn_blocking`

---

### 4. Structured Logging Framework

**Decision**: `tracing` + `tracing-subscriber` with JSON formatting

**Rationale**:
- Structured logging with spans and events (better than log crate)
- Built-in async support for non-blocking writes
- JSON formatter for machine-readable logs (observability requirement)
- Context propagation across async boundaries
- Wide adoption in modern Rust ecosystem

**Alternatives Considered**:
- `log` crate: Rejected due to lack of structured context and spans
- `slog`: Rejected in favor of more modern tracing ecosystem
- `env_logger`: Rejected as insufficient for structured observability

**Implementation Notes**:
- Use `tracing::instrument` for automatic span creation
- JSON formatter for production, pretty formatter for development
- Log rotation via `tracing-appender::rolling`
- Sensitive data redaction via custom layer (API keys, passwords)

---

### 5. Hugging Face Model Download

**Decision**: Use `reqwest` with streaming downloads and checksum validation

**Rationale**:
- Already a project dependency for remote backends
- Streaming support for large model files (avoid memory issues)
- Async/await compatible with tokio runtime
- Progress reporting capability for user feedback

**Alternatives Considered**:
- `ureq` (sync): Rejected due to blocking I/O
- `curl` command: Rejected in favor of native Rust solution
- `hf_hub` crate: Evaluated but provides more than needed; using reqwest directly

**Implementation Notes**:
- Stream downloads directly to cache directory (no temp files)
- SHA256 checksum validation using `sha2` crate
- Partial download resume using HTTP Range headers
- Progress callback for CLI progress bars

---

### 6. Shell Detection Strategy

**Decision**: Environment variable parsing with fallback detection

**Rationale**:
- Primary: `SHELL` environment variable (most reliable)
- Fallback 1: Process inspection via `/proc/self/stat` (Linux)
- Fallback 2: Parent process name parsing (cross-platform)
- Fallback 3: Default to `sh` (POSIX baseline)

**Alternatives Considered**:
- `SHELL` only: Rejected due to missing/unreliable in some environments
- Process tree parsing only: Rejected as platform-specific
- User configuration only: Rejected as insufficient for auto-detection

**Implementation Notes**:
- Use `std::env::var("SHELL")` as primary method
- PowerShell detection via `PSModulePath` environment variable
- Fish shell detection via `FISH_VERSION`
- Cache detection result in ExecutionContext to avoid repeated checks

---

### 7. Configuration Defaults and Validation

**Decision**: Serde with custom validators and builder pattern

**Rationale**:
- Type-safe deserialization with automatic validation
- Custom validators for enum variants (SafetyLevel, ShellType)
- Builder pattern for programmatic config construction
- Clear error messages via `serde::de::Error`

**Alternatives Considered**:
- Manual parsing: Rejected due to boilerplate and error-prone
- External validation library: Rejected in favor of serde built-ins
- Runtime validation only: Rejected in favor of parse-time validation

**Implementation Notes**:
```rust
#[derive(Deserialize)]
struct UserConfig {
    #[serde(default = "default_safety_level")]
    safety_level: SafetyLevel,

    #[serde(default = "default_shell")]
    shell: Option<ShellType>,

    #[serde(default)]
    model: Option<String>,
}
```

---

### 8. Error Handling Strategy

**Decision**: `thiserror` for library errors, `anyhow` for binary context

**Rationale**:
- Constitution requirement V: Error context chains
- Libraries export typed errors (better API contract)
- Binary adds context via `anyhow::Context`
- Clear distinction between recoverable and fatal errors

**Alternatives Considered**:
- `anyhow` everywhere: Rejected due to type erasure in library APIs
- Manual error types: Rejected in favor of thiserror's derive macros
- Unwrap/expect: Rejected due to poor error experience

**Implementation Notes**:
```rust
// Library (src/cache/mod.rs)
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
}

// Binary (src/main.rs)
cache_manager.get_model(&model_id)
    .context("Failed to load model from cache")?;
```

---

## Technology Stack Summary

| Component | Technology | Version | Justification |
|-----------|-----------|---------|---------------|
| Directory Management | `directories` | 5.x | XDG-compliant, cross-platform |
| Config Parsing | `toml` | 0.8.x | Human-readable, serde integration |
| Async Runtime | `tokio` | 1.x | Existing dependency, non-blocking I/O |
| Logging Framework | `tracing` + `tracing-subscriber` | 0.1.x | Structured, async-compatible |
| HTTP Client | `reqwest` | 0.11.x | Existing dependency, streaming |
| Checksums | `sha2` | 0.10.x | SHA256 validation for cache integrity |
| Error Handling | `thiserror` + `anyhow` | Latest | Typed library errors, contextual binary errors |

---

## Performance Considerations

### Cache Module
- Streaming downloads: Avoid loading entire model into memory
- Checksum validation: Use `spawn_blocking` for CPU-intensive hashing
- Target: <5s for models under 1GB (NFR-001)

### Config Module
- File size expected: <10KB (parse in <10ms)
- Cached in memory after first load
- Target: <100ms total load time (NFR-002)

### Execution Module
- Environment variable access: O(1) system calls
- Shell detection: Cached after first check
- Target: <50ms context capture (NFR-003)

### Logging Module
- Async writes via `tracing-appender::non_blocking`
- Ring buffer for in-memory log queue
- Target: Zero blocking on main execution flow (NFR-004)

---

## Security Considerations

### Cache Integrity
- SHA256 checksum validation before loading cached models
- Directory permissions: 0700 (user-only access)
- Atomic writes: Download to temp, then rename

### Configuration Security
- Never log sensitive configuration values
- Validate all inputs before persistence
- Clear error messages without exposing internals

### Logging Redaction
- Redact patterns: API keys, tokens, passwords, credentials
- Use regex-based scanner on log messages
- Redaction happens before serialization

---

## Dependencies to Add

```toml
[dependencies]
# Existing (already in Cargo.toml)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["stream"] }

# New for infrastructure modules
directories = "5.0"
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-appender = "0.2"
sha2 = "0.10"
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
# Existing
tokio-test = "0.4"
tempfile = "3.8"

# New for infrastructure testing
proptest = "1.4"  # Already added in Feature 002
```

---

## Open Questions (Resolved)

1. **Q: How to handle concurrent config writes?**
   **A**: Use file locking via `fs2` crate for atomic writes. Config writes are rare (user-initiated only).

2. **Q: Log rotation strategy?**
   **A**: Use `tracing-appender::rolling::daily` for daily rotation, keep last 7 days. Configurable via user config.

3. **Q: Cache size limits?**
   **A**: Implement configurable cache size limit (default 10GB). LRU eviction when limit reached.

4. **Q: Offline detection for cache fallback?**
   **A**: Attempt network request, catch connection errors, fall back to cache. No proactive network checking.

---

**Research Status**: ✅ **COMPLETE** - All technology decisions made, no remaining unknowns
