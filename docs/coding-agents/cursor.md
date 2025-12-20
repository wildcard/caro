# Cursor

**AI-First Code Editor**

Cursor is an AI-first code editor built on VS Code, designed to make you extraordinarily productive with AI-assisted coding.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Cursor, Inc. |
| **Type** | IDE |
| **Base** | VS Code Fork |
| **License** | Proprietary |
| **Website** | [cursor.sh](https://cursor.sh) |
| **Platforms** | macOS, Windows, Linux |

## Installation

### Direct Download

Download from [cursor.sh](https://cursor.sh) for your platform:
- macOS (Intel & Apple Silicon)
- Windows (x64)
- Linux (x64, AppImage)

### macOS with Homebrew

```bash
brew install --cask cursor
```

### Linux Package Managers

```bash
# AppImage (universal)
wget https://download.cursor.sh/linux/appImage/latest
chmod +x cursor-*.AppImage
./cursor-*.AppImage

# Arch Linux (AUR)
yay -S cursor-bin
```

### Verify Installation

```bash
# Open Cursor from terminal
cursor --version

# Open in current directory
cursor .
```

## Configuration

### Settings

Cursor inherits VS Code settings with AI-specific additions:

```json
// ~/.config/Cursor/User/settings.json (Linux)
// ~/Library/Application Support/Cursor/User/settings.json (macOS)
{
  "cursor.cpp.enabled": true,
  "cursor.chat.defaultModel": "claude-sonnet-4-20250514",
  "cursor.prediction.enabled": true,
  "cursor.aiContext.enabled": true,
  "terminal.integrated.defaultProfile.linux": "zsh"
}
```

### Project Configuration

Create `.cursor/` directory for project-specific settings:

```
.cursor/
├── settings.json     # Project settings
├── rules.json        # AI behavior rules
└── commands/         # Custom commands
```

Example `.cursor/rules.json`:

```json
{
  "rules": [
    "Always validate shell commands with caro before execution",
    "Follow the project's CLAUDE.md guidelines",
    "Use POSIX-compliant commands for cross-platform support"
  ]
}
```

## Key Features

### AI Features
- **Tab Completion** - AI-powered code completion
- **Cmd+K** - Inline code generation/editing
- **Chat** - Sidebar chat with codebase context
- **Composer** - Multi-file editing agent
- **@symbols** - Reference files, docs, web in prompts

### Editor Features
- Full VS Code extension compatibility
- Integrated terminal
- Git integration
- Debugger support
- Remote development

## Integration with Caro

### Method 1: Terminal Integration

Use Caro directly in Cursor's integrated terminal:

```bash
# In Cursor terminal
caro "find all TypeScript files with TODO comments"
# Generates platform-safe command with validation
```

### Method 2: Custom Task

Create a VS Code task for Caro:

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Caro: Generate Command",
      "type": "shell",
      "command": "caro",
      "args": ["${input:prompt}"],
      "presentation": {
        "reveal": "always",
        "panel": "new"
      }
    },
    {
      "label": "Caro: Validate Command",
      "type": "shell",
      "command": "caro",
      "args": ["--validate", "${input:command}"],
      "presentation": {
        "reveal": "always"
      }
    }
  ],
  "inputs": [
    {
      "id": "prompt",
      "type": "promptString",
      "description": "Describe the command you need"
    },
    {
      "id": "command",
      "type": "promptString",
      "description": "Command to validate"
    }
  ]
}
```

### Method 3: Cursor Rules

Add Caro awareness to Cursor's AI:

```json
// .cursor/rules.json
{
  "rules": [
    "When generating shell commands, consider using 'caro' for safety validation",
    "For potentially dangerous operations (rm, chmod, dd), always suggest running through caro first",
    "Platform-specific commands should be validated with 'caro --validate' to ensure POSIX compliance"
  ],
  "context": {
    "tools": ["caro - Safe shell command generation and validation CLI"]
  }
}
```

### Method 4: Custom Commands

Create Cursor commands that use Caro:

```markdown
<!-- .cursor/commands/safe-shell.md -->
# Safe Shell Command

Generate a shell command for the following request, then validate it with caro:

Request: $ARGUMENTS

Steps:
1. Generate the appropriate shell command
2. Run: `caro --validate "<your_command>"`
3. If validation passes, show the command
4. If validation fails, explain why and suggest alternatives
```

## Cursor Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+K` | Inline AI edit |
| `Cmd+L` | Open chat |
| `Cmd+I` | Open Composer |
| `Cmd+Shift+K` | Generate in selection |
| `Ctrl+`` ` | Toggle terminal |
| `Cmd+Shift+P` | Command palette |

## Cursor vs VS Code

| Feature | Cursor | VS Code |
|---------|--------|---------|
| AI Chat | Built-in | Extension needed |
| Tab Completion | Native AI | Copilot extension |
| Multi-file Agent | Composer | Not available |
| Codebase Context | @codebase | Manual |
| Price | $20/mo Pro | Free (+ Copilot $10/mo) |

## Best Practices with Caro

### 1. Terminal Safety

```bash
# Before running dangerous commands in terminal
caro --validate "docker system prune -af"
# Shows risk level and requires confirmation
```

### 2. Composer Integration

When using Composer for multi-file changes:
```
Generate a shell script to deploy this project.
Validate each command with caro for safety.
```

### 3. Chat Context

Use @-mentions with Caro awareness:
```
@caro-docs How do I safely delete old log files?
# References Caro documentation in chat
```

## Troubleshooting

### Common Issues

**Issue**: Caro not found in terminal
```bash
# Add to PATH in Cursor's terminal profile
export PATH="$HOME/.cargo/bin:$PATH"

# Or configure in settings.json
{
  "terminal.integrated.env.linux": {
    "PATH": "${env:HOME}/.cargo/bin:${env:PATH}"
  }
}
```

**Issue**: Extensions not working
```bash
# Cursor uses its own extension directory
# Install extensions via Cursor, not VS Code CLI
```

**Issue**: AI context not including files
```json
// Ensure AI context is enabled
{
  "cursor.aiContext.enabled": true,
  "cursor.aiContext.includeSymbols": true
}
```

## Extension Recommendations

Extensions that work well with Caro:

| Extension | Purpose |
|-----------|---------|
| ShellCheck | Shell script linting |
| POSIX Shell | POSIX compliance checking |
| Terminal Tabs | Better terminal management |

## Resources

- [Cursor Documentation](https://cursor.sh/docs)
- [Cursor Blog](https://cursor.sh/blog)
- [VS Code Documentation](https://code.visualstudio.com/docs)
- [Cursor GitHub Issues](https://github.com/getcursor/cursor/issues)

## See Also

- [VS Code Extensions](./cline.md) - Standalone agent extensions
- [Windsurf](./windsurf.md) - Codeium's AI IDE
- [Continue](./continue.md) - Open-source alternative
