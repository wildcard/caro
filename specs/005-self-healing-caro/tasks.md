# Tasks: Self-Healing CARO

**Feature**: 005-self-healing-caro
**Generated**: 2024-03-20
**Total Tasks**: 24
**Estimated Complexity**: Medium

## Task Legend

- `[P]` - Can be parallelized with other `[P]` tasks in same group
- `[S]` - Sequential, depends on previous tasks
- `[T]` - Test task (write test first)
- `[I]` - Implementation task (make test pass)

---

## Group 1: Data Models [P]

Foundation types for diagnostic reports and healing workflow.

### Task 1.1: DiagnosticReport struct [T]
**File**: `src/doctor/report.rs`
**Contract**: `tests/contract/diagnostic_report_test.rs`

```
- [ ] Create DiagnosticReport struct with fields:
      id (Uuid), timestamp, caro_version, platform, shell, backend,
      request (SanitizedRequest), failure (FailureInfo), contact (Option)
- [ ] Implement Serialize/Deserialize
- [ ] Write contract test: serializes to valid JSON
- [ ] Write contract test: deserializes from JSON
- [ ] Write contract test: all required fields present
```

### Task 1.2: SanitizedRequest struct [T][P]
**File**: `src/doctor/report.rs`

```
- [ ] Create SanitizedRequest struct:
      intent_category (IntentCategory enum), sanitized_text, complexity_score
- [ ] Create IntentCategory enum: FileOps, Network, System, Package, Git, Other
- [ ] Write test: intent_category derived from keywords
- [ ] Write test: complexity_score in range 1-10
```

### Task 1.3: FailureInfo struct [T][P]
**File**: `src/doctor/report.rs`

```
- [ ] Create FailureInfo struct:
      failure_type (FailureType), stage (PipelineStage),
      error_code (Option), safety_patterns_triggered (Vec<String>)
- [ ] Create FailureType enum: Generation, Validation, Execution, Timeout, Unknown
- [ ] Create PipelineStage enum: Init, Backend, Safety, Execution
- [ ] Write test: failure captures stage correctly
```

### Task 1.4: ContactPreference struct [T][P]
**File**: `src/healing/contact.rs`

```
- [ ] Create ContactMethod enum: Email, Twitter, GitHub, None
- [ ] Create ContactPreference struct: method, value
- [ ] Write test: email format validation
- [ ] Write test: twitter handle validation (starts with @)
- [ ] Write test: github username validation
```

### Task 1.5: ConsentLevel enum [T][P]
**File**: `src/healing/consent.rs`

```
- [ ] Create ConsentLevel enum: Minimal, Standard, Full
- [ ] Implement Default (Standard)
- [ ] Write test: consent levels have correct data inclusion
```

---

## Group 2: Platform Collection [P]

Collect diagnostic information from the system.

### Task 2.1: PlatformInfo collector [T]
**File**: `src/doctor/collector.rs`
**Dependency**: Add `os_info = "3"` to Cargo.toml

```
- [ ] Create PlatformInfo struct: os_type, os_version, arch
- [ ] Implement PlatformInfo::collect() using os_info crate
- [ ] Write test: collects valid OS type (macOS, Linux, Windows)
- [ ] Write test: collects valid architecture (arm64, x86_64)
```

### Task 2.2: ShellInfo collector [T][P]
**File**: `src/doctor/collector.rs`

```
- [ ] Create ShellInfo struct: shell_type, version, path
- [ ] Implement ShellInfo::collect() reusing platform::detect_shell()
- [ ] Write test: detects current shell
- [ ] Write test: shell version is optional (may fail)
```

### Task 2.3: BackendInfo collector [T][P]
**File**: `src/doctor/collector.rs`

```
- [ ] Create BackendInfo struct: backend_type, model_name, available
- [ ] Implement BackendInfo::collect() from current config
- [ ] Write test: captures active backend type
- [ ] Write test: model_name is optional
```

### Task 2.4: Unified DiagnosticCollector [I][S]
**File**: `src/doctor/collector.rs`
**Depends**: 2.1, 2.2, 2.3

```
- [ ] Create DiagnosticCollector struct
- [ ] Implement collect_all() -> DiagnosticReport
- [ ] Integrate with existing config and backend modules
- [ ] Write integration test: full collection under 100ms
```

---

## Group 3: PII Sanitization [S]

Redact personally identifiable information.

