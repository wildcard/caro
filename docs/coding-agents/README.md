# Caro Coding Agent Companion Guide

Caro is designed as a **coding agent companion** - a tool that works alongside any AI coding assistant to provide safe, platform-aware shell command generation. This guide documents all known coding agents that Caro can integrate with.

## What is a Coding Agent Companion?

Caro is unique in that it's not just another coding agent - it's a **companion sub-agent** designed to work *with* any coding AI. When your primary coding agent (Claude, Cursor, Crush, etc.) needs to generate a shell command, Caro provides:

- **Safety validation** - 52+ dangerous command patterns detected
- **Platform awareness** - OS, shell, and command availability detection
- **POSIX compliance** - Portable commands that work across systems
- **Risk assessment** - Color-coded risk levels with confirmation workflows

## Supported Coding Agents

Caro integrates seamlessly with these coding agents:

| Agent | Company | Type | Caro Integration |
|-------|---------|------|------------------|
| [Claude Code](./claude-code.md) | Anthropic | CLI | Native MCP support |
| [Crush](./crush.md) | Charm.land | CLI | Direct integration |
| [Cursor](./cursor.md) | Cursor | IDE | Extension compatible |
| [Codex CLI](./codex.md) | OpenAI | CLI | Prompt compatible |
| [Augie](./augie.md) | aug.dev | CLI | Shell integration |
| [Shai](./shai.md) | OVH | CLI | Unix-native |
| [Aider](./aider.md) | aider-ai | CLI | Shell integration |
| [Continue](./continue.md) | Continue | IDE Extension | Extension compatible |
| [Cody](./cody.md) | Sourcegraph | IDE/CLI | Extension compatible |
| [GitHub Copilot](./github-copilot.md) | GitHub | IDE/CLI | Extension compatible |
| [Windsurf](./windsurf.md) | Codeium | IDE | Extension compatible |
| [Void](./void.md) | void.dev | IDE | Extension compatible |
| [Zed AI](./zed-ai.md) | Zed | IDE | Native integration |
| [Cline](./cline.md) | Community | Extension | VS Code compatible |
| [Roo Code](./roo-code.md) | Community | Extension | VS Code compatible |

## Quick Integration Matrix

### CLI-Based Agents (Direct Shell Integration)

```bash
# Claude Code - Use caro as a shell command tool
claude "generate a command to..." # Claude can call caro internally

# Crush - Configure in .crush.json
crush "find large files" # Crush can pipe to caro for validation

# Codex CLI - Shell alias integration
codex "list files" | caro --validate

# Augie - Direct shell integration
augie exec "$(caro 'compress images')"

# Shai - Unix pipeline native
shai suggest | caro --validate
```

### IDE-Based Agents (Extension Integration)

IDE agents can integrate with Caro through:
1. Terminal commands within the IDE
2. Custom tasks/scripts
3. MCP server connections (where supported)

## Agent Categories

### Terminal-Native Agents
Agents designed for command-line use:
- **Claude Code** - Anthropic's official CLI
- **Crush** - Charm.land's TUI coding agent
- **Codex CLI** - OpenAI's terminal agent
- **Augie** - aug.dev's autonomous agent
- **Shai** - OVH's shell assistant
- **Aider** - Git-aware coding assistant

### IDE-Integrated Agents
Agents embedded in development environments:
- **Cursor** - AI-first IDE (VS Code fork)
- **Continue** - Open-source IDE extension
- **Cody** - Sourcegraph's AI assistant
- **GitHub Copilot** - GitHub's coding AI
- **Windsurf** - Codeium's AI IDE
- **Void** - Open-source AI IDE
- **Zed AI** - Native Zed integration

### VS Code Extensions
Standalone extensions for VS Code:
- **Cline** - Autonomous coding agent
- **Roo Code** - Community coding agent
- **Continue** - Multi-model extension

## Installation Quick Reference

| Agent | Installation |
|-------|-------------|
| Claude Code | `npm install -g @anthropic-ai/claude-code` |
| Crush | `brew install charmbracelet/tap/crush` |
| Cursor | Download from [cursor.sh](https://cursor.sh) |
| Codex CLI | `npm install -g @openai/codex` |
| Augie | `cargo install augie` |
| Shai | `pip install shai` |
| Aider | `pip install aider-chat` |
| Continue | VS Code/JetBrains Extension |
| Cody | VS Code/JetBrains Extension |
| Copilot | VS Code/JetBrains Extension |
| Windsurf | Download from [codeium.com](https://codeium.com/windsurf) |
| Cline | VS Code Extension |

## How Caro Enhances Each Agent

### Safety Layer
All agents benefit from Caro's safety validation:
```bash
# Any agent can validate commands through Caro
caro --validate "rm -rf /"
# Output: BLOCKED - Critical risk pattern detected
```

### Platform Intelligence
Caro provides platform-specific command generation:
```bash
# Detects BSD vs GNU, macOS vs Linux
caro "sort files by size"
# macOS: ls -lhS
# Linux:  ls -lhS --color=auto
```

### Execution Safety
Safe command execution with confirmation:
```bash
caro --execute "find and delete temp files"
# Shows command, risk level, requires confirmation
```

## Configuration Files

Caro recognizes and respects configuration from other agents:

| File | Agent | Purpose |
|------|-------|---------|
| `.claude/` | Claude Code | Claude-specific settings |
| `.crush.json` | Crush | Crush configuration |
| `.cursor/` | Cursor | Cursor settings |
| `.codex/` | Codex | Codex prompts |
| `.continue/` | Continue | Continue config |
| `.aider/` | Aider | Aider settings |

## Agent Discovery Pipeline

Caro maintains awareness of the coding agent ecosystem through:

1. **Community submissions** - PRs to add new agents
2. **Automated monitoring** - Tracking major releases
3. **Integration testing** - Verifying compatibility

See [AGENT_DISCOVERY.md](./AGENT_DISCOVERY.md) for contributing new agents.

## Choosing the Right Agent

### For Terminal Power Users
- **Claude Code** - Best for complex multi-file operations
- **Crush** - Best for TUI experience with LSP support
- **Aider** - Best for git-aware pair programming

### For IDE Users
- **Cursor** - Best for VS Code users wanting full AI integration
- **Copilot** - Best for inline code completion
- **Continue** - Best for open-source flexibility

### For Specific Use Cases
- **Shai** - Best for sysadmin tasks
- **Cody** - Best for large codebase understanding
- **Augie** - Best for autonomous task completion

## Next Steps

- Read individual agent guides for detailed integration instructions
- See [AGENT_DISCOVERY.md](./AGENT_DISCOVERY.md) for adding new agents
- Check [../MACOS_SETUP.md](../MACOS_SETUP.md) for platform-specific setup
