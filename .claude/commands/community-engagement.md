# Community Engagement

Semi-automated community engagement system for identifying, recognizing, and retaining high-value CARO contributors. Coordinates 7 specialized agents to produce personalized outreach drafts for human approval.

## Usage

```
/community-engagement <command> [options]
```

## Commands

### status - Show engagement pipeline status

```
/community-engagement status
```

Displays:
- Queued drafts awaiting approval
- Messages sent today/this week
- Response rates by channel
- Active A/B tests
- Current autonomy phase

### daily - Run daily engagement cycle

```
/community-engagement daily
```

Executes the full pipeline:
1. @insight-agent gathers data from Hub API
2. @scoring-agent ranks contributors
3. @outreach-agent drafts messages for top targets
4. @channel-agent selects delivery channels
5. Drafts queued for human approval

Options:
- `--dry-run` - Show what would be drafted without queuing
- `--limit N` - Override max outreach targets (default: 10)

### review - Interactive review of drafted messages

```
/community-engagement review
```

Presents each pending draft for approval:
- Shows message preview per channel
- Shows personalization evidence (what data was used)
- Options: approve / edit / reject / skip

### approve - Approve outreach messages

```
/community-engagement approve <id|all>
```

Approve a specific draft or all pending drafts.

### reject - Reject with reason

```
/community-engagement reject <id> --reason "too generic"
```

Reject a draft and log the reason for feedback tracking.

### leaderboard - Show contribution leaderboard

```
/community-engagement leaderboard [--period 7d|30d|all] [--limit 20]
```

Displays:
- Ranked contributors with scores
- Score breakdowns (reuse, remix, creation, safety)
- Tier assignments
- Score deltas from previous period

### founder - Manage founder tier

```
/community-engagement founder <subcommand>
```

Subcommands:
- `candidates` - List eligible candidates with scores
- `invite <user_id>` - Draft an invitation for a candidate
- `pipeline` - Show invitation pipeline (eligible/invited/accepted/declined)
- `status` - Overall Founder tier status (cohort size, remaining slots)

### metrics - Engagement metrics

```
/community-engagement metrics [--period 7d|30d]
```

Displays:
- Response rates by channel and message type
- Trend direction (improving/stable/declining)
- A/B test results
- Recommendations from @feedback-agent

### history - Past engagement actions

```
/community-engagement history [--days N] [--user <id>] [--channel <cli|email|web>]
```

Shows sent messages, responses, and outcomes.

### gaps - Show community content gaps

```
/community-engagement gaps [--limit 10]
```

Displays categories where user demand exceeds recipe supply, with gap scores and matching contributors.

## Example Session

```
> /community-engagement daily --dry-run

Running daily engagement cycle (dry run)...

Gathering insights from Hub API... done (24h snapshot)
Scoring 127 active contributors... done
Selecting outreach targets... 7 targets identified

Draft #1: Recognition for brave-ocean-tiger
  Recipe: "batch-image-converter" (47 runs, 12 reusers this week)
  Channel: CLI
  Preview: "caro: Your batch image converter was reused by 12 people this week."

Draft #2: Direction for calm-river-stone
  Gap: PDF tools (230 searches, 3 recipes)
  Channel: CLI
  Preview: "caro: 230 people searched for PDF tools this week. Want to help?"

Draft #3: Amplification for deep-crystal-wave
  Recipe: "auto-git-cleanup" (trending, velocity +150%)
  Channel: Email
  Preview: Subject: "Your git cleanup recipe is trending"

[... 4 more drafts ...]

Dry run complete. Use /community-engagement daily to queue these for approval.

> /community-engagement review

Draft #1 of 7:
  To: brave-ocean-tiger (Builder tier, score: 47)
  Type: Recognition | Channel: CLI
  Message:
    caro: Your batch image converter was reused by 12 people this week.
    caro: That's more than most published recipes.
  
  Evidence: recipe "batch-image-converter", run_count=47, reuse_count=12
  
  [a]pprove | [e]dit | [r]eject | [s]kip | [q]uit
```

## Configuration

All parameters configured in `.claude/automation/config/community_engagement.yaml`.

Key settings:
- `engagement.max_outreach_per_day: 10`
- `engagement.approval_required: true`
- `channels.cli.max_per_user_per_week: 2`
- `scoring.weights.*` - Contribution score weights

## Workflow Integration

- **Social Queue**: Notable engagement stories can be cross-posted via `/social-queue create`
- **Founder Tier**: Managed exclusively through `/community-engagement founder`
- **Feedback**: All engagement outcomes feed into `/community-engagement metrics`

## Skill Reference

Full agent definitions and mechanisms: `skill: community-engagement-agent`
