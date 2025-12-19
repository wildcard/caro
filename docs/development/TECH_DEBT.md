# Technical Debt & Known Issues

This document tracks technical debt, known limitations, and areas where we need community help. Issues are categorized by complexity and impact.

> 💡 **Want to contribute?** Issues marked with 🟢 are great for first-time contributors!

---

## 🔴 High Priority

### Contract Test API Alignment
**Issue**: #4
**Status**: Open
**Complexity**: Medium
**Impact**: High - Blocks full test coverage

**Description**:
Config and logging contract tests were written before implementation and expect different API signatures. This causes ~35 compilation errors in the test suite.

**Examples**:
```rust
// Tests expect:
LogOutput::File { path: log_file }

// Implementation provides:
LogOutput::File(PathBuf)
```

**Tasks**:
- [ ] Review spec.md to determine canonical API
- [ ] Decide: Update tests OR update implementation
- [ ] Fix config contract tests (8 errors)
- [ ] Fix logging contract tests (27 errors)
- [ ] Verify all contract tests compile and pass

**Skills needed**: Rust, API design, testing
**Estimated effort**: 4-8 hours
**Help wanted**: Yes - We need someone to review the specs and make the alignment decision

---

### Hugging Face Model Download Implementation
**Issue**: N/A (needs creation)
**Status**: Placeholder
**Complexity**: High
**Impact**: High - Core feature missing

**Description**:
The cache module has a placeholder `download_model()` implementation that always returns an error. This prevents downloading models from Hugging Face Hub.

**Current State**:
```rust
async fn download_model(&self, model_id: &str) -> Result<PathBuf, CacheError> {
    // Placeholder: In real implementation, this would:
    // 1. Fetch model from Hugging Face Hub
    // 2. Show progress bar
    // 3. Validate checksums
    // 4. Update manifest
    Err(CacheError::DownloadFailed("Download not implemented yet".to_string()))
}
```

**Requirements**:
- [ ] HTTP client for HF Hub API
- [ ] Progress bar using indicatif
- [ ] Resume capability for interrupted downloads
- [ ] Checksum validation during download
- [ ] Manifest updates with proper locking
- [ ] Integration tests with mock HTTP server

**Skills needed**: Rust, async I/O, HTTP APIs, progress indicators
**Estimated effort**: 16-24 hours
**Help wanted**: Yes - This is Feature 004, looking for an owner

---

## 🟡 Medium Priority

### Security Hardening: File Permissions
**Issue**: #6
**Status**: Open
**Complexity**: Low-Medium
**Impact**: Medium - Security best practices

**Description**:
Cache directories and manifest files should have restricted permissions to prevent unauthorized access.

**Tasks**:
- [ ] Add cache directory permission enforcement (0700)
- [ ] Add manifest file permission enforcement (0600)
- [ ] Add verification checks in tests
- [ ] Document TOCTOU limitation in rustdoc
- [ ] Consider adding `--verify-permissions` CLI flag

**Platform Considerations**:
- Unix/Linux/macOS: Use `std::os::unix::fs::PermissionsExt`
- Windows: Different approach needed (ACLs)

**Skills needed**: Rust, Unix permissions, security
**Estimated effort**: 4-6 hours
**Help wanted**: Yes - Good for someone familiar with Unix permissions

---

### 🟢 Better Error Messages in Config Validation
**Issue**: N/A (needs creation)
**Status**: Tech debt
**Complexity**: Low
**Impact**: Low - Developer experience

**Description**:
Configuration validation errors could be more helpful by suggesting valid values and showing context.

**Current**:
```rust
Err("Invalid safety level 'foo'")
```

**Desired**:
```rust
Err(ConfigError::ValidationError(
    "Invalid safety level 'foo'. Valid values: strict, moderate, permissive.\n\
     Found in config file at line 5."
))
```

**Tasks**:
- [ ] Add line number tracking to TOML parser
- [ ] Include valid values in all validation errors
- [ ] Add "did you mean?" suggestions for typos
- [ ] Update error message tests

**Skills needed**: Rust, error handling, user experience
**Estimated effort**: 3-4 hours
**Help wanted**: Yes - Great first contribution!

---

## 🟢 Good First Issues

### 🟢 Add Property-Based Tests for LRU Eviction
**Issue**: N/A (needs creation)
**Status**: Enhancement
**Complexity**: Low-Medium
**Impact**: Low - Improved test coverage

**Description**:
The LRU cache eviction algorithm should have property-based tests to verify it correctly evicts the least-recently-used models under various scenarios.

**Approach**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn lru_evicts_oldest_model(
        models in prop::collection::vec(any::<String>(), 1..100),
        max_size in 1..10usize
    ) {
        // Property: After filling cache beyond max_size,
        // the least recently accessed model should be evicted
    }
}
```

**Tasks**:
- [ ] Add proptest dependency
- [ ] Create property tests for LRU eviction
- [ ] Test with various cache sizes and model counts
- [ ] Verify chronological eviction order

**Skills needed**: Rust, testing, property-based testing
**Estimated effort**: 2-3 hours
**Help wanted**: Yes - Perfect for learning property-based testing!

---

### 🟢 Add Benchmark Suite
**Issue**: N/A (needs creation)
**Status**: Enhancement
**Complexity**: Low
**Impact**: Low - Performance validation

**Description**:
Add Criterion benchmarks to validate performance requirements at scale.

**Benchmarks Needed**:
- Cache operations (get, add, remove, evict)
- Config loading with various file sizes
- Context capture with large environment
- Logging throughput

**Example**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_context_capture(c: &mut Criterion) {
    c.bench_function("context_capture", |b| {
        b.iter(|| ExecutionContext::capture())
    });
}

criterion_group!(benches, benchmark_context_capture);
criterion_main!(benches);
```

