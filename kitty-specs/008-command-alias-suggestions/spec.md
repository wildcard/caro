# Feature Specification: Intelligent Command Alias Suggestions

**Feature ID**: 008-command-alias-suggestions
**Status**: Draft
**Created**: 2025-12-31
**Milestone**: v2.0.0 (Advanced Features)

## Overview

Caro provides intelligent "Did You Know" tips and recommendations based on shell command execution. By inspecting the user's shell configuration (.zshrc, .bashrc), installed plugins (Oh My Zsh, Prezto, Fish), and local aliases, Caro can suggest shorter alternatives, recommend productivity plugins, and even offer to install/configure them automatically.

## Problem Statement

Users often:
1. Type verbose commands (e.g., `git status`) when shorter aliases exist (`gst`)
2. Don't know about shell plugins that could boost productivity (Oh My Zsh, git plugin)
3. Have aliases installed but forget to use them
4. Miss community best practices for shell productivity

## Goals

1. **Alias Awareness**: Detect and suggest installed aliases for common commands
2. **Plugin Recommendations**: Recommend shell plugins based on user's workflow
3. **Actionable Setup**: Offer to install and configure recommended tools
4. **Knowledge Base**: Curated tips from community cheatsheets
5. **Community Sharing**: Allow users to contribute their aliases and tips

## Non-Goals

- Replacing the user's shell or terminal emulator
- Modifying shell config without explicit permission
- Creating complex automation beyond simple setup commands
- Real-time command interception (only post-execution suggestions)

## User Stories

### US-1: Alias Suggestion
As a developer running `git status`, I want Caro to tell me "Did you know? You have `gst` aliased to `git status`. Use it to save keystrokes!" so I can be more productive.

### US-2: Plugin Recommendation
As a Zsh user without Oh My Zsh, I want Caro to recommend "Did you know? Oh My Zsh provides 200+ useful git aliases. Would you like me to install it?" so I can discover productivity tools.

### US-3: Plugin Configuration
As a user who said "yes" to installing Oh My Zsh, I want Caro to handle the installation, configure the git plugin, and reload my shell automatically.

### US-4: Community Tips
As a user, I want to receive curated tips from community-contributed cheatsheets so I can learn shell best practices.

### US-5: Share My Aliases
As a power user, I want to share my custom aliases with the Caro community so others can benefit from my shortcuts.

## Technical Design

### 1. Shell Configuration Detection

```
src/tips/
├── mod.rs              # Tips module entry point
├── shell_detector.rs   # Detect shell type and config files
├── alias_parser.rs     # Parse aliases from shell config
├── plugin_detector.rs  # Detect installed plugins
└── suggester.rs        # Generate suggestions based on context
```

**Shell Detection Logic**:
```rust
pub enum ShellType {
    Zsh,
    Bash,
    Fish,
    Sh,
    Unknown(String),
}

pub struct ShellConfig {
    pub shell_type: ShellType,
    pub config_paths: Vec<PathBuf>,  // .zshrc, .bashrc, etc.
    pub aliases: HashMap<String, String>,
    pub plugins: Vec<Plugin>,
    pub plugin_manager: Option<PluginManager>,
}

pub enum PluginManager {
    OhMyZsh { plugins: Vec<String> },
    Prezto { modules: Vec<String> },
    Zinit { plugins: Vec<String> },
    Fisher { plugins: Vec<String> },  // Fish
    None,
}
```

### 2. Alias Matching Engine

When a command is executed, match against known aliases:

```rust
pub struct AliasSuggestion {
    pub original_command: String,
    pub suggested_alias: String,
    pub source: AliasSource,
    pub savings: CommandSavings,
}

pub struct CommandSavings {
    pub characters_saved: usize,
    pub estimated_time_saved_ms: u64,
}

pub enum AliasSource {
    LocalConfig(PathBuf),      // User's .zshrc
    PluginProvided(String),    // "git" plugin from Oh My Zsh
    CommunityKnowledgeBase,    // From KB
}
```

### 3. Knowledge Base Architecture

**Data Pipeline** (GitHub Actions):

