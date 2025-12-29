# caro

Convert natural language to shell commands using local LLMs.

## Installation

### NuGet Package Manager

```powershell
Install-Package caro
```

### .NET CLI

```bash
dotnet add package caro
```

## Usage

After installation, the `caro.exe` binary is available in the package's `tools` folder.

```powershell
# Run directly from tools folder
.\packages\caro.1.0.2\tools\caro.exe "list all files"

# Or add to PATH and run anywhere
caro "find files modified today"
```

### Examples

```powershell
# Generate a shell command
caro "find all log files larger than 100MB"

# Execute the generated command directly
caro -e "list running processes"

# Use a specific model
caro --model "mlx-community/Qwen2.5-Coder-7B-4bit" "compress images"
```

## Features

- Converts natural language to POSIX shell commands
- Safety-first approach with command validation
- Multiple backend support (MLX, Ollama, vLLM)
- Works offline with local LLM models

## Alternative Installation

### Cargo (Rust)

```bash
cargo install caro
```

### npm

```bash
npm install -g @wildcard/caro
```

### Docker

```bash
docker run ghcr.io/wildcard/caro "list files"
```

## Documentation

See the [GitHub repository](https://github.com/wildcard/caro) for full documentation.

## License

AGPL-3.0
