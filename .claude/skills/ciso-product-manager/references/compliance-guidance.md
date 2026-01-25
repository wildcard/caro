# Compliance Guidance for Caro

## Overview

While caro is an open-source CLI tool (not a cloud service), understanding compliance frameworks helps build better security practices and prepares for enterprise adoption.

## Relevant Frameworks

### Privacy Regulations

#### GDPR Principles (When Applicable)

If caro collects any user data (telemetry), these principles apply:

| Principle | Caro Application |
|-----------|------------------|
| **Lawfulness** | User consent for telemetry (opt-in for GA) |
| **Purpose Limitation** | Data only for product improvement |
| **Data Minimization** | Collect only what's necessary |
| **Accuracy** | Ensure collected data is valid |
| **Storage Limitation** | Define retention periods |
| **Integrity & Confidentiality** | Secure storage and transmission |
| **Accountability** | Document processing activities |

#### Right to Erasure

```
User Request: "Delete my data"

Caro Response:
1. Telemetry is anonymous (session IDs rotate daily)
2. No personal data is collected by design
3. Local storage can be deleted: ~/.caro/telemetry.db
4. If opted-out, no data leaves the machine
```

### Security Standards

#### ISO 27001 Alignment (Reference Only)

Caro's development practices align with key ISO 27001 controls:

| Control | Caro Implementation |
|---------|---------------------|
| A.8.2 - Information Classification | Safety patterns categorized by risk |
| A.12.2 - Protection from Malware | Pattern detection for malicious commands |
| A.12.6 - Technical Vulnerability Management | Security advisory process |
| A.14.2 - Security in Development | Security review process, testing |

#### NIST Cybersecurity Framework

| Function | Caro Relevance |
|----------|----------------|
| Identify | Risk assessment for features |
| Protect | Safety patterns, user confirmation |
| Detect | Pattern matching, telemetry for anomalies |
| Respond | Error handling, graceful degradation |
| Recover | Clear error messages, user guidance |

### POSIX Compliance

Caro validates commands for POSIX compliance:

```rust
/// POSIX validation categories
pub enum PosixCheck {
    /// Command uses POSIX-standard utilities
    StandardUtilities,

    /// Options use POSIX format (-a, not --all)
    OptionFormat,

    /// Portable path handling
    PathHandling,

    /// Cross-shell compatibility
    ShellCompatibility,
}
```

**Why This Matters**:
- Commands work across Linux, macOS, BSD
- Reduces platform-specific bugs
- Improves command reliability

## Privacy-by-Design Principles

### 1. Proactive, Not Reactive

Design for privacy from the start, not after.

**Implementation**:
- Telemetry designed with privacy first
- Data minimization built in
- Redaction layers before collection

### 2. Privacy as the Default

Privacy protection should not require user action.

**Implementation**:
- GA: Telemetry off by default
- Never collect command content
- Local-first architecture

### 3. Privacy Embedded in Design

Privacy is core functionality, not an add-on.

**Implementation**:
- Privacy module is core, not optional
- Redaction in multiple layers
- Session ID rotation (daily)

### 4. Full Functionality

Privacy doesn't mean reduced features.

**Implementation**:
- Full features work without telemetry
- Air-gapped mode fully supported
- No "premium for privacy"

### 5. End-to-End Security

Data protected throughout lifecycle.

**Implementation**:
- Local encryption for telemetry DB
- TLS for any transmission
- Secure deletion when requested

### 6. Visibility and Transparency

Users know what's happening with their data.

**Implementation**:
- `caro telemetry show` command
- Privacy policy documentation
- Clear consent prompts

### 7. Respect for User Privacy

User interests come first.

**Implementation**:
- No dark patterns for consent
- Easy opt-out
- No punishment for opting out

## Telemetry Compliance Checklist

Before collecting any telemetry:

- [ ] **Purpose defined**: Why is this data needed?
- [ ] **Minimization applied**: Is this the minimum data?
- [ ] **Consent obtained**: Has user agreed?
- [ ] **Redaction verified**: Are sensitive patterns removed?
- [ ] **Retention defined**: How long is data kept?
- [ ] **Deletion possible**: Can data be removed on request?
- [ ] **Documentation updated**: Is privacy policy current?

## Data Classification

### What Caro NEVER Collects

| Data Type | Reason |
|-----------|--------|
| Command content | Could contain credentials, paths |
| File paths | Reveals directory structure |
| Environment variables | May contain secrets |
| Natural language input | User intent is private |
| IP addresses | Location tracking concern |
| User identity | Anonymous by design |

### What Caro MAY Collect (With Consent)

| Data Type | Purpose | Retention |
|-----------|---------|-----------|
| Session metadata | Usage patterns | 90 days |
| Command success/fail | Quality improvement | 90 days |
| Backend used | Performance analysis | 90 days |
| Latency metrics | Optimization | 90 days |
| Error categories | Bug prioritization | 90 days |

### Redaction Patterns

```rust
/// Patterns always redacted from any collected data
const REDACTION_PATTERNS: &[&str] = &[
    // Credentials
    r"(?i)(password|passwd|pwd)[=:]\S+",
    r"(?i)(api[_-]?key|apikey)[=:]\S+",
    r"(?i)(secret|token)[=:]\S+",

    // Paths
    r"/home/\w+",
    r"/Users/\w+",
    r"C:\\Users\\\w+",

    // URLs with auth
    r"https?://[^:]+:[^@]+@",

    // SSH keys
    r"-----BEGIN .* KEY-----",

    // AWS
    r"AKIA[A-Z0-9]{16}",
    r"(?i)aws[_-]?secret",
];
```

## Security Advisory Process

### Vulnerability Disclosure

1. **Report**: security@caro.sh (when available) or GitHub Security Advisory
2. **Acknowledge**: Within 48 hours
3. **Assess**: Determine severity (CVSS)
4. **Fix**: Develop patch
5. **Release**: Coordinate disclosure
6. **Document**: Update advisories

### Severity Classification

| CVSS Score | Severity | Response Time |
|------------|----------|---------------|
| 9.0-10.0 | Critical | 24 hours |
| 7.0-8.9 | High | 7 days |
| 4.0-6.9 | Medium | 30 days |
| 0.1-3.9 | Low | Next release |

## Enterprise Considerations

For enterprise adoption, caro should support:

### Configuration Management

```toml
# Enterprise-friendly defaults
[enterprise]
# Allow central policy push
policy_url = "https://internal.corp/caro-policy.toml"

# Audit logging for compliance
audit_log = true
audit_destination = "/var/log/caro/audit.log"

# Restrict backends to approved list
allowed_backends = ["embedded", "vllm"]

# Mandatory safety level
min_safety_level = "high"
```

### Audit Logging

```json
{
  "timestamp": "2026-01-11T12:00:00Z",
  "event": "command_generated",
  "command_hash": "sha256:abc123...",
  "risk_level": "medium",
  "user_action": "executed",
  "backend": "embedded"
}
```

### Integration Points

- **SIEM integration**: Log forwarding for security monitoring
- **SSO**: Enterprise authentication (future)
- **Policy as Code**: Custom safety rules (Dogma engine)

## Documentation Requirements

### Privacy Policy

Must cover:
- What data is collected
- How data is used
- How data is protected
- How to opt out
- Data retention periods
- Contact information

### Security Documentation

Must cover:
- Threat model
- Security architecture
- Vulnerability disclosure process
- Security update policy
- Incident response (if applicable)

## Compliance Roadmap

### Current (v1.x)

- [x] Privacy-by-design architecture
- [x] Telemetry consent flow
- [x] Data minimization
- [x] Redaction layers
- [ ] Formal privacy policy

### Future (v2.x)

- [ ] Audit logging capability
- [ ] Enterprise policy support
- [ ] Compliance documentation
- [ ] Third-party security audit
- [ ] SOC 2 Type II preparation (if SaaS components)
