# Mechanism 4: Channel Selection

## Purpose
Select the optimal delivery channel for each outreach message based on user context, message type, and channel performance history.

## Available Channels

| Channel | Availability | Best For | Frequency Limit |
|---------|-------------|----------|-----------------|
| CLI | Always (user has CARO installed) | Contextual recognition, quick interactions | 2 per user per week |
| Email | Only if user claimed account with email | Milestones, invitations | Milestone events only |
| Web | Only if user visits hub.caro.sh | Badges, leaderboards, public recognition | Continuous (passive) |

## Decision Matrix

| Message Type | CLI Available | Email Available | Web Active | Decision |
|-------------|---------------|-----------------|------------|----------|
| Recognition | Yes, under limit | - | - | **CLI** |
| Recognition | Yes, over limit | - | - | **Web** badge |
| Recognition | - | Yes | - | **Email** (if milestone-worthy) |
| Amplification | Yes | Yes | - | **Email** (needs longer format) |
| Amplification | Yes | No | - | **CLI** |
| Invitation | - | Yes | - | **Email** (always for invitations) |
| Invitation | Yes | No | - | **CLI** (fallback only) |
| Direction | Yes, under limit | - | - | **CLI** |
| Direction | Yes, over limit | Yes | - | **Email** |
| Milestone | - | Yes | - | **Email** (always) |
| Milestone | Yes | No | - | **CLI** |

## Multi-Channel Rules

Some events warrant multi-channel delivery:
- **Founder invitation**: Email (primary) + Web badge
- **Major milestone** (1000 runs): Email + CLI + Web badge
- **Tier promotion**: CLI notification + Web badge update

Never duplicate the same message across channels. Each channel gets a channel-appropriate version.

## Frequency Enforcement

```yaml
limits:
  cli:
    max_per_user_per_week: 2
    min_gap_between_messages: 72h  # 3 days
  email:
    milestones_only: true
    max_per_user_per_month: 2
  web:
    no_limit: true  # Passive display, user chooses to view
```

## Response-Based Learning

Track which channels each user responds to:

```
user_channel_preference:
  user_id: string
  cli_response_rate: 0.0 - 1.0
  email_open_rate: 0.0 - 1.0
  web_engagement_rate: 0.0 - 1.0
```

If a user consistently ignores CLI messages but opens emails, prefer email for future outreach (within limits).

## Fallback Chain

If primary channel is unavailable or over limit:
1. CLI -> Web badge (always available)
2. Email -> CLI (always available)
3. Web -> CLI (always available)

If all channels are over limit: **skip this user for this cycle**. Never force a message.
