# Installation Diversity & Integration Options Analysis

> Inspired by [Beads INSTALLING.md](https://github.com/steveyegge/beads/blob/main/docs/INSTALLING.md)
> Analysis Date: 2026-01-12

## Executive Summary

This analysis compares caro's current installation options with best-in-class CLI tools like Beads, identifying gaps and proposing improvements to maximize adoption across developer environments.

---

## Current State: Caro Installation Options

### What We Have (Good)

| Method | Status | Notes |
|--------|--------|-------|
| Install Script | ✅ | Auto-detects platform, cargo/binary fallback |
| Pre-built Binaries | ✅ | All major platforms (macOS, Linux, Windows) |
| Cargo Install | ✅ | Full features, MLX optimization |
| Build from Source | ✅ | For developers |
| Agent-assisted Install | ✅ | JSON output for AI agents |
| SHA256 Checksums | ✅ | Security verification |

### What We're Missing (Gaps)

| Method | Priority | Adoption Impact |
|--------|----------|-----------------|
| Homebrew Tap | P0 | ~40% of macOS devs prefer brew |
| AUR Package | P1 | Arch Linux power users |
| Windows Package Managers | P1 | Scoop, Winget, Chocolatey |
| Editor Integrations | P0 | Claude Code, Cursor, VS Code, Aider |
| MCP Server | P1 | Claude Desktop, restricted envs |
| Git Hooks | P2 | Workflow automation |
| Claude Code Plugin | P0 | Slash commands (`/caro`) |

---

## Inspired Improvements

### 1. Package Manager Distribution

#### Homebrew Tap (macOS/Linux) - Priority P0

**Why**: ~40% of macOS developers use Homebrew as primary package manager. Reduces friction to zero.

```bash
# Future usage
brew tap wildcard/caro
brew install caro

# Updates
brew upgrade caro
```

**Implementation**:
```ruby
# Formula: caro.rb
class Caro < Formula
  desc "AI-powered shell command generation from natural language"
  homepage "https://caro.sh"
  license "AGPL-3.0"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-macos-silicon"
    sha256 "..." # Auto-generated
  elsif OS.mac?
    url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-macos-intel"
    sha256 "..."
  else
    url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-linux-amd64"
    sha256 "..."
  end

  def install
    bin.install "caro-#{version}-*" => "caro"
  end

  test do
    assert_match "caro", shell_output("#{bin}/caro --version")
  end
end
```

#### AUR Package (Arch Linux) - Priority P1

**Why**: Arch Linux users are power users who influence adoption in tech communities.

```bash
# Future usage
yay -S caro-bin    # Pre-built binary
yay -S caro-git    # Build from source
```

**PKGBUILD** (caro-bin):
```bash
pkgname=caro-bin
pkgver=1.1.0
pkgrel=1
pkgdesc="AI-powered shell command generation from natural language"
arch=('x86_64' 'aarch64')
url="https://github.com/wildcard/caro"
license=('AGPL-3.0')
provides=('caro')
conflicts=('caro' 'caro-git')

source_x86_64=("https://github.com/wildcard/caro/releases/download/v${pkgver}/caro-${pkgver}-linux-amd64")
source_aarch64=("https://github.com/wildcard/caro/releases/download/v${pkgver}/caro-${pkgver}-linux-arm64")

package() {
    install -Dm755 "${srcdir}/caro-${pkgver}-linux-*" "${pkgdir}/usr/bin/caro"
}
```

#### Windows Package Managers - Priority P1

**Scoop** (developer-friendly):
```json
{
    "version": "1.1.0",
    "description": "AI-powered shell command generation from natural language",
    "homepage": "https://caro.sh",
    "license": "AGPL-3.0",
    "architecture": {
        "64bit": {
            "url": "https://github.com/wildcard/caro/releases/download/v1.1.0/caro-1.1.0-windows-amd64.exe",
            "hash": "sha256:..."
        }
    },
    "bin": "caro-1.1.0-windows-amd64.exe -> caro.exe"
}
```

**Winget**:
```yaml
PackageIdentifier: wildcard.caro
PackageVersion: 1.1.0
PackageName: caro
Publisher: Caro Team
License: AGPL-3.0
ShortDescription: AI-powered shell command generation
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/wildcard/caro/releases/download/v1.1.0/caro-1.1.0-windows-amd64.exe
    InstallerType: portable
```

---

### 2. Editor & Tool Integrations

#### `caro setup` Command - Priority P0

Beads offers `bd setup claude`, `bd setup cursor`, etc. We should do the same:

```bash
# Configure for different editors/tools
caro setup claude      # Claude Code integration
caro setup cursor      # Cursor AI integration
caro setup vscode      # VS Code tasks/keybindings
caro setup aider       # Aider AI integration
caro setup atuin       # Atuin shell history integration
caro setup zsh         # Zsh aliases and completion
caro setup bash        # Bash aliases and completion
caro setup fish        # Fish shell integration

# List available integrations
caro setup --list

# Remove integration
caro setup --remove claude
```

**Implementation Vision**:

```rust
// src/commands/setup.rs
pub enum Integration {
    Claude,
    Cursor,
    VSCode,
    Aider,
    Atuin,
    Zsh,
    Bash,
    Fish,
}

impl Integration {
    pub fn setup(&self) -> Result<(), SetupError> {
        match self {
            Integration::Claude => setup_claude_code(),
            Integration::Cursor => setup_cursor(),
            Integration::VSCode => setup_vscode(),
            // ...
        }
    }
}
```

#### Claude Code Integration

**What it does**:
- Configures hooks for session start context injection
- Sets up slash commands (`/shell`, `/safe-command`)
- Adds keyboard shortcuts

**Generated Config** (`~/.claude/hooks.toml`):
```toml
[[hooks]]
name = "caro-session-start"
event = "session_start"
command = "caro context --json"
inject_as = "system_context"

[[hooks]]
name = "caro-pre-execute"
event = "pre_execute"
pattern = "^caro "
command = "caro validate --stdin"
```

#### Cursor Integration

**What it does**:
- Adds Cursor-compatible commands
- Configures terminal integration

**Generated** (`.cursor/settings.json`):
```json
{
  "terminal.integrated.profiles.osx": {
    "caro-shell": {
      "path": "zsh",
      "args": ["-c", "caro repl"]
    }
  },
  "keyboard.shortcuts": {
    "ctrl+shift+c": "caro.generateCommand"
  }
}
```

#### VS Code Integration

**What it does**:
- Adds tasks for command generation
- Creates keybindings
- Installs companion extension (future)

**Generated** (`.vscode/tasks.json`):
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Generate Shell Command",
      "type": "shell",
      "command": "caro",
      "args": ["${input:prompt}"],
      "presentation": {
        "reveal": "always",
        "panel": "shared"
      }
    }
  ],
  "inputs": [
    {
      "id": "prompt",
      "type": "promptString",
      "description": "What command do you need?"
    }
  ]
}
```

---

### 3. MCP Server - Priority P1

**Why**: For Claude Desktop and environments without direct shell access.

```bash
# Install MCP server
uv tool install caro-mcp
# or
pip install caro-mcp
```

**Configuration** (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "caro": {
      "command": "uvx",
      "args": ["caro-mcp"],
      "env": {
        "CARO_SAFETY_LEVEL": "strict"
      }
    }
  }
}
```

