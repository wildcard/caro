# Social Content Queue - Design Requirements Specification

> **Document Type**: DRS
> **Version**: 1.0.0
> **Status**: Active
> **Parent**: [AUTOMATED_DEV_FLOW_DRS.md](./AUTOMATED_DEV_FLOW_DRS.md)
> **Pack**: Content (Semi-Automated)

---

## 1. Overview

The Social Content Queue is a **semi-automated** system that generates, queues, and schedules social media content across platforms, with a one-click approval workflow for admins.

### 1.1 Objectives

1. **Content Pipeline**: Automate content creation from triggers (releases, ideas, events)
2. **Multi-Platform**: Support Twitter/X, LinkedIn, BlueSky, and future platforms
3. **Admin Control**: One-click approval before any public posting
4. **Engagement Tracking**: Track performance and optimize content

### 1.2 Semi-Automated Nature

```
┌─────────────────────────────────────────────────────────────────┐
│                     AUTOMATION BOUNDARY                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   AUTOMATED                         MANUAL (Admin)               │
│   ─────────                         ──────────────               │
│   ┌───────────────┐                ┌───────────────┐            │
│   │ Content       │                │ Review Queue  │            │
│   │ Generation    │ ─────────────▶ │ One-Click     │            │
│   └───────────────┘                │ Approve/Edit  │            │
│                                    └───────┬───────┘            │
│   ┌───────────────┐                        │                    │
│   │ Platform      │ ◀─────────────────────┘                    │
│   │ Adaptation    │   (After approval)                          │
│   └───────────────┘                                             │
│                                                                  │
│   ┌───────────────┐                                             │
│   │ Scheduled     │                                             │
│   │ Posting       │                                             │
│   └───────────────┘                                             │
│                                                                  │
│   ┌───────────────┐                                             │
│   │ Engagement    │                                             │
│   │ Tracking      │                                             │
│   └───────────────┘                                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. System Design

### 2.1 Component Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     SOCIAL CONTENT QUEUE                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  TRIGGERS                                                        │
│  ────────                                                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Release  │  │ Feature  │  │ Content  │  │ Manual   │        │
│  │ Published│  │ Merged   │  │ Idea     │  │ Entry    │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
│       │             │             │             │               │
│       └─────────────┴─────────────┴─────────────┘               │
│                           │                                      │
│                           ▼                                      │
│                   ┌───────────────┐                              │
│                   │   Content     │                              │
│                   │   Generator   │                              │
│                   └───────┬───────┘                              │
│                           │                                      │
│                           ▼                                      │
│                   ┌───────────────┐                              │
│                   │   Platform    │                              │
│                   │   Adapters    │                              │
│                   └───────┬───────┘                              │
│                           │                                      │
│         ┌─────────────────┼─────────────────┐                    │
│         ▼                 ▼                 ▼                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Twitter    │  │  LinkedIn   │  │  BlueSky    │              │
│  │  Adapter    │  │  Adapter    │  │  Adapter    │              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                      │
│         └────────────────┼────────────────┘                      │
│                          │                                       │
│                          ▼                                       │
│                  ┌───────────────┐                               │
│                  │    QUEUE      │                               │
│                  │  (Pending     │                               │
│                  │   Approval)   │                               │
│                  └───────┬───────┘                               │
│                          │                                       │
│                          ▼                                       │
│            ┌──────────────────────────┐                          │
│            │     ADMIN DASHBOARD      │                          │
│            │  ┌──────────────────┐    │                          │
│            │  │ Post Preview     │    │                          │
│            │  │ Platform: X      │    │                          │
│            │  │ Schedule: 10 AM  │    │                          │
│            │  │                  │    │                          │
│            │  │ [Edit] [Approve] │    │                          │
│            │  │ [Reject] [Defer] │    │                          │
│            │  └──────────────────┘    │                          │
│            └────────────┬─────────────┘                          │
│                         │                                        │
│              ┌──────────┴──────────┐                             │
│              ▼                     ▼                             │
│       ┌───────────┐         ┌───────────┐                        │
│       │ Approved  │         │ Rejected  │                        │
│       │ → Schedule│         │ → Archive │                        │
│       └─────┬─────┘         └───────────┘                        │
│             │                                                    │
│             ▼                                                    │
│     ┌───────────────┐                                            │
│     │   Scheduler   │                                            │
│     │   (Cron-like) │                                            │
│     └───────┬───────┘                                            │
│             │                                                    │
│             ▼                                                    │
│     ┌───────────────┐                                            │
│     │   Publisher   │                                            │
│     │   (API calls) │                                            │
│     └───────┬───────┘                                            │
│             │                                                    │
│             ▼                                                    │
│     ┌───────────────┐                                            │
│     │  Engagement   │                                            │
│     │   Tracker     │                                            │
│     └───────────────┘                                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Platform Adapters

```yaml
# .claude/automation/config/social_platforms.yaml
platforms:
  twitter:
    enabled: true
    api: "twitter_api_v2"
    limits:
      max_chars: 280
      max_images: 4
      max_videos: 1
    best_times:
      - "09:00"
      - "12:00"
      - "17:00"
    hashtag_strategy: "inline"  # inline or end
    thread_support: true

  linkedin:
    enabled: true
    api: "linkedin_api"
    limits:
      max_chars: 3000
      max_images: 9
    best_times:
      - "08:00"
      - "10:00"
      - "12:00"
    format: "professional"
    hashtag_strategy: "end"

  bluesky:
    enabled: true
    api: "at_protocol"
    limits:
      max_chars: 300
      max_images: 4
    best_times:
      - "09:00"
      - "14:00"
      - "19:00"
    format: "casual"
    hashtag_strategy: "none"  # BlueSky uses feeds, not hashtags
