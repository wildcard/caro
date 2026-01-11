# Caro Automated Development Flow

This directory contains the infrastructure for automated development workflows that run on cadence or in response to events.

## Quick Start

```bash
# Check automation status
/automation/orchestrate status

# Run a specific pack
/automation/orchestrate run technical

# Run all enabled loops
/automation/orchestrate run all
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                       ORCHESTRATOR                               │
│                  /automation/orchestrate                         │
└─────────────────────────────┬───────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│  TECHNICAL    │     │   CONTENT     │     │  MANAGEMENT   │
│     PACK      │     │     PACK      │     │     PACK      │
├───────────────┤     ├───────────────┤     ├───────────────┤
│ • QA Loop     │     │ • Idea Source │     │ • PR Mgmt     │
│ • Visual Reg  │     │ • Social Queue│     │ • Stale Revival│
│ • Sync Engine │     │               │     │ • Metrics     │
└───────────────┘     └───────────────┘     └───────────────┘
```

## Packs

### Technical Pack (Fully Automated)

| Loop | Schedule | Description |
|------|----------|-------------|
| `/qa-automation-loop` | Daily 9 AM | Unbiased beta testing, issue creation |
| `/visual-regression-test` | Nightly 2 AM | Screenshot comparison |
| `/caro.sync` | Daily 6 PM | Content synchronization |

### Content Pack (Semi-Automated)

| Loop | Schedule | Description |
|------|----------|-------------|
| `/idea-sourcing-loop` | Daily 8 AM | Source ideas from HN, Reddit, etc. |
| `/social-queue` | Continuous | Social posts with approval |

### Management Pack (Automated with Oversight)

| Loop | Schedule | Description |
|------|----------|-------------|
| `/pr-management-loop` | Every 4 hours | PR review, rebase, agent responses |
| `/stale-revival-loop` | Weekly Monday | Revive stale PRs/issues |

## Directory Structure

```
.claude/automation/
├── README.md              # This file
├── config/                # Configuration files
│   ├── schedule.yaml      # Loop schedules
│   ├── qa_profiles.yaml   # QA tester profiles
│   ├── idea_sources.yaml  # Idea sourcing config
│   └── social_queue.yaml  # Social posting config
├── specs/                 # DRS specifications
│   ├── AUTOMATED_DEV_FLOW_DRS.md  # Master spec
│   ├── QA_LOOP_DRS.md
│   ├── VISUAL_REGRESSION_DRS.md
│   ├── IDEA_SOURCING_DRS.md
│   ├── SOCIAL_QUEUE_DRS.md
│   └── PR_MANAGEMENT_DRS.md
├── tests/                 # Test documents
│   ├── AUTOMATION_TESTS_INDEX.md
│   ├── QA_LOOP_TEST.md
│   └── VISUAL_REGRESSION_TEST.md
├── queues/                # Work queues
│   ├── ideas_backlog.yaml
│   └── social_queue.yaml
└── state/                 # Runtime state
    ├── last_run.json
    ├── metrics.json
    ├── qa_runs/
    ├── idea_sourcing/
    ├── pr_management/
    └── visual_regression/
```

## Skills (Commands)

Located in `.claude/commands/`:

| Skill | Purpose |
|-------|---------|
| `automation.orchestrate.md` | Central dispatcher |
| `qa-automation-loop.md` | QA testing loop |
| `visual-regression-test.md` | Visual testing |
| `idea-sourcing-loop.md` | Idea sourcing |
| `social-queue.md` | Social content queue |
| `pr-management-loop.md` | PR management |

## Separation from Manual Workflows

This automated flow is **separate** from:

- Ad-hoc Claude Code sessions
- Manual PR reviews by humans
- Emergency hotfixes
- Strategic planning
- Community moderation
- Security incident response

These remain manual/human-driven processes.

## Configuration

Edit `config/schedule.yaml` to modify schedules:

```yaml
technical:
  qa_automation_loop:
    schedule: "0 9 * * *"   # Daily 9 AM
    enabled: true
```

## Metrics

View automation metrics:

```bash
/automation/orchestrate metrics
```

## Related Documents

- [WORKFLOW_IMPROVEMENT_PLAN.md](../../docs/development/WORKFLOW_IMPROVEMENT_PLAN.md)
- [AUTOMATED_DEV_FLOW_DRS.md](./specs/AUTOMATED_DEV_FLOW_DRS.md)
