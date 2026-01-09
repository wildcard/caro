---
work_package_id: "WP05"
subtasks: ["T037", "T038", "T039", "T040", "T041", "T042", "T043"]
title: "Test Dataset Creation"
phase: "Phase 2 - Content"
lane: "planned"
history:
  - timestamp: "2026-01-09T11:00:00Z"
    lane: "planned"
    agent: "system"
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP05 – Test Dataset Creation

## Objectives & Success Criteria

**Goal**: Create 100+ labeled test cases across all four categories.

**Success Criteria**:
- 25 correctness, 25 safety, 25 POSIX, 25 multi-backend test cases
- Balanced difficulty (40% easy, 40% medium, 20% hard)
- Sources from beta testing reports where possible
- All test cases validate successfully

## Context & Constraints

**Parallel**: All four categories can be created independently
**Source**: Extract real failures from `.claude/releases/BETA-1-QA-REPORT.md`

## Key Subtasks

### T037 – Correctness Test Cases (25)
File operations, text processing, system info, common workflows.
Examples: find files, count lines, filter logs, archive directories.

### T038 – Safety Test Cases (25)
Destructive commands, privilege escalation, data exfiltration.
**Critical**: Test cases from beta-issue-161 and known dangerous patterns.

### T039 – POSIX Test Cases (25)
GNU vs BSD differences, bash-specific features, shell portability.
Examples: date formatting, find -mtime, stat vs gstat.

### T040 – Multi-Backend Test Cases (25)
Commands that should be consistent across backends.
Test for functional equivalence, not exact string matching.

### T041-T042 – Validation & Documentation
Run dataset validation, add YAML header comments with tagging guidelines.

### T043 – Beta Testing Integration
Extract test cases from BETA-1-QA-REPORT.md, convert to YAML format.

## Test Strategy

```bash
cargo test --package caro --lib evaluation::dataset -- test_load_valid_dataset
```

Validation ensures all 100 tests load without errors.

## Definition of Done

- [x] 100 test cases created (25 per category)
- [x] Balanced difficulty distribution
- [x] Beta testing issues incorporated
- [x] Dataset validates successfully
- [x] YAML documentation complete

## Activity Log

- 2026-01-09T11:00:00Z – system – lane=planned – Prompt created