```

---

## 3. Content Generation

### 3.1 Content Templates

```yaml
# .claude/automation/config/social_templates.yaml
templates:
  release_announcement:
    trigger: "release_published"
    priority: "high"
    platforms: ["twitter", "linkedin", "bluesky"]

    twitter:
      template: |
        🚀 Caro {version} is here!

        {highlights}

        Install: curl -sSL https://setup.caro.sh | bash

        #CLI #Rust #AI

    linkedin:
      template: |
        Excited to announce Caro {version}!

        {detailed_highlights}

        Key improvements:
        {bullet_points}

        Try it now: https://caro.sh

        #DeveloperTools #CLI #Rust #AI

    bluesky:
      template: |
        Caro {version} just dropped!

        {short_highlights}

        https://caro.sh

  feature_spotlight:
    trigger: "manual"
    priority: "medium"
    platforms: ["twitter", "linkedin"]

    twitter:
      template: |
        Did you know? {feature_description}

        {example_command}

        Try it: https://caro.sh

    linkedin:
      template: |
        Feature Spotlight: {feature_name}

        {detailed_description}

        Example:
        ```
        {example_command}
        ```

        Learn more at https://caro.sh

  community_highlight:
    trigger: "manual"
    priority: "low"
    platforms: ["twitter", "bluesky"]

    twitter:
      template: |
        Shoutout to @{contributor} for {contribution}! 🎉

        {context}

        #OpenSource #Community

    bluesky:
      template: |
        Big thanks to {contributor} for {contribution}!

        {context}

  tutorial_promotion:
    trigger: "content_published"
    priority: "medium"
    platforms: ["twitter", "linkedin", "bluesky"]

    twitter:
      template: |
        New tutorial: {title}

        {teaser}

        Read more: {url}

        #Tutorial #CLI
```

### 3.2 Generation Rules

```yaml
# .claude/automation/config/content_rules.yaml
content_rules:
  # Voice and tone
  voice:
    twitter: "casual, enthusiastic, emoji-friendly"
    linkedin: "professional, informative, value-focused"
    bluesky: "conversational, community-focused"

  # Emoji usage
  emojis:
    twitter: "moderate"  # 2-3 per post
    linkedin: "minimal"  # 0-1 per post
    bluesky: "minimal"

  # Call-to-action
  cta:
    always_include_link: true
    link_shortener: false  # Use full URLs for transparency

  # Scheduling
  scheduling:
    stagger_platforms: true  # Don't post everywhere at once
    stagger_minutes: 30
    avoid_weekends: false
    avoid_holidays: true

  # Approval thresholds
  auto_approve:
    enabled: false  # All posts require approval
    exceptions: []
```

---

## 4. Queue Management

### 4.1 Queue Structure

```yaml
# .claude/automation/queues/social_queue.yaml
metadata:
  last_updated: "2026-01-11T14:30:00Z"
  pending_approval: 3
  scheduled: 5
  posted_today: 2

