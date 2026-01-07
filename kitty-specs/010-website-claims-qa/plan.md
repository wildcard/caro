# Implementation Plan: Website Claims Verification Test Suite

## Overview

This plan outlines the technical implementation of the Website Claims Verification test suite. The goal is to create a comprehensive, maintainable, and automated testing framework that verifies all claims made on caro.sh.

---

## Architecture

### Directory Structure

```
tests/
  website_claims/
    mod.rs                    # Test module root
    claims.rs                 # Claim definitions and metadata
    safety_claims.rs          # Safety pattern verification tests
    platform_claims.rs        # Platform support tests
    privacy_claims.rs         # Offline/privacy tests
    performance_claims.rs     # Performance benchmark tests
    integration_claims.rs     # Claude skill and integration tests
    comparison_claims.rs      # Competitive comparison verification
    report.rs                 # Report generation utilities

.github/
  workflows/
    website-claims.yml        # Dedicated CI workflow
```

### Test Module Design

```rust
// claims.rs - Claim metadata structure
pub struct Claim {
    pub id: &'static str,
    pub category: ClaimCategory,
    pub text: &'static str,
    pub source_url: &'static str,
    pub testable: bool,
    pub status: ClaimStatus,
}

pub enum ClaimCategory {
    Safety,
    Platform,
    Privacy,
    Performance,
    Integration,
    Comparison,
}

pub enum ClaimStatus {
    Verified,
    Failed,
    Skipped,
    NotImplemented,
}
```

---

## Implementation Phases

### Phase 1: Test Infrastructure

**Tasks:**
1. Create test module structure
2. Define claim metadata framework
3. Set up test utilities for blackbox testing
4. Create report generation

**Deliverables:**
- `tests/website_claims/mod.rs`
- `tests/website_claims/claims.rs`
- Report generation utilities

### Phase 2: Safety Claims Tests

**Claims to Verify:**
- SAFETY-001: "52 predefined safety patterns"
- SAFETY-002: "Blocks rm -rf /"
- SAFETY-003: "Blocks fork bombs"
- SAFETY-004: "Risk level assessment"
- SAFETY-005: "Explicit confirmation required"

**Test Strategy:**
```rust
#[test]
fn test_safety_pattern_count() {
    // Use reflection/regex to count patterns in compiled binary
    // or call caro with a diagnostic flag
    let output = Command::new("caro")
        .arg("--list-safety-patterns")
        .output()
        .expect("failed to execute caro");

    let pattern_count = parse_pattern_count(&output.stdout);
    assert!(pattern_count >= 52, "Expected 52+ patterns, got {}", pattern_count);
}

#[test]
fn test_blocks_rm_rf_root() {
    let output = Command::new("caro")
        .arg("--validate-only")
        .arg("rm -rf /")
        .output()
        .expect("failed to execute caro");

    assert!(output.status.code() != Some(0), "Should block rm -rf /");
    assert!(String::from_utf8_lossy(&output.stdout).contains("blocked"));
}
```

### Phase 3: Platform Claims Tests

**Claims to Verify:**
- PLATFORM-001: "Cross-platform: macOS, Linux, Windows"
- PLATFORM-002: "GNU vs BSD syntax awareness"
- PLATFORM-003: "Platform-aware recommendations"

**Test Strategy:**
- Use CI matrix to run on multiple OS
- Check for platform-specific binary artifacts in releases
- Verify platform detection output

### Phase 4: Privacy Claims Tests

**Claims to Verify:**
- PRIVACY-001: "Works 100% offline"
- PRIVACY-002: "No telemetry by default"
- PRIVACY-003: "Air-gapped friendly"

**Test Strategy:**
```rust
#[test]
fn test_offline_operation() {
    // Run in a container with network disabled
    // Verify command generation works
    let output = Command::new("caro")
        .env("CARO_OFFLINE_TEST", "1")
        .env("CARO_BACKEND", "embedded")
        .arg("list files")
        .output()
        .expect("failed to execute caro");

    assert!(output.status.success(), "Should work offline with embedded backend");
}
```

### Phase 5: Performance Claims Tests

