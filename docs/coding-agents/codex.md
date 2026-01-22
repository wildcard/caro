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
  "systemPrompt": "You are a coding assistant. Always validate shell commands with caro."
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

### Method 1: Pipeline Integration

```bash
# Generate and validate in one pipeline
codex "command to delete temp files" | caro --validate

# Or capture and validate
cmd=$(codex --raw "find old log files")
caro --validate "$cmd"
```

### Method 2: System Prompt

Configure Codex to use Caro:

```json
// .codex/config.json
{
  "systemPrompt": "When generating shell commands:\n1. Generate POSIX-compliant commands\n2. Suggest validation: caro --validate '<command>'\n3. For dangerous operations, recommend: caro --execute '<command>'"
}
```

### Method 3: Custom Prompts

Create Caro-aware prompts:

```markdown
<!-- .codex/prompts/safe-shell.md -->
# Safe Shell Command Generator

Generate a shell command for: {{input}}

Requirements:
1. Use POSIX-compliant syntax
2. Avoid destructive operations without confirmation
3. Quote paths properly for spaces/special chars

After generation, validate with:
```bash
caro --validate "<command>"
```

If the command is risky, use:
```bash
caro --execute "<command>"
```
```

### Method 4: Wrapper Script

Create a Codex+Caro wrapper:

```bash
#!/bin/bash
# ~/bin/safe-codex

# Generate command with Codex
cmd=$(codex --raw "$@")

# Validate with Caro
echo "Generated: $cmd"
caro --validate "$cmd"

# Ask for execution
read -p "Execute? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    caro --execute "$cmd"
fi
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

### 1. Safe Command Generation

```bash
# Use Codex for intent, Caro for safety
codex "I need to clean up Docker" --raw | xargs -I {} caro --validate "{}"
```

### 2. Dangerous Operation Workflow

```bash
# Step 1: Generate with Codex
codex "remove all node_modules directories"
# Output: find . -name "node_modules" -type d -exec rm -rf {} +

# Step 2: Validate with Caro
caro --validate "find . -name 'node_modules' -type d -exec rm -rf {} +"
# Output: HIGH RISK - Recursive directory deletion

# Step 3: Execute safely with Caro
caro --execute "find . -name 'node_modules' -type d -exec rm -rf {} +"
# Shows confirmation prompt
```

### 3. Learning Workflow

```bash
# Have Codex explain what Caro validates
codex "explain this command" --context "caro --validate 'rm -rf /tmp/*'"
```

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
