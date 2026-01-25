# Augie

**Autonomous Coding Agent**

Augie is an autonomous coding agent from aug.dev that can complete complex programming tasks with minimal human intervention.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | aug.dev |
| **Type** | CLI Agent |
| **Language** | Rust |
| **License** | Apache 2.0 |
| **Website** | [aug.dev](https://aug.dev) |
| **Repository** | [github.com/augdev/augie](https://github.com/augdev/augie) |

## Installation

### Using Cargo

```bash
# Install from crates.io
cargo install augie

# Or from source
git clone https://github.com/augdev/augie
cd augie
cargo install --path .
```

### Using Homebrew (macOS)

```bash
brew install augdev/tap/augie
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/augdev/augie/releases).

### Verify Installation

```bash
augie --version
augie --help
```

## Configuration

### Project Configuration (.augie/)

```
.augie/
├── config.toml     # Project settings
├── prompts/        # Custom prompts
└── tools/          # Custom tool definitions
```

Example `config.toml`:

```toml
[model]
provider = "anthropic"
name = "claude-sonnet-4-20250514"

[safety]
validate_commands = true
dangerous_patterns = ["rm -rf /", "dd if=/dev/zero"]

[tools.caro]
command = "caro"
description = "Safe shell command generation"
```

### Environment Variables

```bash
export ANTHROPIC_API_KEY="..."    # For Claude models
export OPENAI_API_KEY="..."       # For GPT models
export AUGIE_WORKSPACE="$HOME/projects"
```

## Basic Usage

```bash
# Start autonomous task
augie "implement user authentication"

# With specific scope
augie --files "src/auth/*.rs" "add password hashing"

# Dry-run mode (no changes)
augie --dry-run "refactor this module"

# Interactive mode
augie -i "help me debug this issue"
```

## Key Features

### Autonomous Operation
- Multi-step task completion
- Automatic file discovery
- Dependency analysis
- Self-correction on errors

### Tool Integration
- Shell command execution
- File system operations
- Git operations
- Test running

### Safety Features
- Dangerous command detection
- Rollback capability
- Confirmation prompts
- Audit logging

## Integration with Caro

### Method 1: Tool Configuration

Add Caro as a tool in Augie:

```toml
# .augie/config.toml
[tools.caro]
command = "caro"
args = ["--json"]
description = "Generate and validate shell commands safely"
when_to_use = "When generating shell commands that interact with the file system or system resources"
```

### Method 2: Shell Integration

Configure Augie to use Caro for shell commands:

```toml
# .augie/config.toml
[shell]
validator = "caro --validate"
executor = "caro --execute"
```

### Method 3: Custom Tool Definition

Create a Caro tool wrapper:

```toml
# .augie/tools/caro.toml
[tool]
name = "safe_shell"
description = "Generate safe, platform-aware shell commands"

[tool.command]
binary = "caro"

[tool.parameters]
prompt = { type = "string", description = "Natural language command description" }
validate_only = { type = "boolean", default = false, description = "Only validate, don't execute" }

[tool.examples]
- prompt = "find all large files"
  output = "find . -type f -size +100M"
```

### Method 4: Prompt Integration

Add Caro to system prompts:

```markdown
<!-- .augie/prompts/system.md -->
When executing shell commands:
1. Generate the command based on intent
2. ALWAYS validate using: `caro --validate "<command>"`
3. Only execute if Caro reports SAFE or MODERATE risk
4. For HIGH/CRITICAL risk, ask user confirmation

Available tool: `caro "<description>"` - generates safe POSIX commands
```

## Augie Commands

| Command | Description |
|---------|-------------|
| `augie <task>` | Start autonomous task |
| `augie -i` | Interactive mode |
| `augie --dry-run` | Preview without changes |
| `augie --rollback` | Undo last task |
| `augie status` | Show current state |
| `augie log` | View audit log |

## Autonomous Workflow with Caro

### Example: Cleanup Task

```bash
augie "clean up old build artifacts and temporary files"
```

Augie's internal workflow:
1. **Analyze** - Identifies build directories, temp files
2. **Plan** - Creates list of cleanup commands
3. **Validate** - Each command checked with Caro
4. **Confirm** - Shows plan to user
5. **Execute** - Runs validated commands

### Example: Safe Deployment

```bash
augie "deploy to staging environment"
```

Caro integration:
```
Step 1: Build - cargo build --release ✓ SAFE
Step 2: Test - cargo test ✓ SAFE
Step 3: Package - tar -czf deploy.tar.gz target/release/* ✓ SAFE
Step 4: Deploy - scp deploy.tar.gz staging:/ ✓ MODERATE (confirm?)
Step 5: Restart - ssh staging 'systemctl restart app' ✓ MODERATE (confirm?)
```

## Best Practices with Caro

### 1. Safety-First Configuration

```toml
# .augie/config.toml
[safety]
# Require Caro validation for all shell commands
require_validation = true
validator = "caro --validate --json"

# Block execution of high-risk commands without confirmation
auto_confirm_risk_levels = ["safe"]
require_confirm_risk_levels = ["moderate", "high"]
block_risk_levels = ["critical"]
```

### 2. Audit Trail

```toml
# Log all Caro validations
[logging]
shell_commands = true
validation_results = true
log_path = ".augie/logs/commands.log"
```

### 3. Platform-Aware Tasks

```bash
# Augie automatically uses Caro for platform detection
augie "create a script that works on both macOS and Linux"
# Caro ensures POSIX compliance
```

## Troubleshooting

### Common Issues

**Issue**: Caro tool not recognized
```bash
# Check Caro is in PATH
which caro

# Add to Augie's tool search path
export AUGIE_TOOL_PATH="$HOME/.cargo/bin:$AUGIE_TOOL_PATH"
```

**Issue**: Validation timeouts
```toml
# Increase tool timeout
[tools.caro]
timeout = 30  # seconds
```

**Issue**: Too many confirmations
```toml
# Adjust risk thresholds
[safety]
auto_confirm_risk_levels = ["safe", "moderate"]
```

## Comparison with Other Agents

| Feature | Augie | Claude Code | Aider |
|---------|-------|-------------|-------|
| Autonomy | High | Medium | Low |
| Tool Use | Extensive | Extensive | Limited |
| Rollback | Built-in | Manual | Git-based |
| Language | Rust | TypeScript | Python |

## Resources

- [Augie Documentation](https://aug.dev/docs)
- [GitHub Repository](https://github.com/augdev/augie)
- [Examples](https://github.com/augdev/augie/tree/main/examples)

## See Also

- [Claude Code](./claude-code.md) - Anthropic's agent
- [Aider](./aider.md) - Git-aware assistant
- [Caro Integration Guide](./README.md)
