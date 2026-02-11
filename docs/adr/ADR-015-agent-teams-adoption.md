# ADR-015: Agent Teams Adoption for Caro Development

**Status**: Proposed
**Date**: 2026-02-11
**Deciders**: wildcard
**Context**: Claude Code Agent Teams (experimental)

## Summary

Adopt Claude Code's experimental Agent Teams feature to coordinate parallel Claude Code instances for complex development tasks. Agent Teams differ from our current subagent (Task tool) usage by enabling **inter-agent communication**, **shared task lists**, and **independent context windows** that collaborate autonomously.

## Context

### Current Parallelism Model

Caro already uses three parallelism strategies:

| Strategy | How | Communication | Use Cases |
|----------|-----|---------------|-----------|
| **Subagents** (Task tool) | Child processes within one session | Report results back to parent only | Exploration, focused research, code gen |
| **Git worktrees** | Manual parallel sessions | None (independent) | 4-5 simultaneous developer sessions |
| **Ralph loops** | `loop.sh plan/build` | Log file only | Iterative task execution |

### What Agent Teams Add

Agent Teams introduce a **fourth** strategy: coordinated multi-session work where teammates can **message each other**, **share a task list**, and **self-coordinate** - all within a single orchestrated team.

| Capability | Subagents | Agent Teams |
|------------|-----------|-------------|
| Own context window | Yes | Yes |
| Inter-agent messaging | No (parent-only) | Yes (any-to-any) |
| Shared task list | No | Yes (with locking) |
| Self-claiming tasks | No | Yes |
| Plan approval gates | No | Yes |
| Hook integration | SubagentStop only | TeammateIdle + TaskCompleted |
| Token cost | Lower | Higher |

### When NOT to Use Agent Teams

- Sequential tasks or same-file edits
- Quick operations where subagents suffice
- Tasks with tight interdependencies
- Routine single-concern work

## Decision

Adopt Agent Teams for **five high-value workflows** where parallel exploration and inter-agent coordination provide clear benefits over existing tooling.

## Team Recipes

### Recipe 1: Multi-Perspective Code Review

**When**: PRs touching safety, backends, or cross-cutting concerns
**Why**: Single reviewers anchor on one issue type. Parallel specialized reviewers catch more.

```
Create an agent team to review PR #NNN. Spawn three reviewers:
- Security reviewer: Focus on command injection, input validation,
  and safety pattern coverage in src/safety/
- Performance reviewer: Check for unnecessary allocations, regex
  compilation patterns, and hot-path efficiency
- API contract reviewer: Validate trait implementations, public API
  stability, and backward compatibility

Use Sonnet for each teammate. Have them challenge each other's
findings before synthesizing a final report.
```

**Expected structure**:
- Lead: Orchestrator (delegate mode) - synthesizes final review
- Teammate 1: Security lens
- Teammate 2: Performance lens
- Teammate 3: API contract lens

**Quality gate hook**: `TaskCompleted` hook verifies each reviewer produced at least 3 findings before marking review complete.

---

### Recipe 2: Spec-Kitty Parallel Implementation

**When**: Work packages (WP01, WP02, ...) are independent modules
**Why**: Spec-kitty already generates independent work packages. Teams execute them in parallel with coordination.

```
Create an agent team to implement the feature in kitty-specs/NNN-feature/.
Each teammate owns one work package:
- WP01 teammate: [work package 1 title] - files: [list]
- WP02 teammate: [work package 2 title] - files: [list]
- WP03 teammate: [work package 3 title] - files: [list]

Rules:
- Require plan approval before any teammate starts coding
- Only approve plans that include test coverage
- Each teammate must run `make check` before marking tasks done
- If a teammate needs something from another WP, message directly

Use the shared task list. The lead should NOT implement anything -
delegate mode only.
```

**Integration with spec-kitty**:
- `/spec-kitty.tasks` generates work packages as usual
- Team lead reads `tasks/planned/*.md` to create the shared task list
- Each WP maps to a teammate
- Lane transitions (`planned` -> `doing` -> `done`) happen via task completion
- Lead synthesizes and runs final integration test

**File conflict prevention**: Each WP already specifies its file scope. Lead should verify no overlap at plan approval.

---

### Recipe 3: Competing Hypothesis Debugging

