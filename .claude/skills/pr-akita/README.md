# PR Akita

AI-powered PR triage and fix-merge workflow based on the [Vibe Maintainer](https://steve-yegge.medium.com/vibe-maintainer-a2273a841040) philosophy.

## What It Does

PR Akita is the loyal guardian of caro's PR queue. It triages open pull requests, fixes minor issues itself, and merges contributions -- defaulting to "yes" rather than bouncing PRs back to contributors.

## When to Use

- When open PRs need triage and disposition
- When a specific PR has fixable CI/lint issues
- During the scheduled PR management loop
- When you want to clear the PR backlog

## Quick Start

```
skill: pr-akita
```

Or invoke as part of the PR management loop:
```
/pr-management-loop
```

## Triage Buckets

| Bucket | Action |
|--------|--------|
| Easy Win | Auto-merge |
| Fix-Merge | Fix issues, merge with attribution |
| Needs-Review | Flag for human maintainer |
| Hygiene Issue | Guide contributor kindly |
| Retire | Close with gratitude |

## Key Principle

> "Request changes" is the LAST RESORT. Fix it yourself, credit the contributor, merge.

## Related

- [ADR-015: Vibe Maintainer Workflow](../../docs/adr/ADR-015-vibe-maintainer-workflow.md)
- [Vibe Maintainer Rule](../../.claude/rules/vibe-maintainer.md)
- [PR Management Loop](../../.claude/commands/pr-management-loop.md)
- [Full Skill Documentation](SKILL.md)
