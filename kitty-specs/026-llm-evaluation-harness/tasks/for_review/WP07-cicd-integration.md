---
work_package_id: "WP07"
subtasks: ["T053", "T054", "T055", "T056", "T057", "T058", "T059", "T060"]
title: "CI/CD Integration & Automation"
phase: "Phase 3 - Automation"
lane: "for_review"
agent: "claude"
shell_pid: "72431"
history:
  - timestamp: "2026-01-09T11:00:00Z"
    lane: "planned"
    agent: "system"
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP07 – CI/CD Integration & Automation

## Objectives & Success Criteria

**Goal**: Automate evaluation in GitHub Actions with matrix strategy and baseline management.

**Success Criteria**:
- Workflow runs on every PR
- Detects regressions vs main branch baseline
- Updates baseline on main branch merges
- Stores results as GitHub Actions artifacts
- Handles platform-specific backends (MLX macOS-only)

## Context & Constraints

**Dependencies**: WP06 (cargo test CLI)
**Platform**: GitHub Actions (ubuntu-latest, macos-latest)

## Key Subtasks

### T053-T054 – Workflow & Matrix Strategy (`.github/workflows/evaluation.yml`)
```yaml
strategy:
  matrix:
    backend: [static_matcher, mlx, ollama, vllm]
runs-on: ${{ matrix.backend == 'mlx' && 'macos-latest' || 'ubuntu-latest' }}
```

### T055-T056 – Baseline Management
- PR: Load main-latest.json, compare, fail if regression >5%
- Main merge: Run evaluation, store new baseline with timestamp

### T057-T058 – Artifacts & PR Comments
Upload JSON results. Post comment with pass rates and regression summary.

### T059-T060 – Platform Handling & Testing
Skip MLX on non-macOS gracefully. Test with sample PR to verify all steps.

## Test Strategy

Test workflow in feature branch before merging.

## Definition of Done

- [x] Workflow file created and functional
- [x] Matrix strategy runs backends in parallel
- [x] Baseline comparison works in PRs
- [x] Baseline updates on main merges
- [x] Artifacts uploaded successfully
- [x] Platform detection handles MLX correctly
- [x] Test run validates entire workflow

## Activity Log

- 2026-01-09T11:00:00Z – system – lane=planned – Prompt created
- 2026-01-09T10:59:28Z – claude – shell_pid=71643 – lane=doing – Starting CI/CD integration with GitHub Actions
- 2026-01-09T11:01:30Z – claude – shell_pid=72431 – lane=for_review – Completed CI/CD integration - GitHub Actions workflow with matrix strategy, baseline comparison, and PR comments
