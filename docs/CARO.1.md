# CARO(1) - Caro User Manual

## NAME

**caro**, **cmdai** - convert natural language to safe POSIX shell commands using local LLMs

## SYNOPSIS

```
caro [OPTIONS] <PROMPT>
cmdai [OPTIONS] <PROMPT>
```

## DESCRIPTION

**Caro** (also known as **cmdai**) is a command-line tool that transforms natural language descriptions into safe, executable shell commands using local Large Language Models (LLMs). Built with Rust for performance and safety, Caro provides intelligent platform-aware command generation with comprehensive safety validation.

Caro uses a sophisticated **2-iteration agentic loop** that detects your operating system, architecture, shell environment, and available commands to generate appropriate POSIX-compliant shell commands. The tool prioritizes safety through 52+ pre-compiled dangerous command patterns and a risk assessment system that prevents accidental execution of destructive operations.

### Design Philosophy

**Safety-First**
:   Every generated command undergoes comprehensive validation against dangerous patterns before presentation to the user. Critical operations require explicit confirmation.

**Platform-Aware**
:   Caro automatically detects GNU vs BSD utilities, shell-specific syntax, and platform constraints to generate commands that work on your specific system.

**Self-Contained**
:   Single-binary distribution with embedded local inference capability. No external API keys or cloud services required for core functionality.

**Extensible**
:   Modular backend architecture supporting MLX (Apple Silicon), CPU inference, Ollama, and vLLM for flexible deployment scenarios.

## OPTIONS

### General Options

**PROMPT**
:   Natural language description of the desired shell command. This is the only required argument.

**-h**, **--help**
:   Display help information and exit.

**-V**, **--version**
:   Display version information and exit.

**--show-config**
:   Display current configuration settings and exit. Useful for debugging configuration issues.

### Shell and Output Options

**-s**, **--shell** *SHELL*
:   Specify the target shell for command generation. Supported values:
    - **bash** - Bourne Again Shell (default on Linux)
    - **zsh** - Z Shell (default on macOS)
    - **fish** - Friendly Interactive Shell
    - **sh** - POSIX shell
    - **powershell** - Windows PowerShell
    - **cmd** - Windows Command Prompt

    If not specified, Caro auto-detects the current shell from the **SHELL** environment variable.

**-o**, **--output** *FORMAT*
:   Set output format for command results. Supported values:
    - **plain** - Human-readable colored output (default)
    - **json** - Machine-readable JSON format with full metadata
    - **yaml** - Structured YAML format

    JSON and YAML formats include additional metadata such as confidence scores, risk levels, alternative commands, and timing information.

### Safety and Execution Options

**--safety** *LEVEL*
:   Set the safety validation level. Controls how strictly dangerous commands are handled:
    - **strict** - Block high and critical risk commands; confirm moderate risk
    - **moderate** - Block critical risk commands; confirm high risk (default)
    - **permissive** - Allow all commands with warnings; user confirmation available

    See **SAFETY LEVELS** section for detailed behavior.

**-y**, **--confirm**
:   Auto-confirm dangerous commands without interactive prompting. Use with caution. This flag is useful for scripting scenarios where user interaction is not possible.

**-x**, **--execute**
:   Execute the generated command immediately after validation and user confirmation. The command runs in a subshell appropriate for your platform.

**-i**, **--interactive**
:   Enable interactive mode with step-by-step confirmation. In this mode, each stage of command generation and validation is presented to the user before proceeding.

**--dry-run**
:   Show the execution plan without actually running the command. Displays what would happen if **--execute** were used.

### Configuration Options

**-c**, **--config** *FILE*
:   Use a custom configuration file instead of the default location. The file must be in TOML format.

**-v**, **--verbose**
:   Enable verbose output with timing information, debug messages, and detailed progress updates.

**--completions** *SHELL*
:   Generate shell completion scripts for the specified shell and print to stdout.
    Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

    Example usage:
    ```bash
    # Generate and install bash completions
    cmdai --completions bash > ~/.local/share/bash-completion/completions/cmdai

    # Generate zsh completions
    cmdai --completions zsh > ~/.local/share/zsh/site-functions/_cmdai

    # Generate fish completions
    cmdai --completions fish > ~/.config/fish/completions/cmdai.fish
    ```

