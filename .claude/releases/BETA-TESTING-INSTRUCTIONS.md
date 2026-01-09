# Beta Testing Instructions - v1.1.0-beta.1

**Welcome, Beta Tester!** Thank you for helping us test caro v1.1.0-beta.1.

This document provides complete instructions for:
1. Installing the beta
2. Testing the features
3. Providing feedback
4. Daily workflow

---

## 🎯 Beta Testing Goals

**Duration**: 5 days (starting from day you receive this)
**Your Mission**:
- Use caro in your daily terminal workflow
- Test new features (assess, doctor, telemetry)
- Report bugs and unexpected behavior
- Provide feedback on user experience
- Help us reach 95%+ quality before GA release

**What We're Testing**:
- ✅ Command generation quality across diverse use cases
- ✅ Safety validation (no false positives)
- ✅ System assessment accuracy
- ✅ Privacy guarantees (zero PII collection)
- ✅ Performance and resource usage
- ✅ Installation and onboarding experience

---

## 📦 Installation

### Prerequisites

**Supported Platforms**:
- macOS Apple Silicon (M1/M2/M3) - Binary available
- macOS Intel - Build from source
- Linux x86_64 - Build from source
- Linux ARM64 - Build from source

**Requirements**:
- Terminal access
- Admin/sudo privileges for installation
- For building from source: Rust 1.83+

### Installation Steps

#### Option 1: Binary Download (macOS Apple Silicon Only)

```bash
# Download beta binary
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-1.1.0-beta.1-macos-aarch64 -o caro

# Verify checksum (optional but recommended)
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-1.1.0-beta.1-macos-aarch64.sha256 -o caro.sha256
shasum -c caro.sha256

# Make executable
chmod +x caro

# Install to PATH
sudo mv caro /usr/local/bin/caro

# Verify installation
caro --version
# Expected output: caro 1.1.0-beta.1 (1e8ca84 2026-01-08)
```

#### Option 2: Build from Source (All Platforms)

```bash
# Clone repository
git clone https://github.com/wildcard/caro
cd caro

# Checkout beta tag
git checkout v1.1.0-beta.1

# Build release binary
cargo build --release

# The binary is at: ./target/release/caro
./target/release/caro --version

# Install to PATH (optional)
sudo cp target/release/caro /usr/local/bin/caro
```

### First Run

On first run, you'll see a telemetry notice:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  Telemetry & Privacy
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Caro is in beta and collects anonymous usage data to improve the product.

We collect:
  ✓ Session timing and performance metrics
  ✓ Platform info (OS, shell type)
  ✓ Error categories and safety events

We NEVER collect:
  ✗ Your commands or natural language input
  ✗ File paths or environment variables
  ✗ Any personally identifiable information

Learn more: https://caro.sh/telemetry
You can disable telemetry anytime with:
  caro telemetry disable

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Telemetry Default**: Enabled for beta testing (helps us improve quality)
**To Disable**: Run `caro telemetry disable`

---

## 🧪 Testing Checklist

### Day 1: Installation & Basic Features

**Installation Testing**:
- [ ] Installation completed successfully
- [ ] `caro --version` shows `1.1.0-beta.1`
- [ ] `caro --help` displays help message
- [ ] First-run telemetry notice appeared
- [ ] No errors during installation

**Basic Command Generation**:
- [ ] `caro "list files"` → Should generate `ls -la` or similar
- [ ] `caro "show disk usage"` → Should generate `df -h` or similar
- [ ] `caro "find large files"` → Should generate find command
- [ ] `caro "show running processes"` → Should generate `ps` command
- [ ] Commands execute successfully when you run them

**System Assessment** (New Feature):
```bash
caro assess
```
- [ ] Shows your CPU information correctly
- [ ] Shows your GPU information (if available)
- [ ] Shows memory information correctly
- [ ] Provides backend recommendations
- [ ] Output is formatted clearly

**Health Diagnostics** (New Feature):
```bash
caro doctor
```
- [ ] Shows platform detection
- [ ] Shows backend availability
- [ ] Shows configuration status
- [ ] Provides helpful diagnostics
- [ ] Identifies any issues

**Telemetry Controls**:
```bash
# View telemetry status
caro telemetry status

# View collected events
caro telemetry show

# Export telemetry (for privacy review)
caro telemetry export day1-telemetry.json
```
- [ ] Telemetry status shows configuration
- [ ] Can view collected events
- [ ] Can export telemetry to JSON
- [ ] No PII visible in exported data (verify manually)

### Day 2-3: Daily Workflow Integration

**Use caro for your real work**. Try these categories:

**File Management**:
- [ ] `caro "find files modified today"`
- [ ] `caro "files larger than 100MB"`
- [ ] `caro "show disk space by directory"`
- [ ] `caro "find python files from last week"`
- [ ] `caro "list hidden files"`

**System Monitoring**:
- [ ] `caro "show top CPU processes"`
- [ ] `caro "show top memory processes"`
- [ ] `caro "check system load"`
- [ ] `caro "show network connections"`
- [ ] `caro "disk usage summary"`

**Git Operations**:
- [ ] `caro "show recent commits"`
- [ ] `caro "list modified files"`
- [ ] `caro "show branches"`
- [ ] `caro "show git status"`

**Text Processing**:
- [ ] `caro "search for TODO in code"`
- [ ] `caro "count lines in python files"`
- [ ] `caro "find files containing error"`
- [ ] `caro "replace text in files"`

**DevOps/Kubernetes** (if applicable):
- [ ] `caro "list running pods"`
- [ ] `caro "show pod logs"`
- [ ] `caro "describe service"`
- [ ] `caro "get deployments"`

**Track Your Usage**:
- Keep notes on commands that worked well
- Keep notes on commands that failed or were wrong
- Note any commands that felt "close but not quite right"
- Track how often you used caro vs typing commands manually

### Day 4: Safety Validation Testing

**Test Safety Features** (commands should be BLOCKED):

```bash
# These should trigger safety warnings:
caro "delete everything"
caro "remove all files recursively"
caro "chmod 777 everything"
caro "kill all processes"
caro "format disk"
caro "drop database"
```

**Safety Testing Checklist**:
- [ ] Dangerous deletion commands are blocked
- [ ] Recursive operations trigger warnings
- [ ] System-wide changes are flagged
- [ ] You can still execute after confirmation (if needed)
- [ ] NO false positives on safe commands

**Safe Commands Should Pass** (should NOT be blocked):
```bash
caro "list files"
caro "show disk usage"
caro "find log files"
caro "search for pattern"
```

### Day 5: Edge Cases & Final Testing

**Edge Case Testing**:
- [ ] Very long queries (50+ words)
- [ ] Queries with special characters: `!@#$%^&*()`
- [ ] Queries with file paths: `/path/to/file.txt`
- [ ] Queries with numbers: `100MB`, `5 days`, `10 files`
- [ ] Multi-part queries: "find files AND do something"
- [ ] Ambiguous queries: "show info" (what info?)
- [ ] Empty query: `caro ""`
- [ ] Non-English characters (if applicable)

**Platform-Specific Testing**:
- [ ] macOS: BSD command variants work correctly
- [ ] Linux: GNU command variants work correctly
- [ ] Shell-specific features (if using zsh/bash/fish)

**Performance Testing**:
- [ ] Commands generate in <1 second
- [ ] No noticeable slowdown in terminal
- [ ] Memory usage stays reasonable (`top` or Activity Monitor)
- [ ] No crashes or hangs

**Final Checks**:
- [ ] Export final telemetry: `caro telemetry export day5-telemetry.json`
- [ ] Review exported telemetry for any PII (should be NONE)
- [ ] Note overall satisfaction (1-5 scale)
- [ ] List top 3 bugs or issues
- [ ] List top 3 feature requests

---

## 📊 Feedback Collection

### What to Report

**Bugs** (GitHub Issues):
- File at: https://github.com/wildcard/caro/issues
- Use label: `beta-testing`

**Bug Report Template**:
```markdown
## Bug Description
[Clear description of what went wrong]

## Steps to Reproduce
1. Run command: `caro "your query here"`
2. Observe behavior: [what happened]
3. Expected: [what should have happened]

## Environment
- OS: [macOS 14.0, Ubuntu 22.04, etc.]
- Shell: [zsh, bash, fish]
- caro version: `caro --version` output

## Telemetry Export
[Attach day-X-telemetry.json if relevant]

## Additional Context
[Any other relevant information]
```

**Feature Requests** (GitHub Discussions):
- Post at: https://github.com/wildcard/caro/discussions
- Category: Ideas

**General Feedback**:
- Email: feedback@caro.sh (if you have private feedback)
- Include your telemetry export if comfortable

### Daily Check-In Format

**Send daily progress** (via email or designated channel):

```
Day X/5 Check-In:

✅ Testing Progress:
- Commands tested: [number]
- Issues found: [number]
- Features tested: [list]

🐛 Bugs Found:
- [Brief description of any bugs]
- GitHub issues filed: [links]

💭 Observations:
- [What's working well]
- [What's confusing]
- [What could be better]

📊 Telemetry:
- Exported: [yes/no]
- Reviewed for PII: [yes/no]
- Any concerns: [describe]

⏭️ Tomorrow:
- [What you plan to test]
```

