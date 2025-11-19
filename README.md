# cmdai

> **Natural language to safe shell commands** + **Production-grade TUI Component Showcase**

**cmdai** converts natural language descriptions into safe POSIX shell commands using local LLMs. Built with Rust for blazing-fast performance, single-binary distribution, and safety-first design.

```bash
$ cmdai "list all PDF files in Downloads folder larger than 10MB"
Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N) y
```

---

## 🚀 New Contributor? Start Here!

**Welcome to cmdai!** We're excited to have you here. Whether you're new to Rust, terminal UIs, or open source - you're in the right place!

### 5-Minute Quick Start for TUI Components

Want to contribute but not sure where to begin? Follow this quick path:

```bash
# 1. Clone and run the showcase (2 minutes)
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo run --bin tui-showcase

# 2. Copy the template (1 minute)
cp docs/templates/simple_component_template.rs src/tui/components/hello_world.rs

# 3. See the beginner's guide (2 minutes)
# Open GETTING_STARTED.md and follow along!
```

**That's it!** You just explored the showcase and have a template ready to customize. Head to [GETTING_STARTED.md](GETTING_STARTED.md) for your first component tutorial.

### Choose Your Learning Path

Pick the path that matches your experience level:

| Experience Level | Start Here | What You'll Learn |
|-----------------|------------|-------------------|
| **Complete Beginner**<br>Never used Rust or TUIs | [GETTING_STARTED.md](GETTING_STARTED.md) | Step-by-step component creation with explanations for every line |
| **Experienced Developer**<br>Know Rust, new to this project | [ARCHITECTURE_GUIDE.md](ARCHITECTURE_GUIDE.md) | System design, core abstractions, and extension points |
| **Ready to Contribute**<br>Built components, ready for production | [CONTRIBUTING_TUI.md](CONTRIBUTING_TUI.md) | Production quality guidelines and best practices |
| **Quick Answers**<br>Have a specific question | [FAQ.md](FAQ.md) | Common questions, troubleshooting, quick reference |
| **Understanding CI/CD**<br>Want to know about automation | [CI_CD_EXPLAINED.md](CI_CD_EXPLAINED.md) | How GitHub Actions tests your components |

### Learning Path Visualization

```
┌─────────────────────────────────────────────────────────────┐
│                   YOUR JOURNEY STARTS HERE                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │  Run the TUI Showcase         │
              │  cargo run --bin tui-showcase │
              └───────────────┬───────────────┘
                              │
                ┌─────────────┴─────────────┐
                │                           │
                ▼                           ▼
    ┌───────────────────┐       ┌───────────────────┐
    │ Complete Beginner │       │ Experienced Dev   │
    │ GETTING_STARTED   │       │ ARCHITECTURE_GUIDE│
    └─────────┬─────────┘       └─────────┬─────────┘
              │                           │
              └─────────────┬─────────────┘
                            │
                            ▼
              ┌─────────────────────────┐
              │ Build Your Component    │
              │ Use Template + Guide    │
              └───────────┬─────────────┘
                          │
                          ▼
              ┌─────────────────────────┐
              │ Submit Pull Request     │
              │ CONTRIBUTING_TUI.md     │
              └───────────┬─────────────┘
                          │
                          ▼
              ┌─────────────────────────┐
              │ Component Merged! 🎉    │
              │ You're a Contributor!   │
              └─────────────────────────┘
```

---

## 🎨 TUI Component Showcase

This project includes a **production-grade Storybook-like development tool** for terminal UI components! Develop, test, and showcase Ratatui components in isolation with a comprehensive library of examples.

### Showcase Statistics

- **14 Production Components** across 5 categories
- **73+ Interactive Stories** demonstrating different states
- **5 Component Categories**: Display, Input, Feedback, Workflow, Help
- **40% Growth** from community contributions!

### Run the Showcase

```bash
# Basic run
cargo run --bin tui-showcase

# With hot-reload for fast iteration
cargo install cargo-watch
cargo watch -x 'run --bin tui-showcase'
```

### Showcase Highlights

