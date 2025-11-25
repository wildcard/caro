# Safety & Security

cmdai is built with a **safety-first philosophy**. This guide explains how cmdai protects you from dangerous commands.

## Safety Philosophy

cmdai operates on these principles:

1. **No command executes without user review** (except with `--confirm` flag)
2. **All commands are validated** before execution
3. **Dangerous patterns are detected** and blocked
4. **Risk levels are clearly communicated** to users
5. **Safety can be adjusted** based on user preference

## Safety Checks

### Implemented Protections

cmdai includes comprehensive safety validation:

| Category | Examples | Status |
|----------|----------|--------|
| System Destruction | `rm -rf /`, `rm -rf ~` | ✅ Blocked |
| Fork Bombs | `:(){:\|:&};:` | ✅ Blocked |
| Disk Operations | `mkfs`, `dd if=/dev/zero` | ✅ Blocked |
| Privilege Escalation | `sudo su`, `chmod 777 /` | ✅ Detected |
| Critical Paths | Operations on `/bin`, `/usr`, `/etc` | ✅ Protected |

### How Safety Validation Works

```
┌──────────────────────┐
│ Natural Language     │  "delete old files"
│ Prompt               │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ LLM Generation       │  find . -name "*.tmp" -mtime +30 -delete
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Pattern Matching     │  Checking against dangerous patterns...
│ • System destruction │  ✅ Pass
│ • Fork bombs         │  ✅ Pass
│ • Critical paths     │  ✅ Pass
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Risk Assessment      │  Risk Level: 🔶 High (deletion)
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ User Confirmation    │  Execute? (y/N) ← YOU DECIDE
└──────────┬───────────┘
           │
           ▼ (if approved)
┌──────────────────────┐
│ Execution            │  Command runs safely
└──────────────────────┘
```

> **📖 Learn More:** See [Safety Validation](../technical/safety-validation.md) for technical implementation details.

## Risk Levels

cmdai categorizes commands into four risk levels:

| Risk Level | Indicator | Typical Commands | Default Behavior |
|------------|-----------|------------------|------------------|
| **Safe** | ✅ Green | `ls`, `cat`, `grep` | Confirm to execute |
| **Moderate** | ⚠️ Yellow | `mkdir`, `cp`, `git commit` | Confirm with caution |
| **High** | 🔶 Orange | `rm`, `mv`, `find -delete` | Confirm with warning |
| **Critical** | 🛑 Red | `rm -rf /`, `mkfs`, fork bombs | Blocked in strict mode |

### Safe (Green)

Normal operations that pose no risk:

```bash
$ cmdai "list files"
Generated: ls -la
Safety: ✅ Safe
Execute? (y/N)
```

Examples:
- File listing (`ls`, `find`)
- Information display (`cat`, `head`, `tail`)
- Status checks (`git status`, `docker ps`)

### Moderate (Yellow)

Operations that modify data but are localized:

```bash
$ cmdai "create a backup directory"
Generated: mkdir backup
Safety: ⚠️  Moderate
Execute? (y/N)
```

Examples:
- File creation (`touch`, `mkdir`)
- File copying (`cp`)
- Git operations (`git commit`)

### High (Orange)

Operations with significant impact:

```bash
$ cmdai "delete log files older than 30 days"
Generated: find . -name "*.log" -mtime +30 -delete
Safety: 🔶 High Risk
Execute? (y/N)
```

Examples:
- File deletion (localized)
- System modifications
- Network operations

### Critical (Red)

Potentially destructive operations:

```bash
$ cmdai "remove all files in root"
Safety: 🛑 CRITICAL RISK - Command blocked
Reason: System destruction pattern detected
```

Examples:
- System-wide deletion
- Disk formatting
- Recursive root operations

## Safety Levels

cmdai supports three safety modes:

### Strict (Default)

Maximum protection:

- **Safe**: Execute with confirmation
- **Moderate**: Require confirmation
- **High**: Require confirmation with warning
- **Critical**: Blocked completely

```bash
cmdai --safety strict "your command"
```

### Moderate

Balanced protection:

- **Safe**: Execute with confirmation
- **Moderate**: Execute with confirmation
- **High**: Require confirmation with warning
- **Critical**: Require explicit confirmation

```bash
cmdai --safety moderate "your command"
```

### Permissive

Minimal protection (use with caution):

- **Safe**: Execute with confirmation
- **Moderate**: Execute with confirmation
- **High**: Execute with confirmation
- **Critical**: Require confirmation

```bash
cmdai --safety permissive "your command"
```

