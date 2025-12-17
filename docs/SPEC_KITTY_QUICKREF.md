# Spec-Kitty Quick Reference

## Helper Scripts

```bash
# Create new feature
bin/sk-new-feature "feature description"
bin/sk-new-feature "feature description" --name "Custom Name"

# View features
bin/sk-list                # List all features
bin/sk-dashboard          # Open dashboard (http://127.0.0.1:9237)

# Merge completed feature
bin/sk-merge 001-feature-id
```

## Workflow Commands (from feature worktree)

```bash
cd kitty-specs/001-feature-name/

# Sequential workflow
/spec-kitty.specify      # 1. Create spec.md
/spec-kitty.clarify      # 2. (Optional) Ask clarifying questions
/spec-kitty.plan         # 3. Create plan.md
/spec-kitty.tasks        # 4. Generate work packages
/spec-kitty.analyze      # 5. (Optional) Consistency check
/spec-kitty.implement    # 6. Execute tasks
/spec-kitty.review       # 7. Review completed work
/spec-kitty.accept       # 8. Acceptance checks
/spec-kitty.merge        # 9. Merge and cleanup
```

## Directory Structure

```
cmdai/
├── kitty-specs/              # Feature worktrees (auto-created)
│   └── 001-feature-name/     # Each feature
│       ├── spec.md
│       ├── plan.md
│       ├── tasks.md
│       └── tasks/
│           ├── planned/      # WP01.md, WP02.md, ...
│           ├── doing/        # Currently working on
│           ├── review/       # Pending review
│           └── done/         # Completed
├── .kittify/                 # Spec-Kitty config (in git)
└── .claude/commands/         # Slash commands (in git)
    └── spec-kitty.*.md
```

## Task Lanes

Tasks flow through lanes:
1. **planned** → Work packages ready to start
2. **doing** → Currently being implemented
3. **review** → Waiting for review/validation
4. **done** → Completed and accepted

## Common Operations

```bash
# Start dashboard
bin/sk-dashboard

# Stop dashboard
spec-kitty dashboard --stop

# List worktrees
git worktree list

# Remove worktree manually
git worktree remove kitty-specs/001-feature-name

# Fix encoding issues
spec-kitty validate-encoding --feature 001-feature-name --fix

# Validate tasks
spec-kitty validate-tasks --feature 001-feature-name --fix
```

## Parallel Development Example

```bash
# Terminal 1
bin/sk-new-feature "Add caching"
cd kitty-specs/001-add-caching/
# Work with Claude Code

# Terminal 2
bin/sk-new-feature "Add metrics"
cd kitty-specs/002-add-metrics/
# Work with Cursor/Codex

# Terminal 3
bin/sk-dashboard
# Monitor both features
```

## UTF-8 Encoding Rules

❌ **Avoid**:
- Smart quotes: " " ' '
- Em/en dashes: — –
- Special symbols: → × ± °

✅ **Use**:
- ASCII quotes: " '
- Hyphen: -
- ASCII arrow: ->
- Plain letters: x +/- degrees

## When to Use Spec-Kitty vs .specify/

**Use Spec-Kitty** (`kitty-specs/`):
- Small features (< 1 week)
- Medium features (1-2 weeks)
- Bug fixes with multiple changes
- Parallel development needed

**Use .specify/** (`specs/`):
- Large features (> 2 weeks)
- Major architectural changes
- Extensive research required

## Dashboard URL

http://127.0.0.1:9237

## Agent Rules

See `.kittify/AGENTS.md` for complete rules all AI agents must follow.

## Help

```bash
spec-kitty --help
spec-kitty init --help
spec-kitty dashboard --help
```
