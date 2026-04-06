# Community Engagement System - Design Requirements Specification

## 1. Overview

### 1.1 Objectives

Build a semi-automated community engagement system that:
1. Identifies high-value CARO contributors using weighted scoring
2. Drafts personalized outreach referencing specific contributions
3. Selects optimal delivery channels (CLI, email, web)
4. Manages Founder tier invitations and progression
5. Tracks engagement effectiveness and adjusts strategy

### 1.2 Semi-Automated Nature

```
┌─────────────────────────────────────────────────────────────┐
│                    AUTOMATED                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ Insight   │→│ Scoring  │→│ Drafting │→│ Channel  │   │
│  │ Gathering │  │          │  │          │  │ Selection│   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└────────────────────────────────────┬────────────────────────┘
                                     │
                              ┌──────▼──────┐
                              │   HUMAN     │
                              │   APPROVAL  │
                              │   GATE      │
                              └──────┬──────┘
                                     │
┌────────────────────────────────────▼────────────────────────┐
│                    AUTOMATED                                 │
│  ┌──────────┐  ┌──────────┐                                 │
│  │ Message   │→│ Feedback │                                 │
│  │ Delivery  │  │ Tracking │                                 │
│  └──────────┘  └──────────┘                                 │
└─────────────────────────────────────────────────────────────┘
```

The human approval gate is the critical safety layer. In Phase 1, ALL outreach requires human approval. Automation handles data gathering and drafting; humans decide what gets sent.

## 2. System Design

### 2.1 Component Architecture

```
┌─────────────────────────────────────────────────┐
│              Orchestrator (master-prompt.md)      │
│  Routes requests to appropriate agent chains     │
└──────────┬──────────────────────┬───────────────┘
           │                      │
    ┌──────▼──────┐        ┌──────▼──────┐
    │ Data Layer  │        │ Action Layer │
    │             │        │              │
    │ @insight    │        │ @outreach    │
    │ @scoring    │        │ @channel     │
    │ @feedback   │        │ @founder     │
    │             │        │ @direction   │
    └──────┬──────┘        └──────┬───────┘
           │                      │
    ┌──────▼──────────────────────▼───────┐
    │           Approval Queue            │
    │  Drafts awaiting human review       │
    └──────────────────┬──────────────────┘
                       │
    ┌──────────────────▼──────────────────┐
    │         Delivery Channels           │
    │  CLI  │  Email  │  Web (badges)     │
    └─────────────────────────────────────┘
```

### 2.2 Data Flow

```
Hub API ──→ @insight-agent ──→ Daily Snapshot (cached 1h)
                                    │
                              ┌─────▼─────┐
                              │ @scoring   │──→ Ranked Contributors
                              └─────┬─────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
              @outreach       @founder        @direction
              (drafts)        (invitations)   (recruitment)
                    │               │               │
                    └───────────────┼───────────────┘
                                    ▼
                              @channel-agent ──→ Channel Assignment
                                    │
                              Approval Queue
                                    │
                              Human Review
                                    │
                              Delivery ──→ @feedback-agent
```

### 2.3 State Management

| State | Storage | Retention |
|-------|---------|-----------|
| Daily snapshots | Cache (1h TTL) | Last 7 snapshots |
| Contributor scores | Persistent | All time (with decay) |
| Outreach drafts | Queue file | Until approved/rejected |
| Engagement history | Persistent | 90 days rolling |
| Founder pipeline | Persistent | Permanent |
| Feedback metrics | Persistent | 90 days rolling |

## 3. Workflows

### 3.1 Daily Engagement Cycle

```
1. @insight-agent: Fetch 24h snapshot from Hub API
2. @scoring-agent: Compute scores for active contributors
3. Select targets:
   a. Top 5 by daily score delta
   b. Any milestone crossers
   c. Exclude recently contacted (< 3 days)
   d. Cap at 10 total targets
4. @outreach-agent: Draft messages for each target
5. @channel-agent: Assign channels
6. Queue drafts for human approval
7. Human reviews and approves/edits/rejects
8. Approved messages delivered
9. @feedback-agent: Track responses
```

### 3.2 Founder Tier Workflow

