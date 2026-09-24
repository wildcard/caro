# Caro Shell Command Helper - Claude Skill

A Claude skill that helps you generate safe, POSIX-compliant shell commands using Caro's LLM-powered command generation with comprehensive safety validation.

> **Quick Start**: See [QUICK_START.md](QUICK_START.md) for installation and testing!

## What is this?

This skill integrates [Caro](https://github.com/wildcard/caro) (formerly cmdai) into your Claude Code workflow, enabling:
- 🔍 **Automatic activation** when you need shell command help
- 🛡️ **Safety-first guidance** through 4-tier risk assessment
- 📚 **POSIX compliance education** for portable scripts
- 🚀 **Seamless integration** with existing caro installations
- ⚙️ **Installation assistance** when caro is not available

## Installation

### Quick Install (Recommended)

```bash
# In Claude Code, run:
/plugin install wildcard/caro
```

Done! The skill is now active. See [QUICK_START.md](QUICK_START.md) for test cases.

### Optional: Install Caro CLI

For full functionality, install Caro:

```bash
# Quick install
bash <(curl -sSfL https://setup.caro.sh)

# Or via Cargo
cargo install caro
```

### Manual Installation

1. Clone this repository or copy the `.claude/skills/caro-shell-helper/` directory
2. Place it in one of these locations:
   - `~/.claude/skills/caro-shell-helper/` (user-wide)
   - `.claude/skills/caro-shell-helper/` (project-specific)
3. Restart Claude Code

## How It Works

The skill automatically activates when Claude detects you need shell command assistance:

**You:** "How do I find all PDF files larger than 10MB?"

**Claude (with skill):**
```
I'll help you generate a safe command using Caro.

$ caro "find all PDF files larger than 10MB"

Generated command:
  find . -name "*.pdf" -type f -size +10M

Safety Assessment: ✅ Safe (Green)
- Read-only operation
- POSIX-compliant syntax

This command searches recursively from the current directory for PDF files
larger than 10MB using standard POSIX find utility.

Execute this command? (y/N)
```

## Features

### 1. Smart Detection

Auto-activates when you:
- Ask "how do I [shell operation]..."
- Request "generate a command to..."
- Mention file/directory operations
- Discuss system administration tasks

### 2. Safety Validation

Provides 4-tier risk assessment:
- 🟢 **Safe (Green)**: Read-only operations
- 🟡 **Moderate (Yellow)**: File modifications
- 🟠 **High (Orange)**: Recursive deletions, mass operations
- 🔴 **Critical (Red)**: System destruction, blocked

### 3. POSIX Compliance

Ensures generated commands work across:
- bash, zsh, sh, dash, ksh
- macOS, Linux, Unix systems
- Portable, reliable scripts

### 4. Educational Guidance

Helps you understand:
- What each command does
- Why it's safe (or not)
- POSIX compliance importance
- Better alternatives to dangerous patterns

## Requirements

### Optional: Caro

The skill works best with [Caro](https://github.com/wildcard/caro) installed:

```bash
# Quick install
bash <(curl -sSfL https://setup.caro.sh)

# Or via cargo
cargo install caro
```

**Without caro:** The skill still provides guidance and education about shell commands, but won't generate commands via LLM.

## Usage Examples

### Example 1: Finding Files

**You:** "I need to find all JavaScript files modified this week"

**Skill activates and guides:**
- Checks if Caro is installed
- Generates: `find . -name "*.js" -type f -mtime -7`
- Explains safety level and command parts
- Offers to execute with confirmation

### Example 2: Safe Deletion

**You:** "Delete all .log files older than 30 days"

**Skill provides safety guidance:**
- Warns about deletion being irreversible
- Suggests previewing files first
- Offers interactive deletion option
- Explains risk level

### Example 3: System Operations

**You:** "How do I check disk space?"

**Skill helps:**
- Generates simple, safe command
- Explains what it does
- Confirms it's completely safe
- Executes without concern

## Configuration

### Caro Configuration

Customize behavior in `~/.config/Caro/config.toml`:

```toml
[safety]
enabled = true
level = "moderate"  # strict, moderate, or permissive
require_confirmation = true

[backend]
primary = "embedded"  # embedded, ollama, or vllm
enable_fallback = true
```

### Skill Customization

The skill respects these settings:
- Safety level preferences
- Backend selection
- Output format (json, yaml, plain)

## Documentation

### Included References

- **safety-patterns.md**: Comprehensive list of dangerous command patterns
- **posix-compliance.md**: POSIX vs bash-specific features guide
- **basic-usage.md**: Step-by-step usage examples

### Helper Scripts

- **check-Caro-installed.sh**: Verify Caro availability and guide installation

## Safety Features

### Dangerous Pattern Detection

Blocks or warns about:
- System destruction: `rm -rf /`
- Fork bombs: `:(){ :|:& };:`
- Disk operations: `mkfs`, `dd if=/dev/zero`
- Privilege escalation: `sudo su`, `chmod 777 /`
- Critical path operations: `/bin`, `/usr`, `/etc`

### Safe Alternatives

Suggests safer options for risky operations:
- Preview before deletion
- Interactive confirmations
- Limited scope operations
- Proper backups

## Development

### Project Structure

```
.claude/skills/Caro-shell-helper/
├── SKILL.md                    # Main skill definition
├── README.md                   # This file
├── scripts/
│   └── check-Caro-installed.sh   # Installation checker
├── references/
│   ├── safety-patterns.md      # Dangerous patterns
│   └── posix-compliance.md     # POSIX guide
└── examples/
    └── basic-usage.md          # Usage examples
```

### Contributing

Contributions welcome! Areas for improvement:
- Additional safety patterns
- More usage examples
- Platform-specific guidance
- Enhanced POSIX compliance checks

## Troubleshooting

### Skill Not Activating

1. Verify installation location:
   ```bash
   ls -la ~/.claude/skills/Caro-shell-helper/
   ```

2. Check SKILL.md has proper YAML frontmatter

3. Restart Claude Code

### Caro Not Found

Run the installation checker:
```bash
bash .claude/skills/Caro-shell-helper/scripts/check-Caro-installed.sh
```

Follow installation instructions provided.

## Resources

- **Caro Repository**: https://github.com/wildcard/caro
- **Caro Website**: https://caro.sh
- **Claude Skills Documentation**: https://code.claude.com/docs/en/skills
- **POSIX Standards**: https://pubs.opengroup.org/onlinepubs/9699919799/

## License

AGPL-3.0 - Same as Caro

## Support

- **Issues**: [GitHub Issues](https://github.com/wildcard/caro/issues)
- **Discussions**: [GitHub Discussions](https://github.com/wildcard/caro/discussions)
- **Documentation**: See `/specs` in Caro repository

## About Caro

Caro (caro) is a Rust CLI tool that converts natural language to safe POSIX shell commands using local LLMs. Features:
- 🚀 Single binary, instant startup
- 🧠 Local LLM inference (MLX for Apple Silicon)
- 🛡️ 52 pre-compiled safety patterns
- 📦 Zero external dependencies
- 🌐 Cross-platform support

Learn more at [caro.sh](https://caro.sh)

---

**Built with safety in mind** | **POSIX-compliant** | **Open Source**
