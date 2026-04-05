<div align="center">
  <img src="website/public/caro-pixel.png" alt="Caro" width="120">
  <h1>caro</h1>
  <h3>Natural Language. Safe Commands.</h3>
  <p><em>Every AI assistant can hallucinate <code>rm -rf /</code>. Caro doesn't.</em></p>

  [![Crates.io](https://img.shields.io/crates/v/caro.svg)](https://crates.io/crates/caro)
  [![Downloads](https://img.shields.io/crates/d/caro)](https://crates.io/crates/caro)
  [![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://opensource.org/licenses/AGPL-3.0)
  [![CI](https://github.com/wildcard/caro/workflows/CI/badge.svg)](https://github.com/wildcard/caro/actions)
</div>

---

**caro** converts natural language into safe POSIX shell commands using local LLMs. 52+ dangerous command patterns are blocked before they reach your terminal. No cloud, no API keys, no data leaving your machine.

<div align="center">
  <img src="https://caro.sh/caro-prompting.gif" alt="Caro demo - natural language to shell commands" width="700">
</div>

## Install

```bash
# macOS (Homebrew)
brew install wildcard/tap/caro

# Any platform (Cargo)
cargo install caro

# Or download a binary
curl -fsSL https://setup.caro.sh | bash
```

## Quick Start

```bash
$ caro "list all PDF files in Downloads folder larger than 10MB"

Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (Y)es / (n)o / (e)dit: y
```

## Why Caro?

| Feature | Caro | ChatGPT / Claude paste | ShellGPT | Manual typing |
|---------|------|----------------------|----------|---------------|
| Safety validation | 52+ patterns | None | None | You |
| Runs 100% offline | Yes | No | No | Yes |
| API key required | No | Yes | Yes | No |
| Platform-aware | Yes | No | No | You |
| Single binary | Yes | N/A | No | N/A |

## Features

- **Local LLM inference** — Embedded models optimized for Apple Silicon (MLX) and CPU
- **Safety-first** — 52+ pre-compiled patterns block destructive commands before execution
- **Platform-aware** — Detects your OS, architecture, shell, and available commands
- **Single binary** — No dependencies, no runtime, just download and run
- **Multiple backends** — Built-in MLX/CPU, plus Ollama and vLLM support
- **Smart refinement** — 2-iteration agentic loop for platform-specific command generation
- **Interactive** — Edit generated commands in your shell before executing

## How It Works

```
User: "show top 5 processes by CPU"
  ↓
Context Detection: macOS 14.2, arm64, zsh
  ↓
Iteration 1: Generate with platform rules
  ↓
Safety Check: 52+ patterns validated ✓
  ↓
Result: ps aux | sort -nrk 3,3 | head -6
```

## Installation

### macOS

| Method | Command |
|--------|---------|
| Homebrew | `brew install wildcard/tap/caro` |
| Cargo | `cargo install caro` |
| Cargo + MLX GPU | `cargo install caro --features embedded-mlx` |
| Binary | `curl -fsSL https://setup.caro.sh \| bash` |

### Linux

| Method | Command |
|--------|---------|
| Cargo | `cargo install caro` |
| Binary (x86_64) | `curl -fsSL https://setup.caro.sh \| bash` |
| Binary (ARM64) | `curl -fsSL https://setup.caro.sh \| bash` |

Download from the [releases page](https://github.com/wildcard/caro/releases/latest) for all platforms.

### Windows

| Method | Command |
|--------|---------|
| Cargo | `cargo install caro` |
| Binary | Download from [releases](https://github.com/wildcard/caro/releases/latest) |

### Build from Source

```bash
git clone https://github.com/wildcard/caro.git
cd caro && cargo build --release
```

**Prerequisites**: Rust 1.83+, CMake. For Apple Silicon GPU: Xcode.

## Usage

```bash
# Generate a command
caro "compress all images in current directory"

# Force a specific backend
caro --backend ollama "find large log files"

# JSON output for scripting
caro --output json "show disk usage"

# Dry run (don't execute)
caro --dry-run "clean temporary files"

# Auto-confirm execution
caro --confirm "remove old log files"
```

### CLI Options

| Option | Description |
|--------|-------------|
| `-s, --shell <SHELL>` | Target shell (bash, zsh, fish, sh, powershell) |
| `-b, --backend <BACKEND>` | Inference backend (embedded, ollama, vllm) |
| `-m, --model-name <NAME>` | Model name (e.g., codellama:7b) |
| `--safety <LEVEL>` | Safety level (strict, moderate, permissive) |
| `-o, --output <FORMAT>` | Output format (json, yaml, plain) |
| `-y, --confirm` | Auto-confirm execution |
| `-x, --execute` | Execute after validation |
| `--dry-run` | Show plan without running |
| `--force-llm` | Bypass static pattern matcher |

### Shell Integration

For the best experience, add shell integration for inline editing of generated commands:

```bash
# zsh (~/.zshrc)
eval "$(caro init zsh)"

# bash (~/.bashrc)
eval "$(caro init bash)"

# fish (~/.config/fish/config.fish)
caro init fish | source
```

### Configuration

```bash
caro config set backend ollama
caro config set model-name codellama:7b
caro config set safety strict
caro config show
```

Config file: `~/.config/caro/config.toml` | Priority: CLI flags > env vars > config file > auto-detect

### System Assessment

```bash
caro assess              # Detect hardware and recommend models
caro assess --export json --output assessment.json
```

## Safety

All generated commands pass through safety validation before reaching your terminal:

| Risk Level | Color | Behavior |
|------------|-------|----------|
| Safe | Green | No confirmation needed |
| Moderate | Yellow | Confirmation in strict mode |
| High | Orange | Confirmation in moderate mode |
| Critical | Red | Blocked in strict mode |

**52+ patterns** covering: filesystem destruction, fork bombs, disk operations, privilege escalation, critical path protection, and more. See [`src/safety/patterns.rs`](src/safety/patterns.rs) for the full library.

## Architecture

```
caro/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── inference/            # LLM backend trait + implementations
│   │   ├── embedded_backend  # Local MLX/CPU inference
│   │   ├── ollama_backend    # Ollama API
│   │   └── vllm_backend      # vLLM API
│   ├── safety/               # Command validation (52+ patterns)
│   ├── agent/                # Agentic context loop
│   ├── platform/             # OS/arch/shell detection
│   └── cache/                # Model caching
```

### Backend Configuration

```toml
# ~/.config/caro/config.toml
[backend]
primary = "embedded"    # or "ollama", "vllm"
enable_fallback = true

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"

[backend.vllm]
base_url = "http://localhost:8000"
model_name = "codellama/CodeLlama-7b-hf"
```

## Privacy & Telemetry

Telemetry is **disabled by default**. All inference runs locally. No data leaves your machine unless you explicitly enable opt-in analytics.

```bash
caro telemetry status     # Check current state
caro telemetry show       # View collected data
caro config set telemetry.enabled false  # Disable
```

See [docs/TELEMETRY.md](docs/TELEMETRY.md) for full privacy policy.

## Contributing

We're building the safety layer for AI-to-terminal interactions.

- **New to open source?** Start with our [First-Time Contributors Guide](FIRST_TIME_CONTRIBUTORS.md) and [curated beginner issues](.github/first-time-issues/README.md)
- **Experienced?** Check the [roadmap](ROADMAP.md) and [contributing guide](CONTRIBUTING.md)
- **Domain expert?** [Submit safety patterns](https://github.com/wildcard/caro/issues/new?template=safety_pattern.yml) or [share use cases](https://github.com/wildcard/caro/issues/new?template=use_case.yml)

## Community

- [GitHub Discussions](https://github.com/wildcard/caro/discussions) — Questions, ideas, support
- [Issues](https://github.com/wildcard/caro/issues) — Bug reports, feature requests
- [Documentation](https://caro.sh) — Full docs and guides
- [Roadmap](ROADMAP.md) — What's next

## License

[AGPL-3.0](LICENSE) — Commercial use, modification, and distribution allowed. Network use requires source disclosure.

The Kyaro character artwork in `assets/kyaro/` is separately licensed. See [assets/kyaro/README.md](assets/kyaro/README.md) for terms.

## Acknowledgments

[MLX](https://github.com/ml-explore/mlx) | [vLLM](https://github.com/vllm-project/vllm) | [Ollama](https://ollama.ai) | [Hugging Face](https://huggingface.co) | [clap](https://github.com/clap-rs/clap)

---

<div align="center">
  <strong>Built with Rust</strong> · <strong>Safety First</strong> · <strong>100% Local</strong>
  <br><br>
  <a href="https://caro.sh">Website</a> · <a href="https://docs.caro.sh">Docs</a> · <a href="https://github.com/wildcard/caro/discussions">Community</a>
</div>

<sub>The `caro` crate name was generously provided by its previous maintainer. If you're looking for the original "creation-addressed replicated objects" project, it remains available at [crates.io/crates/caro/0.7.1](https://crates.io/crates/caro/0.7.1).</sub>
