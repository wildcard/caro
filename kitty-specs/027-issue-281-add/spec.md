# Command Autocomplete with Argument Inference

**Issue:** #281
**Milestone:** v1.1.0 (Due: Feb 15, 2026)
**Labels:** enhancement, backend, cli

## Overview

Add shell completion support to the Caro CLI, enabling users to get intelligent autocomplete for:
1. Subcommands and flags (static completions)
2. Argument values (dynamic completions)
3. Natural language command suggestions (argument inference)

## User Stories

### US1: Shell Completion Setup
**As a** power user,
**I want to** install shell completions for caro,
**So that** I can quickly navigate CLI options with Tab.

```bash
# Generate and install completions
caro completion bash > ~/.bash_completion.d/caro
caro completion zsh > ~/.zsh/completions/_caro
caro completion fish > ~/.config/fish/completions/caro.fish
```

### US2: Dynamic Argument Completion
**As a** user typing a command,
**I want to** get completions for argument values,
**So that** I don't have to remember valid options.

```bash
caro --backend <Tab>
# → embedded, ollama, exo, vllm

caro --shell <Tab>
# → bash, zsh, fish, sh, powershell, cmd

caro --safety <Tab>
# → strict, moderate, permissive
```

### US3: Command Suggestion (Argument Inference)
**As a** user typing a partial command description,
**I want to** get command suggestions from caro's pattern library,
**So that** I can discover relevant commands.

```bash
caro suggest "disk"
# Suggestions:
# - "show disk space by directory" → du -h -d 1 .
# - "show disk usage sorted" → du -sh */ | sort -rh

caro suggest "find files"
# Suggestions:
# - "find all files" → find . -type f
# - "find files by name" → find . -name "*.txt"
```

## Technical Design

### New Module Structure
```
src/completion/
├── mod.rs           # Public API exports
├── generator.rs     # Static completion script generation
├── dynamic.rs       # Runtime value completion
└── suggest.rs       # Natural language suggestions
```

### New Subcommand
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Generate shell completion scripts
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: ShellType,
    },

    /// Suggest commands matching a description
    Suggest {
        /// Partial command description
        query: String,
        /// Maximum number of suggestions
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
}
```

### Dependencies
```toml
[dependencies]
clap_complete = "4.5"  # For static completion generation
```

## Success Criteria

1. **Static completions work** for all shells (bash, zsh, fish)
2. **Dynamic completions work** for --backend, --shell, --safety
3. **`caro suggest`** returns relevant commands from StaticMatcher patterns
4. **All existing tests pass** after implementation
5. **New tests cover** completion and suggestion functionality

## Out of Scope (v1.1.0)

- Custom completer integration with shell RC files
- Machine learning-based suggestions
- Context-aware completions based on working directory

## Test Plan

```bash
# Unit tests for completion generation
cargo test completion_

# Integration test for suggest command
cargo test suggest_

# Manual verification
caro completion bash | head
caro suggest "find" --limit 3
```

## References

- [clap_complete documentation](https://docs.rs/clap_complete/latest/clap_complete/)
- Related PR: #41
- StaticMatcher patterns: `src/backends/static_matcher.rs`
