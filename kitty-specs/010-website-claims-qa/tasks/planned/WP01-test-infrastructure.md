# WP01: Test Infrastructure Setup

**Status:** Planned
**Priority:** P0 (Blocking)
**Estimated Effort:** 2-3 hours

---

## Objective

Set up the foundational test infrastructure for the Website Claims Verification test suite.

---

## Implementation Steps

### Step 1: Create Directory Structure

```bash
mkdir -p tests/website_claims
```

### Step 2: Create mod.rs

Create `tests/website_claims/mod.rs` with:
- Module declarations
- Common imports
- Test organization

### Step 3: Create claims.rs

Implement claim metadata types:

```rust
pub struct Claim {
    pub id: &'static str,
    pub category: ClaimCategory,
    pub text: &'static str,
    pub source_url: &'static str,
    pub testable: bool,
}

pub enum ClaimCategory {
    Safety,
    Platform,
    Privacy,
    Performance,
    Integration,
    Comparison,
}

// Registry of all website claims
pub static CLAIMS: &[Claim] = &[
    Claim {
        id: "SAFETY-001",
        category: ClaimCategory::Safety,
        text: "52 predefined safety patterns with risk-level assessment",
        source_url: "https://caro.sh/#compare",
        testable: true,
    },
    // ... more claims
];
```

### Step 4: Create test_utils.rs

Implement blackbox test utilities:
- CaroTestRunner struct
- Command execution helpers
- Output parsing utilities

### Step 5: Create report.rs

Implement report generation:
- JSON report structure
- Markdown summary generation
- Failure remediation suggestions

---

## Acceptance Criteria

- [ ] `tests/website_claims/mod.rs` exists and compiles
- [ ] `tests/website_claims/claims.rs` has claim types
- [ ] `tests/website_claims/test_utils.rs` has test helpers
- [ ] `tests/website_claims/report.rs` has report types
- [ ] `cargo test --test website_claims` runs (even if empty)

---

## Definition of Done

- All files created and compile without errors
- Basic test structure can be executed
- Module organization supports future test additions
