# cmdai - Product Overview

> A simple guide for collaborators to understand what we're building and why

## The Problem We're Solving

**Question**: How do you safely let AI convert natural language into shell commands that run on your computer?

**Risk**: A single wrong command could:
- Delete all your files (`rm -rf /`)
- Corrupt your disk (`mkfs /dev/sda`)
- Crash your system (fork bombs)
- Expose sensitive data

**Current Gap**: Existing tools either:
- Execute commands blindly without safety checks
- Require cloud APIs (privacy/latency concerns)
- Lack proper risk assessment and user guidance

## Our Solution: cmdai

**cmdai** is a safety-first CLI tool that acts like a "smart, cautious assistant" for your terminal.

### Simple Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. USER INTENT                                              │
│    "find all PDF files larger than 10MB"                    │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. AI GENERATES COMMAND                                     │
│    find ~/Downloads -name "*.pdf" -size +10M -ls            │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. SAFETY VALIDATION (Rule Engine)                          │
│    • Check for dangerous patterns                           │
│    • Assess risk level (Safe/Moderate/High/Critical)        │
│    • Validate POSIX compliance                              │
│    • Check paths and permissions                            │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. USER GUIDANCE                                            │
│    ✓ Command: find ~/Downloads -name "*.pdf" -size +10M    │
│    ✓ Risk Level: SAFE (Green)                              │
│    ✓ Explanation: "Lists PDF files over 10MB"              │
│    ✓ What it does: "Searches Downloads folder..."          │
│    ? Execute this command? (y/N)                            │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. EXECUTION (with confirmation)                            │
│    Command runs only if user approves                       │
└─────────────────────────────────────────────────────────────┘
```

## Core Features

### 🛡️ Safety First
- **Rule Engine**: Pattern matching for dangerous operations
- **Risk Assessment**: 4 levels (Safe, Moderate, High, Critical)
- **Dry-Run Mode**: Preview commands without executing
- **User Confirmation**: Required for risky operations
- **Blocklist**: Automatic blocking of destructive patterns

### 🧠 Smart & Local
- **Local LLMs**: No cloud dependency, works offline
- **Fast Inference**: <2s on Apple Silicon (MLX optimized)
- **Multiple Backends**: MLX, Ollama, vLLM support
- **Single Binary**: Zero external dependencies

### 🎯 User-Focused
- **Clear Explanations**: Tells you what the command does
- **Risk Warnings**: Highlights potential dangers
- **POSIX Compliant**: Works across shells (bash, zsh, fish)
- **Interactive**: Always asks before dangerous operations

## Key Components (System Architecture)

Based on the multi-agent breakdown, cmdai consists of:

### 1. **Security Analyst** (Safety Module)
```rust
// Validates commands before execution
safety::validate(command) -> RiskLevel
```
- Threat modeling and pattern detection
- Rule engine for command scrutiny
- Dangerous operation blocking

### 2. **CLI Assistant** (User Interface)
```rust
// Generates commands and provides guidance
cli::generate_and_explain(prompt) -> CommandWithGuidance
```
- Command generation from natural language
- Risk explanation and outcome prediction
- Dry-run simulation support

### 3. **Backend System** (Inference Engines)
```rust
// Pluggable LLM backends
trait CommandGenerator {
    async fn generate_command(request) -> Result<Command>;
}
```
- MLX (Apple Silicon, local)
- Ollama (local, cross-platform)
- vLLM (remote, scalable)

### 4. **Configuration Manager**
```rust
// User preferences and safety settings
config::load() -> UserConfig
```
- Safety level configuration (strict/moderate/permissive)
- Backend preferences
- Custom pattern blocklists

## Example Scenarios

### Scenario 1: Safe Command
```bash
$ cmdai "list files in current directory"

Generated command:
  ls -lah

Risk Level: SAFE ✓
What it does: Lists all files with detailed information
Execute? (Y/n) y
```

### Scenario 2: Moderate Risk
```bash
$ cmdai "delete all .log files"

Generated command:
  find . -name "*.log" -delete

Risk Level: MODERATE ⚠️
What it does: Permanently deletes log files in current directory
Warning: This operation cannot be undone
Execute? (y/N)
```

### Scenario 3: Critical Risk (Blocked)
```bash
$ cmdai "remove everything recursively"

Generated command:
  rm -rf /

Risk Level: CRITICAL ⛔
BLOCKED: This command would destroy your entire filesystem
Reason: Matches dangerous pattern - system root deletion
```

## Future Enhancements

### Community-Driven Safety (Planned)
- **Crowdsourced Ratings**: Users vote on command safety
- **Community Insights**: Share experiences and best practices
- **Pattern Contributions**: Expand the safety rule database
- **Usage Analytics**: Learn from safe/unsafe patterns

### Advanced Features (Roadmap)
- Multi-step goal completion (break complex tasks into steps)
- Command history learning (personalized suggestions)
- Shell script generation (for repeated workflows)
- Integration with system monitoring (detect actual impact)

## Why This Matters

### For Users
- **Confidence**: Run AI-generated commands without fear
- **Learning**: Understand what commands do before running them
- **Efficiency**: Natural language is faster than remembering syntax
- **Safety Net**: Prevent accidental destructive operations

### For Developers
- **Transparency**: Open-source safety rules and validation logic
- **Extensibility**: Plugin system for custom backends and validators
- **Performance**: Optimized for local execution, no API costs
- **Privacy**: No command/prompt data leaves your machine

## Technical Highlights

- **Language**: Rust (safety, performance, single binary)
- **Architecture**: Trait-based, async, modular
- **Testing**: TDD with comprehensive safety test suites
- **Platform**: macOS (MLX optimized), Linux, Windows
- **License**: AGPL-3.0 (open source, network use requires disclosure)

## Getting Started as a Collaborator

### Understanding the Codebase
```
src/
├── backends/     → LLM inference engines
├── safety/       → Command validation & risk assessment
├── cli/          → User interface & interaction
└── config/       → Configuration management
```

### Key Contribution Areas
1. **Safety Patterns**: Add dangerous command detection rules
2. **Backend Support**: Implement new LLM backends
3. **User Experience**: Improve explanations and guidance
4. **Testing**: Expand test coverage for edge cases
5. **Documentation**: Help others understand the system

### Quick Start
```bash
# Clone and build
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release

# Run with examples
cargo run -- "your prompt here"

# Run tests
cargo test

# See what's implemented
cargo run -- --show-config
```

## Core Philosophy

1. **Safety Over Speed**: Never compromise user safety for performance
2. **Transparency**: Always explain what commands do and why
3. **User Agency**: Users must explicitly confirm risky operations
4. **Offline-First**: Works without internet, respects privacy
5. **Community-Driven**: Learn from collective experience (future)

---

## Quick Reference

| Aspect | Description |
|--------|-------------|
| **What** | Natural language → Safe shell commands |
| **How** | Local LLM + Rule engine + Risk assessment |
| **Why** | Prevent destructive operations, educate users |
| **Where** | Your terminal, runs locally, no cloud |
| **When** | Any time you need a shell command but unsure of syntax |

## Questions?

- **Architecture**: See `CLAUDE.md` for technical details
- **Safety Rules**: See `src/safety/mod.rs` for pattern definitions
- **Examples**: See `tests/integration/` for end-to-end scenarios
- **Contributing**: See `README.md` for development guidelines

---

**Built with safety in mind** | **Open source** | **Community-driven**
