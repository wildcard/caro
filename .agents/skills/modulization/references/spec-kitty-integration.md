# Spec-Kitty Integration Guide

This document details how Modulization integrates with the Spec-Kitty spec-driven development workflow.

---

## Overview

Modulization acts as an **upstream feeder** to Spec-Kitty. It discovers and composes work, then produces **spec-ready inputs** that can flow directly into the spec-driven development pipeline.

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Modulization   │────▶│   Spec Seeds    │────▶│  /spec-kitty.   │
│  (Discovery &   │     │   (Ready for    │     │   specify       │
│   Composition)  │     │    speccing)    │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                         │
                                                         ▼
                              ┌─────────────────────────────────────┐
                              │  Full Spec-Kitty Workflow           │
                              │  clarify → plan → tasks → implement │
                              └─────────────────────────────────────┘
```

---

## Spec Seed Format

When Modulization classifies a module as `schedule` or `integrate_now`, it generates a **spec seed** — a structured input for `/spec-kitty.specify`.

### Spec Seed Structure

```yaml
# .modulization/spec-seeds/MOD-2026-007.yaml
spec_seed:
  # === Identity ===
  module_id: "MOD-2026-007"
  generated_at: "2026-01-08T09:00:00Z"
  generator: "modulization/1.0.0"

  # === Core Content ===
  title: "Complete Safety Validation Pattern Coverage"

  # Problem statement (feeds spec problem section)
  problem: |
    Safety validation patterns have incomplete coverage for edge cases
    including nested quotes, timeout commands, and unicode escapes. This
    creates potential security bypass vectors.

  # Background context (feeds spec context section)
  context: |
    Work on safety patterns has been ongoing since v1.0.0. The static
    matcher achieves 86% pass rate but known edge cases remain.

    Related work:
    - PR #156 (draft): Additional safety patterns - stalled waiting for review
    - Issue #134: Unicode escape bypass - reported by security researcher
    - TODOs in src/safety/patterns.rs at lines 142, 189

    Previous decisions:
    - 2025-11-05: Confirmed issue, committed to fix in v1.1.0
    - 2025-12-05: Review requested additional test coverage

  # === Requirements Hints ===
  # These help the specify phase ask the right questions
  user_scenarios:
    - "As a user, I want nested quotes in commands to be safely handled"
    - "As a security user, I want unicode escapes to not bypass validation"
    - "As a developer, I want timeout commands to be pattern-matched"

  constraints:
    - "Must maintain backwards compatibility with existing safe patterns"
    - "Must not degrade performance (validation < 5ms)"
    - "Must have comprehensive test coverage for all edge cases"
    - "Must align with constitution's safety-first principles"

  acceptance_hints:
    - "All TODOs in safety/ directory resolved"
    - "PR #156 merged or superseded"
    - "Issue #134 closed with fix"
    - "Beta test pass rate maintains or improves"

  # === Technical Context ===
  affected_areas:
    - "Core CLI"
    - "Safety"

  affected_files:
    - "src/safety/patterns.rs"
    - "src/safety/validator.rs"
    - "src/safety/mod.rs"
    - "tests/safety_tests.rs"

  related_code:
    - file: "src/safety/patterns.rs"
      line: 142
      snippet: |
        // TODO: Handle nested quotes in command arguments
        fn validate_quotes(&self, input: &str) -> Result<()> {
            if input.contains('"') {
                return Err(QuoteError::Nested);
            }
            Ok(())
        }

    - file: "src/safety/validator.rs"
      line: 89
      snippet: |
        // TODO: Add timeout command patterns
        // timeout, gtimeout, timelimit

  # === Suggested Artifacts ===
  suggested_artifacts:
    - type: "adr"
      rationale: "Document the pattern matching architecture decisions"
      title: "ADR: Safety Pattern Architecture"

    - type: "design"
      rationale: "Detail the edge case handling approach"
      title: "Design: Pattern Testing Strategy"

  # === Open Questions ===
  # Questions for the clarify phase
  open_questions:
    - "Should nested quotes be escaped or rejected?"
    - "What timeout values should trigger validation?"
    - "How should unicode normalization be handled?"

  # === Source Signals ===
  source_signals:
    - "CS-2026-001"  # TODO at patterns.rs:142
    - "CS-2026-002"  # TODO at validator.rs:89
    - "GH-2026-001"  # PR #156
    - "GH-2026-002"  # Issue #134

  # === Module Classification ===
  classification:
    decision: "integrate_now"
    confidence: 0.89
    target_milestone: "v1.1.0-beta"
    priority: "P1"
    rationale: |
      Security-relevant work that aligns with the v1.1.0-beta focus on
      safety improvements. The PR is 80% ready and the TODOs are scoped.
      Low marginal cost to complete, high leverage for security posture.
