# cmdai (caro)

[![Crates.io](https://img.shields.io/crates/v/cmdai.svg)](https://crates.io/crates/cmdai)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://opensource.org/licenses/AGPL-3.0)
[![CI](https://github.com/wildcard/cmdai/workflows/CI/badge.svg)](https://github.com/wildcard/cmdai/actions)

> ✨ **Active Development** - Published on crates.io with core features working. Visit [caro.sh](https://caro.sh) for more info.

**cmdai** (also known as **caro**) converts natural language descriptions into safe POSIX shell commands using local LLMs. Built with Rust for blazing-fast performance, single-binary distribution, and safety-first design with intelligent platform detection.

```bash
$ cmdai "list all PDF files in Downloads folder larger than 10MB"
# or use the alias
$ caro "list all PDF files in Downloads folder larger than 10MB"

Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N) y
```

## 📋 Project Status

**Current Version:** 0.1.0 (Published on [crates.io](https://crates.io/crates/cmdai))

This project is in **active development** with core features implemented and working. The CLI is functional with embedded local inference and advanced platform-aware command generation.

### ✅ Completed & Published
- ✨ **Published to crates.io** - Install via `cargo install cmdai`
- 🎯 Core CLI structure with comprehensive argument parsing
- 🏗️ Modular architecture with trait-based backends
- 🧠 **Embedded model backend** with MLX (Apple Silicon) and CPU variants
- 🤖 **Agentic context loop** - Iterative refinement with platform detection
- 🌍 **Platform-aware generation** - Detects OS, architecture, available commands
- 📍 **Execution context detection** - CWD, shell type, system constraints
- 🌐 Remote backend support (Ollama, vLLM) with automatic fallback
- 🛡️ Safety validation with 52 pre-compiled dangerous command patterns
- ⚙️ Configuration management with TOML support
- 💬 Interactive user confirmation flows with color-coded risk levels
- 🎬 **Command execution engine** - Safe execution with shell detection
- 📄 Multiple output formats (JSON, YAML, Plain)
- 🧪 Contract-based test structure with TDD methodology
- 🔄 Multi-platform CI/CD pipeline with automated publishing
- 📦 Installation script with automatic `caro` alias setup
- 🖥️ Cross-platform detection and validation (macOS, Linux, Windows)
- 🌐 **Official website** at [caro.sh](https://caro.sh)
- 🎥 **Professional demos** with asciinema recordings

### 🚧 In Progress
- Model downloading and caching optimization
- Command history and learning from user feedback
- Performance profiling and optimization
- Extended safety pattern library

### 📅 Planned
- Multi-step goal completion with dependency resolution
- Shell script generation for complex workflows
- Interactive command refinement with explanations
- Plugin system for custom backends and validators

## ✨ Features

- 🚀 **Fast startup** - Single binary with quick initialization
- 🧠 **Local LLM inference** - Embedded models optimized for Apple Silicon (MLX) and CPU
- 🤖 **Intelligent refinement** - 2-iteration agentic loop for platform-specific command generation
- 🌍 **Platform-aware** - Automatically detects OS, architecture, shell, and available commands
- 🛡️ **Safety-first** - Comprehensive validation with 52+ dangerous command patterns
- 📦 **Self-contained** - Single binary distribution with embedded models
- 🎯 **Multiple backends** - Extensible system supporting MLX, CPU, vLLM, and Ollama
- 💾 **Model management** - Built-in model loading with optimization
- 🌐 **Cross-platform** - Full support for macOS (including Apple Silicon), Linux, and Windows
- 🎬 **Safe execution** - Optional command execution with shell-aware handling

## 🚀 Quick Start

### Installation

#### Option 1: Pre-built Binaries (Recommended)

The fastest way to install cmdai is using our pre-built binaries:

**Quick Install Script (macOS/Linux):**
```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
```

Or with wget:
```bash
wget -qO- https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
```

This will automatically:
- Detect your platform and architecture
- Download the appropriate pre-built binary
- Install to `~/.local/bin`
- Set up the `caro` alias
- Configure your PATH if needed

**Manual Download:**

Download the latest release from [GitHub Releases](https://github.com/wildcard/cmdai/releases/latest):

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | `cmdai-vX.X.X-linux-x86_64.tar.gz` |
| Linux | x86_64 (static) | `cmdai-vX.X.X-linux-x86_64-musl.tar.gz` |
| Linux | ARM64 | `cmdai-vX.X.X-linux-aarch64.tar.gz` |
| macOS | Intel | `cmdai-vX.X.X-darwin-x86_64.tar.gz` |
| macOS | Apple Silicon | `cmdai-vX.X.X-darwin-aarch64.tar.gz` |
| Windows | x86_64 | `cmdai-vX.X.X-windows-x86_64.zip` |

```bash
# Example for Linux x86_64
curl -LO https://github.com/wildcard/cmdai/releases/latest/download/cmdai-v0.1.0-linux-x86_64.tar.gz
tar -xzf cmdai-v0.1.0-linux-x86_64.tar.gz
sudo mv cmdai-v0.1.0-linux-x86_64/cmdai /usr/local/bin/

# Add alias to your shell config
echo "alias caro='cmdai'" >> ~/.bashrc
```

#### Option 2: Using Cargo

If you have Rust installed, you can build from source:

```bash
cargo install cmdai

# Add alias manually to your shell config (~/.bashrc, ~/.zshrc, etc.)
alias caro='cmdai'
```

#### Option 3: One-Line Setup Script

For a fully automated setup (installs Rust if needed):

```bash
bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
```

This will:
- Install Rust (if not already installed)
- Install cmdai via cargo with MLX optimization (Apple Silicon)
- Set up the `caro` alias automatically
- Configure your shell (bash, zsh, or fish)

### Building from Source

#### Prerequisites
- **Rust 1.75+** with Cargo
- **CMake** (for model inference backends)
- **macOS with Apple Silicon** (optional, for GPU acceleration)
- **Xcode** (optional, for full MLX GPU support on Apple Silicon)

### Platform-Specific Setup

#### macOS (Recommended for Apple Silicon)

For complete macOS setup instructions including GPU acceleration, see [macOS Setup Guide](docs/MACOS_SETUP.md).

**Quick Install:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install CMake via Homebrew
brew install cmake

# Clone and build
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release

# Run
./target/release/cmdai "list all files"
```

**For GPU Acceleration (Apple Silicon only):**
- Install Xcode from App Store (required for Metal compiler)
- Build with: `cargo build --release --features embedded-mlx`
- See [macOS Setup Guide](docs/MACOS_SETUP.md) for details

**Note:** The default build uses a stub implementation that works immediately without Xcode. For production GPU acceleration, Xcode is required.

#### Linux

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install dependencies (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install cmake build-essential

# Clone and build
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release
```

#### Windows

```bash
# Install Rust from https://rustup.rs
# Install CMake from https://cmake.org/download/

# Clone and build
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Build the project (uses CPU backend by default)
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
4. **AgentLoop** - Iterative refinement with platform detection
5. **ExecutionContext** - Comprehensive system environment detection
6. **Model Loader** - Efficient model initialization and management

### Intelligent Command Generation

cmdai uses a sophisticated **2-iteration agentic loop** for generating platform-appropriate commands:

**Iteration 1: Context-Aware Generation**
- Detects your OS (macOS, Linux, Windows), architecture, and shell
- Identifies available commands on your system
- Applies platform-specific rules (BSD vs GNU differences)
- Generates initial command with confidence score

**Iteration 2: Smart Refinement** (triggered when needed)
- Extracts commands from pipes and chains
- Fetches command-specific help and version info
- Detects and fixes platform compatibility issues
- Refines complex commands (sed, awk, xargs)

**Example Flow:**
```
User: "show top 5 processes by CPU"
  ↓
Context Detection: macOS 14.2, arm64, zsh
  ↓
Iteration 1: Generates with macOS rules
  ↓
Smart Refinement: Fixes BSD sort syntax
  ↓
Result: ps aux | sort -nrk 3,3 | head -6
```

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

## 🛡️ Safety Features

cmdai includes comprehensive safety validation to prevent dangerous operations:

### Implemented Safety Checks
- ✅ System destruction patterns (`rm -rf /`, `rm -rf ~`)
- ✅ Fork bombs detection (`:(){:|:&};:`)
- ✅ Disk operations (`mkfs`, `dd if=/dev/zero`)
- ✅ Privilege escalation detection (`sudo su`, `chmod 777 /`)
- ✅ Critical path protection (`/bin`, `/usr`, `/etc`)
- ✅ Command validation and sanitization

### Risk Levels
- **Safe** (Green) - Normal operations, no confirmation needed
- **Moderate** (Yellow) - Requires user confirmation in strict mode
- **High** (Orange) - Requires confirmation in moderate mode
- **Critical** (Red) - Blocked in strict mode, requires explicit confirmation

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

### Areas for Contribution
- 🔌 Backend implementations
- 🛡️ Safety pattern definitions
- 🧪 Test coverage expansion
- 📚 Documentation improvements
- 🐛 Bug fixes and optimizations

### Getting Started
1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all tests pass
5. Submit a pull request

### Development Guidelines
- Follow Rust best practices
- Add tests for new functionality
- Update documentation as needed
- Use conventional commit messages
- Run `make check` before submitting

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

### Phase 1: Core Structure ✅ Complete
- [x] CLI argument parsing
- [x] Module architecture
- [x] Backend trait system
- [x] Command generation with embedded models

### Phase 2: Safety & Validation ✅ Complete
- [x] Dangerous pattern detection (52+ patterns)
- [x] POSIX compliance checking
- [x] User confirmation workflows
- [x] Risk assessment system with color coding

### Phase 3: Backend Integration ✅ Complete
- [x] Embedded MLX backend (Apple Silicon)
- [x] Embedded CPU backend (cross-platform)
- [x] vLLM HTTP API support
- [x] Ollama local backend
- [x] Response parsing with fallback strategies
- [x] Comprehensive error handling

### Phase 4: Platform Intelligence ✅ Complete
- [x] Execution context detection
- [x] Platform-specific command rules
- [x] Agentic refinement loop
- [x] Command info enrichment
- [x] Shell-aware execution

### Phase 5: Production Ready 🚧 In Progress
- [x] Published to crates.io
- [x] Installation script with alias setup
- [x] Multi-platform CI/CD
- [x] Website and documentation
- [x] Professional demos
- [ ] Extended test coverage
- [ ] Performance benchmarking suite
- [x] Binary distribution optimization (multi-platform pre-built binaries)

---

**Built with Rust** | **Safety First** | **Open Source**

> **Note**: This is an active development project. Features and APIs are subject to change. See the [specs](specs/) directory for detailed design documentation.