# Root Cause Analysis Template

Use this template to document root cause analysis for each issue identified in beta testing.

---

## Issue #{ID}: {Brief Title}

**Severity**: P{0-2}
**Category**: [Code Bug | Config Issue | Missing Pattern | Docs Mismatch | Performance | Security]
**Reporter**: {Tester name or ID}
**Date Reported**: YYYY-MM-DD

---

## Symptoms

What users observed or experienced:
- {Symptom 1}
- {Symptom 2}
- {Symptom 3}

---

## Expected Behavior

What should happen instead:
- {Expected behavior 1}
- {Expected behavior 2}

---

## Reproduction Steps

Exact steps to reproduce the issue:

1. {Step 1}
2. {Step 2}
3. {Step 3}
4. **Observed**: {What happens}
5. **Expected**: {What should happen}

**Environment**:
- OS: {macOS/Linux/Windows}
- Version: {caro version}
- Configuration: {Relevant config settings}

---

## Root Cause

### Location
- **File**: `{path/to/file.rs}`
- **Line**: {line_number or range}
- **Function/Module**: `{function_name}`

### Problem Description

{Detailed explanation of what's wrong in the code}

**Category-specific details**:

- **If Code Bug**: What assumption was violated? What edge case wasn't handled?
- **If Config Issue**: Was config loaded but not saved? Missing validation?
- **If Missing Pattern**: What query pattern isn't matched? Why?
- **If Docs Mismatch**: What does the code actually do vs what docs claim?

### Code Excerpt

```rust
// BEFORE (broken)
{Show the problematic code}
```

**Why this is broken**:
{Explanation of the logic error}

---

## Impact Analysis

**Frequency**: How often does this occur?
- [ ] Every time (100%)
- [ ] Frequently (>50%)
- [ ] Occasionally (<50%)
- [ ] Rare edge case

**User Impact**:
- Affects: {Which user workflows/personas}
- Severity: {How bad is it when it happens}
- Workaround: {Is there a way to avoid it?}

**Related Issues**:
- Issue #{XXX}: {Related issue with shared root cause}
- Issue #{YYY}: {Another related issue}

---

## Fix

### Solution Description

{High-level description of the fix}

### Code Changes

```rust
// AFTER (fixed)
{Show the corrected code}
```

**Why this fixes it**:
{Explanation of how the fix addresses the root cause}

### Additional Changes

- **File**: `{other_file.rs}` - {What changed and why}
- **File**: `{another_file.rs}` - {What changed and why}

---

## Regression Test

### Test Location
- **File**: `{tests/beta_regression.rs}`
- **Test Function**: `test_issue_{id}_{brief_name}`

### Test Strategy

{Describe what the test validates}

**Test Setup**:
- {What environment/state is needed}

**Test Execution**:
- {What operation is performed}

**Test Assertions**:
- {What conditions are verified}

### Test Code Outline

```rust
#[tokio::test]
async fn test_issue_{id}_{brief_name}() {
    // Setup: {describe setup}

    // Execute: {describe action}

    // Assert: {describe verification}

    // Cleanup (if needed)
}
```

---

## Verification

### Manual Verification Steps

```bash
# Step 1: {Command or action}
# Expected: {What should happen}

# Step 2: {Command or action}
# Expected: {What should happen}

# Step 3: {Command or action}
# Expected: {What should happen}
```

### Edge Cases Tested

- [ ] Edge case 1: {Description}
- [ ] Edge case 2: {Description}
- [ ] Edge case 3: {Description}

---

## Prevention

**How to prevent this class of issue in the future**:

- {Prevention measure 1}
- {Prevention measure 2}
- {Code review checklist item}
- {Linting rule or test pattern}

---

## References

- **Beta Report**: `.claude/releases/BETA-{version}-REPORT.md`
- **Related Issues**: #{XXX}, #{YYY}
- **Commits**: {git commit hash(es)}
- **PR**: #{PR_number}

---

## Notes

{Any additional context, lessons learned, or future considerations}
