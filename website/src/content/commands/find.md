---
title: "find: Search for Files Like a Pro"
command: "find"
description: "Recursively search directories for files matching complex criteria"
difficulty: "intermediate"
platforms: ["linux", "macos", "bsd", "posix"]
tags: ["search", "filesystem", "automation", "essential"]
publishedAt: 2025-01-10
featured: true
relatedCommands: ["xargs", "locate", "fd", "grep"]
caroPrompt: "Find all files larger than 100MB modified in the last week"
---

# find: The Swiss Army Knife of File Search

## Quick Summary

Search for files and directories based on name, size, date, permissions, and more. The most powerful file discovery tool in Unix.

## The 5 Commands You'll Actually Use

### 1. Find by name

```bash
find . -name "*.log"
```

Find all log files in current directory and subdirectories.

### 2. Find by size

```bash
find /var -size +100M
```

Find files larger than 100MB in /var.

### 3. Find recently modified

```bash
find . -mtime -7
```

Find files modified in the last 7 days.

### 4. Find and execute

```bash
find . -name "*.tmp" -exec rm {} \;
```

Find and delete all .tmp files.

### 5. Find by type

```bash
find . -type d -name "node_modules"
```

Find directories named node_modules.

## Deep Dive: Search Criteria

### By Time

| Option | Meaning | Example |
|--------|---------|---------|
| `-mtime -N` | Modified < N days ago | `-mtime -1` (today) |
| `-mtime +N` | Modified > N days ago | `-mtime +30` (old) |
| `-mmin -N` | Modified < N minutes ago | `-mmin -60` |
| `-newer file` | Newer than file | `-newer reference.txt` |

### By Size

| Option | Meaning |
|--------|---------|
| `-size +100M` | Larger than 100MB |
| `-size -1k` | Smaller than 1KB |
| `-empty` | Zero size files |

### By Type

| Option | Meaning |
|--------|---------|
| `-type f` | Regular files |
| `-type d` | Directories |
| `-type l` | Symbolic links |
| `-type s` | Sockets |

### By Permissions

```bash
find . -perm 644           # Exact match
find . -perm -u+x          # User executable
find . -perm /a+x          # Anyone executable
```

## Real-World Examples

### Clean up old logs

```bash
find /var/log -name "*.log" -mtime +30 -delete
```

### Find large files hogging disk

```bash
find / -type f -size +1G 2>/dev/null | head -20
```

### Find files not accessed in a year

```bash
find . -atime +365 -type f
```

### Find broken symlinks

```bash
find . -xtype l
```

### Find files owned by nobody

```bash
find / -nouser -o -nogroup 2>/dev/null
```

### Combine with xargs for speed

```bash
find . -name "*.js" -print0 | xargs -0 grep "TODO"
```

## Caro Connection

Ask Caro:
> "Find all files larger than 100MB modified in the last week"

Caro suggests:
```bash
find . -size +100M -mtime -7 -type f
```

## The -exec vs xargs Debate

### Using -exec

```bash
find . -name "*.txt" -exec cat {} \;
```

- Runs command once per file
- Slower for many files
- Simpler syntax

### Using xargs

```bash
find . -name "*.txt" | xargs cat
```

- Batches files together
- Much faster
- Handles edge cases with `-print0`/`-0`

### Best practice

```bash
find . -name "*.txt" -print0 | xargs -0 cat
```

## Platform Differences

| Feature | GNU (Linux) | BSD (macOS) |
|---------|-------------|-------------|
| `-delete` | Yes | Yes |
| `-printf` | Yes | No (use `-print`) |
| `-regex` | Extended | Basic (use `-E`) |
| `-iname` | Yes | Yes |

## Modern Alternative: fd

For interactive use, consider `fd` (rust-based):

```bash
fd "\.log$"           # vs find . -name "*.log"
fd -e js -x wc -l     # vs find . -name "*.js" -exec wc -l {} +
```

`fd` is faster and has better defaults, but `find` is universal.

## History

`find` dates back to Version 5 Unix (1974). Its power comes from its ability to combine multiple search criteria with boolean logic (`-and`, `-or`, `-not`).

---

*Try it yourself with Caro: "find all empty directories"*
