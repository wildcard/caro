# Codex CLI

**OpenAI's Terminal Coding Agent**

Codex CLI is OpenAI's command-line coding assistant that brings GPT-powered code generation to your terminal.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | OpenAI |
| **Type** | CLI Agent |
| **Language** | TypeScript/Node.js |
| **License** | MIT |
| **Repository** | [github.com/openai/codex-cli](https://github.com/openai/codex-cli) |

## Installation

### Using npm

```bash
# Install globally
npm install -g @openai/codex

# Or use npx without installing
npx @openai/codex
```

### Verify Installation

```bash
codex --version
codex --help
```

## Authentication

Codex requires an OpenAI API key:

```bash
# Set via environment variable
export OPENAI_API_KEY="sk-..."

# Or configure interactively
codex auth
```

## Configuration

### Project Configuration (.codex/)

```
.codex/
├── prompts/        # Custom prompts
├── config.json     # Project settings
└── context.md      # Project context
```

Example `config.json`:

```json
{
  "model": "gpt-4",
  "temperature": 0.2,
  "maxTokens": 4096,
  "systemPrompt": "You are a coding assistant."
}
```

### Global Configuration

Located at `~/.config/codex/`:

```json
{
  "defaultModel": "gpt-4",
  "editor": "nvim",
  "shell": "zsh",
  "safety": {
    "validateCommands": true,
    "confirmDangerous": true
  }
}
```

## Basic Usage

```bash
# Start interactive session
codex

# Single prompt
codex "explain this function"

# With specific file
codex "refactor main.py"

# Generate shell command
codex "command to find large files"
```

## Key Features

### Code Generation
- Natural language to code
- Multi-file editing
- Context-aware suggestions
- Language detection

### Terminal Integration
- Shell command generation
- Command explanation
- Script creation
- Error diagnosis

## Integration with Caro

### Guardian hook (`caro guard`, experimental)

Codex CLI supports Claude-compatible `PreToolUse` hooks, configured in
`~/.codex/hooks.json` or `~/.codex/config.toml`. Install Caro at user scope:

```toml
# ~/.codex/config.toml
[[hooks.PreToolUse]]
matcher = "^Bash$"

[[hooks.PreToolUse.hooks]]
type = "command"
command = "caro guard --harness codex"
timeout = 10
statusMessage = "caro guard"
```

- **Shadow mode** (the default) logs what Caro would have decided and changes
  nothing. Review it with `caro guard report`.
- **Enforce mode** (`--mode enforce` or `CARO_GUARD_MODE=enforce`) denies
  Critical commands and asks about High-risk ones.
- Caro never emits `additionalContext` on `PreToolUse`. Codex treats it as an
  error and the hook would fail open.

### Validating a command from a script

`caro guard --harness generic` takes `{"command": "..."}` on stdin. In
`--mode enforce` it prints a decision record and exits 0 for no objection, 2
for deny and 3 for ask; in the default shadow mode it only logs and exits 0:

```bash
cmd='find . -name node_modules -type d -exec rm -rf {} +'
jq -n --arg c "$cmd" '{command: $c}' | caro guard --harness generic --mode enforce --no-log
```

## Codex Commands

| Command | Description |
|---------|-------------|
| `codex` | Start interactive mode |
| `codex <prompt>` | Single prompt |
| `codex --raw` | Output only (no formatting) |
| `codex --model gpt-4` | Specify model |
| `codex auth` | Configure API key |
| `codex config` | Edit configuration |

## Best Practices with Caro

1. **Shadow first, then enforce** once `caro guard report` shows the
   would-be denials are ones you agree with.
2. **Use user-scope hooks** (`~/.codex/`) to keep the guard configuration out
   of the project workspace. This does not stop Codex from editing it when it
   has permission to write there.
3. **Use Caro for generation** when you want platform-correct POSIX commands:
   `caro "remove all node_modules directories"`.

## Troubleshooting

### Common Issues

**Issue**: API key not found
```bash
# Check environment
echo $OPENAI_API_KEY

# Set in shell profile
echo 'export OPENAI_API_KEY="sk-..."' >> ~/.zshrc
source ~/.zshrc
```

**Issue**: Rate limiting
```bash
# Use smaller model for simple tasks
codex --model gpt-3.5-turbo "simple request"
```

**Issue**: Command not found
```bash
# Check npm global bin
npm config get prefix
# Add to PATH if needed
export PATH="$(npm config get prefix)/bin:$PATH"
```

## Comparison with Claude Code

| Feature | Codex CLI | Claude Code |
|---------|-----------|-------------|
| Model | GPT-4 | Claude |
| Tool Use | Limited | Extensive |
| File Operations | Basic | Advanced |
| MCP Support | No | Yes |
| Price | Per-token | Per-token |
| Open Source | Yes | No |

## Resources

- [OpenAI API Documentation](https://platform.openai.com/docs)
- [Codex CLI GitHub](https://github.com/openai/codex-cli)
- [OpenAI Cookbook](https://cookbook.openai.com)

## See Also

- [Claude Code](./claude-code.md) - Anthropic's alternative
- [Aider](./aider.md) - Git-aware coding assistant
- [Caro Integration Guide](./README.md)
