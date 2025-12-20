# Self-Healing CARO

## Overview

Self-Healing transforms CARO failures into opportunities for continuous improvement. When CARO fails to generate a correct or safe command, the user can opt-in to a closed-loop recovery pipeline that diagnoses the failure, creates actionable issues, generates fixes, and notifies the user when resolved.

**Core Principle**: Every failure should make CARO smarter.

---

## Problem Statement

Currently, when CARO fails to:
- Generate a valid command for a request
- Produce a safe command (blocked by safety validator)
- Understand platform-specific nuances
- Handle edge cases in natural language parsing

...the failure is silent. The user is left frustrated, and CARO learns nothing.

### Goals

1. **Capture failures systematically** with rich diagnostic context
2. **Create actionable GitHub issues** automatically from failure reports
3. **Generate PR candidates** for community review
4. **Notify users** when their reported issues are resolved
5. **Build trust** through transparent, public healing processes

### Non-Goals

- Automatic deployment of fixes without human review
- Collection of sensitive user data
- Real-time fix generation during user sessions
- Replacing traditional bug reporting channels

---

## User Journey

### Phase 1: Failure Detection & Consent

```
User: cmdai "compress all logs older than 30 days"
CARO: ❌ Unable to generate a safe command for this request.

Would you like to help improve CARO by sharing this case?
[Y] Yes, share anonymized diagnostics
[N] No, thanks
[D] Show what would be shared
```

**Consent is explicit and informed.** User can preview exactly what data would be shared.

### Phase 2: Diagnostic Collection (CARO Doctor)

If user opts in, `caro doctor` runs automatically:

```
Collecting diagnostics...
✓ Platform: macOS 14.2 (arm64)
✓ Shell: zsh 5.9
✓ CARO version: 0.3.1
✓ Backend: MLX (Qwen-2.5-0.5B)
✓ Safety level: strict
✓ Request context: [sanitized]
✓ Failure reason: [captured]

Ready to submit. Proceed? [Y/n]
```

### Phase 3: Contact Preferences (Optional)

```
How would you like to be notified when this is resolved?
[1] Email: ________
[2] Twitter/X: @________
[3] GitHub username: ________
[4] No notification needed
```

### Phase 4: Submission & Tracking

```
✓ Case submitted: CARO-2024-001
  Track: https://hub.caro.dev/cases/2024-001
  GitHub: Will be created after analysis

You'll be notified when resolved. Thank you!
```

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         CARO CLI                                 │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│  │ Failure     │──▶│ CARO Doctor │──▶│ Consent & Submit    │   │
│  │ Detection   │   │ (Diagnostic)│   │ (User Interaction)  │   │
│  └─────────────┘   └─────────────┘   └──────────┬──────────┘   │
└──────────────────────────────────────────────────┼──────────────┘
                                                   │
                          ┌────────────────────────┼────────────────────────┐
                          │                        ▼                        │
                          │  ┌─────────────────────────────────────────┐   │
                          │  │         Healing Pipeline Service         │   │
                          │  │  ┌───────────┐  ┌───────────┐           │   │
                          │  │  │ Ingestion │─▶│ Analysis  │           │   │
                          │  │  └───────────┘  └─────┬─────┘           │   │
                          │  │                       │                  │   │
                          │  │  ┌───────────────────┴─────────────┐    │   │
                          │  │  ▼                                  ▼    │   │
                          │  │  ┌─────────────┐   ┌─────────────────┐  │   │
                          │  │  │ Issue Gen   │   │ PR Generation   │  │   │
                          │  │  │ (GitHub)    │   │ (Agent-Assisted)│  │   │
                          │  │  └──────┬──────┘   └────────┬────────┘  │   │
                          │  └─────────┼───────────────────┼───────────┘   │
                          │            │                   │               │
                          │  ┌─────────▼───────────────────▼───────────┐   │
                          │  │            Community Review              │   │
                          │  │  (GitHub Issues/PRs + Hub Discussion)   │   │
                          │  └─────────────────────┬───────────────────┘   │
                          │                        │                       │
                          │  ┌─────────────────────▼───────────────────┐   │
                          │  │         Notification Service             │   │
                          │  │  (Email, Twitter, Hub, CLI on next run) │   │
                          │  └─────────────────────────────────────────┘   │
                          │                                                │
                          │                 CARO Backend                   │
                          └────────────────────────────────────────────────┘
