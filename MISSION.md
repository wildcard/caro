# Mission and Values

## The Problem We Solve

Developers know what they want to do. They often don't remember exactly how to express it in shell syntax.

This creates friction:
- Searching documentation for the right flags
- Second-guessing whether a command is safe before running it
- Switching context between your actual work and command lookup
- Wondering if a command does what you think it does

The command line is powerful. But the barrier between intent and action shouldn't require memorizing syntax from decades of accumulated conventions.

## What Caro Is

Caro is a command-line tool that converts natural language into shell commands using local AI models.

You describe what you want. Caro generates a POSIX-compliant command. It asks for confirmation before running anything potentially destructive.

That's it.

The tool runs entirely on your machine. No cloud services. No API keys. No data leaving your computer. Works offline once models are cached.

## Core Values

These values guide every decision in the project—from code architecture to community norms.

### Safety Before Convenience

We validate every generated command against 52+ dangerous patterns before showing it to you. Destructive operations require explicit confirmation. The tool will refuse to execute commands that could cause system damage.

This is not optional. Safety is built into the core, not added as an afterthought. We would rather block a valid command than allow a destructive one through.

When choosing between user convenience and user safety, safety wins.

### Honesty Over Hype

We describe caro as what it is: a command generator that uses local LLMs. We don't claim it replaces shell knowledge—it assists it. We don't promise it never makes mistakes—it validates what it generates.

Documentation tells you what the tool does and doesn't do. Error messages explain what went wrong and suggest what to try. We state limitations clearly.

### Local First

Your prompts and commands stay on your machine. Models run locally via MLX (Apple Silicon) or CPU inference. Network access is only used if you explicitly configure remote backends.

This means:
- Privacy by default, not by policy
- Works offline after initial setup
- No usage tracking or telemetry
- You control your data

### Simplicity Over Cleverness

One binary. Minimal configuration. Sensible defaults that work immediately.

We avoid abstraction layers that don't provide clear value. We prefer readable code over clever code. We document the "why" behind decisions, not just the "what."

When something can be simpler, we make it simpler. When complexity is necessary, we explain it.

### Code Quality Is Not Negotiable

Test-driven development is mandatory. Every feature starts with a failing test. Public APIs have documentation. Linting and formatting are enforced.

We don't ship code that passes tests but fails users. We don't accumulate technical debt to ship faster. We fix problems when we find them.

## What We're Building Toward

### Short-term Goals

- Reliable command generation across common shell workflows
- Fast startup (<100ms) and inference (<2s on Apple Silicon)
- Comprehensive safety validation that users can trust
- Clear documentation that gets people started quickly

### Medium-term Goals

- Command history and learning from user corrections
- Multi-step workflows for complex tasks
- Plugin system for custom backends and validators
- Broader platform support and optimization

### Long-term Vision

Make the command line accessible without making it less powerful.

We want developers to spend mental energy on their actual problems, not on remembering which flags to pass to `find` or whether `xargs` needs `-I{}` on this platform.

The shell should feel like a natural extension of intent, not a test of syntax recall.

## How the Project Works

### Open Development

All work happens in public. Specifications, discussions, and decisions are documented in the repository. You can see why choices were made, not just what was chosen.

Pull requests welcome from anyone. We provide detailed contributing guidelines and maintain good-first-issue labels for newcomers.

### Spec-Driven Development

Major features start with specifications before code. The `specs/` directory contains detailed design documents. This ensures we agree on what we're building before we build it.

Rapid development uses the spec-kitty workflow with isolated worktrees. Complex features use traditional spec-kit with deeper research phases.

### Code Review Standards

All changes require tests. All public APIs need documentation. Formatting and linting are automated. Security audits run in CI.

We review for correctness, clarity, and alignment with project values—not just whether code compiles.

### Community Standards

We follow the Contributor Covenant. Harassment isn't tolerated. Technical disagreement is welcome; personal attacks aren't.

We assume good intent. We explain our reasoning. We help newcomers get oriented.

## Participation

### Using Caro

Install it. Try it. If it works for you, keep using it. If it doesn't, that's useful feedback.

We'd rather have honest criticism than polite silence. File issues when things break. Suggest improvements when things could be better.

### Contributing

You don't need to be a Rust expert. Documentation improvements, bug reports, safety pattern additions—all valuable.

Check CONTRIBUTING.md for setup instructions. Browse issues labeled `good-first-issue` for entry points. Ask questions in discussions if you're unsure where to start.

### Supporting the Project

Star the repo if you find it useful. Tell others if it solves their problem too. But only if it actually does—we'd rather grow slowly with satisfied users than quickly with disappointed ones.

## Summary

Caro exists because the gap between intent and command shouldn't require syntax memorization.

We build it with safety as the foundation, honesty as the standard, and simplicity as the goal.

We welcome contributors who share these values.

That's the mission.

---

*This document reflects the project's current direction. It may evolve as we learn from users and contributors.*
