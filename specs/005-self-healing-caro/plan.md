# Implementation Plan: Self-Healing CARO

**Branch**: `005-self-healing-caro` | **Date**: 2024-03-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-self-healing-caro/spec.md`

## Summary

Self-Healing transforms CARO failures into systematic improvements through a closed-loop pipeline: CARO Doctor collects diagnostics, users opt-in to share anonymized reports, an analysis service creates GitHub issues/PRs, and users are notified when their reported issues are resolved.

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**:
- `serde` + `serde_json` - Report serialization
- `reqwest` - HTTP submission to healing service
- `uuid` - Report identification
- `chrono` - Timestamps
- `dialoguer` - Interactive consent prompts
- `dirs` - Cross-platform config directories

**Storage**:
- Local: `~/.config/cmdai/healing/` for pending notifications
- Remote: Healing service backend (separate deployment)

**Testing**: `cargo test` with mock HTTP responses

**Target Platform**: macOS (arm64, x86_64), Linux (x86_64), Windows (x86_64)

**Project Type**: Single project (library + CLI)

**Performance Goals**:
- Doctor collection: <100ms
- Report submission: <2s (network dependent)
- Notification check: <50ms on startup

**Constraints**:
- No PII in reports by default
- Offline-capable (queue reports for later)
- Minimal binary size impact (<500KB)

**Scale/Scope**:
- Initial: 100 reports/day
- Target: 10,000 reports/day

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **Simplicity** | PASS | Direct implementation, no frameworks |
| **Library-First** | PASS | Doctor module exported via lib.rs |
| **Test-First** | PASS | Contract tests for report format |
| **Safety-First** | PASS | PII redaction, consent flow |
| **Observability** | PASS | Structured logging for diagnostics |

## Project Structure

### Documentation (this feature)
```
specs/005-self-healing-caro/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── diagnostic-report.rs
│   ├── consent-flow.rs
│   └── notification.rs
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)
```
src/
├── doctor/
│   ├── mod.rs           # Public API: run_diagnostics()
│   ├── collector.rs     # Platform, shell, backend info
│   ├── sanitizer.rs     # PII redaction
│   └── report.rs        # DiagnosticReport struct
│
├── healing/
│   ├── mod.rs           # Public API: submit_report(), check_notifications()
│   ├── consent.rs       # User consent workflow
│   ├── contact.rs       # Contact preference management
│   ├── submit.rs        # HTTP submission with retry
│   ├── notification.rs  # Local notification storage
│   └── queue.rs         # Offline queue management
│
└── cli/
    └── mod.rs           # Add --doctor, --no-healing flags

tests/
├── contract/
│   ├── diagnostic_report_test.rs
│   └── consent_flow_test.rs
├── integration/
│   └── healing_pipeline_test.rs
└── unit/
    ├── collector_test.rs
    ├── sanitizer_test.rs
    └── queue_test.rs
```

**Structure Decision**: Single project with two new modules (`doctor/`, `healing/`) following existing patterns from `safety/` and `config/`.

## Phase 0: Outline & Research

### Research Tasks

1. **PII Detection & Redaction**
   - Research regex patterns for email, paths, usernames
   - Evaluate existing crates: `scrub`, `redact`
   - Decision needed: Build custom vs use crate

2. **HTTP Client Patterns**
   - Research `reqwest` async patterns for submission
   - Retry strategies with exponential backoff
   - Offline queue persistence format

3. **Cross-Platform Diagnostics**
   - macOS: `sw_vers`, `uname`, shell detection
   - Linux: `/etc/os-release`, `uname`
   - Windows: `ver`, PowerShell detection

4. **Notification Storage**
   - Research local storage format (JSON files vs SQLite)
   - Startup notification check patterns
   - Expiration and cleanup strategies

5. **Consent UX Patterns**
   - Research CLI consent best practices
   - Preview what will be shared
   - Remember consent preference

### Research Agents to Dispatch

```
Task: "Research PII redaction patterns for CLI diagnostic reports"
Task: "Find best practices for offline-first HTTP submission in Rust"
Task: "Research cross-platform system info collection in Rust"
Task: "Find CLI consent flow patterns that respect user privacy"
```

**Output**: research.md with decisions on each unknown

## Phase 1: Design & Contracts

### Data Model Entities

1. **DiagnosticReport**
   - Fields: id, timestamp, caro_version, platform, shell, backend, request, failure, contact
   - Serialization: JSON
   - Validation: All required fields present, valid enum values