```

### Module Structure

```
src/doctor/
├── mod.rs                 # Public API exports
├── collector.rs           # Diagnostic data collection
├── sanitizer.rs           # PII redaction & data cleaning
├── report.rs              # DiagnosticReport struct & serialization
└── submit.rs              # Submission to healing pipeline

src/healing/
├── mod.rs                 # Self-healing orchestration
├── consent.rs             # User consent workflow
├── contact.rs             # Contact preference management
├── notification.rs        # Local notification on CLI startup
└── tracking.rs            # Case tracking & status
```

---

## CARO Doctor

### Purpose

A lightweight, local diagnostic utility that collects essential context about failures while respecting user privacy.

### Data Collected

| Category | Data | Purpose |
|----------|------|---------|
| **Platform** | OS, version, architecture | Platform-specific bugs |
| **Shell** | Type, version | Shell compatibility |
| **CARO** | Version, build info | Version-specific issues |
| **Backend** | Type, model name | Backend-specific bugs |
| **Config** | Safety level, flags | Configuration context |
| **Request** | Sanitized intent | Understanding user need |
| **Failure** | Error type, message | Root cause analysis |
| **Context** | CWD (hashed), env hints | Reproduction context |

### Data NOT Collected

- Full command history
- File contents
- Environment variables (except shell type)
- API keys or credentials
- Personal file paths (hashed only)
- Network configuration

### DiagnosticReport Structure

```rust
pub struct DiagnosticReport {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub caro_version: String,

    // Platform context
    pub platform: PlatformInfo,
    pub shell: ShellInfo,
    pub backend: BackendInfo,

    // Failure context
    pub request: SanitizedRequest,
    pub failure: FailureInfo,
    pub safety_context: Option<SafetyContext>,

    // User preferences
    pub contact: Option<ContactPreference>,
    pub consent_level: ConsentLevel,
}

pub struct SanitizedRequest {
    pub intent_category: IntentCategory,  // e.g., "file_operations", "compression"
    pub sanitized_text: String,           // PII redacted
    pub complexity_score: u8,             // 1-10
}

pub struct FailureInfo {
    pub failure_type: FailureType,
    pub stage: PipelineStage,             // Generation, Validation, Execution
    pub error_code: Option<String>,
    pub safety_patterns_triggered: Vec<String>,
}
```

---

## Healing Pipeline

### Stage 1: Ingestion

- Receive diagnostic report via secure endpoint
- Validate report structure and consent
- Deduplicate against existing cases
- Assign case ID (e.g., `CARO-2024-001`)

### Stage 2: Analysis

An agent analyzes the report to:

1. **Classify** the failure type
   - Rule gap (missing safety pattern)
   - Platform incompatibility
   - Natural language parsing issue
   - Backend limitation
   - Configuration conflict

2. **Identify** root cause
   - Pattern match against known issues
   - Reproduce in sandbox if possible
   - Determine affected versions/platforms

3. **Propose** fix direction
   - New safety rule needed
   - Code fix required
   - Documentation update
   - Configuration change

4. **Assign** confidence score (0-100)

### Stage 3: Issue Creation

**High Confidence (>70)**: Create GitHub issue with:
- Reproduction context
- Root cause analysis
- Proposed fix
- Affected versions
- Link to Hub case

**Low Confidence (<70)**: Create discussion first:
- Post to Hub for community input
- Gather additional context
- Refine analysis before issue

### Stage 4: PR Generation

For high-confidence cases with clear fixes:

1. Agent generates fix branch
2. Implements proposed solution
3. Adds/updates tests
4. Creates PR with:
   - Linked issue
   - Clear explanation
   - Test coverage
   - Breaking change notes (if any)

**All PRs require human review.** No automatic merges.

### Stage 5: Review & Merge

- Community reviews PR
- CI validates changes
- Maintainer approves
- Merge triggers release pipeline

### Stage 6: Notification

When fix is merged:

1. **Hub**: Case marked as resolved with changelog
2. **Email**: "Your issue has been fixed in v0.3.2"
3. **Twitter**: Optional mention if handle provided
4. **CLI**: Next run shows notification

```
┌────────────────────────────────────────────────────┐
│ 🎉 CARO Self-Healing Update                        │
│                                                    │
│ Your reported issue (CARO-2024-001) has been      │
│ fixed in version 0.3.2!                           │
│                                                    │
│ The fix improves log compression command          │
│ generation. Thank you for helping improve CARO!   │
│                                                    │
│ Details: https://hub.caro.dev/cases/2024-001      │
└────────────────────────────────────────────────────┘
```

---

## Social Healing (Public Transparency)

### CARO Hub Integration

Each healing case is published to the Hub:

```
https://hub.caro.dev/cases/2024-001

