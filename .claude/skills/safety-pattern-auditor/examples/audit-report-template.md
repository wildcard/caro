# Safety Pattern Audit Report

**Date**: [YYYY-MM-DD]
**Auditor**: [Name/Team]
**Audit Type**: [Pre-Release / Monthly / Post-Incident]
**Duration**: [X hours]

---

## Executive Summary

**Overall Status**: [🟢 GOOD / 🟡 NEEDS ATTENTION / 🔴 CRITICAL]

### Key Metrics
- **Total Patterns**: [X]
- **Patterns by Risk**:
  - Critical: [X]
  - High: [X]
  - Medium: [X]
- **Gaps Found**: [X total] ([X critical], [X high], [X medium], [X low])
- **Regressions**: [X]
- **False Positives**: [X]
- **Test Coverage**: [X%]

### Key Findings
1. [Most critical finding]
2. [Second most critical finding]
3. [Third most critical finding]

### Recommendation
[One-sentence recommendation: e.g., "Address 3 P0 gaps before release" or "PASS - safe to release"]

---

## 1. Baseline Metrics

### Pattern Coverage

| Metric | Count | Change from Last Audit |
|--------|-------|------------------------|
| Total Patterns | [X] | [+/-X] |
| Critical Risk | [X] | [+/-X] |
| High Risk | [X] | [+/-X] |
| Medium Risk | [X] | [+/-X] |

### Test Results

| Suite | Tests | Passed | Failed | Pass Rate |
|-------|-------|--------|--------|-----------|
| Static Matcher | [X] | [X] | [X] | [X%] |
| Embedded Backend | [X] | [X] | [X] | [X%] |
| Total | [X] | [X] | [X] | [X%] |

### Baseline File
```
Location: .claude/audits/[date]/baseline.json
Captured: [timestamp]
Git Commit: [commit hash]
```

---

## 2. Gap Analysis

### Summary by Type

| Gap Type | Critical | High | Medium | Low | Total |
|----------|----------|------|--------|-----|-------|
| Argument Order | [X] | [X] | [X] | [X] | [X] |
| Path Variant | [X] | [X] | [X] | [X] | [X] |
| Wildcard | [X] | [X] | [X] | [X] | [X] |
| Platform | [X] | [X] | [X] | [X] | [X] |
| **Total** | **[X]** | **[X]** | **[X]** | **[X]** | **[X]** |

### Critical Gaps (Detailed)

#### GAP-[ID]: [Gap Name]
- **Type**: [argument_order / path_variant / wildcard / platform]
- **Severity**: Critical
- **Command**: `[dangerous command example]`
- **Missing Variant**: `[specific variant not covered]`
- **Impact**: [Description of what could go wrong]
- **Likelihood**: [High / Medium / Low]
- **Current Coverage**: [None / Partial]

[Repeat for each critical gap]

---

## 3. Regression Testing

### Compilation Status
- ✅ All patterns compile successfully
- ✅ No syntax errors
- ✅ No type errors

### Unit Tests
```
cargo test --lib safety::patterns
Result: [X] passed, [X] failed
Status: [PASS / FAIL]
```

### False Positive Check

| Command | Expected | Actual | Status |
|---------|----------|--------|--------|
| `cd ..` | ALLOWED | ALLOWED | ✅ |
| `ls -la` | ALLOWED | ALLOWED | ✅ |
| `cp file backup/` | ALLOWED | ALLOWED | ✅ |
| `cat ../README` | ALLOWED | ALLOWED | ✅ |

### Regression Summary
- Pattern Count: [X] → [X] ([+/-X])
- Risk Distribution: [Stable / Changed]
- Test Pass Rate: [X%] → [X%] ([+/-X%])
- **Status**: [NO REGRESSIONS / REGRESSIONS FOUND]

---

## 4. Priority Assessment

### Priority Matrix

| Priority | Count | Criteria | Est. Effort |
|----------|-------|----------|-------------|
| P0 | [X] | Critical + High likelihood + Easy exploit | [X hours] |
| P1 | [X] | Critical or High + Likely | [X hours] |
| P2 | [X] | Medium risk or Complex fix | [X hours] |
| P3 | [X] | Low risk, document only | [X hours] |

### P0 Gaps (Must Fix Before Release)

1. **GAP-[ID]**: [Name]
   - Command: `[example]`
   - Effort: [X hours]
   - Owner: [Team/Person]
   - Due: [Date]

