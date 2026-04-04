---
title: Caro Bridge — Agent Integration Guide
created: 2026-04-04T00:00:00Z
modified: 2026-04-04T00:00:00Z
tags: [caro, bridge, agents, integration]
order: 1
---

# Caro Bridge — Agent Integration Guide

This document describes how Cabinet agents interact with the Caro development environment. All agents execute Claude Code sessions directly in the Caro repository at `/home/user/caro`.

## Architecture

```
Cabinet (AI Company OS)          Caro (Product Repo)
┌──────────────────────┐        ┌──────────────────────┐
│ Agent Personas       │        │ .claude/agents/ (27)  │
│ Cron Scheduler       │───────>│ .claude/skills/ (26)  │
│ Web Dashboard        │        │ .claude/commands/ (35) │
│ Team Channels        │        │ Automation Orchestrator│
└──────────────────────┘        └──────────────────────┘
         │                                │
         └────── Claude Code Sessions ────┘
```

## How Agents Execute Work

1. Cabinet's cron daemon triggers an agent at its scheduled heartbeat
2. The daemon spawns a Claude Code session with the agent's persona as system prompt
3. Claude Code runs in the Caro repo (`/home/user/caro`) with full tool access
4. The agent uses Caro's existing skills, commands, and automation
5. Results are written back to Cabinet's data directory and/or the Caro repo

## Agent-to-Skill Mapping

### Engineering Agents

| Cabinet Agent | Caro Skills/Commands | Schedule |
|---|---|---|
| **dev-lead** | `/spec-kitty.implement`, `/pr-management-loop`, 27 Claude agents | Weekdays 9:30 AM |
| **qa-engineer** | `/qa-automation-loop`, `/visual-regression-test`, `cargo test` | Weekdays 11 AM |
| **release-engineer** | `/caro.release.prepare`, `.version`, `.security`, `.publish`, `.verify` | Wed + Fri 2 PM |

### Product Agents

| Cabinet Agent | Caro Skills/Commands | Schedule |
|---|---|---|
| **product-manager** | `/caro.feature`, `/spec-kitty.specify`, `/spec-kitty.research` | Weekdays 10 AM |

### Marketing Agents

| Cabinet Agent | Caro Skills/Commands | Schedule |
|---|---|---|
| **growth-marketer** | `/social-queue`, `/idea-sourcing-loop` | Weekdays 8 AM |
| **content-writer** | `ai-marketing-engineering` skill, `/social-queue` | Mon/Wed/Fri 9 AM |
| **community-manager** | GitHub MCP tools, issue triage | Weekdays 10 AM |

### Leadership

| Cabinet Agent | Caro Skills/Commands | Schedule |
|---|---|---|
| **ceo** | `/caro.roadmap`, coordinates all agents | Weekdays 9 AM |

## Caro Automation Packs (Existing)

Cabinet agents leverage Caro's pre-built automation:

### Technical Pack
- `/qa-automation-loop` — Daily 9 AM unbiased beta testing
- `/visual-regression-test` — Nightly 2 AM screenshot comparison
- `/caro.sync` — Daily 6 PM content synchronization

### Content Pack
- `/idea-sourcing-loop` — Daily 8 AM idea sourcing from HN, Reddit
- `/social-queue` — Continuous social posts with approval

### Management Pack
- `/pr-management-loop` — Every 4 hours PR review and triage

## Communication Channels

| Channel | Purpose | Agents |
|---|---|---|
| `#general` | All-hands, daily CEO updates | All |
| `#engineering` | Dev, QA, releases | dev-lead, qa-engineer, release-engineer |
| `#marketing` | Content, growth, community | growth-marketer, content-writer, community-manager |
| `#product` | Features, roadmap | ceo, product-manager |

## Wave 2 Agents (Planned)

These agents will be added in future waves:

- **roadmap-planner** — Maintains ROADMAP.md, tracks milestones
- **beta-test-coordinator** — Runs structured beta test cycles
- **docs-manager** — Maintains documentation site
- **ai-researcher** — Evaluates new LLM models and backends
- **devrel-advocate** — Developer relations and integration guides
- **fundraising-lead** — Pitch materials and investor relations

## Wave 3 Agents (Planned)

- **website-manager** — Website content and analytics
- **vc-relations** — VC research and contact management
- **upwork-manager** — Contractor management
- **art-director** — Brand assets and design direction
- **agent-scout** — AI tool discovery and evaluation
- **integrations-architect** — System integration design
