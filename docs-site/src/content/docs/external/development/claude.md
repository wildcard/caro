---
title: Claude Integration
description: "Documentation: Claude Integration"
editUrl: false
---
This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`caro` (formerly `cmdai`) is a single-binary Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. The tool prioritizes safety, performance, and developer experience with Apple Silicon optimization via MLX framework.

> **Note**: The project was renamed from `cmdai` to `caro` in December 2025. See [docs/NAMING_HISTORY.md](docs/NAMING_HISTORY.md) for details.

**Core Goals:**
- Single binary under 50MB (without embedded model)
- Startup time < 100ms, first inference < 2s on M1 Mac
- Safety-first approach with comprehensive command validation
- Extensible backend system (MLX, vLLM, Ollama)
- Hugging Face model caching with offline capability

## Project Structure

```
caro/
├── src/
│   ├── main.rs              # CLI entry point with clap configuration
│   ├── backends/            # Inference backend implementations
│   │   ├── mod.rs          # Backend trait system
│   │   ├── mlx.rs          # Apple Silicon MLX backend (FFI)
│   │   ├── vllm.rs         # vLLM HTTP API backend
│   │   └── ollama.rs       # Ollama local backend
│   ├── cache/              # Hugging Face model caching
│   ├── safety/             # Command validation and safety checks
│   └── config/             # Configuration management
├── tests/
│   ├── integration/        # End-to-end workflow tests
│   └── unit/              # Component-specific tests
└── .devcontainer/         # Development environment setup
```

## Architecture Overview

### Backend Trait System
All model backends implement `ModelBackend` trait:
- Async inference with `Result<String>` responses
- Availability checking with graceful fallbacks
- Unified configuration through `BackendConfig`
- JSON-only response parsing with multiple fallback strategies

### Safety-First Design
Safety module provides:
- Pattern matching for dangerous commands (`rm -rf /`, `mkfs`, fork bombs)
- POSIX compliance validation
- Path quoting and validation
- Risk level assessment (Safe, Moderate, High, Critical)
- User confirmation workflows

### Platform Optimization
- MLX backend uses FFI with cxx crate for Apple Silicon
- Conditional compilation with feature flags
- Cross-platform cache directory management
- Shell-specific optimizations and detection

### Sync Module (Planned)
Local-first sync with Jazz.tools for multi-device command history:
- `src/sync/`: Rust sync library (identity, encryption, IPC client)
- `sync-daemon/`: Node.js companion for Jazz SDK integration
- IPC: Unix socket at `~/.config/caro/sync.sock`
- Encryption: AES-256-GCM with Argon2id key derivation from BIP39 phrase
- Privacy: E2E encrypted, zero-knowledge relay sync
- See `specs/005-jazz-sync-integration/` for full specification

## Development Commands

> !IMPORTANT:
> Before running `cargo` or any rust development command in the shell, check the the command is installed with `which` and inspect the `$PATH` for the relevant bin.

> If it doesn't run `. "$HOME/.cargo/env"` in your shell before command execution

### Git Workflow

**When reverting changes**: Use `git revert <commit-hash>` or `git reset`, NOT manual file edits. Manual edits break git history and introduce inconsistencies. Always use git commands to manage history.

### Building & Testing
```bash
# Build the project
cargo build --release

# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo run -- "list all files"

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### Development Environment
```bash
# Start development container
devcontainer open .

