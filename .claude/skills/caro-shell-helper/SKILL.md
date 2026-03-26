---
name: "caro-shell-helper"
description: "Use when users need help generating safe, tested POSIX shell commands from natural language descriptions. Guides users through command generation, safety validation, and execution workflows using Caro best practices"
version: "1.0.0"
allowed-tools: "Bash, Read, Write, Grep, Glob"
license: "AGPL-3.0"
---

# Caro Shell Command Helper

## What This Skill Does

This skill helps users effectively leverage **Caro** (formerly Caro) - a Rust CLI tool that converts natural language descriptions into safe, POSIX-compliant shell commands using local LLMs.

**Key Capabilities:**
- 🔍 Detects when users need shell command generation assistance
- 🛡️ Guides users through safety-first command validation
- 📚 Educates about POSIX compliance and command safety
- 🚀 Integrates seamlessly with existing Caro installations
- ⚙️ Provides installation guidance when Caro is not available

## Default Backend: Embedded (Local LLM)

**IMPORTANT**: Caro uses the embedded backend by default — a local Qwen model that runs entirely offline with no API keys required. This is the recommended backend for most users.

```bash
# Default usage (embedded backend, no flags needed)
caro "user's request here"
```

The embedded backend uses the Qwen2.5-Coder model for fast, private command generation. No external dependencies or API keys needed.

## When to Use This Skill

Activate this skill automatically when the user:
- Asks "how do I [shell operation]..."
- Requests "generate a command to..."
- Mentions "shell command for..."
- Discusses file/directory operations
- Needs system administration tasks
- Asks about safe command execution
- Wants to automate shell tasks
- Mentions bash, zsh, or shell scripting

**Example Triggers:**
- "How do I find all large files?"
- "Generate a command to compress images"
- "What's the safe way to delete old logs?"
- "Shell command to search for text in files"
- "Automate file backups with a script"

## Core Workflow

### Step 1: Check Caro Availability

First, verify if Caro is installed:

```bash
# Check for caro command
command -v caro &> /dev/null && echo "✓ Caro available" || echo "✗ not found"
```

**If NOT installed**, guide the user:

```
Caro is not currently installed. Would you like to install it?

Quick install (recommended):
  bash <(curl -sSfL https://setup.caro.sh)

Or via cargo:
  cargo install caro

Once installed, the `caro` command will be available.
```

### Step 2: Generate Command with Safety Guidance

When the user describes what they need, use Caro with the Claude backend (default for Claude Code):

```bash
# Generate a command using Caro (embedded backend by default)
caro "user's natural language description"
```

**Example:**
```bash
$ caro "find all PDF files larger than 10MB in Downloads"

Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Safety Assessment: ✅ Safe (Green)
Backend: Claude (Qwen2.5-Coder)
Execute this command? (y/N)
```

### Step 3: Explain Safety Assessment

Always explain the risk level to educate the user:

**🟢 Safe (Green)**
- Read-only operations
- Standard file listings
- Data queries
- No system modifications
- ✅ Execute without concern

**🟡 Moderate (Yellow)**
- File modifications
- Package operations
- Non-critical deletions
- ⚠️ Review before executing

**🟠 High (Orange)**
- Recursive deletions
- Mass file operations
- System configuration changes
- ⚠️ Carefully review and confirm

**🔴 Critical (Red)**
- System destruction patterns (`rm -rf /`)
- Disk operations (`mkfs`, `dd`)
- Fork bombs
- Privilege escalation
- 🛑 BLOCKED or requires explicit override

### Step 4: POSIX Compliance Education

Emphasize why POSIX compliance matters:

```
✓ POSIX-Compliant Command:
  find ~/Downloads -name "*.pdf" -size +10M

This works on: bash, zsh, sh, dash, ksh (all POSIX shells)

✗ Bash-Specific (Non-Portable):
  find ~/Downloads -name "*.pdf" -size +10M -print0 | xargs -0 ls -lh

Problem: -print0 is not POSIX standard
Better: Use -exec for portability
```

### Step 5: Command Execution Guidance

After Caro generates a command, guide the user:

1. **Review the command**: "Do you understand what each part does?"
2. **Check safety level**: "This is a [risk level] operation"
3. **Verify scope**: "Will this affect the files/directories you intend?"
4. **Test safely**: "Try with a small subset first, or use --dry-run if available"
5. **Execute**: "Ready to run? The command will execute when you confirm"

## Safety Patterns to Highlight

### Always Block These Patterns

