# Aider

**Git-Aware AI Pair Programmer**

Aider is an AI pair programming assistant that works with your local git repository, making it easy to edit code with GPT-4, Claude, and other LLMs.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Paul Gauthier |
| **Type** | CLI Agent |
| **Language** | Python |
| **License** | Apache 2.0 |
| **Website** | [aider.chat](https://aider.chat) |
| **Repository** | [github.com/paul-gauthier/aider](https://github.com/paul-gauthier/aider) |

## Installation

### Using pip

```bash
# Install from PyPI
pip install aider-chat

# Or with pipx for isolation
pipx install aider-chat
```

### Using Homebrew (macOS)

```bash
brew install aider
```

### From Source

```bash
git clone https://github.com/paul-gauthier/aider
cd aider
pip install -e .
```

### Verify Installation

```bash
aider --version
aider --help
```

## Configuration

### API Keys

```bash
# Set API keys
export ANTHROPIC_API_KEY="..."   # For Claude
export OPENAI_API_KEY="..."      # For GPT-4

# Or configure in aider
aider --api-key anthropic=sk-...
```

### Configuration File

Located at `~/.aider.conf.yml`:

```yaml
model: claude-sonnet-4-20250514
auto-commits: true
git-depth: 2
show-diffs: true

# Shell command settings
shell:
  validator: caro --validate
  executor: caro --execute
```

### Project Configuration

Create `.aider.conf.yml` in project root:

```yaml
model: claude-sonnet-4-20250514
read:
  - README.md
  - CLAUDE.md
context:
  - "Always validate shell commands with caro"
  - "Use POSIX-compliant syntax"
```

## Basic Usage

```bash
# Start aider in current directory
aider

# With specific files
aider src/main.rs src/lib.rs

# With specific model
aider --model claude-sonnet-4-20250514

# In architect mode (planning first)
aider --architect
```

## Key Features

### Git Integration
- Automatic commits for changes
- Meaningful commit messages
- Easy rollback with git
- Respects .gitignore

### Multi-File Editing
- Add files to context with `/add`
- Edit multiple files together
- Understands file relationships

### Conversation Modes
- Code mode (default)
- Ask mode (questions only)
- Architect mode (plan first)

## Integration with Caro

### Method 1: Shell Configuration

Configure aider to use Caro for shell commands:

```yaml
# ~/.aider.conf.yml
shell:
  enabled: true
  validator: "caro --validate"
  confirm_dangerous: true
```

### Method 2: /run with Validation

```bash
# In aider session:
> /run caro "find large files"
# Caro generates safe command

> /run caro --validate "rm -rf tmp/"
# Validates before execution
```

### Method 3: Context Instructions

Add Caro awareness to your project:

```yaml
# .aider.conf.yml
context:
  - "When generating shell commands, use 'caro' for generation and validation"
  - "Always run 'caro --validate' before executing potentially dangerous commands"
  - "Use 'caro --execute' for safe command execution with confirmation"
```

### Method 4: Pre-commit Hook

Create an aider-caro integration hook:

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Extract shell commands from staged changes
commands=$(git diff --cached | grep -E "^\+.*\$\(" | sed 's/.*\$(\(.*\))/\1/')

for cmd in $commands; do
    if ! caro --validate "$cmd" --quiet; then
        echo "Dangerous command detected: $cmd"
        exit 1
    fi
done
```

## Aider Commands

| Command | Description |
|---------|-------------|
| `/add <file>` | Add file to chat |
| `/drop <file>` | Remove file from chat |
| `/run <cmd>` | Run shell command |
| `/diff` | Show pending changes |
| `/commit` | Commit changes |
| `/undo` | Undo last change |
| `/ask` | Switch to ask mode |
| `/code` | Switch to code mode |
| `/help` | Show help |

## Git-Aware Workflow with Caro

### Safe Development Cycle

```bash
# Start aider with safety context
aider --context "Validate all shell commands with caro"

# In session:
> Create a script to deploy the application

# Aider generates deploy.sh
# Before running:
> /run caro --validate "bash deploy.sh"

# If safe:
> /run bash deploy.sh
```

### Automatic Commits with Validation

```yaml
# .aider.conf.yml
auto-commits: true
pre-commit:
  - "caro --validate --quiet"
```

## Best Practices with Caro

### 1. Development Environment

```bash
# Start both tools for a session
aider --model claude-sonnet-4-20250514 &
caro --watch  # Watch for command validation requests
```

### 2. Safe Refactoring

```bash
# In aider:
> Refactor the build script for cross-platform support

# Aider modifies build.sh
# Validate each command:
> /run caro --validate "$(cat build.sh)"
```

### 3. CI/CD Integration

```yaml
# .github/workflows/validate.yml
jobs:
  validate-scripts:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Caro
        run: cargo install cmdai
      - name: Validate shell scripts
        run: |
          for script in scripts/*.sh; do
            caro --validate "$(cat $script)" || exit 1
          done
```

## Comparison with Other Agents

| Feature | Aider | Claude Code | Cursor |
|---------|-------|-------------|--------|
| Git Integration | Native | Tool-based | Basic |
| Auto-commit | Yes | No | No |
| Multi-file | Yes | Yes | Yes |
| Interface | CLI | CLI | IDE |
| Models | Multiple | Claude | Multiple |
| Price | Per-token | Per-token | Subscription |

## Troubleshooting

### Common Issues

**Issue**: Aider not finding files
```bash
# Check you're in git repo
git status

# Ensure files are tracked
git add .
```

**Issue**: API key errors
```bash
# Verify key is set
echo $ANTHROPIC_API_KEY

# Use correct format
aider --api-key anthropic=sk-ant-...
```

**Issue**: Caro validation in /run
```bash
# Ensure caro is in PATH
which caro

# Or use full path
/run ~/.cargo/bin/caro --validate "command"
```

## Resources

- [Aider Documentation](https://aider.chat/docs/)
- [Aider GitHub](https://github.com/paul-gauthier/aider)
- [Aider Discord](https://discord.gg/aider)
- [Blog - Pair Programming with AI](https://aider.chat/blog/)

## See Also

- [Claude Code](./claude-code.md) - Anthropic's CLI agent
- [Codex CLI](./codex.md) - OpenAI's terminal agent
- [Caro Integration Guide](./README.md)
