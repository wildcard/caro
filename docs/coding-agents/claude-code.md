# Claude Code

**Anthropic's Official CLI for Claude**

Claude Code is Anthropic's official command-line interface for Claude, designed for software engineering tasks. It provides agentic coding capabilities with direct terminal integration.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Anthropic |
| **Type** | CLI Agent |
| **Language** | TypeScript/Node.js |
| **License** | Proprietary |
| **Website** | [claude.ai/code](https://claude.ai/code) |
| **Documentation** | [docs.anthropic.com](https://docs.anthropic.com/en/docs/claude-code) |

## Installation

### Using npm (Recommended)

```bash
# Install globally
npm install -g @anthropic-ai/claude-code

# Or use npx without installing
npx @anthropic-ai/claude-code
```

### Using Homebrew (macOS)

```bash
brew install anthropic/tap/claude-code
```

### Verify Installation

```bash
claude --version
claude --help
```

## Authentication

Claude Code requires an Anthropic API key:

```bash
# Set via environment variable
export ANTHROPIC_API_KEY="your-api-key"

# Or authenticate interactively
claude auth login
```

## Basic Usage

```bash
# Start an interactive session
claude

# Run a single command
claude "explain this codebase"

# Work on a specific file
claude "fix the bug in src/main.rs"

# Generate and execute commands
claude "find all TODO comments in the project"
```

## Integration with Caro

### Method 1: Direct Command Validation

Claude Code can use Caro to validate generated shell commands:

```bash
# In Claude Code session
> Generate a command to delete old log files
# Claude generates: find /var/log -name "*.log" -mtime +30 -delete

# Validate with Caro
caro --validate "find /var/log -name '*.log' -mtime +30 -delete"
```

### Method 2: MCP Server Integration

Claude Code supports Model Context Protocol (MCP) servers. Configure Caro as an MCP server:

```json
// ~/.claude/mcp_servers.json
{
  "caro": {
    "command": "caro",
    "args": ["--mcp-server"],
    "description": "Safe shell command generation"
  }
}
```

### Method 3: Custom Slash Commands

Create a Claude Code command that uses Caro:

```markdown
<!-- .claude/commands/safe-shell.md -->
Generate a shell command for: $ARGUMENTS

Use caro to validate the command before execution:
1. Generate the command based on the request
2. Run: caro --validate "<command>"
3. If safe, execute with user confirmation
```

## Claude Code Features

### Agentic Capabilities
- Multi-file editing
- Codebase exploration
- Git operations
- Test execution
- Build and deployment

### Tool Use
- File operations (read, write, edit)
- Bash command execution
- Web fetching
- Todo list management
- Task spawning (sub-agents)

### Context Management
- Automatic summarization
- Project context (CLAUDE.md)
- Memory across sessions
- Custom instructions

## Configuration

### Project Configuration (CLAUDE.md)

```markdown
# CLAUDE.md

Project-specific instructions for Claude Code.

## Commands
- Use `cargo test` for running tests
- Use `make lint` for linting
- Always validate shell commands with `caro --validate`

## Safety
- Never run commands that modify system directories
- Always confirm before deleting files
```

### User Settings

Claude Code settings are stored in `~/.claude/`:

```
~/.claude/
├── settings.json     # User preferences
├── mcp_servers.json  # MCP server configurations
├── agents/           # Custom agent definitions
└── commands/         # Custom slash commands
```

## Best Practices with Caro

### 1. Safety-First Workflow

```bash
# Have Claude generate the command
> Generate a command to clean up Docker images

# Claude outputs: docker system prune -af

# Validate before executing
caro --validate "docker system prune -af"
# Output: WARNING - Moderate risk: Removes all unused Docker data
```

### 2. Platform-Aware Generation

```bash
# Use Caro for platform-specific commands
> I need to find large files

# Instead of Claude guessing, use Caro:
caro "find files larger than 100MB"
# Automatically uses correct syntax for BSD/GNU
```

### 3. Execution with Confirmation

```bash
# Safe execution pattern
caro --execute "$(claude 'generate cleanup command')"
```

## Troubleshooting

### Common Issues

**Issue**: Claude Code can't find Caro
```bash
# Ensure caro is in PATH
which caro
# If not found, add to PATH or use full path
export PATH="$PATH:$HOME/.cargo/bin"
```

**Issue**: MCP server not connecting
```bash
# Check MCP server configuration
claude config mcp list
# Restart Claude Code after config changes
```

**Issue**: Rate limiting
```bash
# Check API usage
claude usage
# Consider using local models via Caro for high-volume tasks
```

## Resources

- [Claude Code Documentation](https://docs.anthropic.com/en/docs/claude-code)
- [Claude Code GitHub](https://github.com/anthropics/claude-code)
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [Anthropic API Reference](https://docs.anthropic.com/en/api)

## Version History

| Version | Features |
|---------|----------|
| 1.0 | Initial release with core CLI |
| 1.1 | Added MCP server support |
| 1.2 | Custom slash commands |
| 1.3 | Agent spawning (Task tool) |

## See Also

- [Crush](./crush.md) - Alternative TUI-based agent
- [Caro Integration Guide](./README.md) - Overview of all integrations
