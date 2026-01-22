# Caro - Product Requirements Document

**Version:** 2.0.0
**Status:** Generally Available (GA)
**Last Updated:** January 2026
**Maintainer:** Wildcard ([@bjesuiter](https://github.com/bjesuiter))

---

## Overview

**Caro** is a Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs. It enables developers to describe what they want to accomplish in plain English and receive validated, platform-aware shell commands.

### One-Liner

> Transform natural language into safe, executable shell commands with local AI inference.

### Key Value Propositions

1. **Privacy-First** - All inference happens locally; your commands never leave your machine
2. **Safety-First** - 52+ dangerous command patterns blocked before execution
3. **Platform-Aware** - Automatically detects OS, shell, and available tools
4. **Single Binary** - No dependencies, no Docker, no Python environment required

---

## Problem Statement

### The Pain

Developers frequently need to:
- Remember obscure command-line flags (`find -mtime -7 -type f -name "*.log"`)
- Translate between platforms (BSD vs GNU, macOS vs Linux)
- Avoid dangerous operations (`rm -rf /` vs `rm -rf ./`)
- Chain multiple commands correctly (`find | xargs | grep`)

### Current Solutions Fall Short

| Solution | Problem |
|----------|---------|
| Google/Stack Overflow | Context switching, outdated answers |
| ChatGPT/Claude web | Privacy concerns, no platform awareness |
| tldr/cheat | Still need to know the command name |
| Shell autocomplete | Limited to existing commands |

### Our Solution

Caro bridges the gap between natural language intent and correct, safe shell commands by:
- Running entirely locally (privacy)
- Understanding your specific platform (accuracy)
- Validating commands before execution (safety)
- Learning from corrections (improvement)

---

## User Personas

### Primary: The Pragmatic Developer

**Name:** Alex
**Role:** Full-stack developer
**Experience:** 5+ years
**Platform:** macOS (Apple Silicon)

**Goals:**
- Get working commands quickly without googling
- Trust that commands won't damage their system
- Not memorize obscure flags for rarely-used tools

**Frustrations:**
- Waiting for web-based AI responses
- Commands that don't work on their Mac
- Fear of running destructive commands

**Quote:** *"I know what I want to do, I just can't remember the exact syntax."*

### Secondary: The DevOps Engineer

**Name:** Sam
**Role:** Site Reliability Engineer
**Experience:** 8+ years
**Platform:** Linux servers

**Goals:**
- Generate commands for remote systems safely
- Audit trail for generated commands
- Confidence in safety validation

**Quote:** *"I need to trust this before I run it on prod."*

### Tertiary: The Command-Line Learner

**Name:** Jordan
**Role:** Junior developer
**Experience:** 1 year

**Goals:**
- Learn proper command syntax
- Understand what commands do
- Build terminal confidence

**Quote:** *"I don't even know what command to search for."*

---

## Features

### Core Features (GA)

#### F1: Natural Language to Command

Convert plain English descriptions into shell commands.

```bash
$ caro "find all PDF files larger than 10MB in Downloads"

Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute? (Y)es / (n)o / (e)dit:
```

**Acceptance Criteria:**
- [ ] Accepts natural language input as primary argument
- [ ] Returns syntactically valid shell command
- [ ] Shows command before execution
- [ ] Offers execute/edit/cancel options

#### F2: Safety Validation

Block dangerous commands before they can be executed.

```bash
$ caro "delete everything in root"

[Safety] CRITICAL: Recursive root deletion detected

This command would delete system files:
  rm -rf /

Blocked. Be more specific about what to delete.
```

**Acceptance Criteria:**
- [ ] Detects 52+ dangerous command patterns
- [ ] Blocks CRITICAL risk commands automatically
- [ ] Shows clear explanation of why command was blocked
- [ ] Suggests safer alternatives

**Dangerous Patterns Include:**
- System destruction (`rm -rf /`, `rm -rf ~`)
- Fork bombs (`:(){:|:&};:`)
- Disk operations (`mkfs`, `dd if=/dev/zero`)
- Privilege escalation (`sudo chmod 777 /`)
- Critical path modification (`/bin`, `/usr`, `/etc`)

#### F3: Platform Detection

Automatically detect and adapt to the user's platform.

**Detected Context:**
- Operating System (macOS, Linux, Windows)
- Architecture (x86_64, ARM64/Apple Silicon)
- Shell (bash, zsh, fish, PowerShell)
- Available commands (via `which`)

**Acceptance Criteria:**
- [ ] Detects OS and architecture on startup
- [ ] Identifies user's current shell
- [ ] Generates platform-appropriate commands
- [ ] Uses BSD syntax on macOS, GNU on Linux

#### F4: Multiple Inference Backends

Support various LLM backends for flexibility.

| Backend | Description | Use Case |
|---------|-------------|----------|
| `embedded` | Local MLX/CPU inference | Default, privacy-first |
| `ollama` | Local Ollama server | Custom models |
| `vllm` | Remote vLLM API | Enterprise deployment |

```bash
# Use default embedded backend
caro "list files"

# Use Ollama with specific model
caro --backend ollama --model-name codellama:7b "list files"
```

**Acceptance Criteria:**
- [ ] Embedded backend works without internet
- [ ] Ollama backend connects to local server
- [ ] vLLM backend supports API authentication
- [ ] Automatic fallback on backend failure

#### F5: Interactive Execution

Safe command execution with confirmation.

```bash
$ caro -x "show disk usage"

Generated command:
  df -h

Execute? (Y)es / (n)o / (e)dit: y

Filesystem      Size   Used  Avail  Use%  Mounted on
/dev/disk1s1    466G   234G   198G   55%  /
```

**Acceptance Criteria:**
- [ ] `-x` flag enables execution mode
- [ ] Always prompts before executing
- [ ] `-y` flag auto-confirms (for scripting)
- [ ] Captures and displays command output

#### F6: Shell Integration

Deep integration with user's shell for seamless editing.

```bash
# Add to ~/.zshrc
eval "$(caro init zsh)"

# Now 'edit' mode places command in prompt
$ caro "find rust files"
Execute? (Y)es / (n)o / (e)dit: e
# Command appears in prompt for editing
$ find . -name "*.rs" █
```

**Acceptance Criteria:**
- [ ] Supports bash, zsh, and fish
- [ ] `caro init <shell>` outputs integration script
- [ ] Edit mode places command in prompt
- [ ] Fallback to clipboard if integration not available

### Advanced Features

#### F7: Configuration Management

Persistent settings for customization.

```bash
# Set defaults
caro config set backend ollama
caro config set safety strict
caro config set shell zsh

# View configuration
caro config show
```

**Config File:** `~/.config/caro/config.toml`

```toml
[backend]
primary = "embedded"
enable_fallback = true

[safety]
level = "moderate"
require_confirmation = true

[inference]
shell = "zsh"
```

#### F8: System Assessment

Hardware capability detection for optimal performance.

```bash
$ caro assess

System Assessment:
  CPU: Apple M2 Pro (12 cores)
  RAM: 32GB (28GB available)
  GPU: Apple M2 Pro (19 cores, 16GB unified)

Recommendation:
  Backend: embedded-mlx
  Model: Qwen2.5-Coder-1.5B (quantized)
  Expected latency: <2s
```

#### F9: Output Formats

Multiple output formats for scripting and integration.

```bash
# Plain text (default)
caro "list files"

# JSON for scripting
caro --output json "list files"
{
  "command": "ls -la",
  "confidence": 0.95,
  "risk_level": "safe"
}

# YAML for config generation
caro --output yaml "list files"
```

---

## User Stories

### US-001: Generate Simple Command

**As a** developer
**I want to** describe a task in plain English
**So that** I get the correct shell command without googling

**Acceptance Criteria:**
- Given I run `caro "list all files"`
- When the command is generated
- Then I see `ls -la` (or platform equivalent)
- And I'm prompted to execute or edit

### US-002: Block Dangerous Command

**As a** developer
**I want** dangerous commands to be blocked
**So that** I don't accidentally damage my system

**Acceptance Criteria:**
- Given I run `caro "delete everything"`
- When caro detects destructive intent
- Then the command is blocked
- And I see a warning explaining why
- And I see a safer alternative suggestion

### US-003: Platform-Specific Generation

**As a** macOS user
**I want** commands to use BSD syntax
**So that** they work on my system without modification

**Acceptance Criteria:**
- Given I'm on macOS
- When I run `caro "show file sizes"`
- Then I get `ls -lh` (BSD-compatible)
- And NOT `ls -lh --block-size=M` (GNU-only)

### US-004: Execute with Confirmation

**As a** developer
**I want to** execute commands directly
**So that** I don't need to copy-paste

**Acceptance Criteria:**
- Given I run `caro -x "show current directory"`
- When the command is generated
- Then I see the command and am prompted
- When I confirm with 'y'
- Then the command executes
- And I see the output

### US-005: Edit Before Execute

**As a** developer
**I want to** modify the generated command
**So that** I can fine-tune it before running

**Acceptance Criteria:**
- Given I have shell integration enabled
- When I choose 'edit' option
- Then the command appears in my prompt
- And I can modify it before pressing Enter

### US-006: Work Offline

**As a** privacy-conscious developer
**I want** all inference to happen locally
**So that** my commands never leave my machine

**Acceptance Criteria:**
- Given I'm using the embedded backend
- When I generate commands
- Then no network requests are made
- And generation works without internet

### US-007: Custom Backend

**As a** power user
**I want to** use my own Ollama models
**So that** I can use larger/specialized models

**Acceptance Criteria:**
- Given Ollama is running locally
- When I run `caro --backend ollama "complex query"`
- Then caro connects to Ollama
- And uses my configured model

### US-008: Scripting Integration

**As a** DevOps engineer
**I want** JSON output for automation
**So that** I can integrate caro into scripts

**Acceptance Criteria:**
- Given I run `caro --output json "list files"`
- Then I get valid JSON
- And it includes command, confidence, and risk level
- And I can parse it with `jq`

---

## Technical Requirements

### Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold start | < 500ms | Time to first prompt |
| Command generation | < 2s | End-to-end latency (p95) |
| Safety validation | < 50ms | Pattern matching time |
| Memory usage | < 2GB | Peak during inference |

### Compatibility

| Platform | Support Level |
|----------|---------------|
| macOS (Apple Silicon) | Full (MLX acceleration) |
| macOS (Intel) | Full (CPU inference) |
| Linux (x86_64) | Full (CPU inference) |
| Linux (ARM64) | Full (CPU inference) |
| Windows (x64) | Full (CPU inference) |

### Dependencies

**Runtime:** None (single binary)

**Build Requirements:**
- Rust 1.83+
- CMake (for inference backends)
- Xcode (optional, for MLX on macOS)

### Security

- No telemetry by default (opt-in only)
- No network access in embedded mode
- Commands never sent to external servers
- Local-only model inference

---

## Implementation Guidance

### For AI Agents

When implementing features for caro, follow these patterns:

#### Code Style

```rust
// Use Result types for error handling
async fn generate_command(&self, request: &CommandRequest)
    -> Result<GeneratedCommand, GeneratorError>;

// Prefer async/await for I/O operations
async fn validate_safety(&self, command: &str) -> SafetyResult;

// Use trait-based abstraction for backends
#[async_trait]
trait InferenceBackend {
    async fn generate(&self, prompt: &str) -> Result<String>;
    async fn is_available(&self) -> bool;
}
```

#### Testing

```bash
# Run all tests
cargo test

# Run safety tests specifically
cargo test safety

# Run with coverage
cargo llvm-cov
```

#### Safety Pattern Development

When adding new safety patterns, use TDD:

1. Write failing test for dangerous pattern
2. Add pattern to `src/safety/patterns.rs`
3. Verify test passes
4. Add documentation for pattern

#### Project Structure

```
src/
├── main.rs              # CLI entry point
├── inference/           # Backend implementations
│   ├── mod.rs           # InferenceBackend trait
│   ├── embedded_backend.rs
│   ├── ollama_backend.rs
│   └── vllm_backend.rs
├── safety/              # Safety validation
│   ├── mod.rs
│   ├── patterns.rs      # 52+ dangerous patterns
│   └── validator.rs
├── platform/            # Platform detection
├── prompts/             # LLM prompt templates
└── config/              # Configuration management
```

### For Human Developers

**Getting Started:**

```bash
# Clone and build
git clone https://github.com/wildcard/caro.git
cd caro
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

**Key Files:**

| File | Purpose |
|------|---------|
| `CLAUDE.md` | AI assistant instructions |
| `CONTRIBUTING.md` | Contribution guidelines |
| `src/safety/patterns.rs` | All safety patterns |
| `specs/` | Feature specifications |

---

## Success Metrics

### Quantitative

| Metric | Target | Current |
|--------|--------|---------|
| Test pass rate | > 90% | 93.1% |
| Safety false positives | 0 | 0 |
| Binary size | < 50MB | ~35MB |
| Startup time | < 500ms | ~300ms |

### Qualitative

- Users trust generated commands
- Commands work on first try
- Safety blocks feel appropriate, not annoying
- Platform differences handled transparently

---

## Roadmap

### Completed (v1.x)

- [x] Core CLI with argument parsing
- [x] Multiple inference backends
- [x] Safety validation (52+ patterns)
- [x] Platform detection
- [x] Interactive execution
- [x] Configuration management
- [x] Shell integration
- [x] Multi-platform binaries

### In Progress (v1.2)

- [ ] Model caching optimization
- [ ] Extended safety patterns
- [ ] Performance profiling
- [ ] Telemetry infrastructure (opt-in)

### Planned (v2.0)

- [ ] Intent routing with FunctionGemma
- [ ] Domain-specific prompts
- [ ] Command history learning
- [ ] Multi-step goal completion
- [ ] Plugin system

---

## Resources

- **Website:** [caro.sh](https://caro.sh)
- **Documentation:** [docs.caro.sh](https://docs.caro.sh)
- **Repository:** [github.com/wildcard/caro](https://github.com/wildcard/caro)
- **Crates.io:** [crates.io/crates/caro](https://crates.io/crates/caro)
- **Discussions:** [GitHub Discussions](https://github.com/wildcard/caro/discussions)

---

## License

**AGPL-3.0** - See [LICENSE](https://github.com/wildcard/caro/blob/main/LICENSE)

This PRD is maintained by the Caro team and intended for use with ClawdHub and AI development assistants.
