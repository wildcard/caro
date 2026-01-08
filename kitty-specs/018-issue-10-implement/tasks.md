# Tasks: Hugging Face Model Download

**Feature**: Issue #10 - Implement Hugging Face model download
**Branch**: 018-issue-10-implement
**Plan**: [plan.md](./plan.md)
**Spec**: [Issue #10](https://github.com/wildcard/caro/issues/10)

This document breaks down the implementation into executable work packages. Each work package has a corresponding prompt file in `tasks/planned/` that provides detailed implementation guidance.

## Work Package Status

| ID | Title | Subtasks | Status | Prompt |
|----|-------|----------|--------|--------|
| WP01 | Setup & Dependencies | 6 | ⬜ Planned | [WP01-setup-dependencies.md](tasks/planned/WP01-setup-dependencies.md) |
| WP02 | HTTP Client Module | 6 | ⬜ Planned | [WP02-http-client.md](tasks/planned/WP02-http-client.md) |
| WP03 | Download Orchestrator | 7 | ⬜ Planned | [WP03-download-orchestrator.md](tasks/planned/WP03-download-orchestrator.md) |
| WP04 | Checksum Validation | 6 | ⬜ Planned | [WP04-checksum-validation.md](tasks/planned/WP04-checksum-validation.md) |
| WP05 | Resume Logic | 6 | ⬜ Planned | [WP05-resume-logic.md](tasks/planned/WP05-resume-logic.md) |
| WP06 | Manifest Integration | 6 | ⬜ Planned | [WP06-manifest-integration.md](tasks/planned/WP06-manifest-integration.md) |
| WP07 | Progress UI | 7 | ⬜ Planned | [WP07-progress-ui.md](tasks/planned/WP07-progress-ui.md) |
| WP08 | Error Handling | 6 | ⬜ Planned | [WP08-error-handling.md](tasks/planned/WP08-error-handling.md) |
| WP09 | Unit Tests | 8 | ⬜ Planned | [WP09-unit-tests.md](tasks/planned/WP09-unit-tests.md) |
| WP10 | Integration Tests | 7 | ⬜ Planned | [WP10-integration-tests.md](tasks/planned/WP10-integration-tests.md) |
| WP11 | Documentation | 6 | ⬜ Planned | [WP11-documentation.md](tasks/planned/WP11-documentation.md) |

**Total**: 11 work packages, 71 subtasks

## Execution Strategy

### Phase 1: Foundation (WP01-WP02)
**Goal**: Set up dependencies and HTTP client infrastructure
**Duration**: 1-2 days
**Dependencies**: None

- WP01: Setup & Dependencies (prerequisite for all)
- WP02: HTTP Client Module (prerequisite for WP03-WP05)

### Phase 2: Core Download Logic (WP03-WP05)
**Goal**: Implement file downloading with progress and resume
**Duration**: 2-3 days
**Dependencies**: WP01, WP02

- WP03: Download Orchestrator [P]
- WP04: Checksum Validation [P]
- WP05: Resume Logic (depends on WP03)

**Parallel Opportunity**: WP03 and WP04 can be developed in parallel as they operate on different files.

### Phase 3: Integration (WP06-WP08)
**Goal**: Integrate with existing cache system and error handling
**Duration**: 2 days
**Dependencies**: WP03-WP05

- WP06: Manifest Integration (depends on WP03)
- WP07: Progress UI [P]
- WP08: Error Handling [P]

**Parallel Opportunity**: WP07 and WP08 can be developed in parallel.

### Phase 4: Testing & Documentation (WP09-WP11)
**Goal**: Comprehensive testing and user documentation
**Duration**: 2-3 days
**Dependencies**: WP01-WP08

- WP09: Unit Tests [P]
- WP10: Integration Tests [P]
- WP11: Documentation [P]

**Parallel Opportunity**: All three can be developed in parallel once core implementation is complete.

## Work Package Details

### WP01: Setup & Dependencies

**Priority**: Critical (blocks all other work)
**Effort**: 1 hour
**Dependencies**: None

**Objective**: Add required dependencies to Cargo.toml and verify clean build.

**Subtasks**:
- [ ] T001: Add reqwest 0.11 with stream and json features
- [ ] T002: Add indicatif 0.17
- [ ] T003: Add sha2 0.10
- [ ] T004: Add fd-lock 4.0
- [ ] T005: Add wiremock 0.5 to dev-dependencies
- [ ] T006: Verify clean build with `cargo check`

**Success Criteria**:
- `cargo check` passes without errors
- All dependencies resolve correctly
- No version conflicts

**Risks**: None (standard dependency additions)

---

### WP02: HTTP Client Module

**Priority**: High (prerequisite for core download logic)
**Effort**: 4-6 hours
**Dependencies**: WP01

**Objective**: Create HTTP client wrapper for Hugging Face Hub API with authentication support.

**Subtasks**:
- [x] T007: Create src/cache/http_client.rs skeleton with module structure
- [x] T008: Implement HfHubClient struct with reqwest::Client field
- [x] T009: Implement get_file_url() method (format HF Hub URLs)
- [x] T010: Implement download_stream() method (GET with auth header from HF_TOKEN)
- [x] T011: Implement head_request() method (for file size/resume support)
- [x] T012: Add unit tests with wiremock (auth, URL formatting, stream handling)

**Success Criteria**:
- HTTP client constructs correct HF Hub URLs
- Authentication header included when HF_TOKEN is set
- HEAD requests return file metadata
- GET requests return streaming response
- Unit tests pass with >95% coverage

**Risks**:
- HF Hub URL format changes (mitigation: version API calls)
- Auth token format incorrect (mitigation: test against real HF API)

---

### WP03: Download Orchestrator

**Priority**: High (core feature)
**Effort**: 6-8 hours
**Dependencies**: WP01, WP02

**Objective**: Implement main download orchestration logic with streaming file writes.

**Subtasks**:
- [ ] T013: Create src/cache/download.rs skeleton
- [ ] T014: Implement download_file() function signature and basic flow
- [ ] T015: Implement file streaming logic (write chunks to disk)
- [ ] T016: Integrate progress bar updates per chunk
- [ ] T017: Handle temporary .part files during download
- [ ] T018: Implement atomic file rename on completion (.part → final)
- [ ] T019: Add unit tests for download orchestration

**Success Criteria**:
- Files downloaded successfully to cache directory
- .part files used during download, renamed on completion
- Progress updates work smoothly
- Memory usage stays below 10MB (streaming)
- Downloads complete for files up to 10GB

**Risks**:
- Disk space exhaustion (mitigation: check available space before download)
- File permission issues (mitigation: proper error handling)

**Parallel with**: WP04 (different files)

---

### WP04: Checksum Validation

**Priority**: High (security/integrity)
**Effort**: 4-6 hours
**Dependencies**: WP01

**Objective**: Implement streaming SHA256 checksum validation during download.

**Subtasks**:
- [ ] T020: Create src/cache/checksum.rs module
- [ ] T021: Implement StreamingHasher struct with Sha256
- [ ] T022: Integrate hasher with download_file (update per chunk)
- [ ] T023: Implement checksum validation after download complete
- [ ] T024: Add CacheError::ChecksumMismatch variant
- [ ] T025: Add unit tests for checksum validation (including mismatch scenarios)

**Success Criteria**:
- Checksum computed during download (single pass)
- Mismatched checksums detected and reported
- Memory efficient (no file reload needed)
- Unit tests cover success and failure cases

**Risks**:
- Performance impact of hashing (mitigation: benchmark shows negligible overhead)

**Parallel with**: WP03 (different files)

---

### WP05: Resume Logic

**Priority**: Medium (UX enhancement)
**Effort**: 6-8 hours
**Dependencies**: WP01, WP02, WP03

**Objective**: Implement download resume capability using HTTP Range requests.

**Subtasks**:
- [ ] T026: Implement resume_download() function in download.rs
- [ ] T027: Check for existing .part files and get current size
- [ ] T028: Send HTTP Range header with starting byte offset
- [ ] T029: Append new bytes to existing .part file (don't truncate)
- [ ] T030: Validate checksum of complete file after resume
- [ ] T031: Add unit tests for resume scenarios (wiremock 206 responses)

**Success Criteria**:
- Downloads resume from interruption point
- Checksum validation works after resume
- Resume works across process restarts
- Unit tests verify Range request headers
- Integration tests verify end-to-end resume

**Risks**:
- Server doesn't support Range (mitigation: fallback to full re-download)
- Partial file corruption (mitigation: validate checksum, delete and restart)

---

### WP06: Manifest Integration

**Priority**: High (data integrity)
**Effort**: 4-6 hours
**Dependencies**: WP03

**Objective**: Integrate download with cache manifest using atomic updates and file locking.

**Subtasks**:
- [ ] T032: Enhance src/cache/manifest.rs with fd-lock dependency
- [ ] T033: Implement atomic manifest updates (read → modify → write with lock)
- [ ] T034: Add model entry to manifest after successful download
- [ ] T035: Update CacheManager::download_model() to call download logic
- [ ] T036: Handle manifest lock contention gracefully (fail fast)
- [ ] T037: Add unit tests for concurrent manifest updates

**Success Criteria**:
- Manifest updates are atomic (no partial writes)
- Concurrent operations don't corrupt manifest
- Lock contention handled gracefully
- Downloaded models appear in manifest
- Unit tests verify locking behavior

**Risks**:
- Deadlocks (mitigation: short critical sections, timeouts)
- Lock file corruption (mitigation: robust error handling)

---

### WP07: Progress UI

**Priority**: Medium (UX enhancement)
**Effort**: 4-6 hours
**Dependencies**: WP01, WP03

**Objective**: Implement progress bar with download speed and ETA using indicatif.

**Subtasks**:
- [ ] T038: Create src/cache/progress.rs module
- [ ] T039: Implement DownloadProgress struct (bytes, speed, ETA)
- [ ] T040: Configure indicatif ProgressBar with template from plan
- [ ] T041: Integrate progress bar with download_file
- [ ] T042: Calculate download speed and ETA (rolling average)
- [ ] T043: Handle progress bar cleanup on completion/error
- [ ] T044: Manual testing for visual verification

**Success Criteria**:
- Progress bar displays bytes downloaded / total bytes
- Download speed shown in human-readable format (MB/s)
- ETA calculated and displayed
- Bar updates smoothly (60Hz)
- Clean terminal output on completion

**Risks**:
- Terminal compatibility issues (mitigation: indicatif handles this)

**Parallel with**: WP08

---

### WP08: Error Handling

**Priority**: High (reliability)
**Effort**: 4-6 hours
**Dependencies**: WP01-WP05

**Objective**: Implement comprehensive error handling with user-friendly messages.

**Subtasks**:
- [ ] T045: Add CacheError variants (DownloadFailed, NetworkError, ChecksumMismatch, ResumeNotSupported, AuthenticationRequired)
- [ ] T046: Implement From<reqwest::Error> for CacheError
- [ ] T047: Implement From<std::io::Error> for CacheError
- [ ] T048: Add user-friendly error messages for each variant
- [ ] T049: Implement error logging with context (tracing)
- [ ] T050: Add unit tests for error conversions and messages

**Success Criteria**:
- All error types have CacheError variants
- Error messages are user-friendly (no raw stack traces)
- Error chain preserved (source() works)
- Logging provides debugging context
- Unit tests cover all error paths

**Risks**:
- Error messages too technical (mitigation: user testing)

**Parallel with**: WP07

---

### WP09: Unit Tests

**Priority**: High (quality gate)
**Effort**: 6-8 hours
**Dependencies**: WP01-WP08

**Objective**: Comprehensive unit tests using wiremock for HTTP mocking.

**Subtasks**:
- [ ] T051: Create tests/unit/cache/download_tests.rs structure
- [ ] T052: Test successful download (200 OK) with wiremock
- [ ] T053: Test resume from partial (206 Partial Content)
- [ ] T054: Test network errors (timeouts, connection refused)
- [ ] T055: Test authentication failures (401 Unauthorized)
- [ ] T056: Test checksum mismatches
- [ ] T057: Test server errors (500, 503)
- [ ] T058: Verify >90% code coverage for download module (cargo tarpaulin)

**Success Criteria**:
- All unit tests pass
- Code coverage >90% for new modules
- wiremock properly mocks HF Hub API
- Tests are deterministic and fast (< 5 seconds total)
- Error cases thoroughly covered

**Risks**:
- Flaky tests (mitigation: avoid real network calls, use wiremock)

**Parallel with**: WP10, WP11

---

### WP10: Integration Tests

**Priority**: Medium (end-to-end validation)
**Effort**: 4-6 hours
**Dependencies**: WP01-WP08

**Objective**: End-to-end integration tests with test fixtures.

**Subtasks**:
- [ ] T059: Create tests/integration/cache/download_integration_tests.rs
- [ ] T060: Create test fixture (small model file < 1MB with known checksum)
- [ ] T061: Test end-to-end download with fixture (setup mock server)
- [ ] T062: Test resume after simulated interruption (kill process mid-download)
- [ ] T063: Test checksum validation with fixture
- [ ] T064: Test error recovery scenarios (retry on failure)
- [ ] T065: Test concurrent cache operations don't corrupt manifest

**Success Criteria**:
- Integration tests pass reliably
- Test fixtures are committed to repo
- Resume scenario works end-to-end
- Checksum validation verified with known good file
- No manifest corruption under concurrent load

**Risks**:
- Test fixtures too large (mitigation: use < 1MB files)
- Flaky integration tests (mitigation: proper setup/teardown)

**Parallel with**: WP09, WP11

---

### WP11: Documentation

**Priority**: Medium (user enablement)
**Effort**: 2-4 hours
**Dependencies**: WP01-WP10 (should be last)

**Objective**: Update documentation with download testing and usage guidance.

**Subtasks**:
- [ ] T066: Update CONTRIBUTING.md with download testing section
- [ ] T067: Document HF_TOKEN usage and authentication
- [ ] T068: Document resume capability and limitations
- [ ] T069: Document wiremock testing patterns for contributors
- [ ] T070: Add download examples to CONTRIBUTING.md
- [ ] T071: Update README.md with download command usage (if user-facing)

**Success Criteria**:
- CONTRIBUTING.md has download testing section
- HF_TOKEN usage documented
- Resume capability documented
- Examples clear and copy-pasteable
- Documentation reviewed and approved

**Risks**:
- Outdated quickly (mitigation: keep examples simple)

**Parallel with**: WP09, WP10

---

## Dependencies Graph

```
WP01 (Setup)
  ├─> WP02 (HTTP Client)
  ├─> WP03 (Download Orchestrator)
  ├─> WP04 (Checksum Validation)
  └─> WP07 (Progress UI)

WP02 (HTTP Client)
  ├─> WP03 (Download Orchestrator)
  └─> WP05 (Resume Logic)

WP03 (Download Orchestrator)
  ├─> WP05 (Resume Logic)
  └─> WP06 (Manifest Integration)

WP01-WP05
  └─> WP08 (Error Handling)

WP01-WP08
  ├─> WP09 (Unit Tests)
  ├─> WP10 (Integration Tests)
  └─> WP11 (Documentation)
```

## Acceptance Testing

After all work packages complete, verify:

1. **Functional**:
   - [ ] Can download public models without HF_TOKEN
   - [ ] Can download private models with HF_TOKEN
   - [ ] Resume works after Ctrl+C interruption
   - [ ] Checksum validation prevents corrupted files
   - [ ] Error messages are user-friendly

2. **Performance**:
   - [ ] Memory usage < 10MB during download
   - [ ] Download speed not CPU-bottlenecked
   - [ ] Progress updates smooth (no freezing)

3. **Testing**:
   - [ ] All unit tests pass (`cargo test`)
   - [ ] All integration tests pass
   - [ ] Code coverage >90% for new modules

4. **Documentation**:
   - [ ] CONTRIBUTING.md updated
   - [ ] Examples work as documented
   - [ ] README updated (if needed)

## Notes

- **MVP Scope**: WP01-WP06 constitute the minimum viable product (basic download without resume/progress)
- **Enhanced UX**: WP05, WP07 add resume and progress (important for large files)
- **Quality Gates**: WP09-WP11 ensure reliability and maintainability
- **Parallel Work**: Up to 3 work packages can be developed in parallel during Phases 2-4
- **Estimated Duration**: 7-10 days for complete implementation (single developer)

## Next Steps

1. Review this task breakdown for completeness
2. Start with WP01 (Setup & Dependencies)
3. Use `/spec-kitty.implement WP01` to begin implementation
4. Mark subtasks as complete in this file as work progresses
5. Run `/spec-kitty.review` after each work package completes

---

**Generated**: 2026-01-08
**Last Updated**: 2026-01-08
