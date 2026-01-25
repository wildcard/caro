# Social Content Queue

Semi-automated social media content queue with one-click approval workflow. Generates content from triggers (releases, ideas, events), adapts for each platform, and schedules posting after admin approval.

## Usage

```
/social-queue <command> [options]
```

## Commands

### status - Show queue status

```
/social-queue status
```

### review - Interactive review mode

```
/social-queue review
```

### approve - Approve posts

```
/social-queue approve <id|all>
```

### reject - Reject posts

```
/social-queue reject <id> --reason "reason"
```

### edit - Edit post content

```
/social-queue edit <id>
```

### schedule - Reschedule post

```
/social-queue schedule <id> --time "2026-01-15 10:00"
```

### create - Manually create post

```
/social-queue create --type <type> [--content "..."]
```

### history - Show posting history

```
/social-queue history [--days N]
```

### metrics - Show engagement metrics

```
/social-queue metrics [--period 7d|30d]
```

## Workflow

### 1. Content Triggers

Posts are automatically generated from:

| Trigger | Content Type | Platforms |
|---------|--------------|-----------|
| Release published | Release announcement | All |
| Feature merged | Feature spotlight | Twitter, LinkedIn |
| Content published | Tutorial promotion | All |
| Idea approved | Community discussion | Twitter, BlueSky |
| Manual | Custom | Configurable |

### 2. Platform Adaptation

Each platform has specific formatting:

**Twitter/X:**
- Max 280 characters
- 2-3 emojis
- Inline hashtags
- Casual tone

**LinkedIn:**
- Max 3000 characters
- Minimal emojis
- Professional tone
- End hashtags

**BlueSky:**
- Max 300 characters
- No hashtags (uses feeds)
- Conversational tone

### 3. Approval Queue

All posts require admin approval:

```
┌─────────────────────────────────────────────────────────────────┐
│                    SOCIAL QUEUE REVIEW                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  📝 Pending: 3 posts                                            │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│  Post #1: Release Announcement (v1.1.0)                         │
│                                                                  │
│  ┌─ Twitter ────────────────────────────────────────────────┐   │
│  │ 🚀 Caro v1.1.0 is here!                                  │   │
│  │                                                           │   │
│  │ ✅ Fish shell support                                    │   │
│  │ ✅ 25% faster inference                                  │   │
│  │                                                           │   │
│  │ Install: curl -sSL https://setup.caro.sh | bash          │   │
│  │ #CLI #Rust #AI                                           │   │
│  │                                                           │   │
│  │ 📅 Scheduled: Jan 13, 9:00 AM                            │   │
│  └───────────────────────────────────────────────────────────┘   │
│                                                                  │
│  [A] Approve  [E] Edit  [R] Reject  [S] Schedule  [N] Next     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4. Scheduled Posting

After approval:
- Posts are scheduled at specified times
- Staggered across platforms (30 min gaps)
- Posted via platform APIs
- Engagement tracked

### 5. Engagement Tracking

After 24h and 7d, collect:

```yaml
twitter:
  impressions: 1234
  engagements: 89
  likes: 45
  retweets: 12
  link_clicks: 34
```

## Example Session

```
> /social-queue status

Social Queue Status
═══════════════════

Queue:
  Pending approval: 3
  Scheduled: 5
  Posted today: 2

Pending Posts:
  #001 Release v1.1.0      [release_announcement]  Jan 13
  #002 Fish shell feature  [feature_spotlight]     Jan 14
  #003 Tutorial: Safety    [tutorial_promotion]    Jan 15

Run '/social-queue review' to approve posts.

> /social-queue review

Reviewing post #001: Release v1.1.0

[Twitter Preview]
🚀 Caro v1.1.0 is here!

✅ Fish shell support
✅ 25% faster inference
✅ New safety patterns

Install: curl -sSL https://setup.caro.sh | bash

#CLI #Rust #AI

Characters: 167/280 ✓
Scheduled: Jan 13, 9:00 AM PST

[A] Approve [E] Edit [R] Reject [S] Schedule [N] Next [Q] Quit
> a

✓ Approved for all platforms
  Twitter:  Jan 13, 9:00 AM
  LinkedIn: Jan 13, 9:30 AM
  BlueSky:  Jan 13, 10:00 AM

Moving to next post...
```

## Manual Post Creation

```
> /social-queue create --type feature_spotlight

Feature Spotlight Post Creator
══════════════════════════════

What feature would you like to highlight?
> Fish shell support

Brief description:
> Native fish syntax generation with autosuggestions integration

Example command (optional):
> caro "list files modified today"

Generating platform-specific content...

[Twitter]
Did you know? Caro now supports fish shell natively! 🐟

Generate commands with proper fish syntax:
`caro "list files modified today"`

Try it: https://caro.sh
#CLI #fish

[LinkedIn]
Feature Spotlight: Fish Shell Support

Caro v1.1.0 introduces native fish shell support, generating
commands with proper fish syntax and autosuggestions integration.

Example:
```
caro "list files modified today"
```

Learn more at https://caro.sh

[BlueSky]
Fish shell support just landed in Caro!

Native syntax generation for fish users.

https://caro.sh

Schedule this post? (default: tomorrow 10 AM)
> y

✓ Post created and added to queue
  ID: #004
  Status: pending_approval

Run '/social-queue review' to approve.
```

## Configuration

### Platform Configuration

```yaml
# .claude/automation/config/social_platforms.yaml
platforms:
  twitter:
    enabled: true
    max_chars: 280
    best_times: ["09:00", "12:00", "17:00"]

  linkedin:
    enabled: true
    max_chars: 3000
    format: "professional"

  bluesky:
    enabled: true
    max_chars: 300
```

### Queue Configuration

```yaml
# .claude/automation/config/social_queue.yaml
social_queue:
  approval:
    required: true
    auto_approve: false

  scheduling:
    stagger_platforms: true
    stagger_minutes: 30
    timezone: "America/Los_Angeles"

  posting:
    dry_run: false  # Set true to disable actual posting
```

## Queue File

```yaml
# .claude/automation/queues/social_queue.yaml
queue:
  - id: "post-2026-01-11-001"
    type: "release_announcement"
    status: "pending_approval"

    content:
      twitter:
        text: "..."
        scheduled_for: "2026-01-13T09:00:00Z"
      linkedin:
        text: "..."
        scheduled_for: "2026-01-13T09:30:00Z"

    approval:
      required: true
      approved_by: null

    engagement: null  # Filled after posting
```

## Integration

- **Input**: Release events, merged features, content ideas
- **Output**: Scheduled posts, engagement metrics
- **Requires**: Admin approval before posting
- **APIs**: Twitter v2, LinkedIn, BlueSky AT Protocol

## Related

- [SOCIAL_QUEUE_DRS.md](../.claude/automation/specs/SOCIAL_QUEUE_DRS.md)
- [SOCIAL_MEDIA_GUIDE.md](../../docs/devrel/SOCIAL_MEDIA_GUIDE.md)
- `/idea-sourcing-loop` - Ideas feed into content