2. **GAP-[ID]**: [Name]
   - Command: `[example]`
   - Effort: [X hours]
   - Owner: [Team/Person]
   - Due: [Date]

[Continue for all P0 gaps]

### P1 Gaps (Fix This Release)

[List with same format]

---

## 5. Recommendations

### Immediate Actions (P0)
1. **[Action 1]**
   - Gap: GAP-[ID]
   - Effort: [X hours]
   - Owner: [Name]
   - Due: [Date]

2. **[Action 2]**
   - Gap: GAP-[ID]
   - Effort: [X hours]
   - Owner: [Name]
   - Due: [Date]

### Short-term (P1) - Next 2 Weeks
- [Action with effort and owner]
- [Action with effort and owner]

### Long-term (P2) - Next Quarter
- [Action with effort and owner]
- [Action with effort and owner]

### Process Improvements
- [ ] Add gap analyzer to CI/CD pipeline
- [ ] Schedule monthly audits
- [ ] Create gap tracking dashboard
- [ ] Improve test coverage to [X%]

---

## 6. Trend Analysis

### Historical Comparison

| Date | Patterns | Critical Gaps | High Gaps | Total Gaps | Trend |
|------|----------|---------------|-----------|------------|-------|
| [Date 1] | [X] | [X] | [X] | [X] | - |
| [Date 2] | [X] | [X] | [X] | [X] | [↑/↓/→] |
| [Date 3] | [X] | [X] | [X] | [X] | [↑/↓/→] |
| **Today** | **[X]** | **[X]** | **[X]** | **[X]** | [↑/↓/→] |

### Observations
- Pattern growth rate: [X patterns/month]
- Gap closure rate: [X gaps/month]
- Quality trend: [Improving / Stable / Declining]
- Test coverage trend: [Improving / Stable / Declining]

---

## 7. Risk Assessment

### Current Risk Level: [LOW / MEDIUM / HIGH / CRITICAL]

**Justification**:
- [X] P0 gaps present
- [X] critical patterns missing
- [X%] of dangerous commands not covered
- [Description of highest-risk gap]

### Risk Mitigation
- [Mitigation 1]
- [Mitigation 2]
- [Mitigation 3]

---

## 8. Next Steps

### Before Release
- [ ] Complete P0 gap fixes
- [ ] Re-run regression tests
- [ ] Verify false positive checks
- [ ] Update documentation

### Post-Release
- [ ] Address P1 gaps
- [ ] Begin P2 gap work
- [ ] Schedule next audit
- [ ] Review process improvements

### Action Items with Owners

| Item | Owner | Due Date | Status |
|------|-------|----------|--------|
| [Action 1] | [Name] | [Date] | [TODO/IN PROGRESS/DONE] |
| [Action 2] | [Name] | [Date] | [TODO/IN PROGRESS/DONE] |

---

## 9. Artifacts

### Files Generated
- Baseline: `.claude/audits/[date]/baseline.json`
- Gap Report: `.claude/audits/[date]/gaps.md`
- Priority Matrix: `.claude/audits/[date]/priority-matrix.md`
- This Report: `.claude/audits/[date]/audit-report.md`

### Commands Run
```bash
# Baseline
./scripts/capture-safety-baseline.sh .claude/audits/[date]/baseline.json

# Gap Analysis
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs \
  -o .claude/audits/[date]/gaps.md

# Regression Check
./scripts/check-safety-regressions.sh .claude/audits/[date]/baseline.json
```

---

## 10. Sign-off

### Audit Completion

**Auditor**: [Name]
**Date**: [YYYY-MM-DD]
**Signature**: [Signature/Approval]

### Release Decision

- [  ] **PASS** - Safe to release
- [  ] **CONDITIONAL** - Release after P0 fixes
- [  ] **FAIL** - Do not release until addressed

**Approver**: [Name]
**Date**: [YYYY-MM-DD]
**Notes**: [Any additional context]

---

## Appendices

### A. Full Gap List
See: `.claude/audits/[date]/gaps.md`

### B. Regression Test Output
```
[Paste relevant test output]
```

### C. Gap Analyzer Statistics
```json
{
  "total_patterns": X,
  "total_gaps": X,
  "by_severity": {
    "critical": X,
    "high": X,
    "medium": X,
    "low": X
  }
}
```

---

*Report generated using safety-pattern-auditor skill v1.0.0*
