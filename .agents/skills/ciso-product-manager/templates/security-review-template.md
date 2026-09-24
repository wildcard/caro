# Security Review Template

Use this template for security reviews of PRs, features, and architectural decisions.

---

## Security Review: [Title]

**Review Date**: [Date]
**Reviewer**: CISO & Product Manager
**Review Type**: [PR Review / Feature Assessment / Architecture Review]
**Reference**: [PR #, Issue #, or Spec link]

---

## 1. Overview

### Description
[Brief description of what is being reviewed]

### Classification

| Attribute | Value |
|-----------|-------|
| **Type** | [Bug Fix / Feature / Refactor / Security Fix / Dependency Update] |
| **Security Relevance** | [Critical / High / Medium / Low / None] |
| **Privacy Impact** | [Yes / No] |
| **User-Facing** | [Yes / No] |

---

## 2. Scope Analysis

### Files Changed

| File | Security Concern | Notes |
|------|------------------|-------|
| [path/to/file.rs] | [Concern or None] | [Additional context] |

### Dependencies

| Dependency | Version | Known Vulnerabilities |
|------------|---------|----------------------|
| [name] | [version] | [Yes/No - link if yes] |

---

## 3. Threat Model (STRIDE)

### Spoofing
- [ ] Concern identified
- Description: [If yes, describe the threat]
- Mitigation: [Required mitigation]

### Tampering
- [ ] Concern identified
- Description: [If yes, describe the threat]
- Mitigation: [Required mitigation]

### Repudiation
- [ ] Concern identified
- Description: [If yes, describe the threat]
- Mitigation: [Required mitigation]

### Information Disclosure
- [ ] Concern identified
- Description: [If yes, describe the threat]
- Mitigation: [Required mitigation]

### Denial of Service
- [ ] Concern identified
- Description: [If yes, describe the threat]
- Mitigation: [Required mitigation]

### Elevation of Privilege
- [ ] Concern identified
- Description: [If yes, describe the threat]
- Mitigation: [Required mitigation]

---

## 4. Security Requirements

| ID | Requirement | Status | Verification |
|----|-------------|--------|--------------|
| SR-001 | [Requirement description] | [Met / Not Met / N/A] | [How verified] |
| SR-002 | [Requirement description] | [Met / Not Met / N/A] | [How verified] |

### Common Requirements Checklist

- [ ] Input validation at trust boundaries
- [ ] Output encoding appropriate for context
- [ ] Error messages don't leak sensitive information
- [ ] Logging doesn't include sensitive data
- [ ] Fail-safe defaults implemented
- [ ] Least privilege principle followed
- [ ] Sensitive data redacted where appropriate

---

## 5. Privacy Assessment

### Data Collection

| Data Element | Collected? | Necessity | Justification |
|--------------|------------|-----------|---------------|
| [Element] | [Yes/No] | [Required/Optional] | [Why needed] |

### Privacy Controls

- [ ] No personal data collected
- [ ] Data minimization applied
- [ ] Consent mechanism in place (if needed)
- [ ] Redaction patterns applied
- [ ] Retention period defined

---

## 6. Test Coverage

### Security Tests

| Test Case | Status | Notes |
|-----------|--------|-------|
| [Test description] | [Exists / Missing] | [Notes] |

### Recommendations

- [ ] Add test for [specific scenario]
- [ ] Add negative test for [attack vector]

---

## 7. Risk Assessment

### Risk Summary

| Risk | Likelihood | Impact | Score | Treatment |
|------|------------|--------|-------|-----------|
| [Risk description] | [1-5] | [1-5] | [L×I] | [Mitigate/Accept/etc.] |

### Residual Risk

**Overall Risk Level**: [Low / Medium / High / Critical]

**Residual Risk Description**: [Description of risk remaining after mitigations]

**Acceptable**: [Yes / No - with conditions]

---

## 8. Recommendation

### Decision

- [ ] **APPROVE** - No security concerns
- [ ] **APPROVE WITH NOTES** - Minor items to address post-merge
- [ ] **REQUEST CHANGES** - Security issues must be resolved before merge
- [ ] **NEEDS DISCUSSION** - Architectural decision required

### Required Actions (Before Merge)

1. [Action item with owner]
2. [Action item with owner]

### Suggested Actions (Post-Merge)

1. [Action item with owner]
2. [Action item with owner]

---

## 9. Sign-Off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Reviewer | [Name] | [Date] | [Approved/Pending] |
| Author Acknowledgment | [Name] | [Date] | [Acknowledged] |

---

## Appendix

### A. Related Security Issues

- [Link to related security issues or advisories]

### B. References

- [Link to relevant documentation, standards, or prior reviews]

### C. Notes

[Additional context or discussion points]
