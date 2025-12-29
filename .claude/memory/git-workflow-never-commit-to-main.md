# Git Workflow: NEVER Commit Directly to Main

## ⚠️ Critical Rule

**NEVER commit directly to `main` branch.** Always use feature branches for ALL changes.

## ✅ Proper Workflow

### 1. Branch
Create feature branch from main:
```bash
git checkout main
git pull origin main
git checkout -b feature/descriptive-name
```

### 2. Commit
Make changes and commit to feature branch:
```bash
git add <files>
git commit -m "descriptive message"
```

### 3. Push
Push feature branch to remote:
```bash
git push origin feature/descriptive-name
```

### 4. PR
Create pull request via GitHub:
```bash
gh pr create --title "Title" --body "Description"
```

### 5. Test
Wait for CI/CD checks to pass on the PR.

### 6. Merge
Merge PR after approval:
```bash
gh pr merge --squash  # or --merge, --rebase
```

## Branch Naming Conventions

- `feature/add-roadmap-presentation` - New features
- `fix/correct-milestone-dates` - Bug fixes
- `docs/update-readme` - Documentation
- `refactor/improve-safety-module` - Code refactoring
- `test/add-integration-tests` - Test additions
- `chore/update-dependencies` - Maintenance
- `release/vX.Y.Z` - Release preparation (use `/caro.release.*` skills)

## Why This Matters

- **Code Review**: PRs allow team review before merge
- **CI/CD**: Automated tests run on PRs before merge
- **History**: Clean git history with descriptive PR titles
- **Rollback**: Easier to revert entire features
- **Collaboration**: Multiple people can work without conflicts
- **Safety**: Protects main branch from accidental breaks

## ❌ Exceptions

**NONE.** Even for:
- "Quick fixes"
- "Just documentation"
- "Small typo fixes"
- "Urgent hotfixes" (use `/caro.release.hotfix` instead)

**Always use a feature branch.**

## 🔍 Pre-Work Checklist

Before starting ANY work:

1. **Check current branch**:
   ```bash
   git branch --show-current
   ```

2. **If on `main`, STOP and create feature branch immediately**:
   ```bash
   git checkout -b feature/my-work
   ```

3. **Only merge via PR, never commit directly to main**

## 🎯 This Applies To

- Code changes
- Documentation updates
- Configuration files
- Presentations
- Scripts
- Dependencies
- **Everything**

## 🚨 If You Accidentally Committed to Main

If you realize you committed directly to main:

```bash
# DON'T PUSH! First undo the commit
git reset --soft HEAD~1

# Create feature branch
git checkout -b feature/my-work

# Recommit on feature branch
git commit -m "message"

# Push feature branch and create PR
git push origin feature/my-work
gh pr create
```

## 📝 Memory Created

**Date**: December 29, 2025
**Reason**: Committed roadmap presentation directly to main instead of using feature branch
**Lesson**: ALWAYS use feature branches, no exceptions
