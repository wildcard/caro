# Spec-Kitty Integration Guide

This guide explains how to use Spec-Kitty in the caro project for rapid, multi-branch feature development.

## Overview

**Spec-Kitty** is integrated into caro to enable:
- **Worktree-based development**: Work on multiple features simultaneously without branch switching
- **Real-time dashboard**: Visual kanban board showing all features and their status
- **Multi-agent coordination**: Collaborate with multiple AI agents (Claude Code, Codex, etc.)
- **Spec-driven workflows**: Systematic approach to features, enhancements, and bug fixes

## Architecture

### Directory Structure

```
caro/
├── .kittify/               # Spec-Kitty configuration (committed to git)
│   ├── missions/           # Workflow templates (software-dev, research)
│   ├── scripts/            # Automation scripts
│   ├── memory/             # Project memory (constitution, etc.)
│   └── AGENTS.md           # Rules for all AI agents
├── .claude/
│   ├── commands/           # Slash commands (committed to git)
│   │   ├── spec-kitty.*.md # Spec-Kitty workflow commands
│   │   └── *.md            # Other commands
│   └── sessions/           # Session data (excluded from git)
├── .codex/
│   └── prompts/            # Codex prompts (committed to git)
├── .specify/               # Original spec-kit setup
│   ├── memory/             # Project constitution
│   └── templates/          # Spec templates
├── kitty-specs/            # Feature worktrees (excluded from git)
│   ├── 001-feature-name/   # Each feature is a git worktree
│   └── 002-another-feature/
├── bin/                    # Helper scripts
│   ├── sk-new-feature      # Create new feature
│   ├── sk-dashboard        # Open dashboard
│   ├── sk-list             # List all features
│   └── sk-merge            # Merge completed feature
└── specs/                  # Existing large feature specs
```

### Integration Strategy

**Use Spec-Kitty for:**
- ✅ Small features (< 1 week)
- ✅ Medium features (1-2 weeks)
- ✅ Bug fixes with multiple changes
- ✅ Enhancements requiring specification
- ✅ Features you want to develop in parallel

**Use existing `.specify/` workflow for:**
- 📋 Large features (> 2 weeks)
- 📋 Major architectural changes
- 📋 Features requiring extensive research

**Both workflows can coexist!** The existing `specs/` directory contains large feature specs, while `kitty-specs/` contains rapid development features.

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

### 4. Spec-Kitty Workflow Commands

Use these slash commands in Claude Code **from within the feature worktree**:

#### Phase 1: Specification
```
/spec-kitty.specify
```
Creates `spec.md` with feature requirements, scope, and acceptance criteria.

#### Phase 2: Planning (Optional Enhancement)
```
/spec-kitty.clarify
```
Asks targeted questions to de-risk ambiguous areas before planning.

#### Phase 3: Architecture
```
/spec-kitty.plan
```
Creates `plan.md` with technical design, architecture, and implementation approach.

#### Phase 4: Task Generation
```
/spec-kitty.tasks
```
Generates work packages in `tasks/planned/WP01.md`, `WP02.md`, etc.

#### Phase 5: Implementation
```
/spec-kitty.implement
```
Processes tasks from `tasks/doing/` one by one with confirmation prompts.

Tasks move through lanes:
- `tasks/planned/` → Initial work packages
- `tasks/doing/` → Currently working on
- `tasks/review/` → Pending review
- `tasks/done/` → Completed

#### Phase 6: Quality Checks (Optional)
```
/spec-kitty.analyze
```
Cross-artifact consistency check across spec, plan, and tasks.

```
/spec-kitty.checklist
```
Generate quality checklists for requirements validation.

#### Phase 7: Review and Accept
```
/spec-kitty.review
```
Review prompts and move them to `tasks/done/`.

```
/spec-kitty.accept
```
Run acceptance checks to verify feature is complete and ready to merge.

#### Phase 8: Merge
```
/spec-kitty.merge
```
Merge feature branch to main and clean up the worktree.