2. **SanitizedRequest**
   - Fields: intent_category, sanitized_text, complexity_score
   - Validation: No PII patterns in sanitized_text

3. **FailureInfo**
   - Fields: failure_type, stage, error_code, safety_patterns_triggered
   - Enum: FailureType (Generation, Validation, Execution, Timeout)

4. **ContactPreference**
   - Fields: method (Email/Twitter/GitHub/None), value
   - Validation: Email format, Twitter handle format

5. **ConsentLevel**
   - Enum: Minimal, Standard, Full
   - Storage: Config file preference

### Contract Tests

```rust
// contracts/diagnostic-report.rs
#[test]
fn diagnostic_report_serializes_to_json() {
    let report = DiagnosticReport::new(...);
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"id\":"));
    assert!(json.contains("\"platform\":"));
}

#[test]
fn sanitized_request_redacts_email() {
    let request = SanitizedRequest::from_raw("send to user@example.com");
    assert!(!request.sanitized_text.contains("@"));
}

// contracts/consent-flow.rs
#[test]
fn consent_flow_requires_explicit_approval() {
    let consent = ConsentFlow::new();
    assert!(!consent.is_approved());
    consent.approve(ConsentLevel::Standard);
    assert!(consent.is_approved());
}

// contracts/notification.rs
#[test]
fn notification_persists_across_sessions() {
    let notif = Notification::new("CARO-2024-001", "Your issue was fixed");
    notif.save_local().unwrap();
    let loaded = Notification::load_pending().unwrap();
    assert_eq!(loaded[0].case_id, "CARO-2024-001");
}
```

### Quickstart Scenarios

1. **User reports failure with email contact**
   - Run command that fails
   - Opt-in to self-healing
   - Provide email
   - Verify report submitted
   - Check for notification on next run

2. **User reports failure without contact**
   - Run command that fails
   - Opt-in with minimal consent
   - No contact info
   - Verify anonymous report submitted

3. **Offline report queuing**
   - Disable network
   - Opt-in to self-healing
   - Verify report queued locally
   - Re-enable network
   - Verify queue flushed on next run

## Phase 2: Task Planning Approach

**Task Generation Strategy**:
- Contract tests for DiagnosticReport, ConsentFlow, Notification
- Unit tests for Collector, Sanitizer, Queue
- Integration tests for full healing pipeline
- Implementation tasks following TDD order

**Ordering Strategy**:
1. Data models (DiagnosticReport, FailureInfo, etc.)
2. Collector (platform, shell, backend info)
3. Sanitizer (PII redaction)
4. Consent flow (user interaction)
5. Submission (HTTP with retry)
6. Notification (local storage, startup check)
7. CLI integration (--doctor, --no-healing)
8. Queue (offline support)

**Task Groups** (can run in parallel within groups):
- [P] Models: DiagnosticReport, FailureInfo, ContactPreference
- [P] Core: Collector, Sanitizer
- [S] Flow: Consent → Submit → Notify
- [P] CLI: --doctor flag, --no-healing flag

**Estimated Output**: 22-25 tasks in tasks.md

## Phase 3+: Future Implementation

**Phase 3**: Task execution via /tasks command
**Phase 4**: Implementation following TDD (test → implement → refactor)
**Phase 5**: Validation (run all tests, manual quickstart scenarios)

## Complexity Tracking

*No violations identified - design follows constitution principles*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

## Progress Tracking

**Phase Status**:
- [ ] Phase 0: Research complete
- [ ] Phase 1: Design complete
- [ ] Phase 2: Task planning complete
- [ ] Phase 3: Tasks generated
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [ ] Post-Design Constitution Check: PENDING
- [ ] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented

## Dependencies on Other Features

- **Dogma (006)**: Self-Healing can propose Dogma rules when failures indicate rule gaps
- **Hub Integration**: Case tracking and social healing require Hub backend
- **GitHub API**: Issue and PR creation require GitHub App authentication

## MVP Scope (Phase 1 Implementation)

For MVP, implement:
1. CARO Doctor (local diagnostics only)
2. Console output of diagnostic report
3. `--doctor` CLI flag
4. Basic report structure

Defer to Phase 2+:
- HTTP submission
- Contact preferences
- Notification system
- Offline queue

---
*Based on Constitution v1.0.0 - See `.specify/memory/constitution.md`*
