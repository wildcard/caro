# Shai

**OVH's Shell AI Assistant**

Shai (Shell AI) is OVH's AI-powered shell assistant designed for system administrators and DevOps engineers to generate safe shell commands.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | OVH Cloud |
| **Type** | CLI Assistant |
| **Language** | Python |
| **License** | Apache 2.0 |
| **Website** | [ovh.com/shai](https://www.ovh.com) |
| **Repository** | [github.com/ovh/shai](https://github.com/ovh/shai) |

## Installation

### Using pip

```bash
# Install from PyPI
pip install shai

# Or with pipx for isolation
pipx install shai
```

### Using Homebrew (macOS)

```bash
brew install ovh/tap/shai
```

### From Source

```bash
git clone https://github.com/ovh/shai
cd shai
pip install -e .
```

### Verify Installation

```bash
shai --version
shai --help
```

## Configuration

### Initial Setup

```bash
# Configure API key
shai config set api_key "your-api-key"

# Or use environment variable
export SHAI_API_KEY="your-api-key"
```

### Configuration File

Located at `~/.config/shai/config.yaml`:

```yaml
api:
  provider: anthropic  # or openai, ovh
  model: claude-sonnet-4-20250514

shell:
  default: zsh
  posix_mode: true

safety:
  validate_before_execute: true
  dangerous_command_warning: true

integration:
  caro_enabled: true
  caro_path: ~/.cargo/bin/caro
```

## Basic Usage

```bash
# Generate a command
shai "find all Python files modified today"

# Get command explanation
shai explain "awk '{print $2}' file.txt"

# Suggest improvements
shai suggest "for f in *.txt; do cat $f; done"

# Execute with confirmation
shai exec "list large log files"
```

## Key Features

### Shell-Focused
- POSIX shell expertise
- Common utility knowledge (awk, sed, grep, find)
- Pipeline optimization
- One-liner generation

### Safety Features
- Dangerous command detection
- Path validation
- Permission checks
- Dry-run mode

### Sysadmin Focus
- Server management commands
- Log analysis
- Network diagnostics
- Container operations

## Integration with Caro

### Method 1: Pipeline Validation

```bash
# Generate with Shai, validate with Caro
shai "find and delete old logs" | caro --validate

# Or capture and validate
cmd=$(shai --raw "compress backup files")
caro --validate "$cmd" && eval "$cmd"
```

### Method 2: Configuration Integration

Configure Shai to use Caro for validation:

```yaml
# ~/.config/shai/config.yaml
integration:
  caro:
    enabled: true
    path: ~/.cargo/bin/caro
    validate_all: true
    block_on_critical: true
```

### Method 3: Shell Function

Create a combined function:

```bash
# ~/.zshrc or ~/.bashrc
safe_shai() {
    local cmd
    cmd=$(shai --raw "$@")
    echo "Generated: $cmd"

    if caro --validate "$cmd"; then
        read -p "Execute? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            caro --execute "$cmd"
        fi
    else
        echo "Command failed validation"
    fi
}

# Usage
safe_shai "clean up Docker resources"
```

### Method 4: Complementary Usage

Use Shai for sysadmin tasks, Caro for safety:

```bash
# Shai excels at sysadmin-specific commands
shai "check nginx configuration syntax"
# Output: nginx -t

# Caro adds safety validation
caro --validate "nginx -t"
# Output: SAFE - Read-only configuration test
```

## Shai Commands

| Command | Description |
|---------|-------------|
| `shai <prompt>` | Generate command |
| `shai explain <cmd>` | Explain a command |
| `shai suggest <cmd>` | Suggest improvements |
| `shai exec <prompt>` | Generate and execute |
| `shai history` | View command history |
| `shai config` | Manage configuration |

## Sysadmin Examples

### Log Analysis

```bash
shai "find errors in nginx logs from today"
# Output: grep -E "error|warn" /var/log/nginx/error.log | \
#         awk '$4 ~ /'"$(date +%d\\/%b\\/%Y)"'/'
```

### Disk Management

```bash
shai "show disk usage sorted by size"
# Output: du -sh /* 2>/dev/null | sort -hr | head -20
```

### Network Diagnostics

```bash
shai "list all listening ports with process names"
# Output: ss -tlnp
```

### Container Operations

```bash
shai "remove all stopped containers and unused images"
# Output: docker container prune -f && docker image prune -a -f

# Validate with Caro
caro --validate "docker container prune -f && docker image prune -a -f"
# Output: MODERATE - Removes unused Docker resources
```

## Best Practices with Caro

### 1. Defense in Depth

```bash
# Shai generates, Caro validates, both contribute to safety
shai "clean temporary files older than 7 days" | caro --validate
```

### 2. Learning Workflow

```bash
# Use Shai to understand, Caro to verify
shai explain "$(caro --raw 'find large files')"
```

### 3. Production Safety

```yaml
# In production, always validate
# ~/.config/shai/config.yaml
safety:
  production_hosts:
    - "*.prod.*"
    - "prod-*"
  require_caro_validation: true
  require_confirmation: true
```

## Comparison with Caro

| Feature | Shai | Caro |
|---------|------|------|
| Focus | Sysadmin tasks | General safety |
| Validation | Basic | Comprehensive (52+ patterns) |
| Platform Detection | Limited | Extensive |
| Local Inference | No | Yes (MLX, CPU) |
| Risk Assessment | Yes | Detailed (4 levels) |

**Recommendation**: Use together - Shai for sysadmin expertise, Caro for safety validation.

## Troubleshooting

### Common Issues

**Issue**: Shai not connecting to API
```bash
# Check API key
shai config get api_key

# Test connection
shai "hello" --verbose
```

**Issue**: Wrong shell syntax
```bash
# Specify shell explicitly
shai --shell bash "array iteration"
```

**Issue**: Caro integration not working
```yaml
# Verify caro path in config
integration:
  caro:
    path: /usr/local/bin/caro  # Adjust to actual path
```

## Resources

- [Shai Documentation](https://github.com/ovh/shai#readme)
- [OVH Cloud Blog](https://blog.ovhcloud.com)
- [GitHub Repository](https://github.com/ovh/shai)

## See Also

- [Aider](./aider.md) - Git-aware coding assistant
- [Claude Code](./claude-code.md) - Anthropic's agent
- [Caro Integration Guide](./README.md)
