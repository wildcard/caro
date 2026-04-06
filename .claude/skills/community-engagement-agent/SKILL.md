---
name: "community-engagement-agent"
description: "Behavior-aware community engagement system that identifies high-value CARO contributors, drafts personalized outreach, and converts active users into long-term stakeholders. Use when running daily engagement cycles, drafting contributor outreach, managing Founder tier invitations, analyzing engagement metrics, identifying community gaps, or designing retention workflows. Provides 7 specialized agents: insight gathering, contribution scoring, personalized outreach, channel selection, founder curation, feedback tracking, and direction recruiting."
version: "1.0.0"
allowed-tools: "Bash, Read, Write, Edit, Grep, Glob, Task, WebFetch, WebSearch"
license: "AGPL-3.0"
---

# Community Engagement Agent

A behavior-aware engagement system that reinforces high-value contributors and converts active users into long-term stakeholders. This is the "Reinforce" phase of the CARO growth flywheel.

## Core Philosophy

> **Reinforce behavior, don't manufacture engagement.**

The best community engagement feels earned, not automated. Every message references a specific action the user took. Every recognition is tied to observable impact. The system detects who is creating value and responds with recognition, amplification, and direction.

This is NOT marketing. This is:

- A **growth engine** that identifies and reinforces valuable behavior
- A **reputation system** that converts usage into identity
- A **proto-economy layer** that converts community into ownership

## When to Use This Skill

Activate this skill when:
- Running daily/weekly engagement cycles
- Drafting personalized contributor outreach
- Managing Founder tier invitations and progression
- Analyzing engagement response rates and channel performance
- Identifying community gaps and recruiting contributors to fill them
- Designing retention and re-engagement workflows
- Reviewing contribution leaderboards and scoring

**Example Triggers:**
- "Run the daily engagement cycle"
- "Who are our top contributors this week?"
- "Draft outreach for users whose recipes are being reused"
- "Which categories need more recipes? Find contributors to recruit"
- "Review the founder tier candidate list"
- "What's our engagement response rate by channel?"

## The 7 Engineering Mechanisms

| # | Mechanism | Agent Tag | Purpose |
|---|-----------|-----------|---------|
| 1 | Insight Gathering | `@insight-agent` | Query Hub API for contributor data, trending content, usage patterns |
| 2 | Contribution Scoring | `@scoring-agent` | Compute weighted scores, identify daily/weekly leaders |
| 3 | Personalized Outreach | `@outreach-agent` | Draft messages referencing specific user contributions |
| 4 | Channel Selection | `@channel-agent` | Pick optimal channel (CLI/email/web) per user and message type |
| 5 | Founder Curation | `@founder-agent` | Manage Founder tier invitations, track progression |
| 6 | Feedback Tracking | `@feedback-agent` | Track response rates, A/B test message variants |
| 7 | Direction Recruiting | `@direction-agent` | Identify community gaps and recruit contributors to fill them |

## Quick Start

### 1. Run Daily Engagement Cycle

The default workflow that runs the full pipeline:

```
Run the daily community engagement cycle
→ Loads master-prompt.md
→ Chains: @insight-agent → @scoring-agent → @outreach-agent → @channel-agent
→ Outputs: drafts in engagement queue for human approval
```

### 2. Spawn a Specific Agent

For focused tasks, load the relevant agent:

```
Who are our top 10 contributors this week and what did they build?
→ Load @insight-agent + @scoring-agent from references/agent-cards.md
```

### 3. Manage Founder Tier

For invitation and progression workflows:

```
Review founder tier candidates and draft invitations for qualified users
→ Load @founder-agent from references/agent-cards.md
→ Uses @scoring-agent data for eligibility
```

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Hub API     │────▶│  @insight    │────▶│  @scoring    │
│  (data)      │     │  (gather)    │     │  (rank)      │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                     ┌──────────────┐     ┌───────▼──────┐
                     │  @channel    │◀────│  @outreach   │
                     │  (route)     │     │  (draft)     │
                     └──────┬───────┘     └──────────────┘
                            │
               ┌────────────┼────────────┐
               ▼            ▼            ▼
          ┌────────┐  ┌──────────┐  ┌────────┐
          │  CLI   │  │  Email   │  │  Web   │
          │  msg   │  │  msg     │  │  badge │
          └────────┘  └──────────┘  └────────┘
                            │
                     ┌──────▼───────┐
                     │  @feedback   │
                     │  (track)     │
                     └──────────────┘
