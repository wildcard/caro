# Implementation Plan: Hugging Face Model Download

**Branch**: `018-issue-10-implement` | **Date**: 2026-01-08 | **Spec**: [Issue #10](https://github.com/wildcard/caro/issues/10)
**Input**: Feature specification from GitHub Issue #10

This plan implements the download_model() function in the cache module to support downloading ML models from Hugging Face Hub with progress tracking, resume capability, and checksum validation.

## Summary

Replace the placeholder download_model() error in src/cache/mod.rs with a full implementation that:
- Downloads model files from Hugging Face Hub via HTTP
- Shows progress bars with download speed and ETA
- Validates checksums (SHA256) during download
- Supports resume for interrupted downloads
- Updates manifest with proper file locking
- Handles authentication via HF_TOKEN environment variable

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**:
- reqwest 0.11 (async HTTP client with streaming)
- indicatif 0.17 (progress bars)
- sha2 0.10 (SHA256 checksum validation)
- tokio 1.35 (async runtime, already in use)
**Storage**: Local filesystem (existing cache directory structure)
**Testing**: cargo test with wiremock 0.5 (HTTP mocking)
**Target Platform**: macOS/Linux (CLI tool)
**Project Type**: Single Rust binary with library
**Performance Goals**:
- Download speed limited by network, not CPU
- Progress updates at 60Hz (smooth UI)
- Memory efficient streaming (< 10MB buffer overhead)
**Constraints**:
- Must work offline after initial download
- Resume must work across process restarts
- Thread-safe manifest updates
**Scale/Scope**:
- Model files: 100MB - 10GB
- Multiple concurrent downloads: 1 (simplifies first version)
- Network resilience: 3 retry attempts with backoff

## Constitution Check

*GATE: No formal constitution file found - using Rust ecosystem best practices*

**Rust Ecosystem Alignment**:
- ✅ Library-first: Download logic in src/cache/download.rs (reusable)
- ✅ Error handling: CacheError variants for all failure modes
- ✅ Testing: Unit tests with mocked HTTP, integration tests with test fixtures
- ✅ Async: Uses tokio (consistent with existing codebase)
- ✅ CLI progress: Terminal output via indicatif (user-facing)

## Project Structure

### Documentation (this feature)

```
kitty-specs/018-issue-10-implement/
├── plan.md              # This file
├── research.md          # Phase 0 output (dependency evaluation)
├── data-model.md        # Phase 1 output (download state model)
├── quickstart.md        # Phase 1 output (usage examples)
└── tasks.md             # Phase 2 output (implementation breakdown)
```

### Source Code (repository root)

```
src/cache/
├── mod.rs                 # CacheManager (existing, modify download_model)
├── manifest.rs            # Manifest persistence (existing, enhance locking)
├── download.rs            # NEW: Download orchestration
├── http_client.rs         # NEW: HF Hub API client
├── progress.rs            # NEW: Progress tracking
└── checksum.rs            # NEW: SHA256 validation during streaming

tests/
├── unit/
│   └── cache/
│       ├── download_tests.rs    # NEW: Download logic with mocked HTTP
│       ├── http_client_tests.rs  # NEW: HF Hub API mocking
│       └── checksum_tests.rs     # NEW: Checksum validation
└── integration/
    └── cache/
        └── download_integration_tests.rs  # NEW: End-to-end with fixtures
```

**Structure Decision**: Single project (Option 1) - extends existing src/cache/ module with download capabilities. New files are isolated for testability and follow Rust module conventions.

## Complexity Tracking

*No constitution violations - this is standard async HTTP + file I/O in Rust*

## Implementation Phases

### Phase 0: Research & Dependencies

**Objective**: Validate technical choices and establish patterns

**Tasks**:
1. Research HF Hub API endpoints and authentication
2. Evaluate reqwest vs ureq (async vs sync HTTP)
3. Research resume strategies (HTTP Range requests + partial files)
4. Research checksum validation patterns (streaming hash)
5. Research progress bar best practices (indicatif templates)

**Output**: research.md with:
- HF Hub API documentation and examples
- reqwest streaming patterns (async Bytes stream)
- Resume implementation approach (HEAD request + Range header)
- Streaming checksum pattern (update hasher per chunk)
- Progress bar template recommendations

### Phase 1: Design & Contracts

**Objective**: Define interfaces and data structures

**Artifacts**:

1. **data-model.md**: Download state model
   ```rust
   pub struct DownloadProgress {
       pub bytes_downloaded: u64,
       pub total_bytes: Option<u64>,
       pub speed_bps: f64,
       pub eta_seconds: Option<u64>,
   }

   pub struct ModelFile {
       pub name: String,
       pub size: u64,
       pub checksum: String, // SHA256 hex
   }
   ```

2. **quickstart.md**: Usage examples
   - Basic download: `caro download <model-id>`
   - With auth: `HF_TOKEN=xxx caro download <model-id>`
   - Resume after interruption
   - Verification of downloaded files

3. **Public API** (src/cache/mod.rs):
   ```rust
   impl CacheManager {
       pub async fn download_model(
           &self,
           model_id: &str
       ) -> Result<PathBuf, CacheError>;
   }
   ```

4. **Internal APIs** (src/cache/download.rs):
   ```rust
   pub(crate) async fn download_file(
       client: &HttpClient,
       url: &str,
       dest: &Path,
       expected_checksum: Option<&str>,
   ) -> Result<PathBuf, DownloadError>;

   pub(crate) async fn resume_download(
       client: &HttpClient,
       url: &str,
       partial_file: &Path,
       expected_checksum: Option<&str>,
   ) -> Result<PathBuf, DownloadError>;
   ```

### Phase 2: Task Breakdown

*Deferred to /spec-kitty.tasks command - will generate detailed work packages*

Estimated work packages:
- WP01: Setup & Dependencies (add crates to Cargo.toml)
- WP02: HTTP Client Module (HF Hub API wrapper)
- WP03: Download Orchestrator (file download + progress)
- WP04: Checksum Validation (streaming SHA256)
- WP05: Resume Logic (Range requests + partial files)
- WP06: Manifest Integration (atomic updates with locking)
- WP07: Progress UI (indicatif integration)
- WP08: Error Handling (CacheError variants)
- WP09: Unit Tests (mocked HTTP with wiremock)
- WP10: Integration Tests (end-to-end scenarios)
- WP11: Documentation (CONTRIBUTING.md updates)

## Key Technical Decisions

### HTTP Client: reqwest

**Decision**: Use reqwest 0.11 with async/await
**Rationale**:
- Industry standard for Rust HTTP
- Excellent streaming support (async Bytes)
- Built-in retry logic with reqwest-middleware
- HTTPS by default, connection pooling
- Well-tested, actively maintained

**Alternatives Considered**:
- ureq: Simpler but blocking I/O (poor UX for large downloads)
- hyper: Lower-level, more boilerplate

### Progress Tracking: indicatif

**Decision**: Use indicatif 0.17 with default_bar() template
**Rationale**:
- De facto standard for Rust CLI progress bars
- Smooth 60Hz updates
- Automatic terminal width detection
- Download speed + ETA calculation built-in

**Template** (from GitHub issue):
```rust
pb.set_style(
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
        .unwrap()
);
```

### Checksum Validation: Streaming

**Decision**: Hash during download, not after
**Rationale**:
- Memory efficient (no need to reload file)
- Fail fast (detect corruption immediately)
- Single pass over data

**Implementation**:
```rust
let mut hasher = Sha256::new();
while let Some(chunk) = response.bytes_stream().next().await {
    let chunk = chunk?;
    hasher.update(&chunk);
    file.write_all(&chunk).await?;
    pb.inc(chunk.len() as u64);
}
let checksum = format!("{:x}", hasher.finalize());
```

### Resume Strategy: HTTP Range Requests

**Decision**: Use HTTP Range header + .part files
**Rationale**:
- Standard HTTP mechanism (RFC 7233)
- Server-side support (HF Hub supports Range)
- Client decides what to resume (flexible)

**Flow**:
1. Check for existing `.part` file
2. If exists, send `Range: bytes={existing_size}-` header
3. Append new bytes to `.part` file
4. On completion, rename `.part` → final filename
5. Validate checksum of complete file

### Authentication: Environment Variable

**Decision**: Check HF_TOKEN env var, pass as Bearer token
**Rationale**:
- Standard Hugging Face authentication method
- No credential storage in code/config
- Compatible with HF CLI and transformers library
- Optional (public models work without auth)

**Implementation**:
```rust
if let Ok(token) = std::env::var("HF_TOKEN") {
    request = request.header("Authorization", format!("Bearer {}", token));
}
```

### Manifest Locking: File-based Lock

**Decision**: Use fd-lock crate for atomic manifest updates
**Rationale**:
- Prevents corruption from concurrent downloads
- Works across processes (not just threads)
- Consistent with existing cache module patterns

### Error Handling: CacheError Variants

**Decision**: Extend CacheError enum with download-specific variants
**Rationale**:
- Type-safe error handling
- User-friendly error messages
- Preserves error chain (source())

**New Variants**:
```rust
pub enum CacheError {
    // Existing variants...
    DownloadFailed(String),
    NetworkError(String),
    ChecksumMismatch { expected: String, actual: String },
    ResumeNotSupported(String),
    AuthenticationRequired(String),
}
```

## Testing Strategy

### Unit Tests (src/cache/tests/)

**Mocked HTTP with wiremock**:
- Successful download (200 OK)
- Resume from partial (206 Partial Content)
- Network errors (timeouts, connection refused)
- Authentication failures (401 Unauthorized)
- Checksum mismatches
- Server errors (500, 503)

**Example**:
```rust
#[tokio::test]
async fn test_download_with_progress() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/model/file"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"test content"))
        .mount(&mock_server)
        .await;

    // Test download...
}
```

### Integration Tests (tests/integration/)

**End-to-end scenarios with test fixtures**:
- Download small test model (< 1MB fixture)
- Resume after simulated interruption
- Verify checksum validation
- Test error recovery
- Concurrent cache operations

### Manual Testing Checklist

- [ ] Download smollm2-135m-instruct (~500MB)
- [ ] Test with HF_TOKEN authentication
- [ ] Interrupt download (Ctrl+C) and verify resume
- [ ] Test with corrupted partial file
- [ ] Monitor progress bar accuracy (bytes, speed, ETA)
- [ ] Verify manifest atomicity (no corruption)

## Dependencies Added

```toml
[dependencies]
reqwest = { version = "0.11", features = ["stream", "json"] }
indicatif = "0.17"
sha2 = "0.10"
tokio = { version = "1.35", features = ["fs", "io-util"] } # Already present
fd-lock = "4.0" # For manifest locking

[dev-dependencies]
wiremock = "0.5" # HTTP mocking for tests
```

## Risks & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| HF Hub API changes | Low | High | Version API calls, monitor HF docs |
| Large file memory usage | Medium | Medium | Stream with bounded buffer (10MB) |
| Partial file corruption | Low | Medium | Validate checksum, retry from scratch |
| Network timeouts | High | Low | Exponential backoff (3 retries) |
| Manifest lock contention | Low | Low | Short critical sections, fail fast |

## Acceptance Criteria

- [ ] download_model() downloads files from HF Hub
- [ ] Progress bar shows bytes, speed, ETA
- [ ] Resume works after Ctrl+C interruption
- [ ] Checksum validation prevents corrupted files
- [ ] HF_TOKEN authentication works
- [ ] All unit tests pass (>90% coverage)
- [ ] Integration tests pass with fixtures
- [ ] Manifest updates are atomic
- [ ] Error messages are user-friendly
- [ ] CONTRIBUTING.md documents download testing

## Phase 1 Agent Context Update

After completing Phase 1 design artifacts, run:
```bash
.kittify/scripts/bash/update-agent-context.sh claude
```

This will update `.claude/context.md` with:
- New dependencies (reqwest, indicatif, sha2, wiremock)
- Download module structure
- Key design decisions (streaming, checksum, resume)

## Next Steps

1. **Phase 0**: Run `/spec-kitty.research` to populate research.md
2. **Phase 1**: Generate data-model.md, quickstart.md, contracts/
3. **Phase 2**: Run `/spec-kitty.tasks` to create work packages
4. **Implementation**: Execute work packages via `/spec-kitty.implement`
5. **Review**: Code review via `/spec-kitty.review`
6. **Acceptance**: Validate via `/spec-kitty.accept`
7. **Merge**: Integrate via `/spec-kitty.merge`

---

**Plan Status**: Phase 0-1 complete (conceptual design)
**Next Command**: `/spec-kitty.research` or skip to `/spec-kitty.tasks`