# Watch for changes during development
cargo watch -x check -x test -x run
```

## Implementation Phases

### Phase 1: Core CLI Structure
- Command-line argument parsing with clap
- Mock inference backend for initial testing
- Basic safety validation implementation
- Configuration and cache directory setup

### Phase 2: Safety & Validation
- Comprehensive dangerous command patterns
- POSIX compliance checking
- User confirmation workflows
- Risk assessment and color-coded output

### Phase 3: Remote Backends
- vLLM HTTP API integration
- Ollama local API support
- Error handling and retry mechanisms
- Response format standardization

### Phase 4: MLX Integration
- FFI bindings using cxx crate
- Metal Performance Shaders integration
- Unified memory architecture handling
- Apple Silicon performance optimization

## Key Dependencies

**Core:**
- `clap` - Command-line argument parsing
- `serde` + `serde_json` - JSON serialization
- `tokio` - Async runtime
- `anyhow` - Error handling
- `reqwest` - HTTP client for remote backends

**Platform-Specific:**
- `cxx` - Safe C++ FFI for MLX integration
- `directories` - Cross-platform directory management
- `colored` - Terminal color output

**Development:**
- `tokio-test` - Async testing utilities
- `tempfile` - Temporary file creation for tests

## Safety Validation Patterns

### Dangerous Commands to Block
- Filesystem destruction: `rm -rf /`, `rm -rf ~`
- Disk operations: `mkfs`, `dd if=/dev/zero`
- Fork bombs: `:(){ :|:& };:`
- System path modification: Operations on `/bin`, `/usr`, `/etc`
- Privilege escalation: `sudo su`, `chmod 777 /`

### POSIX Compliance Requirements
- Use standard utilities (ls, find, grep, awk, sed, sort)
- Proper path quoting for spaces and special characters
- Avoid bash-specific features for maximum portability
- Validate command syntax before execution

## System Prompt Template

The tool uses a strict system prompt for JSON-only responses:
- Single command generation with safety constraints
- POSIX-compliant utilities only
- Proper file path quoting
- Destructive operation avoidance
- Clear JSON format: `{"cmd": "command_here"}`

## Performance Requirements

### Startup Optimization
- Lazy loading of all dependencies
- Efficient JSON parsing with fallback strategies
- Minimal memory allocations during initialization
- Cached model loading when available

### Inference Performance
- MLX backend: < 2s on Apple Silicon
- Remote backends: < 5s with network latency
- Streaming support where beneficial
- Memory-conscious resource management

## Testing Strategy

### Unit Tests
- Safety pattern validation
- Command parsing and validation
- Configuration management
- Cache directory operations

### Integration Tests
- End-to-end command generation workflows
- Backend communication and error handling
- Cross-platform compatibility
- Performance benchmarks

### Property Tests
- Safety validation with random inputs
- POSIX compliance checking
- Error recovery mechanisms

## Specialized Agent Usage

When working on specific components:

- **Complex architecture changes**: Use `rust-cli-architect` agent
- **LLM integration & backends**: Use `llm-integration-expert` agent  
- **MLX/Apple Silicon features**: Use `macos-unix-systems-expert` agent
- **Test-driven development**: Use `tdd-rust-engineer` agent
- **Documentation updates**: Use `technical-writer` agent

## Quality Standards

- All public APIs must have documentation
- Comprehensive error handling with helpful messages
- No panics in production code - use `Result` types
- Memory safety without unnecessary allocations
- Security-first approach for system-level operations
- POSIX compliance for maximum portability

## Spec-Driven Development Workflows

This project uses **dual spec-driven workflows** optimized for different feature sizes:

### Spec-Kitty Workflow (Rapid Development)

**Use for**: Small/medium features (< 2 weeks), bug fixes, enhancements, parallel development

**Location**: `kitty-specs/` (git worktrees)

**Commands**: `/spec-kitty.*` slash commands in `.claude/commands/`

**Workflow**:
1. `bin/sk-new-feature "description"` - Creates isolated worktree
2. `/spec-kitty.specify` - Create spec.md
3. `/spec-kitty.plan` - Create plan.md
4. `/spec-kitty.tasks` - Generate work packages
5. `/spec-kitty.implement` - Execute tasks
6. `/spec-kitty.accept` - Run acceptance checks
7. `/spec-kitty.merge` - Merge and cleanup worktree

**Benefits**:
- ✅ Parallel development (multiple features simultaneously)
- ✅ No branch switching overhead
- ✅ Real-time dashboard (http://127.0.0.1:9237)
- ✅ Automated task management
- ✅ Perfect for rapid iteration with tools like Charm.land Crush

**Dashboard**: `bin/sk-dashboard` to monitor all features

### Spec-Kit Workflow (Large Features)

**Use for**: Large features (> 2 weeks), major architecture changes, extensive research

**Location**: `specs/` (traditional directories)

**Commands**: Custom slash commands in `.codex/prompts/`

**Workflow**:
1. Manual directory creation in `specs/NNN-feature-name/`
2. Create spec.md, plan.md, tasks.md manually
3. Use `.specify/templates/` for structure
4. Follow `.specify/memory/constitution.md` principles
5. Standard git workflow on feature branches

**Benefits**:
- ✅ Better for complex, long-running features
- ✅ Explicit constitution-based governance
- ✅ Flexible structure for research-heavy work

### Decision Matrix: Which Workflow?

| Criteria | Spec-Kitty | Spec-Kit |
|----------|------------|----------|
| **Feature size** | < 2 weeks | > 2 weeks |
| **Complexity** | Low-Medium | High |
| **Parallel dev** | Multiple features at once | One at a time |
| **Research phase** | Light research | Extensive research |
| **Architecture** | Incremental changes | Major refactoring |
| **Examples** | Add caching, Fix bug, New API endpoint | MLX backend, Safety system, Multi-backend |

### When to Use Spec-Kitty

✅ **DO use spec-kitty when:**
- Adding a new feature that takes < 2 weeks
- Fixing a bug that requires changes across multiple files
- Building an enhancement to existing functionality
- Working on multiple features in parallel (e.g., with Charm.land Crush)
- You want visual tracking via the dashboard
- The feature has clear, well-defined scope

**Example scenarios**:
- "Add Redis caching with TTL support"
- "Fix memory leak in MLX initialization"
- "Add Prometheus metrics endpoint"
- "Implement command history feature"
- "Add JSON output format option"

### When to Use Spec-Kit

✅ **DO use spec-kit when:**
- Building a major new system (> 2 weeks)
- Extensive research or prototyping needed
- Architectural decisions require deep investigation
- Multiple competing approaches need evaluation
- Long-running feature with many unknowns

**Example scenarios**:
- "Implement complete MLX backend with C++ FFI"
- "Design and build multi-backend inference system"
- "Create comprehensive safety validation framework"
- "Research and implement model quantization pipeline"
- "Architect distributed caching system"

### Both Workflows Coexist

The project supports **both workflows simultaneously**:
- `kitty-specs/` for rapid, parallel development
- `specs/` for large, complex features

**Example**: You can work on a large MLX backend feature in `specs/004-implement-ollama-and/` while simultaneously using spec-kitty for quick bug fixes in `kitty-specs/001-fix-memory-leak/`.

### Integration Points

**Shared resources**:
- Both follow the same constitution principles (`.specify/memory/constitution.md`)
- Both use the same testing standards
- Both require security-first approach
- Both commit to the same git repository

**Different tools**:
- Spec-kitty: Automated task management, worktrees, dashboard
- Spec-kit: Manual planning, traditional branches, constitution-driven

### Quick Reference

```bash
# Spec-Kitty workflow
bin/sk-new-feature "Add caching"      # Create feature
cd kitty-specs/001-add-caching/       # Enter worktree
/spec-kitty.specify                   # Generate spec
/spec-kitty.implement                 # Execute tasks
bin/sk-dashboard                      # Monitor progress