```

### Supporting Agents

```
@founder-agent  ← Uses @scoring-agent data for eligibility decisions
@direction-agent ← Uses @insight-agent data for gap detection
```

## Phased Autonomy Model

### Phase 1: Draft Only (Current)

- Agent generates outreach drafts
- **All messages require human approval** before sending
- `approval_required: true` in config
- Human reviews, edits, approves/rejects each message
- Learning: track which drafts get approved vs. edited vs. rejected

### Phase 2: Semi-Automated

- Recognition messages (low risk) can auto-send
- Invitation and Direction messages still require approval
- Email channel unlocked (milestones only)
- Founder tier invitations activated

### Phase 3: Full Automation

- All message types can auto-send within configured limits
- Human reviews weekly summary instead of individual messages
- Agent adjusts strategy based on feedback loop data
- Escalation to human only for edge cases

## Contribution Scoring

The scoring formula rewards quality and impact, not raw activity:

```
Score = (commands_reused × 3) + (remixes × 2) + (original_creations × 1) + (safety_saves × 2)
```

### Why These Weights

| Signal | Weight | Rationale |
|--------|--------|-----------|
| Commands reused | ×3 | Highest signal of value - others found it useful |
| Remixes | ×2 | Shows ecosystem participation and building on others' work |
| Original creations | ×1 | Base contribution, but volume alone isn't valuable |
| Safety saves | ×2 | Protecting others from dangerous commands is high-value |

### Anti-Gaming Measures

- Score uses **reuse/remix signals**, not raw creation count
- Score **decays over time** (half-life: 30 days) - recent activity matters more
- Burst detection: sudden spikes in creation without corresponding reuse are flagged

### Tier Thresholds

| Tier | Score Range | Meaning |
|------|-------------|---------|
| Explorer | 0-9 | New contributor, still finding their way |
| Builder | 10-49 | Active contributor, creating useful content |
| Leader | 50-199 | Significant contributor, recipes being widely reused |
| Founder-eligible | 200+ | Sustained high-value contribution over 30+ days |

## Message Types

### 1. Recognition
> "Your FFmpeg recipe was reused by 12 people this week."

Acknowledges specific impact. Never generic praise.

### 2. Amplification
> "Want to feature your batch image converter on the homepage?"

Offers visibility for high-quality work.

### 3. Invitation
> "You're exactly the kind of builder we want in our Founding Builders group."

Exclusive access, earned through contribution.

### 4. Direction
> "People are searching for PDF merge recipes but finding nothing. You've built similar tools - want to contribute?"

Recruits contributors to fill specific gaps.

## Channel Guidelines

| Channel | Best For | Frequency Limit | Voice |
|---------|----------|-----------------|-------|
| CLI | Contextual recognition, in-tool engagement | 2/user/week | Concise, earned, native |
| Email | Milestones, founder invitations | Milestones only | Personal, milestone-worthy |
| Web | Badges, leaderboards, public recognition | Continuous | Visual, social proof |

## Integration Points

- **Hub API**: Leaderboard, trending, usage data (from 008 spec)
- **Social Queue**: Cross-post engagement stories (`.claude/commands/social-queue.md`)
- **Identity System**: Machine fingerprint (`src/identity/`)
- **Privacy Engine**: Redaction before any data sharing (`apps/devrel/lib/privacy/redactor.ts`)
- **UserProfile**: Extends socialStats + reputation (`apps/devrel/types/user.ts`)

## Voice & Constraints

See `references/persona-spec.md` for detailed voice guidelines.

**Core Rules:**
- Every message references a **specific action** the user took
- Never generic: "Thanks for contributing!" is banned
- Token/reward talk: "future recognition + rewards" NOT "earn income"
- Founder tier: "permanent recognition" NOT transactional
- Privacy: "local identity" NOT "tracking" or "fingerprinting"

## Configuration

See `.claude/automation/config/community_engagement.yaml` for all configurable parameters.

## Reference Materials

| File | Purpose |
|------|---------|
| `references/persona-spec.md` | Voice, tone, and constraint rules |
| `references/master-prompt.md` | Orchestrator system prompt and routing |
| `references/agent-cards.md` | All 7 agent definitions with system prompts |
| `references/workflow-suite.feature` | BDD test scenarios |
| `references/mechanisms/` | Deep-dive docs for each mechanism |

## CLI Command

Use `/community-engagement` for the workflow command interface. See `.claude/commands/community-engagement.md`.
