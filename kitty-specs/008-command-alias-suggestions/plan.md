# Implementation Plan: Intelligent Command Alias Suggestions

**Feature ID**: 008-command-alias-suggestions
**Plan Version**: 1.0
**Created**: 2025-12-31

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Caro CLI                                   │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Command      │  │ Tips Engine  │  │ Installation             │  │
│  │ Execution    │──│              │──│ Automation               │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
│         │                 │                      │                  │
│         ▼                 ▼                      ▼                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    Shell Intelligence                         │  │
│  │  ┌───────────────┐  ┌───────────────┐  ┌─────────────────┐   │  │
│  │  │ Shell         │  │ Alias         │  │ Plugin          │   │  │
│  │  │ Detector      │  │ Parser        │  │ Detector        │   │  │
│  │  └───────────────┘  └───────────────┘  └─────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    Knowledge Base                             │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │  │
│  │  │ Local Cache │  │ Community   │  │ Cheatsheet          │   │  │
│  │  │ (SQLite)    │  │ Tips        │  │ Processor           │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    External Services                                │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────┐    │
│  │ GitHub Actions  │  │ Community       │  │ KB CDN           │    │
│  │ (KB Processing) │  │ Portal          │  │ (Releases)       │    │
│  └─────────────────┘  └─────────────────┘  └──────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── tips/
│   ├── mod.rs                 # Module exports
│   ├── engine.rs              # TipsEngine - main orchestrator
│   ├── shell/
│   │   ├── mod.rs             # Shell submodule
│   │   ├── detector.rs        # Detect shell type
│   │   ├── config_parser.rs   # Parse shell config files
│   │   ├── alias_parser.rs    # Extract aliases
│   │   └── plugin_detector.rs # Detect plugin managers
│   ├── kb/
│   │   ├── mod.rs             # Knowledge base submodule
│   │   ├── types.rs           # KB data structures
│   │   ├── cache.rs           # Local KB caching
│   │   ├── updater.rs         # Fetch KB updates
│   │   └── matcher.rs         # Match commands to tips
│   ├── suggestions/
│   │   ├── mod.rs             # Suggestions submodule
│   │   ├── alias_suggester.rs # Suggest aliases
│   │   ├── plugin_suggester.rs# Recommend plugins
│   │   └── display.rs         # Format suggestions for display
│   └── automation/
│       ├── mod.rs             # Automation submodule
│       ├── installer.rs       # Plugin installation flows
│       ├── config_editor.rs   # Safe config file editing
│       └── shell_reload.rs    # Reload shell config
├── kb/                        # Knowledge base data (embedded)
│   ├── cheatsheets/           # YAML cheatsheet sources
│   └── processed/             # Compiled KB (build artifact)
└── bin/
    └── kb-processor.rs        # KB build tool
```

## Implementation Phases

### Phase 1: Shell Intelligence Foundation

**Goal**: Detect shell, parse config, extract aliases

**Components**:

1. **ShellDetector** (`shell/detector.rs`)
   - Detect current shell from `$SHELL` environment
   - Map shell to config file paths
   - Handle edge cases (login vs non-login shells)

2. **ConfigParser** (`shell/config_parser.rs`)
   - Read shell config files (.zshrc, .bashrc, etc.)
   - Handle `source` and `.` includes
   - Parse environment variables

3. **AliasParser** (`shell/alias_parser.rs`)
   - Extract `alias foo='bar'` declarations
   - Handle different alias syntaxes (bash vs zsh vs fish)
   - Build alias lookup table

4. **PluginDetector** (`shell/plugin_detector.rs`)
   - Detect Oh My Zsh (`$ZSH/oh-my-zsh.sh`)
   - Detect Prezto (`~/.zprezto`)
   - Detect Zinit (`~/.zinit`)
   - Detect Fish plugins (`~/.config/fish/conf.d`)
   - List enabled plugins

**Data Structures**:

```rust
// shell/mod.rs
pub struct ShellEnvironment {
    pub shell_type: ShellType,
    pub shell_path: PathBuf,
    pub config_files: Vec<PathBuf>,
    pub aliases: HashMap<String, Alias>,
    pub plugin_manager: Option<PluginManager>,
    pub plugins: Vec<String>,
}

