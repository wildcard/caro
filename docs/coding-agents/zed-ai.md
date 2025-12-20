# Zed AI

**Native AI in Zed Editor**

Zed is a high-performance code editor with native AI integration, built from the ground up in Rust.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Zed Industries |
| **Type** | IDE with Native AI |
| **Language** | Rust |
| **License** | GPL-3.0 + MIT |
| **Website** | [zed.dev](https://zed.dev) |
| **Repository** | [github.com/zed-industries/zed](https://github.com/zed-industries/zed) |
| **Platforms** | macOS, Linux (Windows coming) |

## Installation

### macOS

```bash
# Using Homebrew
brew install --cask zed

# Or download from zed.dev
```

### Linux

```bash
# Download from GitHub releases
curl -L https://github.com/zed-industries/zed/releases/latest/download/zed-linux-x86_64.tar.gz | tar xz
sudo mv zed /usr/local/bin/
```

### Verify Installation

```bash
zed --version
zed .
```

## Configuration

### Settings

Press `Cmd+,` to open settings:

```json
// ~/.config/zed/settings.json
{
  "assistant": {
    "enabled": true,
    "default_model": {
      "provider": "anthropic",
      "model": "claude-sonnet-4-20250514"
    }
  },
  "terminal": {
    "shell": {
      "program": "zsh"
    }
  }
}
```

### AI Provider Configuration

```json
{
  "assistant": {
    "provider": {
      "name": "anthropic",
      "api_key": "..."
    }
  }
}
```

## Key Features

### Performance
- GPU-accelerated rendering
- Instant startup
- Minimal memory usage
- Native Rust implementation

### AI Features
- Assistant panel
- Inline transforms
- Multi-model support
- Context-aware suggestions

### Collaboration
- Real-time collaboration
- Voice chat
- Screen sharing

## Integration with Caro

### Method 1: Terminal Integration

Zed has an integrated terminal:

```bash
# Open terminal: Ctrl+`
caro "find all Rust files with TODO comments"
# Generates: grep -rn "TODO" --include="*.rs" .
```

### Method 2: Custom Tasks

Create Caro tasks in Zed:

```json
// .zed/tasks.json
[
  {
    "label": "Caro: Generate Command",
    "command": "caro",
    "args": ["$ZED_SELECTED_TEXT"],
    "use_new_terminal": true
  },
  {
    "label": "Caro: Validate Command",
    "command": "caro",
    "args": ["--validate", "$ZED_SELECTED_TEXT"],
    "use_new_terminal": false
  },
  {
    "label": "Caro: Execute Safe",
    "command": "caro",
    "args": ["--execute", "$ZED_SELECTED_TEXT"],
    "use_new_terminal": true
  }
]
```

### Method 3: Assistant Context

Add Caro awareness to AI context:

```markdown
<!-- .zed/context.md -->
# Project Context

## Shell Command Safety
This project uses `caro` for safe command generation:
- Generate: `caro "<description>"`
- Validate: `caro --validate "<command>"`
- Execute: `caro --execute "<command>"`

When the user asks for shell commands, suggest using caro.
```

### Method 4: Snippets

Create Caro snippets:

```json
// .zed/snippets.json
{
  "caro-validate": {
    "prefix": "carov",
    "body": "caro --validate \"$1\"",
    "description": "Validate command with Caro"
  },
  "caro-execute": {
    "prefix": "carox",
    "body": "caro --execute \"$1\"",
    "description": "Execute command safely with Caro"
  }
}
```

## Zed AI Features

### Assistant Panel

Open with `Cmd+?`:

```
User: How do I clean up old Docker images?

Zed AI: You can use Docker's prune commands. For safety, validate with caro:

caro --validate "docker image prune -a"

This shows the risk level before execution.
```

### Inline Transforms

Select code and press `Cmd+Enter`:
- Transform code with AI
- Generate documentation
- Refactor patterns

## Best Practices with Caro

### 1. Rust Development Workflow

Since both Zed and Caro are written in Rust:

```bash
# Use Caro for Cargo commands
caro "run tests with output"
# Generates: cargo test -- --nocapture

caro "build with optimizations"
# Generates: cargo build --release
```

### 2. Performance-Focused

Both prioritize performance:
- Zed: GPU-accelerated, instant startup
- Caro: Local inference, MLX optimization

### 3. Terminal-First

Zed's terminal is first-class:

```bash
# Efficient workflow
caro "find all files larger than 10MB" | pbcopy
# Copies safe command to clipboard
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+?` | Open Assistant |
| `Ctrl+`` ` | Open Terminal |
| `Cmd+Enter` | Inline transform |
| `Cmd+K` | Command palette |
| `Cmd+T` | Go to file |

## Comparison with Other Editors

| Feature | Zed | Cursor | VS Code |
|---------|-----|--------|---------|
| Language | Rust | Electron | Electron |
| Startup | <100ms | ~2s | ~3s |
| Memory | Low | High | Medium |
| AI | Native | Native | Extension |
| Collab | Native | No | Extension |

## Troubleshooting

### Common Issues

**Issue**: AI not responding
```json
// Check API key in settings
{
  "assistant": {
    "provider": {
      "api_key": "your-key"
    }
  }
}
```

**Issue**: Caro not in PATH
```json
// Add to terminal settings
{
  "terminal": {
    "env": {
      "PATH": "$HOME/.cargo/bin:$PATH"
    }
  }
}
```

**Issue**: Tasks not appearing
```bash
# Ensure tasks.json is valid
cat .zed/tasks.json | jq .
```

## Resources

- [Zed Documentation](https://zed.dev/docs)
- [Zed GitHub](https://github.com/zed-industries/zed)
- [Zed Discord](https://discord.gg/zed)
- [Zed Blog](https://zed.dev/blog)

## See Also

- [Cursor](./cursor.md) - AI-first editor
- [Void](./void.md) - Open-source alternative
- [Continue](./continue.md) - VS Code extension