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

This project is in **active early development**. The architecture and module structure are in place, with implementation ongoing.

### ✅ Completed
- Core CLI structure with comprehensive argument parsing
- Modular architecture with trait-based backends
- **Embedded model backend with MLX (Apple Silicon) and CPU variants** ✨
- **Remote backend support (Ollama, vLLM) with automatic fallback** ✨
- **Terminal sprite animation rendering system** ✨
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
- 🎨 **Terminal animations** - Pixel art sprite rendering with color palettes

> 💡 **New to animations?** Check out the [Animation Quick Start Guide](docs/QUICKSTART_ANIMATIONS.md) or browse the [complete documentation](docs/README.md).

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
│   ├── rendering/          # Terminal sprite animation
│   │   ├── mod.rs          # Rendering system
│   │   ├── sprites.rs      # Sprite data structures
│   │   ├── animator.rs     # Animation engine
│   │   ├── terminal.rs     # Terminal rendering
│   │   └── examples.rs     # Example sprites
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

## 🎨 Sprite Animation System

cmdai includes a powerful terminal-based sprite animation system for rendering pixel art characters using colored Unicode blocks.

### Features
- **Color palettes** with hex color definitions
- **Multi-frame animations** with customizable timing
- **Transparency support** for complex sprites
- **Unicode block rendering** (█) for true pixel-based graphics
- **True color (24-bit RGB)** or 256-color ANSI mode
- **Animation modes**: Play once, loop, or loop N times
- **ANSI art file support** - Parse and render traditional ANSI art files (.ans)
- **SAUCE metadata** - Full support for SAUCE headers in ANSI files
- **DurDraw format** - Modern JSON-based ANSI art format (.dur) with full metadata
- **Aseprite format** - Binary .ase/.aseprite files with layers, animations, and compression

### Example Usage

```rust
use cmdai::rendering::*;

// Create a simple animated sprite
let palette = ColorPalette::from_hex_strings(&[
    "#000000",  // Transparent
    "#FF5733",  // Red
    "#33FF57",  // Green
])?.with_transparent(0);

// Define frames
let frame1 = SpriteFrame::new(4, 4, vec![
    0, 1, 1, 0,
    1, 1, 1, 1,
    0, 1, 1, 0,
    0, 2, 2, 0,
], 200)?;

let sprite = Sprite::new("demo", palette, vec![frame1])?;

// Animate it
let animator = Animator::new();
let mut animation = Animation::new(sprite, AnimationMode::Loop);
animator.play(&mut animation).await?;
```

### Pre-built Examples

The module includes several ready-to-use sprites:
- **Idle Character** - 8x8 humanoid sprite
- **Walking Animation** - 4-frame walk cycle
- **Heart Pulse** - Animated beating heart
- **Spinning Coin** - 3D coin rotation effect
- **Loading Spinner** - Circular loading indicator

### ANSI Art File Support

Load and render traditional ANSI art files:

```rust
use cmdai::rendering::{AnsiParser, TerminalRenderer};

// Load ANSI art file
let (frame, sauce) = AnsiParser::load_file("artwork.ans")?;

// Display metadata
if let Some(metadata) = sauce {
    println!("Title: {}", metadata.title);
    println!("Author: {}", metadata.author);
}

// Render it
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;
```

Supports:
- Full ANSI escape sequence parsing
- SAUCE metadata extraction
- 16-color and 256-color palettes
- Foreground/background colors
- Character preservation (€, ‹, ﬂ, etc.)

### DurDraw Format Support

Load and save modern DurDraw format files:

```rust
use cmdai::rendering::{DurDrawParser, DurDrawFile, DurDrawColor};

// Load DurDraw file
let (frame, metadata) = DurDrawParser::load_with_metadata("artwork.dur")?;

// Display with full colors
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;

// Or create programmatically
let dur = DurDrawFile {
    title: "My Art".to_string(),
    author: "Artist".to_string(),
    width: 10,
    height: 5,
    data: vec![/* cells */],
    palette: vec![/* colors */],
    // ...
};
DurDrawParser::save_file(&dur, "output.dur")?;
```

Features:
- JSON-based, human-readable format
- Multiple color formats (RGB, hex, named, palette)
- Full metadata (title, author, date, group)
- Bidirectional conversion with ANSI

### Aseprite Format Support

Load pixel art directly from Aseprite source files:

```rust
use cmdai::rendering::{AsepriteParser, Animator, Animation, AnimationMode};

// Load Aseprite file
let ase_file = AsepriteParser::load_file("sprite.ase")?;

// Display file information
println!("Dimensions: {}x{}", ase_file.header.width, ase_file.header.height);
println!("Frames: {}", ase_file.header.frames);
println!("Layers: {}", ase_file.layers.len());

// Convert to Sprite for animation
let sprite = AsepriteParser::to_sprite(&ase_file)?;

// Animate it
let animator = Animator::new();
let mut animation = Animation::new(sprite, AnimationMode::Loop);
animator.play(&mut animation).await?;
```

Supported features:
- Binary .ase and .aseprite file formats
- Multiple animation frames with individual durations
- Layer system with visibility and opacity
- Alpha blending for layer compositing
- Zlib-compressed cel data
- Color palettes (RGBA, Grayscale, Indexed modes)
- Raw and linked cel types

### Try the Demos

```bash
# Sprite animation demo
cargo run --example sprite_demo

# ANSI art parsing demo
cargo run --example ansi_art_demo

# DurDraw format demo
cargo run --example durdraw_demo

# Aseprite format demo
cargo run --example aseprite_demo
```

### Documentation

**📚 Complete Animation Documentation**:
- **[Quick Start Guide](docs/QUICKSTART_ANIMATIONS.md)** - Get started in 5 minutes
- **[Animation Guide](docs/ANIMATION_GUIDE.md)** - Complete technical reference (developers)
- **[Designer Guide](docs/DESIGNER_GUIDE.md)** - Workflow for UX designers & artists
- **[Testing Guide](docs/TESTING_ANIMATIONS.md)** - Testing and validation procedures
- **[Documentation Index](docs/README.md)** - Full documentation directory

**For Developers**: Start with the [Quick Start Guide](docs/QUICKSTART_ANIMATIONS.md), then dive into the [Animation Guide](docs/ANIMATION_GUIDE.md) for complete API reference.

**For Designers**: Check out the [Designer Guide](docs/DESIGNER_GUIDE.md) for Aseprite workflows, color palettes, and animation principles.

**For Testing**: See the [Testing Guide](docs/TESTING_ANIMATIONS.md) for validation checklists and debugging procedures.

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

### Dual Licensing: Code and Artwork

This project uses **different licenses** for code and artwork:

#### Source Code License

The **cmdai source code** is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)** - see the [LICENSE](LICENSE) file for details.

**Code License Summary**:
- ✅ Commercial use
- ✅ Modification
- ✅ Distribution
- ✅ Private use
- ⚠️ Network use requires source disclosure
- ⚠️ Same license requirement
- ⚠️ State changes documentation

#### Artwork and Assets License

**Artwork, animations, and visual assets** in the `assets/` directory are **NOT covered by the AGPL-3.0 license**.

Each artist retains copyright over their work and specifies their own license terms:

- 🎨 **Original characters and artwork**: Typically under restrictive licenses (not open source)
- 📁 **Check individual licenses**: See `assets/[artist-name]/LICENSE.md` for specific terms
- ⚠️ **Not redistributable**: Most artwork cannot be used outside of cmdai without permission
- ✅ **Attribution required**: Always credit the original artists

**For full details**, see:
- [Assets Directory README](assets/README.md) - Overview of all contributed artwork
- [Contributing Assets Guide](docs/CONTRIBUTING_ASSETS.md) - How to contribute your artwork
- Individual `LICENSE.md` files in each artist's folder

#### Why Separate Licenses?

This dual-licensing approach:
- ✅ Protects artists' creative work and original characters
- ✅ Allows open-source collaboration on the code
- ✅ Encourages contributions from both developers and artists
- ✅ Ensures proper attribution and copyright respect

**Important**: If you fork or redistribute cmdai, you may need to **exclude artwork** with restrictive licenses or get explicit permission from the artists.

## 🙏 Acknowledgments

### Technology
- [MLX](https://github.com/ml-explore/mlx) - Apple's machine learning framework
- [vLLM](https://github.com/vllm-project/vllm) - High-performance LLM serving
- [Ollama](https://ollama.ai) - Local LLM runtime
- [Hugging Face](https://huggingface.co) - Model hosting and caching
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing

### Artwork Contributors

Thank you to all artists who have contributed to cmdai! 🎨

<!-- Artists will be listed here as they contribute -->
<!-- See assets/README.md for the full list of contributors -->

**Want to contribute artwork?** See the [Contributing Assets Guide](docs/CONTRIBUTING_ASSETS.md).

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