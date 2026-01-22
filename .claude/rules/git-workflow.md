# Git Workflow - Feature Branches Required

**CRITICAL**: NEVER work directly on main branch for ANY changes, no matter how small.

## Why This Matters

The user runs 4-5 parallel Claude sessions simultaneously. Direct commits to main cause constant merge conflicts and workflow disruption across all sessions.

## Required Workflow

### Before Making ANY Changes

1. **Create a feature branch with worktree:**
   ```bash
   bin/sk-new-feature "description"
   cd .worktrees/NNN-feature-name/
   ```

2. **OR create a hotfix branch:**
   ```bash
   git worktree add .worktrees/hotfix-name -b hotfix/name
   cd .worktrees/hotfix-name/
   ```

### Work in the Worktree

- Make changes, commit, test in the worktree
- Push branch: `git push -u origin branch-name`
- Create PR: `gh pr create` or via GitHub UI
- Wait for review and merge

### NEVER Commit to Main

The ONLY exceptions:
- User explicitly says "commit directly to main" (rare)
- Merging approved PRs via GitHub UI
- Emergency situations explicitly requested by user

## Enforcement

A PreToolUse hook blocks `git commit` commands when on main branch.

If you need to commit:
1. Check branch: `git branch --show-current`
2. If output is `main` → CREATE A FEATURE BRANCH FIRST
3. Never bypass the hook

## Examples

❌ **WRONG:**
```bash
# On main branch
git add file.txt
git commit -m "fix typo"  # BLOCKED BY HOOK
```

✅ **CORRECT:**
```bash
# Create feature branch first
bin/sk-new-feature "fix typo in readme"
cd .worktrees/001-fix-typo-in-readme/
git add file.txt
git commit -m "fix typo"  # Allowed on feature branch
git push -u origin fix/typo-in-readme
gh pr create
```

## Why Feature Branches Matter

- **Parallel work**: Multiple sessions can work independently
- **No conflicts**: Each session has its own branch
- **Code review**: All changes reviewed before merging
- **Rollback**: Easy to abandon bad branches
- **CI/CD**: Test changes before they hit main
