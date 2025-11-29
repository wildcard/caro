---
id: "guide-git-001"
title: "Undo last commit but keep changes"
description: "Reset your last Git commit while preserving your work in the staging area"
category: Git
difficulty: Beginner
tags: [git, undo, reset, commit, staging]
natural_language_prompt: "undo my last git commit but keep the changes"
generated_command: "git reset --soft HEAD~1"
shell_type: Bash
risk_level: Safe
author: "cmdai-community"
created_at: "2024-01-10T10:00:00Z"
updated_at: "2024-11-28T14:30:00Z"
prerequisites:
  - "You have made at least one Git commit"
  - "You have not pushed the commit to a remote repository"
expected_outcomes:
  - "Last commit is removed from Git history"
  - "All changes from that commit are in staging area"
  - "You can modify files and recommit"
related_guides:
  - "guide-git-002"
  - "guide-git-015"
related_guardrails: []
alternatives:
  - "git reset --mixed HEAD~1  # Undo commit, unstage changes"
  - "git reset --hard HEAD~1   # Undo commit, discard changes (⚠️ destructive)"
  - "git commit --amend        # Modify last commit message/content"
---

# Undo Last Commit But Keep Changes

## What it does

Undoes your most recent Git commit, moving the changes back to your staging area. Your work is preserved, but the commit is removed from history.

## When to use this

- ✅ You committed too early and want to add more changes
- ✅ You made a typo in the commit message (and want to recommit)
- ✅ You want to split one commit into multiple smaller commits
- ✅ You forgot to add a file to the commit
- ⚠️ **Don't use** if you've already pushed the commit to a shared branch

## The cmdai way

Instead of remembering Git flags, just ask:

```bash
cmdai "undo my last git commit but keep the changes"
```

cmdai generates:
```bash
git reset --soft HEAD~1
```

## Understanding the command

```bash
git reset --soft HEAD~1
```

Breaking it down:
- `git reset`: Moves the current branch pointer
- `--soft`: Keep changes in staging area (safest option)
- `HEAD~1`: Go back one commit from current HEAD

**Other reset options:**
- `--soft`: Changes stay staged (ready to commit again)
- `--mixed`: Changes unstaged but still in working directory
- `--hard`: Changes completely discarded (⚠️ dangerous!)

## Step-by-step example

Let's say you just committed:

```bash
$ git log --oneline
abc1234 (HEAD -> main) Added new feature
def5678 Previous commit
```

Run the reset:
```bash
$ git reset --soft HEAD~1
```

Check status:
```bash
$ git status
On branch main
Changes to be committed:
  (use "git restore --staged <file>..." to unstage)
        modified:   src/feature.rs
        new file:   tests/feature_test.rs
```

Your commit is gone, but changes are staged:
```bash
$ git log --oneline
def5678 (HEAD -> main) Previous commit
```

Now you can:
- Add more changes: `git add other-file.rs`
- Modify files: `vim src/feature.rs`
- Commit again: `git commit -m "Better message"`

## Safety notes

✓ **Safe operation** - Your changes are preserved in the staging area

✗ **History modification** - Don't use this on commits you've already pushed to shared branches (causes divergence)

✗ **Only affects last commit** - To undo multiple commits, use `HEAD~2`, `HEAD~3`, etc.

## Common mistakes

**Mistake 1: Using --hard by accident**
```bash
# ⚠️ WRONG - This deletes your changes!
git reset --hard HEAD~1
```
Always use `--soft` if you want to keep your work.

**Mistake 2: Trying to undo pushed commits**
If you've already pushed, use `git revert` instead:
```bash
git revert HEAD  # Creates new commit that undoes changes
```

**Mistake 3: Forgetting to commit again**
After reset, your changes are staged but not committed. Don't forget to:
```bash
git commit -m "Your message"
```

## Related guides

- [Undo last commit and discard changes](../git/undo-and-discard.md) - `git reset --hard HEAD~1`
- [Amend last commit message](../git/amend-commit.md) - `git commit --amend`
- [Undo multiple commits](../git/undo-multiple.md) - `git reset --soft HEAD~3`
- [Undo pushed commit safely](../git/revert-commit.md) - `git revert HEAD`

## Try it yourself

```bash
# Try this guide in cmdai
cmdai guides run guide-git-001

# Or execute the command directly
git reset --soft HEAD~1

# Verify changes are staged
git status
```

## Community metrics

- **Upvotes:** 142
- **Downvotes:** 3
- **Execution count:** 1,834
- **Success rate:** 98%
- **Quality score:** 0.94

## Community feedback

> "Perfect for when I commit too early. Use this daily!" - *developer123*

> "I used to Google this every time. Now I just ask cmdai." - *junior_dev*

> "Would be helpful to mention that this doesn't work after pushing" - *senior_eng*
> *Note: Updated guide to include this warning! - cmdai team*
