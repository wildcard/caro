# QA Automation Loop - Test Document

> **Document Type**: Test Specification
> **DRS Reference**: [QA_LOOP_DRS.md](../specs/QA_LOOP_DRS.md)
> **Skill**: `/qa-automation-loop`

---

## 1. Test Categories

### 1.1 Unit Tests

| ID | Test Case | Input | Expected Output |
|----|-----------|-------|-----------------|
| QA-U01 | Profile loading | Valid yaml | Profiles array |
| QA-U02 | Profile loading (invalid) | Malformed yaml | Error message |
| QA-U03 | Severity classification | Finding with severity | Correct label |
| QA-U04 | Deduplication | 2 similar findings | 1 merged finding |
| QA-U05 | Issue template generation | Finding object | Valid markdown |

### 1.2 Integration Tests

| ID | Test Case | Setup | Expected |
|----|-----------|-------|----------|
| QA-I01 | Profile spawn | Mock unbiased-tester | Findings returned |
| QA-I02 | GitHub issue creation | Mock gh CLI | Issue created |
| QA-I03 | Duplicate detection | Existing issue | No duplicate created |
| QA-I04 | Run report generation | Completed run | YAML file created |
| QA-I05 | Metrics update | Run complete | Metrics.json updated |

### 1.3 End-to-End Tests

| ID | Test Case | Scenario | Validation |
|----|-----------|----------|------------|
| QA-E01 | Full cycle (dry run) | All profiles, no issue creation | Report generated, no issues |
| QA-E02 | Full cycle (live) | Single profile, issue creation | Issue exists on GitHub |
| QA-E03 | Parallel execution | 4 profiles concurrent | All complete, no race conditions |
| QA-E04 | Timeout handling | Slow profile | Graceful timeout, partial results |
| QA-E05 | Error recovery | Profile crash | Other profiles complete |

---

## 2. Test Scenarios

### Scenario 1: Normal Execution

**Given**: Valid configuration with 4 profiles
**When**: `/qa-automation-loop` is executed
**Then**:
- All profiles complete
- Findings are consolidated
- Issues are created for unique findings
- Run report is generated

### Scenario 2: No Findings

**Given**: All profiles pass all tests
**When**: `/qa-automation-loop` is executed
**Then**:
- No issues created
- Report shows 100% pass rate
- Metrics updated

### Scenario 3: Duplicate Finding

**Given**: Finding matches existing GitHub issue
**When**: Issue creation is attempted
**Then**:
- Duplicate detected
- No new issue created
- Link to existing issue in report

### Scenario 4: Profile Failure

**Given**: One profile fails to execute
**When**: `/qa-automation-loop` is executed
**Then**:
- Other profiles complete
- Warning logged
- Partial results reported

---

## 3. Test Data

### Sample Profile

```yaml
name: "test_profile"
description: "Test profile for QA loop"
test_categories:
  - basic_commands
```

### Sample Finding

```yaml
type: "bug"
severity: "medium"
title: "Unexpected output for pipe commands"
reproduction:
  - "Run: caro 'list files | count'"
  - "Observe output format"
expected: "Numeric count"
actual: "JSON object"
command_output: |
  {"count": 5}
```

---

## 4. Validation Checklist

- [ ] All unit tests pass
- [ ] Integration tests pass with mocked dependencies
- [ ] E2E test completes in < 10 minutes
- [ ] No memory leaks during parallel execution
- [ ] Error messages are clear and actionable
- [ ] Metrics are accurate
- [ ] Report format is correct

---

## 5. Manual Testing Steps

1. **Setup**
   ```bash
   cd /path/to/caro
   ```

2. **Dry Run**
   ```bash
   /qa-automation-loop --dry-run --verbose
   ```
   Verify: No issues created, report generated

3. **Single Profile**
   ```bash
   /qa-automation-loop --profiles beginner --dry-run
   ```
   Verify: Only beginner profile runs

4. **Full Run (Staging)**
   ```bash
   /qa-automation-loop
   ```
   Verify: Issues created, report accurate

---

## 6. Performance Requirements

| Metric | Target | Maximum |
|--------|--------|---------|
| Total execution time | < 10 min | 30 min |
| Memory usage | < 500 MB | 1 GB |
| API calls (GitHub) | < 20 | 50 |
