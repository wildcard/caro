# @wildcard/caro

Convert natural language to shell commands using local LLMs.

## Installation

```bash
npm install -g @wildcard/caro
```

Or with npx (no installation required):

```bash
npx @wildcard/caro "list all files in current directory"
```

## Usage

```bash
# Generate a shell command from natural language
caro "find all JavaScript files modified in the last week"

# Execute the generated command directly
caro -e "list all running processes"

# Use a specific model
caro --model "mlx-community/Qwen2.5-Coder-7B-4bit" "compress all PNG files"
```

## Features

- Converts natural language to POSIX shell commands
- Safety-first approach with command validation
- Multiple backend support (MLX, Ollama, vLLM)
- Apple Silicon optimization via MLX
- Works offline with local LLM models

## Requirements

- Node.js >= 16.0.0
- Supported platforms: macOS (Intel/Apple Silicon), Linux (x64/arm64), Windows (x64)

## Alternative Installation Methods

### Cargo (Rust)

```bash
cargo install caro
```

### Binary Download

Download pre-built binaries from [GitHub Releases](https://github.com/wildcard/caro/releases).

### Docker

```bash
docker run ghcr.io/wildcard/caro "list files"
```

## Documentation

See the [full documentation](https://github.com/wildcard/caro) for more information.

## License

AGPL-3.0
