---
description: Create structured, atomic commits with semantic tagging
---

## Commit: Structured Atomic Commits

This command guides you through creating well-structured commits following conventional commit standards.

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/caro.commit` | Interactive commit workflow |
| `/caro.commit "message"` | Quick commit with message |
| `/caro.commit --amend` | Amend previous commit |
| `/caro.commit --dry-run` | Preview without committing |

---

## What This Command Does

`/caro.commit` ensures commits are:
- **Atomic** - One logical change per commit
- **Well-described** - Clear, semantic commit messages
- **Safe** - No sensitive files accidentally committed
- **Verified** - Pre-commit checks pass

---

## Outline

### 1. Parse Arguments

```
ARGUMENTS check:
- Empty → INTERACTIVE_MODE (guided workflow)
- "message" → QUICK_MODE (direct commit)
- "--amend" → AMEND_MODE (modify last commit)
- "--dry-run" → PREVIEW_MODE (show what would happen)
```

### 2. Pre-flight Checks

**Check for changes:**
```bash
# See what's changed
git status --porcelain

# If nothing to commit, exit early
if [ -z "$(git status --porcelain)" ]; then
    echo "Nothing to commit"
    exit 0
fi
```

**Check for sensitive files:**
```bash
# Look for potentially sensitive files
git status --porcelain | grep -E '\.(env|pem|key|secret|credentials)' && \
    echo "WARNING: Potentially sensitive files detected!"

# Check for common secret patterns in staged content
git diff --cached | grep -iE '(password|secret|api_key|token).*=' && \
    echo "WARNING: Possible credentials in staged changes!"
```

### 3. Review Changes

**Show what will be committed:**
```bash
# Summary of changes
git diff --stat HEAD

# Detailed diff (for review)
git diff HEAD

# List untracked files
git ls-files --others --exclude-standard
```

### 4. Categorize Changes

Analyze the changes to suggest commit type:

| Type | Description | Example Changes |
|------|-------------|-----------------|
| `feat` | New feature | New function, module, capability |
| `fix` | Bug fix | Corrected behavior, error handling |
| `docs` | Documentation | README, comments, docstrings |
| `test` | Tests | New or updated test cases |
| `refactor` | Code change (no behavior change) | Restructure, rename, simplify |
| `perf` | Performance improvement | Optimization, caching |
| `style` | Formatting | Whitespace, formatting |
| `chore` | Maintenance | Dependencies, build config |
| `ci` | CI/CD | Workflow updates |

**Auto-detect type:**
```
If changes are only in:
- tests/ → suggest "test"
- *.md → suggest "docs"
- .github/ → suggest "ci"
- Cargo.toml (deps only) → suggest "chore"
- *_test.rs → suggest "test"

If changes fix something:
- Commit message contains "fix" → suggest "fix"

Otherwise → suggest "feat" or "refactor"
```

### 5. Compose Commit Message

**Conventional commit format:**
```
<type>(<scope>): <short description>

[optional body]

[optional footer]
```

**Guidelines:**
- Type: Use conventional types (feat, fix, docs, etc.)
- Scope: Module or area affected (safety, backend, cli)
- Description: Imperative mood, lowercase, no period
- Body: Explain "why" not "what"
- Footer: References (Fixes #123, Closes #456)

**Examples:**
```
feat(safety): add rm -rf * pattern detection

The wildcard deletion pattern was missing from our safety
validation. This critical pattern blocks accidental deletion
of all files in the current directory.

Fixes #200
```

```
fix(backend): handle empty model response gracefully

Previously, an empty response would cause a panic. Now we
return a clear error with the model's status.
```

```
test(cli): add regression tests for issue #161

Added 7 test cases covering unquoted argument parsing
edge cases.
```

### 6. Stage and Commit

**INTERACTIVE_MODE:**
```bash
# Show changes
git status

# Ask what to stage
echo "Stage all changes? [Y/n/s(elect)]"

# Based on response:
# Y → git add .
# n → exit
# s → git add -p (interactive staging)

git add .

# Show final staging
git diff --cached --stat

# Confirm
echo "Commit with message:"
echo "  <type>(<scope>): <description>"
echo ""
echo "Proceed? [Y/n/e(dit)]"

