# Suggested Queries Feature - Implementation Plan

## Overview

This feature adds a proactive suggestion system to Caro that solves the cold-start problem by analyzing the user's terminal environment and suggesting relevant queries or actions based on their usage patterns.

## Core Concept

Like ChatGPT and Claude's suggested prompts, Caro will:
1. Display suggested queries when invoked without a prompt
2. Personalize suggestions based on user's terminal history and environment
3. Adapt to new users with generic, educational suggestions
4. Run analysis in background workers using Tokio for performance

## User Experience

```bash
# When user runs caro without arguments:
$ caro

🐱 Caro - What can I help you with?

Based on your environment, here are some suggestions:

  [1] "find files modified in the last hour"     # Based on frequent find usage
  [2] "show git status and recent commits"       # Detected git repo + history
  [3] "list running docker containers"           # docker in PATH
  [4] "search for TODO comments in code"         # Detected dev environment

Or type your own request...
```

## Architecture

### Directory Structure

```
~/.caro/
├── profile.json           # User profile with analysis results
├── history_cache.json     # Cached analysis of shell history
├── tools_cache.json       # Cached PATH analysis
├── preferences.toml       # User preferences for suggestions
└── atuin/                 # Reserved for Atuin integration
    └── sync_state.json
```

### Module Structure

```
src/
├── suggestions/
│   ├── mod.rs             # Module root, public API
│   ├── analyzer.rs        # Parallel analysis coordinator
│   ├── history.rs         # Shell history analysis
│   ├── tools.rs           # PATH and installed tools analysis
│   ├── environment.rs     # Environment variable analysis
│   ├── profile.rs         # User profile management
│   ├── generator.rs       # Query suggestion generator
│   ├── atuin.rs           # Atuin integration (future)
│   └── defaults.rs        # Default suggestions for new users
├── ...
```

## Data Models

### UserProfile

```rust
/// User profile with analyzed data for personalized suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique profile version for cache invalidation
    pub version: String,

    /// When the profile was last updated
    pub last_analyzed: DateTime<Utc>,

    /// User experience level (inferred)
    pub experience_level: ExperienceLevel,

    /// Detected primary workflows
    pub workflows: Vec<Workflow>,

    /// Command frequency patterns
    pub command_patterns: CommandPatterns,

    /// Installed tools of interest
    pub detected_tools: Vec<DetectedTool>,

    /// Environment insights
    pub environment_insights: EnvironmentInsights,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperienceLevel {
    /// New to terminal, limited history
    Beginner,
    /// Some experience, uses common tools
    Intermediate,
    /// Power user with diverse tool usage
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,           // "git", "docker", "python-dev", etc.
    pub confidence: f32,        // 0.0 - 1.0
    pub related_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPatterns {
    /// Most frequently used commands
    pub top_commands: Vec<(String, u32)>,

    /// Common command patterns (e.g., "git add && git commit")
    pub common_sequences: Vec<Vec<String>>,

    /// Time-of-day usage patterns
    pub usage_hours: [u32; 24],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTool {
    pub name: String,
    pub path: PathBuf,
    pub category: ToolCategory,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ToolCategory {
    VersionControl,    // git, hg, svn
    ContainerRuntime,  // docker, podman
    PackageManager,    // npm, cargo, pip
    Editor,            // vim, nvim, emacs
    Language,          // python, node, rustc
    SystemUtil,        // find, grep, awk, sed
    NetworkTool,       // curl, wget, nc
    DatabaseClient,    // psql, mysql, redis-cli
    CloudCli,          // aws, gcloud, kubectl
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInsights {
    /// Detected project type in current directory
    pub project_type: Option<ProjectType>,

    /// Notable environment variables
    pub notable_env_vars: Vec<String>,

    /// Custom PATH entries
    pub custom_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Git,
    Docker,
    Unknown,
}
```

### SuggestedQuery

```rust
/// A suggested query for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedQuery {
    /// The natural language query to suggest
    pub query: String,

    /// Short description of what this does
    pub description: String,

    /// Why this was suggested
    pub reason: SuggestionReason,

    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,

    /// Category for grouping
    pub category: QueryCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionReason {
    /// Based on shell history patterns
    HistoryPattern { command: String },

    /// Based on detected tool
    DetectedTool { tool: String },

    /// Based on current directory context
    DirectoryContext { context: String },

    /// Based on time of day
    TimeBasedHabit,

    /// Generic suggestion for new users
    NewUserOnboarding,

    /// Git-specific (Caro loves Git!)
    GitWorkflow { state: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QueryCategory {
    FileOperations,
    Git,
    Docker,
    Development,
    System,
    Network,
    Learning,
}
```

