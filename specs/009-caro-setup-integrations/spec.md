# Spec 009: Caro Setup Command - Editor & Tool Integrations

## Overview

Add a `caro setup` command that configures integrations with editors, tools, and shells with a single command.

**Inspiration**: Beads' `bd setup claude`, `bd setup cursor`, `bd setup aider`

## Problem Statement

Currently, users must manually configure:
- Shell completions
- Editor integrations
- Git hooks
- Environment variables

This friction reduces adoption. Power users who configure properly get 10x more value.

## Solution: `caro setup`

### Commands

```bash
# List available integrations
caro setup --list

# Configure specific integration
caro setup <INTEGRATION>

# Configure with options
caro setup <INTEGRATION> [OPTIONS]

# Remove integration
caro setup --remove <INTEGRATION>

# Check integration status
caro setup --status
```

### Supported Integrations

#### Phase 1: Shell Integrations

| Integration | Description | Files Modified |
|-------------|-------------|----------------|
| `zsh` | Zsh completion, aliases | `~/.zshrc` |
| `bash` | Bash completion, aliases | `~/.bashrc`, `~/.bash_profile` |
| `fish` | Fish completion, aliases | `~/.config/fish/config.fish` |

#### Phase 2: Editor Integrations

| Integration | Description | Files Modified |
|-------------|-------------|----------------|
| `claude` | Claude Code hooks, slash commands | `~/.claude/hooks.toml` |
| `cursor` | Cursor terminal profile | `.cursor/settings.json` |
| `vscode` | VS Code tasks, keybindings | `.vscode/tasks.json`, `.vscode/keybindings.json` |
| `aider` | Aider AI config | `~/.aider/config.yml` |

#### Phase 3: Workflow Integrations

| Integration | Description | Files Modified |
|-------------|-------------|----------------|
| `git-hooks` | Pre-commit safety checks | `.git/hooks/*` |
| `atuin` | Shell history integration | `~/.config/atuin/config.toml` |
| `starship` | Prompt integration | `~/.config/starship.toml` |

## Detailed Specifications

### Shell Integration: Zsh

```bash
caro setup zsh
```

**What it configures**:

1. **Completion** - Tab completion for caro commands
2. **Aliases** - Quick access shortcuts
3. **Function** - `c` function for quick command generation

**Generated `~/.zshrc` additions**:
```zsh
# Caro shell integration
eval "$(caro --completion zsh)"

# Quick alias
alias c='caro'

# Interactive command generation
function cmd() {
    local result=$(caro "$*" --output json 2>/dev/null | jq -r '.command')
    if [[ -n "$result" ]]; then
        print -z "$result"  # Add to command line buffer
    fi
}
```

### Claude Code Integration

```bash
caro setup claude
```

**What it configures**:

1. **Session start hook** - Injects system context
2. **Pre-execute hook** - Validates shell commands
3. **Slash commands** - `/caro:*` commands

**Generated `~/.claude/hooks.toml`**:
```toml
# Caro integration
# Installed by: caro setup claude

[[hooks]]
name = "caro-context"
event = "session_start"
command = "caro context --for-agent"
inject = true
type = "system_context"

[[hooks]]
name = "caro-validate"
event = "pre_shell_execute"
command = "caro validate --stdin --strict"
block_on_failure = true
```

### VS Code Integration

```bash
caro setup vscode
```

**What it configures**:

1. **Task** - Generate command task
2. **Keybinding** - Ctrl+Shift+G for generate
3. **Snippet** - Quick snippets

**Generated `.vscode/tasks.json`**:
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Caro: Generate Command",
      "type": "shell",
      "command": "caro",
      "args": ["${input:caroPrompt}"],
      "group": "none",
      "presentation": {
        "reveal": "always",
        "panel": "shared",
        "focus": true
      },
      "problemMatcher": []
    }
  ],
  "inputs": [
    {
      "id": "caroPrompt",
      "type": "promptString",
      "description": "Describe the command you need",
      "default": ""
    }
  ]
}
```

### Git Hooks Integration

```bash
caro setup git-hooks
```

**What it configures**:

1. **pre-commit** - Validates shell scripts in commit
2. **prepare-commit-msg** - Suggests commit messages (optional)

**Generated `.git/hooks/pre-commit`**:
```bash
#!/bin/bash
# Caro safety validation hook
# Installed by: caro setup git-hooks

set -e

# Find shell scripts in staged changes
SHELL_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(sh|bash|zsh)$' || true)

if [ -n "$SHELL_FILES" ]; then
    echo "Validating shell scripts with caro..."
    for file in $SHELL_FILES; do
        if ! caro validate --file "$file" --quiet; then
            echo "Safety validation failed: $file"
            echo "Run 'caro validate --file $file' for details"
            exit 1
        fi
    done
    echo "All shell scripts passed safety validation"
fi
```

## User Experience

### Interactive Setup

When run without arguments, `caro setup` offers interactive selection:

```
$ caro setup

Caro Setup - Configure integrations

Available integrations:
  [1] zsh       - Zsh shell completion and aliases
  [2] bash      - Bash shell completion and aliases
  [3] claude    - Claude Code hooks and context
  [4] vscode    - VS Code tasks and keybindings
  [5] git-hooks - Git pre-commit safety validation

  [a] All recommended for your environment
  [q] Quit

Select integrations (comma-separated): 1,3

Setting up zsh...
  Added completion to ~/.zshrc
  Added aliases to ~/.zshrc

Setting up claude...
  Created ~/.claude/hooks.toml
  Added session context hook
  Added validation hook

Setup complete! Reload your shell:
  source ~/.zshrc
```

### Status Check

```bash
$ caro setup --status

Caro Integration Status

  Shell:
    [x] zsh      - Configured (completion + aliases)
    [ ] bash     - Not configured
    [ ] fish     - Not configured

  Editors:
    [x] claude   - Configured (hooks active)
    [ ] vscode   - Not configured
    [ ] cursor   - Not configured

  Workflow:
    [ ] git-hooks - Not configured

  Recommendations:
    - Run 'caro setup git-hooks' to add safety checks to commits
```

## Implementation

### CLI Structure

```rust
#[derive(Parser)]
pub struct SetupCommand {
    /// Integration to configure
    #[arg(value_enum)]
    integration: Option<Integration>,

    /// List available integrations
    #[arg(long)]
    list: bool,

    /// Remove integration
    #[arg(long)]
    remove: bool,

    /// Check integration status
    #[arg(long)]
    status: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Clone, ValueEnum)]
pub enum Integration {
    Zsh,
    Bash,
    Fish,
    Claude,
    Cursor,
    Vscode,
    Aider,
    GitHooks,
    Atuin,
}
```

### Module Structure

```
src/
  commands/
    setup/
      mod.rs           # Setup command dispatcher
      shells/
        mod.rs         # Shell integration trait
        zsh.rs
        bash.rs
        fish.rs
      editors/
        mod.rs         # Editor integration trait
        claude.rs
        vscode.rs
        cursor.rs
      workflow/
        mod.rs         # Workflow integration trait
        git_hooks.rs
        atuin.rs
```

## Success Criteria

1. [ ] `caro setup zsh` configures completion in <5 seconds
2. [ ] `caro setup claude` creates valid hooks.toml
3. [ ] `caro setup --status` accurately reflects state
4. [ ] All integrations are idempotent (safe to run multiple times)
5. [ ] Removal with `--remove` cleanly undoes setup

## Future Extensions

- `caro setup --export` - Export setup as dotfiles
- `caro setup --import <url>` - Import setup from URL
- `caro setup team` - Team-shared configurations
- Integration with chezmoi/dotfiles managers