pub struct Alias {
    pub name: String,
    pub expansion: String,
    pub source: AliasSource,
    pub line_number: Option<usize>,
}

pub enum AliasSource {
    UserConfig(PathBuf),
    Plugin(String),
    System,
}
```

**Tests**:
- Unit tests for each shell type parsing
- Integration tests with mock config files
- Property tests for alias extraction

### Phase 2: Tips Engine Core

**Goal**: Generate contextual suggestions from local aliases

**Components**:

1. **TipsEngine** (`engine.rs`)
   - Orchestrate tip generation
   - Manage tip frequency and rate limiting
   - Track shown tips in session

2. **AliasSuggester** (`suggestions/alias_suggester.rs`)
   - Match commands against known aliases
   - Calculate keystroke savings
   - Generate suggestion message

3. **Display** (`suggestions/display.rs`)
   - Format tips for terminal output
   - Handle different output modes (inline, panel)
   - Colorize output appropriately

**Flow**:
```
Command executed -> TipsEngine.suggest(cmd)
                    -> AliasSuggester.match_aliases(cmd)
                    -> Display.format(suggestion)
                    -> Output to terminal
```

**Configuration**:
```rust
pub struct TipsConfig {
    pub enabled: bool,
    pub frequency: TipFrequency,
    pub max_per_session: usize,
    pub categories: TipCategories,
    pub show_savings: bool,
}

pub enum TipFrequency {
    Always,
    Sometimes(f32),  // probability 0.0-1.0
    Rarely,
    Never,
}
```

### Phase 3: Knowledge Base System

**Goal**: Build and distribute community tips

**Components**:

1. **KB Types** (`kb/types.rs`)
   - Define KB schema
   - Version information
   - Tip and alias structures

2. **KB Cache** (`kb/cache.rs`)
   - Local SQLite storage
   - Cache invalidation logic
   - Offline support

3. **KB Updater** (`kb/updater.rs`)
   - Fetch KB from GitHub releases
   - Verify integrity (SHA256)
   - Incremental updates

4. **KB Matcher** (`kb/matcher.rs`)
   - Match commands against KB patterns
   - Regex-based pattern matching
   - Priority and relevance scoring

**KB Format** (MessagePack for efficiency):
```rust
#[derive(Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub version: semver::Version,
    pub updated_at: DateTime<Utc>,
    pub checksum: String,
    pub tips: Vec<KbTip>,
    pub aliases: Vec<KbAlias>,
    pub plugins: Vec<KbPlugin>,
}
```

**GitHub Actions Workflow**:
```yaml
# .github/workflows/kb-build.yml
name: Build Knowledge Base

on:
  push:
    paths: ['kb/cheatsheets/**']
  workflow_dispatch:

jobs:
  build-kb:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build KB
        run: |
          cargo run --bin kb-processor -- \
            --input kb/cheatsheets/ \
            --output kb/processed/caro-kb.msgpack

      - name: Generate checksum
        run: sha256sum kb/processed/caro-kb.msgpack > kb/processed/checksum.txt

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: caro-knowledge-base
          path: kb/processed/
```

### Phase 4: Plugin Recommendations

**Goal**: Recommend and install productivity plugins

**Components**:

1. **PluginSuggester** (`suggestions/plugin_suggester.rs`)
   - Detect when user could benefit from plugins
   - Match usage patterns to plugin capabilities
   - Generate plugin recommendations

2. **Installer** (`automation/installer.rs`)
   - Define installation flows
   - Execute with user confirmation
   - Handle errors and rollback

3. **ConfigEditor** (`automation/config_editor.rs`)
   - Safe config file modification
   - Automatic backup creation
   - Atomic writes

4. **ShellReload** (`automation/shell_reload.rs`)
   - Source config files
   - Handle different shells
   - Verify changes took effect

**Installation Flow Example**:
```rust
pub struct InstallationPlan {
    pub name: String,
    pub description: String,
    pub prerequisites: Vec<Prerequisite>,
    pub steps: Vec<InstallStep>,
    pub verification: Vec<VerificationStep>,
    pub rollback: Option<RollbackPlan>,
}

