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

### Method 1: Guardian hook (`caro guard`, experimental)

Claude Code runs a `PreToolUse` hook before every tool call. `caro guard`
reads that hook payload from stdin and validates the **agent's own** shell
command with Caro's safety patterns. Install it at **user scope**, so a
project-local `.claude/settings.json` cannot replace it. This is not a security
boundary: an agent that can write to your home directory can still edit
`~/.claude/settings.json`.

```json
// ~/.claude/settings.json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "caro guard --harness claude", "timeout": 10 }]
      }
    ]
  }
}
```

- **Shadow mode** (the default) never changes Claude Code's behavior. Caro
  logs what it *would* have decided and prints a one-line notice for High and
  Critical commands. Review the log with `caro guard report`.
- **Enforce mode** (`"command": "caro guard --harness claude --mode enforce"`,
  or `CARO_GUARD_MODE=enforce`) **denies** Critical commands and **asks** you
  about High-risk ones, even when `Bash(*)` is pre-approved.
- Caro never answers `allow`. In Claude Code, `allow` would skip your own
  permission prompt.

See [spec 011](../../specs/011-harness-guardian/spec.md) for the full contract.

### Method 2: The `caro-shell` skill

The bundled Claude Code skill (`.claude/skills/caro-shell/SKILL.md`) has
Claude ask Caro to *generate* a command from natural language
(`caro --dry-run "<request>"`) and present it for approval.

### Planned: MCP server

`caro mcp serve` with a `validate_command` tool is tracked in #928 and is not
available yet. MCP tools are optional for the model to call. The hook above is
what the harness enforces.

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
- Shell commands are checked by the `caro guard` PreToolUse hook

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

1. **Start in shadow mode and read the report** (`caro guard report`) before
   enforcing. It shows which of your agent's real commands Caro would have
   stopped.
2. **Use user-scope hooks.** A project `.claude/settings.json` sits inside the
   workspace the agent edits; user scope keeps the guard out of it (though not
   out of reach of an agent with home-directory write access).
3. **Bless routine commands with an allowlist** rather than turning the guard
   off. Add them to `[safety] allowlist_patterns` in Caro's config file
   (`caro config show` prints its path; on Linux `~/.config/caro/config.toml`).
   Catastrophic commands (`rm -rf /`, disk wipes, fork bombs) cannot be
   allowlisted.
4. **Use Caro to generate platform-correct commands:** `caro "find files larger than 100MB"`.

## Troubleshooting

### Common Issues

**Issue**: Claude Code can't find Caro
```bash
# Ensure caro is in PATH
which caro
# If not found, add to PATH or use full path
export PATH="$PATH:$HOME/.cargo/bin"
```

**Issue**: The guard never fires
```bash
# Feed the hook a sample payload by hand
echo '{"tool_name":"Bash","tool_input":{"command":"rm -rf /"}}' | caro guard --mode enforce --no-log
# Then check `/hooks` inside Claude Code lists the PreToolUse entry
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
