# Announcing Caro: Your Terminal's AI Companion

**Published**: December 20, 2025
**Author**: The Caro Team

---

## A New Name, The Same Mission

We're excited to announce that **cmdai** has been renamed to **caro**! 🎉

Thanks to the incredible generosity of [@aeplay](https://github.com/aeplay), who graciously transferred the `caro` crate name to our project, we now have a name that better reflects our vision: a friendly, approachable AI companion for your terminal.

## What is Caro?

**caro** is a single-binary Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. Built with safety-first design and optimized for Apple Silicon via the MLX framework, caro makes complex command-line operations accessible to everyone.

```bash
$ caro "find all PDF files larger than 10MB in Downloads"

Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N)
```

## Why "Caro"?

The name **caro** offers several advantages:

- **Brevity**: Shorter and easier to type (4 characters)
- **Memorability**: More distinctive and memorable as a brand
- **Pronounceability**: Natural pronunciation in multiple languages
- **Brandability**: Better suited for a product name

In Latin, *caro* means "dear" or "beloved" - fitting for a tool designed to make your terminal experience more friendly and approachable.

## Key Features

- 🚀 **Fast startup** - Single binary with quick initialization
- 🧠 **Local LLM inference** - Embedded models optimized for Apple Silicon (MLX) and CPU
- 🤖 **Intelligent refinement** - 2-iteration agentic loop for platform-specific commands
- 🌍 **Platform-aware** - Automatically detects OS, architecture, shell, and available commands
- 🛡️ **Safety-first** - Comprehensive validation with 52+ dangerous command patterns
- 📦 **Self-contained** - Single binary distribution
- 🎯 **Multiple backends** - Extensible system supporting MLX, CPU, vLLM, and Ollama

## Getting Started

### Quick Installation

```bash
# One-line setup (recommended)
bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)

# Or via cargo
cargo install caro
```

### First Command

```bash
$ caro "show me the top 5 processes by CPU usage"
```

caro will generate the appropriate command for your platform, validate it for safety, and ask for confirmation before execution.

## What's New in v0.1.0

- ✅ Core CLI with comprehensive argument parsing
- ✅ Embedded model backend with MLX (Apple Silicon) and CPU variants
- ✅ Agentic context loop with iterative refinement
- ✅ Platform-aware command generation
- ✅ Safety validation with 52+ dangerous command patterns
- ✅ Remote backend support (Ollama, vLLM)
- ✅ Interactive execution with shell detection
- ✅ Multiple output formats (JSON, YAML, Plain)

## Migration from cmdai

If you previously installed `cmdai`, migration is simple:

```bash
# Uninstall old version
cargo uninstall cmdai

# Install new version
cargo install caro

# Migrate config (if you have custom configuration)
mv ~/.config/cmdai ~/.config/caro
```

For detailed migration instructions, see our [Naming History](https://github.com/wildcard/caro/blob/main/docs/NAMING_HISTORY.md) documentation.

## Thank You, @aeplay

We want to extend our heartfelt gratitude to [@aeplay](https://github.com/aeplay) for:

- Graciously transferring the `caro` crate name to this project
- Believing in the project's future and potential
- Supporting the open-source Rust community

This generosity has enabled us to have a better, more memorable name that will serve the project well as it grows.

## What's Next

We have exciting plans for caro's future:

- 📊 **Enhanced analytics** - Command history and usage insights
- 🔄 **Multi-step workflows** - Complex task automation
- 🎓 **Learning from feedback** - Adaptive command generation
- 🔌 **Plugin system** - Custom backends and validators
- 📱 **Shell integration** - Direct shell plugins for zsh, bash, fish

## Join the Community

- 🌐 **Website**: [caro.sh](https://caro.sh)
- 💻 **GitHub**: [github.com/wildcard/caro](https://github.com/wildcard/caro)
- 📦 **Crates.io**: [crates.io/crates/caro](https://crates.io/crates/caro)
- 🐛 **Issues**: [GitHub Issues](https://github.com/wildcard/caro/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/wildcard/caro/discussions)

## Try It Today

Ready to supercharge your terminal experience?

```bash
bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
```

We can't wait to see what you build with caro!

---

**The Caro Team**

*Built with Rust | Safety First | Open Source*
