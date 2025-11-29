# Post-Installation Setup

After installing cmdai, follow these optional steps to enhance your experience.

## 1. Verify Installation

Check that cmdai is installed correctly:

```bash
cmdai --version
```

You should see output like:
```
cmdai 0.1.0
```

## 2. Configuration (Optional)

cmdai works out of the box with sensible defaults, but you can customize it.

### Create Configuration File

```bash
# Create config directory
mkdir -p ~/.config/cmdai

# Create configuration file
touch ~/.config/cmdai/config.toml
```

### Basic Configuration

Edit `~/.config/cmdai/config.toml`:

```toml
# Backend Configuration
[backend]
primary = "embedded"      # Options: embedded, ollama, vllm
enable_fallback = true    # Fall back to other backends if primary fails

# Embedded Backend Settings (MLX for Apple Silicon, CPU for others)
[backend.embedded]
model_name = "Qwen/Qwen2.5-Coder-1.5B-Instruct"
quantization = "q4_0"

# Optional: Ollama Backend
[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"

# Optional: vLLM Backend
[backend.vllm]
base_url = "http://localhost:8000"
model_name = "codellama/CodeLlama-7b-hf"
api_key = ""  # Optional API key

# Safety Configuration
[safety]
enabled = true
level = "moderate"              # Options: strict, moderate, permissive
require_confirmation = true     # Ask before executing commands

# Dangerous patterns to block (in addition to built-in patterns)
custom_patterns = []

# Output Configuration
[output]
format = "plain"    # Options: plain, json, yaml
verbose = false     # Show detailed timing and backend info
colors = true       # Use colored output

# Shell Configuration
[shell]
default = "bash"    # Your preferred shell: bash, zsh, fish, sh
```

## 3. Test Your First Command

Try generating a simple command:

```bash
cmdai "list all files in the current directory"
```

Expected output:
```
Generated command:
  ls -la

Execute this command? (y/N)
```

## 4. Shell Aliases (Optional but Recommended)

Add these aliases to your shell configuration for faster access:

### Bash / Zsh

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# Quick cmdai alias
alias ai='cmdai'

# Auto-confirm safe commands (use with caution)
alias aic='cmdai --confirm'

# Verbose mode for debugging
alias aiv='cmdai --verbose'

# JSON output for scripting
alias aij='cmdai --output json'
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
# Quick cmdai alias
alias ai='cmdai'
alias aic='cmdai --confirm'
alias aiv='cmdai --verbose'
alias aij='cmdai --output json'
```

### PowerShell (Windows)

Add to your PowerShell profile:

```powershell
# Quick cmdai alias
Set-Alias -Name ai -Value cmdai

# Functions for additional options
function aic { cmdai --confirm $args }
function aiv { cmdai --verbose $args }
function aij { cmdai --output json $args }
```

Apply the changes:
```bash
# Bash
source ~/.bashrc

# Zsh
source ~/.zshrc

# Fish
source ~/.config/fish/config.fish

# PowerShell
. $PROFILE
```

## 5. Shell Completions (Future Enhancement)

*Note: Shell completions are planned for a future release. This section will be updated when available.*

When available, you'll be able to generate completions:

```bash
# Bash
cmdai completions bash > ~/.local/share/bash-completion/completions/cmdai

# Zsh
cmdai completions zsh > ~/.local/share/zsh/site-functions/_cmdai

# Fish
cmdai completions fish > ~/.config/fish/completions/cmdai.fish

# PowerShell
cmdai completions powershell > cmdai.ps1
```

## 6. Backend Setup

### Using Embedded Backend (Default)

No additional setup required! The embedded backend works out of the box:
- **macOS (Apple Silicon)**: Uses MLX for optimized performance
- **Other platforms**: Uses CPU-based inference

### Using Ollama Backend

1. Install Ollama: https://ollama.ai
2. Pull a model:
   ```bash
   ollama pull codellama:7b
   ```
3. Update your config to use Ollama:
   ```toml
   [backend]
   primary = "ollama"
   ```

### Using vLLM Backend

1. Set up vLLM server: https://github.com/vllm-project/vllm
2. Start the server:
   ```bash
   python -m vllm.entrypoints.api_server \
     --model codellama/CodeLlama-7b-hf \
     --port 8000
   ```
3. Update your config:
   ```toml
   [backend]
   primary = "vllm"
   ```

## 7. Safety Settings

cmdai includes comprehensive safety validation. Configure the safety level based on your needs:

### Strict (Default for beginners)
```toml
[safety]
level = "strict"
```
- Blocks all potentially dangerous commands
- Requires confirmation for moderate-risk commands
- Recommended for new users

### Moderate (Recommended for most users)
```toml
[safety]
level = "moderate"
```
- Requires confirmation for high-risk commands
- Allows moderate-risk commands with warnings
- Balances safety and convenience

### Permissive (Advanced users only)
```toml
[safety]
level = "permissive"
```
- Only blocks critically dangerous commands
- Minimal confirmations
- For experienced users who understand the risks

## 8. Common Use Cases

### File Operations
```bash
cmdai "find all PDF files larger than 10MB"
cmdai "compress all images in this directory"
cmdai "find duplicate files by name"
```

### System Monitoring
```bash
cmdai "show disk usage by directory"
cmdai "find the top 10 largest files"
cmdai "list all running processes using more than 100MB of memory"
```

### Text Processing
```bash
cmdai "find all TODO comments in Python files"
cmdai "count lines of code in this project"
cmdai "search for email addresses in all text files"
```

### Git Operations
```bash
cmdai "show commits from last week"
cmdai "find all branches merged into main"
cmdai "list files changed in the last commit"
```

## 9. Troubleshooting

### Command Not Found

If `cmdai` is not found after installation:

1. Check if it's in your PATH:
   ```bash
   which cmdai
   ```

2. Add the install directory to PATH (if needed):
   ```bash
   # For ~/.local/bin
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

### Permission Errors

If you get permission errors:
```bash
chmod +x $(which cmdai)
```

### Backend Errors

If the embedded backend fails:
1. Check available disk space
2. Ensure you have internet connectivity for first-time model download
3. Try using a different backend (Ollama or vLLM)

### Configuration Errors

View current configuration:
```bash
cmdai --show-config
```

Reset to defaults:
```bash
rm ~/.config/cmdai/config.toml
```

## 10. Next Steps

- Read the [README](../README.md) for feature overview
- Check [Safety Features](../README.md#-safety-features) to understand command validation
- Explore [CLI Options](../README.md#cli-options) for advanced usage
- Report issues at: https://github.com/wildcard/cmdai/issues

## 11. Uninstallation

If you need to uninstall cmdai:

### Package Managers
```bash
# Homebrew
brew uninstall cmdai

# Cargo
cargo uninstall cmdai

# Scoop
scoop uninstall cmdai

# Chocolatey
choco uninstall cmdai

# Winget
winget uninstall wildcard.cmdai
```

### Manual Uninstall
```bash
# Remove binary
sudo rm /usr/local/bin/cmdai  # or rm ~/.local/bin/cmdai

# Remove configuration (optional)
rm -rf ~/.config/cmdai
```

---

**Need Help?**
- 📖 Documentation: https://github.com/wildcard/cmdai
- 🐛 Report Issues: https://github.com/wildcard/cmdai/issues
- 💬 Discussions: https://github.com/wildcard/cmdai/discussions
