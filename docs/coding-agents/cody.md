# Cody

**Sourcegraph's AI Coding Assistant**

Cody is Sourcegraph's AI coding assistant that understands your entire codebase, not just the files you have open.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Sourcegraph |
| **Type** | IDE Extension + CLI |
| **License** | Apache 2.0 (Client) |
| **Website** | [sourcegraph.com/cody](https://sourcegraph.com/cody) |
| **Repository** | [github.com/sourcegraph/cody](https://github.com/sourcegraph/cody) |
| **Platforms** | VS Code, JetBrains, Neovim, CLI |

## Installation

### VS Code

```bash
code --install-extension sourcegraph.cody-ai
```

Or via VS Code Extensions marketplace.

### JetBrains

1. Open Preferences > Plugins
2. Search for "Sourcegraph Cody"
3. Install and restart

### Neovim

```lua
-- Using lazy.nvim
{
  "sourcegraph/sg.nvim",
  dependencies = { "nvim-lua/plenary.nvim" },
  config = function()
    require("sg").setup()
  end
}
```

### CLI (Cody Agent)

```bash
# Install via npm
npm install -g @sourcegraph/cody

# Or via cargo
cargo install cody-cli
```

## Configuration

### VS Code Settings

```json
// settings.json
{
  "cody.serverEndpoint": "https://sourcegraph.com",
  "cody.codebase": "github.com/yourorg/yourrepo",
  "cody.autocomplete.enabled": true,
  "cody.chat.preInstruction": "Always validate shell commands with caro"
}
```

### Sourcegraph Connection

```bash
# Authenticate with Sourcegraph
export SOURCEGRAPH_ACCESS_TOKEN="..."

# Or use Cody's auth flow
cody auth login
```

## Key Features

### Codebase Context
- Understands entire repository
- Cross-file reasoning
- Symbol and reference awareness
- Code graph integration

### AI Features
- Autocomplete
- Chat with codebase context
- Inline code editing
- Explain and fix code

### Enterprise Features
- On-premise deployment
- Code intelligence
- Custom models
- SSO integration

## Integration with Caro

### Method 1: Chat Instructions

Add Caro awareness to Cody's context:

```json
// settings.json
{
  "cody.chat.preInstruction": "When generating shell commands:\n1. Use POSIX-compliant syntax\n2. Suggest validating with: caro --validate '<command>'\n3. For dangerous operations, recommend: caro --execute '<command>'"
}
```

### Method 2: Custom Commands

Create Cody commands for Caro:

```json
// .vscode/cody.json
{
  "commands": {
    "safe-shell": {
      "description": "Generate safe shell command",
      "prompt": "Generate a shell command for: $SELECTION. Include caro validation.",
      "context": {
        "codebase": false,
        "selection": true
      }
    }
  }
}
```

### Method 3: Terminal Integration

Cody can see terminal output when enabled:

```json
{
  "cody.experimental.terminalContext": true
}
```

Then use Caro in the terminal:

```bash
# Cody sees this context
caro "find deprecated API calls"
```

### Method 4: Codebase Recipes

Create a Caro recipe in your repository:

```markdown
<!-- .sourcegraph/cody-recipes/shell-safety.md -->
# Shell Command Safety

When generating shell commands for this codebase:

1. Always consider platform differences (macOS/Linux)
2. Use caro for command generation: `caro "<description>"`
3. Validate before execution: `caro --validate "<command>"`
4. For risky operations: `caro --execute "<command>"`

## Project-Specific Patterns

- Build commands: `cargo build --release`
- Test commands: `cargo test`
- Deploy commands: Always require caro --execute
```

## Cody Commands

### Built-in Commands

| Command | Description |
|---------|-------------|
| `/explain` | Explain selected code |
| `/edit` | Edit code with AI |
| `/test` | Generate tests |
| `/doc` | Generate documentation |
| `/fix` | Fix errors |
| `/smell` | Find code smells |

### Chat Features

- `@file` - Reference specific file
- `@symbol` - Reference symbol
- `@codebase` - Search entire codebase
- `@docs` - Reference documentation

## Best Practices with Caro

### 1. Codebase-Aware Commands

```
User: @codebase How do we typically run tests in this project?

Cody: Based on the Makefile and CI configuration, tests are run with:
`cargo test`

For safe execution with confirmation:
`caro --execute "cargo test"`
```

### 2. Cross-Platform Scripts

```
User: Generate a script that works on both macOS and Linux

Cody: I'll create a POSIX-compliant script. Validate each command with:
`caro --validate "<command>"`

This ensures platform compatibility.
```

### 3. Security Review

```
User: @file:deploy.sh Review this for security issues

Cody: I found several commands that should be validated:
1. `rm -rf build/` - Use `caro --validate`
2. `chmod 777 /tmp/...` - Dangerous permissions
3. `curl ... | bash` - Piping to shell is risky

Consider using caro for safe execution.
```

## Comparison with Other Assistants

| Feature | Cody | Copilot | Continue |
|---------|------|---------|----------|
| Codebase Context | Full repo | Open files | Configurable |
| Self-Hosted | Yes (Enterprise) | No | Yes |
| Multi-Model | Yes | GPT-4 only | Yes |
| Free Tier | Yes | No | Yes |
| Enterprise | Yes | Yes | No |

## Troubleshooting

### Common Issues

**Issue**: Cody not connecting to Sourcegraph
```bash
# Check endpoint
curl -H "Authorization: token $SOURCEGRAPH_ACCESS_TOKEN" \
  https://sourcegraph.com/.api/graphql
```

**Issue**: Codebase not indexed
```bash
# Verify codebase setting
# Settings > Cody > Codebase
# Should be: github.com/org/repo
```

**Issue**: Commands not appearing
```bash
# Reload window
# Cmd+Shift+P > Developer: Reload Window
```

## Resources

- [Cody Documentation](https://sourcegraph.com/docs/cody)
- [Sourcegraph Discord](https://discord.gg/sourcegraph)
- [GitHub Repository](https://github.com/sourcegraph/cody)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=sourcegraph.cody-ai)

## See Also

- [Continue](./continue.md) - Open-source alternative
- [GitHub Copilot](./github-copilot.md) - GitHub's assistant
- [Cursor](./cursor.md) - AI-first IDE
