> **Note:** Versions prior to 1.0.0 were a different project ("creation-addressed replicated objects"). If you're looking for that project, it remains available at [crates.io/crates/caro/0.7.1](https://crates.io/crates/caro/0.7.1). From version 1.0.0 onwards, this crate is the project described below.

# caro

**caro** converts natural language descriptions into safe POSIX shell commands using local LLMs. Built with Rust for blazing-fast performance, single-binary distribution, and safety-first design with intelligent platform detection.

```bash
$ caro "list all PDF files in Downloads folder larger than 10MB"

Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N) y
```

## Features

- **Fast startup** - Single binary with quick initialization
- **Local LLM inference** - Embedded models optimized for Apple Silicon (MLX) and CPU
- **Intelligent refinement** - 2-iteration agentic loop for platform-specific command generation
- **Platform-aware** - Automatically detects OS, architecture, shell, and available commands
- **Safety-first** - Comprehensive validation with 52+ dangerous command patterns
- **Self-contained** - Single binary distribution with embedded models
- **Multiple backends** - Extensible system supporting MLX, CPU, vLLM, and Ollama
- **Cross-platform** - Full support for macOS (including Apple Silicon), Linux, and Windows

## Quick Start

### Installation

```bash
cargo install caro
```

Or use the one-line setup script:
```bash
bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
```

### Usage

```bash
# Basic command generation
caro "list all files in the current directory"

# With specific shell
caro --shell zsh "find large files"

# JSON output for scripting
caro --output json "show disk usage"
```

## Documentation

For full documentation, visit the [GitHub repository](https://github.com/wildcard/caro) or [caro.sh](https://caro.sh).

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.