Or from project root:
```bash
bin/sk-merge 001-add-caching
```

### 5. Dashboard Features

The dashboard shows:
- ✅ All features across the project
- ✅ Feature status (spec, planned, in progress, review, done)
- ✅ Task breakdown per feature
- ✅ File integrity checks
- ✅ Git worktree status
- ✅ Live updates as you work

Access at: http://127.0.0.1:9237

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

### Example 3: Using with Charm.land Crush for Rapid Iteration

Charm.land Crush allows you to develop across multiple branches quickly. Spec-Kitty complements this:

```bash
# Use Crush to create multiple branches
# Then use spec-kitty to manage them as features

# Feature A
bin/sk-new-feature "Feature A description"

# Feature B
bin/sk-new-feature "Feature B description"

# Feature C
bin/sk-new-feature "Feature C description"

# View all three in dashboard
bin/sk-dashboard

# Work on each in parallel using different AI agents or terminals
# spec-kitty tracks the state of each independently
```

## Helper Scripts

Located in `bin/`:

- **`sk-new-feature`**: Create new feature with worktree
- **`sk-dashboard`**: Open dashboard in browser
- **`sk-list`**: List all features and their status
- **`sk-merge`**: Merge completed feature and cleanup

All scripts include colored output and helpful messages.

## Integration with Existing Workflows

### Coexistence with `.specify/`

The project has two complementary systems:

1. **`.specify/` (Original)**: For large features and complex architecture
   - Used for `specs/001-*`, `specs/002-*`, etc.
   - Manual workflow with constitution-based development
   - Shared slash commands in `.codex/prompts/`

2. **`.kittify/` (Spec-Kitty)**: For rapid, parallel development
   - Used for `kitty-specs/001-*`, `kitty-specs/002-*`, etc.
   - Automated workflows with worktrees
   - Slash commands in `.claude/commands/spec-kitty.*`

**Both can be used together!** Choose based on feature size and complexity.

### Constitution Alignment

Both `.specify/memory/constitution.md` and `.kittify/memory/constitution.md` exist.

**Recommendation**: Keep `.specify/memory/constitution.md` as the source of truth, and sync important principles to `.kittify/memory/constitution.md` as needed.

## Git Workflow

### Gitignore Strategy

The `.gitignore` is configured to:
- ✅ **Include** in git: Command files (`.claude/commands/`, `.codex/prompts/`)
- ✅ **Include** in git: Spec-Kitty config (`.kittify/`)
- ❌ **Exclude** from git: Feature worktrees (`kitty-specs/`)
- ❌ **Exclude** from git: Agent session data (`.claude/sessions/`, etc.)
- ❌ **Exclude** from git: Dashboard runtime (`.kittify/.dashboard`)

This ensures:
- Team members can clone and get all workflow commands
- Each developer manages their own feature worktrees
- No credentials or session data leaks into git

### Worktree Mechanics

When you run `bin/sk-new-feature`:
1. Creates a new branch `feature/001-feature-name`
2. Creates a git worktree in `kitty-specs/001-feature-name/`
3. The worktree is a full git repository linked to the main repo
4. You can work in the worktree while the main repo stays on a different branch

**Benefits**:
- No branch switching overhead
- Multiple features in progress simultaneously
- Each worktree is isolated (different node_modules, different .env, etc.)
- Easy cleanup with `/spec-kitty.merge`

## Dashboard Details

### Starting the Dashboard

```bash
# Option 1: Use helper script
bin/sk-dashboard

# Option 2: Use spec-kitty CLI
spec-kitty dashboard

# Option 3: Use slash command in Claude Code
/spec-kitty.dashboard
```

### Stopping the Dashboard

```bash
spec-kitty dashboard --stop
```

### Dashboard Views

1. **Features Overview**: All features and their branches
2. **Task Kanban**: Tasks organized by lane (planned → doing → review → done)
3. **File Status**: Spec.md, plan.md, tasks.md status per feature
4. **Git Status**: Worktree status and branch information