// Oh My Zsh installation plan
fn ohmyzsh_plan() -> InstallationPlan {
    InstallationPlan {
        name: "Oh My Zsh".into(),
        prerequisites: vec![
            Prerequisite::ShellType(ShellType::Zsh),
            Prerequisite::CommandExists("curl".into()),
        ],
        steps: vec![
            InstallStep::Backup { path: "~/.zshrc".into() },
            InstallStep::Run {
                command: "sh -c \"$(curl -fsSL https://...)\"".into(),
                env: vec![("RUNZSH", "no")],
            },
            InstallStep::ConfigAdd {
                path: "~/.zshrc".into(),
                pattern: "^plugins=".into(),
                content: "plugins=(git)".into(),
            },
        ],
        verification: vec![
            VerificationStep::PathExists("~/.oh-my-zsh".into()),
            VerificationStep::ConfigContains {
                path: "~/.zshrc".into(),
                pattern: "oh-my-zsh.sh".into(),
            },
        ],
        rollback: Some(RollbackPlan {
            restore_backup: true,
            cleanup_paths: vec!["~/.oh-my-zsh".into()],
        }),
    }
}
```

### Phase 5: Community Features

**Goal**: Enable community contribution of tips and aliases

**Components**:

1. **Cheatsheet Format** (`kb/cheatsheets/`)
   - YAML-based contribution format
   - Schema validation
   - Documentation and examples

2. **Website Integration** (separate project)
   - `/community/contribute` page
   - Moderation dashboard
   - Contributor profiles

3. **CLI Contribution** (`caro community`)
   - Submit aliases from CLI
   - View contribution status
   - Local preview

**Cheatsheet Schema**:
```yaml
# kb/cheatsheets/docker-aliases.yaml
$schema: ./schema/cheatsheet.json
name: Docker Productivity Aliases
version: 1.0.0
author: contributor-name
shell: [zsh, bash]
requires:
  - docker

aliases:
  - name: dps
    expansion: docker ps
    description: List running containers

  - name: dex
    expansion: docker exec -it
    description: Execute command in container

tips:
  - id: docker-ps-alias
    pattern: "^docker ps$"
    message: "Use `dps` for faster container listing"
    category: alias_shortcut

  - id: docker-compose-plugin
    condition:
      command_frequency:
        pattern: "docker-compose"
        threshold: 5
    message: "Install Docker Compose as a plugin: `docker compose`"
    category: best_practice
```

## CLI Interface

**New Commands**:

```bash
# Tips commands
caro tips                      # Show a random tip
caro tips "command"            # Show tips for specific command
caro tips search "query"       # Search tips
caro tips history              # Show recently shown tips

# Alias commands
caro aliases list              # List all known aliases
caro aliases search "pattern"  # Search aliases
caro aliases suggest           # Suggest aliases for recent commands

# Knowledge base commands
caro kb update                 # Update knowledge base
caro kb status                 # Show KB version and stats
caro kb export                 # Export local aliases to cheatsheet

# Community commands
caro community contribute      # Start contribution wizard
caro community status          # Check contribution status
caro community browse          # Browse community tips
```

**Integration with Main Flow**:

```rust
// In main command execution flow
pub async fn execute_command(cmd: &str, config: &Config) -> Result<()> {
    // ... existing command execution ...

    // Post-execution tip suggestion
    if config.tips.enabled {
        if let Some(tip) = tips_engine.suggest(cmd).await? {
            if tips_engine.should_show(&tip) {
                tips_engine.display(&tip)?;
                tips_engine.record_shown(&tip);
            }
        }
    }

    Ok(())
}
```

## Data Storage

**Local Cache** (`~/.cache/caro/`):

```
~/.cache/caro/
├── kb/
│   ├── caro-kb.msgpack        # Cached knowledge base
│   └── checksum.txt           # KB checksum
├── tips/
│   ├── session.json           # Current session state
│   └── history.db             # SQLite - tip history
└── shell/
    ├── aliases.json           # Cached parsed aliases
    └── plugins.json           # Cached plugin detection