```
1. @scoring-agent: Identify founder-eligible contributors
2. @founder-agent: Validate eligibility (score >= 200, active >= 30 days)
3. @founder-agent: Check pipeline (remaining cohort slots)
4. @outreach-agent: Draft invitation email
5. Human approval (always required for founder invitations)
6. Deliver invitation
7. Track response (accept/decline/pending)
8. If accepted: onboard to Founder program
9. If declined: log reason, cooldown 90 days
```

### 3.3 Direction Recruiting Workflow

```
1. @insight-agent: Fetch search gap analytics
2. @direction-agent: Identify critical gaps (gap_score > 10)
3. @direction-agent: Match contributors by expertise
4. @outreach-agent: Draft recruitment messages
5. @channel-agent: Select channel
6. Human approval
7. Deliver message
8. Track: did they publish a relevant recipe within 14 days?
```

### 3.4 Error Handling

| Error | Response | Escalation |
|-------|----------|------------|
| Hub API timeout | Retry 3x with backoff | Skip cycle, alert human |
| Hub API 500 | Abort cycle | Alert human operator |
| Rate limited (429) | Respect Retry-After | Queue for next cycle |
| Empty snapshot | Skip cycle (no data) | Log warning |
| All targets recently contacted | Skip cycle | Normal (no action needed) |
| Channel delivery failure | Try fallback channel | Log for investigation |

## 4. Data Schema

### 4.1 Engagement Queue Entry
```yaml
entry:
  id: ULID
  created_at: ISO8601
  status: "pending_approval" | "approved" | "rejected" | "sent" | "failed"
  
  target:
    user_id: string
    display_name: string
    tier: string
    score: number
    
  message:
    type: "recognition" | "amplification" | "invitation" | "direction"
    channel: "cli" | "email" | "web"
    content:
      cli_lines: string[]
      email_subject: string
      email_body: string
      web_badge: string
    personalization_evidence: string[]
    
  review:
    reviewed_by: string | null
    reviewed_at: ISO8601 | null
    rejection_reason: string | null
    edits_made: boolean
    
  delivery:
    sent_at: ISO8601 | null
    delivered: boolean
    response_received: boolean
    response_action: string | null
    response_at: ISO8601 | null
```

### 4.2 Contributor Score Record
```yaml
score_record:
  user_id: string
  computed_at: ISO8601
  period: "24h" | "7d" | "30d"
  
  raw_score: number
  decayed_score: number
  tier: string
  
  breakdown:
    reuse_points: number
    remix_points: number
    creation_points: number
    safety_points: number
    
  flags:
    gaming_suspect: boolean
    burst_activity: boolean
    approaching_promotion: boolean
    
  previous:
    score: number
    tier: string
    delta: number
```

## 5. Configuration

All configuration in `.claude/automation/config/community_engagement.yaml`.

See that file for full schema and defaults.

## 6. Integration Points

| System | Integration | Direction |
|--------|-------------|-----------|
| Hub API | REST endpoints for leaderboard, trending, gaps, stats | Read |
| Social Queue | Cross-post engagement stories | Write |
| Identity System | Machine fingerprint resolution | Read |
| Privacy Engine | Redaction before sharing user data | Transform |
| UserProfile | Social stats, reputation | Read/Write |
| Bluesky AT Protocol | Post engagement stories to feeds | Write (optional) |

## 7. Testing

### 7.1 BDD Scenarios
Full test scenarios in `references/workflow-suite.feature`.

### 7.2 Key Test Cases
1. Daily cycle produces correct number of drafts
2. Recently contacted users are excluded
3. Scoring formula produces expected rankings
4. Gaming detection flags suspicious patterns
5. Channel selection respects frequency limits
6. Founder eligibility validates all criteria
7. All drafts require approval in Phase 1

## 8. Security Considerations

1. **PII handling**: Never expose user machine fingerprint in messages
2. **Opt-in consent**: Users must opt-in to receive engagement messages
3. **Data minimization**: Only fetch data needed for current cycle
4. **Human gate**: All outreach reviewed before delivery
5. **Rate limiting**: Respect Hub API rate limits
6. **No financial promises**: Token/reward language restricted by persona spec
7. **Audit trail**: All engagement actions logged with timestamps
