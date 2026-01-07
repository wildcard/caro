# 🛠️ Create Community-Curated Tool Library with Safety Ratings

**Labels**: `good-first-issue`, `first-time-contributor`, `community`, `curation`, `safety`, `database`
**Difficulty**: Medium ⭐⭐⭐
**Skills**: Data modeling, YAML/JSON, community curation, documentation
**Perfect for**: Tool curators, open-source enthusiasts, knowledge organizers, community builders

## The Vision

The terminal has **thousands of tools** (find, grep, awk, fd, ripgrep, bat, etc.). cmdai should maintain a **community-curated library** that:

1. **Catalogs terminal tools** with descriptions and examples
2. **Rates tools by safety** (read-only vs destructive)
3. **Recommends modern alternatives** (ripgrep over grep, bat over cat)
4. **Provides usage examples** for each tool
5. **Tracks platform availability** (macOS, Linux, Windows)

Think of it as **Awesome Lists + Package Manager + Safety Database** for terminal tools!

## What You'll Build

A structured database of terminal tools that cmdai uses to:
- Choose the best tool for each task
- Warn about dangerous tools
- Suggest better alternatives
- Provide usage examples

### Tool Library Structure

```yaml
tools:
  - name: find
    category: file_search
    description: Search for files and directories
    safety_rating: safe
    risk_factors:
      - "-delete flag can be destructive"
      - "avoid using with -exec and rm"
    platforms:
      - linux
      - macos
      - windows_wsl
    installation:
      builtin: true
    examples:
      - description: Find all PDF files
        command: find . -name "*.pdf"
        safety: safe
      - description: Find and delete
        command: find . -name "*.tmp" -delete
        safety: moderate
    alternatives:
      - name: fd
        reason: "Faster, simpler syntax, better defaults"
        example: fd '\.pdf$'

  - name: rm
    category: file_deletion
    description: Remove files or directories
    safety_rating: dangerous
    risk_factors:
      - "permanent deletion (no trash/recycle bin)"
      - "-rf combination is especially dangerous"
      - "can delete system files if run as root"
    platforms:
      - linux
      - macos
      - windows_wsl
    installation:
      builtin: true
    safer_alternatives:
      - name: trash-cli
        reason: "Moves to trash instead of permanent deletion"
        installation: "apt install trash-cli"
      - name: rm -i
        reason: "Interactive mode asks before each deletion"
        builtin: true
    examples:
      - description: Safe single file deletion
        command: rm file.txt
        safety: moderate
      - description: DANGEROUS recursive deletion
        command: rm -rf directory/
        safety: critical

  - name: ripgrep
    category: text_search
    description: "Blazingly fast grep alternative with smart defaults"
    safety_rating: safe
    platforms:
      - linux
      - macos
      - windows
    installation:
      cargo: ripgrep
      homebrew: ripgrep
      apt: ripgrep
    examples:
      - description: Search for pattern
        command: rg "TODO"
        safety: safe
      - description: Search with file type
        command: rg "TODO" -t rust
        safety: safe
    replaces:
      - grep
      - ack
      - ag
```

## Implementation Guide

### Step 1: Design the Data Schema

Create `data/tools/schema.yaml`:

```yaml
# Schema definition for tool library
schema_version: "1.0"

tool:
  required:
    - name
    - category
    - description
    - safety_rating
    - platforms
  optional:
    - risk_factors
    - alternatives
    - examples
    - installation
    - replaces

categories:
  - file_search
  - text_search
  - file_deletion
  - file_modification
  - system_info
  - network
  - process_management
  - archive
  - permissions

safety_ratings:
  - safe          # Read-only, no side effects
  - moderate      # Writes files, but limited scope
  - dangerous     # Can delete/modify important data
  - critical      # System-level changes, requires root
```

### Step 2: Create Tool Database

Create `data/tools/catalog.yaml`:

