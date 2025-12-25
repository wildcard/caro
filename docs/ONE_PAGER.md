# cmdai - One Page Overview

## What is it?

**cmdai** turns natural language into safe shell commands using local AI.

```bash
$ cmdai "find PDFs larger than 10MB in Downloads"
→ find ~/Downloads -name "*.pdf" -size +10M -ls
```

## The Problem

❌ Running AI-generated commands is **dangerous**
❌ One wrong command = lost data, corrupted system
❌ Existing tools don't validate safety or explain risks

## Our Solution

✅ **Local AI** - No cloud, works offline, private
✅ **Safety First** - Blocks dangerous patterns automatically
✅ **User Control** - Explains risks, requires confirmation
✅ **Fast** - Single binary, <2s inference on Apple Silicon

## How It Works

```
1. You type: "delete old log files"
   ↓
2. AI generates: find . -name "*.log" -mtime +30 -delete
   ↓
3. Safety check: ⚠️ MODERATE risk (permanent deletion)
   ↓
4. Explanation: "Deletes log files older than 30 days"
   ↓
5. You confirm: y/N
   ↓
6. Command runs (only if you approve)
```

## Safety Levels

| Level | What It Blocks | Example |
|-------|----------------|---------|
| 🟢 **Safe** | Nothing | `ls`, `cat`, `grep` |
| 🟡 **Moderate** | Asks confirmation | `rm *.log`, `mv files/` |
| 🟠 **High** | Strong warning | `rm -rf folder`, `chmod 777` |
| 🔴 **Critical** | Blocked entirely | `rm -rf /`, fork bombs |

## Key Features

### 🛡️ Built-in Safety
- Pattern matching for dangerous commands
- POSIX compliance validation
- Path safety checking
- No accidental system damage

### 🧠 Smart Inference
- Local LLM (MLX, Ollama, vLLM)
- Optimized for Apple Silicon
- Works offline
- Multiple backend support

### 👤 User-Friendly
- Plain English explanations
- Color-coded risk levels
- Dry-run preview mode
- Interactive confirmation

### ⚡ Performance
- Single binary (<50MB)
- <100ms startup time
- <2s first inference (M1 Mac)
- Zero dependencies

## Architecture

```
User → CLI → [Backend] → Security Check → Validation → User Confirmation → Execute
              (AI)        (Risk Level)    (POSIX)       (y/N)
```

**Sub-Agents:**
1. **Backend Engine** - Generates commands (AI)
2. **Security Analyst** - Assesses risk
3. **Safety Validator** - Checks compliance
4. **User Guide** - Explains and confirms

## Technology

- **Language**: Rust (fast, safe, single binary)
- **AI Backends**: MLX (Apple Silicon), Ollama, vLLM
- **Safety**: Pattern matching + rule engine
- **Interface**: Command-line (clap)

## Example Session

```bash
$ cmdai "compress all images in current directory"

Generated command:
  find . -type f \( -name "*.jpg" -o -name "*.png" \) -exec convert {} {}.compressed.jpg \;

Risk Level: MODERATE ⚠️

What it does:
  • Finds all JPG and PNG files
  • Compresses each one using ImageMagick
  • Creates new .compressed.jpg files

Warnings:
  • Uses disk space for compressed versions
  • Requires ImageMagick to be installed
  • Process may take time for many files

Execute this command? (y/N) _
```

## Use Cases

| Scenario | cmdai Command | Result |
|----------|---------------|--------|
| **File Search** | "find Python files modified today" | `find . -name "*.py" -mtime 0` |
| **System Info** | "show disk usage by directory" | `du -sh */ | sort -h` |
| **Git Ops** | "list uncommitted changes" | `git status --short` |
| **Data Processing** | "count lines in all text files" | `find . -name "*.txt" -exec wc -l {} +` |
| **Cleanup** | "remove files older than 90 days" | Asks confirmation, explains risk |

## Installation

```bash
# From source
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release

# Run
./target/release/cmdai "your prompt here"
```

## Configuration

```toml
# ~/.config/cmdai/config.toml

[safety]
level = "moderate"  # strict | moderate | permissive
require_confirmation = true

[backend]
primary = "mlx"     # mlx | ollama | vllm
enable_fallback = true

[output]
use_color = true
verbose = false
```

## Why cmdai?

| Aspect | cmdai | Traditional Shell | Other AI Tools |
|--------|-------|-------------------|----------------|
| **Safety** | ✅ Built-in validation | ❌ No protection | ⚠️ Limited checks |
| **Privacy** | ✅ Local, offline | ✅ Local | ❌ Cloud APIs |
| **Speed** | ✅ <2s inference | ✅ Instant | ⚠️ API latency |
| **Learning** | ✅ Explains commands | ❌ Assumes knowledge | ⚠️ Minimal context |
| **Risk** | ✅ Color-coded levels | ❌ User responsible | ⚠️ Limited awareness |

## Future Roadmap

### Phase 1 (Current)
- ✅ Core CLI structure
- ✅ Safety validation
- ✅ Multiple backends
- 🚧 Command execution

### Phase 2 (Next)
- 📅 Dry-run simulation
- 📅 Command history
- 📅 Learning from usage
- 📅 Performance optimization

### Phase 3 (Future)
- 📅 Community ratings
- 📅 Multi-step goals
- 📅 Shell script generation
- 📅 Advanced context awareness

## Community & Contribution

**Open Source**: AGPL-3.0 License
**Repository**: github.com/wildcard/cmdai
**Stack**: Rust, MLX, Tokio, Clap

**Contribute:**
- Add safety patterns
- Implement backends
- Improve explanations
- Write tests
- Enhance docs

## Quick Start for Developers

```bash
# Build
cargo build --release

# Test
cargo test

# Run with debug
RUST_LOG=debug cargo run -- "list files"

# Format
cargo fmt

# Lint
cargo clippy -- -D warnings
```

## Core Philosophy

1. **Safety Over Speed** - Never compromise user safety
2. **Transparency** - Always explain what and why
3. **User Agency** - Users must confirm risky operations
4. **Privacy First** - Local execution, no data leaves machine
5. **Community Driven** - Learn from collective experience

## Contact & Support

- 🐛 **Issues**: GitHub Issues
- 💡 **Ideas**: GitHub Discussions
- 📖 **Docs**: `/docs` directory
- 🤝 **Contributing**: See CONTRIBUTING.md

---

## TL;DR

**cmdai** = Safe AI-powered shell command generator

- 💬 Natural language → Shell commands
- 🛡️ Automatic safety validation
- 🧠 Local AI (no cloud needed)
- 👤 User always in control
- ⚡ Fast, single binary

**Example:**
```bash
cmdai "what's using the most disk space?"
→ du -sh */ | sort -hr | head -10
```

---

**Built with Rust** | **Safety First** | **Open Source** | **Privacy Focused**

Try it: `cargo install cmdai` (coming soon)
