---
title: Beta Testing Profiles
description: Available beta testing personas for systematic caro validation
---

## Overview

Beta testing profiles are **persona-based templates** that represent different types of caro users. Each profile defines a specific environment, skill level, and set of use cases to test.

Using profiles ensures we test caro across diverse scenarios without gaps or redundancy.

## How to Use Profiles

1. **Choose a profile** that matches your environment or interests
2. **Set up your environment** to match the profile's specifications
3. **Follow the testing workflow** as described in the [Beta Testing Guide](/contributing/beta-testing)
4. **Report findings** using the profile ID (e.g., bt_001) in issue reports

You can also **create your own profile** by following the same structure.

---

## Available Profiles

### bt_001: Alex (Terminal Novice)

**Focus**: First-time CLI users, basic installation, getting started experience

**Environment**:
- OS: macOS 14.3
- Shell: zsh
- Tools: curl, python3 (no Rust, no Homebrew)
- Skill: Novice - copies commands from tutorials

**Use Cases**:
- Install caro from website instructions
- Generate first command from natural language
- Understand safety warnings
- Get help when stuck

**Patience**: Very low - gives up after 1-2 failures

**Testing Value**: Validates onboarding and documentation clarity

---

### bt_002: Jordan (Power CLI User)

**Focus**: Advanced users, power features, scripting integration

**Environment**:
- OS: Linux (Arch)
- Shell: bash with custom aliases
- Tools: git, vim, tmux, jq, fzf
- Skill: Expert - writes shell scripts daily

**Use Cases**:
- Integrate caro into existing shell workflows
- Use `--output json` for scripting
- Test edge cases and complex commands
- Verify POSIX compliance

**Patience**: High - will debug issues

**Testing Value**: Validates advanced features and integration

---

### bt_003: Sam (Windows Developer)

**Focus**: Windows/WSL compatibility, cross-platform behavior

**Environment**:
- OS: Windows 11 with WSL2 (Ubuntu 22.04)
- Shell: PowerShell and bash (WSL)
- Tools: VS Code, Docker Desktop, Git for Windows
- Skill: Intermediate - comfortable with both Windows and Linux

**Use Cases**:
- Install on Windows vs WSL
- Test path handling (backslash vs forward slash)
- Verify line ending compatibility
- Test Windows-specific commands

**Patience**: Medium - expects some friction

**Testing Value**: Validates cross-platform support

---

### bt_004: Alex (Corporate Developer)

**Focus**: Restricted environments, proxy/firewall, security policies

**Environment**:
- OS: Ubuntu 20.04 LTS
- Shell: bash
- Network: Behind corporate proxy
- Permissions: No sudo access
- Tools: Limited by IT policy

**Use Cases**:
- Install without admin privileges
- Work behind proxy/firewall
- Use airgapped (offline) mode
- Verify supply chain security

**Patience**: Low - blocked by corporate policy

**Testing Value**: Validates enterprise scenarios

---

### bt_005: Casey (DevOps Engineer)

**Focus**: CI/CD integration, automation, non-interactive use

**Environment**:
- OS: Linux (multi-distro: Ubuntu, CentOS, Alpine)
- Shell: bash (scripting focus)
- Tools: Jenkins, GitHub Actions, Docker, Kubernetes
- Skill: Expert - automation mindset

**Use Cases**:
- Run in CI/CD pipelines
- Use `--quiet` and `--execute` flags
- Verify idempotent behavior
- Test exit codes and error handling

**Patience**: Very high - will debug thoroughly

**Testing Value**: Validates automation and non-interactive usage

---

### bt_006: Riley (Data Scientist)

**Focus**: Data processing commands, Python/conda workflows

**Environment**:
- OS: Linux (Ubuntu 22.04)
- Shell: bash
- Tools: conda, Jupyter, pandas, numpy, GPU drivers
- Skill: Intermediate - Python expert, shell beginner

**Use Cases**:
- Generate commands for CSV/JSON manipulation
- Find files by pattern for data pipelines
- Process large datasets
- Batch operations

**Patience**: Medium - will try a few times

**Testing Value**: Validates data science workflows

---

### bt_007: Yuki (Japanese Developer)