**Claims to Verify:**
- PERF-001: "Sub-100ms startup time"
- PERF-002: "Sub-2s inference on Apple Silicon"

**Test Strategy:**
- Use `std::time::Instant` for timing
- Run multiple iterations for statistical significance
- Use P95 percentiles, not single measurements
- Only run Apple Silicon tests on macos-latest with M-series

### Phase 6: Integration Claims Tests

**Claims to Verify:**
- INTEG-001: "Official Claude Code skill"
- INTEG-002: "Install with /plugin install wildcard/caro"

**Test Strategy:**
- Verify skill manifest file exists
- Verify skill configuration is valid JSON/YAML
- Check for required skill metadata

### Phase 7: Comparison Claims Tests

**Claims to Verify:**
- COMPARE-001: "Rule-based safety checks (vs competitors: No)"
- COMPARE-002: "Local inference supported"
- COMPARE-003: "Multi-backend support"

**Test Strategy:**
- Verify each "Yes" claim with functional tests
- Document test coverage for comparison table

---

## CI/CD Integration

### GitHub Workflow

```yaml
name: Website Claims Verification

on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [main]

jobs:
  website-claims:
    name: Verify Website Claims
    runs-on: ${{ matrix.os }}
    continue-on-error: true  # Non-blocking
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: macos-latest
            run_performance_tests: true

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run Website Claims Tests
        run: cargo test --test website_claims -- --test-threads=1
        env:
          CARO_CLAIMS_TEST: "1"

      - name: Generate Report
        if: always()
        run: cargo test --test website_claims -- --report-json > claims-report.json

      - name: Upload Report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: claims-report-${{ matrix.os }}
          path: claims-report.json
```

---

## Test Utilities

### Blackbox Test Helper

```rust
// tests/website_claims/test_utils.rs

use std::process::Command;

pub struct CaroTestRunner {
    binary_path: PathBuf,
    env_vars: HashMap<String, String>,
}

impl CaroTestRunner {
    pub fn new() -> Self {
        let binary_path = env::var("CARO_TEST_BINARY")
            .map(PathBuf::from)
            .unwrap_or_else(|_| cargo_bin("caro"));

        Self {
            binary_path,
            env_vars: HashMap::new(),
        }
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate_command(&self, command: &str) -> ValidationResult {
        let output = Command::new(&self.binary_path)
            .arg("--validate-only")
            .arg(command)
            .envs(&self.env_vars)
            .output()
            .expect("failed to execute caro");

        ValidationResult::from_output(output)
    }
}
```

### Report Generator

```rust
// tests/website_claims/report.rs

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ClaimsReport {
    pub suite: String,
    pub timestamp: String,
    pub claims_tested: usize,
    pub claims_passed: usize,
    pub claims_failed: usize,
    pub claims_skipped: usize,
    pub failures: Vec<ClaimFailure>,
}

#[derive(Serialize, Deserialize)]
pub struct ClaimFailure {
    pub claim_id: String,
    pub claim_text: String,
    pub expected: String,
    pub actual: String,
    pub remediation: String,
}
```

---

## Maintenance Strategy

### Website Sync Process

1. **Monitor website changes** (manual or future automation)
2. **Update claims.rs** when website copy changes
3. **Add/modify tests** to match new claims
4. **Track coverage** in claims registry

### Test Stability

- Use retries for flaky tests (max 2 retries)
- Use generous thresholds for performance tests
- Document known variance sources

---

## Success Criteria

- [ ] All testable claims have corresponding tests
- [ ] Tests run on every PR
- [ ] Tests pass on main branch
- [ ] Report is generated and archived
- [ ] ADR and spec are complete

---

## Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Infrastructure | 2-3 hours | None |
| Safety Tests | 2-3 hours | Infrastructure |
| Platform Tests | 1-2 hours | Infrastructure |
| Privacy Tests | 1-2 hours | Infrastructure |
| Performance Tests | 2-3 hours | Infrastructure |
| Integration Tests | 1 hour | Infrastructure |
| CI Workflow | 1 hour | All tests |
| Documentation | 1 hour | All above |
