# Context Engineering Improvements for Caro

Based on: [My experience with Claude Code 2.0](https://sankalp.bearblog.dev/my-experience-with-claude-code-20-and-how-to-get-better-at-using-coding-agents/)

**Date**: 2026-01-14
**Status**: Ready for Implementation

---

## Gap Analysis: What We Have vs What's Recommended

### Current State Assessment

| Feature | Blog Recommendation | Caro Status | Gap Level |
|---------|---------------------|-------------|-----------|
| CLAUDE.md | Central project context file | **Missing** | CRITICAL |
| Skills | On-demand domain expertise | 18 skills | Good |
| Agents | Specialized sub-agents | 26 agents | Good |
| Commands | Reusable workflow prompts | 25 commands | Good |
| Hooks | Lifecycle automation | 5 hooks active | Good |
| Context Management | /handoff, /compact workflows | Partial | MEDIUM |
| Attention Recitation | Todo/plan files in context | Partial | MEDIUM |
| Model Selection | Haiku/Sonnet/Opus guidance | **Missing** | HIGH |
| Throwaway Drafts | Feature branch exploration | Via spec-kitty | Good |
| Prompt Caching | Large context optimization | Unknown | LOW |

---

## CRITICAL: Create CLAUDE.md

The blog emphasizes keeping objectives in recent context. We're missing the foundational `CLAUDE.md` file that Claude Code uses for project context.

### Recommended CLAUDE.md Structure

```markdown
# Caro - Natural Language to Shell Commands

## Project Overview
[Brief description of what caro does]

## Key Files & Architecture
- `src/main.rs` - Entry point
- `src/inference/` - Backend trait + implementations
- `src/safety/` - Command safety validation

## Development Commands
- `cargo build` - Build the project
- `cargo test` - Run tests
- `cargo run -- "your query"` - Test locally

## Available Skills (Load On-Demand)
Use skill: `name` when working on specific domains:
- `safety-pattern-developer` - Adding safety patterns (TDD workflow)
- `beta-test-cycles` - Running test cycles
- `prompt-tuner` - Improving LLM prompts

## Current Focus Areas
[Updated per-session or per-sprint]
```

**Action**: Create `/home/user/caro/CLAUDE.md`

---

## HIGH: Model Selection Guidance

The blog recommends using different models for different tasks:
- **Opus 4.5**: Complex implementation, pair programming
- **Sonnet**: Faster sub-agent tasks, code review
- **Haiku**: Quick searches, simple validations

### Current Gap

Our agents don't specify model preferences. All agents run on the same model.

### Recommended Changes

1. **Update agent definitions** to include model hints:

```yaml
# In agent-profiles.yaml or individual agent files
agents:
  safety-regression-tester:
    model: sonnet  # Speed-optimized for repetitive checks

  pattern-gap-analyzer:
    model: opus    # Intelligence-optimized for complex analysis

  quick-search:
    model: haiku   # Cost-optimized for simple lookups
```

2. **Add model guidance to Task invocations** in commands:

```markdown
# In commands that spawn agents
Use the Task tool with model="haiku" for this quick search...
Use the Task tool with model="opus" for this complex analysis...
```

---

## MEDIUM: Context Management Commands

### What the Blog Recommends

- **Monitor context**: Check usage with `/context`
- **Start fresh at 60%**: Begin new tasks before hitting limits
- **Use /compact or /handoff**: Preserve state when context fills

### Current State

- We have `create_handoff` and `resume_handoff` skills
- We have pre-compact hooks
- Missing: Easy context monitoring commands

### Recommended Addition

Create `/caro.context` command:

```markdown
# .claude/commands/caro.context.md

Check current context usage and provide recommendations:

1. Use /context to check current usage percentage
2. If > 60%, recommend:
   - Using skill: create_handoff before starting new major task
   - Using /compact for quick continuation
3. If > 80%, strongly recommend:
   - skill: create_handoff immediately
   - Complete current task, then start fresh session
```

---

## MEDIUM: Attention Recitation - Active Todo Files

### What the Blog Recommends

> "Maintain todo lists and plans as markdown files. Regularly update them to keep objectives in recent context, combating 'lost-in-the-middle' performance degradation."

### Current State

- We use TodoWrite tool (in-memory)
- We have `.claude/memory/` directory (unclear usage)
- Missing: Persistent markdown todo files that get loaded into context

### Recommended Changes

1. **Create session-persistent todo file**:

```markdown
# .claude/memory/current-tasks.md

## Active Sprint Tasks
- [ ] Task 1 description
- [ ] Task 2 description

## Completed This Session
- [x] What was done

## Blocked / Needs Discussion
- Item needing clarification

Last Updated: [timestamp]
```

2. **Update SessionStart hook** to inject current-tasks.md into context

3. **Update PreCompact hook** to save TodoWrite state to current-tasks.md

---

## Enhancement: Parallel Sub-Agent Patterns

### What the Blog Recommends

> "Launch multiple parallel Explore agents for different code sections simultaneously when conducting comprehensive codebase analysis."

### How to Apply

Update commands like `/caro.qa` to explicitly spawn parallel agents:

```markdown
# Parallel investigation pattern
Launch these agents in parallel using a single message with multiple Task tool calls:

1. Task: Explore agent for "find all safety validation code"
2. Task: Explore agent for "find all test files for safety"
3. Task: Explore agent for "find configuration related to safety"

Wait for all results, then synthesize findings.
```

---

## Enhancement: Exploration-First Workflow

### What the Blog Recommends

> "Ask clarifying questions about requirements and codebase context before making changes."

### Current Gap

Our spec-kitty workflow starts with `/specify` but doesn't have an explicit exploration phase.

### Recommended Addition

Add exploration phase to `/caro.feature`:

```markdown
## Phase 0: Exploration (NEW)

Before creating specifications:

1. **Understand the request**:
   - What exactly is being asked?
   - What are the acceptance criteria?

2. **Explore the codebase**:
   - Use Explore agent to find related code
   - Identify files that will need changes
   - Note any existing patterns to follow

3. **Ask clarifying questions**:
   - What edge cases should be handled?
   - Are there platform-specific concerns?
   - What's the priority if time is limited?

Only proceed to /specify after exploration is complete.
```

---

## Enhancement: Skill Loading Reminders via Hooks

### What the Blog Recommends

> "Use hooks to inject reminders about available skills. This reduces reliance on permanent CLAUDE.md entries while ensuring relevant expertise loads when needed."

### Current State

We have skills but no automatic reminders about them.

### Recommended Hook Addition

Create a UserPromptSubmit hook that suggests relevant skills:

```javascript
// .claude/hooks/skill-suggestion.mjs
// When user mentions keywords, suggest loading relevant skill

const SKILL_KEYWORDS = {
  'safety': 'safety-pattern-developer',
  'pattern': 'safety-pattern-developer',
  'beta': 'beta-test-cycles',
  'release': 'quality-engineer-manager',
  'prompt': 'prompt-tuner',
  'holiday': 'multicultural-holidays'
};

// Inject suggestion into response context
```

---

## Enhancement: Throwaway First Draft Pattern

### What the Blog Recommends

> "Create feature branches and let Claude implement end-to-end while observing. Compare output to your expectations to sharpen subsequent iterations with better prompts."

### How This Maps to Caro

Our spec-kitty workflow already supports this via worktrees:

```bash
# Current pattern
/caro.feature  # Creates worktree branch
# ... implementation ...
/spec-kitty.merge  # Merge or discard
```

### Recommended Documentation

Add explicit "throwaway draft" guidance to CLAUDE.md:

```markdown
## Experimental Implementation Pattern

For complex features where requirements are unclear:

1. Create feature branch: `/caro.feature "experimental: feature name"`
2. Let implementation proceed end-to-end
3. Review the result - compare to your mental model
4. Either:
   - Merge if good: `/spec-kitty.merge`
   - Discard and retry with refined requirements
```

---

## Implementation Priority

### Immediate (This Session)

1. **Create CLAUDE.md** - Foundation for all other improvements
2. **Add model hints to key agents** - Quick efficiency win

### Short-Term (This Week)

3. **Create /caro.context command** - Context awareness
4. **Update SessionStart hook** - Load current tasks into context
5. **Document throwaway draft pattern** - In CLAUDE.md

### Medium-Term (Next Sprint)

6. **Implement skill suggestion hook** - Just-in-time expertise
7. **Add parallel sub-agent patterns** - To key commands
8. **Create persistent todo file workflow** - For attention recitation

---

## Key Takeaways from Blog Post

1. **Context is precious** - Monitor it, manage it, don't waste it
2. **Load knowledge just-in-time** - Skills > permanent context bloat
3. **Use the right model** - Haiku for quick, Opus for complex
4. **Explore before executing** - Understand before changing
5. **Parallel when possible** - Launch multiple agents together
6. **Checkpoint often** - Use /rewind when going off-track
7. **Maintain attention** - Keep objectives in recent context

---

## Success Metrics

After implementing these improvements:

- [ ] CLAUDE.md exists and is actively maintained
- [ ] Agents specify model preferences where appropriate
- [ ] Context warnings appear before hitting limits
- [ ] Session state persists across compacts
- [ ] Skills get suggested when relevant
- [ ] Parallel agent patterns are documented and used

---

*Analysis based on blog post by Sankalp - January 2026*