**When**: Bug with unclear root cause (e.g., intermittent test failures, platform-specific issues)
**Why**: Single-agent investigation anchors on first plausible theory. Parallel adversarial investigation converges faster.

```
Users report [describe bug]. Spawn 4 teammates to investigate:
- Hypothesis A: [theory about cause 1]
- Hypothesis B: [theory about cause 2]
- Hypothesis C: [theory about cause 3]
- Devil's advocate: Challenge all theories, look for alternative explanations

Have teammates talk to each other to disprove theories. Update
findings as they go. The theory that survives adversarial challenge
is most likely correct.
```

**Integration with existing debugging**:
- Works alongside `systematic-debug-agent` (subagent for focused investigation)
- Team approach for broad investigation; subagent for deep-dive once hypothesis is identified
- Lead creates a findings document at `thoughts/debug/[issue-id].md`

---

### Recipe 4: Release Preparation

**When**: Running `/caro.release.prepare` for a new version
**Why**: Release checks are independent and time-consuming when run sequentially.

```
Create an agent team for release v{VERSION} preparation:
- Security auditor: Run `cargo audit`, check dependencies, review
  OWASP patterns in src/safety/
- Test runner: Execute full test suite including contract tests,
  integration tests, and beta regression on all platforms
- Documentation reviewer: Verify CHANGELOG.md, README.md, version
  numbers, and API docs are current
- Cross-platform builder: Build release binaries for all 5 targets,
  verify binary size limits

Each teammate reports pass/fail with details. All must pass for
release to proceed.
```

**Quality gate**: `TaskCompleted` hook for each release check requires explicit PASS/FAIL verdict. Lead blocks release if any teammate reports FAIL.

---

### Recipe 5: Research Sprint

**When**: Evaluating new technology (new backend, new safety approach, new platform)
**Why**: Research benefits most from parallel exploration with synthesis.

```
Create an agent team to research [topic]. Spawn researchers:
- Technical researcher: Evaluate implementation feasibility, API
  surface, performance characteristics
- Ecosystem researcher: Check community adoption, maintenance status,
  license compatibility, alternatives
- Integration researcher: Prototype integration points with caro's
  existing architecture

Require plan approval so I can steer the research direction.
Have researchers share findings and challenge assumptions.
Produce a recommendation document.
```

**Output**: Research findings go to `docs/research/[topic].md` or feed into spec-kitty's Phase 0 research artifacts.

## Configuration Changes

### 1. Enable Agent Teams (settings.json)

```json
{
  "env": {
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
  }
}
```

### 2. New Hooks

**TeammateIdle hook** - Enforce completion standards:
```bash
# .claude/hooks/teammate-idle-check.sh
# Exit code 2 = send feedback and keep teammate working
# Check: did the teammate run tests? produce deliverables?
```

**TaskCompleted hook** - Quality gates:
```bash
# .claude/hooks/task-completed-gate.sh
# Exit code 2 = prevent completion, send feedback
# Check: tests passing? linting clean? files in scope?
```

### 3. Teammate Display Mode

Recommended default for our workflow:
```json
{
  "teammateMode": "in-process"
}
```

Use `in-process` since caro development happens on remote/cloud instances where tmux may not be available. Operators who run locally with tmux can override to `"auto"`.

## Integration with Existing Infrastructure

### Hooks

| Existing Hook | Agent Teams Impact |
|---------------|-------------------|
| `block-main-commits.sh` | Teammates inherit - all work on feature branches |
| `worktree-protection.sh` | Teammates inherit - worktrees protected |
| `quick-actions-reminder.sh` | Only applies to lead (user-facing) |
| `session-start-continuity.sh` | Each teammate loads project context normally |

New hooks to add:
- `TeammateIdle` - Enforce deliverable standards
- `TaskCompleted` - Quality gates per recipe

### Spec-Kitty

Agent Teams enhance spec-kitty at the **implementation phase**:
```
specify → clarify → plan → research → tasks → [AGENT TEAM] implement → review → accept → merge
                                                    ↑
                                          Parallel WP execution
```

The `tasks` phase already produces independent work packages. Agent Teams execute them in parallel instead of sequentially.

### Agent Profiles

The existing `agent-profiles.yaml` maps to teammate specialization:
- `rust` profile → Implementation teammates
- `security` profile → Security review teammate
- `docs` profile → Documentation teammate
- `devops` profile → Release/CI teammate
- `ai` profile → Backend/inference teammate

