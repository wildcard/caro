# Example: Fix-Merge PR

This walkthrough shows how PR Akita handles a code PR with fixable CI issues -- the core of the vibe-maintainer philosophy.

## Scenario

**PR #203**: "Add support for zsh history expansion in queries"
- Author: @helpful-contributor
- Files changed: `src/platform/shell.rs`, `tests/platform_tests.rs` (2 files)
- Lines: +47 -3
- CI: **Failing** (clippy warning + 1 test failure)
- Age: 3 days
- Reviews: 0

## Triage

PR Akita scans the PR and classifies it:

```
PR #203: Add support for zsh history expansion in queries
  Author: @helpful-contributor
  Files: src/platform/shell.rs, tests/platform_tests.rs (+47 -3)
  CI: ✗ failing
    - clippy: unused import on line 12
    - test: platform_tests::test_shell_detection expected "bash" got "zsh"
  Reviews: 0
  Bucket: Fix-Merge
  Reason: CI failures are fixable (unused import + test expectation), intent is sound

  → Disposition: Merge-fix
```

## Why Fix-Merge, Not Request Changes

**Traditional approach** (gatekeeping):
> "CI is failing. Please remove the unused import on line 12 and fix the test expectation in `test_shell_detection`. Thanks!"

Result: Contributor may not see this for days. PR goes stale. Contributor moves on.

**Vibe Maintainer approach** (fix-merge):
> Fix it yourself. Merge. Credit them. Move on.

## Fix-Merge Execution

### Step 1: Checkout
```bash
git fetch origin pull/203/head:pr-203
git checkout pr-203
```

### Step 2: Identify Issues
```bash
cargo clippy 2>&1 | head -20
# warning: unused import `std::env::consts::OS`

cargo test platform_tests 2>&1 | tail -10
# assertion failed: expected "bash", got "zsh"
```

### Step 3: Fix

**Fix 1**: Remove unused import in `src/platform/shell.rs:12`
```diff
-use std::env::consts::OS;
```

**Fix 2**: Update test expectation in `tests/platform_tests.rs`
```diff
-    assert_eq!(detected_shell, "bash");
+    assert!(["bash", "zsh", "fish", "sh"].contains(&detected_shell.as_str()));
```

### Step 4: Commit with Attribution
```bash
git add src/platform/shell.rs tests/platform_tests.rs
git commit -m "fix: remove unused import and broaden shell detection test

Co-authored-by: helpful-contributor <helpful-contributor@users.noreply.github.com>"
```

### Step 5: Push
```bash
git push origin pr-203 --force-with-lease
```

### Step 6: Comment
> Thanks for adding zsh history expansion support, @helpful-contributor!
>
> I fixed two small issues:
> - Removed an unused import (`std::env::consts::OS`)
> - Broadened the shell detection test to accept multiple valid shells
>
> Merging now -- you get full credit as co-author. Great contribution!

### Step 7: Merge
Squash merge PR #203.

## Time Elapsed

~10 minutes from scan to merge.

## Comparison

| Approach | Time to Merge | Contributor Experience |
|----------|--------------|----------------------|
| Request changes | 3-7 days (if ever) | Frustrating, may not return |
| Fix-merge | 10 minutes | Delightful, likely to contribute again |

## Key Takeaway

The fix took 10 minutes. Requesting changes would have added days of latency and risked losing the contributor entirely. Fix-merge is faster for everyone and creates a better community experience.