**MCP Server Capabilities**:
- `caro.generate` - Generate shell command from natural language
- `caro.validate` - Validate command safety
- `caro.explain` - Explain what a command does
- `caro.assess` - System assessment
- `caro.history` - Command history

**Trade-offs** (from Beads insight):
> "The CLI + Hooks approach minimizes context overhead and latency compared to MCP, making it preferable when shell access exists. MCP becomes necessary only in restricted environments."

---

### 4. Claude Code Plugin - Priority P0

**Installation**:
```bash
# Within Claude Code
/plugin install caro
```

**Slash Commands**:
```bash
/caro:generate "list large files"     # Generate command
/caro:safe "rm old logs"              # Safe command with validation
/caro:explain "find . -name '*.log'"  # Explain command
/caro:assess                          # System assessment
/caro:doctor                          # Health check
```

**Implementation**: Create as Claude Code plugin with manifest:
```json
{
  "name": "caro",
  "version": "1.1.0",
  "description": "AI-powered shell command generation",
  "commands": [
    {
      "name": "generate",
      "description": "Generate shell command from natural language",
      "args": ["prompt"]
    },
    {
      "name": "safe",
      "description": "Generate and validate safe command",
      "args": ["prompt"]
    }
  ],
  "requires": {
    "cli": "caro"
  }
}
```

