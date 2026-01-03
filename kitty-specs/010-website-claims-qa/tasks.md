# Tasks: Website Claims Verification Test Suite

## Work Packages Overview

| WP | Title | Status | Lane |
|----|-------|--------|------|
| WP01 | Test Infrastructure Setup | Planned | planned |
| WP02 | Safety Claims Tests | Planned | planned |
| WP03 | Platform Claims Tests | Planned | planned |
| WP04 | Privacy and Offline Tests | Planned | planned |
| WP05 | Performance Benchmark Tests | Planned | planned |
| WP06 | Integration Claims Tests | Planned | planned |
| WP07 | GitHub Actions Workflow | Planned | planned |
| WP08 | Documentation and Polish | Planned | planned |

---

## WP01: Test Infrastructure Setup

**Status:** Planned
**Priority:** P0 (Blocking)
**Estimated Effort:** 2-3 hours

### Description

Set up the foundational test infrastructure including directory structure, claim metadata framework, and test utilities.

### Acceptance Criteria

- [ ] Create `tests/website_claims/` directory structure
- [ ] Implement `mod.rs` with test module organization
- [ ] Implement `claims.rs` with claim metadata types
- [ ] Create test utilities for blackbox testing
- [ ] Add report generation utilities

### Files to Create/Modify

- `tests/website_claims/mod.rs`
- `tests/website_claims/claims.rs`
- `tests/website_claims/test_utils.rs`
- `tests/website_claims/report.rs`

---

## WP02: Safety Claims Tests

**Status:** Planned
**Priority:** P0 (Critical - Core Feature)
**Estimated Effort:** 2-3 hours

### Description

Implement tests for all safety-related claims including pattern count, dangerous command blocking, and risk assessment.

### Acceptance Criteria

- [ ] Test: 52+ safety patterns exist
- [ ] Test: `rm -rf /` is blocked
- [ ] Test: Fork bombs are blocked
- [ ] Test: Pipe-to-shell attacks are blocked
- [ ] Test: Each blocked command has explanation
- [ ] Test: Risk levels are assigned correctly

### Claims Covered

| Claim ID | Claim Text |
|----------|------------|
| SAFETY-001 | "52 predefined safety patterns with risk-level assessment" |
| SAFETY-002 | "Blocks dangerous commands like rm -rf /, fork bombs" |
| SAFETY-003 | "Explicit confirmation required before execution" |
| SAFETY-004 | "Risk level assessment for generated commands" |

### Files to Create/Modify

- `tests/website_claims/safety_claims.rs`

---

## WP03: Platform Claims Tests

**Status:** Planned
**Priority:** P1 (High)
**Estimated Effort:** 1-2 hours

### Description

Implement tests for cross-platform support claims including binary availability, platform detection, and GNU/BSD awareness.

### Acceptance Criteria

- [ ] Test: Platform detection works correctly
- [ ] Test: GNU vs BSD syntax is handled
- [ ] Test: Windows path handling works
- [ ] Test: Release assets include all platforms

### Claims Covered

| Claim ID | Claim Text |
|----------|------------|
| PLATFORM-001 | "Cross-platform: macOS, Linux, Windows, GNU, BSD" |
| PLATFORM-002 | "Distinguishes between BSD and GNU command syntax" |
| PLATFORM-003 | "Platform-aware recommendations" |

### Files to Create/Modify

- `tests/website_claims/platform_claims.rs`

---

## WP04: Privacy and Offline Tests

**Status:** Planned
**Priority:** P1 (High)
**Estimated Effort:** 1-2 hours

### Description

Implement tests for privacy and offline operation claims.

### Acceptance Criteria

- [ ] Test: Embedded backend works without network
- [ ] Test: No telemetry by default
- [ ] Test: Air-gapped operation verified

### Claims Covered

| Claim ID | Claim Text |
|----------|------------|
| PRIVACY-001 | "Works 100% offline" |
| PRIVACY-002 | "Privacy-first design" |
| PRIVACY-003 | "Air-gapped friendly" |
| PRIVACY-004 | "Commands never leave your machine" |

### Files to Create/Modify

- `tests/website_claims/privacy_claims.rs`

---

## WP05: Performance Benchmark Tests

**Status:** Planned
**Priority:** P2 (Medium)
**Estimated Effort:** 2-3 hours

### Description

Implement performance benchmark tests for startup time and inference time claims.

### Acceptance Criteria

- [ ] Test: Startup time < 100ms (P95)
- [ ] Test: Inference time < 2s on Apple Silicon (P95)
- [ ] Performance results logged for analysis
- [ ] Tests are skipped on non-Apple Silicon hardware

### Claims Covered

| Claim ID | Claim Text |
|----------|------------|
| PERF-001 | "Sub-100ms startup time" |
| PERF-002 | "Sub-2s inference on Apple Silicon" |

### Files to Create/Modify

- `tests/website_claims/performance_claims.rs`

---

## WP06: Integration Claims Tests

**Status:** Planned
**Priority:** P2 (Medium)
**Estimated Effort:** 1 hour

### Description

Implement tests for Claude skill integration and other integration claims.

### Acceptance Criteria

- [ ] Test: Skill manifest exists and is valid
- [ ] Test: Multi-backend support works
- [ ] Test: Self-hosting is possible

### Claims Covered

| Claim ID | Claim Text |
|----------|------------|
| INTEG-001 | "Official Claude Code skill" |
| INTEG-002 | "Multi-backend support" |
| INTEG-003 | "Self-hostable" |

### Files to Create/Modify

- `tests/website_claims/integration_claims.rs`

---

## WP07: GitHub Actions Workflow

**Status:** Planned
**Priority:** P0 (Blocking)
**Estimated Effort:** 1-2 hours

### Description

Create the GitHub Actions workflow for running website claims tests as a separate, non-blocking check.

### Acceptance Criteria

- [ ] Workflow file created
- [ ] Runs on PRs to main/develop
- [ ] Non-blocking (continue-on-error: true)
- [ ] Runs on matrix of OSes
- [ ] Generates and uploads reports
- [ ] Clear pass/fail status visible

### Files to Create/Modify

- `.github/workflows/website-claims.yml`

---

## WP08: Documentation and Polish

**Status:** Planned
**Priority:** P2 (Medium)
**Estimated Effort:** 1-2 hours

### Description

Complete documentation, add README, and polish the test suite.

### Acceptance Criteria

- [ ] README for test suite
- [ ] Link ADR and spec from README
- [ ] Add test run instructions
- [ ] Document how to add new claims
- [ ] Final code review and cleanup

### Files to Create/Modify

- `tests/website_claims/README.md`
- Update `docs/adr/ADR-010-website-claims-verification.md` with final links

---

## Dependencies

```
WP01 (Infrastructure) --> WP02, WP03, WP04, WP05, WP06
                      --> WP07 (Workflow)
                      --> WP08 (Docs)
```

All test WPs depend on WP01.
WP07 and WP08 can proceed in parallel after WP01.