**System Destruction:**
```bash
rm -rf /          # Deletes entire filesystem
rm -rf ~          # Deletes home directory
rm -rf /*         # Same as rm -rf /
```

**Fork Bombs:**
```bash
:(){ :|:& };:     # Creates infinite processes
```

**Disk Operations:**
```bash
mkfs /dev/sda     # Formats disk (destroys data)
dd if=/dev/zero of=/dev/sda  # Overwrites disk
```

**Privilege Escalation:**
```bash
sudo su           # Gains root access
chmod 777 /       # Makes root world-writable
```

### Suggest Safe Alternatives

When users request dangerous operations, suggest safer alternatives:

**User wants:** "Delete all log files"
**Dangerous:** `rm -rf /var/log/*`
**Safe alternative:**
```bash
# First, preview what will be deleted
find /var/log -name "*.log" -type f -mtime +30 -ls

# Then delete only old logs with confirmation
find /var/log -name "*.log" -type f -mtime +30 -exec rm -i {} \;
```

**User wants:** "Clean up temp files"
**Dangerous:** `rm -rf /tmp/*`
**Safe alternative:**
```bash
# Use tmpwatch or tmpreaper (proper cleanup tools)
tmpwatch 24 /tmp

# Or manually with safety checks
find /tmp -type f -mtime +7 -user $(whoami) -delete
```

## Backend Configuration Guidance

Caro supports multiple inference backends. Help users choose:

### Embedded Backend (Default)
- **Best for**: Running directly within Claude Code or with Anthropic API access
- **Advantages**: No external dependencies, works offline, privacy-first
- **Model**: Qwen2.5-Coder (quantized)
- **Setup**:
  ```toml
  # ~/.config/caro/config.toml
  [backend]
  primary = "claude"

  [backend.claude]
  model_name = "Qwen2.5-Coder"  # Fast and cost-effective
  # API key read from ANTHROPIC_API_KEY environment variable
  ```

**When running in Claude Code**: The Claude backend is automatically preferred as it provides the best integration and fastest response times. No additional configuration is needed - Claude Code provides the API key automatically.



### Ollama Backend
- **Best for**: Local model control, privacy-focused users
- **Advantages**: Many model choices, easy model management
- **Setup**:
  ```toml
  # ~/.config/caro/config.toml
  [backend]
  primary = "ollama"

  [backend.ollama]
  base_url = "http://localhost:11434"
  model_name = "codellama:7b"
  ```

### vLLM Backend
- **Best for**: High-performance inference, server deployments
- **Advantages**: Optimized for throughput, production-ready
- **Setup**:
  ```toml
  [backend]
  primary = "vllm"

  [backend.vllm]
  base_url = "http://localhost:8000"
  model_name = "codellama/CodeLlama-7b-hf"
  ```

## Usage Examples

### Example 1: Finding Files

**User:** "I need to find all Python files modified in the last week"

**Response:**
```
I'll help you generate a safe command using Caro with embedded backend.

$ caro "find all Python files modified in the last week"

Generated command:
  find . -name "*.py" -type f -mtime -7

Safety Assessment: ✅ Safe (Green)
Backend: Claude (Qwen2.5-Coder)
- Read-only operation
- POSIX-compliant syntax
- Searches current directory and subdirectories

Explanation:
- find .           : Search from current directory
- -name "*.py"     : Match files ending in .py
- -type f          : Only regular files (not directories)
- -mtime -7        : Modified within last 7 days

Execute? (y/N)
```

### Example 2: Safe Deletion

**User:** "Delete all .log files older than 30 days"

**Response:**
```
This involves deletion, so let's be careful. I'll use Caro with embedded backend.

$ caro "delete all .log files older than 30 days"

Generated command:
  find . -name "*.log" -type f -mtime +30 -delete

Safety Assessment: 🟠 High (Orange)
Backend: Claude (Qwen2.5-Coder)
- Performs deletion (irreversible)
- Recursive operation
- Affects multiple files

⚠️ SAFETY RECOMMENDATION:
Preview what will be deleted first:

$ find . -name "*.log" -type f -mtime +30 -ls

Review the list, then if it looks correct, run with -delete.

Better yet, use interactive deletion:
$ find . -name "*.log" -type f -mtime +30 -exec rm -i {} \;

This will ask for confirmation on each file.

Proceed with preview? (y/N)
```

### Example 3: System Operations

**User:** "How do I check disk space?"