```yaml
# .github/workflows/kb-update.yml
name: Update Knowledge Base
on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  push:
    paths:
      - 'kb/cheatsheets/**'

jobs:
  process-cheatsheets:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Process Cheatsheets
        run: |
          cargo run --bin kb-processor -- \
            --input kb/cheatsheets/ \
            --output kb/processed/

      - name: Validate KB
        run: cargo run --bin kb-validator

      - name: Publish KB Artifact
        uses: actions/upload-artifact@v4
        with:
          name: caro-knowledge-base
          path: kb/processed/
```

**Knowledge Base Schema**:

```rust
pub struct KnowledgeBase {
    pub version: String,
    pub updated_at: DateTime<Utc>,
    pub tips: Vec<Tip>,
    pub aliases: Vec<CommunityAlias>,
    pub plugins: Vec<PluginInfo>,
}

pub struct Tip {
    pub id: String,
    pub category: TipCategory,
    pub shell: Option<ShellType>,
    pub command_pattern: String,  // Regex to match
    pub message: String,
    pub action: Option<TipAction>,
    pub source: TipSource,
}

pub enum TipCategory {
    AliasShortcut,
    PluginRecommendation,
    BestPractice,
    SafetyTip,
    PerformanceTip,
}

pub enum TipAction {
    SuggestAlias { alias: String, command: String },
    InstallPlugin { manager: String, plugin: String, commands: Vec<String> },
    RunCommand { command: String, description: String },
    OpenUrl { url: String },
}

pub struct TipSource {
    pub cheatsheet: Option<String>,  // Original cheatsheet file
    pub contributor: Option<String>,  // GitHub username
    pub url: Option<String>,          // Source URL
}
```

### 4. Community Contribution System

**Cheatsheet Upload Flow**:

```
User uploads cheatsheet -> Moderation queue ->
Automated processing -> Manual review ->
Integration into KB -> Weekly release
```

**Cheatsheet Format** (kb/cheatsheets/):

```yaml
# kb/cheatsheets/git-aliases.yaml
name: Git Productivity Aliases
author: wildcard
shell: zsh
plugin: git
description: Common git aliases from Oh My Zsh git plugin

aliases:
  - alias: gst
    command: git status
    description: Short form of git status

  - alias: gco
    command: git checkout
    description: Short form of git checkout

  - alias: gcm
    command: git checkout $(git_main_branch)
    description: Checkout main branch

tips:
  - pattern: "^git status$"
    message: "Did you know? `gst` is an alias for `git status`"
    category: alias_shortcut

  - pattern: "^git checkout "
    message: "Did you know? `gco` is an alias for `git checkout`"
    category: alias_shortcut
```

**Community Portal** (website feature):

- `/community/contribute` - Submit cheatsheets
- `/community/aliases` - Browse community aliases
- `/community/tips` - Browse tips and tricks
- Moderation dashboard for maintainers
- GitHub OAuth for contributor attribution

### 5. CLI Integration

**New Commands**:

```bash
# Show tips for a command
caro tips "git status"

# Show all available aliases (local + community)
caro aliases list

# Search community tips
caro tips search "docker"

# Enable/disable tips
caro config set tips.enabled true
caro config set tips.frequency sometimes  # always, sometimes, rarely, never

# Contribute an alias
caro community contribute-alias "gp" "git push"

# Update knowledge base
caro kb update
```

**Post-Execution Hook**:

After Caro generates/runs a command, optionally show tips:

```
$ caro "show git status"
Generated: git status

> Did you know?
> You have `gst` aliased to `git status` in your .zshrc
> Use it to save 7 keystrokes!
>
> [Press Enter to dismiss, or 't' for more tips]
```

### 6. Installation Automation

**Plugin Installation Flows**:

```rust
pub struct InstallationFlow {
    pub name: String,
    pub description: String,
    pub steps: Vec<InstallStep>,
    pub post_install: Vec<String>,
}

pub enum InstallStep {
    ShellCommand { command: String, description: String },
    AddToConfig { file: PathBuf, content: String, position: InsertPosition },
    SourceFile { file: PathBuf },
    UserConfirmation { message: String },
}

// Example: Oh My Zsh installation
let ohmyzsh_flow = InstallationFlow {
    name: "Oh My Zsh".to_string(),
    steps: vec![
        InstallStep::UserConfirmation {
            message: "This will install Oh My Zsh. Continue?".to_string()
        },
        InstallStep::ShellCommand {
            command: "sh -c \"$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\"".to_string(),
            description: "Download and run Oh My Zsh installer".to_string(),
        },
        InstallStep::AddToConfig {
            file: PathBuf::from("~/.zshrc"),
            content: "plugins=(git docker kubectl)".to_string(),
            position: InsertPosition::ReplacePattern("^plugins=".to_string()),
        },
    ],
    post_install: vec!["source ~/.zshrc".to_string()],
};
```