```yaml
version: "1.0"
last_updated: "2025-01-15"
tools:
  # File search tools
  - name: find
    category: file_search
    description: POSIX-compliant file search utility
    safety_rating: safe
    risk_factors:
      - "-delete flag performs destructive operations"
      - "-exec can run arbitrary commands"
    platforms: [linux, macos, windows_wsl]
    installation:
      builtin: true
    examples:
      - description: Find all JavaScript files
        command: find . -name "*.js"
        safety: safe
      - description: Find files modified in last 7 days
        command: find . -type f -mtime -7
        safety: safe
    alternatives:
      - name: fd
        reason: "Simpler syntax, faster, better defaults"
        installation:
          cargo: fd-find
          homebrew: fd
        example: fd '\.js$'

  # Text search tools
  - name: grep
    category: text_search
    description: Search text patterns in files
    safety_rating: safe
    platforms: [linux, macos, windows_wsl]
    installation:
      builtin: true
    examples:
      - description: Search for pattern
        command: grep "error" logfile.txt
        safety: safe
    alternatives:
      - name: ripgrep
        reason: "Much faster, better defaults, colored output"
        example: rg "error"

  # File deletion tools
  - name: rm
    category: file_deletion
    description: Remove files or directories
    safety_rating: dangerous
    risk_factors:
      - "Permanent deletion without recovery"
      - "-rf combination bypasses confirmations"
      - "No built-in undo mechanism"
    platforms: [linux, macos, windows_wsl]
    installation:
      builtin: true
    safer_alternatives:
      - name: trash-cli
        reason: "Reversible deletion via trash"
        installation:
          apt: trash-cli
          homebrew: trash
      - name: safe-rm
        reason: "Wrapper that prevents dangerous deletions"

  # Modern alternatives
  - name: bat
    category: file_viewing
    description: "cat clone with syntax highlighting and git integration"
    safety_rating: safe
    platforms: [linux, macos, windows]
    installation:
      cargo: bat
      homebrew: bat
      apt: bat
    examples:
      - description: View file with syntax highlighting
        command: bat script.py
        safety: safe
    replaces: [cat, less]

  - name: exa
    category: file_listing
    description: "Modern ls replacement with better defaults"
    safety_rating: safe
    platforms: [linux, macos, windows]
    installation:
      cargo: exa
      homebrew: exa
    examples:
      - description: List with icons and git status
        command: exa --icons --git
        safety: safe
    replaces: [ls]

  # Add 50+ more tools...
```

### Step 3: Implement Tool Library Loader

Create `src/tools/library.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub category: Category,
    pub description: String,
    pub safety_rating: SafetyRating,
    pub risk_factors: Option<Vec<String>>,
    pub platforms: Vec<Platform>,
    pub examples: Vec<Example>,
    pub alternatives: Option<Vec<Alternative>>,
    pub safer_alternatives: Option<Vec<SafeAlternative>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SafetyRating {
    Safe,
    Moderate,
    Dangerous,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub name: String,
    pub reason: String,
    pub example: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Catalog {
    version: String,
    tools: Vec<Tool>,
}

pub struct ToolLibrary {
    tools: HashMap<String, Tool>,
}

impl ToolLibrary {
    pub fn load() -> Result<Self, LibraryError> {
        let catalog_yaml = include_str!("../../data/tools/catalog.yaml");
        let catalog: Catalog = serde_yaml::from_str(catalog_yaml)?;

        let tools_map = catalog.tools
            .into_iter()
            .map(|t| (t.name.clone(), t))
            .collect();

        Ok(Self { tools: tools_map })
    }

    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    pub fn find_safer_alternative(&self, risky_tool: &str) -> Option<&SafeAlternative> {
        let tool = self.get_tool(risky_tool)?;
        tool.safer_alternatives.as_ref()?.first()
    }

    pub fn get_by_category(&self, category: Category) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|t| {
                t.name.contains(query) ||
                t.description.to_lowercase().contains(&query.to_lowercase())
            })
            .collect()
    }
}
```

### Step 4: Integrate with Command Generation

