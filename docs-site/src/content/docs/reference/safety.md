---
title: Safety Validation
description: How caro validates commands for safety before execution
---

caro includes a comprehensive safety validation system to prevent dangerous command execution.

## Risk Levels

Commands are categorized into four risk levels:

| Level | Color | Description | Examples |
|-------|-------|-------------|----------|
| **Safe** | Green | Normal read operations | `ls`, `cat`, `find`, `grep` |
| **Moderate** | Yellow | File modifications | `mv`, `cp`, `chmod` (non-system) |
| **High** | Orange | System-level changes | `sudo`, `chown`, system paths |
| **Critical** | Red | Blocked - dangerous | `rm -rf /`, fork bombs |

## Dangerous Pattern Detection

### Filesystem Destruction

```bash
# These patterns are BLOCKED:
rm -rf /
rm -rf ~
rm -rf /*
rm -rf /home
rm -rf /usr
rm -rf /bin
rm -rf /etc
```

### Disk Operations

```bash
# These patterns are BLOCKED:
dd if=/dev/zero of=/dev/sda
mkfs.ext4 /dev/sda
> /dev/sda
shred /dev/sda
```

### Fork Bombs

```bash
# These patterns are BLOCKED:
:(){ :|:& };:
bomb() { bomb | bomb & }; bomb
while true; do $0 & done
```

### Privilege Escalation

```bash
# These patterns are flagged HIGH risk:
sudo su -
sudo bash
chmod 777 /
chmod -R 777 /etc
```

### System Path Modifications

```bash
# Operations on these paths are flagged HIGH risk:
/bin/
/sbin/
/usr/bin/
/usr/sbin/
/etc/
/var/
/boot/
```

## Validation Pipeline

```
┌─────────────┐     ┌───────────────┐     ┌──────────────┐
│  Generated  │────▶│   Pattern     │────▶│   Path       │
│   Command   │     │   Matching    │     │  Validation  │
└─────────────┘     └───────────────┘     └──────────────┘
                                                 │
                                                 ▼
                    ┌───────────────┐     ┌──────────────┐
                    │   Risk Level  │◀────│   POSIX      │
                    │   Assignment  │     │  Compliance  │
                    └───────────────┘     └──────────────┘
```

## POSIX Compliance

caro validates commands for POSIX compliance:

### Allowed Utilities

Standard POSIX utilities are preferred:

```bash
# File operations
ls, find, cp, mv, rm, mkdir, rmdir

# Text processing
cat, head, tail, grep, sed, awk, sort, uniq, wc

# System info
ps, df, du, who, date, uname

# Network
ping, curl, wget (where available)
```

### Bash-Specific Avoidance

When possible, bash-specific features are avoided for portability:

```bash
# Avoid:
[[ condition ]]    # Use [ condition ] instead
$((arithmetic))    # Use expr instead where possible
{a..z}             # Use seq or explicit lists
```

## Path Quoting

caro automatically quotes paths with special characters:

```bash
# Input: file with spaces.txt
# Output: "file with spaces.txt"

# Input: file's name.txt
# Output: "file's name.txt"

# Input: file$var.txt
# Output: 'file$var.txt'
```

## Override Safety (Not Recommended)

For advanced users who understand the risks:

```bash
# Skip safety validation (DANGEROUS)
caro --unsafe "dangerous command"

# Acknowledge specific risk level
caro --allow-high-risk "system command"
```

These flags require explicit confirmation and are logged.

## Configuration

Customize safety behavior in `config.toml`:

```toml
[safety]
# Enable safety warnings (default: true)
warnings = true

# Require confirmation for moderate risk (default: true)
confirm_moderate = true

# Require confirmation for high risk (default: true)
confirm_high = true

# Block critical commands (default: true)
block_critical = true

# Custom blocked patterns
blocked_patterns = [
    "custom-dangerous-command",
]

# Custom allowed patterns (override blocks)
allowed_patterns = [
    "rm -rf ./node_modules",  # Allow specific cleanup
]
```

## Reporting Security Issues

If you find a way to bypass safety validation:

1. **Do not** disclose publicly
2. **Email** security@caro.sh with details
3. **Include** the command that bypassed validation
4. **Wait** for confirmation before public disclosure

We take security seriously and will respond within 48 hours.