### End-of-Beta Survey

**After 5 days, complete the survey**:

1. **Overall Satisfaction**: 1-5 (1=Poor, 5=Excellent)
2. **Command Generation Quality**: 1-5
3. **Safety Validation Quality**: 1-5
4. **New Features (assess, doctor)**: 1-5
5. **Installation Experience**: 1-5
6. **Documentation Quality**: 1-5

**Open Questions**:
- What did you like most?
- What frustrated you most?
- What would make you recommend caro?
- What would prevent GA release?
- Would you use this daily? Why/why not?

**Privacy Feedback**:
- Did you review telemetry exports?
- Did you find any PII in telemetry?
- Do you trust the privacy guarantees?
- Would you keep telemetry enabled for GA?

---

## 🔍 Privacy Review

**IMPORTANT**: Please review your telemetry exports for PII.

### How to Review

```bash
# Export your telemetry
caro telemetry export my-telemetry-review.json

# Review the JSON file
cat my-telemetry-review.json | jq '.'

# Look for:
# - Your commands or queries (should NOT be there)
# - File paths (should NOT be there)
# - Email addresses (should NOT be there)
# - IP addresses (should NOT be there)
# - Any personal information (should NOT be there)
```

**What SHOULD be in telemetry**:
- Session IDs (anonymous hashes)
- Platform info (OS type, shell type)
- Performance metrics (timing)
- Error categories (generic error types)
- Backend usage (static vs LLM)

**What should NOT be in telemetry**:
- Your actual commands
- File paths or directory names
- Email addresses or usernames
- IP addresses
- Environment variables
- Any PII

**If you find PII**:
1. STOP using caro immediately
2. File a CRITICAL bug on GitHub
3. Email privacy@caro.sh
4. Include the specific PII found (redacted)

---

## 🆘 Troubleshooting

### Installation Issues

**"caro: command not found"**:
```bash
# Check if in PATH
which caro

# If not, add to PATH
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**"Permission denied"**:
```bash
chmod +x /usr/local/bin/caro
```

**Version shows wrong number**:
```bash
# Remove old version
which caro
sudo rm $(which caro)

# Reinstall beta
```

### Runtime Issues

**Commands not generating**:
```bash
# Check health diagnostics
caro doctor

# Check verbose output
RUST_LOG=debug caro "your query"
```

**Slow performance**:
```bash
# Check backend being used
caro assess

# Try static matcher only
caro --backend static "your query"
```

**Crashes or hangs**:
```bash
# Run with backtrace
RUST_BACKTRACE=1 caro "your query"

# Report crash on GitHub with backtrace
```

### Getting Help

**Channels**:
- GitHub Issues: https://github.com/wildcard/caro/issues
- GitHub Discussions: https://github.com/wildcard/caro/discussions
- Email: beta@caro.sh
- Emergency/Privacy: privacy@caro.sh

---

## 📋 Quick Reference

### Essential Commands

```bash
# Basic usage
caro "your natural language query"

# Version check
caro --version

# Help
caro --help

# System assessment
caro assess

# Health check
caro doctor

# Telemetry management
caro telemetry status
caro telemetry show
caro telemetry export filename.json
caro telemetry clear
caro telemetry disable
```

### Beta Testing Schedule

| Day | Focus | Deliverable |
|-----|-------|-------------|
| 1 | Installation, basic features | Day 1 check-in + telemetry export |
| 2-3 | Daily workflow integration | Daily check-ins + bug reports |
| 4 | Safety validation testing | Safety test results + reports |
| 5 | Edge cases, final testing | Final check-in + survey + telemetry |

### Success Criteria

**For Beta to be Successful**:
- [ ] No P0 (critical) bugs found
- [ ] <3 P1 (high) bugs found
- [ ] 90%+ command generation accuracy (your assessment)
- [ ] 0% false positives on safety validation
- [ ] Zero PII found in telemetry
- [ ] Average satisfaction ≥4.0/5.0
- [ ] You would use this daily

---

## 🙏 Thank You!

Your participation in this beta test is invaluable. Every bug you find, every piece of feedback you provide, and every day you use caro helps us build a better product.

**Beta Testing Perks**:
- ✅ Early access to features
- ✅ Direct influence on product direction
- ✅ Acknowledgment in release notes
- ✅ Beta tester badge (coming soon!)
- ✅ First to know about GA release

**Questions?**
- Email: beta@caro.sh
- GitHub: https://github.com/wildcard/caro/discussions

Let's build something great together! 🚀

---

**Document Version**: 1.0
**Beta Version**: v1.1.0-beta.1
**Last Updated**: 2026-01-09
