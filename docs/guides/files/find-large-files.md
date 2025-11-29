---
id: "guide-files-001"
title: "Find largest files in directory"
description: "Quickly identify which files are taking up the most disk space"
category: FileManagement
difficulty: Intermediate
tags: [find, disk-usage, du, sort, files, cleanup]
natural_language_prompt: "find the largest files in this directory"
generated_command: "du -ah . | sort -rh | head -20"
shell_type: Bash
risk_level: Safe
author: "cmdai-community"
created_at: "2024-02-01T14:00:00Z"
updated_at: "2024-11-28T16:00:00Z"
prerequisites:
  - "You are in the directory you want to search"
  - "You have read permissions for the files"
expected_outcomes:
  - "List of 20 largest files/directories"
  - "Human-readable sizes (MB, GB, etc.)"
  - "Sorted from largest to smallest"
related_guides:
  - "guide-files-002"
  - "guide-files-010"
related_guardrails: []
alternatives:
  - "ncdu                                    # Interactive disk usage analyzer"
  - "find . -type f -exec du -h {} + | sort -rh | head -20  # Files only (no dirs)"
  - "du -h --max-depth=1 | sort -rh         # Top-level directories only"
---

# Find Largest Files in Directory

## What it does

Lists the 20 largest files and directories in the current location, showing their sizes in human-readable format (KB, MB, GB) sorted from largest to smallest.

## When to use this

- ✅ Your disk is running out of space
- ✅ You want to find what's taking up space before cleaning up
- ✅ You're investigating unexpectedly large directories
- ✅ You need to identify files for archiving or deletion
- ✅ You're auditing project file sizes

## The cmdai way

```bash
cmdai "find the largest files in this directory"
```

cmdai generates:
```bash
du -ah . | sort -rh | head -20
```

## Understanding the command

```bash
du -ah . | sort -rh | head -20
```

Breaking it down:
- `du`: Disk usage command
- `-a`: Show all files (not just directories)
- `-h`: Human-readable sizes (1.2G instead of 1234567)
- `.`: Current directory
- `|`: Pipe output to next command
- `sort -rh`: Sort in reverse (-r) human-readable (-h) order
- `head -20`: Show only first 20 results

## Step-by-step example

Navigate to directory:
```bash
cd ~/projects/my-app
```

Run the command:
```bash
$ du -ah . | sort -rh | head -20
5.2G    .
1.8G    ./node_modules
1.2G    ./target/release
850M    ./data/logs
420M    ./node_modules/webpack
380M    ./target/release/my-app
156M    ./data/database.sqlite
98M     ./assets/videos
45M     ./dist/bundle.js
32M     ./coverage
18M     ./docs/images
12M     ./src
8.4M    ./.git
5.6M    ./tests
2.1M    ./config
890K    ./scripts
456K    ./README.md
234K    ./package.json
89K     ./Cargo.toml
12K     ./.gitignore
```

Now you know:
- Total size: 5.2GB
- Biggest culprit: `node_modules` at 1.8GB
- Second biggest: Build artifacts in `target/release`

## What the output means

Each line shows:
```
<size>  <path>
```

**First line (`.`)** = Total size of current directory and all subdirectories

**Subsequent lines** = Individual files and directories, largest first

## Common findings and solutions

**Problem 1: `node_modules` is huge**
```bash
# Clean and reinstall
rm -rf node_modules
npm install

# Or use pnpm for smaller installs
pnpm install
```

**Problem 2: Build artifacts taking space**
```bash
# Rust: Clean build directory
cargo clean

# Node: Clean build
rm -rf dist/ build/

# Python: Clean cache
find . -type d -name __pycache__ -exec rm -r {} +
```

**Problem 3: Log files growing out of control**
```bash
# Truncate old logs
find ./logs -name "*.log" -mtime +30 -delete

# Or compress them
find ./logs -name "*.log" -mtime +7 -exec gzip {} \;
```

## Variations for different needs

**Only show files (exclude directories):**
```bash
find . -type f -exec du -h {} + | sort -rh | head -20
```

**Only show directories:**
```bash
du -h --max-depth=1 | sort -rh
```

**Search entire home directory:**
```bash
du -ah ~/ | sort -rh | head -50
```

**Find files larger than specific size:**
```bash
find . -type f -size +100M -exec du -h {} + | sort -rh
```

**Interactive alternative (better UX):**
```bash
ncdu  # Interactive, navigable disk usage analyzer
```

## Limiting search scope

**Max depth (don't recurse too deep):**
```bash
du -ah --max-depth=2 . | sort -rh | head -20
```

**Specific subdirectory:**
```bash
du -ah ./src | sort -rh | head -20
```

**Exclude certain directories:**
```bash
du -ah --exclude=node_modules --exclude=target . | sort -rh | head -20
```

## Performance tips

**For very large directories:**
```bash
# Use --max-depth to limit recursion
du -h --max-depth=3 . | sort -rh | head -20

# Or use find with size filter first
find . -type f -size +50M -exec du -h {} + | sort -rh
```

**For network drives (slow):**
```bash
# Cache results for repeated analysis
du -ah . > sizes.txt
sort -rh sizes.txt | head -20
```

## Safety notes

✓ **Read-only operation** - This command only reads, never modifies files

✓ **Safe to run** - No risk of data loss

⚠️ **Can be slow** - Large directories may take minutes to scan

⚠️ **Permissions** - Won't show files you don't have read access to

## After finding large files

**Delete safely:**
```bash
# Verify what it is first
file large-file.dat
head large-file.dat

# Then delete
rm large-file.dat
```

**Archive instead of delete:**
```bash
# Compress and move to archive
tar czf archive/large-file.tar.gz large-file.dat
rm large-file.dat
```

**Move to external storage:**
```bash
# Move to external drive
mv large-file.dat /mnt/external/backups/
```

## Related guides

- [Find files by size](../files/find-by-size.md) - Using `find -size`
- [Clean up disk space](../files/disk-cleanup.md) - Systematic cleanup
- [Compress large files](../files/compress-files.md) - Using `tar` and `gzip`
- [Interactive disk analyzer](../files/ncdu-guide.md) - Using `ncdu`

## Try it yourself

```bash
# Try this guide in cmdai
cmdai guides run guide-files-001

# Or execute the command directly
du -ah . | sort -rh | head -20

# For even better results, try interactive mode
ncdu
```

## Community metrics

- **Upvotes:** 203
- **Downvotes:** 7
- **Execution count:** 4,521
- **Success rate:** 99%
- **Quality score:** 0.96

## Community feedback

> "This saved my CI/CD pipeline. Found 40GB of cached artifacts!" - *devops_hero*

> "Love this command. Use it weekly to keep my projects lean." - *clean_coder*

> "Suggestion: Add ncdu alternative - way better UX for interactive exploration" - *ux_matters*
> *Done! Added ncdu as alternative - thanks for the suggestion!*

> "Would be cool to combine with fzf for interactive selection" - *cli_power_user*
> *Great idea! We'll add a guide for that integration!*
