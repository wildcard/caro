# Context Management Check

Analyze current session context, project state, and feedback pipeline to provide a unified situational awareness view.

## Instructions

### Part 1: Context Health

1. **Check context usage** by evaluating:
   - How many files have been read this session
   - How many tool calls have been made
   - Approximate conversation length

2. **Assess context health**:
   - LOW (< 40%): Continue normally
   - MEDIUM (40-60%): Consider wrapping up current task, avoid starting new major work
   - HIGH (60-80%): Complete current task, then create handoff
   - CRITICAL (> 80%): Create handoff immediately before context overflow

### Part 2: Project State

3. **Read cognitive state** from `.claude/memory/cognitive-state.md`:
   - Show top 3 priorities
   - Show active risks
   - Note when it was last synthesized (suggest `/caro.synthesize` if stale > 1 day)

4. **Check active work**:
   - Run `git branch --show-current` to show current branch
   - Check `thoughts/ledgers/` for active continuity ledgers
   - Check `thoughts/shared/handoffs/` for recent unresolved handoffs

### Part 3: Feedback Pipeline

5. **Read feedback status** from `.claude/memory/feedback-insights.yaml`:
   - Total insights count
   - Untriaged count
   - If untriaged > 0, suggest running `/insight-to-issue`

### Part 4: Recommendations

6. **Provide recommendations** based on all three dimensions:

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

7. **Output format**:
   ```
   ## Context Health: [LOW/MEDIUM/HIGH/CRITICAL]

   **Estimated Usage**: ~X%

   **Current Session Activity**:
   - Files read: N
   - Tools used: N
   - Active tasks: [list]

   ---

   ## Project State

   **Top Priorities**:
   1. [Priority from cognitive-state.md]
   2. [Priority from cognitive-state.md]
   3. [Priority from cognitive-state.md]

   **Active Work**:
   - Branch: [current branch]
   - Ledgers: [active ledger count]
   - Open handoffs: [count]

   **Risks**: [any active risks from cognitive-state.md]

   ---

   ## Feedback Pipeline

   **Insights**: [total] total, [untriaged] untriaged
   **Action needed**: [Yes - run /insight-to-issue | No - pipeline clear]

   ---

   ## Recommendation

   [Primary action to take based on context health + project state]
   ```

## Tips for Managing Context

- **Prefer Explore agents** over direct grep/glob for codebase exploration
- **Use skills on-demand** rather than loading everything upfront
- **Complete tasks to checkpoints** before starting new work
- **Create handoffs proactively** at 60% rather than waiting for overflow
- **Use /compact** to summarize and continue when you just need a bit more room

## Related Commands

| Command | Purpose |
|---------|---------|
| `/caro.synthesize` | Refresh cognitive state from all sources |
| `/feedback-intake` | Capture new feedback into the pipeline |
| `/insight-to-issue` | Convert insights to GitHub issues |
| `skill: create_handoff` | Save session state for next session |
| `skill: resume_handoff` | Resume from a previous handoff |