- 🎯 **Component Library** - Browse 14 production-ready components
- 🔄 **Hot Reload Support** - Instant feedback during development
- 📚 **Story-Based Development** - Multiple variations per component
- 🖥️ **Interactive Browser** - Full keyboard navigation and help system
- 🎨 **Visual Gallery** - ASCII art previews and comprehensive examples
- 📖 **Complete Documentation** - 5 comprehensive guides for all skill levels
- 💬 **Community-Driven** - New components based on user requests
- 🤖 **Automated Testing** - GitHub Actions visual testing pipeline

### Component Categories

**Display Components (6)**
- SimpleText - Basic text rendering with styling
- CommandPreview - Shell command display with syntax highlighting
- TableSelector - Interactive table with selection
- CommandOutputViewer - Scrollable command output display 🌟
- HistoryTimeline - Visual timeline of command history 🌟
- GenerationComparison - Side-by-side command comparison 🌟

**Input Components (3)**
- ConfirmationDialog - User confirmation prompts
- CommandEditor - Multi-line command editing
- CommandRating - Feedback collection interface 🌟

**Feedback Components (3)**
- SafetyIndicator - Risk level visualization
- ProgressSpinner - Loading animations
- NotificationToast - Toast notifications

**Workflow Components (1)**
- CommandFlow - Multi-step workflow visualization

**Help Components (1)**
- KeyboardShortcuts - Interactive shortcut reference

🌟 = Community-requested components!

### Documentation Guide

| Document | Purpose | Best For |
|----------|---------|----------|
| [GETTING_STARTED.md](GETTING_STARTED.md) | 5-minute quick start + first component tutorial | Beginners, first-time contributors |
| [ARCHITECTURE_GUIDE.md](ARCHITECTURE_GUIDE.md) | System internals, design decisions, extending framework | Understanding how it works |
| [CONTRIBUTING_TUI.md](CONTRIBUTING_TUI.md) | Production quality guidelines, code standards | Ready to contribute |
| [COMPONENT_GALLERY.md](COMPONENT_GALLERY.md) | Visual reference with ASCII art | Inspiration, examples |
| [FAQ.md](FAQ.md) | Common questions and quick answers | Troubleshooting, quick reference |
| [CI_CD_EXPLAINED.md](CI_CD_EXPLAINED.md) | GitHub Actions visual testing explained | Understanding automation |
| [TUI_SHOWCASE.md](TUI_SHOWCASE.md) | Complete system documentation | Comprehensive reference |

**Not sure where to start?** Most new contributors should begin with [GETTING_STARTED.md](GETTING_STARTED.md) - it assumes zero prior knowledge and walks you through creating your first component in 5 minutes!

---

## 📋 Project Status

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
- **TUI Component Showcase - Storybook for Ratatui development**
- **Comprehensive onboarding documentation for all skill levels**

### 🚧 In Progress
- Model downloading and caching system
- Advanced command execution engine
- Performance optimization

### 📅 Planned
- Multi-step goal completion
- Advanced context awareness
- Shell script generation
- Command history and learning

---

## ✨ Features (Planned & In Development)

- 🚀 **Instant startup** - Single binary with <100ms cold start (target)
- 🧠 **Local LLM inference** - Optimized for Apple Silicon with MLX
- 🛡️ **Safety-first** - Comprehensive command validation framework
- 📦 **Zero dependencies** - Self-contained binary distribution
- 🎯 **Multiple backends** - Extensible backend system (MLX, vLLM, Ollama)
- 💾 **Smart caching** - Hugging Face model management
- 🌐 **Cross-platform** - macOS, Linux, Windows support

---

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

---

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

---

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
│   ├── cache/              # Model caching
│   ├── config/             # Configuration management
│   ├── cli/                # CLI interface
│   ├── models/             # Data models
│   ├── execution/          # Command execution
│   └── tui/                # Terminal UI components
│       ├── showcase.rs     # Component showcase framework
│       └── components/     # Reusable TUI components
├── tests/                   # Contract-based tests
├── specs/                  # Project specifications
└── docs/                   # Documentation and templates
    └── templates/          # Component templates