## Analysis Workers

### Parallel Analysis Architecture

```rust
/// Coordinator for parallel environment analysis
pub struct AnalysisCoordinator {
    /// Runtime for spawning analysis tasks
    runtime: tokio::runtime::Handle,

    /// Path to .caro directory
    data_dir: PathBuf,

    /// Maximum analysis time
    timeout: Duration,
}

impl AnalysisCoordinator {
    /// Run all analyzers in parallel and consolidate results
    pub async fn analyze(&self) -> Result<UserProfile> {
        let (history_result, tools_result, env_result) = tokio::join!(
            self.analyze_history(),
            self.analyze_tools(),
            self.analyze_environment(),
        );

        // Consolidate results
        self.build_profile(history_result?, tools_result?, env_result?)
    }
}
```

### History Analyzer

Analyzes shell history files:
- `~/.bash_history`
- `~/.zsh_history`
- `~/.local/share/fish/fish_history`

Key analysis:
1. Command frequency counting
2. Sequence pattern detection
3. Temporal usage patterns
4. Error pattern detection (commands followed by corrections)

### Tools Analyzer

Analyzes PATH and common tool locations:
1. Parse `$PATH` variable
2. Check for common development tools
3. Detect package managers (cargo, npm, pip, etc.)
4. Identify version control tools
5. Find container runtimes

### Environment Analyzer

Analyzes current environment:
1. Current directory project type (Cargo.toml, package.json, etc.)
2. Git repository state (branch, uncommitted changes)
3. Notable environment variables
4. Custom PATH entries

## New User Detection

Heuristics for detecting new/beginner users:
1. **History size**: < 100 commands in history
2. **Tool diversity**: Only basic commands (ls, cd, cat)
3. **No development tools**: No git, docker, package managers
4. **Standard PATH**: No custom entries in PATH

For new users, Caro provides educational suggestions:
```rust
const BEGINNER_SUGGESTIONS: &[SuggestedQuery] = &[
    SuggestedQuery {
        query: "list files in current directory",
        description: "See what files are here",
        reason: SuggestionReason::NewUserOnboarding,
        category: QueryCategory::Learning,
    },
    SuggestedQuery {
        query: "find files by name",
        description: "Search for files matching a pattern",
        reason: SuggestionReason::NewUserOnboarding,
        category: QueryCategory::Learning,
    },
    // ... more educational suggestions for find, awk, sed, git
];
```

## Git Integration (Caro loves Git!)

Special handling for Git repositories:
1. Detect `.git` directory
2. Analyze git state (clean, dirty, ahead/behind)
3. Suggest relevant git commands based on state

```rust
/// Git-specific suggestions based on repository state
pub fn git_suggestions(repo_state: &GitState) -> Vec<SuggestedQuery> {
    match repo_state {
        GitState::Clean => vec![
            suggest("show recent commits and changes"),
            suggest("create a new branch for feature work"),
        ],
        GitState::UnstagedChanges(files) => vec![
            suggest("stage all changes and commit"),
            suggest("show what changed in modified files"),
        ],
        GitState::StagedChanges => vec![
            suggest("commit staged changes with a message"),
            suggest("show diff of staged changes"),
        ],
        GitState::AheadOfRemote(n) => vec![
            suggest(&format!("push {} commits to remote", n)),
        ],
        // ... more git states
    }
}
```

## Atuin Integration (Future)

Reserved space for Atuin integration:
- Atuin provides synchronized shell history across machines
- Rich metadata (timestamps, exit codes, duration)
- Already structured data, easier to analyze

```rust
/// Placeholder for Atuin integration
pub mod atuin {
    use super::*;

    /// Check if Atuin is available
    pub fn is_available() -> bool {
        std::process::Command::new("atuin")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Future: Import history from Atuin
    pub async fn import_history() -> Result<Vec<HistoryEntry>> {
        // Reserved for future implementation
        Err(anyhow::anyhow!("Atuin integration not yet implemented"))
    }
}
```

