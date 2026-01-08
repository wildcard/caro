---
work_package_id: "WP05"
subtasks: ["T032", "T033", "T034", "T035", "T036", "T037", "T038", "T039", "T040", "T041", "T042", "T043", "T044"]
title: "CI Workflow & Regression Detection"
phase: "Phase 2 - CI Integration"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP05 – CI Workflow & Regression Detection

## Objectives & Success Criteria

**Goal**: Implement FR2 automated benchmarking and regression detection

Create .github/workflows/benchmarks.yml with baseline comparison. Implement scripts/benchmark-compare.py (parse Criterion JSON, check thresholds, generate PR comments). Implement scripts/benchmark-aggregate.py and benchmark-monthly-aggregate.py for historical data. Test locally with act.

**Related Documents**:
- Spec: kitty-specs/017-issue-9-add/spec.md
- Plan: kitty-specs/017-issue-9-add/plan.md
- Research: kitty-specs/017-issue-9-add/research.md
- Tasks: kitty-specs/017-issue-9-add/tasks.md

## Implementation Guidance

Refer to tasks.md for detailed subtask breakdown and dependencies.

Key implementation points:
- Follow patterns from research.md decisions
- Use data-model.md schemas for CI integration
- Reference contracts/ for CI workflow structure
- Validate against quickstart.md scenarios

## Definition of Done

- [ ] All subtasks from tasks.md completed
- [ ] Independent test criteria passes
- [ ] Documentation updated as needed
- [ ] Code formatted (cargo fmt) and linted (cargo clippy)
- [ ] tasks.md updated with completion status

## Activity Log

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created
