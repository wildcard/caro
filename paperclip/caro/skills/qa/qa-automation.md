# Skill: QA Automation Loop

Wraps existing Caro command: `.claude/commands/qa-automation-loop.md`

## Purpose

Automated quality assurance loop that runs unbiased beta testers, collects results, and creates GitHub issues for failures.

## When to Use

- Scheduled daily QA runs (9 AM)
- Manual QA trigger before releases
- After significant code changes

## Workflow

1. **Initialize**: Load test plans and create tester personas
2. **Execute**: Run test scenarios against current build
3. **Evaluate**: Compare results against expected outcomes
4. **Report**: Create issues for failures, update metrics
5. **Notify**: Alert board of critical findings

## Invocation

```
/qa-automation-loop
```

## Schedule

- Daily 9 AM (automated via `.claude/automation/config/schedule.yaml`)
- Timeout: 30 minutes
- Retries: 2 attempts on failure

## Key Files

- `.claude/commands/qa-automation-loop.md` — Loop definition
- `.claude/automation/specs/QA_LOOP_DRS.md` — Design spec
- `.claude/automation/tests/QA_LOOP_TEST.md` — Test spec