## CLI Integration

### New CLI Flag

```rust
/// cmdai CLI with suggestion support
#[derive(Parser)]
struct Cli {
    /// Natural language task description (optional with --suggest)
    prompt: Option<String>,

    /// Show suggested queries based on environment analysis
    #[arg(long, short = 'S')]
    suggest: bool,

    /// Number of suggestions to show
    #[arg(long, default_value = "5")]
    num_suggestions: usize,

    /// Force re-analysis of environment
    #[arg(long)]
    refresh_profile: bool,

    // ... existing args
}
```

### Interactive Mode

When no prompt is provided, enter interactive suggestion mode:
1. Load or generate user profile
2. Generate contextual suggestions
3. Display with numbered selection
4. Allow user to select or type custom

## Implementation Phases

### Phase 1: Core Infrastructure
- [ ] Create `~/.caro/` directory management
- [ ] Define data models (UserProfile, SuggestedQuery, etc.)
- [ ] Implement profile serialization/deserialization
- [ ] Add basic CLI flag (`--suggest`)

### Phase 2: History Analysis
- [ ] Implement history file detection
- [ ] Parse bash_history format
- [ ] Parse zsh_history format
- [ ] Parse fish_history format
- [ ] Command frequency analysis
- [ ] Sequence pattern detection

### Phase 3: Tools Analysis
- [ ] PATH parsing and analysis
- [ ] Tool detection with categories
- [ ] Version detection for key tools
- [ ] Custom PATH entry detection

### Phase 4: Environment Analysis
- [ ] Project type detection
- [ ] Git state detection
- [ ] Environment variable analysis
- [ ] Current directory context

### Phase 5: Suggestion Generation
- [ ] Query generator from profile
- [ ] Git-specific suggestions
- [ ] New user detection and defaults
- [ ] Relevance scoring

### Phase 6: Interactive Mode
- [ ] Interactive suggestion display
- [ ] Numbered selection
- [ ] Integration with existing command flow

### Phase 7: Atuin Integration (Future)
- [ ] Atuin availability detection
- [ ] History import from Atuin
- [ ] Sync state management

## File Changes Summary

### New Files
- `src/suggestions/mod.rs` - Module root
- `src/suggestions/analyzer.rs` - Analysis coordinator
- `src/suggestions/history.rs` - History parsing
- `src/suggestions/tools.rs` - Tool detection
- `src/suggestions/environment.rs` - Environment analysis
- `src/suggestions/profile.rs` - Profile management
- `src/suggestions/generator.rs` - Query generation
- `src/suggestions/defaults.rs` - Default suggestions
- `src/suggestions/atuin.rs` - Atuin integration stub

### Modified Files
- `src/lib.rs` - Add `suggestions` module
- `src/main.rs` - Add `--suggest` flag and interactive mode
- `src/cli/mod.rs` - Suggestion display logic

## Dependencies

### New Dependencies
```toml
[dependencies]
# Already have tokio for async
# May need:
shellexpand = "3.1"  # For expanding ~ and $VAR in paths
regex = "1.10"       # For history parsing patterns
walkdir = "2.4"      # For directory traversal
```

## Performance Considerations

1. **Lazy Analysis**: Only analyze when suggestions are requested
2. **Caching**: Cache analysis results with TTL (1 hour default)
3. **Parallel Workers**: Use Tokio for concurrent analysis
4. **Timeout**: Cap analysis at 2 seconds for responsiveness
5. **Incremental Updates**: Only re-analyze changed data

## Testing Strategy

### Unit Tests
- History parser for each shell format
- Tool detection accuracy
- Profile serialization roundtrip
- Suggestion relevance scoring

### Integration Tests
- Full analysis pipeline
- CLI integration
- Cache invalidation

### Property Tests
- Random history input parsing
- Profile invariants

## Security Considerations

1. **No sensitive data in profile**: Filter passwords, tokens, etc.
2. **Local storage only**: All data stays in `~/.caro/`
3. **Read-only analysis**: Never modify user's shell history
4. **Permission checks**: Respect file permissions when reading

## Success Metrics

1. Startup time with suggestions < 500ms (from cache)
2. Full analysis < 2 seconds
3. Suggestion relevance > 70% (user follows suggestion)
4. New user education improves command discovery