# Spec-Kit workflow
mkdir -p specs/005-new-feature/       # Create directory
# Manually create spec.md, plan.md
# Use .specify/templates/ as reference
# Follow constitution-based development
```

See `docs/SPEC_KITTY_GUIDE.md` for comprehensive spec-kitty documentation.

## Project Management Workflow

This project uses a structured project management system with GitHub Projects, milestones, and roadmap tracking. The `/caro.roadmap` skill helps agents select work aligned with project priorities.

### Quick Start: Using /caro.roadmap

**Before starting any work**, use `/caro.roadmap` to select aligned tasks:

```bash
# 1. Check roadmap status and get recommendation
/caro.roadmap              # Shows milestones, progress, blockers, and suggested next work

# 2. Get next recommended work (auto-selected by priority algorithm)
/caro.roadmap next         # Returns highest-scored issue with breakdown

# 3. Start work on recommended issue (auto-routes to spec-kitty or spec-kit)
/caro.roadmap start #123   # Creates worktree or spec directory, updates issue status

# 4. When done, mark complete
/caro.roadmap complete #123 # Closes issue, updates roadmap, suggests next work
```

**Typical workflow**:
1. Run `/caro.roadmap` → See status
2. Run `/caro.roadmap next` → Get top recommendation
3. Run `/caro.roadmap start #XXX` → Begin implementation
4. Implement the feature
5. Run `/caro.roadmap complete #XXX` → Mark done, get next task

