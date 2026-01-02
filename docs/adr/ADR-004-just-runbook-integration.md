# ADR-004: Just-Based Runbook Integration with Workspace-Scoped Command History

**Status**: Proposed

**Date**: 2026-01-02

**Authors**: @wildcard

**Target**: Community

## Context

Caro currently generates shell commands from natural language descriptions on a per-request basis. While this provides immediate value, users frequently need to:

1. **Re-run previously generated commands** without re-invoking the LLM
2. **Build project-specific command collections** for recurring workflows
3. **Share command recipes** across teams or persist them in version control
4. **Discover relevant commands** based on project context and history

### Current Situation

- Each `caro` invocation is stateless—commands are generated, optionally executed, and forgotten
- Users manually copy commands to scripts, aliases, or Makefiles for reuse
- No correlation exists between command generation history and project context
- Platform-specific command variations are re-computed on each invocation

### Stakeholders

- **Individual developers**: Need fast access to frequently-used project commands
- **Teams**: Want standardized, shareable command recipes per project
- **DevOps engineers**: Seek automation without maintaining complex shell scripts
- **Caro power users**: Desire workflow acceleration and history-based suggestions

### Technical Drivers

1. **Workspace scoping**: Caro already detects execution context (`ExecutionContext::detect()`) including current working directory, which naturally maps to project workspaces
2. **Git repository awareness**: Most developer projects are git repositories, providing a natural workspace boundary
3. **Command generation patterns**: Users tend to generate similar commands for similar projects (build, test, lint, deploy)
4. **Just compatibility**: The `just` command runner is Rust-native, cross-platform, and has strong backwards-compatibility guarantees

### Forces at Play

- **Simplicity vs. power**: Users want one-command access without complex configuration
- **Persistence vs. privacy**: Some commands should persist to disk; others should remain ephemeral
- **Automation vs. control**: Auto-generated runbooks must be editable and overridable
- **Portability vs. platform-specificity**: Justfiles should work across platforms while respecting Caro's platform-aware generation

### Assumptions

1. Users invoke Caro repeatedly within the same project directories
2. Git repository root is a reliable workspace boundary when present
3. The `just` command runner is acceptable as a dependency or optional integration
4. Users want both persistent (justfile) and virtual (in-memory) storage options

## Decision

Implement a **workspace-scoped command history and runbook generation system** that:

1. **Tracks command generation history** per workspace (directory-based, git-aware)
2. **Correlates commands with workspace context** (project type, common tools, usage patterns)
3. **Generates justfiles** (runbooks) from history—either on-demand or automatically
4. **Provides dual storage modes**: filesystem (`.justfile`) or virtual (Caro's internal database)
5. **Enables alias-based execution** of historical/frequent commands via `just` integration

### Core Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CARO CLI                                  │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────┐  │
│  │ NL → Command  │  │ Workspace     │  │ Runbook Generator   │  │
│  │ Generator     │──│ History Store │──│ (Just Integration)  │  │
│  └───────────────┘  └───────────────┘  └─────────────────────┘  │
│         │                   │                     │              │
│         ▼                   ▼                     ▼              │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                  Workspace Manager                           ││
│  │  • Git repository detection                                  ││
│  │  • Directory-based workspace ID                              ││
│  │  • Project type inference (Cargo.toml, package.json, etc.)   ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
   │  Workspace  │     │   Virtual   │     │  Generated  │
   │   History   │     │   Runbook   │     │  Justfile   │
   │  (SQLite)   │     │ (In-Memory) │     │   (File)    │
   └─────────────┘     └─────────────┘     └─────────────┘
```

### Workspace Identification Strategy

```rust
pub struct Workspace {
    /// Canonical workspace identifier (git root or directory hash)
    pub id: WorkspaceId,

    /// Absolute path to workspace root
    pub root: PathBuf,

    /// True if workspace is a git repository
    pub is_git_repo: bool,

    /// Detected project type(s)
    pub project_types: Vec<ProjectType>,

    /// Workspace-specific configuration
    pub config: WorkspaceConfig,
}

pub enum ProjectType {
    Rust,      // Cargo.toml present
    Node,      // package.json present
    Python,    // pyproject.toml, setup.py, or requirements.txt
    Go,        // go.mod present
    Generic,   // No specific detection
}
```

### Command History Schema

```rust
pub struct HistoricalCommand {
    /// Unique identifier
    pub id: Uuid,

    /// Workspace this command belongs to
    pub workspace_id: WorkspaceId,

    /// Original natural language prompt
    pub prompt: String,

    /// Generated shell command
    pub command: String,

    /// Command explanation from LLM
    pub explanation: Option<String>,

    /// Safety validation result
    pub risk_level: RiskLevel,

    /// Whether the command was executed
    pub was_executed: bool,

    /// Execution outcome if run
    pub execution_result: Option<ExecutionOutcome>,

    /// Platform context when generated
    pub platform: PlatformContext,

    /// Generation timestamp
    pub created_at: DateTime<Utc>,

    /// Usage count for frequency tracking
    pub usage_count: u32,

    /// User-assigned alias (optional)
    pub alias: Option<String>,

    /// User-assigned tags for categorization
    pub tags: Vec<String>,
}
```

### Justfile Generation

Caro will generate justfiles in the standard `just` format:

```just
# Auto-generated by Caro - https://github.com/wildcat/caro
# Workspace: /Users/dev/myproject
# Generated: 2026-01-02T15:30:00Z

# ─────────────────────────────────────────────────────────────────
# Build & Test
# ─────────────────────────────────────────────────────────────────

# Build the project in release mode
build:
    cargo build --release

# Run all tests with verbose output
test:
    cargo test --verbose

# Run tests and watch for changes
test-watch:
    cargo watch -x test

# ─────────────────────────────────────────────────────────────────
# Code Quality
# ─────────────────────────────────────────────────────────────────

# Format code with rustfmt
fmt:
    cargo fmt --all

# Run clippy with strict settings
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# ─────────────────────────────────────────────────────────────────
# Custom Commands (from history)
# ─────────────────────────────────────────────────────────────────

# Find large files (>10MB) - generated from: "find files larger than 10MB"
find-large-files:
    find . -type f -size +10M -exec ls -lh {} \;

# Monitor memory usage - generated from: "watch memory usage"
mem-watch:
    watch -n 1 'free -h'
```

### CLI Integration

New subcommands for runbook management:

```bash
# Generate runbook from history
caro runbook generate              # Interactive: select commands to include
caro runbook generate --all        # Include all historical commands
caro runbook generate --frequent   # Include top 10 most-used commands
caro runbook generate --auto       # AI-curated based on project type

# Manage runbook storage
caro runbook save                  # Save to .justfile in workspace root
caro runbook save --virtual        # Keep in Caro's internal database only
caro runbook export path/to/file   # Export to specific location

# List and manage historical commands
caro history                       # List command history for current workspace
caro history --all                 # List across all workspaces
caro history alias <id> <name>     # Assign alias to historical command
caro history tag <id> <tags...>    # Tag command for categorization
caro history delete <id>           # Remove from history

# Quick execution via aliases
caro run <alias>                   # Execute command by alias
caro run --list                    # List available aliases

# Workspace management
caro workspace                     # Show current workspace info
caro workspace list                # List all known workspaces
caro workspace forget <id>         # Remove workspace and its history
```

### Smart Suggestions

Caro will proactively suggest relevant commands based on:

1. **Project type detection**: Rust projects get `cargo` commands, Node projects get `npm`/`yarn`
2. **Frequency analysis**: Surface commonly-used commands for easy access
3. **Recency**: Recently generated commands are more relevant
4. **Peer learning** (future): Anonymized patterns from similar project types

```bash
$ caro suggest
Based on your Rust project and history:

  build     → cargo build --release
  test      → cargo test --verbose
  lint      → cargo clippy --all-targets
  fmt       → cargo fmt --all

  [h]istory commands:
  find-logs → find . -name "*.log" -mtime -7

Run with: just <recipe> or caro run <alias>
```

## Rationale

### Why `just`?

| Criteria | Make | `just` | npm scripts | Task (go-task) |
|----------|------|--------|-------------|----------------|
| Cross-platform | ⚠️ Varies | ✅ Full | ✅ Full | ✅ Full |
| Language | Custom DSL | Make-inspired, cleaner | JSON | YAML |
| Rust ecosystem | ❌ External | ✅ Native crate | ❌ Node dep | ❌ Go binary |
| Stability guarantee | N/A | ✅ No breaking changes | ⚠️ Depends | ⚠️ Newer |
| Recipe parameters | ⚠️ Hacky | ✅ First-class | ⚠️ Limited | ✅ Good |
| Shell flexibility | ❌ sh-focused | ✅ Any shell/language | ❌ npm context | ✅ Good |
| `.env` loading | ❌ Manual | ✅ Automatic | ⚠️ Via dotenv | ✅ Yes |
| Crate availability | N/A | ✅ `just` crate | N/A | N/A |

**Key advantages of `just`**:

1. **Rust-native**: Available as a crate for tight integration
2. **Cross-platform**: Consistent behavior on Linux, macOS, Windows
3. **Simple syntax**: Easier than Make, familiar to developers
4. **Backwards-compatible**: Explicit stability promise—justfiles won't break
5. **Shebang recipes**: Support for Python, Node, Ruby, etc. in recipes
6. **`.env` integration**: Automatic environment variable loading
7. **Subdirectory support**: Works from any project subdirectory

### Why Workspace Scoping?

1. **Natural boundary**: Projects have distinct command needs
2. **Git awareness**: Most projects are repos; git root is unambiguous
3. **Privacy isolation**: Command history stays project-local
4. **Context relevance**: Commands generated for one project rarely apply to others
5. **Team sharing**: Justfiles can be committed and shared via version control

### Why Dual Storage Modes?

1. **Persistence preference varies**: Some users want justfiles in git; others want ephemeral
2. **Team vs. individual**: Committed justfiles are team resources; virtual is personal
3. **Sensitive commands**: Some commands shouldn't be persisted to disk
4. **Experimentation**: Virtual mode allows testing without file clutter

### Alignment with Caro's Mission

- **Safety-first**: Only validated commands make it to runbooks
- **Local-first**: All history stored locally; no cloud dependency
- **CLI-native**: Seamless terminal workflow integration
- **Platform-aware**: Generated commands respect platform differences

## Consequences

### Benefits

1. **Workflow acceleration**: One-command access to previously-generated recipes
2. **Reduced LLM calls**: Cached commands don't need regeneration
3. **Team standardization**: Shared justfiles ensure consistent workflows
4. **Discoverability**: `just --list` shows all available project commands
5. **Version control friendly**: Justfiles are text, diffable, reviewable
6. **Cross-platform recipes**: `just` handles platform differences
7. **Learning effect**: Repeated commands reinforce terminal knowledge
8. **Offline capability**: Historical commands work without LLM access

### Trade-offs

1. **Optional dependency**: Users need `just` installed for full functionality
   - Mitigation: Caro can execute commands directly; `just` is for recipe management
2. **Storage overhead**: SQLite database for history (~10-100KB per workspace)
   - Mitigation: Configurable retention; automatic cleanup of old entries
3. **Complexity increase**: New concepts (workspaces, runbooks) to learn
   - Mitigation: Progressive disclosure; simple defaults with advanced options
4. **Justfile conflicts**: Projects may have existing justfiles
   - Mitigation: Merge mode, separate file option (`.caro.just`), or virtual-only

### Risks

1. **History privacy leakage**: Command history could contain sensitive data
   - Mitigation: Local-only storage; explicit export required; `--no-history` flag

2. **Stale commands**: Generated commands may become outdated
   - Mitigation: Timestamp display; re-generation option; validation on execution

3. **`just` dependency fragility**: External tool dependency
   - Mitigation: `just` has strong stability guarantees; Caro fallback execution

4. **Workspace detection edge cases**: Non-git directories, nested repos
   - Mitigation: Explicit workspace registration; configuration overrides

## Alternatives Considered

### Alternative 1: Custom Script Format

- **Description**: Create Caro-specific runbook format instead of justfiles
- **Pros**: Full control over format and features; no external dependency
- **Cons**: Yet another format to learn; no ecosystem; reinventing the wheel
- **Decision**: Rejected—`just` is mature, well-documented, and Rust-native

### Alternative 2: Makefile Generation

- **Description**: Generate Makefiles instead of justfiles
- **Pros**: Ubiquitous; no new tool needed
- **Cons**: Complex syntax; `.PHONY` required; cross-platform issues; tabs vs. spaces
- **Decision**: Rejected—Make's quirks outweigh its ubiquity for this use case

### Alternative 3: Shell Script Generation

- **Description**: Generate `.sh` scripts with command functions
- **Pros**: No dependencies; universally executable
- **Cons**: Manual sourcing; no listing; platform-specific; less discoverable
- **Decision**: Rejected—shell scripts lack the recipe/alias UX of task runners

### Alternative 4: Integration with Existing Task Runners

- **Description**: Support multiple formats (Make, npm scripts, Taskfile.yml)
- **Pros**: Flexibility; works with existing setups
- **Cons**: Complexity explosion; inconsistent features; maintenance burden
- **Decision**: Deferred—start with `just`, consider multi-format later

### Alternative 5: No Persistence (Stateless)

- **Description**: Keep Caro stateless; users manage their own persistence
- **Pros**: Simplicity; no storage management
- **Cons**: Loses major UX opportunity; forces manual workflow on users
- **Decision**: Rejected—the value proposition of history-based runbooks is compelling

## Implementation Notes

### Phase 1: Workspace & History Foundation

1. **Workspace detection module** (`src/workspace/mod.rs`)
   - Git repository detection via `.git` directory
   - Project type inference from manifest files
   - Workspace ID generation (content-hash of canonical path)

2. **History storage** (`src/history/mod.rs`)
   - SQLite database using `rusqlite` or `sqlx`
   - Schema migrations for future extensibility
   - Configurable retention policies

3. **CLI extensions**
   - `caro history` subcommand family
   - `caro workspace` subcommand family
   - History opt-out via `--no-history` flag

### Phase 2: Runbook Generation

1. **Justfile generator** (`src/runbook/just.rs`)
   - Template-based generation with categories
   - Smart recipe naming (slug from prompt or alias)
   - Conflict detection with existing justfiles

2. **Storage modes**
   - Filesystem writer for `.justfile` or `.caro.just`
   - Virtual storage in SQLite for ephemeral recipes

3. **CLI extensions**
   - `caro runbook generate` with interactive mode
   - `caro runbook save` / `caro runbook export`

### Phase 3: Smart Features

1. **Suggestion engine**
   - Project-type-based default recipes
   - Frequency-weighted historical commands
   - Time-decay for recency scoring

2. **Alias system**
   - Short names for common commands
   - `caro run <alias>` quick execution

3. **Integration hooks**
   - Post-generation auto-add to runbook option
   - Configurable auto-save behavior

### Testing Strategy

- **Unit tests**: Workspace detection, history storage, justfile generation
- **Integration tests**: Full workflow from generation to runbook
- **Property tests**: History schema invariants, justfile syntax validity
- **Platform tests**: Cross-platform workspace detection and justfile execution

### Migration Path

- Existing users: No migration required; history starts fresh
- Workspace detection: Automatic on first use in a directory
- Configuration: Sensible defaults; advanced options in config file

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Runbook adoption | 30% of active users generate at least one runbook | Telemetry (opt-in) |
| History query latency | < 50ms for workspace history retrieval | Performance tests |
| Repeat command ratio | 40% of executions via aliases vs. new generation | Usage analytics |
| Justfile generation success | 99% of generations produce valid justfiles | CI validation |
| Cross-platform parity | 100% feature parity on Linux/macOS/Windows | E2E test matrix |

## Business Implications

### Community Value

- **Differentiation**: No other NL→command tool offers integrated runbook generation
- **Stickiness**: History and runbooks create valuable user investment
- **Shareability**: Justfiles in git repos advertise Caro to team members
- **Power user appeal**: Advanced workflow automation attracts CLI enthusiasts

### Future Enterprise Opportunities

- **Team runbooks**: Shared, governed command libraries
- **Audit trails**: Command history for compliance
- **Runbook templates**: Pre-built recipes for common enterprise workflows
- **Analytics**: Aggregated insights on command patterns (opt-in, anonymized)

## References

### Related ADRs

- [ADR-001: LLM Inference Architecture](./ADR-001-enterprise-community-architecture.md) — Backend architecture
- [ADR-003: Monitoring and Audit Trail](./ADR-003-monitoring-audit-trail.md) — Relevant for history storage

### External Resources

- [Just Command Runner](https://github.com/casey/just) — Official repository
- [Just Manual](https://just.systems/man/en/) — Comprehensive documentation
- [Just on crates.io](https://crates.io/crates/just) — Rust crate
- [Justfile Cheat Sheet](https://cheatography.com/linux-china/cheat-sheets/justfile/) — Quick reference

### Prior Art

- [Warp AI](https://www.warp.dev/) — Terminal with AI command suggestions (cloud-based)
- [Fig](https://fig.io/) — Terminal autocomplete with history (acquired by AWS)
- [Atuin](https://github.com/atuinsh/atuin) — Shell history sync and search
- [direnv](https://direnv.net/) — Directory-scoped environment management

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-02 | @wildcard | Initial draft |
