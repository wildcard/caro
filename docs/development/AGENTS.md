# Agent Guidelines for caro Development

This file provides guidance for **all AI agents** (Claude Code, Cursor, Codex, Crush, etc.) working on the caro project.

## Project Context

**caro** is a Rust CLI tool that converts natural language to safe POSIX shell commands using local LLMs. The project prioritizes safety, performance, and developer experience with Apple Silicon optimization.

## Workflow Selection: Spec-Kitty vs Spec-Kit

This project uses **dual spec-driven workflows**. Choose the right one based on feature characteristics:

### Quick Decision Guide

```
Feature Request → Estimate complexity and time

< 2 weeks, clear scope, incremental changes
    ↓
USE SPEC-KITTY (kitty-specs/)
    • bin/sk-new-feature "description"
    • /spec-kitty.* commands
    • Real-time dashboard
    • Worktree-based parallel development

> 2 weeks, research needed, major architecture
    ↓
USE SPEC-KIT (specs/)
    • Manual spec/plan creation
    • Constitution-driven governance
    • Traditional git workflow
```

### Spec-Kitty Workflow (Rapid Development)

**Use when:**
- ✅ Feature size: **< 2 weeks**
- ✅ Complexity: **Low to Medium**
- ✅ Scope: **Well-defined, clear requirements**
- ✅ Need: **Parallel development capability**
- ✅ Want: **Visual dashboard tracking**

**Location**: `kitty-specs/` (git worktrees)

**Commands**: `/spec-kitty.*` slash commands

**Workflow**:
1. `bin/sk-new-feature "description"` - Creates isolated worktree
2. `cd kitty-specs/001-feature-name/`
3. `/spec-kitty.specify` - Create spec.md
4. `/spec-kitty.plan` - Create plan.md
5. `/spec-kitty.tasks` - Generate work packages
6. `/spec-kitty.implement` - Execute tasks
7. `/spec-kitty.accept` - Run acceptance checks
8. `/spec-kitty.merge` - Merge and cleanup

**Examples**:
- "Add Redis caching with TTL support"
- "Fix memory leak in MLX backend initialization"
- "Add Prometheus metrics endpoint"
- "Implement command history feature"
- "Add --json output flag"
- "Improve error messages for validation failures"

### Spec-Kit Workflow (Large Features)

**Use when:**
- ✅ Feature size: **> 2 weeks**
- ✅ Complexity: **High**
- ✅ Scope: **Requires extensive research**
- ✅ Need: **Deep architectural investigation**
- ✅ Want: **Constitution-driven governance**

**Location**: `specs/` (traditional directories)

**Commands**: Custom commands in `.codex/prompts/`

**Workflow**:
1. `mkdir -p specs/NNN-feature-name/`
2. Manually create spec.md, plan.md, tasks.md
3. Use `.specify/templates/` for structure
4. Follow `.specify/memory/constitution.md` principles
5. Standard git workflow on feature branches

**Examples**:
- "Implement complete MLX backend with C++ FFI"
- "Design and build multi-backend inference system"
- "Create comprehensive safety validation framework"
- "Research and implement model quantization pipeline"
- "Architect distributed caching system"

## Decision Logic

When a user requests a feature or bug fix, use this 4-step process:

### 1. Estimate Complexity and Time

**Ask yourself:**
- Can this be completed in < 2 weeks of focused work?
- Does it require extensive research or investigation?

```
< 2 weeks → spec-kitty
> 2 weeks → spec-kit
```

### 2. Check Scope Clarity

**Ask yourself:**
- Is the scope well-defined and clear?
- Or does it need investigation to understand the problem?

```
Clear, well-defined → spec-kitty
Needs investigation → spec-kit
```

### 3. Assess Architecture Impact

**Ask yourself:**
- Is this an incremental change to existing systems?
- Or does it require major refactoring or new core systems?

```
Incremental changes → spec-kitty
Major refactoring → spec-kit
```

### 4. Consider Parallel Work

**Ask yourself:**
- Is the user working on multiple features simultaneously?
- Or is this a single, focused effort?

```
Multiple features → spec-kitty (worktrees enable parallel work)
Single large feature → spec-kit (or spec-kitty if < 2 weeks)
```

## When in Doubt

If you're **unsure** which workflow to use:

1. **Ask the user**: "This looks like a [small/medium/large] feature. Should we use spec-kitty for rapid development or spec-kit for comprehensive planning?"

2. **Default to spec-kitty**: For most features and bugs, spec-kitty is the faster, more practical choice. Only use spec-kit when clearly justified.

3. **Check existing work**: Look at `specs/` directory to see examples of features that warranted spec-kit (e.g., `specs/004-implement-ollama-and/` for multi-backend system).

## Both Workflows Coexist

The project supports **both workflows simultaneously**:
- `kitty-specs/001-add-caching/` - spec-kitty rapid development
- `specs/004-implement-ollama-and/` - spec-kit large feature

You can work on quick bug fixes in `kitty-specs/` while major features progress in `specs/`.

