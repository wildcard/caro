# Void

**Open-Source AI Code Editor**

Void is an open-source AI-powered code editor that provides Cursor-like features with full transparency and customization.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Void Team |
| **Type** | IDE |
| **Base** | VS Code Fork |
| **License** | MIT |
| **Website** | [voideditor.com](https://voideditor.com) |
| **Repository** | [github.com/voideditor/void](https://github.com/voideditor/void) |
| **Platforms** | macOS, Windows, Linux |

## Installation

### Direct Download

Download from [voideditor.com](https://voideditor.com).

### Build from Source

```bash
git clone https://github.com/voideditor/void
cd void
npm install
npm run build
```

### macOS with Homebrew

```bash
brew install --cask void
```

## Configuration

### Settings

```json
// ~/.config/Void/User/settings.json
{
  "void.ai.provider": "anthropic",
  "void.ai.model": "claude-sonnet-4-20250514",
  "void.ai.apiKey": "...",
  "void.chat.systemPrompt": "Validate shell commands with caro"
}
```

### Project Configuration

```
.void/
├── settings.json     # Project AI settings
├── context/          # Additional context files
└── prompts/          # Custom prompts
```

## Key Features

### Open Source
- Fully transparent codebase
- Community-driven development
- Self-hostable
- Privacy-focused

### AI Features
- Multi-model support
- Inline editing
- Chat interface
- Code completion

### Customization
- Custom providers
- Prompt templates
- Context providers
- Extension compatibility

## Integration with Caro

### Method 1: Settings Integration

Configure Void to use Caro:

```json
// .void/settings.json
{
  "ai": {
    "systemPrompt": "When generating shell commands, use caro for validation:\n1. Generate the command\n2. Suggest: caro --validate '<command>'\n3. For dangerous ops: caro --execute '<command>'"
  },
  "terminal": {
    "defaultCommand": "caro"
  }
}
```

### Method 2: Custom Provider

Create a Caro-aware provider:

```json
// .void/providers/caro.json
{
  "name": "caro",
  "type": "shell",
  "description": "Safe command generation",
  "command": "caro",
  "args": ["--json"],
  "parseOutput": "json"
}
```

### Method 3: Context Files

Add Caro documentation to context:

```markdown
<!-- .void/context/caro.md -->
# Caro Integration

This project uses `caro` for safe shell command execution.

## Commands
- `caro "<description>"` - Generate safe command
- `caro --validate "<cmd>"` - Validate existing command
- `caro --execute "<cmd>"` - Execute with confirmation

## Risk Levels
- SAFE - No confirmation needed
- MODERATE - May need confirmation
- HIGH - Requires confirmation
- CRITICAL - Blocked by default
```

### Method 4: Prompt Templates

Create Caro-aware prompts:

```markdown
<!-- .void/prompts/safe-shell.md -->
Generate a shell command for: {{input}}

Requirements:
1. POSIX-compliant syntax
2. Quote paths with spaces
3. Avoid destructive patterns

After generation, the user should validate with:
```bash
caro --validate "<your command>"
```
```

## Best Practices with Caro

### 1. Development Workflow

```bash
# In Void terminal
caro "set up development environment"
# Generates safe setup commands

caro --execute "cargo build"
# Builds with confirmation
```

### 2. Open Source Contribution

Since Void is open source, you can contribute Caro integration:

```typescript
// src/providers/caro.ts
export class CaroProvider implements CommandProvider {
  async generateCommand(prompt: string): Promise<Command> {
    const result = await exec(`caro --json "${prompt}"`);
    return JSON.parse(result);
  }

  async validateCommand(cmd: string): Promise<ValidationResult> {
    const result = await exec(`caro --validate --json "${cmd}"`);
    return JSON.parse(result);
  }
}
```

### 3. Privacy-Focused Usage

Void + Caro both support local operation:

```json
// Local-only configuration
{
  "void.ai.provider": "ollama",
  "void.ai.model": "codellama",
  "caro.backend": "embedded"
}
```

## Comparison with Cursor

| Feature | Void | Cursor |
|---------|------|--------|
| License | MIT (Open Source) | Proprietary |
| Price | Free | $20/mo Pro |
| Self-Host | Yes | No |
| Privacy | Full control | Cloud-based |
| Extensions | VS Code compatible | VS Code compatible |

## Troubleshooting

### Common Issues

**Issue**: Build failing
```bash
# Update dependencies
npm install
npm run rebuild
```

**Issue**: AI not connecting
```json
// Check API key
{
  "void.ai.apiKey": "your-key-here"
}
```

**Issue**: Caro integration not working
```bash
# Verify caro is installed
which caro

# Test command
caro --version
```

## Contributing

Void is open source - contribute Caro integration:

```bash
# Fork and clone
git clone https://github.com/your-fork/void
cd void

# Create feature branch
git checkout -b feature/caro-integration

# Make changes
# Submit PR
```

## Resources

- [Void Documentation](https://voideditor.com/docs)
- [GitHub Repository](https://github.com/voideditor/void)
- [Discord Community](https://discord.gg/void)
- [Contributing Guide](https://github.com/voideditor/void/blob/main/CONTRIBUTING.md)

## See Also

- [Cursor](./cursor.md) - Proprietary alternative
- [Windsurf](./windsurf.md) - Codeium's IDE
- [Continue](./continue.md) - Open-source extension