queue:
  - id: "post-2026-01-11-001"
    type: "release_announcement"
    created: "2026-01-11T14:00:00Z"
    created_by: "release_pipeline"
    trigger_source: "v1.1.0 release"

    status: "pending_approval"  # draft, pending_approval, approved, scheduled, posting, posted, failed

    content:
      twitter:
        text: |
          🚀 Caro v1.1.0 is here!

          ✅ Fish shell support
          ✅ 25% faster inference
          ✅ New safety patterns

          Install: curl -sSL https://setup.caro.sh | bash

          #CLI #Rust #AI
        media: []
        scheduled_for: "2026-01-13T09:00:00Z"

      linkedin:
        text: |
          Excited to announce Caro v1.1.0!

          This release brings significant improvements:

          • Fish shell support - Native fish syntax generation
          • 25% faster inference - Optimized model loading
          • New safety patterns - Better protection against dangerous commands

          Caro converts natural language to shell commands using local AI,
          keeping your data private and your workflow fast.

          Try it now: https://caro.sh
        media: []
        scheduled_for: "2026-01-13T09:30:00Z"

      bluesky:
        text: |
          Caro v1.1.0 just dropped!

          Fish shell support, faster inference, better safety.

          https://caro.sh
        media: []
        scheduled_for: "2026-01-13T10:00:00Z"

    approval:
      required: true
      approved_by: null
      approved_at: null
      rejection_reason: null

    engagement: null  # Filled after posting

  - id: "post-2026-01-11-002"
    # ... more posts
```

### 4.2 Status Flow

```
                     ┌───────────┐
                     │   DRAFT   │
                     └─────┬─────┘
                           │
                           ▼
               ┌─────────────────────┐
               │  PENDING_APPROVAL   │
               └──────────┬──────────┘
                          │
            ┌─────────────┼─────────────┐
            ▼             ▼             ▼
     ┌───────────┐ ┌───────────┐ ┌───────────┐
     │ APPROVED  │ │ REJECTED  │ │  EDITED   │
     └─────┬─────┘ └───────────┘ └─────┬─────┘
           │                           │
           │       ┌───────────────────┘
           ▼       ▼
     ┌───────────────┐
     │   SCHEDULED   │
     └───────┬───────┘
             │
             ▼
     ┌───────────────┐
     │    POSTING    │
     └───────┬───────┘
             │
      ┌──────┴──────┐
      ▼             ▼
┌───────────┐ ┌───────────┐
│  POSTED   │ │  FAILED   │
└───────────┘ └─────┬─────┘
                    │
                    ▼
              ┌───────────┐
              │  RETRY    │
              └───────────┘
```

---

## 5. Admin Dashboard

### 5.1 Dashboard Commands

```
/social-queue status              # Show queue summary
/social-queue review              # Interactive review mode
/social-queue approve <id>        # Approve specific post
/social-queue approve all         # Approve all pending
/social-queue reject <id> <reason> # Reject with reason
/social-queue edit <id>           # Edit post content
/social-queue schedule <id> <time> # Reschedule post
/social-queue history             # Show posting history
/social-queue metrics             # Show engagement metrics
```

### 5.2 Review Interface

```
┌─────────────────────────────────────────────────────────────────┐
│                    SOCIAL QUEUE REVIEW                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  📝 Pending Approval: 3 posts                                   │
│                                                                  │
│  ────────────────────────────────────────────────────────────   │
│                                                                  │
│  Post #1: Release Announcement (v1.1.0)                         │
│  Type: release_announcement                                      │
│  Created: 2026-01-11 14:00                                      │
│                                                                  │
│  ┌─ Twitter (280 chars) ────────────────────────────────────┐   │
│  │ 🚀 Caro v1.1.0 is here!                                  │   │
│  │                                                           │   │
│  │ ✅ Fish shell support                                    │   │
│  │ ✅ 25% faster inference                                  │   │
│  │ ✅ New safety patterns                                   │   │
│  │                                                           │   │
│  │ Install: curl -sSL https://setup.caro.sh | bash          │   │
│  │                                                           │   │
│  │ #CLI #Rust #AI                                           │   │
│  │                                                           │   │
│  │ 📅 Scheduled: Jan 13, 9:00 AM                            │   │
│  └───────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─ LinkedIn ───────────────────────────────────────────────┐   │
│  │ (truncated preview)                                       │   │
│  │ 📅 Scheduled: Jan 13, 9:30 AM                            │   │
│  └───────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Actions:                                                        │
│  [A] Approve All Platforms                                       │
│  [E] Edit Post                                                   │
│  [R] Reject                                                      │
│  [S] Reschedule                                                  │
│  [N] Next Post                                                   │
│  [Q] Quit Review                                                 │
│                                                                  │
│  > _                                                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Engagement Tracking

