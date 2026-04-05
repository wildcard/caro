# Deep Research: Hwee-Boon Yar's Agentic Coding Workflow

> Research compiled from 16 blog posts at hboon.com (2025-2026).
> Use this document as a reference for designing skills, workflows, and CLAUDE.md improvements.

## Pages Read

| # | Post Title | URL | Date |
|---|-----------|-----|------|
| 1 | Using tmux with Claude Code | hboon.com/using-tmux-with-claude-code/ | Nov 28, 2025 |
| 2 | My Complete Agentic Coding Setup and Tech Stack | hboon.com/my-complete-agentic-coding-setup-and-tech-stack/ | Feb 21, 2026 |
| 3 | How I Use Claude Code | hboon.com/how-i-use-claude-code/ | mid-2025 |
| 4 | How I Set Up Droid, Claude Code, and Codex with Shared Configuration | hboon.com/how-i-set-up-droid-claude-code-and-codex-with-shared-configuration/ | Jan 24, 2026 |
| 5 | Let Your Coding Agent Finish for You | hboon.com/let-your-coding-agent-finish-for-you/ | Feb 20, 2026 |
| 6 | How I Write and Maintain AGENTS.md for My Coding Agents | hboon.com/how-i-write-and-maintain-agents-md-for-my-coding-agents/ | Mar 2, 2026 |
| 7 | A Lighter Way to Review and Fix Your Coding Agent's Work | hboon.com/a-lighter-way-to-review-and-fix-your-coding-agent-s-work/ | Feb 15, 2026 |
| 8 | Using a Second LLM to Review Your Coding Agent's Work | hboon.com/using-a-second-llm-to-review-your-coding-agent-s-work/ | Feb 2, 2026 |
| 9 | Build a Spec Skill for Your Coding Agent | hboon.com/build-a-spec-skill-for-your-coding-agent/ | Feb 13, 2026 |
| 10 | How to Use Coding Agents While You Are Still Learning | hboon.com/how-to-use-coding-agents-while-you-are-still-learning/ | Mar 5, 2026 |
| 11 | Using reMarkable and Claude Code to Triage My wip.md | hboon.com/using-remarkable-and-claude-code-to-triage-my-wip-md/ | Dec 1, 2025 |
| 12 | Automatically Copy macOS Screenshot Path for Claude Code | hboon.com/automatically-copy-macos-screenshot-path-for-claude-code/ | Oct 16, 2025 |
| 13 | Using the Skill-Creator Skill to Improve Your Existing Skills | hboon.com/using-the-skill-creator-skill-to-improve-your-existing-skills/ | Mar 3, 2026 |
| 14 | Auto-Renaming tmux Windows for AI Coding Agents | hboon.com/auto-renaming-tmux-windows-for-ai-coding-agents/ | ~2026 |
| 15 | How I Use tmux | hboon.com/how-i-use-tmux/ | earlier post |
| 16 | Using z.ai with Claude Code for Cheaper | hboon.com/using-z-ai-with-claude-code-for-cheaper/ | Sep 7, 2025 |

---

## Executive Summary

Hwee-Boon Yar has built an **agent-native development environment** — not just "a person using AI assistants," but a small operating system for coder agents. The key components are:

- **Shared persistent memory** (`AGENTS.md`) across all agents
- **Reusable composable skills** (markdown procedures)
- **Terminal orchestration** (tmux as the control plane)
- **Execution isolation** (Docker for YOLO mode)
- **Model specialization** (Droid/Claude Code for generation, Codex for review)
- **Agent-triggerable deployment** (Kamal 2 + Hetzner)
- **Staged autonomy** (interactive → take-over → YOLO, matched to task uncertainty)

His core insight: **minimize per-session prompting, maximize persistent operational context**. The agent should start each session already knowing the repo, commands, conventions, and deployment rules.

---

## 1. AGENTS.md — The Core Control Plane

**Source:** Posts #2, #6

AGENTS.md is what he calls "the single most effective thing I do for coding agent quality." It is read automatically at session start and stays in context throughout.

### What goes in:
- **Exact dev/test/lint/build/deploy commands** — "The agent needs these constantly and will guess wrong without them"
- **Architecture overview** — project structure, tech stack, path aliases, directory organization
- **Coding conventions** — function styles, TypeScript patterns, bracket rules, commenting philosophy
- **Critical prohibitions** — "never generate migrations unless explicitly told," "never commit without running formatters," "never modify reference directories"
- **Git commit style** — atomic commits, message format, metadata attribution
- **Common task hints** — server locations, live-reload reminders, skill trigger phrases
- **Workflow integration** — tmux window mappings, skill references, notification commands

### What stays out:
- Extended architectural rationale
- Exhaustive command lists (only frequent ones)
- Human-oriented documentation
- **"If a section isn't changing agent behavior, it doesn't belong"**

### Two-level structure:
- **Global** (`~/.config/coding-agents/`) — workflow habits, tool preferences, notification commands
- **Project-level** (repo root) — architecture, dev commands, coding style, project-specific rules

