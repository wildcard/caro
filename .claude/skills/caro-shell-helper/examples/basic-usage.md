# Caro Basic Usage Examples

## Getting Started

### Installation Check

```bash
# Check if Caro is installed
command -v caro && echo "✓ Installed" || echo "✗ Not installed"

# Show version
caro --version
```

### First Command

```bash
# Simple file listing
$ caro "list all files in current directory"

Generated command:
  ls -lah

Safety Assessment: ✅ Safe (Green)
Execute this command? (y/N) y
```

## Common Use Cases

### 1. Finding Files

```bash
# Find by name
$ caro "find all PDF files"
Generated: find . -name "*.pdf" -type f

# Find by size
$ caro "find files larger than 100MB"
Generated: find . -type f -size +100M

# Find by modification time
$ caro "find files modified in last 24 hours"
Generated: find . -type f -mtime -1

# Complex search
$ caro "find all Python files modified this week that contain 'TODO'"
Generated: find . -name "*.py" -type f -mtime -7 -exec grep -l "TODO" {} \;
```

### 2. Text Processing

```bash
# Search in files
$ caro "search for 'error' in all log files"
Generated: grep -r "error" --include="*.log" .

# Count occurrences
$ caro "count how many times 'function' appears in all JavaScript files"
Generated: grep -r "function" --include="*.js" . | wc -l

# Replace text (preview mode)
$ caro "show what would change if I replace 'old' with 'new' in file.txt"
Generated: sed -n 's/old/new/gp' file.txt
```

### 3. File Operations

```bash
# Copy with structure
$ caro "copy all PDF files to ~/Documents/PDFs preserving directory structure"
Generated: find . -name "*.pdf" -type f -exec cp --parents {} ~/Documents/PDFs \;

# Create backup
$ caro "create a compressed backup of my project folder"
Generated: tar czf project-backup-$(date +%Y%m%d).tar.gz project/

# Rename files
$ caro "rename all .jpeg files to .jpg in current directory"
Generated: for f in *.jpeg; do mv "$f" "${f%.jpeg}.jpg"; done
```

### 4. System Information

```bash
# Disk usage
$ caro "show disk usage by directory, sorted by size"
Generated: du -sh */ | sort -h

# Process information
$ caro "show top 10 processes by memory usage"
Generated: ps aux --sort=-%mem | head -11

# Network connections
$ caro "show all listening TCP ports"
Generated: netstat -tln
# or: ss -tln
```

### 5. Data Processing

```bash
# Sort and filter
$ caro "sort users.txt by email address (second column) and remove duplicates"
Generated: sort -u -k2 users.txt

# Extract specific columns
$ caro "show just the usernames from /etc/passwd"
Generated: cut -d: -f1 /etc/passwd

# Combine files
$ caro "merge all CSV files in current directory into combined.csv"
Generated: cat *.csv > combined.csv
```

## Interactive Examples

### Example Session 1: Safe File Cleanup

```bash
$ caro "find and delete all .tmp files older than 7 days"

Generated command:
  find . -name "*.tmp" -type f -mtime +7 -delete

Safety Assessment: 🟠 High (Orange)
- Performs deletion (irreversible)
- Recursive operation
- Affects multiple files

⚠️ RECOMMENDATION: Preview files first:
  find . -name "*.tmp" -type f -mtime +7 -ls

Preview files? (y/N) y
[Shows list of files that would be deleted]

Proceed with deletion? (y/N) y
[Deletes files]
```

### Example Session 2: System Monitoring

```bash
$ caro "watch disk space every 5 seconds"

Generated command:
  watch -n 5 df -h

Safety Assessment: ✅ Safe (Green)
- Read-only operation
- No system modifications

Execute this command? (y/N) y
[Runs watch command, Ctrl+C to exit]
```

### Example Session 3: Log Analysis

```bash
$ caro "show the 20 most common error messages in application.log"

Generated command:
  grep -i error application.log | sort | uniq -c | sort -rn | head -20

Safety Assessment: ✅ Safe (Green)
- Read-only operation
- Standard text processing

Explanation:
1. grep -i error application.log    : Find lines with 'error' (case-insensitive)
2. sort                              : Sort error messages
3. uniq -c                           : Count unique occurrences
4. sort -rn                          : Sort by count (highest first)
5. head -20                          : Show top 20

Execute this command? (y/N) y
```

