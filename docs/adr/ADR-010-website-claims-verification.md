# ADR-010: Website Claims Verification Test Suite

**Status:** Accepted
**Date:** 2025-01-03
**Decision Makers:** Engineering Team
**Category:** Testing, Quality Assurance

---

## Context

The caro.sh website makes specific claims about the product's capabilities, including:
- Safety features (52+ patterns, blocks dangerous commands)
- Platform support (macOS, Linux, Windows, BSD)
- Performance targets (sub-100ms startup, sub-2s inference)
- Privacy guarantees (100% offline, no telemetry by default)
- Competitive differentiators (vs Copilot CLI, Warp, etc.)

Currently, there is no automated mechanism to verify that these claims remain accurate as the codebase evolves. This creates a risk of "claim drift" where marketing promises diverge from actual product behavior.

---

## Decision

We will implement a **dedicated Website Claims Verification test suite** with the following characteristics:

### 1. Separate Test Module

Create a new test module `tests/website_claims/` that is independent from the existing contract tests. This separation ensures:
- Clear ownership and purpose
- Can be run independently
- Does not interfere with development-focused tests

### 2. Blackbox Testing Approach

Tests will treat caro as a blackbox, testing only:
- CLI interface behavior
- Observable outputs
- Public API responses

Tests will NOT:
- Access internal implementation details
- Mock internal modules
- Depend on specific implementation patterns

**Rationale:** Blackbox tests are more resilient to refactoring and better reflect actual user experience.

### 3. Claim-to-Test Traceability

Each test will include metadata linking it to the specific website claim:

```rust
/// Website Claim: "52 predefined safety patterns with risk-level assessment"
/// Source: https://caro.sh/#compare
/// Claim ID: SAFETY-001
#[test]
fn test_safety_pattern_count() {
    // ...
}
```

This enables:
- Impact analysis when website copy changes
- Coverage tracking
- Clear ownership

### 4. Non-Blocking GitHub Check

The test suite runs as a separate GitHub Actions job with `continue-on-error: true`:

```yaml
website-claims:
  name: Website Claims Verification
  runs-on: ubuntu-latest
  continue-on-error: true  # Non-blocking
  steps:
    - run: cargo test --test website_claims
```

**Rationale:**
- Initial deployment should be informational, not blocking
- Allows time to fix existing gaps without blocking PRs
- Can be promoted to required check once stable

### 5. Test Categories

| Category | Description | Automation Level |
|----------|-------------|------------------|
| Safety Patterns | Verify pattern count and blocking behavior | Full |
| Platform Support | Verify binary availability and detection | Full |
| Performance | Verify startup and inference times | Partial (thresholds) |
| Privacy/Offline | Verify network isolation | Full (in CI container) |
| Integration | Verify skill manifest and config | Full |
| Comparisons | Verify claimed differentiators | Full |

### 6. CI Matrix Strategy

For platform-specific claims, use a matrix strategy:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    include:
      - os: macos-latest
        performance_tests: true  # Only run perf tests on macOS
```

### 7. Reporting

Generate both:
- **Machine-readable:** JSON report for automation
- **Human-readable:** Markdown summary for PR comments

Example report structure:

```json
{
  "suite": "website-claims",
  "timestamp": "2025-01-03T12:00:00Z",
  "claims_tested": 35,
  "claims_passed": 33,
  "claims_failed": 2,
  "claims_skipped": 3,
  "failures": [
    {
      "claim_id": "PERF-001",
      "claim_text": "Sub-100ms startup time",
      "expected": "< 100ms",
      "actual": "145ms",
      "remediation": "Investigate startup overhead; consider lazy initialization"
    }
  ]
}
```

---

## Alternatives Considered

### Alternative 1: Extend Existing Contract Tests

**Rejected because:**
- Mixes concerns (development contracts vs marketing claims)
- Harder to track claim coverage
- Different audience (developers vs marketing)

### Alternative 2: External Testing Service

**Rejected because:**
- Adds external dependency
- Less integrated with development workflow
- Higher cost and complexity

### Alternative 3: Manual QA Checklist

**Rejected because:**
- Does not scale with release frequency
- Prone to human error
- No enforcement mechanism

---

## Consequences

### Positive

1. **Trust Assurance:** Every PR validates marketing claims
2. **Early Detection:** Regressions are caught before release
3. **Documentation:** Test suite serves as executable documentation
4. **Accountability:** Clear ownership of each claim

### Negative

1. **Maintenance Overhead:** Must keep tests in sync with website
2. **Flakiness Risk:** Performance tests may have variance
3. **Initial Gap:** May reveal existing claim gaps (expected)

### Neutral

1. **Cultural Shift:** Marketing and engineering aligned on claims
2. **Process Change:** Website updates trigger test updates

---

## Implementation Plan

### Phase 1: Foundation (Current)

1. Create test directory structure
2. Implement safety pattern tests
3. Create GitHub workflow
4. Document test suite

### Phase 2: Expansion

1. Add platform matrix tests
2. Add performance benchmarks
3. Add integration verification

### Phase 3: Maturity

1. Promote to required check
2. Add website change detection
3. Integrate with release process

---

## Metrics and Monitoring

| Metric | Target | Current |
|--------|--------|---------|
| Claim Coverage | 100% | TBD |
| Test Pass Rate | > 95% | TBD |
| Flakiness Rate | < 5% | TBD |
| Time to Fix Failures | < 1 sprint | TBD |

---

## Related Documents

- [PRD: Website Claims Verification](../kitty-specs/010-website-claims-qa/spec.md)
- [CI Workflow](../.github/workflows/website-claims.yml)
- [Test Module](../tests/website_claims/)

---

## Decision Record

| Date | Author | Decision |
|------|--------|----------|
| 2025-01-03 | Claude | Initial ADR created |
