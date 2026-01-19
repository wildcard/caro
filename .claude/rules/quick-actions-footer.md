# Quick Actions Footer

**CRITICAL**: Always include a quick actions footer when stopping work and waiting for user reply.

## When to Add Footer

Add the quick actions footer when:
- ✅ Asking a question
- ✅ Requesting approval or confirmation
- ✅ Completing a task and waiting
- ✅ Presenting options for user to choose
- ✅ Finishing implementation and asking "what's next?"

Do NOT add when:
- ❌ Using AskUserQuestion tool (it has its own interactive UI)
- ❌ In the middle of multi-step work
- ❌ Presenting information without asking for input
- ❌ Footer already present in message

## Footer Format

```markdown
---
**Quick Actions:**
  `y` = yes | `c` = continue | `ta` = try again | `n` = next | `rp` = recommended plan

**💡 Recommended:** [context-specific next step]
```

## Quick Actions Meaning

| Action | Meaning | When User Types This |
|--------|---------|---------------------|
| `y` | Yes | Approve/confirm the current action |
| `c` | Continue | Keep going with current work |
| `ta` | Try again | Retry the last operation |
| `n` | Next | Move to next task/step |
| `rp` | Recommended plan | Show the recommended next step |

**Note**: `n` means "next task", NOT "no". If user wants to decline, they'll type what they want done instead.

## Recommended Plan Examples

Choose based on context:

| Context | Recommended |
|---------|-------------|
| PR created | "Review and merge the PR" |
| Tests needed | "Run tests and verify functionality" |
| Feature completed | "Continue with next feature" |
| Bug fixed | "Verify the fix works" |
| Plan created | "Review plan and start implementation" |
| All done | "Move to next task or close session" |

## Example Usage

```markdown
I've created the feature branch enforcement hook and opened PR #603.
The hook will block direct commits to main and guide users to create
feature branches instead.

---
**Quick Actions:**
  `y` = yes | `c` = continue | `ta` = try again | `n` = next | `rp` = recommended plan

**💡 Recommended:** Review and merge PR #603 to activate the hook
```

## Implementation

Always add this footer before your final closing statement when waiting for user input.
Make the "Recommended" text specific and actionable based on what you just accomplished.