```rust
impl CommandGenerator {
    fn select_best_tool(&self, intent: &CommandIntent) -> Result<String, GeneratorError> {
        let library = ToolLibrary::load()?;

        // Get tools for this category
        let candidates = library.get_by_category(intent.category);

        // Prefer safer tools
        let best_tool = candidates
            .iter()
            .filter(|t| self.is_available(t))
            .min_by_key(|t| t.safety_rating)
            .ok_or(GeneratorError::NoToolAvailable)?;

        Ok(best_tool.name.clone())
    }

    fn suggest_modern_alternative(&self, command: &str) -> Option<String> {
        let library = ToolLibrary::load().ok()?;

        // Extract tool name from command
        let tool_name = command.split_whitespace().next()?;

        // Check for modern alternatives
        let tool = library.get_tool(tool_name)?;

        if let Some(alternatives) = &tool.alternatives {
            let alt = alternatives.first()?;
            return Some(format!(
                "💡 Consider using {} instead: {} ({})",
                alt.name,
                alt.example.as_ref().unwrap_or(&"".to_string()),
                alt.reason
            ));
        }

        None
    }
}
```

### Step 5: Add CLI Commands

```rust
#[derive(Parser)]
pub struct Cli {
    // ... existing fields

    /// List available tools
    #[command(subcommand)]
    pub tools: Option<ToolsCommand>,
}

#[derive(Subcommand)]
pub enum ToolsCommand {
    /// List all tools in library
    List {
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Search for tools
    Search {
        query: String,
    },
    /// Show tool details
    Info {
        tool_name: String,
    },
}
```

### Step 6: Create Contribution Guide

Create `data/tools/CONTRIBUTING.md`:

```markdown
# Contributing to the Tool Library

## Adding a New Tool

1. Edit `data/tools/catalog.yaml`
2. Add your tool following the schema
3. Include examples and safety information
4. Submit a PR

## Tool Entry Template

```yaml
- name: your-tool-name
  category: appropriate_category
  description: Clear, concise description
  safety_rating: safe | moderate | dangerous | critical
  risk_factors:
    - List any safety concerns
  platforms:
    - linux
    - macos
  examples:
    - description: What it does
      command: the-command --example
      safety: safe
  alternatives:
    - name: better-tool
      reason: Why it's better
```

## Quality Standards

- Accurate safety ratings
- Working examples
- Clear descriptions
- Platform verification
```

## Acceptance Criteria

- [ ] Tool library schema is defined and documented
- [ ] At least 30 common tools are cataloged
- [ ] Each tool has safety rating, examples, and platform info
- [ ] Library loader reads YAML and provides query interface
- [ ] `cmdai tools list` displays available tools
- [ ] `cmdai tools search <query>` finds relevant tools
- [ ] `cmdai tools info <name>` shows detailed tool information
- [ ] Modern alternatives are suggested when using old tools
- [ ] Community contribution guide is clear
- [ ] Tests verify library loading and queries
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Tool Categories to Cover

**File Operations**:
- find, fd, locate
- ls, exa, lsd
- cat, bat, less
- cp, rsync
- rm, trash-cli

**Text Processing**:
- grep, ripgrep, ag
- sed, awk
- sort, uniq

**Archives**:
- tar, gzip, zip
- unzip, unrar

**System**:
- ps, htop, btop
- df, du, ncdu
- kill, pkill

**Network**:
- curl, wget
- ping, traceroute
- netstat, ss

## Why This Matters

1. **Knowledge Sharing**: Community curates best practices
2. **Safety**: Prevent users from choosing dangerous tools
3. **Modernization**: Introduce users to better alternatives
4. **Education**: Learn about available tools
5. **Consistency**: cmdai always chooses the best tool for the job

## Community Curation

This is a **living database**! We need community help to:
- Add new tools as they emerge
- Update safety ratings based on real usage
- Provide better examples
- Document platform-specific quirks
- Suggest improvements

## Resources

- [Awesome CLI Apps](https://github.com/agarrharr/awesome-cli-apps)
- [Modern Unix](https://github.com/ibraheemdev/modern-unix)
- [Command Line Tools You Can't Live Without](https://dev.to/lissy93/cli-tools-you-cant-live-without-57f6)

## Questions?

We'll help you with:
- YAML schema design
- Tool categorization
- Safety rating guidelines
- Data validation

**Ready to build the definitive terminal tool knowledge base? Let's curate together! 🛠️**