**Tasks**:
- [ ] Add criterion dependency
- [ ] Create benchmark suite for each module
- [ ] Set up CI to track performance regressions
- [ ] Document performance baseline

**Skills needed**: Rust, benchmarking, performance
**Estimated effort**: 3-5 hours
**Help wanted**: Yes - Good for performance enthusiasts!

---

### 🟢 Improve Documentation Examples
**Issue**: N/A (needs creation)
**Status**: Enhancement
**Complexity**: Low
**Impact**: Low - Developer experience

**Description**:
Many public APIs lack rustdoc examples. Adding examples improves discoverability and usability.

**Priority APIs**:
```rust
// src/cache/mod.rs
impl CacheManager {
    /// Get a cached model by ID
    ///
    /// # Example
    /// ```no_run
    /// # use cmdai::cache::CacheManager;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = CacheManager::new()?;
    /// let model_path = cache.get_model("facebook/opt-125m").await?;
    /// println!("Model cached at: {:?}", model_path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_model(&self, model_id: &str) -> Result<PathBuf, CacheError>
}
```

**Tasks**:
- [ ] Add examples to CacheManager methods
- [ ] Add examples to ConfigManager methods
- [ ] Add examples to ExecutionContext methods
- [ ] Add examples to Logger methods
- [ ] Verify examples compile with `cargo test --doc`

**Skills needed**: Rust, documentation
**Estimated effort**: 2-4 hours
**Help wanted**: Yes - Perfect for beginners!

---

## 🔵 Low Priority / Nice to Have

### Config Hot Reloading
**Issue**: N/A (needs creation)
**Status**: Future enhancement
**Complexity**: Medium
**Impact**: Low - Quality of life

**Description**:
Support watching the config file for changes and reloading without restarting.

**Approach**:
- Use `notify` crate for file watching
- Add `ConfigManager::watch()` method
- Emit events on config changes
- Validate before applying

**Skills needed**: Rust, async I/O, file watching
**Estimated effort**: 6-8 hours
**Help wanted**: Maybe - Low priority

---

### Cache Compression Support
**Issue**: N/A (needs creation)
**Status**: Future enhancement
**Complexity**: Medium
**Impact**: Low - Disk space savings

**Description**:
Support compressed cache storage to save disk space.

**Options**:
- gzip compression for individual models
- zstd for better compression ratio
- Transparent decompression on access

**Trade-offs**:
- Pro: Significant disk space savings (50-70%)
- Con: CPU overhead for compression/decompression
- Con: Increased complexity

**Skills needed**: Rust, compression algorithms
**Estimated effort**: 8-12 hours
**Help wanted**: Maybe - Depends on user demand

---

### JSON Schema for Configuration
**Issue**: N/A (needs creation)
**Status**: Future enhancement
**Complexity**: Low
**Impact**: Low - IDE support

**Description**:
Generate JSON schema for the TOML configuration to enable autocomplete and validation in editors.

**Benefits**:
- Editor autocomplete for config values
- Real-time validation in IDEs
- Better documentation

**Approach**:
```rust
use schemars::{schema_for, JsonSchema};

#[derive(JsonSchema)]
struct UserConfiguration { ... }

fn generate_schema() {
    let schema = schema_for!(UserConfiguration);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

**Skills needed**: Rust, JSON Schema
**Estimated effort**: 2-3 hours
**Help wanted**: Yes - Good starter task!

---

## 📋 Process for Claiming Issues

1. **Comment on the issue** saying you'd like to work on it
2. **Wait for assignment** - Maintainers will assign the issue to you
3. **Ask questions** - Use the issue for clarifications
4. **Submit PR** - Reference the issue number in your PR
5. **Respond to review** - Address feedback promptly

## 🎯 Contribution Guidelines

- All contributions must include tests
- Follow the TDD workflow documented in `TDD-WORKFLOW.md`
- Update `CHANGELOG.md` with your changes
- Add rustdoc examples for new public APIs
- Run `cargo clippy -- -D warnings` before submitting
- Run `cargo test` to ensure all tests pass

## 📚 Resources

- [CONTRIBUTING.md](CONTRIBUTING.md) - Full contribution guidelines
- [TDD-WORKFLOW.md](TDD-WORKFLOW.md) - Test-driven development process
- [CLAUDE.md](CLAUDE.md) - Project structure and architecture
- [Rust Book](https://doc.rust-lang.org/book/) - For Rust beginners

## 💬 Getting Help

- **Questions about issues**: Comment on the GitHub issue
- **General questions**: Open a Discussion
- **Bugs**: File a bug report
- **Chat**: Join our community (link TBD)

---

**Last Updated**: 2025-10-03
**Total Open Tech Debt Items**: 10
**Good First Issues**: 4
