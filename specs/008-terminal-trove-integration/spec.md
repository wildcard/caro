# Feature Specification: Terminal Trove Knowledge Integration

**Feature ID:** 008
**Status:** Draft
**Created:** 2026-01-04
**Last Updated:** 2026-01-04
**Owners:** Development Team

---

## Executive Summary

This specification defines an integration with [Terminal Trove](https://terminaltrove.com/) to enrich Caro's command generation capabilities with comprehensive knowledge about terminal tools, their use cases, and modern alternatives.

### Problem Statement

Caro's command generation currently relies on:
1. **Local tool availability** - Only knows about tools installed on the user's system
2. **Man page analysis** - Flag validation from parsed man pages
3. **Static prompt templates** - Pre-defined platform-specific examples

This creates limitations:
- Cannot suggest modern alternatives (e.g., `fd` instead of `find`) unless already installed
- Limited knowledge of tool categories and use cases
- No awareness of popular tools the user might want to install
- Missing context about which tools are best for specific tasks

### Solution Overview

Integrate Terminal Trove's curated CLI/TUI tool database to:
- **Enrich** prompt context with tool knowledge and descriptions
- **Suggest** modern alternatives and complementary tools
- **Categorize** user requests to select optimal tools
- **Recommend** tools for installation when appropriate

---

## Goals & Success Metrics

### Primary Goals

1. **Enhanced Tool Recommendations**: Suggest the best tool for each task, not just what's installed
2. **Modern Alternatives**: Recommend modern Rust/Go alternatives to legacy POSIX tools
3. **Tool Discovery**: Help users discover relevant tools they don't have installed
4. **Improved Accuracy**: Better command generation through richer tool knowledge

### Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Modern tool suggestions | 0% | 30% | Commands using modern alternatives |
| Tool discovery prompts | 0% | 10% | Sessions with tool recommendations |
| User satisfaction | N/A | 4.5/5 | Post-command feedback |
| Command accuracy | 85% | 95% | First-attempt success rate |

### Non-Goals

- Real-time scraping of Terminal Trove website
- Automatic tool installation (only recommendations)
- Replacing local man page analysis (complementary, not replacement)
- Becoming a full terminal tool marketplace

---

## User Stories

### US-001: Modern Alternative Suggestions

**As a** user running `caro "find all rust files"`
**I want** Caro to suggest using `fd` if I have it installed
**So that** I get the most efficient command for my task

**Acceptance Criteria:**
- System detects `fd` is installed
- Prompt includes knowledge that `fd` is a modern `find` alternative
- Generated command uses `fd -e rs` instead of `find . -name "*.rs"`
- Explanation mentions the modern alternative being used

**Example:**
```bash
$ caro "find all rust files"

Command: fd -e rs
Explanation: Using fd (modern find alternative) for faster file search
Alternatives: find . -name "*.rs" (POSIX compatible)
```

---

### US-002: Tool Installation Recommendations

**As a** user asking for a task that benefits from a modern tool
**I want** Caro to recommend installing the tool if it's not present
**So that** I can improve my workflow

**Acceptance Criteria:**
- System detects optimal tool is not installed
- Provides working command with available tools
- Offers recommendation for better tool
- Shows installation command for recommended tool

**Example:**
```bash
$ caro "search code for API endpoints"

Command: grep -r "api/" --include="*.rs" .
Explanation: Using grep for code search

Recommendation: For faster code search, consider installing ripgrep:
  brew install ripgrep  # or cargo install ripgrep
  Then: rg "api/"
```

---

### US-003: Category-Aware Tool Selection

**As a** user asking to "view a JSON file nicely"
**I want** Caro to know about JSON viewers and select the best one
**So that** I get optimal results based on my installed tools

**Acceptance Criteria:**
- System understands "view JSON nicely" requires a JSON viewer
- Checks for: `jq`, `fx`, `jless`, `bat`, or falls back to `cat`
- Selects best available tool based on capability
- Generates appropriate command

**Example:**
```bash
$ caro "view the config.json file nicely"

# If jless is installed:
Command: jless config.json

# If only bat is installed:
Command: bat --language json config.json

# Fallback:
Command: cat config.json | python3 -m json.tool
```

---

## Technical Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Request                             │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Orchestrator Agent                           │
│                                                                  │
│  ┌───────────────────┐  ┌────────────────────┐                 │
│  │  Local Tool       │  │  Terminal Trove    │                 │
│  │  Analyzer         │  │  Knowledge Base    │                 │
│  │  (Man Pages)      │  │  (Curated Data)    │                 │
│  └────────┬──────────┘  └────────┬───────────┘                 │
│           │                      │                              │
│           └──────────┬───────────┘                              │
│                      │                                          │
│                      ▼                                          │
│           ┌──────────────────────┐                              │
│           │  Tool Context        │                              │
│           │  Merger              │                              │
│           │                      │                              │
│           │  - Available tools   │                              │
│           │  - Tool categories   │                              │
│           │  - Modern alts       │                              │
│           │  - Recommendations   │                              │
│           └──────────────────────┘                              │
│                      │                                          │
└──────────────────────┼──────────────────────────────────────────┘
                       │
                       ▼
              ┌────────────────┐
              │   Generator    │
              │   Agent        │
              │                │
              │ Enhanced with: │
              │ - Tool knowledge│
              │ - Categories   │
              │ - Alternatives │
              └────────────────┘
```

### Data Sources

#### 1. Terminal Trove Static Dataset

A curated, versioned JSON dataset extracted from Terminal Trove containing:

```json
{
  "version": "1.0.0",
  "last_updated": "2026-01-04",
  "tools": {
    "fd": {
      "name": "fd",
      "description": "A simple, fast and user-friendly alternative to find",
      "url": "https://github.com/sharkdp/fd",
      "categories": ["file-search", "filesystem"],
      "languages": ["rust"],
      "replaces": ["find"],
      "install": {
        "homebrew": "fd",
        "cargo": "fd-find",
        "apt": "fd-find"
      },
      "common_flags": ["-e", "-H", "-I", "-t"],
      "examples": {
        "find_by_extension": "fd -e rs",
        "find_hidden": "fd -H pattern",
        "find_type": "fd -t f pattern"
      }
    },
    "ripgrep": {
      "name": "ripgrep",
      "description": "Recursively search directories for a regex pattern",
      "url": "https://github.com/BurntSushi/ripgrep",
      "categories": ["search", "grep"],
      "languages": ["rust"],
      "replaces": ["grep", "ack", "ag"],
      "install": {
        "homebrew": "ripgrep",
        "cargo": "ripgrep",
        "apt": "ripgrep"
      }
    }
  },
  "categories": {
    "file-search": {
      "description": "Tools for finding files",
      "tools": ["fd", "find", "fzf", "locate"]
    },
    "search": {
      "description": "Tools for searching content",
      "tools": ["ripgrep", "grep", "ack", "ag", "ugrep"]
    },
    "json": {
      "description": "Tools for working with JSON",
      "tools": ["jq", "fx", "jless", "gron"]
    }
  },
  "alternatives": {
    "find": ["fd", "bfs"],
    "grep": ["ripgrep", "ag", "ack", "ugrep"],
    "cat": ["bat", "ccat"],
    "ls": ["eza", "lsd", "exa"],
    "du": ["dust", "dua", "ncdu"],
    "top": ["btop", "htop", "bottom"],
    "diff": ["delta", "difftastic"]
  }
}
```

#### 2. Update Strategy

Options for keeping the dataset current:

**Option A: Bundled Static Dataset (Recommended for v1)**
- Ship curated dataset with Caro binary
- Update with each Caro release
- No network requests required
- ~100KB compressed

**Option B: Periodic Sync (Future)**
- Optional background sync from hosted endpoint
- Cache locally with TTL
- Fallback to bundled version

**Option C: Terminal Trove API (If Available)**
- Direct API integration if Terminal Trove offers one
- Real-time tool information
- Requires network connectivity

### Component Design

#### 1. ToolKnowledgeBase

**Location:** `src/knowledge/terminal_trove.rs`

```rust
pub struct ToolKnowledgeBase {
    tools: HashMap<String, ToolInfo>,
    categories: HashMap<String, CategoryInfo>,
    alternatives: HashMap<String, Vec<String>>,
    version: String,
}

impl ToolKnowledgeBase {
    /// Load bundled knowledge base
    pub fn load_bundled() -> Result<Self> {
        let data = include_str!("../../data/terminal-trove.json");
        serde_json::from_str(data)
    }

    /// Find modern alternatives for a tool
    pub fn get_alternatives(&self, tool: &str) -> Vec<&ToolInfo> {
        self.alternatives
            .get(tool)
            .map(|alts| {
                alts.iter()
                    .filter_map(|name| self.tools.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get tools for a category
    pub fn get_category_tools(&self, category: &str) -> Vec<&ToolInfo> {
        self.categories
            .get(category)
            .map(|cat| {
                cat.tools.iter()
                    .filter_map(|name| self.tools.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Categorize a user request
    pub fn categorize_request(&self, request: &str) -> Vec<String> {
        let keywords = self.extract_keywords(request);
        self.categories
            .iter()
            .filter(|(_, cat)| cat.matches_keywords(&keywords))
            .map(|(name, _)| name.clone())
            .collect()
    }
}
```

#### 2. ToolContextEnricher

**Location:** `src/agents/tool_enricher.rs`

```rust
pub struct ToolContextEnricher {
    knowledge_base: Arc<ToolKnowledgeBase>,
    local_analyzer: Arc<ManPageAnalyzer>,
}

impl ToolContextEnricher {
    /// Enrich generation context with tool knowledge
    pub fn enrich_context(
        &self,
        request: &str,
        context: &mut GenerationContext,
    ) -> EnrichmentResult {
        // 1. Categorize the request
        let categories = self.knowledge_base.categorize_request(request);

        // 2. Find relevant tools for each category
        let relevant_tools: Vec<_> = categories
            .iter()
            .flat_map(|cat| self.knowledge_base.get_category_tools(cat))
            .collect();

        // 3. Check which are available locally
        let available: Vec<_> = relevant_tools
            .iter()
            .filter(|tool| self.local_analyzer.is_tool_available(&tool.name))
            .collect();

        // 4. Identify modern alternatives
        let legacy_tools = self.detect_legacy_tools(request);
        let alternatives: Vec<_> = legacy_tools
            .iter()
            .flat_map(|tool| self.knowledge_base.get_alternatives(tool))
            .filter(|alt| self.local_analyzer.is_tool_available(&alt.name))
            .collect();

        // 5. Build recommendations for unavailable tools
        let recommendations = self.build_recommendations(&relevant_tools, &available);

        EnrichmentResult {
            categories,
            available_tools: available,
            modern_alternatives: alternatives,
            recommendations,
        }
    }
}
```

#### 3. Enhanced Prompt Template

**Location:** `prompts/enriched-base.toml`

```toml
[meta]
name = "Enriched Base Template"
version = "1.0.0"
requires_knowledge_base = true

[prompt]
system = """
You are a command-line expert for {{unix_flavor}}/{{os}} systems.

AVAILABLE TOOLS:
{{#each available_tools}}
- {{this.name}}: {{this.description}}
{{/each}}

MODERN ALTERNATIVES (prefer these when available):
{{#each modern_alternatives}}
- {{this.name}} (replaces: {{this.replaces}}): {{this.description}}
{{/each}}

TOOL CATEGORIES FOR THIS REQUEST:
{{#each categories}}
- {{this}}
{{/each}}

REQUIREMENTS:
1. Use modern tools when available (e.g., fd over find, ripgrep over grep)
2. Generate commands compatible with {{unix_flavor}} utilities
3. Consider tool-specific flags and syntax

Response Format (JSON ONLY):
{
  "cmd": "your_command_here",
  "tool_used": "tool_name",
  "is_modern_alt": true/false
}

Request: {{user_input}}
"""
```

---

## Data Collection Strategy

### Initial Dataset Creation

1. **Manual Curation from Terminal Trove**
   - Browse terminaltrove.com categories
   - Extract ~200 most popular/useful tools
   - Categorize and add metadata

2. **Focus Categories**
   - File operations (find, copy, move)
   - Text search (grep, ripgrep)
   - File viewing (cat, bat, less)
   - System monitoring (top, htop, btop)
   - Git operations (git, lazygit, gh)
   - JSON/YAML processing (jq, yq)
   - Network tools (curl, httpie, wget)
   - Development tools (make, cargo, npm)

3. **Data Structure per Tool**
   ```json
   {
     "name": "string",
     "description": "string",
     "url": "string",
     "categories": ["string"],
     "languages": ["string"],
     "replaces": ["string"],
     "platforms": ["macos", "linux", "windows"],
     "install": {
       "homebrew": "string",
       "cargo": "string",
       "apt": "string",
       "npm": "string"
     },
     "common_flags": ["string"],
     "examples": {
       "use_case": "command"
     }
   }
   ```

### Maintenance Plan

1. **Version Control**: Dataset versioned in repository
2. **Community Contributions**: Accept PRs to add/update tools
3. **Periodic Review**: Quarterly review against Terminal Trove
4. **Automated Validation**: CI checks for schema compliance

---

## Implementation Plan

### Phase 1: Foundation (Week 1-2)

**Deliverables:**
1. Initial Terminal Trove dataset (~100 tools)
2. ToolKnowledgeBase struct and loader
3. Basic categorization logic

**Tasks:**
- [ ] Curate initial tool dataset from Terminal Trove
- [ ] Define JSON schema for tool data
- [ ] Implement ToolKnowledgeBase loader
- [ ] Add unit tests for knowledge base

### Phase 2: Context Enrichment (Week 3-4)

**Deliverables:**
1. ToolContextEnricher agent
2. Enhanced prompt templates
3. Integration with Orchestrator

**Tasks:**
- [ ] Implement ToolContextEnricher
- [ ] Create enriched prompt templates
- [ ] Integrate with generation flow
- [ ] Add integration tests

### Phase 3: Recommendations (Week 5-6)

**Deliverables:**
1. Tool recommendation system
2. Installation command generation
3. User-facing recommendation UI

**Tasks:**
- [ ] Implement recommendation logic
- [ ] Generate installation commands per platform
- [ ] Add CLI output for recommendations
- [ ] User acceptance testing

---

## Configuration

### User Configuration Options

```toml
# ~/.config/caro/config.toml

[knowledge]
# Enable Terminal Trove knowledge integration
enabled = true

# Prefer modern alternatives when available
prefer_modern_tools = true

# Show installation recommendations
show_recommendations = true

# Categories to prioritize (empty = all)
preferred_categories = ["rust-tools", "modern-unix"]

# Tools to always prefer
preferred_tools = ["fd", "ripgrep", "bat", "eza"]

# Tools to never recommend
blocked_tools = []
```

---

## Success Criteria

### Launch Criteria

- [ ] Dataset includes 100+ curated tools
- [ ] Categorization accuracy > 80%
- [ ] Modern alternatives suggested when available
- [ ] No performance regression (< 50ms added latency)
- [ ] All tests passing

### Post-Launch Metrics (30 days)

- Modern tool usage rate: > 25%
- User-installed recommended tools: > 5%
- Command accuracy improvement: > 5%
- User satisfaction: 4.5/5

---

## Security Considerations

1. **Data Integrity**: Bundled dataset signed/verified
2. **No Code Execution**: Dataset is data-only, no executable code
3. **Optional Feature**: Can be disabled if privacy concerns
4. **No Telemetry**: No data sent to Terminal Trove or anywhere

---

## Open Questions

1. **Q:** Should we contact Terminal Trove for official partnership/API?
   **A:** Explore after v1 launch with static dataset

2. **Q:** How to handle tool version differences?
   **A:** Focus on common flags, note version-specific features

3. **Q:** Should recommendations be interactive (install now)?
   **A:** Phase 2 consideration, start with informational only

4. **Q:** How to handle conflicting tool names across platforms?
   **A:** Platform-specific tool entries where needed

---

## References

- [Terminal Trove](https://terminaltrove.com/)
- [Terminal Trove GitHub](https://github.com/terminaltrove)
- [Caro Intelligent Prompt Generation Spec](../006-intelligent-prompt-generation/spec.md)
- [Modern Unix Tools List](https://github.com/ibraheemdev/modern-unix)

---

**Document Status:** Draft - Ready for Review
**Next Steps:** Curate initial dataset, begin Phase 1 implementation