### 6.1 Metrics Collected

```yaml
# Per post after publishing
engagement:
  twitter:
    posted_at: "2026-01-13T09:00:00Z"
    url: "https://twitter.com/caro_cli/status/..."
    metrics_24h:
      impressions: 1234
      engagements: 89
      likes: 45
      retweets: 12
      replies: 8
      link_clicks: 34
    metrics_7d:
      # ... same structure

  linkedin:
    posted_at: "2026-01-13T09:30:00Z"
    url: "https://linkedin.com/..."
    metrics_24h:
      impressions: 567
      reactions: 23
      comments: 5
      shares: 3
      clicks: 45

  bluesky:
    posted_at: "2026-01-13T10:00:00Z"
    url: "https://bsky.app/..."
    metrics_24h:
      likes: 34
      reposts: 8
      replies: 5
```

### 6.2 Aggregate Metrics

```yaml
# .claude/automation/state/social_metrics.yaml
weekly_summary:
  week: "2026-W02"
  posts_published: 8

  by_platform:
    twitter:
      total_impressions: 12456
      total_engagements: 567
      engagement_rate: 4.5%
      best_performing:
        post_id: "post-2026-01-11-001"
        engagement_rate: 7.2%

    linkedin:
      total_impressions: 3456
      total_engagements: 234
      engagement_rate: 6.8%

    bluesky:
      total_likes: 156
      total_reposts: 34

  content_type_performance:
    release_announcement:
      avg_engagement_rate: 6.2%
    feature_spotlight:
      avg_engagement_rate: 4.1%
    tutorial_promotion:
      avg_engagement_rate: 5.5%

  recommendations:
    - "Feature spotlights underperform - try more visuals"
    - "Tuesday morning posts get highest engagement"
    - "LinkedIn audience responds well to technical depth"
```

---

## 7. Configuration

```yaml
# .claude/automation/config/social_queue.yaml
social_queue:
  enabled: true

  approval:
    required: true
    auto_approve: false
    reminder_after_hours: 24
    expire_after_hours: 168  # 1 week

  scheduling:
    default_lead_time_hours: 24
    stagger_platforms: true
    stagger_minutes: 30
    timezone: "America/Los_Angeles"

  posting:
    dry_run: true  # Set to false to enable actual posting
    retry_on_failure: true
    max_retries: 3
    retry_delay_minutes: 15

  tracking:
    collect_24h_metrics: true
    collect_7d_metrics: true
    collect_30d_metrics: false

  notifications:
    on_post_pending: true
    on_post_failed: true
    on_weekly_summary: true
```

---

## 8. API Integration

### 8.1 Platform API Handlers

```typescript
// Pseudocode for platform integration
interface SocialPlatformAPI {
  authenticate(): Promise<void>;
  post(content: PostContent): Promise<PostResult>;
  getMetrics(postId: string): Promise<Metrics>;
  deletePost(postId: string): Promise<void>;
}

// Twitter/X API v2
class TwitterAPI implements SocialPlatformAPI {
  // OAuth 2.0 PKCE flow
  // Tweet creation with media
  // Analytics API for metrics
}

// LinkedIn API
class LinkedInAPI implements SocialPlatformAPI {
  // OAuth 2.0 flow
  // Share API for posts
  // Analytics API for metrics
}

// BlueSky AT Protocol
class BlueSkyAPI implements SocialPlatformAPI {
  // AT Protocol authentication
  // Record creation for posts
  // Engagement tracking via likes/reposts
}
```

---

## 9. Related Documents

- [SOCIAL_QUEUE_TEST.md](../tests/SOCIAL_QUEUE_TEST.md) - Test cases
- [SOCIAL_MEDIA_GUIDE.md](../../docs/devrel/SOCIAL_MEDIA_GUIDE.md) - Editorial guidelines
- [apps/devrel/](../../apps/devrel/) - Web Hub platform integration
