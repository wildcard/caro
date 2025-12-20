# Windsurf

**Codeium's AI-Powered IDE**

Windsurf is Codeium's next-generation AI IDE with autonomous agents and deep codebase understanding.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Codeium |
| **Type** | IDE |
| **Base** | VS Code Fork |
| **License** | Proprietary |
| **Website** | [codeium.com/windsurf](https://codeium.com/windsurf) |
| **Platforms** | macOS, Windows, Linux |

## Installation

### Direct Download

Download from [codeium.com/windsurf](https://codeium.com/windsurf):
- macOS (Intel & Apple Silicon)
- Windows (x64)
- Linux (x64)

### macOS with Homebrew

```bash
brew install --cask windsurf
```

### Verify Installation

```bash
windsurf --version
windsurf .
```

## Configuration

### Settings

```json
// ~/.config/Windsurf/User/settings.json
{
  "windsurf.cascade.enabled": true,
  "windsurf.autocomplete.enabled": true,
  "windsurf.chat.defaultModel": "cascade",
  "terminal.integrated.defaultProfile.linux": "zsh"
}
```

### Project Configuration

```
.windsurf/
├── settings.json      # Project settings
├── cascade.json       # Cascade agent config
└── rules.md           # AI behavior rules
```

## Key Features

### Cascade Agent
- Autonomous coding agent
- Multi-step task completion
- Codebase navigation
- Terminal command execution

### Supercomplete
- Context-aware completions
- Multi-line suggestions
- Real-time updates

### Flow Mode
- Predictive editing
- Action anticipation
- Seamless transitions

## Integration with Caro

### Method 1: Terminal Integration

Use Caro in Windsurf's integrated terminal:

```bash
# In Windsurf terminal
caro "list all TypeScript files with errors"
# Generates safe, platform-aware command
```

### Method 2: Cascade Instructions

Configure Cascade to use Caro:

```json
// .windsurf/cascade.json
{
  "tools": {
    "caro": {
      "command": "caro",
      "description": "Generate and validate safe shell commands",
      "useFor": ["shell", "command", "script"]
    }
  },
  "rules": [
    "When executing shell commands, validate with: caro --validate",
    "For dangerous operations, use: caro --execute"
  ]
}
```

### Method 3: Rules File

Create Caro-aware rules:

```markdown
<!-- .windsurf/rules.md -->
# Windsurf Rules for This Project

## Shell Commands
- Always validate shell commands with `caro --validate "<command>"`
- Use `caro "<description>"` for generating new commands
- For destructive operations, require `caro --execute`

## Available Tools
- caro: Safe shell command generation and validation

## Workflow
1. Generate command: `caro "<what you want to do>"`
2. Validate: `caro --validate "<generated command>"`
3. Execute: `caro --execute "<validated command>"`
```

### Method 4: Task Configuration

Create Caro tasks:

```json
// .vscode/tasks.json (works in Windsurf)
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
      "label": "Caro: Safe Execute",
      "type": "shell",
      "command": "caro",
      "args": ["--execute", "${input:command}"]
    }
  ],
  "inputs": [
    {
      "id": "prompt",
      "type": "promptString",
      "description": "Describe the command"
    },
    {
      "id": "command",
      "type": "promptString",
      "description": "Command to execute"
    }
  ]
}
```

## Windsurf Features

### Cascade Agent

Cascade is Windsurf's autonomous agent:

```
User: Set up a new Rust project with tests

Cascade:
1. Creating project structure...
2. Initializing Cargo.toml...
3. Setting up test framework...
4. Validating build with caro --validate "cargo build"...
Done!
```

### Chat Commands

| Command | Description |
|---------|-------------|
| `/cascade` | Start Cascade agent |
| `/explain` | Explain code |
| `/edit` | Edit with AI |
| `/terminal` | Run in terminal |

## Best Practices with Caro

### 1. Cascade + Caro Workflow

```
User: Clean up the build directory

Cascade: I'll create a safe cleanup command.
Running: caro "delete build artifacts"
Generated: rm -rf target/debug target/release

Validating: caro --validate "rm -rf target/debug target/release"
Result: MODERATE - Removes build directories

Proceed with execution? [y/N]
```

### 2. Terminal Safety

```bash
# Always validate in Windsurf terminal
caro --validate "docker system prune -af"
# Shows risk level before you accidentally run it
```

### 3. Multi-Step Tasks

Configure Cascade for safe execution:

```markdown
<!-- .windsurf/rules.md -->
For multi-step operations:
1. Plan all commands first
2. Validate each with caro
3. Get user confirmation for MODERATE+ risk
4. Execute sequentially with error checking
```

## Comparison with Cursor

| Feature | Windsurf | Cursor |
|---------|----------|--------|
| Base | VS Code | VS Code |
| Agent | Cascade | Composer |
| Completions | Supercomplete | Tab |
| Flow Mode | Yes | No |
| Price | Free tier | $20/mo |

## Troubleshooting

### Common Issues

**Issue**: Cascade not executing
```json
// Enable cascade in settings
{
  "windsurf.cascade.enabled": true,
  "windsurf.cascade.allowTerminal": true
}
```

**Issue**: Caro not found
```bash
# Add to PATH in settings
{
  "terminal.integrated.env.linux": {
    "PATH": "${env:HOME}/.cargo/bin:${env:PATH}"
  }
}
```

**Issue**: Rules not applying
```bash
# Ensure rules.md is in project root
ls .windsurf/rules.md
# Restart Windsurf after changes
```

## Resources

- [Windsurf Documentation](https://codeium.com/windsurf/docs)
- [Codeium Blog](https://codeium.com/blog)
- [Discord Community](https://discord.gg/codeium)

## See Also

- [Cursor](./cursor.md) - Similar AI IDE
- [Void](./void.md) - Open-source alternative
- [Continue](./continue.md) - VS Code extension
