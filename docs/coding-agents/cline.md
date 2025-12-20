# Cline

**Autonomous Coding Agent for VS Code**

Cline (formerly Claude Dev) is a VS Code extension that provides an autonomous AI coding agent with file editing, terminal access, and browser capabilities.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Cline Team |
| **Type** | VS Code Extension |
| **License** | Apache 2.0 |
| **Repository** | [github.com/cline/cline](https://github.com/cline/cline) |
| **Marketplace** | [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=saoudrizwan.claude-dev) |

## Installation

### VS Code Marketplace

```bash
code --install-extension saoudrizwan.claude-dev
```

Or search "Cline" in VS Code Extensions.

### Configuration

After installation:
1. Open Cline panel (Cmd+Shift+P > "Cline: Open")
2. Configure API key (Anthropic, OpenAI, or other)
3. Set preferences

## Configuration

### Extension Settings

```json
// settings.json
{
  "cline.apiProvider": "anthropic",
  "cline.apiKey": "...",
  "cline.model": "claude-sonnet-4-20250514",
  "cline.customInstructions": "Always validate shell commands with caro",
  "cline.alwaysAllowWrite": false,
  "cline.alwaysAllowExecute": false
}
```

### Custom Instructions

Add project-specific instructions:

```markdown
<!-- .cline/instructions.md -->
# Project Instructions

## Shell Command Safety
- Use `caro` for command generation
- Validate before execution: `caro --validate "<cmd>"`
- For risky operations: `caro --execute "<cmd>"`

## This Project
- Rust CLI tool
- Uses cargo for building/testing
- Safety-first approach
```

## Key Features

### Autonomous Operation
- Multi-step task completion
- Automatic file editing
- Terminal command execution
- Error detection and recovery

### Tool Capabilities
- File read/write
- Terminal commands
- Browser automation
- Image analysis

### Safety Features
- Human approval for changes
- Diff preview before writes
- Command confirmation
- Checkpoint/resume

## Integration with Caro

### Method 1: Custom Instructions

Configure Cline to use Caro:

```json
// settings.json
{
  "cline.customInstructions": "When executing shell commands:\n1. Generate safe POSIX-compliant commands\n2. ALWAYS validate with: caro --validate \"<command>\"\n3. For destructive operations, use: caro --execute \"<command>\"\n4. Show risk level to user before proceeding"
}
```

### Method 2: Project Instructions

Create `.cline/instructions.md`:

```markdown
# Caro Integration

This project uses caro for safe shell command execution.

## Commands
- Generate: `caro "<natural language>"`
- Validate: `caro --validate "<command>"`
- Execute: `caro --execute "<command>"`

## Workflow
1. When a shell command is needed, generate with caro
2. Show the command and risk level to user
3. Get approval before execution
4. Execute with caro --execute for safety

## Never
- Run rm -rf without caro validation
- Execute curl | bash patterns
- Use chmod 777 or similar risky permissions
```

### Method 3: Tool Integration

Cline can be configured to use external tools:

```json
// .cline/tools.json
{
  "tools": [
    {
      "name": "caro",
      "description": "Safe shell command generation and validation",
      "command": "caro",
      "args": ["--json"]
    },
    {
      "name": "caro_validate",
      "description": "Validate a shell command for safety",
      "command": "caro",
      "args": ["--validate", "--json"]
    }
  ]
}
```

### Method 4: Workflow Automation

Create Caro-aware workflows:

```yaml
# .cline/workflows/safe-deploy.yaml
name: Safe Deployment
steps:
  - name: Build
    command: caro --execute "cargo build --release"
  - name: Test
    command: caro --execute "cargo test"
  - name: Deploy
    command: caro --execute "deploy.sh"
    confirm: true
```

## Cline Commands

### Chat Commands

| Command | Description |
|---------|-------------|
| Type message | Ask Cline to do something |
| Attach image | Include screenshot for context |
| Attach file | Include file for context |

### Panel Controls

| Button | Description |
|--------|-------------|
| Approve | Accept proposed change |
| Reject | Decline change |
| Stop | Cancel current task |
| Resume | Continue from checkpoint |

## Best Practices with Caro

### 1. Safety-First Workflow

```
User: Delete all temporary files from this project

Cline: I'll help you clean up temporary files safely.

First, let me generate a safe command:
$ caro "delete temporary files"

Generated: find . -name "*.tmp" -type f -delete
Risk level: MODERATE

Validating...
$ caro --validate "find . -name '*.tmp' -type f -delete"
Result: MODERATE - Recursive file deletion

Shall I proceed with caro --execute?
```

### 2. Autonomous with Guardrails

Configure for autonomous operation with safety:

```json
{
  "cline.alwaysAllowExecute": false,
  "cline.customInstructions": "Use caro --execute for all shell commands. Never bypass validation."
}
```

### 3. Multi-Step Tasks

```
User: Set up CI/CD for this Rust project

Cline: I'll set up CI/CD with safe commands.

Step 1: Create GitHub Actions workflow
[creates .github/workflows/ci.yml]

Step 2: Validate CI commands
$ caro --validate "cargo build --release"
$ caro --validate "cargo test"
$ caro --validate "cargo clippy"

All commands validated as SAFE.

Step 3: Test locally
$ caro --execute "cargo build && cargo test"
```

## Comparison with Other Agents

| Feature | Cline | Claude Code | Cursor |
|---------|-------|-------------|--------|
| Interface | VS Code Panel | CLI | IDE |
| Autonomy | High | Medium | Low |
| File Editing | Visual diff | Direct | Visual diff |
| Browser | Yes | No | No |
| Open Source | Yes | No | No |

## Troubleshooting

### Common Issues

**Issue**: API key not working
```json
// Verify in settings
{
  "cline.apiProvider": "anthropic",
  "cline.apiKey": "sk-ant-..."
}
```

**Issue**: Caro commands failing
```bash
# Ensure caro is in PATH
which caro

# Add to VS Code terminal PATH
{
  "terminal.integrated.env.osx": {
    "PATH": "${env:HOME}/.cargo/bin:${env:PATH}"
  }
}
```

**Issue**: Too many approvals needed
```json
// Adjust auto-approve settings
{
  "cline.alwaysAllowRead": true,
  "cline.alwaysAllowWrite": false,  // Keep false for safety
  "cline.alwaysAllowExecute": false  // Keep false for safety
}
```

## Resources

- [Cline GitHub](https://github.com/cline/cline)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=saoudrizwan.claude-dev)
- [Discord Community](https://discord.gg/cline)

## See Also

- [Roo Code](./roo-code.md) - Similar VS Code agent
- [Continue](./continue.md) - Open-source extension
- [Claude Code](./claude-code.md) - CLI alternative
