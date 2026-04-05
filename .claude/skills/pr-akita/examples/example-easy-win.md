# Example: Easy Win PR

This walkthrough shows how PR Akita handles a documentation-only PR -- the simplest triage case.

## Scenario

**PR #157**: "Fix typo in README installation instructions"
- Author: @new-contributor
- Files changed: `README.md` (1 file)
- Lines: +1 -1
- CI: Passing
- Age: 2 hours

## Triage

PR Akita scans the PR and classifies it:

```
PR #157: Fix typo in README installation instructions
  Author: @new-contributor
  Files: README.md (1 file, +1 -1)
  CI: ✓ passing
  Reviews: 0
  Bucket: Easy Win
  Reason: docs-only, <20 lines, CI passing, single concern

  → Disposition: Merge
```

## Signals That Made This an Easy Win

1. **Files changed**: Only `README.md` -- documentation only
2. **Size**: 1 addition, 1 deletion -- trivially small
3. **CI**: All checks passing
4. **Single concern**: Just a typo fix

## Actions Taken

1. **Approve** the PR
2. **Merge** with squash
3. **Comment**:
   > Thanks for catching this typo, @new-contributor! Merged. Welcome to the project! 🎉

The `pr-merged.yml` workflow automatically:
- Detects this is the contributor's first PR
- Posts a milestone celebration comment
- Updates the all-contributors list

## Time Elapsed

Under 1 minute from scan to merge.

## Key Takeaway

Easy wins should be merged immediately. No review needed for trivial documentation fixes from anyone. The contributor gets instant positive feedback, making them more likely to contribute again.