# Commit
git commit -m "<message>"
```

**QUICK_MODE:**
```bash
git add .
git commit -m "$MESSAGE"
```

### 7. Post-commit Verification

```bash
# Show the commit
git log -1 --oneline

# Verify working tree is clean
git status
```

---

## Sensitive File Protection

### Blocked Files
These files are flagged and require explicit confirmation:
- `.env`, `.env.*`
- `*.pem`, `*.key`
- `*credentials*`, `*secret*`
- `config.local.*`

### Content Patterns
These patterns in staged content trigger warnings:
- `password=`, `secret=`, `api_key=`
- Private keys (`-----BEGIN.*PRIVATE KEY-----`)
- AWS credentials (`AKIA...`)

---

## Examples

### Interactive Commit
```
User: /caro.commit
Claude:
  Reviewing changes...

  Modified files:
    src/safety/mod.rs (+15, -3)
    tests/safety_test.rs (+25, -0)

  Detected type: fix (safety validation change)
  Suggested scope: safety

  Suggested message:
    fix(safety): block rm -rf * wildcard pattern

  Stage all changes? [Y/n/s]: Y
  Proceed with commit? [Y/n/e]: Y

  ✓ Committed: abc1234
    fix(safety): block rm -rf * wildcard pattern

  Working tree clean.
```

### Quick Commit
```
User: /caro.commit "docs: update CLAUDE.md with reference docs"
Claude:
  Staging all changes...

  Files to commit:
    CLAUDE.md
    .claude/reference/rust-cli-best-practices.md
    .claude/reference/safety-patterns.md
    .claude/reference/testing-patterns.md

  ✓ Committed: def5678
    docs: update CLAUDE.md with reference docs
```

### Dry Run
```
User: /caro.commit --dry-run
Claude:
  DRY RUN - No changes will be made

  Would stage:
    src/backends/mlx.rs (+42, -10)
    src/backends/mod.rs (+5, -2)

  Would commit with message:
    feat(backend): add MLX model caching

  No issues detected. Ready to commit.
```

### Sensitive File Warning
```
User: /caro.commit
Claude:
  ⚠️ WARNING: Sensitive files detected!

  The following files may contain secrets:
    .env.local

  These files are typically NOT committed.

  Options:
    [1] Exclude sensitive files and continue
    [2] Show file contents for review
    [3] Cancel commit

  Your choice:
```

---

## Amending Commits

### When to Amend
- Fix a typo in commit message
- Add forgotten file to last commit
- Fix small issue discovered immediately after commit

### Safety Checks
Before amending, verify:
```bash
# Check authorship
git log -1 --format='%an <%ae>'

# Verify not pushed
git status | grep "Your branch is ahead"
```

### Amend Workflow
```
User: /caro.commit --amend
Claude:
  Checking last commit...

  Last commit: abc1234
    Author: Your Name <you@example.com>
    Message: fix(safety): block rm -rf pattern

  ⚠️ Safety check:
    ✓ You are the author
    ✓ Not yet pushed

  What to amend?
    [1] Add more changes
    [2] Fix commit message
    [3] Both

  Your choice:
```

---

## Integration with Workflow

### Typical Workflow
```bash
# 1. Make changes
# 2. Review changes
/caro.code-review

# 3. Commit
/caro.commit

# 4. Push
git push -u origin <branch>
```

### With Feature Development
```bash
# During /spec-kitty.implement
# After completing a work package:
/caro.commit "feat(feature): implement WP03 validation layer"
```

---

## Commit Message Templates

### Feature
```
feat(<scope>): <what was added>

<Why this feature is needed>
<How it works at a high level>

Implements #<issue>
```

### Bug Fix
```
fix(<scope>): <what was fixed>

<What was the bug>
<What was the root cause>
<How it's fixed now>

Fixes #<issue>
```

### Refactor
```
refactor(<scope>): <what changed>

<Why the refactor was needed>
<What improvement this provides>

No behavior change.
```

---

## Notes

- Run `/caro.code-review` before committing
- One logical change per commit
- Use imperative mood ("add" not "added")
- Reference issues when applicable
- Never commit credentials or secrets
- Amend only your own, unpushed commits
