# cmdai

> 🚧 **Early Development Stage** - Architecture defined, core implementation in progress

**cmdai** converts natural language descriptions into safe POSIX shell commands using local LLMs. Built with Rust for blazing-fast performance, single-binary distribution, and safety-first design.

```bash
$ cmdai "list all PDF files in Downloads folder larger than 10MB"
Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N) y
```

## 📋 Project Status

**Current State**: **70% Complete** (revised) | **Timeline**: **8-10 weeks** to v1.0 (revised)
**Test Suite**: 133/136 passing (98%) | **Gaps**: 13 total (4 P0, 5 P1, 4 P2)

> **⚠️ UPDATED**: Comprehensive MVP gap analysis revealed 9 additional gaps beyond original 4 blockers
>
> **📖 For complete gap analysis, see [MVP_GAPS.md](MVP_GAPS.md)** ← **START HERE**
> **📊 For detailed project assessment, see [PROJECT_STATUS.md](PROJECT_STATUS.md)**
> **🚨 For critical blockers and solutions, see [BLOCKERS.md](BLOCKERS.md)**
> **💻 For implementation guides, see [IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md)**

### ✅ Production-Ready Components (80%)

**Core Infrastructure** (100% Complete):
- ✅ CLI argument parsing with `clap` - comprehensive flag support
- ✅ Safety validation system - 52 pre-compiled dangerous command patterns
- ✅ Configuration management - TOML-based with validation
- ✅ Model caching infrastructure - LRU eviction, manifest tracking
- ✅ Multi-backend architecture - trait-based, async, extensible

**Remote Backends** (100% Complete):
- ✅ Ollama local server integration - HTTP API, streaming, auto-detection
- ✅ vLLM remote server support - OpenAI-compatible API, auth, timeouts
- ✅ Automatic fallback system - graceful degradation

**Testing & CI** (98% Complete):
- ✅ 133 tests passing - unit, integration, contract, E2E
- ✅ Multi-platform CI/CD - Linux, macOS, Windows builds
- ✅ Clippy clean - no warnings with `--deny warnings`
- ✅ Security audit - `cargo audit` passing

### 🔴 Critical Gaps (13 items, 100-140 hours to resolve)

**⚠️ UPDATED**: Originally identified 4 blockers, now 13 total gaps after comprehensive analysis.
**📖 See [MVP_GAPS.md](MVP_GAPS.md) for complete details.**

**P0 - Must Fix Before Launch** (80-118 hours):
1. **Embedded Backend Not Functional** (8-12 hours) - 3 failing tests
2. **Model Download Not Implemented** (16-24 hours) - Fresh installs broken
3. **🆕 Command Execution Missing** (12-16 hours) - **Tool only displays commands, doesn't execute**
4. **🆕 Tokenizer Download Missing** (2-4 hours) - **Will break embedded backend**
5. **Binary Distribution Not Setup** (8-12 hours) - Users can't install
6. **🆕 User Documentation Missing** (8-12 hours) - **Only developer docs exist**
7. **🆕 Performance Unvalidated** (6-8 hours) - **Promises not verified**
8. **🆕 Cross-Platform Testing Incomplete** (8-12 hours) - **No Windows/Linux evidence**
9. **🆕 Error Messages Not User-Friendly** (4-6 hours) - **Need actionable suggestions**
10. **🆕 Real-World Validation Missing** (8-12 hours) - **No alpha testing done**

**P1 - Should Fix Before Launch** (13-23 hours):
11. **MLX Backend Not Optimized** (8-16 hours) - Apple Silicon performance
12-13. **Install Scripts & Config Validation** (5-7 hours) - User experience polish

**For detailed analysis of ALL 13 gaps**: See [MVP_GAPS.md](MVP_GAPS.md)
**For implementation solutions**: See [BLOCKERS.md](BLOCKERS.md) and [IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md)

### 🎯 Revised Path to v1.0 Production Launch

**Week 1-2**: Core functionality (50-70h) - Backend, download, execution
**Week 3**: Quality & validation (30-50h) - Testing, distribution
**Week 4**: Documentation & polish (20-30h) - User docs, MLX, config
**Week 5-6**: Beta testing & launch prep

**Total Effort**: 100-140 hours of focused development (2.5-3.5 weeks full-time)

### 📊 Test Status

```
Library tests:              53/53  ✅ (100%)
Backend trait contracts:    11/11  ✅ (100%)
Cache contracts:            12/12  ✅ (100%, 2 ignored)
CLI interface contracts:    13/13  ✅ (100%, 1 ignored)
Config contracts:           17/17  ✅ (100%)
E2E CLI tests:              20/20  ✅ (100%)
Embedded backend contracts:  7/11  ⚠️  (64%, 3 failing - BLOCKER)

Total:                     133/136 (98%)
```

### 🤝 Contributing

**Want to help reach v1.0?** We have well-documented, ready-to-implement tasks:

- 🟢 **Good First Issues**: Documentation, error messages, safety patterns
- 🟡 **Medium Difficulty**: Model download, Homebrew setup
- 🔴 **Critical Path**: Embedded backend implementation (HIGHEST IMPACT)

See [IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md) for step-by-step instructions.

### 📚 Project Documentation

| Document | Purpose | Priority |
|----------|---------|----------|
| **[MVP_GAPS.md](MVP_GAPS.md)** | **Complete MVP gap analysis - 13 gaps discovered** | **READ FIRST** ⭐ |
| [PROJECT_STATUS.md](PROJECT_STATUS.md) | Complete project assessment - honest 70% complete status | High |
| [BLOCKERS.md](BLOCKERS.md) | Original 4 critical blockers with detailed solutions | High |
| [IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md) | Step-by-step code implementation instructions | High |
| [ROADMAP.md](ROADMAP.md) | Detailed 5-phase plan to production (842 lines) | Medium |
| [TECH_DEBT.md](TECH_DEBT.md) | Known issues and improvement opportunities | Medium |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute to the project | Medium |

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

**Built with Rust** | **Safety First** | **Open Source**

> **Note**: This is an active development project. Features and APIs are subject to change. See the [specs](specs/) directory for detailed design documentation.