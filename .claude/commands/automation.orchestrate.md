# Automation Orchestrator

Central dispatcher for all automated development flows. Manages scheduling, execution, and monitoring of Technical, Content, and Management packs.

## Usage

```
/automation/orchestrate <command> [options]
```

## Commands

### run - Execute automation loops

```
/automation/orchestrate run <pack|loop|all>
```

**Examples:**
- `/automation/orchestrate run technical` - Run all technical pack loops
- `/automation/orchestrate run qa_automation_loop` - Run specific loop
- `/automation/orchestrate run all` - Run all enabled loops

### status - Show automation status

```
/automation/orchestrate status [--verbose]
```

Shows:
- Last run time for each loop
- Pass/fail status
- Upcoming scheduled runs
- Any errors or warnings

### schedule - Show/modify schedule

```
/automation/orchestrate schedule [--show|--edit]
```

### history - Show run history

```
/automation/orchestrate history [--days N] [--loop NAME]
```

### metrics - Show performance metrics

```
/automation/orchestrate metrics [--period 7d|30d|all]
```

## Automation Packs

### Technical Pack (Fully Automated)

| Loop | Schedule | Description |
|------|----------|-------------|
| `qa_automation_loop` | Daily 9 AM | Unbiased beta testers, issue creation |
| `visual_regression` | Nightly 2 AM | Screenshot comparison |
| `sync_engine` | Daily 6 PM | Content synchronization |

### Content Pack (Semi-Automated)

| Loop | Schedule | Description |
|------|----------|-------------|
| `idea_sourcing` | Daily 8 AM | Source ideas from HN, Reddit, etc. |
| `content_digest` | Weekly Mon 9 AM | Weekly content summary |

### Management Pack (Automated with Oversight)

| Loop | Schedule | Description |
|------|----------|-------------|
| `pr_management` | Every 4 hours | PR review, rebase, agent responses |
| `stale_revival` | Weekly Mon 10 AM | Revive stale PRs/issues |
| `metrics_collection` | Daily midnight | Aggregate metrics |
| `roadmap_status` | Daily 6 AM | Sync roadmap from GitHub |

## Execution Process

When invoked, this skill:

1. **Load Configuration**
   - Read `.claude/automation/config/schedule.yaml`
   - Load state from `.claude/automation/state/`

2. **Validate Environment**
   - Check required tools (gh, git, npm)
   - Verify API access if needed

3. **Execute Loops**
   - Run specified loops in order
   - Capture output and timing
   - Handle errors gracefully

4. **Update State**
   - Record run results
   - Update metrics
   - Send notifications if configured

## Example Session

```
> /automation/orchestrate status

┌─────────────────────────────────────────────────────────────────┐
│                    AUTOMATION STATUS                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  TECHNICAL PACK                                                  │
│  ──────────────                                                  │
│  qa_automation_loop   ✓ Last: 2026-01-11 09:00  Next: Tomorrow  │
│  visual_regression    ✓ Last: 2026-01-11 02:00  Next: 02:00     │
│  sync_engine          ✓ Last: 2026-01-10 18:00  Next: 18:00     │
│                                                                  │
│  CONTENT PACK                                                    │
│  ────────────                                                    │
│  idea_sourcing        ✓ Last: 2026-01-11 08:00  Next: Tomorrow  │
│  content_digest       ○ Disabled                                │
│                                                                  │
│  MANAGEMENT PACK                                                 │
│  ───────────────                                                 │
│  pr_management        ✓ Last: 2026-01-11 12:00  Next: 16:00     │
│  stale_revival        ○ Last: 2026-01-06 10:00  Next: Monday    │
│  metrics_collection   ✓ Last: 2026-01-11 00:00  Next: Midnight  │
│                                                                  │
│  OVERALL: 7/8 loops healthy                                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

> /automation/orchestrate run technical

Running Technical Pack...

[1/3] qa_automation_loop
  ├── Spawning 4 beta tester profiles...
  ├── Collecting feedback...
  ├── Creating 2 issues...
  └── ✓ Complete (5m 23s)

[2/3] visual_regression
  ├── Starting dev servers...
  ├── Capturing 30 screenshots...
  ├── Comparing with baseline...
  └── ✓ All screenshots match (2m 45s)

[3/3] sync_engine
  ├── Syncing roadmap...
  ├── Syncing installation docs...
  └── ✓ Complete (45s)

Technical Pack Complete
  Total time: 8m 53s
  All loops passed ✓
```

## State Files

```
.claude/automation/state/
├── last_run.json           # Last run time per loop
├── run_history/            # Historical run data
│   ├── 2026-01-11.yaml
│   └── ...
├── metrics.json            # Aggregate metrics
└── errors.json             # Error log
```

## Configuration

See `.claude/automation/config/schedule.yaml` for full configuration options.

## Related Skills

- `/qa-automation-loop` - QA testing loop
- `/visual-regression-test` - Visual regression testing
- `/idea-sourcing-loop` - Idea sourcing from external sources
- `/social-queue` - Social content queue management
- `/pr-management-loop` - PR management and review
- `/caro.sync` - Content synchronization

## DRS Reference

See [AUTOMATED_DEV_FLOW_DRS.md](../.claude/automation/specs/AUTOMATED_DEV_FLOW_DRS.md) for complete specification.
