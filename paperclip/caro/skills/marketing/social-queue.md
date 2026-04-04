# Skill: Social Content Queue

Wraps existing Caro command: `.claude/commands/social-queue.md`

## Purpose

Manage the social media content queue — schedule posts, track engagement, maintain consistent publishing cadence.

## When to Use

- Scheduling social media posts
- Managing content pipeline
- Tracking post performance
- Maintaining publishing cadence

## Workflow

1. **Source**: Pull from ideas backlog and content calendar
2. **Draft**: Create platform-specific posts
3. **Review**: Check brand voice and accuracy
4. **Schedule**: Add to queue with target publish dates
5. **Monitor**: Track engagement after publishing

## Invocation

```
/social-queue
```

## Key Files

- `.claude/commands/social-queue.md` — Command definition
- `.claude/automation/queues/social_queue.yaml` — Queue data
- `.claude/automation/queues/ideas_backlog.yaml` — Ideas source

## Platforms

- Twitter/X
- LinkedIn
- Mastodon
- Hacker News (for launches/milestones)
- Reddit r/rust, r/commandline
