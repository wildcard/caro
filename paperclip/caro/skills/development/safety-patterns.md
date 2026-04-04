# Skill: Safety Pattern Development

Wraps existing Caro skills: `.claude/skills/safety-pattern-developer/`, `.claude/skills/safety-pattern-auditor/`

## Purpose

Develop and audit dangerous command patterns for caro's safety validation system using strict TDD methodology.

## When to Use

- Adding new dangerous command patterns
- Auditing existing patterns for gaps
- Responding to reported false positives/negatives
- Expanding coverage for new threat vectors

## Workflow

1. **Identify threat**: Determine the dangerous command or pattern
2. **Write failing test**: Create test cases for both positive (should match) and negative (should not match) scenarios
3. **Implement pattern**: Add regex pattern to `src/safety/patterns.rs`
4. **Verify**: Run `cargo test safety` to confirm zero false positives
5. **Audit**: Check for edge cases and bypasses

## Invocation

```
skill: safety-pattern-developer
skill: safety-pattern-auditor
```

## Key Files

- `src/safety/patterns.rs` — All 52+ dangerous command patterns
- `src/safety/validator.rs` — Pattern matching implementation
- `src/safety/mod.rs` — SafetyValidator trait

## Critical Rules

- ALWAYS use TDD — no pattern without tests
- ZERO false positives tolerance
- Every pattern needs positive AND negative test cases
- Risk levels: CRITICAL, HIGH, MEDIUM, LOW
- Pre-compiled regex for performance