## CONFIGURATION

### Configuration File Location

Caro reads its configuration from a TOML file located at:

| Platform | Location |
|----------|----------|
| macOS    | `~/.config/cmdai/config.toml` |
| Linux    | `~/.config/cmdai/config.toml` |
| Windows  | `%APPDATA%\cmdai\config.toml` |

The configuration file is created automatically on first run with default values.

### Configuration Priority

Settings are applied in the following order (highest priority first):

1. Command-line flags and arguments
2. Environment variables
3. Configuration file
4. Built-in defaults

### Configuration File Format

```toml
# ~/.config/cmdai/config.toml
# Caro Configuration File

[general]
# Safety validation level: strict, moderate, or permissive
safety_level = "moderate"

# Default target shell: bash, zsh, fish, sh, powershell, cmd
default_shell = "bash"

# Default model for embedded backend (optional)
default_model = "qwen2.5"

[logging]
# Log verbosity: debug, info, warn, error
log_level = "info"

# Number of days to retain log files (1-365)
log_rotation_days = 7

[cache]
# Maximum cache size in gigabytes (1-1000)
max_size_gb = 10

[backend]
# Primary inference backend: embedded, ollama, or vllm
primary = "embedded"

# Enable automatic fallback to embedded backend
enable_fallback = true

[backend.ollama]
# Ollama server URL
base_url = "http://localhost:11434"

# Model name for Ollama
model_name = "codellama:7b"

[backend.vllm]
# vLLM server URL
base_url = "http://localhost:8000"

# Model name for vLLM
model_name = "codellama/CodeLlama-7b-hf"

# Optional API key for vLLM authentication
api_key = ""

[safety]
# Enable safety validation
enabled = true

# Require user confirmation for dangerous commands
require_confirmation = true

# Additional dangerous patterns (regular expressions)
custom_patterns = []
```

### Configuration Options Reference

#### General Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `safety_level` | string | `"moderate"` | Default safety validation level |
| `default_shell` | string | auto-detect | Default target shell |
| `default_model` | string | `"qwen2.5"` | Model identifier for embedded backend |

#### Logging Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `log_level` | string | `"info"` | Minimum log level to record |
| `log_rotation_days` | integer | `7` | Days to retain log files |

#### Cache Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_size_gb` | integer | `10` | Maximum model cache size |

#### Backend Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `primary` | string | `"embedded"` | Primary inference backend |
| `enable_fallback` | boolean | `true` | Fall back to embedded on failure |

## SAFETY LEVELS

Caro implements a three-tier safety system to prevent accidental execution of dangerous commands.

### Strict Mode (`--safety strict`)

The most restrictive safety mode, recommended for production systems.

**Behavior:**
- **Critical risk commands:** Blocked entirely
- **High risk commands:** Blocked entirely
- **Moderate risk commands:** Require user confirmation
- **Safe commands:** Execute normally

### Moderate Mode (`--safety moderate`)

The default safety mode, balancing safety with usability.

**Behavior:**
- **Critical risk commands:** Blocked entirely
- **High risk commands:** Require user confirmation
- **Moderate risk commands:** Execute with warning
- **Safe commands:** Execute normally

### Permissive Mode (`--safety permissive`)

The least restrictive mode, for trusted environments.

**Behavior:**
- **Critical risk commands:** Require user confirmation
- **High risk commands:** Execute with warning
- **Moderate risk commands:** Execute with warning
- **Safe commands:** Execute normally

### Risk Level Classifications

#### Critical Risk (Red)

Operations that can cause irreversible system damage:

- Recursive deletion of root: `rm -rf /`
- Home directory destruction: `rm -rf ~`
- Disk formatting: `mkfs.* /dev/*`
- Disk overwriting: `dd if=/dev/zero of=/dev/sd*`
- Fork bombs: `:(){ :|:& };:`
- Privilege escalation with downloads: `curl | sudo bash`
- Reverse shells: `nc -e /bin/bash`

#### High Risk (Orange)

Operations requiring careful consideration:

- System directory modification: `/bin`, `/sbin`, `/usr`, `/etc`
- Root permission changes: `chmod 777 /`
- Privilege escalation: `sudo su`
- Service manipulation: `systemctl stop`, `systemctl disable`
- System file operations with sudo

#### Moderate Risk (Yellow)