```

---

## Workflow Integration

### Step 1: Module → Spec Seed

When Modulization runs, it produces spec seeds for modules that need specification:

```bash
# Spec seeds are saved to:
.modulization/spec-seeds/
├── MOD-2026-001.yaml
├── MOD-2026-002.yaml
└── MOD-2026-005.yaml
```

### Step 2: Spec Seed → Feature Worktree

A spec seed can be used to initialize a new feature worktree:

```bash
# Create worktree from spec seed
/caro.feature init --from-seed .modulization/spec-seeds/MOD-2026-001.yaml

# This creates:
.worktrees/
└── 010-safety-pattern-coverage/
    ├── spec.md          # Pre-populated from seed
    ├── plan.md          # Empty, awaiting /spec-kitty.plan
    ├── tasks.md         # Empty, awaiting /spec-kitty.tasks
    └── .seed-origin     # Link back to module
```

### Step 3: Spec Seed → /spec-kitty.specify

The seed feeds directly into the specify phase:

```
User: /spec-kitty.specify

Spec-Kitty: I see a spec seed for MOD-2026-001: "Complete Safety Validation
Pattern Coverage"

The seed provides:
- Problem statement ✓
- Context ✓
- 3 user scenarios (hints)
- 4 constraints
- 5 affected files
- 3 open questions

I'll use this to draft the spec. Would you like me to:
1. Generate spec directly from seed (fast)
2. Ask clarification questions first (thorough)
3. Review the seed content before proceeding
```

### Step 4: Clarification with Seed Context

The `/spec-kitty.clarify` phase uses seed context:

```
Spec-Kitty: Based on the spec seed's open questions:

1. The seed asks: "Should nested quotes be escaped or rejected?"
   - Looking at the code context, the current behavior rejects.
   - The PR #156 adds escaping support.
   → Which approach do you prefer?

2. The seed asks: "What timeout values should trigger validation?"
   - No current handling exists.
   → Should all timeout commands be flagged, or only those below a threshold?

3. The seed asks: "How should unicode normalization be handled?"
   - Issue #134 reported a bypass using unicode.
   → Should we normalize to ASCII or block non-ASCII entirely?
```

---

## Spec Template Mapping

Spec seeds map to the Spec-Kitty template structure:

### spec-template.md Sections

| Template Section | Seed Field | Notes |
|-----------------|------------|-------|
| Feature Name | `title` | Direct mapping |
| Problem Statement | `problem` | Direct mapping |
| Context | `context` | Includes historical trace |
| User Scenarios | `user_scenarios` | As hints, not final |
| Requirements | `constraints` + `acceptance_hints` | Combined |
| Technical Scope | `affected_areas` + `affected_files` | Pre-populated |
| Open Questions | `open_questions` | For clarify phase |

### plan-template.md Sections

| Template Section | Seed Field | Notes |
|-----------------|------------|-------|
| Architecture Context | `related_code` | Code snippets |
| Key Decisions | `suggested_artifacts[type=adr]` | ADR hints |
| Implementation Notes | `affected_files` | File scope |
| Risk Assessment | `classification.rationale` | From module |

---

## Automatic Spec Creation

For high-confidence modules, Modulization can create draft specs automatically:

### Configuration

```yaml
# .modulization/config.yaml
integration:
  spec_kitty:
    enabled: true

    # Auto-create specs for high-confidence modules
    auto_spec:
      enabled: true
      confidence_threshold: 0.85
      classifications:
        - integrate_now
        - schedule

    # Create worktrees automatically
    auto_worktree:
      enabled: false  # Requires manual trigger

    # Link back to modules
    track_origin:
      enabled: true
      origin_file: ".seed-origin"
```

### Auto-Spec Workflow

```
1. Modulization runs on cadence
2. Module MOD-2026-007 classified as integrate_now (confidence: 0.89)
3. Auto-spec enabled, threshold met
4. Draft spec created at .modulization/draft-specs/MOD-2026-007/spec.md
5. Notification: "Draft spec ready for review"
6. Developer reviews and runs /spec-kitty.clarify
```

---

## Tracking Module → Spec → Implementation

Modulization maintains traceability through the entire lifecycle:

### Origin Tracking

```yaml
# .worktrees/010-safety-patterns/.seed-origin
origin:
  module_id: "MOD-2026-007"
  module_title: "Complete Safety Validation Pattern Coverage"
  seed_path: ".modulization/spec-seeds/MOD-2026-007.yaml"
  created_from_seed_at: "2026-01-08T10:00:00Z"
  modulization_run: "weekly-2026-01-08"

  original_signals:
    - "CS-2026-001"  # TODO at patterns.rs:142
    - "GH-2026-001"  # PR #156

  target_milestone: "v1.1.0-beta"