## Configuration Examples

### Custom Safety Level

```bash
# Edit config
vim ~/.config/Caro/config.toml
```

```toml
[safety]
enabled = true
level = "moderate"  # strict, moderate, or permissive
require_confirmation = true
```

### Backend Selection

```toml
[backend]
primary = "embedded"  # embedded, ollama, or vllm
enable_fallback = true

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"
```

### Output Format

```bash
# JSON output (for scripting)
$ caro --output json "list all files"
{
  "command": "ls -lah",
  "safety_level": "safe",
  "explanation": "List files with details"
}

# YAML output
$ caro --output yaml "find large files"
command: "find . -type f -size +100M"
safety_level: "safe"
explanation: "Find files larger than 100MB"
```

## Tips for Better Results

### 1. Be Specific

```bash
# ✗ Vague
$ caro "find files"

# ✓ Specific
$ caro "find all JavaScript files modified in the last week in the src/ directory"
```

### 2. Include Safety Context

```bash
# ✗ No context
$ caro "delete log files"

# ✓ With safety context
$ caro "safely delete log files older than 30 days, with preview first"
```

### 3. Mention Output Requirements

```bash
# ✗ Unclear format
$ caro "show processes"

# ✓ Specific format
$ caro "show processes sorted by memory usage, with headers, human-readable sizes"
```

### 4. Request Explanations

```bash
# ✗ Just the command
$ caro "complicated operation"

# ✓ With explanation
$ caro "explain how to recursively find and compress all log files older than a month"
```

## Common Workflows

### Workflow 1: Safe Deletion

1. Generate command: `caro "delete old logs"`
2. Review safety assessment
3. Preview files: `caro "show me which log files would be deleted"`
4. Confirm scope is correct
5. Execute deletion with confirmation

### Workflow 2: Data Transformation

1. Test on sample: `caro "convert first 10 lines of data.csv to JSON"`
2. Verify output format
3. Run on full dataset: `caro "convert entire data.csv to JSON"`
4. Save output: `caro "convert data.csv to JSON and save as data.json"`

### Workflow 3: System Administration

1. Check current state: `caro "show current disk usage"`
2. Plan action: `caro "what's the safe way to clean up /var/log"`
3. Preview impact: `caro "show which logs would be deleted"`
4. Execute with confirmation
5. Verify result: `caro "show disk usage again"`

## Error Handling

### Safety Violation

```bash
$ caro "delete everything in root directory"

Error: Unsafe command detected: Detected dangerous pattern(s) at Critical risk level

Safety block: Critical risk: recursive deletion

Safer alternatives:
  Try: find ~/projects -name '*.tmp' -type f -mtime +30 -ls
  Preview files to delete first. Then use 'rm' on specific directories.
```

## Advanced Usage

### Chaining Commands

```bash
$ caro "find all large files, compress them, and move to archive/"
Generated: find . -type f -size +100M -exec gzip {} \; -exec mv {}.gz archive/ \;

# Or as separate steps
$ caro "first step: find large files"
$ caro "second step: compress the files I found"
$ caro "third step: move compressed files to archive/"
```

### Custom Patterns

Add to `~/.config/Caro/config.toml`:

```toml
[aliases]
"cleanup logs" = "find /var/log -name '*.log' -mtime +30 -delete"
"backup home" = "tar czf ~/backup-$(date +%Y%m%d).tar.gz ~"
```

Then:
```bash
$ caro "cleanup logs"
# Uses your custom pattern
```

## Getting Help

```bash
# Show help
caro --help

# Show current configuration
caro --show-config

# Verbose mode (for debugging)
caro --verbose "your command description"

# Check backend status
caro --backend-info
```

## Summary

Caro makes shell command generation:
- ✅ **Safe**: Comprehensive safety validation
- ✅ **Easy**: Natural language input
- ✅ **Educational**: Explains what commands do
- ✅ **Portable**: POSIX-compliant output
- ✅ **Fast**: Local LLM inference

Start simple, build confidence, and let Caro help you become more effective on the command line!