```

**SQLite Schema** (history.db):
```sql
CREATE TABLE tip_history (
    id INTEGER PRIMARY KEY,
    tip_id TEXT NOT NULL,
    shown_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    command TEXT,
    dismissed BOOLEAN DEFAULT FALSE
);

CREATE TABLE user_preferences (
    tip_id TEXT PRIMARY KEY,
    preference TEXT CHECK(preference IN ('like', 'dislike', 'hide')),
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tip_history_tip_id ON tip_history(tip_id);
CREATE INDEX idx_tip_history_shown_at ON tip_history(shown_at);
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    // Shell detection tests
    #[test]
    fn test_detect_zsh_shell() { ... }

    #[test]
    fn test_parse_bash_aliases() { ... }

    // Alias matching tests
    #[test]
    fn test_match_git_status_to_gst() { ... }

    // KB tests
    #[test]
    fn test_kb_pattern_matching() { ... }
}
```

### Integration Tests

```rust
// tests/integration/tips_test.rs
#[tokio::test]
async fn test_tip_suggestion_flow() {
    let shell_env = ShellEnvironment::mock_zsh_with_ohmyzsh();
    let engine = TipsEngine::new(shell_env);

    let tip = engine.suggest("git status").await.unwrap();
    assert_eq!(tip.suggestion_type, SuggestionType::AliasShortcut);
    assert_eq!(tip.alias, "gst");
}
```

### Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn alias_parser_handles_any_input(input: String) {
        let result = parse_aliases(&input);
        // Should never panic, may return empty
        prop_assert!(result.is_ok());
    }
}
```

## Security Considerations

1. **Command Privacy**: Never send executed commands to external services
2. **Config Backup**: Always create backups before modifications
3. **Confirmation Required**: All automated changes require user consent
4. **KB Integrity**: Verify KB checksum before use
5. **Sandboxed Installs**: Installation commands run with user privileges only

## Performance Requirements

| Operation | Target | Notes |
|-----------|--------|-------|
| Shell detection | < 50ms | Cache results |
| Alias parsing | < 100ms | Cache parsed aliases |
| Tip matching | < 10ms | Pre-compiled patterns |
| KB load | < 200ms | Binary format (msgpack) |
| KB update | < 5s | Background, non-blocking |

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Slow shell detection | Cache detection results; lazy load |
| Large KB size | Use msgpack; incremental updates |
| Config corruption | Atomic writes; automatic backups |
| Plugin install failure | Rollback plan; verification steps |
| Overwhelming users with tips | Rate limiting; frequency settings |

## Dependencies

**New Dependencies**:
```toml
# Cargo.toml additions
[dependencies]
shellexpand = "3.1"           # Path expansion
rmp-serde = "1.1"             # MessagePack serialization
rusqlite = { version = "0.31", features = ["bundled"] }
sha2 = "0.10"                 # Checksum verification

[dev-dependencies]
proptest = "1.4"              # Property-based testing
tempfile = "3.10"             # Temp files for tests
```

## Definition of Done

- [ ] Shell detection works for Zsh, Bash, Fish
- [ ] Alias parsing extracts aliases from config files
- [ ] Plugin detection identifies Oh My Zsh, Prezto, etc.
- [ ] Tips engine suggests aliases for matching commands
- [ ] KB build pipeline processes cheatsheets
- [ ] CLI commands implemented (`caro tips`, `caro aliases`)
- [ ] Installation automation with backup/rollback
- [ ] Unit tests > 80% coverage for tips module
- [ ] Integration tests for end-to-end flows
- [ ] Documentation updated
- [ ] Performance targets met
