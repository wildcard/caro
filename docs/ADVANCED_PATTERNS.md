# Advanced Tool Use Patterns for Caro

This guide covers advanced usage patterns, safety best practices, and common pitfalls when using caro to generate shell commands.

## Table of Contents

- [Safety First](#safety-first)
- [Pattern Library](#pattern-library)
  - [File Operations](#file-operations)
  - [System Administration](#system-administration)
  - [Text Processing](#text-processing)
  - [Network Operations](#network-operations)
- [Common Pitfalls](#common-pitfalls)
- [Best Practices](#best-practices)

## Safety First

### Understanding Risk Levels

Caro classifies commands into three risk levels:

- **Safe**: Read-only operations with no side effects (e.g., `ls`, `cat`, `grep`)
- **Moderate**: Operations that modify data but are reversible (e.g., `cp`, `mv`, `mkdir`)
- **Dangerous**: Irreversible operations or system-wide changes (e.g., `rm -rf`, `dd`, `chmod -R`)

Configure safety level in `~/.config/caro/config.toml`:

```toml
safety_level = "moderate"  # strict | moderate | permissive
```

### Using --dry-run

Always test generated commands with `--dry-run` before execution:

```bash
# Generate and review without executing
caro --dry-run "delete all log files older than 30 days"

# Review the command, then execute manually if safe
find /var/log -name "*.log" -mtime +30 -delete
```

### Confirmation Prompts

Dangerous commands require explicit confirmation:

```bash
$ caro "remove all temporary files"
⚠️  This command is classified as DANGEROUS
Command: find /tmp -type f -delete

Proceed? (y/N): _
```

Override with `-y` flag (use with caution):

```bash
caro -y "clear system cache"  # Auto-confirms dangerous commands
```

## Pattern Library

### File Operations

#### Safe Recursive Operations

**Finding files without executing actions:**

```bash
# Good: Find first, review, then act
caro "find all Python files modified in the last week"
# Generates: find . -name "*.py" -mtime -7

# Then pipe to verify before deleting:
find . -name "*.py" -mtime -7 | xargs ls -lh
```

**Creating backups before modifications:**

```bash
# Always backup before bulk operations
caro "copy all markdown files to backup directory"
# Generates: find . -name "*.md" -exec cp {} ./backup/ \;

# Or use rsync for better safety
caro "sync documents folder to backup with preserve"
# Generates: rsync -av --progress Documents/ backup/Documents/
```

#### Handling Special Characters

**Files with spaces or special characters:**

```bash
# Caro automatically generates proper quoting
caro "find files with spaces in names"
# Generates: find . -name "* *" -type f

# For processing, use -print0 with xargs -0
caro "safely delete files with spaces in /tmp"
# Generates: find /tmp -name "* *" -type f -print0 | xargs -0 rm
```

#### Backup Strategies

**Incremental backups with timestamps:**

```bash
caro "create timestamped backup of config directory"
# Generates: tar -czf "config-backup-$(date +%Y%m%d-%H%M%S).tar.gz" ~/.config/
```

**Differential backups:**

```bash
caro "sync only changed files to backup using rsync"
# Generates: rsync -av --update --progress ~/Documents/ /backup/Documents/
```

### System Administration

#### Privilege Escalation Patterns

**Reviewing before sudo:**

```bash
# Generate command first, review, then sudo manually
caro --dry-run "install nginx"
# Review: apt-get install nginx
# Then: sudo apt-get install nginx
```

**Safe service management:**

```bash
# Check status before restarting
caro "check nginx status"
# Generates: systemctl status nginx

# Reload config instead of restart when possible
caro "reload nginx configuration"
# Generates: sudo systemctl reload nginx  # Safer than restart
```

#### Log Analysis

**Efficient log parsing:**

```bash
# Find errors in last hour
caro "show nginx errors from the last hour"
# Generates: journalctl -u nginx --since "1 hour ago" | grep -i error

# Analyze patterns with awk
caro "count requests by status code in access log"
# Generates: awk '{print $9}' /var/log/nginx/access.log | sort | uniq -c | sort -rn
```

**Monitoring system resources:**

```bash
# Real-time CPU usage
caro "show top 10 processes by CPU usage"
# Generates: ps aux --sort=-%cpu | head -11

# Disk space by directory
caro "show disk usage by directory sorted by size"
# Generates: du -h --max-depth=1 | sort -hr
```

### Text Processing

#### Stream Processing Patterns

**Using pipes effectively:**

```bash
# Multi-stage filtering
caro "find lines with ERROR, exclude DEBUG, show last 20"
# Generates: grep ERROR app.log | grep -v DEBUG | tail -20

# Extract and transform data
caro "get unique IP addresses from nginx access log"
# Generates: awk '{print $1}' /var/log/nginx/access.log | sort -u
```

#### Data Extraction

**CSV/TSV processing:**

```bash
# Extract specific columns
caro "show first and third columns from CSV file"
# Generates: cut -d',' -f1,3 data.csv

# Sum values in a column
caro "sum all values in column 2 of numbers.txt"
# Generates: awk '{sum+=$2} END {print sum}' numbers.txt
```

### Network Operations

#### Safe Port Scanning

**Checking open ports:**

```bash
# macOS (uses lsof)
caro "show which processes are listening on network ports"
# Generates: lsof -iTCP -sTCP:LISTEN -n -P

# Linux (uses ss)
caro "list all listening TCP ports"
# Generates: ss -tuln | grep LISTEN
```

#### API Testing

**Safe curl patterns:**

```bash
# GET requests with headers
caro "test API endpoint with authorization header"
# Might generate: curl -H "Authorization: Bearer TOKEN" https://api.example.com/data

# POST with JSON (use --dry-run to review before sending)
caro --dry-run "send JSON to webhook"
# Review before executing to ensure data is correct
```

## Common Pitfalls

### 1. Recursive Deletion Without Review

❌ **Dangerous:**
```bash
caro "delete all node_modules directories"
# Might generate: find . -name "node_modules" -type d -exec rm -rf {} +
# Could delete dependencies across entire system!
```

✅ **Safe approach:**
```bash
# Step 1: Find and review
caro "find all node_modules directories"
find . -name "node_modules" -type d

# Step 2: Review output carefully
# Step 3: Delete specific ones manually if needed
rm -rf ./project1/node_modules
```

### 2. Assuming Command Availability

❌ **Problematic:**
```bash
# Assumes GNU tools on macOS
caro "find files modified today"
# Might generate: find . -mtime 0 -printf "%f\n"  # -printf not available on BSD find
```

✅ **Platform-aware:**
```bash
# Caro detects macOS and uses BSD-compatible syntax
caro "find files modified today"
# Generates: find . -mtime -1 -exec basename {} \;
```

### 3. Not Using Dry Run for Destructive Operations

❌ **Risky:**
```bash
caro "format USB drive as ext4"  # Immediately executes!
```

✅ **Safe:**
```bash
caro --dry-run "format USB drive as ext4"
# Review generated command
# Manually execute with correct device path
```

### 4. Over-Trusting Generated Commands

Even with safety validation, always review generated commands:

```bash
# Generated command might be syntactically correct but logically wrong
caro "copy all files except txt to backup"
# Always verify the exclusion pattern works as expected
find . ! -name "*.txt" -type f  # Test with ls first
find . ! -name "*.txt" -type f -exec cp {} backup/ \;
```

## Best Practices

### 1. Use Version Control for Config Changes

Before modifying configuration files:

```bash
# Backup first
caro "create backup of nginx config with timestamp"
cp /etc/nginx/nginx.conf "/etc/nginx/nginx.conf.backup-$(date +%Y%m%d)"

# Or use git for config management
cd /etc && sudo git add nginx/ && sudo git commit -m "Before changes"
```

### 2. Test in Isolated Environments

```bash
# Use Docker for testing destructive operations
docker run -it --rm ubuntu:latest bash
# Now test commands in isolated container
```

### 3. Leverage Shell History

```bash
# Review and refine generated commands
caro "find large files"
# Generated command appears in shell history
# Edit with up-arrow + modifications
# Execute when satisfied
```

### 4. Combine with Shell Aliases

Create aliases for common safety patterns:

```bash
# In ~/.bashrc or ~/.zshrc
alias caro-safe='caro --dry-run'
alias caro-explain='caro --verbose'

# Usage
caro-safe "delete old logs"  # Always dry-run by default
```

### 5. Use Configuration for Team Standards

Enforce safety levels for teams:

```bash
# In ~/.config/caro/config.toml
safety_level = "strict"  # Block dangerous commands
default_shell = "bash"   # Consistent across team
```

### 6. Document Generated Commands

When generating complex commands for runbooks:

```bash
# Generate, review, document
caro "archive logs older than 30 days" > scripts/archive-logs.sh
chmod +x scripts/archive-logs.sh
# Add comments and version to script
```

### 7. Verify Before Batch Operations

```bash
# For operations affecting multiple files:
# 1. Generate find/select command
# 2. Pipe to `wc -l` to count affected items
# 3. Pipe to `head` to preview
# 4. Execute action only after verification

caro "find Python files to format" | wc -l  # Count
caro "find Python files to format" | head -5  # Preview
# Then run formatter if counts look correct
```

## Safety Checklist

Before executing generated commands:

- [ ] Run with `--dry-run` first
- [ ] Understand what each part of the command does
- [ ] Check if command requires elevated privileges
- [ ] Verify file paths are correct
- [ ] Confirm command won't affect unintended files
- [ ] Have backups of important data
- [ ] Test in non-production environment if possible
- [ ] Know how to undo or rollback if needed

## Getting Help

- See command explanation: `caro --verbose "your query"`
- Check safety validation: Commands are automatically validated
- Report issues: https://github.com/wildcard/caro/issues
- Documentation: https://caro.run/docs

## Related Documentation

- [Safety Patterns](./SECURITY_SETTINGS.md) - Detailed safety configuration
- [Benchmarking Guide](./BENCHMARKING.md) - Performance testing patterns
- [Platform-Specific Guides](./MACOS_SETUP.md) - macOS and other platforms
