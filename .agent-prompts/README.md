# cmdai Sub-Agent System

## Overview

This directory contains master prompts for specialized sub-agents that work on different phases of the cmdai MVP roadmap. Each agent has focused expertise and can work independently while coordinating with other agents.

## Agent Architecture

```
Main Coordinator (You)
├── Phase 1 Agent: Model Inference Engineer
├── Phase 2 Agent: UX/Branding Specialist
├── Phase 3 Agent: Distribution Engineer
├── Phase 4 Agent: Documentation Lead
├── Phase 5 Agent: QA & Testing Engineer
└── Cross-Cutting Agents:
    ├── Technical PM / Task Coordinator
    ├── Code Reviewer / Quality Guardian
    └── Community Manager
```

## How to Use Sub-Agents

### Launching a Sub-Agent

Use the Task tool with the appropriate agent master prompt:

```bash
# Example: Launch Phase 1 agent
Task tool with:
  subagent_type: "general-purpose" or appropriate specialized type
  prompt: [Copy content from .agent-prompts/phase-1-inference.md]
```

### When to Use Which Agent

**Phase 1 Agent (Model Inference Engineer)**:
- Implementing llama-cpp-rs or candle integration
- Debugging inference issues
- Optimizing model performance
- Prompt engineering

**Phase 2 Agent (UX/Branding Specialist)**:
- Designing first-run experience
- Creating branding assets
- Writing error messages
- Improving CLI output formatting

**Phase 3 Agent (Distribution Engineer)**:
- Setting up cross-platform builds
- Creating package manager formulas
- Configuring CI/CD pipelines
- Binary optimization

**Phase 4 Agent (Documentation Lead)**:
- Writing user guides
- Creating API documentation
- Recording demos
- Community setup

**Phase 5 Agent (QA & Testing Engineer)**:
- Writing test suites
- Running security audits
- Performance benchmarking
- Beta program management

**Technical PM / Task Coordinator**:
- Breaking down features into tasks
- Tracking progress across phases
- Identifying blockers
- Coordinating between agents

**Code Reviewer / Quality Guardian**:
- Reviewing PRs from all agents
- Ensuring code quality standards
- Catching regressions
- Architecture consistency

**Community Manager**:
- Managing GitHub Discussions
- Facilitating voting
- Onboarding contributors
- Communication

## Agent Coordination Protocol

### 1. Independence
Each agent works independently on their phase/tasks without needing constant supervision.

### 2. Coordination Points
Agents coordinate at milestone boundaries:
- Phase 1 → Phase 2: Model inference complete
- Phase 2 → Phase 3: UX assets ready
- Phase 3 → Phase 4: Binaries built
- Phase 4 → Phase 5: Docs complete

### 3. Escalation
Agents escalate to Main Coordinator when:
- Blocked by another phase
- Major architectural decision needed
- Scope clarification required
- Timeline slippage > 2 days

### 4. Communication
Agents communicate via:
- GitHub Issues (for blockers)
- GitHub Discussions (for questions)
- PRs (for code reviews)
- This README (for status updates)

## Agent Status Board

Track which agents are active and their current tasks:

| Agent | Status | Current Task | Completion % | Blockers |
|-------|--------|--------------|--------------|----------|
| Phase 1 | 🔴 Not Started | - | 0% | - |
| Phase 2 | ⚪ Waiting | Waiting on Phase 1 | 0% | Phase 1 |
| Phase 3 | ⚪ Waiting | Waiting on Phase 2 | 0% | Phase 2 |
| Phase 4 | ⚪ Waiting | Waiting on Phase 3 | 0% | Phase 3 |
| Phase 5 | ⚪ Waiting | Waiting on Phase 4 | 0% | Phase 4 |
| Tech PM | 🟢 Active | Task breakdown | 100% | - |
| Code Review | 🟡 Standby | Ready for PRs | 0% | - |
| Community | 🟢 Active | Voting setup | 20% | - |

**Legend**: 🔴 Not Started | ⚪ Waiting | 🟡 Standby | 🟢 Active | ✅ Complete

## Parallel Work Opportunities

Some agents can work in parallel:

### Week 1-4 (Phase 1 in progress)
```
Phase 1 Agent: Implementing inference (critical path)
Phase 2 Agent: Design branding, write error messages (parallel)
Phase 3 Agent: Research cross-compilation (parallel)
Phase 4 Agent: Start documentation structure (parallel)
```

### Week 5-6 (Phase 2 in progress)
```
Phase 2 Agent: Polish UX (critical path)
Phase 3 Agent: Set up CI/CD (parallel)
Phase 4 Agent: Write user guides (parallel)
Phase 5 Agent: Create test plan (parallel)
```

## Agent Master Prompts

Each agent has a detailed master prompt in this directory:

1. `phase-1-inference.md` - Model Inference Engineer
2. `phase-2-ux.md` - UX/Branding Specialist
3. `phase-3-distribution.md` - Distribution Engineer
4. `phase-4-documentation.md` - Documentation Lead
5. `phase-5-testing.md` - QA & Testing Engineer
6. `technical-pm.md` - Technical PM / Task Coordinator
7. `code-reviewer.md` - Code Reviewer / Quality Guardian
8. `community-manager.md` - Community Manager

## Quick Start Guide

### For Main Coordinator

1. **Choose which phase to start**: Usually Phase 1 (critical path)
2. **Launch the appropriate agent**: Use Task tool with master prompt
3. **Monitor progress**: Check GitHub Projects board
4. **Coordinate handoffs**: Ensure agents communicate at milestones

### For Sub-Agents

1. **Read your master prompt**: Understand your role and deliverables
2. **Check dependencies**: Are you blocked by another phase?
3. **Review task list**: See PHASE_X_TASKS.md for your tasks
4. **Execute independently**: Work through tasks autonomously
5. **Report progress**: Update status board daily
6. **Escalate blockers**: Alert Main Coordinator if stuck

## Success Metrics

Each agent is measured on:
- **Velocity**: Tasks completed per week
- **Quality**: Code review pass rate
- **Communication**: Updates provided on time
- **Collaboration**: Smooth handoffs to other agents
- **Autonomy**: Minimal escalations needed

## Version History

- v1.0 (2025-11-19): Initial agent system created
- Future: Will update as agents are refined

---

**Remember**: The goal is parallel execution while maintaining quality. Each agent is trusted to do excellent work in their domain.
