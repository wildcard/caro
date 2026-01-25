# GitHub Copilot

**Your AI Pair Programmer**

GitHub Copilot is GitHub's AI-powered code completion and chat assistant, powered by OpenAI Codex and GPT-4.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | GitHub / Microsoft |
| **Type** | IDE Extension + CLI |
| **License** | Proprietary |
| **Website** | [github.com/features/copilot](https://github.com/features/copilot) |
| **Platforms** | VS Code, JetBrains, Neovim, Xcode, CLI |
| **Price** | $10/month (Individual), $19/month (Business) |

## Installation

### VS Code

```bash
code --install-extension GitHub.copilot
code --install-extension GitHub.copilot-chat
```

### JetBrains

1. Preferences > Plugins
2. Search "GitHub Copilot"
3. Install and restart

### Neovim

```lua
-- Using lazy.nvim
{
  "github/copilot.vim",
  config = function()
    vim.g.copilot_no_tab_map = true
  end
}
```

### CLI (Copilot in the CLI)

```bash
# Install GitHub CLI first
brew install gh

# Install Copilot extension
gh extension install github/gh-copilot

# Authenticate
gh auth login
```

## Configuration

### VS Code Settings

```json
// settings.json
{
  "github.copilot.enable": {
    "*": true,
    "yaml": true,
    "markdown": true
  },
  "github.copilot.advanced": {
    "inlineSuggestCount": 3
  }
}
```

### CLI Configuration

```bash
# Configure Copilot CLI
gh copilot config

# Set default behavior
gh copilot alias add "explain" "gh copilot explain"
gh copilot alias add "suggest" "gh copilot suggest"
```

## Key Features

### Code Completion
- Inline suggestions
- Multi-line completions
- Context-aware
- Language-agnostic

### Copilot Chat
- Chat sidebar
- Inline chat (Cmd+I)
- @workspace mentions
- /commands

### Copilot CLI
- Command explanations
- Command suggestions
- Shell integration

## Integration with Caro

### Method 1: CLI Integration

Use Copilot with Caro in terminal:

```bash
# Copilot suggests a command
gh copilot suggest "find large files"
# Output: find . -type f -size +100M

# Validate with Caro
caro --validate "find . -type f -size +100M"
# Output: SAFE - Standard file search

# Or use Caro directly
caro "find large files"
# Platform-aware, validated output
```

### Method 2: Chat Instructions

Configure Copilot Chat context:

```markdown
<!-- .github/copilot-instructions.md -->
# Copilot Instructions for this Repository

## Shell Commands
When generating shell commands:
1. Use POSIX-compliant syntax for portability
2. Suggest validation: `caro --validate "<command>"`
3. For destructive operations, recommend: `caro --execute "<command>"`

## Tools Available
- `caro` - Safe shell command generation and validation
- `cmdai` - Alias for caro

## Safety Patterns
Never suggest these without caro validation:
- rm -rf with wildcards
- chmod 777
- dd commands
- curl | bash patterns
```

### Method 3: Shell Aliases

Create combined workflows:

```bash
# ~/.zshrc
copilot_safe() {
    local suggestion
    suggestion=$(gh copilot suggest -t shell "$@" 2>/dev/null | tail -1)
    echo "Suggested: $suggestion"
    caro --validate "$suggestion"
}

# Usage
copilot_safe "delete old log files"
```

### Method 4: Workspace Chat

In Copilot Chat, reference Caro:

```
@workspace How do I safely delete temporary files?

# Copilot will see .github/copilot-instructions.md and suggest caro
```

## Copilot Commands

### Chat Commands

| Command | Description |
|---------|-------------|
| `/explain` | Explain selected code |
| `/fix` | Fix problems |
| `/tests` | Generate tests |
| `/docs` | Generate documentation |
| `/new` | Create new code |
| `/simplify` | Simplify code |

### CLI Commands

```bash
# Explain a command
gh copilot explain "find . -name '*.log' -mtime +30 -delete"

# Suggest a command
gh copilot suggest "compress all images"

# Interactive mode
gh copilot
```

## Best Practices with Caro

### 1. Defense in Depth

```bash
# Copilot generates
gh copilot suggest "clean docker system"
# Output: docker system prune -af

# Caro validates
caro --validate "docker system prune -af"
# Output: MODERATE - Removes all unused Docker data

# Safe execution
caro --execute "docker system prune -af"
```

### 2. Learning Workflow

```bash
# Use both for learning
gh copilot explain "$(caro --raw 'find large files')"
# Copilot explains the Caro-generated command
```

### 3. Cross-Platform Development

```
User: @workspace Generate a build script

Copilot: I'll create a cross-platform build script.
Based on the copilot-instructions.md, I'll suggest validating with caro:

```bash
caro --validate "make build"
```
```

## Copilot Chat Context

### @mentions

| Mention | Description |
|---------|-------------|
| `@workspace` | Entire workspace |
| `@vscode` | VS Code settings |
| `@terminal` | Terminal context |

### File Instructions

Create `.github/copilot-instructions.md` for project-specific guidance:

```markdown
# Project Guidelines

This project uses `caro` for safe shell command execution.

## Build Commands
- `cargo build --release` - Build project
- `caro --execute "cargo build"` - Safe build with confirmation

## Test Commands
- `cargo test` - Run tests
- `make test` - Run full test suite

## Deployment
All deployment commands MUST use caro:
- `caro --execute "deploy-prod.sh"`
```

## Comparison with Other Assistants

| Feature | Copilot | Cody | Continue |
|---------|---------|------|----------|
| Completions | Best-in-class | Good | Good |
| Chat | Yes | Yes | Yes |
| CLI | Yes | Limited | No |
| Free | No ($10/mo) | Free tier | Free |
| Enterprise | Yes | Yes | No |

## Troubleshooting

### Common Issues

**Issue**: Copilot not suggesting
```bash
# Check subscription
gh auth status

# Verify extension is enabled
# Settings > GitHub Copilot > Enable
```

**Issue**: CLI not working
```bash
# Reinstall extension
gh extension remove github/gh-copilot
gh extension install github/gh-copilot
```

**Issue**: Chat not responding
```bash
# Check rate limits
gh api rate_limit
```

## Resources

- [Copilot Documentation](https://docs.github.com/copilot)
- [Copilot CLI](https://githubnext.com/projects/copilot-cli/)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=GitHub.copilot)
- [GitHub Community](https://github.com/orgs/community/discussions/categories/copilot)

## See Also

- [Cody](./cody.md) - Sourcegraph's alternative
- [Continue](./continue.md) - Open-source option
- [Claude Code](./claude-code.md) - Anthropic's CLI