### Task 3.1: Sanitizer patterns [T]
**File**: `src/doctor/sanitizer.rs`

```
- [ ] Create EMAIL_REGEX pattern
- [ ] Create HOME_PATH_REGEX pattern (/Users/xxx/, /home/xxx/)
- [ ] Create IP_REGEX pattern
- [ ] Create API_KEY_REGEX pattern (sk-, pk-, api_key, etc.)
- [ ] Write test: each pattern matches expected strings
- [ ] Write test: each pattern does NOT match safe strings
```

### Task 3.2: Sanitizer implementation [I][S]
**File**: `src/doctor/sanitizer.rs`
**Depends**: 3.1

```
- [ ] Create Sanitizer struct with pattern list
- [ ] Implement sanitize(input: &str) -> String
- [ ] Write test: email replaced with [EMAIL]
- [ ] Write test: home path replaced with [HOME]/
- [ ] Write test: chained replacements work
- [ ] Write test: preserves non-PII content
```

### Task 3.3: SanitizedRequest builder [I][S]
**File**: `src/doctor/sanitizer.rs`
**Depends**: 3.2

```
- [ ] Implement SanitizedRequest::from_raw(user_input: &str)
- [ ] Classify intent_category from keywords
- [ ] Calculate complexity_score based on length/operators
- [ ] Write test: file operations detected (rm, mv, cp)
- [ ] Write test: network operations detected (curl, wget)
```

---

## Group 4: Consent Flow [S]

User interaction for consent and contact preferences.

### Task 4.1: Consent prompt [T]
**File**: `src/healing/consent.rs`

```
- [ ] Create ConsentChoice enum: Yes, No, Details, Remember(bool)
- [ ] Create prompt_consent() function signature
- [ ] Write test: returns No by default (no interaction)
- [ ] Write test: remembers preference in config
```

### Task 4.2: Consent implementation [I][S]
**File**: `src/healing/consent.rs`
**Depends**: 4.1

```
- [ ] Implement prompt_consent() using dialoguer
- [ ] Display: [Y] Yes  [N] No  [D] Details  [A] Always  [V] Never
- [ ] Handle Details: show report preview
- [ ] Save preference for Always/Never
- [ ] Write integration test: consent flow works in terminal
```

### Task 4.3: Contact prompt [I][S]
**File**: `src/healing/contact.rs`
**Depends**: 4.2

```
- [ ] Implement prompt_contact() using dialoguer
- [ ] Display: [1] Email  [2] Twitter  [3] GitHub  [4] Skip
- [ ] Validate input format for each type
- [ ] Return ContactPreference
- [ ] Write test: validation rejects invalid email
```

---

## Group 5: Report Submission [S]

HTTP submission with retry and offline queue.

### Task 5.1: Submit function [T]
**File**: `src/healing/submit.rs`

```
- [ ] Create submit_report() function signature
- [ ] Create HealingError enum: Network, Queued, ServerError
- [ ] Create CaseId type (String wrapper)
- [ ] Write test: returns CaseId on success (mock server)
- [ ] Write test: returns HealingError on failure
```

### Task 5.2: Retry logic [I][S]
**File**: `src/healing/submit.rs`
**Depends**: 5.1

```
- [ ] Implement exponential backoff: 1s, 2s, 4s, 8s, 16s
- [ ] Add jitter (0-500ms random)
- [ ] Max 5 retries before queuing
- [ ] Write test: retries on 503
- [ ] Write test: no retry on 400
```

### Task 5.3: Offline queue [I][S]
**File**: `src/healing/queue.rs`
**Depends**: 5.2

```
- [ ] Create queue directory: ~/.config/cmdai/healing/queue/
- [ ] Implement queue_report() - save as JSON file
- [ ] Implement flush_queue() - send all pending reports
- [ ] Implement cleanup() - remove sent reports
- [ ] Write test: report survives restart
- [ ] Write test: flush sends all pending
```

---

## Group 6: Notifications [S]

Store and display notifications on CLI startup.

### Task 6.1: Notification storage [T]
**File**: `src/healing/notification.rs`

```
- [ ] Create Notification struct: case_id, message, created_at, shown, dismissed
- [ ] Create NotificationStore struct with Vec<Notification>
- [ ] Implement save() and load() for ~/.config/cmdai/healing/notifications.json
- [ ] Write test: persists across process restart
```

### Task 6.2: Notification display [I][S]
**File**: `src/healing/notification.rs`
**Depends**: 6.1

