# QUICKSTART - 5 Minutes to Productivity

Get from zero to generating shell commands in under 5 minutes.

## Installation (30 seconds)

### macOS
```bash
brew install wildcard/tap/cmdai
```

### Linux
```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | sh
```

### Windows
```powershell
irm https://raw.githubusercontent.com/wildcard/cmdai/main/install.ps1 | iex
```

### From Source
```bash
cargo install cmdai
```

## First Command (1 minute)

First run downloads the model (1.1GB, one-time only):

```bash
cmdai "list all files in current directory"
```

**Output:**
```
📦 First-time setup: Downloading model (Qwen2.5-Coder-1.5B)...
[████████████████████████████████] 1.1 GB/1.1 GB (5.2 MB/s, 3m 32s)
✅ Model downloaded successfully

Generated command:
  ls -la

Execute this command? (y/N)
```

Type `y` and press Enter. That's it. You just converted natural language to a shell command.

## Common Use Cases (2 minutes)

Real examples that show immediate value:

### File Operations
```bash
# Find large files
cmdai "find all PDF files larger than 10MB"
# → find . -name "*.pdf" -size +10M

# Clean up
cmdai "delete all .DS_Store files recursively"
# → find . -name ".DS_Store" -type f -delete

# Archive directory
cmdai "compress the logs folder into a tar.gz archive"
# → tar -czf logs.tar.gz logs/
```

### Git Workflows
```bash
# Branch management
cmdai "create a new branch called feature/auth and switch to it"
# → git checkout -b feature/auth

# Cleanup
cmdai "delete all local branches that have been merged"
# → git branch --merged | grep -v '\*\|main\|master' | xargs -n 1 git branch -d

# Status check
cmdai "show commits that are ahead of origin/main"
# → git log origin/main..HEAD
```

### System Tasks
```bash
# Disk usage
cmdai "show disk usage for home directory, sorted by size"
# → du -sh ~/* | sort -h

# Process management
cmdai "find processes using more than 1GB of memory"
# → ps aux | awk '$6 > 1048576 {print $0}'

# Network
cmdai "show all listening TCP ports"
# → netstat -tuln | grep LISTEN
```

### Text Processing
```bash
# Code analysis
cmdai "count lines in all Python files, excluding blank lines"
# → find . -name "*.py" -exec wc -l {} + | awk '{total+=$1} END {print total}'

# Log parsing
cmdai "find all ERROR entries in server.log from today"
# → grep "ERROR" server.log | grep "$(date +%Y-%m-%d)"
```

### Docker & Containers
```bash
# Container management
cmdai "list all running containers with their CPU usage"
# → docker stats --no-stream

# Cleanup
cmdai "remove all stopped containers and dangling images"
# → docker container prune -f && docker image prune -f
```

## Key Flags (1 minute)

```bash
# Execute without confirmation (use with caution!)
cmdai --execute "show current directory size"

# Target specific shell
cmdai --shell zsh "create alias for git status"

# Increase safety validation
cmdai --safety strict "remove old log files"

# JSON output for scripting
cmdai --output json "list processes" | jq '.cmd'

# Verbose mode with timing
cmdai --verbose "search for TODO comments"
```

### Safety Levels
- `strict` - Blocks potentially dangerous operations (default)
- `moderate` - Warns on risky commands, requires confirmation
- `permissive` - Allows most operations with user confirmation

### Combining Flags
```bash
# Common patterns
cmdai -s zsh --execute "git status"           # Quick execution
cmdai --safety permissive -o json "cleanup"   # Scripting mode
cmdai -v --dry-run "complex operation"        # Debug mode
```

## Next Steps (30 seconds)

**Learn More:**
- [USER_GUIDE.md](USER_GUIDE.md) - Complete documentation
- [FAQ.md](FAQ.md) - Common questions answered
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - When things go wrong

**Configuration:**
```bash
# View current config
cmdai --show-config

# Edit config file
$EDITOR ~/.config/cmdai/config.toml
```

**Shell Integration:**
```bash
# Add to ~/.bashrc or ~/.zshrc
alias cmd="cmdai --execute"
alias cmds="cmdai --safety strict"
```

**Community:**
- Report bugs: [GitHub Issues](https://github.com/wildcard/cmdai/issues)
- Request features: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- Contribute: [CONTRIBUTING.md](CONTRIBUTING.md)

---

**Pro Tip:** Start with simple queries and build complexity. The LLM learns from your patterns and generates better commands over time.

**Safety First:** cmdai validates all commands before execution. Dangerous operations (like `rm -rf /`) are blocked or require explicit confirmation.

Ready to level up your command-line productivity? Start typing.
