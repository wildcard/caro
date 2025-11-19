```
   ╔═══════════════════════════════════════╗
   ║     ██████╗███╗   ███╗██████╗        ║
   ║    ██╔════╝████╗ ████║██╔══██╗       ║
   ║    ██║     ██╔████╔██║██║  ██║       ║
   ║    ██║     ██║╚██╔╝██║██║  ██║       ║
   ║    ╚██████╗██║ ╚═╝ ██║██████╔╝       ║
   ║     ╚═════╝╚═╝     ╚═╝╚═════╝        ║
   ║         ▲ AI                          ║
   ╚═══════════════════════════════════════╝
```

<div align="center">

# ⚡🛡️ cmdai

**AI-Powered Commands. Human-Level Safety.**

[![AGPL License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Development Stage](https://img.shields.io/badge/status-early%20development-yellow.svg)](https://github.com/wildcard/cmdai)

**Guard Rails for the Fast Lane** | **Think Fast. Stay Safe.**

[Quick Start](#-quick-start) · [Documentation](#-usage) · [Contributing](CONTRIBUTING.md) · [Brand Guide](brand-assets/interactive/brand-guide.html)

</div>

---

> **Early Development Stage** - Architecture defined, core implementation in progress

**cmdai** converts natural language descriptions into safe POSIX shell commands using local LLMs. Built with Rust for blazing-fast performance, single-binary distribution, and safety-first design.

**Every command validated. Every time.**

```bash
$ cmdai "list all PDF files in Downloads folder larger than 10MB"
Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N) y
```

## 📋 Project Status

This project is in **active early development**. The architecture and module structure are in place, with implementation ongoing.

### ✅ Completed
- Core CLI structure with comprehensive argument parsing
- Modular architecture with trait-based backends
- **Embedded model backend with MLX (Apple Silicon) and CPU variants** ✨
- **Remote backend support (Ollama, vLLM) with automatic fallback** ✨
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

## ✨ Features (Planned & In Development)

- 🚀 **Instant startup** - Single binary with <100ms cold start (target)
- 🧠 **Local LLM inference** - Optimized for Apple Silicon with MLX
- 🛡️ **Safety-first** - Comprehensive command validation framework
- 📦 **Zero dependencies** - Self-contained binary distribution
- 🎯 **Multiple backends** - Extensible backend system (MLX, vLLM, Ollama)
- 💾 **Smart caching** - Hugging Face model management
- 🌐 **Cross-platform** - macOS, Linux, Windows support

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ with Cargo
- macOS with Apple Silicon (for MLX backend, optional)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Build the project
cargo build --release

# Run the CLI
./target/release/cmdai --version
```

### Development Commands

```bash
# Run tests
make test

# Format code
make fmt

# Run linter
make lint

# Build optimized binary
make build-release

# Run with debug logging
RUST_LOG=debug cargo run -- "your command"
```

## 📖 Usage

### Basic Syntax
```bash
cmdai [OPTIONS] <PROMPT>
```

### Examples
```bash
# Basic command generation
cmdai "list all files in the current directory"

# With specific shell
cmdai --shell zsh "find large files"

# JSON output for scripting
cmdai --output json "show disk usage"

# Adjust safety level
cmdai --safety permissive "clean temporary files"

# Auto-confirm dangerous commands
cmdai --confirm "remove old log files"

# Verbose mode with timing info
cmdai --verbose "search for Python files"
```

### CLI Options

| Option | Description | Status |
|--------|-------------|--------|
| `-s, --shell <SHELL>` | Target shell (bash, zsh, fish, sh, powershell, cmd) | ✅ Implemented |
| `--safety <LEVEL>` | Safety level (strict, moderate, permissive) | ✅ Implemented |
| `-o, --output <FORMAT>` | Output format (json, yaml, plain) | ✅ Implemented |
| `-y, --confirm` | Auto-confirm dangerous commands | ✅ Implemented |
| `-v, --verbose` | Enable verbose output with timing | ✅ Implemented |
| `-c, --config <FILE>` | Custom configuration file | ✅ Implemented |
| `--show-config` | Display current configuration | ✅ Implemented |
| `--auto` | Execute without confirmation | 📅 Planned |
| `--allow-dangerous` | Allow potentially dangerous commands | 📅 Planned |
| `--verbose` | Enable verbose logging | ✅ Available |

### Examples (Target Functionality)

```bash
# Simple command generation
cmdai "compress all images in current directory"

# With specific backend
cmdai --backend mlx "find large log files"

# Verbose mode for debugging
cmdai --verbose "show disk usage"
```

## 🏗️ Architecture

### Module Structure

```
cmdai/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── backends/            # LLM backend implementations
│   │   ├── mod.rs          # Backend trait definition
│   │   ├── mlx.rs          # Apple Silicon MLX backend
│   │   ├── vllm.rs         # vLLM remote backend
│   │   └── ollama.rs       # Ollama local backend
│   ├── safety/             # Command validation
│   │   └── mod.rs          # Safety validator
│   ├── cache/              # Model caching
│   ├── config/             # Configuration management
│   ├── cli/                # CLI interface
│   ├── models/             # Data models
│   └── execution/          # Command execution
├── tests/                   # Contract-based tests
└── specs/                  # Project specifications
```

### Core Components

1. **CommandGenerator Trait** - Unified interface for all LLM backends
2. **SafetyValidator** - Command validation and risk assessment
3. **Backend System** - Extensible architecture for multiple inference engines
4. **Cache Manager** - Hugging Face model management (planned)

### Backend Architecture

```rust
#[async_trait]
trait CommandGenerator {
    async fn generate_command(&self, request: &CommandRequest) 
        -> Result<GeneratedCommand, GeneratorError>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
}
```

## 🔧 Development

### Prerequisites
- Rust 1.75+ 
- Cargo
- Make (optional, for convenience commands)
- Docker (optional, for development container)

### Setup Development Environment

```bash
# Clone and enter the project
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Install dependencies and build
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt -- --check

# Run clippy linter
cargo clippy -- -D warnings
```

### Backend Configuration

cmdai supports multiple inference backends with automatic fallback:

#### Embedded Backend (Default)
- **MLX**: Optimized for Apple Silicon Macs (M1/M2/M3)
- **CPU**: Cross-platform fallback using Candle framework
- Model: Qwen2.5-Coder-1.5B-Instruct (quantized)
- No external dependencies required

#### Remote Backends (Optional)
Configure in `~/.config/cmdai/config.toml`:

```toml
[backend]
primary = "embedded"  # or "ollama", "vllm"
enable_fallback = true

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"

[backend.vllm]
base_url = "http://localhost:8000"
model_name = "codellama/CodeLlama-7b-hf"
api_key = "optional-api-key"
```

### Project Configuration

The project uses several configuration files:
- `Cargo.toml` - Rust dependencies and build configuration
- `~/.config/cmdai/config.toml` - User configuration
- `clippy.toml` - Linter rules
- `rustfmt.toml` - Code formatting rules
- `deny.toml` - Dependency audit configuration

### Testing Strategy

The project uses contract-based testing:
- Unit tests for individual components
- Integration tests for backend implementations
- Contract tests to ensure trait compliance
- Property-based testing for safety validation

## 🛡️ Safety System: Your Command Line Guardian

cmdai includes comprehensive safety validation to prevent dangerous operations. Every command is analyzed and color-coded before execution.

### Safety Level Visualization

```
┌─ cmdai Safety Levels ────────────────────────────────┐
│                                                       │
│  SAFE      ▓▓▓▓▓▓▓▓▓▓ 100%   🟢 Green               │
│  MODERATE  ▓▓▓▓▓▓░░░░  60%   🟡 Yellow              │
│  HIGH      ▓▓▓▓░░░░░░  40%   🟠 Orange              │
│  CRITICAL  ▓░░░░░░░░░  10%   🔴 Red                 │
│                                                       │
└───────────────────────────────────────────────────────┘
```

### What We Protect Against
- ✅ System destruction patterns (`rm -rf /`, `rm -rf ~`)
- ✅ Fork bombs detection (`:(){:|:&};:`)
- ✅ Disk operations (`mkfs`, `dd if=/dev/zero`)
- ✅ Privilege escalation detection (`sudo su`, `chmod 777 /`)
- ✅ Critical path protection (`/bin`, `/usr`, `/etc`)
- ✅ Command validation and sanitization

### How It Works
- **🟢 SAFE** - Normal operations, executes immediately
- **🟡 MODERATE** - Requires user confirmation in strict mode
- **🟠 HIGH** - Requires confirmation in moderate mode
- **🔴 CRITICAL** - Blocked by default, requires explicit override

**Ship faster. Sleep better.** Your terminal now has a brain.

### Safety Configuration
Configure safety levels in `~/.config/cmdai/config.toml`:
```toml
[safety]
enabled = true
level = "moderate"  # strict, moderate, or permissive
require_confirmation = true
custom_patterns = ["additional", "dangerous", "patterns"]
```

## 🤝 Contributing

We welcome contributions! This is an early-stage project with many opportunities to contribute.

**Your terminal. Now with a brain.** Help us make it even smarter and safer.

### Areas for Contribution
- 🔌 **Backend implementations** - MLX, vLLM, Ollama support
- 🛡️ **Safety pattern definitions** - Help protect against dangerous commands
- 🧪 **Test coverage expansion** - TDD is our way
- 📚 **Documentation improvements** - Clear docs = happy developers
- 🐛 **Bug fixes and optimizations** - Make it faster, safer, better

### Getting Started
1. Read our [Contributor Onboarding Guide](culture/CONTRIBUTOR_ONBOARDING.md)
2. Check out [Community Guidelines](culture/COMMUNITY_GUIDELINES.md)
3. Fork the repository
4. Create a feature branch
5. Make your changes with tests
6. Ensure all tests pass (`make check`)
7. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

**Think Fast. Stay Safe.** We value quality contributions over quick fixes.

## 📜 License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)** - see the [LICENSE](LICENSE) file for details.

### License Summary
- ✅ Commercial use
- ✅ Modification
- ✅ Distribution
- ✅ Private use
- ⚠️ Network use requires source disclosure
- ⚠️ Same license requirement
- ⚠️ State changes documentation

## 🙏 Acknowledgments

- [MLX](https://github.com/ml-explore/mlx) - Apple's machine learning framework
- [vLLM](https://github.com/vllm-project/vllm) - High-performance LLM serving
- [Ollama](https://ollama.ai) - Local LLM runtime
- [Hugging Face](https://huggingface.co) - Model hosting and caching
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing

## 📞 Support & Community

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/wildcard/cmdai/issues)
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- 📖 **Documentation**: See `/specs` directory for detailed specifications

## 🗺️ Roadmap

### Phase 1: Core Structure (Current)
- [x] CLI argument parsing
- [x] Module architecture
- [x] Backend trait system
- [ ] Basic command generation

### Phase 2: Safety & Validation
- [ ] Dangerous pattern detection
- [ ] POSIX compliance checking
- [ ] User confirmation workflows
- [ ] Risk assessment system

### Phase 3: Backend Integration
- [ ] vLLM HTTP API support
- [ ] Ollama local backend
- [ ] Response parsing
- [ ] Error handling

### Phase 4: MLX Optimization
- [ ] FFI bindings with cxx
- [ ] Metal Performance Shaders
- [ ] Unified memory handling
- [ ] Apple Silicon optimization

### Phase 5: Production Ready
- [ ] Comprehensive testing
- [ ] Performance optimization
- [ ] Binary distribution
- [ ] Package manager support

---

<div align="center">

## ⚡🛡️ Think Fast. Stay Safe.

**Built with Rust** · **Safety First** · **Open Source (AGPL-3.0)**

```
┌─────────────────────────────────────────────────────┐
│  "Guard Rails for the Fast Lane"                   │
│                                                     │
│  Your AI assistant. Your safety validator.         │
│  Your terminal's new best friend.                  │
└─────────────────────────────────────────────────────┘
```

[Website (planned)](https://cmdai.dev) · [GitHub](https://github.com/wildcard/cmdai) · [Interactive Brand Guide](brand-assets/interactive/brand-guide.html)

---

**1985 Vibes. 2025 Brains.**

> **Note**: This is an active development project. Features and APIs are subject to change. See the [specs](specs/) directory for detailed design documentation.

</div>