# Quick Start

This guide covers common usage patterns and examples for cmdai.

## Basic Usage

### Simple Command Generation

```bash
cmdai "list all files in the current directory"
```

This will:
1. Generate a command (e.g., `ls -la`)
2. Display it for review
3. Ask for confirmation
4. Execute if confirmed

### With Auto-Confirmation

Skip the confirmation prompt:

```bash
cmdai --confirm "show disk usage"
```

### Specify Target Shell

Generate commands for a specific shell:

```bash
# For Bash
cmdai --shell bash "list processes"

# For Zsh
cmdai --shell zsh "find large files"

# For Fish
cmdai --shell fish "show network interfaces"

# For PowerShell (Windows)
cmdai --shell powershell "list services"
```

## Output Formats

### JSON Output

For scripting and automation:

```bash
cmdai --output json "list running containers"
```

Output:
```json
{
  "command": "docker ps",
  "safety_level": "safe",
  "shell": "bash"
}
```

### YAML Output

```bash
cmdai --output yaml "show git status"
```

Output:
```yaml
command: git status
safety_level: safe
shell: bash
```

### Plain Output (Default)

```bash
cmdai "compress all logs"
```

## Safety Levels

### Strict Mode (Default)

Blocks dangerous commands automatically:

```bash
cmdai "remove all files in root"
# ❌ Error: Command rejected - critical risk detected
```

### Moderate Mode

Requires confirmation for risky operations:

```bash
cmdai --safety moderate "delete old logs"
# ⚠️  Warning: This command will delete files
# Execute this command? (y/N)
```

### Permissive Mode

Allows more operations (use with caution):

```bash
cmdai --safety permissive "clean temporary files"
```

## Common Examples

### File Operations

```bash
# Find large files
cmdai "find files larger than 100MB"

# List recent files
cmdai "show files modified in last 7 days"

# Count files by type
cmdai "count files by extension in current directory"
```

### System Operations

```bash
# Show system information
cmdai "display system information"

# Check disk space
cmdai "show disk space usage"

# List running processes
cmdai "show processes sorted by memory usage"
```

### Network Operations

```bash
# Check network connectivity
cmdai "test internet connection"

# Show network interfaces
cmdai "list all network interfaces"

# Display active connections
cmdai "show active network connections"
```

### Git Operations

```bash
# Show uncommitted changes
cmdai "show all uncommitted changes in git"

# List recent commits
cmdai "show last 10 git commits"

# Find large files in git history
cmdai "find largest files tracked by git"
```

### Docker Operations

```bash
# List containers
cmdai "show all docker containers"

# Clean up images
cmdai "remove unused docker images"

# Show container logs
cmdai "display logs for nginx container"
```

## Verbose Mode

Enable detailed output for debugging:

```bash
cmdai --verbose "list files"
```

This shows:
- Backend selection
- Model loading time
- Inference time
- Safety check results
- Total execution time

## Configuration File

Use a custom configuration file:

```bash
cmdai --config /path/to/config.toml "your command"
```

## Tips and Tricks

### Be Specific

More specific prompts generate better commands:

```bash
# ❌ Vague
cmdai "find files"

# ✅ Specific
cmdai "find PDF files larger than 10MB modified in last 30 days"
```

### Include Context

Mention the directory or scope:

```bash
cmdai "list all Python files in src/ directory"
```

### Specify Desired Format

Tell cmdai what output format you want:

```bash
cmdai "show disk usage in human-readable format sorted by size"
```

### Combine with Shell Features

```bash
# Use with xargs
cmdai "find old log files" | xargs rm

# Redirect output
cmdai --output json "list files" > files.json

# Pipe to other commands
cmdai "list large files" | grep ".log"
```

## Next Steps

- [Configuration](./configuration.md) - Customize cmdai
- [Safety & Security](./safety.md) - Understand safety features
- [Architecture](../dev-guide/architecture.md) - Learn how it works