**Optional - Set your expertise** for better work matching:
```bash
/caro.roadmap profile rust     # If working on Rust/CLI
/caro.roadmap profile docs     # If writing documentation
/caro.roadmap profile devops   # If doing CI/CD/releases
```

### Roadmap Structure

**ROADMAP.md** defines three release milestones:
- **v1.1.0** (Feb 15, 2026): Core improvements - production-ready functionality
- **v1.2.0** (Mar 31, 2026): Website & docs launch - public marketing
- **v2.0.0** (Jun 30, 2026): Advanced features - innovation and capabilities

**GitHub Projects**:
- [Caro Product Development](https://github.com/users/wildcard/projects/2) - Technical work (36 items)
- [Caro Marketing & DevRel](https://github.com/users/wildcard/projects/3) - Content work (29 items)

Each project uses custom fields:
- **Status**: Todo, In Progress, Done
- **Priority**: Critical, High, Medium, Low, Backlog
- **Type**: Feature, Bug, Infrastructure, Research, Documentation, Marketing
- **Area**: Core CLI, Safety, Backends, DevOps, DX, Website

### The /caro.roadmap Skill

Use `/caro.roadmap` to intelligently select and manage work:

```bash
/caro.roadmap              # Show roadmap status overview
/caro.roadmap next         # Get next recommended work item
/caro.roadmap select       # Interactive work selection
/caro.roadmap start #123   # Begin work on issue (routes to spec-kitty or spec-kit)
/caro.roadmap complete #123 # Mark issue as done
/caro.roadmap blocked      # List all release blockers
/caro.roadmap profile      # Show/set agent expertise
```

### Work Selection Algorithm

The skill uses a weighted scoring system to recommend work:

1. **Blocker Check** (+1000): Issues labeled `release-blocker` take absolute priority
2. **Milestone Priority** (+100-300): Earlier milestones ranked higher (v1.1.0 > v1.2.0 > v2.0.0)
3. **Priority Level** (+10-50): Critical > High > Medium > Low
4. **Area Matching** (+25): Matches agent expertise to issue area
5. **Status Filter**: Only suggests "Todo" items, skips "blocked" or assigned items

### Agent Expertise Profiles

Configure your expertise in `.claude/agent-profiles.yaml` to get better work matches:

**Available profiles**:
- `rust` - Rust CLI Expert (areas: Core CLI, Backends, Safety)
- `docs` - Documentation Writer (areas: DX, Website)
- `devops` - DevOps Engineer (areas: DevOps, Infrastructure)
- `web` - Web Developer (areas: Website, DX)
- `marketing` - Marketing Specialist (areas: Website, DX)
- `security` - Security Engineer (areas: Safety, Core CLI)
- `ai` - AI/ML Engineer (areas: Backends, Core CLI)

Switch profiles with: `/caro.roadmap profile <name>`

### Workflow Routing: Spec-Kitty vs Spec-Kit

When starting work, the skill automatically routes to the appropriate workflow:

| Criteria | Spec-Kitty | Spec-Kit |
|----------|------------|----------|
| **Scope** | < 2 weeks (small/medium) | > 2 weeks (large) |
| **Complexity** | Low-Medium | High |
| **Labels** | `quick-fix`, `enhancement`, `bug` | `architecture`, `research`, `major-refactor` |
| **Workflow** | Worktree-based rapid iteration | Constitution-based manual process |

**Spec-Kitty routing** (automatic):
```bash
/caro.roadmap start #123
# → Creates .worktrees/NNN-feature/
# → Suggests: /caro.feature
```

**Spec-Kit routing** (manual):
```bash
/caro.roadmap start #456
# → Creates specs/NNN-feature/ directory
# → Follow .specify/memory/constitution.md
```

### Integration with Development Workflow

1. **Before starting work**: Check roadmap and select aligned issue
   ```bash
   /caro.roadmap              # View current status
   /caro.roadmap next         # Get recommended work
   ```

2. **Start implementation**: Route to appropriate workflow
   ```bash
   /caro.roadmap start #123   # Auto-routes to spec-kitty or spec-kit
   ```

3. **Complete work**: Update status and get next task
   ```bash
   /caro.roadmap complete #123
   ```

4. **Check blockers**: Before releases, verify no blockers
   ```bash
   /caro.roadmap blocked
   ```

This ensures all work aligns with project milestones, priorities, and strategic themes documented in ROADMAP.md.

## Release Management Workflow

This project enforces a **security-first release workflow** using Claude Code slash commands. All releases MUST go through feature branches and pull requests - direct commits to `main` for release-related changes are prohibited.

### Release Skills

The release workflow is automated through 6 Claude skills in `.claude/commands/`:

1. **`/caro.release.prepare`** - Start a new release
   - Creates `release/vX.Y.Z` branch from main
   - Runs pre-flight checks (CI status, release blockers)
   - Lists pending changes since last tag
   - **Prerequisite**: Must be on `main` branch with clean working directory

2. **`/caro.release.security`** - Security audit and fixes
   - Runs `cargo audit` and categorizes vulnerabilities
   - Guides through fixing critical/unsound issues
   - Updates dependencies to maintained versions
   - Runs tests and creates detailed commit
   - **Prerequisite**: Must be on `release/*` or `hotfix/*` branch

3. **`/caro.release.version`** - Version bump and changelog
   - Updates version in `Cargo.toml`
   - Updates `CHANGELOG.md` (moves [Unreleased] to [X.Y.Z])
   - Runs `cargo check` for verification
   - Creates version bump commit
   - **Prerequisite**: Must be on `release/*` or `hotfix/*` branch

4. **`/caro.release.publish`** - Create PR, merge, and tag
   - Creates pull request with release checklist
   - Monitors CI checks and waits for approval
   - Merges PR to main
   - Creates and pushes annotated git tag
   - Monitors automated publish workflows
   - **Prerequisite**: Must be on `release/*` or `hotfix/*` branch

5. **`/caro.release.verify`** - Post-release verification
   - Installs from crates.io and verifies version
   - Runs functionality tests
   - Checks GitHub release creation
   - Verifies documentation links
   - **Prerequisite**: None (can run from any branch)

6. **`/caro.release.hotfix`** - Emergency security patches
   - Creates hotfix branch from latest tag
   - Fast-tracks critical security fixes
   - Publishes security advisories
   - **Use ONLY for**: Critical vulnerabilities, data loss, crashes
   - **Prerequisite**: None (emergency mode)

### Standard Release Flow

Execute commands in this order:

```bash
# 1. Start release (creates release/vX.Y.Z branch)
/caro.release.prepare

# 2. Run security audit and fix vulnerabilities
/caro.release.security

# 3. Bump version and update changelog
/caro.release.version

# 4. Create PR, merge, tag, and publish
/caro.release.publish

# 5. Verify published release
/caro.release.verify
```

### Emergency Hotfix Flow

For critical security vulnerabilities only:

```bash
# Creates hotfix branch, applies fix, and fast-tracks release
/caro.release.hotfix
```

### Branch Enforcement

Each command enforces branch requirements:
- **prepare**: Must start on `main`
- **security, version, publish**: Must be on `release/*` or `hotfix/*`
- **verify**: Can run from any branch
- **hotfix**: Can start from any branch (emergency mode)

Commands will **REFUSE to proceed** if branch requirements aren't met, preventing accidental direct commits to `main`.

### Design Principles

- **Security-first**: Mandatory security audits before every release
- **Consistency**: Same process every time, no missed steps
- **Transparency**: All actions documented in commits
- **Enforcement**: Branch protection enforced by commands
- **Automation**: Reduces manual errors

See `docs/RELEASE_PROCESS.md` for complete release procedures and security requirements.

## Session Continuity (Continuous-Claude)

This project integrates **Continuous-Claude** for session state preservation across context clears. Instead of relying on lossy compaction (summarizing conversations repeatedly), we use a "clear, don't compact" philosophy.

### The Problem

When Claude Code runs low on context, it compacts (summarizes) conversations. Multiple compactions create "a summary of a summary of a summary," degrading quality and eventually producing hallucinations.

### The Solution

Save state to a **ledger**, clear context cleanly, and resume with full signal integrity.

### Directory Structure

```
thoughts/
├── ledgers/                    # In-session state files (survive /clear)
│   └── CONTINUITY_CLAUDE-*.md  # Active session ledgers
└── shared/
    ├── handoffs/               # Cross-session transfer documents
    ├── plans/                  # Implementation plans
    └── research/               # Research documents
```

### Core Skills

**Continuity Ledger** (`/continuity_ledger`)
- Creates/updates ledgers for state preservation within a session
- Use before running `/clear`
- Use when context usage approaches 70%+
- Ledgers survive `/clear` and reload automatically on resume

**Create Handoff** (`/create_handoff`)
- Creates end-of-session transfer documents
- Includes task status, learnings, artifacts, and next steps
- Perfect for handing off work to another session

**Resume Handoff** (`/resume_handoff`)
- Resumes work from a handoff document
- Validates current state against handoff
- Creates todo list from action items

**Onboard** (`/onboard`)
- Analyzes brownfield codebases
- Creates initial continuity ledger
- Use when first working in an existing project

### Natural Language Triggers

The system responds to conversational cues:

| Phrase | Action |
|--------|--------|
| "save state", "update ledger" | Updates continuity ledger |
| "done for today", "create handoff" | Creates handoff document |
| "resume work", "continue from handoff" | Loads and continues |
| "onboard", "analyze this project" | Runs codebase analysis |

### When to Use Continuity

**Use ledgers when:**
- Context usage approaching 70%+
- Multi-day implementations
- Complex refactors you pick up/put down
- Any session expected to hit 85%+ context

**Use handoffs when:**
- Ending a work session
- Transferring work to another session/person
- Need detailed context for future work

**Don't use when:**
- Quick tasks (< 30 min)
- Simple bug fixes
- Single-file changes

### Quick Reference

```bash
# Save state before clearing context
/continuity_ledger

# Clear context (ledger reloads automatically)
/clear

# Create end-of-session handoff
/create_handoff

# Resume from a handoff
/resume_handoff thoughts/shared/handoffs/feature-name/2025-01-15_14-30-00_description.md

# Onboard to a new codebase
/onboard
```

### Comparison with Other Tools

| Tool | Scope | Fidelity |
|------|-------|----------|
| CLAUDE.md | Project | Always fresh, stable patterns |
| TodoWrite | Turn | Survives compaction, but understanding degrades |
| CONTINUITY_CLAUDE-*.md | Session | External file - never compressed, full fidelity |
| Handoffs | Cross-session | External file - detailed context for new session |

See `.claude/skills/` for detailed skill documentation.

## Multi-Agent Development Process

This project follows spec-driven development with coordinated multi-agent teams:
1. Specification phase with clear requirements
2. Architecture and design review
3. Phased implementation with safety validation
4. Quality assurance and documentation

Each phase includes specific agent coordination for optimal development flow and maintains alignment with project constitution and safety standards.

## PRD-First Feature Development

**Rule**: All new features with cultural, regional, or significant user-facing impact MUST follow PRD-first development.

### When to Create a PRD

Create a PRD before implementation when:
- Adding holiday themes or cultural features
- Building features that affect users from specific regions/cultures
- Implementing accessibility-sensitive features
- Adding features with localization requirements
- Creating features that require cultural research or sensitivity review

### PRD Workflow

1. **Create PRD** in appropriate directory:
   - Holiday themes: `docs/prds/holidays/`
   - Localization: `docs/prds/i18n/`
   - Accessibility: `docs/prds/a11y/`
   - General features: `docs/prds/features/`

2. **PRD Approval**: Get stakeholder review before implementation

3. **Route to Spec Workflow**:
   - Small/medium features (< 2 weeks): Use Spec-Kitty (`/caro.feature`)
   - Large features (> 2 weeks): Use Spec-Kit (`specs/` directory)

4. **Implementation**: Follow the chosen spec workflow

5. **Cultural Review** (if applicable): Verify cultural accuracy before launch

### PRD Template Location

See `website/GLOBAL_HOLIDAY_THEMES_PLAN.md` for the holiday theme PRD template.

### Example: Adding a New Holiday Theme

```bash
# 1. Create PRD
mkdir -p docs/prds/holidays
# Write PRD based on template in GLOBAL_HOLIDAY_THEMES_PLAN.md

# 2. After PRD approval, start implementation
/caro.feature  # For spec-kitty workflow

# 3. Follow cultural sensitivity guidelines
# 4. Get cultural review before merge
```

This ensures cultural sensitivity, user experience quality, and proper documentation for all culturally-significant features.
