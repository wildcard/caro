# Skill: Rust TDD Development

Wraps existing Caro skill: `.claude/skills/tdd-rust-engineer/`

## Purpose

Guide Rust development using strict Test-Driven Development methodology with the red-green-refactor cycle.

## When to Use

- Implementing new features for the caro CLI
- Adding new modules or components
- Refactoring existing code with test coverage
- Building safety-critical functionality

## Workflow

1. **Red**: Write a failing test that defines the desired behavior
2. **Green**: Write the minimum code to make the test pass
3. **Refactor**: Clean up while keeping tests green

## Invocation

```
skill: tdd-rust-engineer
```

Or use the Claude Code agent with subagent_type: `tdd-rust-engineer`

## Key Principles

- Never write production code without a failing test first
- Keep functions small and single-purpose
- Use `thiserror` for error types, `anyhow::Result` for application errors
- Follow existing Rust idioms in the codebase
- Run `cargo test` after every change
- Run `cargo clippy` before committing

## Context Files

- `src/` — Main source code
- `tests/` — Integration tests
- `Cargo.toml` — Dependencies and configuration