## Workflow Switching

**Important**: You **cannot switch** workflows mid-feature. Once started in one workflow, complete it there.

If a spec-kitty feature grows beyond initial estimates, continue in spec-kitty but document lessons learned for future scoping.

## Helper Scripts

The project provides convenience scripts:

```bash
# Spec-Kitty helpers
bin/sk-new-feature "description"    # Create new feature worktree
bin/sk-dashboard                    # Open real-time dashboard
bin/sk-list                         # List all features
bin/sk-merge <feature-id>           # Merge completed feature

# Dashboard
http://127.0.0.1:9237              # Real-time kanban view
```

## Development Standards

All features (regardless of workflow) must follow:

### Safety-First Approach
- Pattern matching for dangerous commands
- POSIX compliance validation
- Comprehensive testing before merge
- Security audits for system-level operations

### Code Quality
- All public APIs documented
- No panics in production code (use `Result` types)
- Comprehensive error handling with helpful messages
- Memory safety without unnecessary allocations

### Testing Requirements
- Unit tests for individual components
- Integration tests for workflows
- Performance benchmarks where applicable
- Security validation for command generation

## Product Management Work (SIGMA)

When performing product management tasks, activate the SIGMA persona and follow the SIGMA operating model:

- Review `docs/SIGMA_AGENT.md` for scope, prompts, metrics, and templates.
- Review `docs/PRODUCT_MANAGER_AGENT_STACK.md` for sub-agent delegation, cadence, and reporting expectations.
- Align outputs to roadmap and public messaging sources (`ROADMAP.md`, `README.md`, `docs-site/**`, `website/**`).

If GitHub issues/PRs or project boards are required but not accessible, document the visibility gap and request access before proceeding.

### Git Discipline
- Meaningful commit messages (imperative mood)
- Commit only complete units of work
- Never commit secrets, tokens, or credentials
- Keep branches up to date with main

## Rust-Specific Considerations

Before running `cargo` commands:
```bash
# Check if cargo is in PATH
which cargo

# If not found, source the environment
. "$HOME/.cargo/env"
```

### Build Commands
```bash
cargo build --release              # Production build
cargo test                         # Run all tests
cargo clippy -- -D warnings        # Linting
cargo fmt --check                  # Format check
cargo audit                        # Security audit
```

## Documentation

- **Full guide**: `docs/SPEC_KITTY_GUIDE.md`
- **Quick reference**: `docs/SPEC_KITTY_QUICKREF.md`
- **Project overview**: `CLAUDE.md`
- **Contributing**: `CONTRIBUTING.md`

## Example: Choosing the Right Workflow

### Scenario 1: User requests "Add caching"

**Agent thinks:**
- Estimate: 3-5 days (< 2 weeks) ✓
- Scope: Clear - add Redis caching with TTL ✓
- Architecture: Incremental - new module, existing patterns ✓
- Parallel work: User might be working on other features ✓

**Decision**: Use spec-kitty

**Action**:
```bash
bin/sk-new-feature "Add Redis caching layer with TTL support"
cd kitty-specs/001-add-redis-caching/
/spec-kitty.specify
```

### Scenario 2: User requests "Implement distributed consensus"

**Agent thinks:**
- Estimate: 4-6 weeks (> 2 weeks) ✗
- Scope: Needs research - which consensus algorithm? ✗
- Architecture: Major change - affects core systems ✗
- Parallel work: Likely focused single effort ✗

**Decision**: Use spec-kit

**Action**:
```bash
mkdir -p specs/005-distributed-consensus/
# Create spec.md with research phase
# Use .specify/templates/spec-template.md
```

### Scenario 3: User requests "Optimize performance"

**Agent thinks:**
- Estimate: Unknown - needs profiling first
- Scope: Unclear - where are the bottlenecks?
- Architecture: Unknown until investigated

**Decision**: Ask for clarification

**Action**:
"Before choosing a workflow, let's profile the application to understand where the bottlenecks are. This could be a small optimization (spec-kitty) or a major refactoring (spec-kit). Should we run performance benchmarks first?"

## Quick Reference

| Criteria | Spec-Kitty | Spec-Kit |
|----------|------------|----------|
| **Time** | < 2 weeks | > 2 weeks |
| **Scope** | Clear, defined | Needs research |
| **Architecture** | Incremental | Major refactoring |
| **Parallel work** | Yes (worktrees) | Focused effort |
| **Examples** | Bug fixes, features | New core systems |
| **Tools** | Dashboard, automated tasks | Manual planning, constitution |

**Default choice**: When in doubt, prefer spec-kitty for faster iteration.

## Resources

- Spec-Kitty GitHub: https://github.com/Priivacy-ai/spec-kitty
- Dashboard: http://127.0.0.1:9237
- Helper scripts: `bin/sk-*`
- Full documentation: `docs/SPEC_KITTY_GUIDE.md`

---

**Remember**: The goal is to ship safe, performant, well-tested code. Choose the workflow that best supports that goal for the specific feature at hand.
