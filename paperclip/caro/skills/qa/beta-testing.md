# Skill: Beta Testing Cycles

Wraps existing Caro skill: `.claude/skills/beta-test-cycles/`

## Purpose

Run systematic beta testing cycles with diverse tester personas to find bugs before release.

## When to Use

- Before any release milestone
- After major feature additions
- When validating safety-critical changes
- Regular quality assurance cycles

## Workflow

1. **Create tester profiles**: Generate diverse personas (beginner, expert, malicious, etc.)
2. **Run test scenarios**: Execute predefined test cases from YAML definitions
3. **Collect evidence**: Document results, screenshots, logs
4. **File issues**: Create GitHub issues for failures
5. **Report**: Summarize findings to board

## Invocation

```
skill: beta-test-cycles
skill: unbiased-beta-tester
```

## Key Files

- `.claude/beta-testing/` — Test plans and case definitions
- `.claude/skills/beta-test-cycles/` — Cycle orchestration
- `.claude/skills/unbiased-beta-tester/` — Persona generation

## Testing Infrastructure

- Test plans in YAML format
- Safety validation test suites
- Beta cycle tracking
- Evidence collection system