CARO-2024-001: Log compression command generation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Status: ✅ Resolved (v0.3.2)
Reported: 2024-03-15
Resolved: 2024-03-18

What failed:
  CARO couldn't generate safe command for compressing
  old log files on macOS with zsh.

Why it failed:
  Missing find + gzip pattern for date-based file selection.

How we fixed it:
  Added safe pattern for find -mtime with gzip compression.
  PR #234: https://github.com/caro-cli/caro/pull/234

Impact:
  Affects: macOS, zsh users
  Fixed in: v0.3.2
```

### Benefits

1. **Transparency**: Users see exactly how failures become fixes
2. **Learning**: Community learns from failure patterns
3. **Trust**: Public accountability for improvement
4. **Contribution**: Encourages more failure reports

---

## Privacy & Security

### Data Minimization

- Collect only what's necessary for diagnosis
- Hash sensitive paths before transmission
- Redact PII from request text
- No file content ever collected

### Consent Levels

```rust
pub enum ConsentLevel {
    Minimal,    // Platform + error only
    Standard,   // + Sanitized request context
    Full,       // + Optional debug data
}
```

### Data Retention

- Reports retained for 90 days after resolution
- Contact info deleted after notification
- Aggregated statistics retained indefinitely

### Security Measures

- TLS for all transmissions
- Signed reports with local key
- Rate limiting on submissions
- Abuse detection for spam reports

---

## CLI Integration

### New Flags

```
--no-healing          Disable self-healing prompts
--healing-consent     Pre-approve diagnostic sharing
--doctor              Run CARO Doctor standalone
--doctor-report       Generate diagnostic report without submitting
```

### Configuration

```toml
# ~/.config/cmdai/config.toml

[healing]
enabled = true
consent_level = "standard"
contact_email = "user@example.com"
show_notifications = true
```

### Environment Variables

```bash
CARO_HEALING_ENABLED=true
CARO_HEALING_CONSENT=standard
CARO_CONTACT_EMAIL=user@example.com
```

---

## Metrics & Success Criteria

### Key Metrics

| Metric | Target |
|--------|--------|
| Opt-in rate | >30% of failures |
| Time to issue | <1 hour |
| Time to PR | <24 hours |
| PR acceptance rate | >70% |
| User notification success | >95% |

### Success Definition

1. **Measurable improvement**: Failure rate decreases over time
2. **Community engagement**: Users actively report and track issues
3. **Trust building**: Positive feedback on transparency
4. **Velocity**: Faster issue resolution than traditional reporting

---

## Implementation Phases

### Phase 1: CARO Doctor (MVP)
- Diagnostic collection
- Local report generation
- `--doctor` CLI command
- Basic consent flow

### Phase 2: Submission Pipeline
- Backend ingestion service
- GitHub issue creation
- Case tracking on Hub
- Email notifications

### Phase 3: PR Generation
- Agent-assisted fix generation
- Automated PR creation
- CI integration

### Phase 4: Social Healing
- Hub case pages
- Twitter integration
- CLI notification system
- Community dashboards

---

## Related Features

- **Dogma Rule Engine**: Self-healing can generate Dogma rule proposals
- **Hub Integration**: Cases published to community hub
- **Telemetry**: Aggregated healing metrics (opt-in)

---

## Open Questions

1. **Abuse prevention**: How to handle spam or malicious reports?
2. **Enterprise**: Should enterprise users have private healing pipelines?
3. **Prioritization**: How to prioritize cases when volume is high?
4. **Attribution**: How to credit users who report issues that get fixed?

---

## References

- [CARO Constitution](.specify/memory/constitution.md)
- [Safety Validation Pipeline](../command-validation-pipeline.md)
- [Hub Architecture](TBD)