---

### 5. Git Hooks Integration - Priority P2

**Why**: Automatic context syncing and workflow automation.

```bash
# Setup git hooks
caro setup git-hooks

# What it does:
# - pre-commit: Validates any shell scripts in commit
# - post-checkout: Syncs context for branch
# - prepare-commit-msg: Suggests commit messages
```

**Generated** (`.git/hooks/pre-commit`):
```bash
#!/bin/bash
# Caro safety validation for shell scripts

for file in $(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(sh|bash|zsh)$'); do
    if ! caro validate --file "$file" --strict; then
        echo "Safety validation failed for $file"
        exit 1
    fi
done
```

---

### 6. Windows PowerShell Installer - Priority P1

**Why**: Native Windows experience.

```powershell
# One-liner installation
irm https://raw.githubusercontent.com/wildcard/caro/main/install.ps1 | iex
```

**Script Features**:
- Detects architecture (x64, ARM64)
- Downloads appropriate binary
- Adds to PATH
- Sets up PowerShell profile integration
- Configures execution policy warnings

---

## Implementation Roadmap

### Phase 1: Package Managers (Week 1-2)

1. **Homebrew Tap** - Create `homebrew-caro` repo
   - Formula generation in release workflow
   - Auto-update on release

2. **Windows Installers**
   - PowerShell install script
   - Scoop bucket submission
   - Winget manifest submission

### Phase 2: Editor Integrations (Week 2-3)

3. **`caro setup` Command**
   - Shell integrations (zsh, bash, fish)
   - Claude Code hooks
   - VS Code tasks

4. **Claude Code Plugin**
   - Plugin manifest
   - Slash commands
   - Documentation

### Phase 3: Advanced Integration (Week 3-4)

5. **MCP Server**
   - Python MCP server package
   - Claude Desktop config
   - Documentation

6. **AUR Package**
   - PKGBUILD submission
   - Maintainer setup

---

## Comparison Matrix

| Feature | Beads | Caro Current | Caro Proposed |
|---------|-------|--------------|---------------|
| Homebrew | ✅ | ❌ | ✅ Planned |
| Universal Install | ✅ | ✅ | ✅ |
| Go/Cargo Install | ✅ | ✅ | ✅ |
| AUR | ✅ | ❌ | ✅ Planned |
| PowerShell Script | ✅ | ❌ | ✅ Planned |
| Editor Setup Commands | ✅ | ❌ | ✅ Planned |
| Claude Code Plugin | ✅ | ❌ | ✅ Planned |
| MCP Server | ✅ | ❌ | ✅ Planned |
| Git Hooks | ✅ | ❌ | ✅ Planned |
| Agent Install | ❌ | ✅ | ✅ |
| Binary Checksums | ? | ✅ | ✅ |
| Interactive Setup | ? | ✅ | ✅ |

---

## Key Insights from Beads

1. **CLI + Hooks is the Golden Path**: Prioritize shell integration over MCP. MCP should be fallback for restricted environments.

2. **Editor Setup Commands Lower Friction**: `bd setup claude` is genius - one command configures everything.

3. **Plugin System Adds Discoverability**: Claude Code `/plugin install` is native to the workflow.

4. **Package Managers Matter**: Homebrew alone could increase macOS adoption 2-3x.

5. **Trade-off Communication**: Beads clearly explains when to use CLI vs MCP. We should do the same.

---

## Success Metrics

| Metric | Current | Target (3mo) |
|--------|---------|--------------|
| Install methods | 4 | 10+ |
| Editor integrations | 0 | 4 |
| Package managers | 1 (cargo) | 5 |
| One-liner installs | 1 | 4 |
| Setup time (avg) | ~5 min | <1 min |

---

## Next Steps

1. [ ] Create `homebrew-caro` tap repository
2. [ ] Add `install.ps1` PowerShell script
3. [ ] Implement `caro setup` command with shell integrations
4. [ ] Design Claude Code plugin manifest
5. [ ] Prototype MCP server
6. [ ] Submit to AUR, Scoop, Winget

---

*This analysis was created to improve caro's installation diversity and integration options, inspired by the excellent work in [Beads](https://github.com/steveyegge/beads).*
