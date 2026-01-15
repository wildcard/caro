---
work_package_id: "WP06"
subtasks: ["T045", "T046", "T047", "T048", "T049", "T050", "T051", "T052", "T053"]
title: "Claude Skill Integration"
phase: "Phase 2 - Developer Tools"
lane: "done"
assignee: "claude"
agent: "claude"
shell_pid: "54050"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP06 – Claude Skill Integration

## Objectives & Success Criteria

**Goal**: Implement FR4 Claude skill for benchmark recommendations

Create .claude/skills/benchmark-advisor/SKILL.md with git diff analysis. Create mapping.toml for file-to-benchmark mappings (src/cache/**/*.rs -> cargo bench --bench cache). Generate specific recommendations with explanations.

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
- 2026-01-08T14:25:36Z – claude – shell_pid=51067 – lane=doing – Started Claude skill for intelligent benchmark recommendations
- 2026-01-08T14:29:13Z – claude – shell_pid=54050 – lane=for_review – Claude skill complete with SKILL.md and mapping.toml configuration
- 2026-01-08T15:39:46Z – claude – shell_pid=54050 – lane=done – Approved