**Focus**: Internationalization, Unicode handling, non-English UX

**Environment**:
- OS: macOS Sonoma
- Shell: zsh
- Locale: ja-JP
- Tools: Standard macOS tools
- Skill: Intermediate

**Use Cases**:
- Work with Japanese filenames and paths
- Test Unicode in prompts and outputs
- Verify error messages are clear
- Test non-ASCII characters

**Patience**: Medium

**Testing Value**: Validates i18n and Unicode support

---

### bt_008: Morgan (Fish Shell User)

**Focus**: Non-POSIX shell compatibility

**Environment**:
- OS: macOS Sonoma
- Shell: **fish** (not POSIX)
- Tools: tmux, neovim, Docker, AWS CLI
- Skill: Expert - fish power user

**Use Cases**:
- Verify fish-specific syntax
- Test completions in fish
- Validate environment variable handling
- Check `set` vs `export` differences

**Patience**: High - understands shell quirks

**Testing Value**: Validates non-POSIX shell support

---

### bt_009: Jamie (Accessibility User)

**Focus**: Screen reader, keyboard-only navigation, accessible output

**Environment**:
- OS: macOS Sonoma
- Shell: zsh with VoiceOver screen reader
- Input: Keyboard only (no mouse)
- Tools: Standard macOS accessibility tools
- Skill: Intermediate - experienced screen reader user

**Use Cases**:
- Navigate output with screen reader
- Use keyboard shortcuts
- Verify clear audio descriptions
- Test structured output formats

**Patience**: Medium - knows accessibility standards

**Testing Value**: Validates accessibility compliance

---

### bt_010: Chris (SSH-Only Remote Admin)

**Focus**: Offline/airgapped environments, legacy systems, user-space installs

**Environment**:
- OS: CentOS 7 (legacy, old glibc)
- Shell: bash
- Access: SSH-only (no GUI)
- Network: High latency, intermittent
- Permissions: No sudo, user-space only

**Use Cases**:
- Install without admin privileges
- Work in airgapped environment
- Handle old system libraries
- Verify binary compatibility

**Patience**: Very high - used to workarounds

**Testing Value**: Validates offline and legacy support

---

## Creating Your Own Profile

If your environment isn't covered, create a custom profile:

### Profile Template

```markdown
### bt_XXX: [Your Name] ([Role/Focus])

**Focus**: [What you're testing]

**Environment**:
- OS: [Operating system and version]
- Shell: [Shell type and version]
- Tools: [Relevant tools installed]
- Skill: [novice/intermediate/expert]

**Use Cases**:
1. [First use case]
2. [Second use case]
3. [Third use case]

**Patience**: [low/medium/high]

**Testing Value**: [What this profile validates]
```

### Submit Your Profile

Share your profile in [GitHub Discussions](https://github.com/wildcard/caro/discussions) or submit a PR adding it to `.claude/skills/unbiased-beta-tester/examples/preset-profiles.md`.

---

## Profile Coverage Matrix

| Profile | OS | Shell | Network | Skill | Focus Area |
|---------|-----|-------|---------|-------|------------|
| bt_001 | macOS | zsh | Normal | Novice | Onboarding |
| bt_002 | Linux | bash | Normal | Expert | Power features |
| bt_003 | Windows/WSL | PowerShell/bash | Normal | Intermediate | Cross-platform |
| bt_004 | Linux | bash | Restricted | Intermediate | Enterprise |
| bt_005 | Linux (multi) | bash | Normal | Expert | Automation |
| bt_006 | Linux | bash | Normal | Intermediate | Data science |
| bt_007 | macOS | zsh | Normal | Intermediate | i18n/Unicode |
| bt_008 | macOS | fish | Normal | Expert | Non-POSIX shell |
| bt_009 | macOS | zsh | Normal | Intermediate | Accessibility |
| bt_010 | CentOS | bash | Limited | Expert | Airgapped/legacy |

---

## Resources

- [Beta Testing Guide](/contributing/beta-testing) - How to test
- [GitHub Issues](https://github.com/wildcard/caro/issues) - Report findings
- [ROADMAP](https://github.com/wildcard/caro/blob/main/ROADMAP.md) - Current testing cycles
