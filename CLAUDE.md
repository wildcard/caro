# Caro - Natural Language to Shell Commands

A Rust CLI that converts natural language descriptions into safe POSIX shell commands using local LLMs.

## Project Overview

- **Language**: Rust (edition 2021, MSRV 1.83)
- **License**: AGPL-3.0
- **Version**: 1.5.0 (GA)
- **Crate**: [crates.io/crates/caro](https://crates.io/crates/caro)

## Key Architecture

```
src/
├── main.rs              # Entry point, CLI argument parsing
├── backends/            # CommandGenerator trait + implementations
│   ├── mod.rs           # CommandGenerator trait + GeneratorError
│   ├── static_matcher.rs    # Template-based deterministic backend
│   ├── embedded/            # Local LLM (MLX/CPU)
│   ├── remote/              # Remote HTTP backends (feature = remote-backends)
│   │   ├── ollama.rs        # Ollama API
│   │   ├── vllm.rs          # vLLM OpenAI-compatible API
│   │   ├── exo.rs           # Exo distributed cluster
│   │   ├── mesh.rs          # Mesh-LLM pooled mesh (OpenAI-compatible, :9337)
│   │   └── ai_horde.rs      # AI-Horde async volunteer cluster
│   └── hybrid/              # Privacy gateway: local sanitizer + remote enhancer
│       ├── mod.rs           # HybridBackend (sanitize→enhance→restore→fallback)
│       └── sanitizer.rs     # Deterministic reversible PII redaction
├── safety/              # Command safety validation
│   ├── mod.rs           # SafetyValidator trait
│   ├── patterns.rs      # 52+ dangerous command patterns
│   └── validator.rs     # Pattern matching implementation
├── platform/            # Platform detection (OS, arch, shell)
├── agent/               # Agentic context loop
├── prompts/             # LLM prompt templates
├── cache/               # Model caching system
└── eval/                # Evaluation framework
```

## Development Commands

```bash
# Build and test
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # Run all tests
cargo test safety              # Run safety tests only
cargo clippy                   # Lint check

# Run locally
cargo run -- "your query"      # Basic usage
cargo run -- --backend embedded "query"  # Force embedded backend
cargo run -- --dry-run "query" # Generate without executing

# Benchmarks and evaluation
cargo bench                    # Run benchmarks
cargo test --test evaluation   # Run evaluation harness (tests/evaluation/)
```

## Context Management

**Monitor context usage**: Use `/context` to check. Start fresh tasks at ~60% capacity.

**Before context fills**: Use `skill: create_handoff` to preserve state for next session.

**After compacting**: Use `skill: resume_handoff` to restore context from previous session.

## Available Skills (Load On-Demand)

Use `skill: name` when working on specific domains:

| Skill | Use When |
|-------|----------|
| `safety-pattern-developer` | Adding new dangerous command patterns (TDD workflow) |
| `safety-pattern-auditor` | Auditing existing patterns for gaps |
| `beta-test-cycles` | Running systematic test cycles |
| `prompt-tuner` | Improving embedded LLM system prompts |
| `quality-engineer-manager` | Release validation and QA sign-off |
| `unbiased-beta-tester` | Simulating user testing scenarios |
| `create_handoff` / `resume_handoff` | Session state preservation |
| `continuity_ledger` | Long-running project state |

## Key Commands

| Command | Purpose |
|---------|---------|
| `/caro.feature` | Start new feature development (spec-kitty workflow) |
| `/caro.qa` | Quality assurance investigation |
| `/caro.roadmap` | View and align with project roadmap |
| `/caro.release.prepare` | Prepare a new release |
| `/caro.sync` | Sync documentation across codebase |
| `/spec-kitty.plan` | Create implementation plan |
| `/spec-kitty.implement` | Execute implementation |

## Model Selection Hints

When spawning sub-agents via Task tool:

- **model: "haiku"** - Quick searches, simple validations, file lookups
- **model: "sonnet"** - Code review, test execution, documentation
- **model: "opus"** - Complex analysis, architecture decisions, safety audits

## Safety Validation

All generated commands go through safety validation:

1. **52+ dangerous patterns** - Pre-compiled regex patterns
2. **Risk levels** - CRITICAL, HIGH, MEDIUM, LOW
3. **Zero false positives** - Validated through extensive testing

When adding safety patterns, ALWAYS use TDD:
```
skill: safety-pattern-developer
```

## Testing Standards

- **93.1% pass rate** on comprehensive test suite
- **Zero false positives** in safety validation
- **TDD methodology** for all safety-critical code
- Run `cargo test` before any commit

## Current Focus Areas

Check `.claude/memory/current-tasks.md` for active work items.

## Code Style

- Follow existing Rust idioms in the codebase
- Use `thiserror` for error types
- Prefer `anyhow::Result` for application errors
- Keep functions small and single-purpose
- Add tests for any new functionality

## Important Files to Know

- `.claude/rules/constitution.md` - **Precedence index for all `.claude/rules/` files (read first)**
- `src/safety/patterns.rs` - All dangerous command patterns
- `src/prompts/command_templates.rs` - LLM prompt templates
- `.claude/skills/` - Domain expertise (load on-demand)
- `.claude/commands/` - Workflow commands
- `.claude/beta-testing/` - Test infrastructure
- `bin/notify` - Cross-session agent notification log helper
- `.hermes/` - Hermes strategic intelligence agent (see below)

## Agent Ecosystem

### Hermes — Strategic Intelligence & Coordination

Hermes is the always-on strategic layer. It does NOT write code. It watches,
synthesizes, triages, and routes — so coding agents can focus on execution.

**What Hermes does:**
- Daily PR triage and digest (`.hermes/digests/`)
- Competitive intelligence (weekly market scans)
- Cross-agent coordination and conflict detection
- Integration health monitoring
- Release readiness assessments
- Executive briefings

**How to communicate with Hermes:**
- `bin/notify hermes "<msg>"` — real-time event stream
- `.hermes/messages/<topic>.md` — structured inter-agent messages
- `gh pr comment <N>` tagged `[agent]` — PR-specific feedback (from Hermes)
- `.hermes/digests/` — daily digests for human consumption

**Hermes does NOT:** write Rust code, merge PRs, run QA tests, or manage
the beads backlog. Those are owned by Claude Code, Crush, and specialist agents.

Full definition: `.hermes/AGENT.md`
Communication protocol: `.hermes/PROTOCOL.md`

## Exploration Pattern

Before implementing features:

1. **Understand**: Ask clarifying questions about requirements
2. **Explore**: Use Explore agent to find related code
3. **Plan**: Use `/spec-kitty.plan` for complex features
4. **Implement**: Follow the plan with TDD
5. **Validate**: Run tests, check safety

## Parallel Agent Pattern

For comprehensive analysis, launch multiple agents in a single message:

```
Use the Task tool to launch these in parallel:
1. Explore agent: "find safety validation code"
2. Explore agent: "find related tests"
3. Explore agent: "find configuration"
```

---

*Last updated: 2026-01-14*

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
<!-- SPECKIT END -->
