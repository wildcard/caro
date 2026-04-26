# Agent Instructions: Branch Workflow

## CRITICAL: Never Push Directly to `main`

**All code changes MUST go through a dedicated worktree, branch, and PR.** The project has multiple agents working in parallel. Direct pushes to `main` will cause race conditions, lost work, and broken CI for other agents.

## Workflow for AI Agents

```bash
# 1. Create dedicated branch from latest main
git checkout -b agent/<feature-name>

# 2. Make changes, commit as normal
git add . && git commit -m "..."

# 3. Push branch and create PR
git push origin agent/<feature-name>
gh pr create --base main --title "..." --body "..."
```

## What NOT to Push to `main`

- Code changes (new features, bug fixes, refactors)
- Configuration changes (`.claude/rules/`, `website/`, `src/`)
- New files or renamed files

## What CAN Be Pushed to `main`

- `bd sync` commits (automated, idempotent)
- Automated translation PR merges

## Before Pushing to `main`

Always verify you're on a branch, not `main`:
```bash
CURRENT=$(git branch --show-current)
if [ "$CURRENT" = "main" ]; then
  echo "ERROR: Do not push code changes to main. Create a branch first."
  exit 1
fi
```
