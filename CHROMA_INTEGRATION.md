# ChromaDB Integration for cmdai

## Overview

This document describes the ChromaDB vector database integration for enhanced command generation using Retrieval Augmented Generation (RAG).

## Status: 🚧 IN DEVELOPMENT

The ChromaDB integration is currently under active development. The core architecture and module structure have been implemented, but compilation fixes are needed before the feature is fully functional.

## Features

### 1. Knowledge Base System (`src/knowledge/`)

The knowledge base system provides RAG capabilities to enhance command generation with:

- **Command Documentation Indexing**
  - Man pages
  - tldr pages (following tealdeer conventions)
  - Help output (`--help`, `-h`)

- **Execution History Tracking**
  - Learn from past successful commands
  - Track user-specific prompting patterns
  - Build command preference profiles

- **User Profile Management**
  - Multiple profiles for different work contexts (e.g., "work", "personal", "devops")
  - Per-profile safety preferences
  - Command aliases and shortcuts
  - Persistent storage in `~/.config/cmdai/profiles/`

- **Mistake Learning**
  - Track failed/dangerous commands
  - Learn from corrections
  - Avoid repeating past errors

- **Project-Specific Context**
  - Index GitHub/GitLab repository documentation
  - README.md and markdown files
  - Wiki pages
  - GitHub Pages (project websites)
  - Automatically detect project directories

### 2. Module Structure

```
src/knowledge/
├── mod.rs              # Module exports and configuration
├── client.rs           # ChromaDB client wrapper
├── collections.rs      # Collection types and metadata schemas
├── indexers.rs         # Documentation indexing system
├── profiles.rs         # User profile management
└── retrieval.rs        # RAG retrieval logic
```

### 3. Collection Schema

#### Command Documentation Collection (`cmdai_command_docs`)
```json
{
  "command_name": "ls",
  "source_type": "man|tldr|help",
  "shell_type": "bash|zsh|fish|...",
  "last_updated": "2025-11-29T...",
  "platform": "linux|macos|windows"
}
```

#### Execution History Collection (`cmdai_execution_history`)
```json
{
  "timestamp": "2025-11-29T...",
  "user_profile": "default|work|personal",
  "success": true,
  "command": "find . -name '*.rs'",
  "prompt": "find all rust files",
  "safety_level": "moderate",
  "shell_type": "bash"
}
```

#### User Preferences Collection (`cmdai_user_preferences`)
```json
{
  "user_profile": "work",
  "preference_type": "command_pattern",
  "command_pattern": "git commit -m",
  "frequency": 42,
  "last_used": "2025-11-29T..."
}
```

#### Mistakes Learned Collection (`cmdai_mistakes_learned`)
```json
{
  "timestamp": "2025-11-29T...",
  "user_profile": "default",
  "error_type": "safety_violation",
  "original_command": "rm -rf /",
  "correction": "rm -rf ./target"
}
```

#### Project Context Collection (`cmdai_project_context`)
```json
{
  "project_path": "https://github.com/user/repo",
  "file_type": "README.md",
  "relevance_score": 1.0,
  "last_indexed": "2025-11-29T..."
}
```

### 4. RAG Pipeline

When generating a command, the system:

1. **Retrieves** relevant context:
   - Similar past commands from execution history
   - Relevant man pages/tldr/help output
   - User preferences for similar tasks
   - Past mistakes to avoid
   - Project-specific documentation (if in a git repo)

2. **Augments** the LLM prompt with:
   ```
   # User Request
   [original prompt]

   # Relevant Documentation
   [top 3 relevant docs]

   # Similar Past Commands
   [successful similar commands]

   # User Preferences
   [user's typical patterns]

   # Previous Mistakes to Avoid
   [corrections and safety violations]

   # Project-Specific Context
   [relevant README/docs if in project]
   ```

3. **Generates** enhanced command with better:
   - Accuracy (learns from docs)
   - Personalization (learns from user)
   - Safety (avoids past mistakes)
   - Context-awareness (uses project docs)

### 5. User Profile System

Users can create multiple profiles for different contexts:

```rust
// Default profile
let profile = UserProfile::default_profile();

// Work profile - strict safety, bash preference
let work = UserProfile::new(
    "work",
    "Corporate work environment",
    ShellType::Bash,
    SafetyLevel::Strict
);

// DevOps profile - moderate safety, custom aliases
let mut devops = UserProfile::new(
    "devops",
    "Infrastructure management",
    ShellType::Zsh,
    SafetyLevel::Moderate
);
devops.add_alias("k", "kubectl");
devops.add_alias("tf", "terraform");
```