```

### Status Synchronization

When spec work progresses, Modulization can update module status:

```yaml
# .modulization/modules.yaml
modules:
  - id: "MOD-2026-007"
    status: "in_progress"
    spec_status: "specified"
    worktree: "010-safety-patterns"
    last_updated: "2026-01-09T14:00:00Z"

    progress:
      - event: "spec_seed_created"
        timestamp: "2026-01-08T09:00:00Z"

      - event: "worktree_created"
        timestamp: "2026-01-08T10:00:00Z"
        worktree: "010-safety-patterns"

      - event: "spec_drafted"
        timestamp: "2026-01-08T10:30:00Z"

      - event: "spec_clarified"
        timestamp: "2026-01-08T11:00:00Z"

      - event: "plan_complete"
        timestamp: "2026-01-09T09:00:00Z"

      - event: "tasks_generated"
        timestamp: "2026-01-09T14:00:00Z"
        task_count: 8
```

---

## Module Lifecycle States

Modules flow through states as they progress:

```
┌────────────┐     ┌────────────┐     ┌────────────┐
│ discovered │────▶│ classified │────▶│ seed_ready │
└────────────┘     └────────────┘     └────────────┘
                                             │
                         ┌───────────────────┘
                         ▼
┌────────────┐     ┌────────────┐     ┌────────────┐
│ in_progress│◀────│  planned   │◀────│  specified │
└────────────┘     └────────────┘     └────────────┘
       │
       ▼
┌────────────┐     ┌────────────┐
│ implemented│────▶│  resolved  │
└────────────┘     └────────────┘
```

### State Definitions

| State | Meaning | Trigger |
|-------|---------|---------|
| `discovered` | Signals found, not yet grouped | Signal collection |
| `classified` | Grouped into module, classification assigned | Module composition |
| `seed_ready` | Spec seed generated | Post-classification |
| `specified` | Spec created from seed | `/spec-kitty.specify` |
| `planned` | Plan created | `/spec-kitty.plan` |
| `in_progress` | Tasks being worked | `/spec-kitty.implement` |
| `implemented` | Code complete, pending review | `/spec-kitty.review` |
| `resolved` | Merged and closed | `/spec-kitty.merge` |

---

## Integration Commands

### From Modulization to Spec-Kitty

```bash
# View available spec seeds
/caro.modulization list-seeds

# Create feature from seed
/caro.feature init --from-seed MOD-2026-007

# Create spec from seed
/spec-kitty.specify --seed MOD-2026-007

# View module-to-spec mapping
/caro.modulization trace MOD-2026-007
```

### From Spec-Kitty to Modulization

```bash
# Mark module as resolved when spec complete
/caro.modulization resolve MOD-2026-007

# Link existing spec to module
/caro.modulization link --module MOD-2026-007 --worktree 010-safety-patterns

# Check for new work from modules
/caro.modulization pending-specs
```

---

## Report Integration

Modulization reports include spec integration status:

```markdown
## Modulization Report - 2026-01-08

### Spec-Kitty Integration

| Module | Classification | Spec Status | Worktree |
|--------|---------------|-------------|----------|
| MOD-2026-007 | integrate_now | specified | 010-safety-patterns |
| MOD-2026-003 | schedule | seed_ready | - |
| MOD-2026-008 | schedule | seed_ready | - |
| MOD-2026-002 | ice | - | - |

### Pending Spec Seeds

3 modules have spec seeds ready for `/spec-kitty.specify`:
- MOD-2026-003: CLI help improvements
- MOD-2026-008: Backend retry logic
- MOD-2026-011: Documentation sync

### Spec Progress Summary

| Phase | Count |
|-------|-------|
| Seed Ready | 3 |
| Specified | 1 |
| Planned | 0 |
| In Progress | 0 |
| Resolved | 0 |
```

---

## Best Practices

### DO:
- Review spec seeds before creating worktrees
- Use clarify phase to validate seed assumptions
- Keep module → spec links for traceability
- Run modulization before major planning sessions
- Let seeds inform the specify phase, don't skip it

### DON'T:
- Auto-create worktrees without review
- Skip clarification for seed-based specs
- Ignore open questions from seeds
- Break module → spec links
- Treat seeds as final specs

---

## Configuration Reference

```yaml
# .modulization/config.yaml
integration:
  spec_kitty:
    enabled: true

    # Spec seed generation
    seed_generation:
      enabled: true
      output_path: ".modulization/spec-seeds/"
      format: yaml

    # Auto-spec creation
    auto_spec:
      enabled: true
      confidence_threshold: 0.85
      classifications:
        - integrate_now
        - schedule

    # Worktree integration
    worktree:
      auto_create: false
      naming_pattern: "{sequence}-{slug}"
      origin_tracking: true

    # Template customization
    templates:
      spec_template: ".kittify/missions/software-dev/templates/spec-template.md"
      seed_to_spec_mapping: default

    # Progress tracking
    tracking:
      enabled: true
      sync_on_phase_complete: true
      update_module_status: true
```
