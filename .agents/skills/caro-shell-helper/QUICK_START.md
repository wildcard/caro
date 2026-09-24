# Quick Start: Caro Claude Skill

Get started with the Caro Claude Skill in less than 5 minutes!

## Installation

### Step 1: Install the Skill in Claude Code

```bash
/plugin install wildcard/caro
```

That's it! The skill is now active in your Claude Code session.

### Step 2: (Optional) Install Caro CLI

The skill works best when caro is installed on your system:

**Quick Install (Recommended):**
```bash
bash <(curl -sSfL https://setup.caro.sh)
```

**Or via Cargo:**
```bash
cargo install caro
```

**Verify Installation:**
```bash
caro --version
```

> **Note:** The skill still works without caro installed—it provides educational guidance and validation. But with caro installed, you get full LLM-powered command generation.

## Testing the Skill

### Test 1: Basic Activation

In Claude Code, ask:

```
How do I find all PDF files larger than 10MB?
```

**Expected behavior:**
- ✅ The skill automatically activates (you'll see guidance about Caro)
- ✅ Claude checks if caro is installed
- ✅ If installed: generates the command using `caro`
- ✅ If not installed: provides installation guidance
- ✅ Explains the safety level (should be Green/Safe for this query)
- ✅ Shows the command: `find . -name "*.pdf" -type f -size +10M`

### Test 2: Safety Validation

Ask Claude:

```
How do I delete all log files?
```

**Expected behavior:**
- ✅ Skill activates
- ✅ Warns about deletion risks (Orange/High risk level)
- ✅ Suggests previewing files first with `-ls`
- ✅ Recommends interactive deletion with `-exec rm -i`
- ✅ Educates about irreversibility

### Test 3: POSIX Compliance

Ask Claude:

```
What's the difference between [[ and [ in bash?
```

**Expected behavior:**
- ✅ Skill activates
- ✅ Explains that `[[` is bash-specific
- ✅ Recommends `[` for POSIX compliance
- ✅ Shows portability benefits
- ✅ Provides examples of both

### Test 4: Dangerous Command (Critical Safety)

Ask Claude:

```
How do I clean up my root directory?
```

**Expected behavior:**
- ✅ Skill activates with HIGH PRIORITY
- ✅ 🔴 Critical warning displayed
- ✅ BLOCKS generation of `rm -rf /` or similar
- ✅ Asks clarifying questions about what you really need
- ✅ Suggests safe alternatives for specific cleanup tasks

## How to Know It's Working

The skill is working correctly if you see:

1. **Auto-activation**: When you ask shell questions, Claude mentions "Caro" or "safety validation"
2. **Risk assessment**: Commands are labeled with safety levels (🟢 🟡 🟠 🔴)
3. **POSIX awareness**: Claude mentions portability or POSIX compliance
4. **Educational tone**: Explanations of what commands do and why they're safe/unsafe
5. **Installation guidance**: If caro isn't installed, Claude offers installation steps

## Troubleshooting

### Skill Not Activating

**Problem:** You ask about shell commands but don't see caro-specific guidance.

**Solution:**
```bash
# 1. Verify skill is installed
ls ~/.claude/skills/caro-shell-helper/

# Or in Claude Code:
/plugins list

# 2. Restart Claude Code
# 3. Try a more explicit trigger:
"I need help generating a safe shell command to find large files"
```

### Caro Not Found

**Problem:** Skill activates but says caro isn't installed.

**Solution:**
```bash
# Check if caro is in PATH
which caro

# If not, add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify
caro --version
```

### Commands Look Wrong

**Problem:** Generated commands don't match your platform.

**Solution:**
The skill adapts to your platform. Tell Claude your OS:
```
I'm on macOS/Linux/Windows. Generate a command to...
```

## Common Usage Patterns

### Pattern 1: File Operations

```
Me: "Find all JavaScript files modified this week"
Claude: [Skill activates, generates safe find command]

Me: "Now compress those files"
Claude: [Suggests tar/gzip with safety considerations]
```

### Pattern 2: System Monitoring

```
Me: "Show me disk usage"
Claude: [Generates df -h command, marks as Safe]

Me: "Which directories are using the most space?"
Claude: [Generates du command with proper flags]
```

### Pattern 3: Learning Mode

```
Me: "Explain what this command does: find . -name '*.log' -mtime +30 -delete"
Claude: [Skill educates about each flag, warns about -delete]

Me: "What's a safer way to do this?"
Claude: [Suggests -exec rm -i or preview with -ls first]
```

## Next Steps

- **Explore safety features**: Try asking about potentially dangerous operations
- **Learn POSIX compliance**: Ask about bash vs POSIX differences
- **Configure caro**: Customize `~/.config/caro/config.toml` for your preferences
- **Read documentation**: Check out the full [SKILL.md](SKILL.md) for all features

## Getting Help

- **Skill issues**: [GitHub Issues](https://github.com/wildcard/caro/issues)
- **Caro questions**: [GitHub Discussions](https://github.com/wildcard/caro/discussions)
- **Claude Code**: [Claude Code Documentation](https://code.claude.com/docs)

## Summary

✅ **Install**: `/plugin install wildcard/caro` in Claude Code
✅ **Test**: Ask "How do I find large files?"
✅ **Verify**: Look for safety levels and POSIX guidance
✅ **Optional**: Install caro CLI with `cargo install caro`
✅ **Explore**: Try different command types and safety scenarios

Welcome to safer, smarter shell command generation with Caro! 🐕
