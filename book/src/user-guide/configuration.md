# Configuration

cmdai can be configured through configuration files, environment variables, and command-line flags.

## Configuration File

cmdai uses TOML for configuration. The default location is:

- **Linux**: `~/.config/cmdai/config.toml`
- **macOS**: `~/.config/cmdai/config.toml`
- **Windows**: `%APPDATA%\cmdai\config.toml`

### Creating a Configuration File

```bash
# Create config directory
mkdir -p ~/.config/cmdai

# Create config file
cat > ~/.config/cmdai/config.toml << 'EOF'
[backend]
primary = "embedded"
enable_fallback = true

[safety]
enabled = true
level = "moderate"

[output]
format = "plain"
verbose = false
EOF
```

## Configuration Sections

### Backend Configuration

Configure which LLM backend to use:

```toml
[backend]
# Primary backend: "embedded", "ollama", "vllm"
primary = "embedded"

# Enable automatic fallback to other backends
enable_fallback = true

# Timeout for backend requests (seconds)
timeout = 30
```

#### Embedded Backend (Default)

```toml
[backend.embedded]
# Automatically selects MLX on Apple Silicon, CPU otherwise
# No additional configuration needed
enabled = true
```

#### Ollama Backend

```toml
[backend.ollama]
enabled = true
base_url = "http://localhost:11434"
model_name = "codellama:7b"

# Optional: Request timeout
timeout = 60

# Optional: Number of retries
max_retries = 3
```

#### vLLM Backend

```toml
[backend.vllm]
enabled = true
base_url = "http://localhost:8000"
model_name = "codellama/CodeLlama-7b-hf"

# Optional: API key for authentication
api_key = "your-api-key-here"

# Optional: Request timeout
timeout = 60
```

### Safety Configuration

Control command safety validation:

```toml
[safety]
# Enable safety validation
enabled = true

# Safety level: "strict", "moderate", or "permissive"
level = "moderate"

# Require confirmation before execution
require_confirmation = true

# Custom dangerous patterns (regex)
custom_patterns = [
    "curl.*\\|.*bash",
    "wget.*\\|.*sh",
    "chmod 777",
]

# Protected paths
protected_paths = [
    "/",
    "/bin",
    "/usr",
    "/etc",
    "/var",
    "/boot",
    "/System",  # macOS
]
```

### Output Configuration

Control output format and verbosity:

```toml
[output]
# Output format: "plain", "json", or "yaml"
format = "plain"

# Enable verbose output
verbose = false

# Show timing information
show_timing = true

# Color output (auto, always, never)
color = "auto"
```

### Shell Configuration

Default shell preferences:

```toml
[shell]
# Default shell: "bash", "zsh", "fish", "sh", "powershell", "cmd"
default = "bash"

# Shell-specific options
[shell.options]
bash = "--noprofile --norc"
zsh = "--no-rcs"
```

### Cache Configuration

Model and data caching:

```toml
[cache]
# Cache directory (default: platform-specific)
directory = "~/.cache/cmdai"

# Maximum cache size in MB (0 = unlimited)
max_size = 5000

# Cache expiration in days (0 = never expire)
expiration_days = 30
```

## Environment Variables

Override configuration with environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `CMDAI_BACKEND` | Primary backend | `embedded`, `ollama`, `vllm` |
| `CMDAI_SAFETY_LEVEL` | Safety level | `strict`, `moderate`, `permissive` |
| `CMDAI_OUTPUT_FORMAT` | Output format | `plain`, `json`, `yaml` |
| `CMDAI_VERBOSE` | Enable verbose mode | `true`, `false` |
| `CMDAI_CONFIG` | Config file path | `/path/to/config.toml` |
| `OLLAMA_HOST` | Ollama API URL | `http://localhost:11434` |
| `RUST_LOG` | Logging level | `debug`, `info`, `warn`, `error` |

### Example Usage

```bash
# Use Ollama backend with verbose output
CMDAI_BACKEND=ollama CMDAI_VERBOSE=true cmdai "list files"

# Use strict safety mode
CMDAI_SAFETY_LEVEL=strict cmdai "delete old files"

# JSON output for scripting
CMDAI_OUTPUT_FORMAT=json cmdai "show disk usage"
```

## Command-Line Flags

Override configuration with CLI flags (highest priority):

```bash
# Backend selection
cmdai --backend ollama "your command"

# Safety level
cmdai --safety strict "your command"

# Output format
cmdai --output json "your command"

# Verbose mode
cmdai --verbose "your command"

# Custom config file
cmdai --config /path/to/config.toml "your command"
```

## Configuration Priority

Configuration is loaded in this order (later overrides earlier):

1. **Built-in defaults**
2. **Configuration file** (`~/.config/cmdai/config.toml`)
3. **Environment variables** (`CMDAI_*`)
4. **Command-line flags** (`--backend`, `--safety`, etc.)

### Example

```bash
# Config file says: backend = "embedded"
# Environment says: CMDAI_BACKEND=ollama
# CLI says: --backend vllm

# Result: vLLM backend is used (CLI has highest priority)
cmdai --backend vllm "list files"
```

## Viewing Configuration

Check your current configuration:

```bash
# Show effective configuration
cmdai --show-config

# Show configuration in JSON format
cmdai --show-config --output json

# Show with debug information
RUST_LOG=debug cmdai --show-config
```

## Example Configurations

### Development Setup

```toml
[backend]
primary = "embedded"
enable_fallback = true

[safety]
enabled = true
level = "strict"

[output]
format = "plain"
verbose = true
show_timing = true
```

### Production Scripts

```toml
[backend]
primary = "ollama"
enable_fallback = false

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"

[safety]
enabled = true
level = "strict"
require_confirmation = true

[output]
format = "json"
verbose = false
```

### Personal Use

```toml
[backend]
primary = "embedded"

[safety]
enabled = true
level = "moderate"

[output]
format = "plain"
color = "auto"
show_timing = true
```

## Troubleshooting

### Configuration Not Loading

```bash
# Check config file location
cmdai --show-config | grep config

# Verify file exists
ls -la ~/.config/cmdai/config.toml

# Check for syntax errors
cat ~/.config/cmdai/config.toml
```

### Backend Not Working

```bash
# Test backend availability
cmdai --verbose "echo test"

# Check backend logs
RUST_LOG=debug cmdai "echo test"

# Verify backend configuration
cmdai --show-config
```

### Permission Issues

```bash
# Fix config directory permissions
chmod 700 ~/.config/cmdai

# Fix config file permissions
chmod 600 ~/.config/cmdai/config.toml
```

## Next Steps

- [Safety & Security](./safety.md) - Configure safety settings
- [Backend Development](../dev-guide/backends.md) - Develop custom backends
- [Architecture](../dev-guide/architecture.md) - Understand the system
