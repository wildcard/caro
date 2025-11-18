# Introduction

> 🚧 **Early Development Stage** - Architecture defined, core implementation in progress

**cmdai** converts natural language descriptions into safe POSIX shell commands using local LLMs. Built with Rust for blazing-fast performance, single-binary distribution, and safety-first design.

```bash
$ cmdai "list all PDF files in Downloads folder larger than 10MB"
Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N) y
```

## Why cmdai?

- 🚀 **Instant startup** - Single binary with <100ms cold start (target)
- 🧠 **Local LLM inference** - Optimized for Apple Silicon with MLX
- 🛡️ **Safety-first** - Comprehensive command validation framework
- 📦 **Zero dependencies** - Self-contained binary distribution
- 🎯 **Multiple backends** - Extensible backend system (MLX, vLLM, Ollama)
- 💾 **Smart caching** - Hugging Face model management
- 🌐 **Cross-platform** - macOS, Linux, Windows support

## Project Status

This project is in **active early development**. The architecture and module structure are in place, with implementation ongoing.

### ✅ Completed

- Core CLI structure with comprehensive argument parsing
- Modular architecture with trait-based backends
- **Embedded model backend with MLX (Apple Silicon) and CPU variants**
- **Remote backend support (Ollama, vLLM) with automatic fallback**
- Safety validation with pattern matching and risk assessment
- Configuration management with TOML support
- Interactive user confirmation flows
- Multiple output formats (JSON, YAML, Plain)
- Contract-based test structure with TDD methodology
- Multi-platform CI/CD pipeline

### 🚧 In Progress

- Model downloading and caching system
- Advanced command execution engine
- Performance optimization

### 📅 Planned

- Multi-step goal completion
- Advanced context awareness
- Shell script generation
- Command history and learning

## Quick Example

```bash
# Basic command generation
cmdai "list all files in the current directory"

# With specific shell
cmdai --shell zsh "find large files"

# JSON output for scripting
cmdai --output json "show disk usage"

# Adjust safety level
cmdai --safety permissive "clean temporary files"
```

## Next Steps

- [Getting Started](./user-guide/getting-started.md) - Install and run cmdai
- [Architecture](./dev-guide/architecture.md) - Understand the design
- [Contributing](./dev-guide/contributing.md) - Join the project

---

**Built with Rust** | **Safety First** | **Open Source**
