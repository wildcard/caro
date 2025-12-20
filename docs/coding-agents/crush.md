# Crush

**Charm.land's TUI Coding Agent**

Crush is a terminal-based AI coding assistant from Charm.land with beautiful TUI (Text User Interface), LSP integration, and direct shell access.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Charm.land |
| **Type** | TUI Agent |
| **Language** | Go |
| **License** | MIT |
| **Website** | [charm.land/crush](https://charm.land) |
| **Repository** | [github.com/charmbracelet/crush](https://github.com/charmbracelet/crush) |

## Installation

### Using Homebrew (macOS/Linux)

```bash
# Add Charm tap
brew tap charmbracelet/tap

# Install Crush
brew install charmbracelet/tap/crush
```

### Using Go

```bash
go install github.com/charmbracelet/crush@latest
```

### Using Curl (Binary)

```bash
# macOS/Linux
curl -fsSL https://charm.land/install/crush.sh | bash

# Or download from releases
# https://github.com/charmbracelet/crush/releases
```

### Verify Installation

```bash
crush --version
crush --help
```

## Configuration

Crush uses a JSON configuration file:

### .crush.json (Project-level)

```json
{
  "$schema": "https://charm.land/crush.json",
  "env": {
    "RUST_LOG": "warn",
    "RUST_BACKTRACE": "1"
  },
  "lsp": {
    "rust": {
      "command": "rust-analyzer",
      "settings": {
        "rust-analyzer.checkOnSave.command": "clippy"
      }
    },
    "python": {
      "command": "pylsp"
    },
    "typescript": {
      "command": "typescript-language-server",
      "args": ["--stdio"]
    }
  },
  "mcp": {
    "memory": {
      "command": "mcp-server-memory"
    },
    "caro": {
      "command": "caro",
      "args": ["--mcp-server"]
    }
  },
  "prompts": {
    "test": "Run tests for this file",
    "safety": "Check this code for safety issues",
    "shell": "Generate a safe shell command for: "
  },
  "rules": [
    "Always validate shell commands with caro before execution",
    "Follow TDD: RED-GREEN-REFACTOR cycle",
    "Run tests after changes"
  ]
}
```

### Global Configuration

Located at `~/.config/crush/config.json`:

```json
{
  "theme": "catppuccin",
  "editor": "nvim",
  "shell": "zsh",
  "models": {
    "default": "claude-sonnet-4-20250514",
    "fast": "claude-3-5-haiku-20241022"
  }
}
```

## Basic Usage

```bash
# Start Crush in current directory
crush

# Start with a specific prompt
crush "explain this codebase"

# Start in a specific directory
crush -C /path/to/project

# Use a specific model
crush --model claude-sonnet-4-20250514 "refactor this function"
```

## Key Features

### TUI Interface
- Beautiful terminal UI with Bubble Tea
- Split panes for code and chat
- File tree navigation
- Syntax highlighting

### LSP Integration
- Real-time diagnostics
- Code completion
- Go to definition
- Find references

### MCP Support
- Memory server for context
- Custom tool servers
- Caro integration

## Integration with Caro

### Method 1: MCP Server

Add Caro as an MCP server in `.crush.json`:

```json
{
  "mcp": {
    "caro": {
      "command": "caro",
      "args": ["--mcp-server"],
      "description": "Safe shell command generation and validation"
    }
  }
}
```

### Method 2: Custom Prompts

Create shell-aware prompts:

```json
{
  "prompts": {
    "shell": "Generate a shell command using caro: ",
    "validate": "Validate this command with caro --validate: ",
    "exec": "Generate and execute safely with caro --execute: "
  }
}
```

### Method 3: Rules-Based Integration

Enforce Caro usage through rules:

```json
{
  "rules": [
    "Before executing any shell command, validate with: caro --validate '<command>'",
    "For command generation, use: caro '<natural language request>'",
    "Never execute dangerous commands without caro safety confirmation"
  ]
}
```

## Crush Commands

### Built-in Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help |
| `/clear` | Clear conversation |
| `/files` | List files in context |
| `/add <file>` | Add file to context |
| `/remove <file>` | Remove file from context |
| `/run <cmd>` | Run shell command |
| `/test` | Run tests |
| `/diff` | Show pending changes |
| `/apply` | Apply changes |
| `/reject` | Reject changes |

### Custom Prompts (Shortcuts)

Access prompts with `@`:

```
@test          # Runs the "test" prompt
@safety        # Runs the "safety" prompt
@shell find    # Runs "shell" prompt with "find"
```

## Best Practices with Caro

### 1. Project Setup

Create a `.crush.json` with Caro integration:

```json
{
  "mcp": {
    "caro": {
      "command": "caro",
      "args": ["--mcp-server"]
    }
  },
  "prompts": {
    "cmd": "Use caro to generate: "
  },
  "rules": [
    "Use caro for all shell command generation",
    "Validate dangerous operations before execution"
  ]
}
```

### 2. Safe Shell Workflow

```bash
# In Crush session:
> @cmd delete old log files

# Crush asks Caro to generate
# Caro returns: find /var/log -name "*.log" -mtime +30 -delete
# With safety assessment: MODERATE RISK

# You can then:
/run caro --execute "find /var/log -name '*.log' -mtime +30 -delete"
```

### 3. Parallel with Claude Code

Both Crush and Claude Code can use the same Caro installation:

```
Crush (TUI) ──┐
              ├── Caro (Shell Safety) ──> Safe Commands
Claude Code ──┘
```

## Troubleshooting

### Common Issues

**Issue**: LSP not connecting
```bash
# Check LSP is installed
which rust-analyzer
which pylsp

# Verify .crush.json syntax
cat .crush.json | jq .
```

**Issue**: Caro MCP not working
```bash
# Ensure caro supports MCP mode
caro --help | grep mcp

# Check MCP server logs
crush --debug 2>&1 | grep caro
```

**Issue**: Theme not applying
```bash
# List available themes
crush themes

# Set theme in config
crush config set theme catppuccin
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+C` | Cancel/Exit |
| `Ctrl+L` | Clear screen |
| `Tab` | Autocomplete |
| `Ctrl+P` | Previous command |
| `Ctrl+N` | Next command |
| `Ctrl+/` | Toggle file tree |
| `F1` | Help |

## Resources

- [Crush Documentation](https://charm.land/crush/docs)
- [Charm.land Blog](https://charm.land/blog)
- [GitHub Repository](https://github.com/charmbracelet/crush)
- [Bubble Tea Framework](https://github.com/charmbracelet/bubbletea)

## Comparison with Claude Code

| Feature | Crush | Claude Code |
|---------|-------|-------------|
| Interface | TUI | CLI |
| LSP Support | Native | Via tools |
| File Navigation | Built-in tree | Tool-based |
| MCP | Yes | Yes |
| Editor Integration | Terminal | Terminal |
| Streaming | Yes | Yes |

## See Also

- [Claude Code](./claude-code.md) - Anthropic's CLI agent
- [Cursor](./cursor.md) - IDE-based agent
- [Caro Integration Guide](./README.md)