```

### Core Components

1. **CommandGenerator Trait** - Unified interface for all LLM backends
2. **SafetyValidator** - Command validation and risk assessment
3. **Backend System** - Extensible architecture for multiple inference engines
4. **Cache Manager** - Hugging Face model management (planned)
5. **TUI Showcase** - Component development and testing framework

---

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

---

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

---

## 🤝 Contributing

We welcome contributions! This project is designed to be accessible to contributors of all skill levels.

### Areas for Contribution
- 🎨 **TUI Components** - Add new components to the showcase (great for beginners!)
- 🔌 **Backend implementations** - New LLM backends or optimizations
- 🛡️ **Safety pattern definitions** - Improve command validation
- 🧪 **Test coverage expansion** - Unit and integration tests
- 📚 **Documentation improvements** - Tutorials, examples, guides
- 🐛 **Bug fixes and optimizations** - Performance and reliability

### Getting Started (TUI Components)

**Never contributed to open source before?** Perfect! TUI components are a great place to start:

1. **Read** [GETTING_STARTED.md](GETTING_STARTED.md) - 5-minute tutorial
2. **Run** the showcase: `cargo run --bin tui-showcase`
3. **Copy** the template: `cp docs/templates/simple_component_template.rs src/tui/components/my_component.rs`
4. **Customize** the component with your own ideas
5. **Test** it in the showcase
6. **Submit** a pull request!

**Need help?** Open a GitHub issue with the `question` label. The community is friendly and responsive!

### Development Guidelines
- Follow Rust best practices
- Add tests for new functionality
- Update documentation as needed
- Use conventional commit messages
- Run `make check` before submitting

For detailed contribution guidelines, see [CONTRIBUTING_TUI.md](CONTRIBUTING_TUI.md).

---

## 💬 Community & Support

### Get Help

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/wildcard/cmdai/issues) - Report bugs or problems
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions) - Propose new features or components
- ❓ **Questions**: [GitHub Issues](https://github.com/wildcard/cmdai/issues) with `question` label
- 📖 **Documentation**: See documentation files above or `/specs` directory

### Contributing Components

We especially welcome TUI component contributions! Check out:
- Components tagged `component-request` in issues
- `good-first-issue` - Perfect for beginners
- `help-wanted` - Community needs help with these

### Recognition

All contributors are:
- Listed in the project's contributor graph
- Recognized in release notes
- Welcome to showcase their contributions in portfolios/resumes

**Your first contribution is just one component away!** We can't wait to see what you build.

---

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

---

## 🙏 Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework powering our components
- [MLX](https://github.com/ml-explore/mlx) - Apple's machine learning framework
- [vLLM](https://github.com/vllm-project/vllm) - High-performance LLM serving
- [Ollama](https://ollama.ai) - Local LLM runtime
- [Hugging Face](https://huggingface.co) - Model hosting and caching
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- **Our amazing community of contributors** - Thank you for making this project better!

---

## 🗺️ Roadmap

### Phase 1: Core Structure (Current)
- [x] CLI argument parsing
- [x] Module architecture
- [x] Backend trait system
- [x] TUI Component Showcase
- [x] Comprehensive documentation
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

**Built with Rust** | **Safety First** | **Community Driven** | **Open Source**

> **Note**: This is an active development project. Features and APIs are subject to change. The TUI Component Showcase is production-ready and actively accepting contributions! See the documentation guides above to get started.

---

## 🎯 Quick Links

**For New Contributors:**
- [5-Minute Quick Start](GETTING_STARTED.md#5-minute-quick-start)
- [Your First Component](GETTING_STARTED.md#your-first-component-detailed)
- [Common Patterns](GETTING_STARTED.md#common-patterns)
- [FAQ - Getting Started](FAQ.md#getting-started)

**For Understanding the System:**
- [Architecture Overview](ARCHITECTURE_GUIDE.md#high-level-overview)
- [Component Lifecycle](ARCHITECTURE_GUIDE.md#the-component-lifecycle)
- [Design Decisions](ARCHITECTURE_GUIDE.md#design-decisions)

**For Contributing:**
- [Contribution Guidelines](CONTRIBUTING_TUI.md)
- [Component Template](docs/templates/simple_component_template.rs)
- [Visual Gallery](COMPONENT_GALLERY.md)

**Need Help?**
- [FAQ](FAQ.md) - Quick answers to common questions
- [GitHub Issues](https://github.com/wildcard/cmdai/issues) - Ask questions or report issues
- [GitHub Discussions](https://github.com/wildcard/cmdai/discussions) - Community chat

**Ready to contribute?** Pick a component idea from the issues, follow the [GETTING_STARTED.md](GETTING_STARTED.md) guide, and submit your first PR. We're here to help every step of the way!