## Configuration

```toml
# ~/.config/caro/config.toml

[tips]
enabled = true
frequency = "sometimes"  # always, sometimes, rarely, never
show_keyboard_hints = true
max_tips_per_session = 5

[tips.categories]
alias_shortcuts = true
plugin_recommendations = true
best_practices = true
safety_tips = true

[tips.community]
enable_community_tips = true
auto_update_kb = true
update_frequency = "weekly"

[tips.privacy]
send_anonymous_stats = false  # Never send command data
local_only = false            # If true, no community features
```

## Acceptance Criteria

### AC-1: Shell Detection
- [ ] Correctly detect Zsh, Bash, Fish shell types
- [ ] Parse aliases from .zshrc, .bashrc, config.fish
- [ ] Detect Oh My Zsh, Prezto, Zinit, Fisher plugin managers
- [ ] List installed plugins and their provided aliases

### AC-2: Alias Suggestions
- [ ] Match executed commands against local aliases
- [ ] Show "Did you know?" message with alias suggestion
- [ ] Display character/keystroke savings
- [ ] Respect frequency settings (don't spam)

### AC-3: Plugin Recommendations
- [ ] Detect when user could benefit from a plugin
- [ ] Show plugin recommendation with benefits
- [ ] Offer to install plugin with user confirmation
- [ ] Execute installation flow successfully

### AC-4: Knowledge Base
- [ ] Process YAML cheatsheet files via GitHub Actions
- [ ] Generate versioned KB artifact
- [ ] CLI can fetch and cache KB locally
- [ ] Tips from KB appear in suggestions

### AC-5: Community Features
- [ ] Users can submit cheatsheets via website
- [ ] Moderation queue for review
- [ ] Contributors credited in KB
- [ ] Weekly KB updates published

### AC-6: Privacy & Safety
- [ ] No command data sent to servers
- [ ] Installation flows require explicit confirmation
- [ ] Config backup before modifications
- [ ] Dry-run mode for installation flows

## Security Considerations

1. **No Command Telemetry**: Shell commands never leave the device
2. **Installation Safety**: All automated installations require explicit user confirmation
3. **Config Backups**: Always backup .zshrc etc. before modifications
4. **Sandboxed Execution**: Installation commands run with user privileges only
5. **KB Integrity**: Knowledge base signed and verified before use
6. **Community Moderation**: All contributed content reviewed before publication

## Dependencies

- `shellexpand` - Expand ~ and environment variables in paths
- `regex` - Pattern matching for command detection
- `serde_yaml` - Parse cheatsheet YAML files
- `reqwest` - Fetch KB updates
- `sha2` - Verify KB integrity

## Milestones

### Phase 1: Shell Intelligence (Core)
- Shell type detection
- Alias parsing from config files
- Plugin manager detection
- Basic "Did you know?" suggestions

### Phase 2: Knowledge Base
- Cheatsheet YAML format
- GitHub Actions processor
- KB fetch and caching
- Community tips integration

### Phase 3: Installation Automation
- Plugin installation flows
- Oh My Zsh installer
- Config modification with backup
- Shell reload automation

### Phase 4: Community Portal
- Website contribution page
- Moderation dashboard
- Contributor attribution
- Community alias browser

## Related Issues

- Relates to: Local Context Indexing (#152)
- Relates to: Self-Healing Features (#155)
- Milestone: v2.0.0 (Advanced Features)

## Open Questions

1. **Q**: Should tips be shown inline or in a separate "tips" panel?
   **A**: Start with inline, add panel option in v2.

2. **Q**: How to handle conflicting aliases (user vs community)?
   **A**: User aliases always take precedence.

3. **Q**: KB update frequency?
   **A**: Weekly by default, configurable.

4. **Q**: Rate limiting for tips?
   **A**: Max 5 tips per session by default, configurable.
