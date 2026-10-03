#!/usr/bin/env bash
# UserPromptSubmit hook - Reminds assistant to include quick actions footer
# when stopping work and waiting for user reply
#
# SKIPS reminder when AskUserQuestion tool was used (has its own interactive UI)

# Check if last assistant message used AskUserQuestion tool
# This tool already provides interactive options, so footer would be redundant
if echo "${CLAUDE_CONVERSATION_HISTORY:-}" | grep -q "AskUserQuestion" 2>/dev/null; then
  # Skip footer - AskUserQuestion provides its own UI
  exit 0
fi

cat <<'EOF'

**[Quick Actions Footer Reminder]**

When you stop work and wait for user input (questions, confirmations, task completion), ALWAYS end your message with:

```markdown
---
**Quick Actions:**
  `y` = yes | `c` = continue | `ta` = try again | `n` = next | `rp` = recommended plan

**💡 Recommended:** [specific next step based on context]
```

**Examples of when to include:**
- ✅ "Should I proceed?" → Include footer
- ✅ "I've completed X. What's next?" → Include footer
- ✅ "Here are 3 options..." → Include footer (unless using AskUserQuestion tool)
- ✅ "Created PR #603" → Include footer
- ❌ Using AskUserQuestion tool → Skip footer (has its own UI)
- ❌ Mid-task status update → Skip footer
- ❌ Continuing multi-step work → Skip footer

**Recommended text examples:**
- After PR: "Review and merge PR #603"
- After test: "Run tests and verify functionality"
- After feature: "Continue with next feature"
- After fix: "Verify the fix works"
- After plan: "Review plan and start implementation"

**[Legible Output]** (`.claude/rules/legible-output.md`)
- Line 1: `**Needs you:** <question>` if the owner must decide. Then the result, then details.
- STE-lite: ≤20 words per instruction, ≤25 per description, active voice, one term per concept.
- Text not enough? Use a diagram, then an HTML artifact. Number open questions and give a default.

EOF
