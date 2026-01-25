# Context Management Check

Analyze current session context and provide recommendations for maintaining productivity.

## Instructions

1. **Check context usage** by evaluating:
   - How many files have been read this session
   - How many tool calls have been made
   - Approximate conversation length

2. **Assess context health**:
   - LOW (< 40%): Continue normally
   - MEDIUM (40-60%): Consider wrapping up current task, avoid starting new major work
   - HIGH (60-80%): Complete current task, then create handoff
   - CRITICAL (> 80%): Create handoff immediately before context overflow

3. **Provide recommendations**:

   **If context is healthy (< 60%)**:
   - Continue current work
   - Can start new tasks safely

   **If context is filling (60-80%)**:
   - Finish current task to a stable checkpoint
   - Use `skill: create_handoff` to preserve state
   - Consider using `/compact` for quick continuation

   **If context is critical (> 80%)**:
   - Stop starting new work
   - Use `skill: create_handoff` immediately
   - Document what's incomplete in handoff
   - Start fresh session and use `skill: resume_handoff`

4. **Output format**:
   ```
   ## Context Health: [LOW/MEDIUM/HIGH/CRITICAL]

   **Estimated Usage**: ~X%

   **Current Session Activity**:
   - Files read: N
   - Tools used: N
   - Active tasks: [list]

   **Recommendation**: [action to take]
   ```

## Tips for Managing Context

- **Prefer Explore agents** over direct grep/glob for codebase exploration
- **Use skills on-demand** rather than loading everything upfront
- **Complete tasks to checkpoints** before starting new work
- **Create handoffs proactively** at 60% rather than waiting for overflow
- **Use /compact** to summarize and continue when you just need a bit more room
