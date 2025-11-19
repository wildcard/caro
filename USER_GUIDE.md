# cmdai User Guide

Welcome to cmdai! This guide will help you install, configure, and use cmdai to convert natural language into safe shell commands using local AI models.

**What is cmdai?** cmdai is a command-line tool that understands what you want to do and generates the right shell command for you. Instead of searching Stack Overflow or remembering complex command syntax, just describe your task in plain English.

**Safety First**: cmdai includes built-in safety checks to prevent dangerous operations. It will warn you about risky commands and ask for confirmation before proceeding.

---

## Table of Contents

- [Installation](#installation)
  - [macOS](#macos)
  - [Linux](#linux)
  - [Windows](#windows)
  - [Building from Source](#building-from-source)
- [Quick Start](#quick-start)
- [Basic Usage](#basic-usage)
- [Configuration](#configuration)
- [Advanced Usage](#advanced-usage)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

---

## Installation

### macOS

#### Option 1: Homebrew (Recommended)

```bash
# Add the cmdai tap
brew tap wildcard/tap

# Install cmdai
brew install cmdai

# Verify installation
cmdai --version
```

#### Option 2: Manual Installation

1. Download the latest release for macOS:
   ```bash
   curl -L -o cmdai https://github.com/wildcard/cmdai/releases/latest/download/cmdai-macos-arm64
   # For Intel Macs, use: cmdai-macos-x64
   ```

2. Make it executable:
   ```bash
   chmod +x cmdai
   ```

3. Move to a directory in your PATH:
   ```bash
   sudo mv cmdai /usr/local/bin/
   ```

4. Verify installation:
   ```bash
   cmdai --version
   ```

**Note for macOS users**: On first run, macOS Gatekeeper may block cmdai. See [Troubleshooting](#gatekeeper-warning-macos) for solutions.

---

### Linux

#### Ubuntu/Debian

1. Download the binary:
   ```bash
   curl -L -o cmdai https://github.com/wildcard/cmdai/releases/latest/download/cmdai-linux-x64
   ```

2. Make it executable:
   ```bash
   chmod +x cmdai
   ```

3. Move to your local bin directory:
   ```bash
   sudo mv cmdai /usr/local/bin/
   # Or for single-user installation:
   mkdir -p ~/.local/bin
   mv cmdai ~/.local/bin/
   # Make sure ~/.local/bin is in your PATH
   ```

4. Verify installation:
   ```bash
   cmdai --version
   ```

#### Fedora/RHEL/CentOS

Same as Ubuntu/Debian above. cmdai is a self-contained binary with no system dependencies.

#### Arch Linux

```bash
# Using yay (AUR helper)
yay -S cmdai

# Or manually download the binary as shown in Ubuntu section
```

---

### Windows

#### Option 1: PowerShell Installation (Recommended)

1. Open PowerShell as Administrator

2. Download and install:
   ```powershell
   # Download the Windows binary
   Invoke-WebRequest -Uri "https://github.com/wildcard/cmdai/releases/latest/download/cmdai-windows-x64.exe" -OutFile "$env:USERPROFILE\cmdai.exe"

   # Add to PATH (for current user)
   $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
   [Environment]::SetEnvironmentVariable("Path", "$userPath;$env:USERPROFILE", "User")

   # Refresh current session
   $env:Path = [System.Environment]::GetEnvironmentVariable("Path","User")
   ```

3. Verify installation:
   ```powershell
   cmdai --version
   ```

#### Option 2: Manual Installation

1. Download `cmdai-windows-x64.exe` from the [releases page](https://github.com/wildcard/cmdai/releases/latest)

2. Rename it to `cmdai.exe`

3. Move it to a folder in your PATH (e.g., `C:\Program Files\cmdai\`)

4. Add that folder to your system PATH:
   - Search for "Environment Variables" in Windows Settings
   - Edit the PATH variable
   - Add the folder containing `cmdai.exe`

---

### Building from Source

If you prefer to build cmdai yourself:

**Prerequisites:**
- Rust 1.75 or later ([install from rustup.rs](https://rustup.rs))
- Git

**Build steps:**
```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Build the release binary
cargo build --release

# The binary will be at: target/release/cmdai
# Copy it to your PATH
sudo cp target/release/cmdai /usr/local/bin/

# Verify installation
cmdai --version
```

---

## Quick Start

Get up and running with cmdai in 5 minutes!

### Step 1: First-Time Setup

The first time you run cmdai, it will download the AI model (approximately 1.1 GB). This only happens once.

```bash
# Run version command to trigger setup
cmdai --version
```

You'll see output like:
```
Downloading model for first-time setup...
Progress: [=============>              ] 45% (500 MB / 1.1 GB)
ETA: 2 minutes

Model downloaded successfully!
cmdai version 1.0.0
```

**Note**: The model is stored in your cache directory:
- macOS/Linux: `~/.cache/cmdai/models/`
- Windows: `%LOCALAPPDATA%\cmdai\cache\models\`

### Step 2: Generate Your First Command

Let's create a simple command to list files:

```bash
cmdai "list all files in the current directory"
```

Output:
```
Command:
  ls -la

Explanation:
  Lists all files including hidden ones (-a) in long format (-l)

Execute this command? (y/N)
```

### Step 3: Execute the Command

Type `y` and press Enter to run the command, or just press Enter to cancel.

For automatic execution without confirmation prompts:
```bash
cmdai --confirm "list all files in the current directory"
# or shorthand:
cmdai -y "list all files in the current directory"
```

### Step 4: Understanding Safety Validation

cmdai protects you from dangerous commands. Try this:

```bash
cmdai "delete everything in my home folder"
```

Output:
```
⚠️  Warning: This command performs potentially dangerous operations

Blocked: Command contains dangerous pattern: 'rm -rf ~'

This operation could result in data loss and has been blocked for safety.
If you absolutely must perform this operation, adjust the safety level
with --safety permissive (use with extreme caution).
```

**That's it!** You're ready to use cmdai. Read on for more features and configuration options.

---

## Basic Usage

### Command Syntax

```bash
cmdai [OPTIONS] "<your natural language prompt>"
```

**Important**: Always put your prompt in quotes to handle special characters and spaces.

### Common Examples

#### File Operations

```bash
# Find files
cmdai "find all PDF files larger than 10MB in Downloads"

# Compress files
cmdai "create a zip archive of all images in the current directory"

# Change permissions
cmdai "make this script executable"

# Copy files
cmdai "copy all text files from src to backup folder"
```

#### Git Operations

```bash
# Create branches
cmdai "create a new git branch called feature-login"

# View history
cmdai "show git commits from the last week"

# Check status
cmdai "show which files have been modified but not committed"

# Undo changes
cmdai "discard all uncommitted changes"
```

#### System Tasks

```bash
# Disk usage
cmdai "show disk space usage by directory"

# Process management
cmdai "find and kill the process using port 8080"

# Network
cmdai "show my IP address"

# System info
cmdai "show CPU and memory usage"
```

#### Docker

```bash
# Container management
cmdai "stop all running docker containers"

# Cleanup
cmdai "remove all unused docker images"

# Logs
cmdai "show logs for the web container"
```

#### Text Processing

```bash
# Search in files
cmdai "find all occurrences of TODO in Python files"

# Replace text
cmdai "replace all instances of oldname with newname in this file"

# Count lines
cmdai "count how many lines of code are in this project"
```

### Available Flags

| Flag | Description | Example |
|------|-------------|---------|
| `-s, --shell <SHELL>` | Target shell (bash, zsh, fish, sh, powershell, cmd) | `cmdai --shell zsh "list files"` |
| `--safety <LEVEL>` | Safety level (strict, moderate, permissive) | `cmdai --safety moderate "delete logs"` |
| `-o, --output <FORMAT>` | Output format (json, yaml, plain) | `cmdai --output json "show disk usage"` |
| `-y, --confirm` | Auto-confirm without prompting | `cmdai -y "list files"` |
| `-v, --verbose` | Show detailed debug information | `cmdai -v "find files"` |
| `-c, --config <FILE>` | Use custom configuration file | `cmdai -c ~/.cmdai-custom.toml "list files"` |
| `--show-config` | Display current configuration | `cmdai --show-config` |
| `--version` | Show version information | `cmdai --version` |
| `--help` | Display help message | `cmdai --help` |

### Output Formats

#### Plain (Default)

Human-readable output with colors and formatting:

```bash
cmdai "show disk usage"
```

Output:
```
Command:
  df -h

Explanation:
  Shows disk space usage in human-readable format (-h flag)
```

#### JSON

Machine-readable format for scripting:

```bash
cmdai --output json "show disk usage"
```

Output:
```json
{
  "generated_command": "df -h",
  "explanation": "Shows disk space usage in human-readable format",
  "safety_level": "Safe",
  "requires_confirmation": false,
  "alternatives": [
    "du -sh *",
    "df -h --total"
  ]
}
```

#### YAML

Structured format, easier to read than JSON:

```bash
cmdai --output yaml "show disk usage"
```

Output:
```yaml
generated_command: df -h
explanation: Shows disk space usage in human-readable format
safety_level: Safe
requires_confirmation: false
alternatives:
  - du -sh *
  - df -h --total
```

### Safety Levels

cmdai includes comprehensive safety validation to protect against dangerous operations.

#### Strict (Default)

Blocks dangerous commands entirely:
- Prevents: `rm -rf /`, fork bombs, `mkfs`, privilege escalation
- Requires confirmation for: File deletions, system modifications

```bash
cmdai "delete all files recursively"
# → Blocked with explanation
```

#### Moderate

Allows dangerous commands with explicit confirmation:
- Prevents: Only critically dangerous operations
- Requires confirmation for: All potentially destructive commands

```bash
cmdai --safety moderate "delete all log files"
# → Shows command and asks for confirmation
```

#### Permissive

Minimal safety checks (use with caution):
- Prevents: Only the most dangerous operations (e.g., `rm -rf /`)
- Requires confirmation for: Only critical operations

```bash
cmdai --safety permissive "clean up temporary files"
# → Generates command with minimal warnings
```

**Recommendation**: Keep the default `strict` level unless you have a specific reason to change it.

---

## Configuration

cmdai can be configured through a TOML configuration file.

### Configuration File Location

cmdai looks for its configuration file in these locations:

- **Linux/macOS**: `~/.config/cmdai/config.toml`
- **Windows**: `%APPDATA%\cmdai\config.toml`

### Creating a Configuration File

The configuration file is optional. cmdai will use sensible defaults if no config file exists.

To create a configuration file:

1. Create the config directory:
   ```bash
   # Linux/macOS
   mkdir -p ~/.config/cmdai

   # Windows (PowerShell)
   New-Item -ItemType Directory -Force -Path "$env:APPDATA\cmdai"
   ```

2. Create the config file with your preferred settings:
   ```bash
   # Linux/macOS
   nano ~/.config/cmdai/config.toml

   # Windows
   notepad %APPDATA%\cmdai\config.toml
   ```

### Configuration Options

Here's a complete example configuration file with all available options:

```toml
# cmdai configuration file

[general]
# Default shell type for generated commands
# Options: bash, zsh, fish, sh, powershell, cmd
default_shell = "bash"

# Safety level for command validation
# Options: strict, moderate, permissive
safety_level = "strict"

# Default AI model to use (optional)
# If not specified, uses the embedded model
default_model = "qwen2.5-coder-1.5b-instruct"

[logging]
# Logging verbosity
# Options: error, warn, info, debug, trace
log_level = "info"

# Number of days to keep log files before rotation
log_rotation_days = 30

[cache]
# Maximum cache size in gigabytes
# The AI model cache will be cleaned if it exceeds this size
max_size_gb = 5

[backend]
# Backend preference: embedded, ollama, or vllm
# Default: embedded (runs locally without external dependencies)
primary = "embedded"

# Enable automatic fallback to other backends if primary fails
enable_fallback = true

# Ollama backend configuration (optional)
[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"

# vLLM backend configuration (optional)
[backend.vllm]
base_url = "http://localhost:8000"
model_name = "codellama/CodeLlama-7b-hf"
# api_key = "your-api-key-here"  # Optional
```

### Viewing Current Configuration

To see your current configuration:

```bash
cmdai --show-config
```

Output:
```
Configuration file: /home/user/.config/cmdai/config.toml
Configuration exists: true

Current configuration:
  Default shell: Bash
  Safety level: Strict
  Log level: Info
  Cache max size: 5 GB
  Log rotation: 30 days
  Default model: qwen2.5-coder-1.5b-instruct
```

### Environment Variables

You can also configure cmdai using environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `CMDAI_SHELL` | Override default shell | `export CMDAI_SHELL=zsh` |
| `CMDAI_SAFETY` | Override safety level | `export CMDAI_SAFETY=moderate` |
| `CMDAI_LOG_LEVEL` | Set logging level | `export CMDAI_LOG_LEVEL=debug` |
| `CMDAI_CONFIG` | Custom config file path | `export CMDAI_CONFIG=~/my-config.toml` |
| `RUST_LOG` | Detailed logging (for debugging) | `export RUST_LOG=cmdai=debug` |

Environment variables take precedence over config file settings.

### Configuration Precedence

Settings are applied in this order (later sources override earlier ones):

1. Default values (built into cmdai)
2. Configuration file (`~/.config/cmdai/config.toml`)
3. Environment variables (`CMDAI_*`)
4. Command-line flags (`--shell`, `--safety`, etc.)

---

## Advanced Usage

### Using Different Backends

cmdai supports multiple backend options for AI inference:

#### Embedded Backend (Default)

The embedded backend runs completely offline using a built-in AI model:
- **Pros**: No external dependencies, complete privacy, works offline
- **Cons**: Slower than GPU-accelerated options
- **Model**: Qwen2.5-Coder-1.5B-Instruct (1.1 GB)

No configuration needed - this is the default.

#### Ollama Backend

Use Ollama for local inference with GPU acceleration:

1. Install Ollama from [ollama.ai](https://ollama.ai)

2. Pull a code generation model:
   ```bash
   ollama pull codellama:7b
   ```

3. Configure cmdai to use Ollama:
   ```toml
   # ~/.config/cmdai/config.toml
   [backend]
   primary = "ollama"

   [backend.ollama]
   base_url = "http://localhost:11434"
   model_name = "codellama:7b"
   ```

4. Use cmdai as normal - it will now use Ollama:
   ```bash
   cmdai "list all files"
   ```

#### vLLM Backend

Use vLLM for high-performance remote inference:

1. Set up a vLLM server (see [vLLM docs](https://docs.vllm.ai))

2. Configure cmdai:
   ```toml
   # ~/.config/cmdai/config.toml
   [backend]
   primary = "vllm"

   [backend.vllm]
   base_url = "http://your-vllm-server:8000"
   model_name = "codellama/CodeLlama-7b-hf"
   api_key = "your-api-key"  # Optional
   ```

### Scripting with cmdai

cmdai's JSON output makes it perfect for automation and scripting.

#### Execute Commands Programmatically

```bash
#!/bin/bash
# generate-and-run.sh

# Generate command and extract it from JSON
COMMAND=$(cmdai --output json "$1" | jq -r '.generated_command')

# Check if command generation succeeded
if [ $? -eq 0 ] && [ -n "$COMMAND" ]; then
    echo "Generated command: $COMMAND"
    echo "Executing..."
    eval "$COMMAND"
else
    echo "Failed to generate command"
    exit 1
fi
```

Usage:
```bash
./generate-and-run.sh "show disk usage"
```

#### Batch Command Generation

```bash
#!/bin/bash
# batch-generate.sh

# Read prompts from file, generate commands
while IFS= read -r prompt; do
    echo "Prompt: $prompt"
    cmdai --output json "$prompt" | jq -r '.generated_command'
    echo ""
done < prompts.txt
```

#### Integration with CI/CD

```yaml
# .github/workflows/deploy.yml
name: Deploy

on: [push]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Install cmdai
        run: |
          curl -L -o cmdai https://github.com/wildcard/cmdai/releases/latest/download/cmdai-linux-x64
          chmod +x cmdai
          sudo mv cmdai /usr/local/bin/

      - name: Generate deployment command
        run: |
          cmdai --output json "deploy to production with zero downtime" > deploy-cmd.json
          cat deploy-cmd.json

      - name: Review generated command
        run: |
          jq -r '.generated_command' deploy-cmd.json
```

### Shell-Specific Features

#### Bash/Zsh

```bash
# Create an alias for quick access
echo 'alias cmd="cmdai -y"' >> ~/.bashrc
source ~/.bashrc

# Now you can use:
cmd "find large files"
```

#### Fish Shell

```fish
# Create an alias
echo 'alias cmd="cmdai -y"' >> ~/.config/fish/config.fish

# Create a function for interactive mode
function ai
    cmdai $argv
end
funcsave ai
```

#### PowerShell

```powershell
# Add to your PowerShell profile
function cmd {
    cmdai -y $args
}

# Create an alias
Set-Alias ai cmdai

# Add to profile permanently
Add-Content $PROFILE "function cmd { cmdai -y `$args }"
```

### Custom Safety Patterns

You can add custom dangerous patterns to the safety validator:

```toml
# ~/.config/cmdai/config.toml
[safety]
# Add custom patterns to block
custom_dangerous_patterns = [
    "production",      # Warn on commands mentioning production
    "database drop",   # Block database drops
    "DELETE FROM",     # Block SQL deletes
]

# Add custom safe patterns (to allow otherwise blocked commands)
custom_safe_patterns = [
    "rm -rf ./tmp",   # Allow deleting tmp directory
]
```

### Performance Tuning

#### Reduce Model Download Size

If the 1.1 GB model is too large, you can use a smaller quantized version:

```toml
[general]
default_model = "qwen2.5-coder-1.5b-instruct-q4"  # ~600 MB
```

#### Improve Inference Speed

1. **Use GPU acceleration** (if available):
   - Switch to Ollama backend with GPU support
   - Or wait for MLX support (Apple Silicon only)

2. **Reduce model size**:
   - Smaller models = faster inference
   - Trade-off: Slightly lower accuracy

3. **Enable caching** (planned feature):
   - Frequently used commands will be cached

---

## Troubleshooting

### Common Issues

#### "Model not found" Error

**Problem**:
```
Error: Model not found in cache
Failed to load model: qwen2.5-coder-1.5b-instruct
```

**Solution**:
1. Ensure you have internet connectivity
2. Trigger the model download:
   ```bash
   cmdai --version
   ```
3. If the download fails, check your proxy settings:
   ```bash
   export HTTPS_PROXY=http://your-proxy:port
   cmdai --version
   ```
4. For manual download issues, clear the cache and retry:
   ```bash
   # Linux/macOS
   rm -rf ~/.cache/cmdai/models
   cmdai --version

   # Windows
   rmdir /s %LOCALAPPDATA%\cmdai\cache\models
   cmdai --version
   ```

#### Slow Performance

**Problem**: Command generation takes more than 10 seconds.

**Solutions**:

1. **Check system resources**:
   ```bash
   # Linux/macOS
   top

   # Windows
   taskmgr
   ```
   Make sure you have at least 2 GB of free RAM.

2. **Use a faster backend**:
   ```bash
   # Install and use Ollama
   ollama pull codellama:7b
   ```
   Update config to use Ollama (see [Using Different Backends](#using-different-backends))

3. **Try a smaller model** (if using Ollama):
   ```bash
   ollama pull codellama:3b  # Smaller, faster model
   ```

4. **Close other applications** to free up system resources

#### Network Errors During Download

**Problem**:
```
Error: Failed to download model
Network connection failed
```

**Solutions**:

1. **Check internet connection**:
   ```bash
   ping huggingface.co
   ```

2. **Configure proxy** (if behind corporate firewall):
   ```bash
   export HTTPS_PROXY=http://proxy.company.com:8080
   export HTTP_PROXY=http://proxy.company.com:8080
   ```

3. **Disable VPN temporarily** (some VPNs block Hugging Face)

4. **Use mobile hotspot** as a workaround

5. **Manual download** (last resort):
   ```bash
   # Download from Hugging Face manually
   wget https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf

   # Place in cache directory
   mkdir -p ~/.cache/cmdai/models
   mv qwen2.5-coder-1.5b-instruct-q4_k_m.gguf ~/.cache/cmdai/models/
   ```

#### Permission Denied Errors

**Problem**:
```
Error: Permission denied
Failed to write to /home/user/.cache/cmdai
```

**Solutions**:

1. **Fix cache directory permissions**:
   ```bash
   # Linux/macOS
   mkdir -p ~/.cache/cmdai
   chmod 755 ~/.cache/cmdai
   ```

2. **Fix config directory permissions**:
   ```bash
   mkdir -p ~/.config/cmdai
   chmod 755 ~/.config/cmdai
   ```

3. **Check disk space**:
   ```bash
   df -h
   ```
   Ensure you have at least 2 GB free.

4. **Run with explicit permissions** (last resort):
   ```bash
   # Don't use sudo for normal operation!
   # Only if cmdai was accidentally installed with wrong permissions
   sudo chown -R $USER:$USER ~/.cache/cmdai
   sudo chown -R $USER:$USER ~/.config/cmdai
   ```

#### Gatekeeper Warning (macOS)

**Problem**:
```
"cmdai" cannot be opened because the developer cannot be verified
```

**Solutions**:

**Option 1**: Remove quarantine attribute:
```bash
xattr -d com.apple.quarantine /usr/local/bin/cmdai
```

**Option 2**: Allow in System Preferences:
1. Go to System Preferences > Security & Privacy
2. Click "Allow Anyway" next to the blocked cmdai message
3. Run cmdai again and click "Open"

**Option 3**: Bypass Gatekeeper (not recommended):
```bash
sudo spctl --master-disable
# Run cmdai
sudo spctl --master-enable  # Re-enable after
```

#### Command Not Found

**Problem**:
```bash
cmdai: command not found
```

**Solutions**:

1. **Check if cmdai is in PATH**:
   ```bash
   which cmdai
   ```
   If no output, cmdai isn't in your PATH.

2. **Add to PATH** (Linux/macOS):
   ```bash
   # If you installed to ~/.local/bin
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc

   # For zsh users
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

3. **Add to PATH** (Windows):
   ```powershell
   $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
   [Environment]::SetEnvironmentVariable("Path", "$userPath;C:\path\to\cmdai", "User")
   ```

4. **Use absolute path**:
   ```bash
   /usr/local/bin/cmdai "list files"
   ```

#### Configuration Errors

**Problem**:
```
Error: Invalid configuration
Failed to parse config.toml
```

**Solutions**:

1. **Validate TOML syntax**:
   ```bash
   # Use an online TOML validator or
   cat ~/.config/cmdai/config.toml
   ```
   Check for:
   - Missing quotes around strings
   - Unclosed brackets
   - Invalid section names

2. **Reset to defaults**:
   ```bash
   # Backup your config
   mv ~/.config/cmdai/config.toml ~/.config/cmdai/config.toml.backup

   # cmdai will use defaults
   cmdai --version
   ```

3. **Use --show-config to debug**:
   ```bash
   cmdai --show-config
   ```

### Platform-Specific Issues

#### Windows PowerShell Execution Policy

**Problem**:
```
cmdai.exe : File cannot be loaded because running scripts is disabled
```

**Solution**:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

#### Linux: Missing GLIBC

**Problem**:
```
./cmdai: /lib/x86_64-linux-gnu/libc.so.6: version 'GLIBC_2.29' not found
```

**Solution**: Update your system or build from source:
```bash
# Ubuntu/Debian
sudo apt update && sudo apt upgrade

# Or build from source
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release
```

#### macOS: Rosetta Required (Intel Macs on Apple Silicon)

**Problem**:
```
Bad CPU type in executable
```

**Solution**: Download the correct architecture:
- Apple Silicon (M1/M2/M3): `cmdai-macos-arm64`
- Intel Macs: `cmdai-macos-x64`

### Getting Help

If you encounter an issue not covered here:

1. **Enable verbose logging**:
   ```bash
   cmdai --verbose "your prompt" 2>&1 | tee cmdai-debug.log
   ```

2. **Check existing issues**:
   Visit [GitHub Issues](https://github.com/wildcard/cmdai/issues)

3. **Create a new issue**:
   Include:
   - Operating system and version
   - cmdai version (`cmdai --version`)
   - Full error message
   - Steps to reproduce
   - Debug log (from verbose mode)

4. **Community support**:
   - GitHub Discussions: [General category]
   - Discord: #cmdai-support (if available)

---

## FAQ

### General Questions

**Q: Is my data sent to the cloud?**

A: No! cmdai runs 100% locally using an embedded AI model. Your prompts, generated commands, and file system information never leave your computer. This is why the first-time download is required - the entire model runs on your machine.

**Q: How much disk space does cmdai need?**

A: Approximately 1.5-2 GB total:
- Binary: ~50 MB
- AI model: ~1.1 GB
- Cache/logs: ~100-300 MB

**Q: Can cmdai access my files?**

A: cmdai itself only reads its configuration and cache directories. However, the commands it generates may access files based on what you ask for. Always review commands before execution.

**Q: Does cmdai require an internet connection?**

A: Only for the initial model download (~1.1 GB). After that, cmdai works completely offline.

**Q: Can I use cmdai in scripts or automation?**

A: Yes! Use `--output json` for machine-readable output and `--confirm` to skip prompts. See [Scripting with cmdai](#scripting-with-cmdai).

### Usage Questions

**Q: How do I execute the generated command?**

A: Type `y` when prompted, or use the `--confirm` flag:
```bash
cmdai --confirm "list files"
# or shorthand:
cmdai -y "list files"
```

**Q: Can I use cmdai with PowerShell on Windows?**

A: Yes! cmdai auto-detects your shell, or you can specify:
```powershell
cmdai --shell powershell "list files"
```

**Q: What languages does cmdai understand?**

A: cmdai works best with English prompts, but the model may understand other languages with varying accuracy.

**Q: Can cmdai generate complex multi-line scripts?**

A: cmdai is designed for single commands. For multi-step operations, generate commands one at a time or combine them with `&&`:
```bash
cmdai "backup database and then restart service"
# Generates: pg_dump mydb > backup.sql && systemctl restart myapp
```

**Q: Why was my command blocked?**

A: cmdai includes safety validation to prevent dangerous operations like:
- `rm -rf /` (deleting root directory)
- Fork bombs
- Disk formatting
- Unintended privilege escalation

You can adjust safety levels with `--safety moderate` or `--safety permissive`, but use caution.

### Technical Questions

**Q: Which AI model does cmdai use?**

A: By default, cmdai uses Qwen2.5-Coder-1.5B-Instruct, a code-specialized language model quantized to ~1.1 GB. You can use other models via Ollama or vLLM backends.

**Q: Why is the first command slow?**

A: The AI model needs to load into memory (1-2 seconds). Subsequent commands are faster. For best performance, consider using the Ollama backend with GPU acceleration.

**Q: Can I use cmdai with my own LLM?**

A: Yes! Configure the vLLM or Ollama backend to point to your model:
```toml
[backend]
primary = "ollama"

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "your-model-name"
```

**Q: Does cmdai support GPU acceleration?**

A: Not yet for the embedded backend. However, you can use the Ollama backend (with GPU support) or wait for MLX backend support (Apple Silicon only).

**Q: Where are logs stored?**

A:
- Linux/macOS: `~/.cache/cmdai/logs/`
- Windows: `%LOCALAPPDATA%\cmdai\cache\logs\`

**Q: How do I update cmdai?**

A:
```bash
# Homebrew (macOS)
brew upgrade cmdai

# Manual installation - download latest release
curl -L -o cmdai https://github.com/wildcard/cmdai/releases/latest/download/cmdai-<platform>
chmod +x cmdai
sudo mv cmdai /usr/local/bin/
```

**Q: How do I uninstall cmdai?**

A:
```bash
# Remove binary
sudo rm /usr/local/bin/cmdai

# Remove cache and config
rm -rf ~/.cache/cmdai
rm -rf ~/.config/cmdai

# Homebrew users
brew uninstall cmdai
```

### Safety and Privacy

**Q: Is cmdai safe to use?**

A: cmdai includes multiple safety layers:
1. Built-in dangerous pattern detection
2. Command validation before execution
3. User confirmation for risky operations
4. Configurable safety levels

However, always review generated commands before execution.

**Q: What data does cmdai collect?**

A: cmdai collects no telemetry or usage data. Everything runs locally.

**Q: Can cmdai execute commands without my permission?**

A: No. cmdai always asks for confirmation unless you explicitly use `--confirm` flag. Even then, dangerous commands require confirmation in strict/moderate safety modes.

**Q: What if cmdai generates a wrong or dangerous command?**

A:
1. Always review commands before executing
2. Use the default "strict" safety level
3. If you notice a dangerous pattern, report it on GitHub Issues
4. You can add custom safety patterns in your config

---

## Additional Resources

- **GitHub Repository**: [github.com/wildcard/cmdai](https://github.com/wildcard/cmdai)
- **Issue Tracker**: [GitHub Issues](https://github.com/wildcard/cmdai/issues)
- **Discussions**: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- **Contributing Guide**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)

---

## Getting Support

Need help? Here's how to get support:

1. **Check this guide** - Most common issues are covered in [Troubleshooting](#troubleshooting)
2. **Search existing issues** - Someone may have had the same problem
3. **Ask in Discussions** - For general questions and tips
4. **Create an issue** - For bugs or feature requests
5. **Enable verbose mode** - Include debug logs when reporting issues:
   ```bash
   cmdai --verbose "your prompt" 2>&1 | tee debug.log
   ```

---

**Thank you for using cmdai!** We hope this tool makes your command-line experience more productive and enjoyable. If you have suggestions for improving this guide, please open an issue or pull request.

**Happy command generating! 🚀**