### 6. Configuration

Configuration in `~/.config/cmdai/config.toml`:

```toml
[knowledge_base]
enabled = true
chroma_url = "http://localhost:8000"
max_retrieval_docs = 5
min_similarity_score = 0.7
track_history = true
auto_index_man_pages = false
project_context_enabled = true

[knowledge_base.profiles]
directory = "~/.config/cmdai/profiles"
```

### 7. CLI Commands (Planned)

```bash
# Profile management
cmdai profile create work --shell bash --safety strict
cmdai profile list
cmdai profile switch work
cmdai profile delete old-profile

# Knowledge base management
cmdai index man ls find grep  # Index man pages
cmdai index tldr ls find grep # Index tldr pages
cmdai index repo https://github.com/user/repo  # Index git repo
cmdai index project .  # Index current project

# History
cmdai history list
cmdai history clear
cmdai history export history.json
```

## Implementation Progress

### ✅ Completed
- [x] Architecture design and schema definition
- [x] Module structure creation
- [x] ChromaDB client wrapper
- [x] Collection type definitions
- [x] User profile management system
- [x] Document metadata schemas
- [x] Indexer system (man, tldr, help, git repos, GitHub Pages)
- [x] RAG retrieval logic
- [x] Dependencies added to Cargo.toml

### 🚧 In Progress
- [ ] Fix ChromaDB API compatibility issues
- [ ] Complete client.rs implementation
- [ ] Complete retrieval.rs implementation
- [ ] Test compilation and fix type errors

### 📋 TODO
- [ ] Integrate RAG into backend inference pipeline
- [ ] Add execution history tracking to CLI
- [ ] Implement profile management CLI commands
- [ ] Add configuration options to config system
- [ ] Write integration tests
- [ ] Write unit tests for all modules
- [ ] Update CLAUDE.md with new features
- [ ] Create user documentation
- [ ] Add examples and tutorials

## Dependencies

- `chromadb = "2.3"` - ChromaDB Rust client
- `uuid = { version = "1.10", features = ["v4", "serde"] }` - Unique IDs
- `reqwest = "0.11"` - HTTP client for GitHub Pages
- `regex = "1"` - HTML parsing and pattern matching

## ChromaDB Setup

### Local Development

```bash
# Run ChromaDB with Docker
docker run -p 8000:8000 chromadb/chroma:latest

# Or install locally
pip install chromadb
chroma run --path ./chroma_data
```

### Production

For production use, consider:
- Persistent storage configuration
- Authentication setup
- Resource limits and scaling
- Backup strategies

## References

- [ChromaDB Official Docs](https://docs.trychroma.com/)
- [chromadb Rust Crate](https://crates.io/crates/chromadb)
- [GitHub: chromadb-rs](https://github.com/Anush008/chromadb-rs)
- [tealdeer](https://github.com/tealdeer-rs/tealdeer) - tldr Rust implementation

## Architecture Decisions

### Why ChromaDB?

1. **Simplicity**: Easy to set up and use
2. **Python Ecosystem**: Large community and good documentation
3. **Built-in Embeddings**: Can generate embeddings automatically
4. **Performance**: Fast vector similarity search
5. **Metadata Filtering**: Rich query capabilities

### Design Choices

1. **Separate Collections**: Different collection types for different data to allow targeted retrieval
2. **User Profiles**: Enable context switching for different work environments
3. **Project Context**: Automatically index project docs when in a git repository
4. **Learning System**: Track both successes (history) and failures (mistakes)
5. **Relevance Scoring**: Priority-based indexing (README > docs > other files)

## Future Enhancements

- [ ] Streaming retrieval for real-time context updates
- [ ] Automatic model download indexing (HuggingFace model cards)
- [ ] Integration with shell history (bash_history, zsh_history)
- [ ] Collaborative learning (opt-in anonymous command sharing)
- [ ] Natural language query for knowledge base
- [ ] Command suggestion based on context
- [ ] Integration with man page search tools
- [ ] Custom embedding models for command-specific semantics

## Contributing

When working on ChromaDB integration:

1. **API Compatibility**: Always check chromadb crate docs for current API
2. **Error Handling**: All ChromaDB operations should have proper error handling
3. **Privacy**: Never index sensitive data (credentials, tokens, etc.)
4. **Testing**: Add both unit and integration tests
5. **Documentation**: Update this file and code comments

## Known Issues

1. ChromaDB crate API may have changed - needs verification
2. Type annotations needed for query result processing
3. Where clause metadata structure needs investigation
4. Need to verify embedding generation behavior

## License

AGPL-3.0 (same as cmdai project)
