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

- 🚀 **Instant startup** - Single binary with <100ms cold start (target) → [Performance Details](./technical/performance.md)
- 🧠 **Local LLM inference** - Optimized for Apple Silicon with MLX → [MLX Integration](./technical/mlx-integration.md)
- 🛡️ **Safety-first** - Comprehensive command validation framework → [Safety & Security](./user-guide/safety.md)
- 📦 **Zero dependencies** - Self-contained binary distribution
- 🎯 **Multiple backends** - Extensible backend system (MLX, vLLM, Ollama) → [Backend Development](./dev-guide/backends.md)
- 💾 **Smart caching** - Hugging Face model management
- 🌐 **Cross-platform** - macOS, Linux, Windows support

## How It Works

```
┌─────────────────────────────────────────────────────┐
│  "Find PDF files larger than 10MB"                  │  ← Natural Language
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
         ┌───────────────┐
         │   cmdai CLI   │
         └───────┬───────┘
                 │
                 ▼
    ┌────────────────────────┐
    │  LLM Backend Selection │  ← MLX / Ollama / vLLM
    └────────┬───────────────┘
             │
             ▼
    ┌────────────────────┐
    │ Command Generation │  → find . -name "*.pdf" -size +10M
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ Safety Validation  │  ← Pattern Matching, Risk Assessment
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ User Confirmation  │  ← Execute? (y/N)
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │   Execute Shell    │  ✅ Command runs safely
    └────────────────────┘
```

## Project Status

This project is in **active early development**. The architecture and module structure are in place, with implementation ongoing.

> **📊 Current Status:** See the [Project Roadmap](./community/roadmap.md) for detailed progress and [What's Being Built](./community/active-development.md) for active tasks.

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

## Learn by Example

New to cmdai? Start with our interactive tutorials:

### 🎯 [Tutorial: Your First Command](./tutorial/first-command.md)
**5 minutes** - Step-by-step introduction to generating commands
- Generate your first command
- Understand safety validation
- Execute commands safely

### 📂 [Tutorial: Working with Files](./tutorial/working-with-files.md)
**15 minutes** - Master file operations through real-world examples
- Find and organize files
- Safe deletion and cleanup
- Advanced file operations

### 💻 [Tutorial: System Operations](./tutorial/system-operations.md)
**15 minutes** - Monitor and manage your system
- Check disk space and memory
- Monitor processes
- Network troubleshooting

### 🎮 [Try It Online](./tutorial/playground.md)
**Coming Soon** - Interactive playground in your browser
- No installation required
- Safe experimentation
- Share examples

---

## Quick Examples

Here's what you can do with cmdai:

| Task | Command | What It Does |
|------|---------|--------------|
| Basic query | `cmdai "list all files"` | Generates: `ls -la` |
| Shell-specific | `cmdai --shell zsh "find large files"` | Optimized for your shell |
| JSON output | `cmdai --output json "show disk usage"` | Machine-readable output |
| Safety adjustment | `cmdai --safety permissive "clean temps"` | Control safety levels |

```bash
# Example session
$ cmdai "find PDF files larger than 10MB"

🤖 Generating command...
✨ Generated: find . -name "*.pdf" -size +10M
Safety: ✅ Safe
Execute? (y/N)
```

> **💡 See it in action:** Try the [interactive tutorial](./tutorial/first-command.md)

## Next Steps

**New users:**
- 🚀 [Tutorial: Your First Command](./tutorial/first-command.md) - Start here!
- 📖 [Getting Started](./user-guide/getting-started.md) - Install and run cmdai
- 🎯 [Quick Start](./user-guide/quick-start.md) - Common patterns

**Contributors:**
- 🏗️ [Architecture](./dev-guide/architecture.md) - Understand the design
- 🤝 [Contributing](./dev-guide/contributing.md) - Join the project

---

## See Also

**For Users:**
- [Installation Guide](./user-guide/installation.md) - Detailed installation instructions
- [Configuration](./user-guide/configuration.md) - Customize cmdai for your needs
- [Safety Features](./user-guide/safety.md) - Understanding command validation

**For Developers:**
- [Architecture Overview](./dev-guide/architecture.md) - System design and components
- [Contributing Guide](./dev-guide/contributing.md) - How to contribute
- [TDD Workflow](./dev-guide/tdd-workflow.md) - Development methodology

**Technical Deep Dives:**
- [Rust Learnings](./technical/rust-learnings.md) - Insights from implementation
- [Safety Validation](./technical/safety-validation.md) - How safety works under the hood

---

**Built with Rust** | **Safety First** | **Open Source**
