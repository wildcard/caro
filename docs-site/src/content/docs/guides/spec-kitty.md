---
title: Spec-Kitty Development Guide
description: Use Spec-Kitty for rapid, multi-branch feature development with caro
---

This guide explains how to use Spec-Kitty in the caro project for rapid, multi-branch feature development.

## Overview

**Spec-Kitty** is integrated into caro to enable:
- **Worktree-based development**: Work on multiple features simultaneously without branch switching
- **Real-time dashboard**: Visual kanban board showing all features and their status
- **Multi-agent coordination**: Collaborate with multiple AI agents (Claude Code, Codex, etc.)
- **Spec-driven workflows**: Systematic approach to features, enhancements, and bug fixes

## Quick Start

### 1. Create a New Feature

```bash
# From project root
bin/sk-new-feature "Add caching layer with TTL support"

# Or with a custom name
bin/sk-new-feature "Add caching layer" --name "Redis Cache Integration"
```

This creates:
- A new git worktree in `kitty-specs/001-add-caching/`
- A feature branch `feature/001-add-caching`
- Initial directory structure with `tasks/` folders

### 2. View All Features

```bash
# List all features and their status
bin/sk-list

# Open the real-time dashboard
bin/sk-dashboard
```

Dashboard URL: http://127.0.0.1:9237

### 3. Work on a Feature

```bash
# Navigate to the feature worktree
cd kitty-specs/001-add-caching/

# Now you're in an isolated git worktree
# Main repo is still on your original branch!
```

## Workflow Commands

Use these slash commands in Claude Code **from within the feature worktree**:

### Phase 1: Specification
```
/spec-kitty.specify
```
Creates `spec.md` with feature requirements, scope, and acceptance criteria.

### Phase 2: Planning (Optional Enhancement)
```
/spec-kitty.clarify
```
Asks targeted questions to de-risk ambiguous areas before planning.

### Phase 3: Architecture
```
/spec-kitty.plan
```
Creates `plan.md` with technical design, architecture, and implementation approach.

### Phase 4: Task Generation
```
/spec-kitty.tasks
```
Generates work packages in `tasks/planned/WP01.md`, `WP02.md`, etc.

### Phase 5: Implementation
```
/spec-kitty.implement
```
Processes tasks from `tasks/doing/` one by one with confirmation prompts.

Tasks move through lanes:
- `tasks/planned/` - Initial work packages
- `tasks/doing/` - Currently working on
- `tasks/review/` - Pending review
- `tasks/done/` - Completed

### Phase 6: Quality Checks (Optional)
```
/spec-kitty.analyze
```
Cross-artifact consistency check across spec, plan, and tasks.

```
/spec-kitty.checklist
```
Generate quality checklists for requirements validation.

### Phase 7: Review and Accept
```
/spec-kitty.review
```
Review prompts and move them to `tasks/done/`.

```
/spec-kitty.accept
```
Run acceptance checks to verify feature is complete and ready to merge.

### Phase 8: Merge
```
/spec-kitty.merge
```
Merge feature branch to main and clean up the worktree.

Or from project root:
```bash
bin/sk-merge 001-add-caching
```

## Workflow Examples

### Example 1: Small Bug Fix with Multiple Changes

```bash
# 1. Create feature from project root
bin/sk-new-feature "Fix memory leak in MLX backend initialization"

# 2. Navigate to worktree
cd kitty-specs/001-fix-memory-leak/

# 3. Create spec
/spec-kitty.specify
# AI creates spec.md with bug description, root cause, fix approach

# 4. Create plan
/spec-kitty.plan
# AI creates technical plan

# 5. Generate tasks
/spec-kitty.tasks
# AI creates WP01.md, WP02.md, etc.

# 6. Implement
/spec-kitty.implement
# AI processes each task with confirmation

# 7. Accept and merge
/spec-kitty.accept
/spec-kitty.merge

# 8. Return to main repo
cd ../../
```

### Example 2: Parallel Development of Two Features

```bash
# Terminal 1: Work on caching feature
bin/sk-new-feature "Add Redis caching layer"
cd kitty-specs/001-add-redis-caching/
# Use Claude Code here with /spec-kitty.* commands

# Terminal 2: Work on metrics feature (simultaneously!)
bin/sk-new-feature "Add Prometheus metrics"
cd kitty-specs/002-add-prometheus-metrics/
# Use Cursor here (different AI agent, same project)

# Terminal 3: Monitor both features
bin/sk-dashboard
# See both features progressing in real-time
```

Each feature is isolated in its own git worktree, so no conflicts!

## Best Practices

1. **One feature per worktree**: Keep features isolated for clarity
2. **Use descriptive feature descriptions**: Helps with auto-generated IDs
3. **Complete `/spec-kitty.specify` first**: Good specs lead to better plans
4. **Review tasks before `/spec-kitty.implement`**: Adjust work packages if needed
5. **Commit frequently**: Each worktree is a full git repo
6. **Use the dashboard**: Visual feedback helps track progress
7. **Clean up merged features**: Run `/spec-kitty.merge` to remove worktrees
8. **Follow UTF-8 encoding rules**: See `.kittify/AGENTS.md`

## Summary

Spec-Kitty enables rapid, systematic feature development with:
- Worktree-based isolation
- Real-time visual dashboard
- Multi-agent coordination
- Automated task management
- Parallel development workflows

Use it for small/medium features and bugs, while keeping the existing `.specify/` workflow for large architectural work.
