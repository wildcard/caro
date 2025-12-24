---
title: "grep: Search Text Like a Detective"
command: "grep"
description: "Search files for lines matching a pattern using regular expressions"
difficulty: "beginner"
platforms: ["linux", "macos", "bsd", "posix"]
tags: ["search", "text-processing", "regex", "essential"]
publishedAt: 2025-01-12
featured: false
relatedCommands: ["find", "awk", "sed", "ripgrep"]
caroPrompt: "Find all lines containing 'error' in log files"
---

# grep: The Pattern Matcher

## Quick Summary

Search for patterns in files. The name comes from the ed command `g/re/p` (globally search for regular expression and print).

## The 5 Commands You'll Actually Use

### 1. Basic search

```bash
grep "pattern" file.txt
```

Find lines containing "pattern" in file.txt.

### 2. Search recursively

```bash
grep -r "TODO" ./src
```

Search all files in src directory for "TODO".

### 3. Case-insensitive search

```bash
grep -i "error" log.txt
```

Find "error", "Error", "ERROR", etc.

### 4. Show line numbers

```bash
grep -n "function" script.js
```

Show matching lines with their line numbers.

### 5. Invert match

```bash
grep -v "debug" log.txt
```

Show all lines that DON'T contain "debug".

## Essential Options

| Option | Purpose | Example |
|--------|---------|---------|
| `-r` | Recursive search | `grep -r "api" ./` |
| `-i` | Case insensitive | `grep -i "error"` |
| `-n` | Line numbers | `grep -n "main"` |
| `-l` | Files only | `grep -l "TODO" *.js` |
| `-c` | Count matches | `grep -c "error" log` |
| `-v` | Invert match | `grep -v "^#"` |
| `-w` | Whole words | `grep -w "is"` |
| `-A N` | N lines after | `grep -A 3 "error"` |
| `-B N` | N lines before | `grep -B 3 "error"` |
| `-C N` | N lines context | `grep -C 3 "error"` |

## Real-World Examples

### Find errors with context

```bash
grep -C 5 "Exception" app.log
```

Show 5 lines before and after each exception.

### Count occurrences per file

```bash
grep -c "import" *.py | grep -v ":0$"
```

Count imports in each Python file, hide files with zero.

### Find files containing pattern

```bash
grep -l "deprecated" src/**/*.js
```

List only filenames containing "deprecated".

### Search multiple patterns

```bash
grep -E "error|warning|critical" log.txt
```

Match any of these patterns (extended regex).

### Exclude directories

```bash
grep -r --exclude-dir=node_modules "useState" .
```

Skip node_modules when searching.

### Match whole words only

```bash
grep -w "is" text.txt
```

Matches "is" but not "this" or "island".

## Regular Expression Power

```bash
# Lines starting with #
grep "^#" config.txt

# Lines ending with ;
grep ";$" code.c

# Match email-like patterns
grep -E "[a-zA-Z]+@[a-zA-Z]+\.[a-zA-Z]+" file.txt

# Match IP addresses (simple)
grep -E "[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+" log.txt
```

## Caro Connection

Ask Caro:
> "Find all lines containing 'error' in log files"

Caro suggests:
```bash
grep -r "error" *.log
```

Or with case-insensitivity:
```bash
grep -ri "error" /var/log/
```

## Modern Alternative: ripgrep

For faster searching in large codebases:

```bash
# Install: cargo install ripgrep
rg "pattern" ./src

# ripgrep ignores .gitignore patterns automatically
# and is significantly faster on large codebases
```

## The Name's Origin

`grep` comes from the ed text editor command:
- **g** - globally (all lines)
- **re** - regular expression
- **p** - print

So `g/re/p` means "globally search for regular expression and print".

## Common Mistakes

1. **Forgetting quotes**: `grep hello world.txt` searches for "hello" in two files
2. **Literal dots**: Use `\.` to match actual dots (`.` matches any character)
3. **Special characters**: Escape `$`, `^`, `*`, `[`, `]` in patterns

---

*grep: Finding needles in haystacks since 1973.*