Operations that may have unintended side effects:

- Package management with force flags
- Process termination: `kill -9`
- Firewall rule changes
- Permission modifications
- Environment variable changes
- File deletion operations

#### Safe (Green)

Normal operations with no confirmation needed:

- Directory listing and navigation
- File reading and searching
- Text processing
- Archive operations
- Read-only queries

### Dangerous Command Patterns

Caro detects and validates against 52+ dangerous command patterns including:

| Category | Examples |
|----------|----------|
| Filesystem destruction | `rm -rf /`, `rm -rf ~`, `rm -rf /*` |
| Disk operations | `mkfs.*`, `dd if=/dev/zero` |
| Fork bombs | `:(){ :|:& };:` and variants |
| System paths | Operations on `/bin`, `/usr`, `/etc` |
| Privilege escalation | `sudo su`, `sudo -i` |
| Download & execute | `curl \| bash`, `wget \| sh` |
| Network backdoors | `nc -e`, `bash -i >& /dev/tcp/` |
| Windows destructive | `Remove-Item -Recurse C:\`, `format C:` |

## BACKENDS

### Embedded Backend (Default)

The embedded backend provides local inference without external dependencies.

#### MLX Variant (Apple Silicon)

Optimized for Apple M1, M2, M3, and M4 chips using Metal Performance Shaders.

**Requirements:**
- macOS with Apple Silicon
- Xcode (for full GPU acceleration)

**Performance:**
- First inference: < 2 seconds
- Subsequent inferences: < 1 second

#### CPU Variant (Cross-Platform)

Uses the Candle ML framework for cross-platform inference.

**Supported platforms:**
- macOS (Intel and Apple Silicon)
- Linux (x64 and ARM64)
- Windows (x64)

**Performance:**
- First inference: 3-5 seconds
- Subsequent inferences: 2-3 seconds

#### Model Information

| Property | Value |
|----------|-------|
| Model | Qwen2.5-Coder-1.5B-Instruct |
| Size | ~1.5 GB (quantized) |
| Download | Automatic from Hugging Face |
| Caching | Local cache with offline support |

### Ollama Backend

Connect to a local or remote Ollama instance.

**Configuration:**
```toml
[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"
```

**Requirements:**
- Ollama installed and running
- Model pre-pulled: `ollama pull codellama:7b`

### vLLM Backend

Connect to a vLLM server for high-performance inference.

**Configuration:**
```toml
[backend.vllm]
base_url = "http://localhost:8000"
model_name = "codellama/CodeLlama-7b-hf"
api_key = "your-api-key"  # Optional
```

**Requirements:**
- vLLM server running
- Network access to server

### Backend Fallback

When `enable_fallback = true` (default), Caro automatically falls back to the embedded backend if the primary backend is unavailable.

Fallback order:
1. User-specified backend (if any)
2. Configured primary backend
3. Embedded backend (always available)

## EXAMPLES

### Basic Usage

Generate a simple command:

```console
$ caro "list all files in the current directory"

Generated command:
  ls -la

Execute this command? (y/N)
```

### Specifying Target Shell

Generate a zsh-specific command:

```console
$ caro --shell zsh "show last 10 commands from history"

Generated command:
  fc -l -10

Execute this command? (y/N)
```

### JSON Output for Scripting

Get machine-readable output:

```console
$ caro --output json "find all Python files"
{
  "command": "find . -name '*.py' -type f",
  "shell": "bash",
  "risk_level": "safe",
  "confidence": 0.95,
  "explanation": "Recursively finds all Python files in the current directory",
  "alternatives": [
    "fd -e py",
    "locate '*.py'"
  ],
  "timing": {
    "generation_ms": 1234,
    "validation_ms": 12
  }
}
```

### Automatic Execution

Generate and execute a command:

```console
$ caro --execute "show disk usage sorted by size"

Generated command:
  du -sh * | sort -hr | head -20

Execute this command? (y/N) y

4.2G    node_modules
1.1G    target
256M    docs
128M    src
...
```

### Dry Run Mode

Preview execution without running:

```console
$ caro --dry-run --execute "delete all .tmp files"

Generated command:
  find . -name '*.tmp' -type f -delete

Dry run - command would execute:
  Shell: bash
  Working directory: /home/user/project
  Risk level: Moderate
  Would require confirmation: Yes
```

### Verbose Mode

Enable detailed output:

```console
$ caro --verbose "compress all images in photos directory"

[DEBUG] Platform detection: macOS 14.2, arm64, zsh
[DEBUG] Available commands: gzip, tar, zip, xz
[DEBUG] Backend: embedded (MLX)
[INFO] Starting inference...
[DEBUG] Iteration 1: Initial generation (confidence: 0.87)
[DEBUG] Iteration 2: Refinement with command help
[INFO] Generation complete in 1.23s

Generated command:
  find ~/photos -type f \( -name '*.jpg' -o -name '*.png' \) -exec gzip {} \;

Explanation: Finds all JPG and PNG files in the photos directory and
compresses each one using gzip.

Risk level: Safe
Execute this command? (y/N)
```

### Complex Multi-Pipe Commands

Generate sophisticated command pipelines:

```console
$ caro "show top 10 processes by memory usage with their PIDs"

Generated command:
  ps aux --sort=-%mem | head -11 | awk '{print $2, $4, $11}'

Execute this command? (y/N)
```

### Platform-Aware Generation

Caro automatically adapts to your platform:

**On macOS (BSD utilities):**
```console
$ caro "sort files by modification time"

Generated command:
  ls -lt
```

**On Linux (GNU utilities):**
```console
$ caro "sort files by modification time"

Generated command:
  ls -lt --time-style=long-iso
```

### Working with Dangerous Commands

Handle potentially dangerous operations safely:

```console
$ caro "remove all node_modules directories"

Generated command:
  find . -type d -name 'node_modules' -exec rm -rf {} +

Risk level: High
This command will recursively delete directories.
Execute this command? (y/N)
```

### Safety Override

For automation scenarios (use with caution):

```console
$ caro --confirm --safety permissive "clean temporary files"

Generated command:
  find /tmp -type f -mtime +7 -delete

Warning: Auto-confirmed dangerous command
Executing...
```

## AGENTIC LOOP ARCHITECTURE

Caro employs a 2-iteration agentic refinement loop for intelligent command generation.

### Iteration 1: Context-Aware Generation

1. **Platform Detection**
   - Operating system (macOS, Linux, Windows)
   - CPU architecture (x86_64, aarch64)
   - Shell type and version

2. **Command Discovery**
   - Available utilities on the system
   - GNU vs BSD variants
   - Shell-specific features

3. **Initial Generation**
   - Apply platform-specific rules
   - Generate candidate command
   - Calculate confidence score

### Iteration 2: Smart Refinement

Triggered when confidence is below threshold or complex commands detected:

1. **Command Extraction**
   - Parse pipes and chains
   - Identify individual commands

2. **Help Enrichment**
   - Fetch `--help` output for each command
   - Check version compatibility

3. **Refinement**
   - Fix platform-specific issues
   - Optimize command structure
   - Validate syntax

### Example Flow

```
User prompt: "show top 5 processes by CPU"

Iteration 1:
  ├─ Platform: macOS 14.2, arm64, zsh
  ├─ Commands: ps, sort, head (BSD variants)
  └─ Initial: ps aux | sort -k 3 -rn | head -6
      Confidence: 0.78 (below threshold)

Iteration 2:
  ├─ Extract: [ps, sort, head]
  ├─ Help: BSD sort uses -nrk not --numeric-sort
  └─ Refined: ps aux | sort -nrk 3,3 | head -6
      Confidence: 0.94

Final command: ps aux | sort -nrk 3,3 | head -6
```

## ENVIRONMENT

**SHELL**
:   Current shell, used for auto-detection when `--shell` is not specified.

**CMDAI_SAFETY_LEVEL**
:   Override safety level. Values: `strict`, `moderate`, `permissive`.

**CMDAI_DEFAULT_SHELL**
:   Override default shell when not specified.

**CMDAI_LOG_LEVEL**
:   Override log verbosity. Values: `debug`, `info`, `warn`, `error`.

**CMDAI_CONFIG**
:   Path to configuration file.

**HOME**
:   User home directory, used for configuration and cache locations.

**XDG_CONFIG_HOME**
:   Override configuration directory location (Linux/macOS).

**APPDATA**
:   Configuration directory location (Windows).

## FILES

**~/.config/cmdai/config.toml**
:   User configuration file (macOS/Linux).

**%APPDATA%\cmdai\config.toml**
:   User configuration file (Windows).

**~/.cache/huggingface/**
:   Model cache directory for embedded backend.

**~/.config/cmdai/logs/**
:   Log file directory.

## EXIT STATUS

| Code | Description |
|------|-------------|
| **0** | Success |
| **1** | General error |
| **2** | Invalid arguments or configuration |
| **3** | Safety validation blocked command |
| **4** | User cancelled execution |
| **5** | Backend unavailable |
| **6** | Inference timeout |
| **7** | Command execution failed |

## SIGNALS

Caro handles the following signals:

**SIGINT** (Ctrl+C)
:   Gracefully terminate and clean up resources.

**SIGTERM**
:   Gracefully terminate and clean up resources.

## PERFORMANCE

### Startup Time

- Cold start: < 100ms
- Warm start: < 50ms

### Inference Time

| Backend | First Inference | Subsequent |
|---------|-----------------|------------|
| MLX (Apple Silicon) | < 2s | < 1s |
| CPU (Cross-platform) | 3-5s | 2-3s |
| Ollama (Local) | 2-5s | 1-3s |
| vLLM (Remote) | < 1s | < 1s |

### Memory Usage

- Binary size: < 50 MB
- Model cache: ~1.5 GB (Qwen2.5-Coder-1.5B)
- Runtime memory: 100-500 MB (varies by backend)

## COMPATIBILITY

### Operating Systems

| OS | Status | Notes |
|----|--------|-------|
| macOS 12+ | Full support | Apple Silicon optimized |
| Ubuntu 20.04+ | Full support | |
| Debian 11+ | Full support | |
| Fedora 36+ | Full support | |
| Windows 10+ | Full support | |

### Shell Support

| Shell | Status | Detection |
|-------|--------|-----------|
| Bash 4+ | Full support | Automatic |
| Zsh 5+ | Full support | Automatic |
| Fish 3+ | Full support | Automatic |
| POSIX sh | Full support | Automatic |
| PowerShell 7+ | Full support | Automatic |
| cmd.exe | Full support | Automatic |

### CPU Architectures

| Architecture | Status |
|--------------|--------|
| x86_64 (Intel/AMD) | Full support |
| aarch64 (ARM64) | Full support |
| Apple Silicon (M1/M2/M3/M4) | Optimized |

## TROUBLESHOOTING

### Common Issues

**Model download fails**
:   Check internet connectivity. Models are downloaded from Hugging Face on first run. Use a VPN if Hugging Face is blocked in your region.

**Slow inference on Apple Silicon**
:   Ensure Xcode is installed for GPU acceleration: `xcode-select --install`

**Backend unavailable**
:   Check that Ollama or vLLM server is running. Verify the URL in configuration.

**Command blocked unexpectedly**
:   Use `--safety permissive` to see warnings instead of blocks. Review the safety patterns documentation.

### Debug Mode

Enable full debug logging:

```console
$ RUST_LOG=debug caro --verbose "your prompt"
```

### Configuration Verification

Check current configuration:

```console
$ caro --show-config
```

## REPORTING BUGS

Report bugs at: https://github.com/wildcard/cmdai/issues

When reporting, please include:
- Caro version (`caro --version`)
- Operating system and version
- Shell type and version
- Full command used
- Error message or unexpected behavior
- Debug output (`--verbose`)

## AUTHORS

Caro is developed and maintained by the Wildcard team.

**Website:** https://caro.sh

**Repository:** https://github.com/wildcard/cmdai

## COPYRIGHT

Copyright (C) 2024-2025 Wildcard

Caro is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).

This is free software: you are free to change and redistribute it. There is NO WARRANTY, to the extent permitted by law.

See the LICENSE file or https://www.gnu.org/licenses/agpl-3.0.html for full license terms.

## SEE ALSO

**bash**(1), **zsh**(1), **fish**(1), **sh**(1)

**Ollama:** https://ollama.ai

**vLLM:** https://github.com/vllm-project/vllm

**MLX:** https://github.com/ml-explore/mlx

**Hugging Face:** https://huggingface.co

---

*Caro User Manual* | *Version 0.1.0* | *December 2024*