### Maintenance strategy:
- **Reactive:** add rules when the agent makes mistakes — convert failures into permanent guardrails
- **Reflect skill:** periodically ask the agent to review its session and suggest AGENTS.md improvements
- **Model sensitivity:** recalibrate after major model updates; newer models respond differently to directives
- **File hygiene:** treat as code; bloated files waste context tokens

---

## 2. Skills — Composable Reusable Procedures

**Source:** Posts #2, #5, #7, #8, #9, #13

Skills are markdown files that encode reusable operational procedures.

### Core skills:
| Skill | What it does |
|-------|-------------|
| **commit** | Stages changes, writes concise commit messages |
| **review+fix** | Iterative code review and cleanup loop |
| **review-dirty** | Sends uncommitted changes to Codex for external review |
| **review-plus-fix-relentlessly** | Multi-cycle review loop until clean |
| **spec / interview-spec** | Structured interview → specification file |
| **take-over** | Chains: review+fix → log task → commit → deploy → submit URLs → exit |
| **reflect-agents.md-file** | Agent suggests AGENTS.md improvements from session learnings |
| **skill-creator** | Meta-skill that evaluates and improves other skills |

### Design pattern:
- Short, high-frequency hints go in AGENTS.md
- Long, task-specific procedures go in skills
- AGENTS.md contains brief references to nudge the agent toward the right skill
- Skills compose: `take-over` calls `review+fix`, then `commit`, then `deploy`

---

## 3. Three Autonomy Modes

**Source:** Posts #2, #3, #5

### Mode 1: YOLO (Docker-isolated autonomy)
- Runs `claude --dangerously-skip-permissions` inside a Docker container
- Uses a custom `/yolo` command referencing a `plan.md` with simplified TODOs
- Best for **simple, well-scoped tasks**

### Mode 2: Interactive
- Human stays in the loop
- Used for **larger features, real-time output review, visual tweaking**
- This is where architectural and design decisions happen

### Mode 3: Take-over (staged handoff)
- After ~80-90% interactive work with key decisions made, say **"take over"**
- Agent runs the chain: review+fix → log task → commit → deploy → submit URLs → exit
- Agent retains full conversation context from the interactive phase
- **"YOLO mode requires full upfront specification. Complex work demands iterative refinement. Take-over preserves interactive flexibility during exploration while enabling autonomous execution during mechanical work."**

---

## 4. Multi-Model Review Loop

**Source:** Posts #2, #7, #8

### Architecture:
- **Primary agent** (Claude Opus): generates code
- **Reviewer** (secondary model): provides independent assessment

### Why two models:
- "Different LLMs think differently. When one gets stuck, it tends to bang its head against the wall."
- The reviewer sees the code fresh and catches oversights or unnecessary complexity

### review-plus-fix-relentlessly:
- Creates an iterative cycle: generate → review → fix → re-review
- Terminates when review produces no additional flagged issues
- **Typical convergence: 2-3 cycles**

---

## 5. tmux as the Orchestration Layer

**Source:** Posts #1, #14, #15

### Buffer capture:
```bash
tmux capture-pane -t 0 -p -S -10000 | v
```
- `-t 0`: targets pane 0
- `-p`: outputs to stdout
- `-S -10000`: retrieves 10,000 lines of history

### Copy mode for reading agent output:
| Action | Key |
|--------|-----|
| Page up | `ctrl-u` |
| Page down | `ctrl-d` |
| Search forward/backward | `/` / `?` |
| Next/prev match | `n` / `N` |
| Exit | `ctrl-c` |

### Passthrough binding:
```
bind o send-keys C-o
```
Sends ctrl-o to agents instead of tmux intercepting it.

---

## Deeper Patterns and Principles

1. **Persistent context beats session prompting** — push commands and conventions into AGENTS.md and skills
2. **Compose behaviors, don't monolithically prompt** — skills are small and chainable
3. **Separate judgment from execution** — human owns architecture, agent owns implementation
4. **Use heterogeneous models for quality control** — reviewer/generator role separation
5. **Turn mistakes into system updates** — update AGENTS.md or create a skill when the agent gets it wrong
6. **Keep the environment machine-readable** — exact commands, not prose
7. **Match autonomy to task uncertainty** — YOLO for simple, interactive for complex, take-over for mechanical cleanup

---

## Priority Ordering: What to Copy First

1. **Create a strong AGENTS.md/CLAUDE.md** with exact commands, architecture, conventions, and hard prohibitions
2. **Build a small skills library** for recurring actions: spec, review+fix, commit, deploy, reflect
3. **Add a second-model review loop** — one of the clearest quality multipliers
4. **Adopt staged autonomy** (interactive → take-over → YOLO) rather than binary manual/autonomous
5. **Run agents in tmux** and encode terminal topology in your instructions
6. **Share configuration across agents** via symlinks so model switching is frictionless
