# Continue

**Open-Source AI Code Assistant**

Continue is an open-source AI code assistant for VS Code and JetBrains that connects to any LLM.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Continue |
| **Type** | IDE Extension |
| **License** | Apache 2.0 |
| **Website** | [continue.dev](https://continue.dev) |
| **Repository** | [github.com/continuedev/continue](https://github.com/continuedev/continue) |
| **Platforms** | VS Code, JetBrains |

## Installation

### VS Code

1. Open VS Code
2. Go to Extensions (Cmd+Shift+X / Ctrl+Shift+X)
3. Search for "Continue"
4. Click Install

Or install via command line:

```bash
code --install-extension continue.continue
```

### JetBrains

1. Open your JetBrains IDE
2. Go to Preferences > Plugins
3. Search for "Continue"
4. Click Install

## Configuration

### config.json

Located at `~/.continue/config.json`:

```json
{
  "models": [
    {
      "title": "Claude Sonnet",
      "provider": "anthropic",
      "model": "claude-sonnet-4-20250514",
      "apiKey": "..."
    },
    {
      "title": "GPT-4",
      "provider": "openai",
      "model": "gpt-4",
      "apiKey": "..."
    },
    {
      "title": "Ollama Local",
      "provider": "ollama",
      "model": "codellama:7b"
    }
  ],
  "customCommands": [
    {
      "name": "safe-shell",
      "description": "Generate safe shell command",
      "prompt": "Generate a shell command for: {{{ input }}}. Validate with caro."
    }
  ],
  "contextProviders": [
    { "name": "code" },
    { "name": "docs" },
    { "name": "terminal" }
  ]
}
```

### Project Configuration

Create `.continue/` in your project:

```
.continue/
├── config.json       # Project-specific config
├── prompts/          # Custom prompts
└── context/          # Context providers
```

## Key Features

### Multi-Model Support
- Anthropic Claude
- OpenAI GPT-4
- Google Gemini
- Local models (Ollama, LM Studio)

### IDE Integration
- Inline code editing
- Chat sidebar
- Context-aware completions
- Terminal integration

### Customization
- Custom commands
- Context providers
- Prompt templates
- Model switching

## Integration with Caro

### Method 1: Custom Command

Add a Caro command to Continue:

```json
// ~/.continue/config.json
{
  "customCommands": [
    {
      "name": "caro",
      "description": "Generate safe shell command with Caro",
      "prompt": "Generate a shell command for the user's request. After generation, tell them to validate with: caro --validate \"<command>\".\n\nUser request: {{{ input }}}"
    },
    {
      "name": "caro-validate",
      "description": "Validate a shell command",
      "prompt": "The user wants to run this command: {{{ input }}}\n\nAnalyze it for safety. Then tell them to validate with: caro --validate \"{{{ input }}}\""
    }
  ]
}
```

### Method 2: Terminal Context

Continue can see terminal output. Use Caro in integrated terminal:

```bash
# In VS Code/JetBrains terminal
caro "find large files"

# Continue sees the output and can help further
```

### Method 3: Context Provider

Create a Caro context provider:

```typescript
// ~/.continue/context/caro.ts
export const CaroContextProvider = {
  name: "caro",
  displayTitle: "Caro Commands",
  description: "Safe shell command suggestions",

  getContextItems: async (query: string) => {
    const { execSync } = require("child_process");
    const result = execSync(`caro --json "${query}"`).toString();
    const data = JSON.parse(result);

    return [{
      name: "caro-suggestion",
      description: data.command,
      content: `Suggested command: ${data.command}\nRisk level: ${data.risk_level}`
    }];
  }
};
```

### Method 4: Slash Command

Create a `/caro` slash command:

```json
// ~/.continue/config.json
{
  "slashCommands": [
    {
      "name": "caro",
      "description": "Generate safe shell command",
      "command": "caro --json"
    }
  ]
}
```

## Continue Commands

### Chat Commands

| Command | Description |
|---------|-------------|
| `/edit` | Edit selected code |
| `/comment` | Add comments |
| `/explain` | Explain code |
| `/test` | Generate tests |
| `/terminal` | Run in terminal |

### Custom Commands

Access with `/command-name`:

```
/caro find all large files
/caro-validate rm -rf tmp/
```

## Best Practices with Caro

### 1. Terminal Workflow

```bash
# In Continue's terminal panel
caro "clean up build artifacts"
# Shows command with risk assessment

# If safe, execute
caro --execute "make clean"
```

### 2. Chat Integration

```
User: How do I find all files larger than 100MB?

Continue: You can use the find command. Let me help you generate a safe version:

Run this in your terminal:
caro "find files larger than 100MB"

This will generate a platform-appropriate command with safety validation.
```

### 3. Code Review

```json
// Custom command for reviewing shell scripts
{
  "name": "review-shell",
  "description": "Review shell script for safety",
  "prompt": "Review this shell script for safety issues. Suggest validating each command with 'caro --validate'.\n\n{{{ input }}}"
}
```

## Troubleshooting

### Common Issues

**Issue**: Continue not loading
```bash
# Check extension is installed
code --list-extensions | grep continue

# Reload VS Code
# Cmd+Shift+P > Developer: Reload Window
```

**Issue**: Model not connecting
```json
// Verify API key in config.json
{
  "models": [{
    "apiKey": "sk-..."  // Check this is correct
  }]
}
```

**Issue**: Custom command not working
```bash
# Check config.json syntax
cat ~/.continue/config.json | jq .
```

## Comparison with Other Extensions

| Feature | Continue | Copilot | Cody |
|---------|----------|---------|------|
| Open Source | Yes | No | Yes |
| Multi-Model | Yes | No | Yes |
| Self-Hosted | Yes | No | Yes |
| Price | Free | $10/mo | Free tier |
| Customization | Extensive | Limited | Good |

## Resources

- [Continue Documentation](https://continue.dev/docs)
- [Continue Discord](https://discord.gg/continue)
- [GitHub Repository](https://github.com/continuedev/continue)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=Continue.continue)

## See Also

- [Cursor](./cursor.md) - AI-first IDE
- [Cody](./cody.md) - Sourcegraph's assistant
- [GitHub Copilot](./github-copilot.md) - GitHub's AI assistant