**Response:**
```
$ caro "check disk space for all mounted filesystems"

Generated command:
  df -h

Safety Assessment: ✅ Safe (Green)
Backend: Claude (Qwen2.5-Coder)
- Read-only operation
- Standard system utility
- No modifications

Explanation:
- df    : Disk free - shows filesystem disk space usage
- -h    : Human-readable format (GB, MB instead of bytes)

This is completely safe to run. Execute? (y/N)
```

## Advanced Features

### Multi-Step Workflows

For complex tasks, break them into steps:

**User:** "Find large files and compress them"

**Response:**
```
This is a multi-step operation. Let's do it safely using embedded backend:

Step 1: Find large files (>100MB)
$ caro "find files larger than 100MB"
  find . -type f -size +100M -ls

Step 2: Review the list (don't automate yet)
[User reviews output]

Step 3: Compress specific files
$ caro "compress file.txt with gzip"
  gzip -k file.txt

The -k flag keeps the original file as a backup.

Would you like to proceed with step 1?
```

### Configuration Customization

Help users customize Caro behavior:

```bash
# Set configuration values using caro config set
caro config set safety moderate          # strict, moderate, or permissive
caro config set backend embedded         # embedded (default) or ollama
caro config set shell bash               # bash, zsh, fish, sh, powershell, cmd
caro config set model-name codellama:7b  # model name for the backend
caro config set telemetry.enabled false  # enable/disable telemetry
caro config set safety.level strict      # safety level (strict/moderate/permissive)
caro config set output.format json       # output format (json/yaml/plain)

# View current configuration
caro config show
caro config get safety
caro config get backend
```

## Educational Notes

### Why POSIX Compliance Matters

```
POSIX-compliant commands work across:
- ✓ bash (most common)
- ✓ zsh (macOS default since Catalina)
- ✓ sh (minimal, universal)
- ✓ dash (Debian/Ubuntu default /bin/sh)
- ✓ ksh (Korn shell)

Non-POSIX features that break portability:
- Bash arrays: myarray=(one two three)
- Process substitution: <(command)
- Extended globbing: shopt -s extglob
- [[ ]] conditional (use [ ] instead)

Caro generates POSIX-compliant commands by default for maximum portability.
```

### Risk Assessment Philosophy

```
Caro uses a 4-tier risk model:

1. Safe (Green): Read-only, no system impact
   - Listings, searches, queries
   - Safe to execute without review

2. Moderate (Yellow): Limited modifications
   - File edits, package updates
   - Review recommended

3. High (Orange): Significant changes
   - Deletions, mass operations
   - Careful review required

4. Critical (Red): Dangerous operations
   - System destruction, privilege escalation
   - Blocked or requires explicit override

This helps users develop safety intuition over time.
```

## Troubleshooting

### Caro Not Found

```bash
# Add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Command Generation Fails

```bash
# Check Caro status
caro --version

# Enable verbose mode for debugging
caro --verbose "your prompt here"

# Try different backend
caro "your prompt here"   # Use embedded backend (fastest)
caro --backend ollama "your prompt here"   # Use local Ollama
caro --backend embedded "your prompt here" # Use embedded model
```

### Claude Backend Issues

```bash
# Check if API key is set
echo $ANTHROPIC_API_KEY

# Set API key if missing
export ANTHROPIC_API_KEY="sk-ant-your-key-here"

# When running in Claude Code, the API key is provided automatically
```

### Safety Validation Too Strict

```toml
# Adjust in ~/.config/caro/config.toml
[safety]
level = "permissive"  # or "moderate"
```

## Best Practices Summary

1. **Always check if Caro is installed** before suggesting usage
2. **Explain risk levels** to educate users about command safety
3. **Highlight POSIX compliance** for portable, reliable commands
4. **Suggest preview before execution** for destructive operations
5. **Break complex tasks into steps** for safety and clarity
6. **Provide safe alternatives** to dangerous commands
7. **Educate, don't just execute** - help users learn command safety

## Resources

- **Caro Repository**: https://github.com/wildcard/caro
- **Website**: https://caro.sh
- **Installation Guide**: https://caro.sh/install
- **Safety Patterns**: See `references/safety-patterns.md`
- **POSIX Reference**: See `references/posix-compliance.md`

## Remember

The goal is not just to generate commands, but to help users:
- ✅ Understand what commands do
- ✅ Recognize dangerous patterns
- ✅ Develop safety-first habits
- ✅ Write portable, POSIX-compliant scripts
- ✅ Make informed decisions about command execution

Always prioritize user education and safety over convenience.