## Configuration

Configure safety settings in `~/.config/cmdai/config.toml`:

```toml
[safety]
enabled = true
level = "moderate"  # strict, moderate, or permissive
require_confirmation = true

# Add custom dangerous patterns
custom_patterns = [
    "curl.*bash",           # Piping curl to bash
    "wget.*sh",             # Downloading and executing scripts
    "chmod\\s+777",         # Overly permissive permissions
]

# Protected paths
protected_paths = [
    "/",
    "/bin",
    "/usr",
    "/etc",
    "/var",
    "/boot",
]
```

## Dangerous Pattern Detection

### Built-in Patterns

cmdai detects these dangerous patterns:

**System Destruction**:
- `rm -rf /`
- `rm -rf ~`
- `rm -rf /*`
- `rm -rf $HOME`

**Disk Operations**:
- `mkfs`
- `dd if=/dev/zero`
- `dd if=/dev/random`

**Fork Bombs**:
- `:(){:|:&};:`
- `$0 & $0 &`

**Privilege Escalation**:
- `sudo su`
- `chmod 777 /`
- `chown root:root /`

### Custom Patterns

Add your own patterns to catch organization-specific risks:

```toml
[safety.custom_patterns]
patterns = [
    "kubectl delete.*--all",
    "docker rm.*-f.*-v",
    "npm publish --force",
]
```

## Security Best Practices

### 1. Review Before Executing

Always review the generated command before execution:

```bash
$ cmdai "delete old files"
Generated: find . -name "*.tmp" -mtime +7 -delete

# Check:
# ✅ Correct file pattern (*.tmp)
# ✅ Correct time threshold (+7 days)
# ✅ Correct directory scope (.)
```

### 2. Use Appropriate Safety Levels

- **Development/Testing**: Use `strict` mode
- **Production Scripts**: Use `strict` or `moderate`
- **Personal Exploration**: Use `moderate`
- **Permissive**: Only with full understanding

### 3. Verify Critical Operations

For critical operations, verify manually:

```bash
# Generate the command
cmdai "delete all Docker volumes"

# Review the command
# Verify with dry-run if available
docker volume ls

# Execute manually if needed
```

### 4. Keep cmdai Updated

Security improvements are released regularly:

```bash
# Update cmdai
cargo install --path . --force

# Check version
cmdai --version
```

### 5. Report Security Issues

If you discover a security vulnerability, report it privately:

- Use [GitHub Security Advisories](https://github.com/wildcard/cmdai/security)
- **Do NOT** create public issues for security vulnerabilities
- See [Security Policy](../reference/security.md) for details

## Limitations

### What cmdai Cannot Prevent

1. **Malicious LLM Models**: If using untrusted models, they could generate harmful commands
2. **Logic Errors**: Valid-looking commands that do the wrong thing
3. **User Override**: Users can bypass safety with `--safety permissive`
4. **Novel Attack Patterns**: Unknown dangerous patterns

### Defense in Depth

cmdai is **one layer** of protection. Also use:

- File system permissions
- Backups
- Version control
- Sandboxed environments
- Read-only mounts

## Emergency Response

If cmdai generated a dangerous command:

1. **Do NOT execute it**
2. **Report the issue**: [GitHub Issues](https://github.com/wildcard/cmdai/issues)
3. **Include**:
   - The prompt you used
   - The generated command
   - Your configuration
   - cmdai version

## Next Steps

**User Guides:**
- [Configuration](./configuration.md) - Configure custom safety settings
- [Getting Started](./getting-started.md) - Installation and setup
- [Quick Start](./quick-start.md) - Usage examples

**Technical Details:**
- [Safety Validation](../technical/safety-validation.md) - Implementation deep dive
- [Architecture](../dev-guide/architecture.md) - System design
- [Security Policy](../reference/security.md) - Full security policy

**Tutorials:**
- [Tutorial: Working with Files](../tutorial/working-with-files.md) - Safe file operations
- [Tutorial: System Operations](../tutorial/system-operations.md) - Safe system commands

---

## See Also

**Configuration:**
- [Configuration Guide](./configuration.md) - Customize safety levels and patterns
- Custom pattern documentation for organization-specific risks

**Security:**
- [Security Policy](../reference/security.md) - Vulnerability reporting
- [Code of Conduct](../reference/code-of-conduct.md) - Community guidelines

**Technical:**
- [Safety Validation Deep Dive](../technical/safety-validation.md) - Pattern matching algorithms
- [Backend Development](../dev-guide/backends.md) - Safe command generation