### CLAUDE.md

Teammates automatically read `CLAUDE.md` and load all skills. The existing model selection hints apply:
- Implementation teammates: Sonnet (cost-effective for code generation)
- Review teammates: Sonnet (sufficient for analysis)
- Architecture/safety teammates: Opus (complex reasoning)

### Context Management

Each teammate has its own context window. The existing continuity system (`create_handoff`, `resume_handoff`) operates per-session. Teams are ephemeral - they exist for a task and are cleaned up afterward. Long-running state stays in the ledger/handoff system.

## Phased Rollout

### Phase 1: Read-Only Teams (Research & Review)

**Goal**: Build familiarity without risk of file conflicts.

1. Enable `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` in settings
2. Use Recipe 1 (Code Review) on next 3 PRs
3. Use Recipe 5 (Research) for next technical evaluation
4. Document token usage and quality of results
5. Iterate on prompts based on experience

**Success criteria**: Reviews catch issues that single-reviewer missed. Research produces actionable recommendations.

### Phase 2: Quality-Gated Implementation

**Goal**: Use teams for parallel coding with safety nets.

1. Add `TeammateIdle` and `TaskCompleted` hooks
2. Use Recipe 2 (Spec-Kitty Implementation) for next feature with 3+ independent WPs
3. Require plan approval for all teammates
4. Lead runs integration tests after all WPs complete
5. Compare quality and token cost vs sequential implementation

**Success criteria**: All WPs integrate cleanly. No file conflicts. Tests pass on first integration.

### Phase 3: Full Integration

**Goal**: Teams as a standard tool in the development workflow.

1. Add team recipes to `/caro.feature` workflow routing
2. Create `/caro.team` skill for quick team creation
3. Use Recipe 3 (Debugging) for next hard bug
4. Use Recipe 4 (Release) for next release cycle
5. Update CLAUDE.md with team usage guidance

**Success criteria**: Teams feel natural as part of the workflow. Token costs justified by quality gains.

## Token Cost Considerations

Agent Teams are expensive. Each teammate is a full Claude session.

| Recipe | Teammates | Estimated Token Multiplier | Justification |
|--------|-----------|---------------------------|---------------|
| Code Review | 3 | ~4x (3 + lead) | Catches more issues; worth it for safety-critical PRs |
| Implementation | 2-4 | ~3-5x | Parallelism gains; only for truly independent WPs |
| Debugging | 4 | ~5x | Faster convergence; only for hard bugs |
| Release | 4 | ~5x | Parallelism is pure time savings |
| Research | 3 | ~4x | Breadth of exploration; always worth it |

**Cost control**:
- Use Sonnet for teammates where possible (cheaper than Opus)
- Reserve Opus teammates for safety audits and architecture decisions
- Don't use teams for tasks that a single session + subagents can handle
- Monitor token usage in Phase 1 before expanding

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| File conflicts between teammates | Each teammate gets explicit file scope in spawn prompt |
| Runaway token usage | Monitor per-team; set iteration limits |
| Lead implements instead of delegating | Use delegate mode (Shift+Tab) |
| Teammates stopping on errors | Lead monitors; spawns replacements if needed |
| Feature is experimental / may change | Phase 1 is read-only, low risk if feature changes |
| No session resumption for teammates | Teams are ephemeral by design; long-term state in handoffs |

## Alternatives Considered

1. **Stay with subagents only**: Lower cost, but no inter-agent communication. Can't do adversarial review or coordinated parallel implementation.

2. **Manual parallel sessions with worktrees**: Already in use. Works but has no coordination layer. Agent Teams add structured task management and messaging.

3. **Wait for GA release**: The feature may change, but Phase 1 (read-only) is low-risk and builds valuable experience.

## Decision Outcome

Proceed with **phased rollout** starting with Phase 1 (read-only teams). Enable the experimental flag, create the team recipes skill, and add quality gate hooks. Evaluate after 3-5 team sessions before moving to Phase 2.

## References

- [Agent Teams Documentation](https://code.claude.com/docs/en/agent-teams)
- [Subagents Documentation](https://code.claude.com/docs/en/sub-agents)
- [Hooks Documentation](https://code.claude.com/docs/en/hooks)