### Dashboard Auto-Updates

The dashboard automatically detects changes:
- New files created
- Tasks moved between lanes
- Git commits
- File modifications

Refresh your browser to see the latest state.

## Troubleshooting

### Dashboard Not Starting

```bash
# Check if port 9237 is already in use
lsof -i :9237

# Stop existing dashboard
spec-kitty dashboard --stop

# Restart
bin/sk-dashboard
```

### Feature Directory Issues

```bash
# List all git worktrees
git worktree list

# Remove a worktree manually if needed
git worktree remove kitty-specs/001-feature-name

# Prune deleted worktrees
git worktree prune
```

### UTF-8 Encoding Issues

If the dashboard shows blank pages, check for encoding issues:

```bash
# Check encoding
spec-kitty validate-encoding --feature 001-feature-name

# Auto-fix (creates .bak backups)
spec-kitty validate-encoding --feature 001-feature-name --fix

# Fix all features
spec-kitty validate-encoding --all --fix
```

**Common causes**:
- Copy-pasting from Microsoft Word/Outlook (smart quotes)
- Using em-dashes (—) instead of hyphens (-)
- Using arrows (→) instead of ASCII (->)

See `.kittify/AGENTS.md` for full encoding rules.

## Advanced Usage

### Custom Missions

Spec-Kitty supports different "missions" (workflow modes):

```bash
# Check current mission
cat .kittify/active-mission

# Available missions
ls .kittify/missions/
# - software-dev (default)
# - research

# Switch missions
spec-kitty mission switch research
```

### Task Validation

```bash
# Validate task metadata
spec-kitty validate-tasks --feature 001-feature-name

# Auto-fix task issues
spec-kitty validate-tasks --feature 001-feature-name --fix
```

### Manual Task Movement

```bash
# From within a feature worktree
bash ../.kittify/scripts/bash/move-task-to-doing.sh WP01

bash ../.kittify/scripts/bash/tasks-move-to-lane.sh WP01 review

bash ../.kittify/scripts/bash/tasks-move-to-lane.sh WP01 done
```

## Best Practices

1. **One feature per worktree**: Keep features isolated for clarity
2. **Use descriptive feature descriptions**: Helps with auto-generated IDs
3. **Complete `/spec-kitty.specify` first**: Good specs lead to better plans
4. **Review tasks before `/spec-kitty.implement`**: Adjust work packages if needed
5. **Commit frequently**: Each worktree is a full git repo
6. **Use the dashboard**: Visual feedback helps track progress
7. **Clean up merged features**: Run `/spec-kitty.merge` to remove worktrees
8. **Follow UTF-8 encoding rules**: See `.kittify/AGENTS.md`

## Integration with CI/CD

The spec-kitty commands can be used in CI/CD:

```yaml
# Example GitHub Actions workflow
- name: Run acceptance checks
  run: |
    cd kitty-specs/${{ env.FEATURE_ID }}/
    spec-kitty accept --feature ${{ env.FEATURE_ID }}
```

Non-interactive mode available with `--json` flag for automation.

## Resources

- **Spec-Kitty Docs**: https://github.com/Priivacy-ai/spec-kitty
- **Agent Rules**: `.kittify/AGENTS.md`
- **Command Reference**: `.claude/commands/spec-kitty.*.md`
- **Helper Scripts**: `bin/sk-*`

## Summary

Spec-Kitty enables rapid, systematic feature development with:
- ✅ Worktree-based isolation
- ✅ Real-time visual dashboard
- ✅ Multi-agent coordination
- ✅ Automated task management
- ✅ Parallel development workflows

Use it for small/medium features and bugs, while keeping the existing `.specify/` workflow for large architectural work.

**Next Steps**:
1. Create your first feature: `bin/sk-new-feature "your feature description"`
2. Open the dashboard: `bin/sk-dashboard`
3. Follow the workflow: specify → plan → tasks → implement → accept → merge