```
- [ ] Implement pending() - filter unshown notifications
- [ ] Implement mark_shown() - update shown flag
- [ ] Implement display_pending() - format for terminal
- [ ] Implement expire_old() - remove >30 day notifications
- [ ] Write test: displays unshown only
- [ ] Write test: marks as shown after display
```

### Task 6.3: Startup hook [I][S]
**File**: `src/cli/mod.rs`
**Depends**: 6.2

```
- [ ] Add check_notifications() call on CLI startup
- [ ] Display pending notifications before main output
- [ ] Respect --quiet flag (suppress notifications)
- [ ] Write test: notifications shown before command output
```

---

## Group 7: CLI Integration [S]

Add flags and orchestrate healing workflow.

### Task 7.1: Doctor CLI flag [T]
**File**: `src/main.rs`

```
- [ ] Add --doctor flag to clap args
- [ ] Add --doctor-report flag (generate without submit)
- [ ] Write test: --doctor runs diagnostics
- [ ] Write test: --doctor-report outputs JSON
```

### Task 7.2: Doctor implementation [I][S]
**File**: `src/cli/mod.rs`
**Depends**: Group 2, 7.1

```
- [ ] Implement run_doctor() function
- [ ] Collect all diagnostics
- [ ] Display formatted output
- [ ] Support JSON output with --output json
- [ ] Write integration test: doctor completes under 1s
```

### Task 7.3: Healing CLI flags [T][P]
**File**: `src/main.rs`

```
- [ ] Add --no-healing flag (disable prompts)
- [ ] Add --healing-consent flag (pre-approve)
- [ ] Read CARO_HEALING_ENABLED env var
- [ ] Write test: --no-healing suppresses consent prompt
```

### Task 7.4: Failure hook [I][S]
**File**: `src/cli/mod.rs`
**Depends**: All previous groups

```
- [ ] Hook into generation failure path
- [ ] Check if healing enabled
- [ ] Run consent flow
- [ ] Collect diagnostics
- [ ] Submit or queue report
- [ ] Write integration test: full healing flow on failure
```

---

## Group 8: Configuration [P]

Config file support for healing preferences.

### Task 8.1: Healing config section [T]
**File**: `src/config/mod.rs`

```
- [ ] Add [healing] section to config schema
- [ ] Fields: enabled, consent_level, contact_email, show_notifications
- [ ] Write test: parses healing config
- [ ] Write test: defaults when section missing
```

### Task 8.2: Config integration [I][S]
**File**: `src/config/mod.rs`
**Depends**: 8.1

```
- [ ] Load healing config on startup
- [ ] Override with CLI flags
- [ ] Override with env vars
- [ ] Write test: CLI flag overrides config
- [ ] Write test: env var overrides config
```

---

## Verification Tasks

### Task V1: Contract test suite [S]
**Depends**: Groups 1-3

```
- [ ] Run all contract tests: cargo test --test contract
- [ ] Verify 100% pass rate
- [ ] Check coverage for data models
```

### Task V2: Integration test suite [S]
**Depends**: Groups 4-7

```
- [ ] Run integration tests: cargo test --test integration
- [ ] Test full healing workflow end-to-end
- [ ] Test offline queue functionality
```

### Task V3: Performance validation [S]
**Depends**: All groups

```
- [ ] Benchmark: doctor collection < 100ms
- [ ] Benchmark: report submission < 2s
- [ ] Benchmark: notification check < 50ms
```

---

## Summary

| Group | Tasks | Parallelizable | Dependencies |
|-------|-------|----------------|--------------|
| 1. Data Models | 5 | Yes | None |
| 2. Platform Collection | 4 | Partial | None |
| 3. PII Sanitization | 3 | No | Group 1 |
| 4. Consent Flow | 3 | No | Group 3 |
| 5. Report Submission | 3 | No | Group 4 |
| 6. Notifications | 3 | No | Group 1 |
| 7. CLI Integration | 4 | Partial | Groups 2-6 |
| 8. Configuration | 2 | Yes | None |
| Verification | 3 | No | All |

**Critical Path**: Groups 1 → 3 → 4 → 5 → 7

**MVP Subset** (for initial release):
- Group 1: All tasks
- Group 2: All tasks
- Group 3: All tasks
- Group 7: Tasks 7.1, 7.2 only
- Task V1

---

*Tasks generated from plan.md and research.md*
