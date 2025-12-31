# Installer Design and Dotfiles Strategy

## Design Principles

1. **Minimal Footprint**: Single include line in shell rc files
2. **Reversible**: Clean uninstall without residue
3. **XDG Compliant**: Respect standard directory conventions
4. **Non-Destructive**: Never overwrite user content
5. **Idempotent**: Safe to run multiple times
6. **Framework Compatible**: Works with oh-my-zsh, prezto, fisher, etc.

---

## Directory Layout

### XDG Base Directory Specification

```bash
# Configuration (user-editable)
XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
CARO_CONFIG_DIR="$XDG_CONFIG_HOME/caro"

# Data (application-managed)
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CARO_DATA_DIR="$XDG_DATA_HOME/caro"

# Cache (disposable)
XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"
CARO_CACHE_DIR="$XDG_CACHE_HOME/caro"

# Runtime (session-specific)
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp}"
CARO_RUNTIME_DIR="$XDG_RUNTIME_DIR/caro-$UID"
```

### Complete Directory Structure

```
$XDG_CONFIG_HOME/caro/           # ~/.config/caro/
├── config.toml                   # Main configuration
├── shell/                        # Shell integration scripts
│   ├── bash.init                 # bash initialization
│   ├── zsh.init                  # zsh initialization
│   ├── sh.init                   # POSIX sh initialization
│   └── fish/                     # fish configuration
│       ├── conf.d/
│       │   └── caro.fish         # Auto-loaded config
│       └── functions/
│           └── __caro_*.fish     # Helper functions
├── patterns/                     # Custom safety patterns
│   └── custom.toml               # User-defined patterns
└── policies/                     # Custom policies
    └── default.toml              # Policy configuration

$XDG_DATA_HOME/caro/             # ~/.local/share/caro/
├── history/                      # Command history (encrypted)
├── corrections/                  # Learned corrections
└── sessions/                     # Session logs

$XDG_CACHE_HOME/caro/            # ~/.cache/caro/
├── models/                       # Cached LLM models
└── suggestions/                  # Suggestion cache

$XDG_RUNTIME_DIR/                # /run/user/$UID/ or /tmp/
└── caro-$UID.sock               # IPC socket
```

---

## Installation Process

### Overview Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    caro shell install                        │
├─────────────────────────────────────────────────────────────┤
│ 1. Detect shells in use                                     │
│ 2. Create XDG directories                                   │
│ 3. Copy shell integration scripts                           │
│ 4. Add source line to shell rc files                       │
│ 5. Validate installation                                    │
│ 6. Show next steps                                          │
└─────────────────────────────────────────────────────────────┘
```

### Shell Detection

```bash
detect_active_shells() {
    local shells=()

    # Check user's login shell
    local login_shell=$(basename "$SHELL")
    shells+=("$login_shell")

    # Check for additional shells in use
    [[ -f "$HOME/.bashrc" ]] && shells+=("bash")
    [[ -f "$HOME/.zshrc" ]] && shells+=("zsh")
    [[ -d "$HOME/.config/fish" ]] && shells+=("fish")

    # Deduplicate
    printf '%s\n' "${shells[@]}" | sort -u
}
```

### RC File Modifications

#### What We Add

A **single, marked block** that sources our integration:

```bash
# bash: ~/.bashrc
# >>> caro shell integration >>>
# Do not edit this block manually. Managed by 'caro shell install'.
[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/bash.init" ] && \
    source "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/bash.init"
# <<< caro shell integration <<<
```

```zsh
# zsh: ~/.zshrc
# >>> caro shell integration >>>
# Do not edit this block manually. Managed by 'caro shell install'.
[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/zsh.init" ] && \
    source "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/zsh.init"
# <<< caro shell integration <<<
```

```fish
# fish: ~/.config/fish/config.fish
# Note: fish auto-loads from conf.d/, but we add a check anyway
# >>> caro shell integration >>>
if test -f "$XDG_CONFIG_HOME/caro/shell/fish/conf.d/caro.fish"
    source "$XDG_CONFIG_HOME/caro/shell/fish/conf.d/caro.fish"
end
# <<< caro shell integration <<<
```

#### Placement Strategy

1. **bash/zsh**: Add near end of file (after other plugins)
2. **fish**: Use conf.d auto-loading (preferred) or config.fish
3. **Check for existing**: Never add duplicate blocks

### Install Command

```bash
#!/usr/bin/env bash
# caro shell install

set -euo pipefail

CARO_VERSION="1.0.0"
MARKER_START="# >>> caro shell integration >>>"
MARKER_END="# <<< caro shell integration <<<"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# XDG directories
CARO_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/caro"
CARO_DATA="${XDG_DATA_HOME:-$HOME/.local/share}/caro"
CARO_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/caro"

install_shell_integration() {
    info "Installing Caro shell integration..."

    # 1. Create directories
    mkdir -p "$CARO_CONFIG/shell" "$CARO_CONFIG/policies" "$CARO_CONFIG/patterns"
    mkdir -p "$CARO_DATA/history" "$CARO_DATA/corrections" "$CARO_DATA/sessions"
    mkdir -p "$CARO_CACHE/models" "$CARO_CACHE/suggestions"

    # 2. Copy shell scripts from installation source
    # (In practice, these come from the caro binary or a known location)
    local source_dir
    source_dir=$(dirname "$(command -v caro)")/shell-scripts
    if [[ ! -d "$source_dir" ]]; then
        # Fallback: embedded in binary, extract
        caro --extract-shell-scripts "$CARO_CONFIG/shell"
    else
        cp -r "$source_dir"/* "$CARO_CONFIG/shell/"
    fi

    # 3. Fish-specific: also install to fish conf.d
    if [[ -d "$HOME/.config/fish" ]]; then
        mkdir -p "$HOME/.config/fish/conf.d" "$HOME/.config/fish/functions"
        ln -sf "$CARO_CONFIG/shell/fish/conf.d/caro.fish" "$HOME/.config/fish/conf.d/"
        for func in "$CARO_CONFIG/shell/fish/functions"/__caro_*.fish; do
            [[ -f "$func" ]] && ln -sf "$func" "$HOME/.config/fish/functions/"
        done
        success "Installed fish integration (auto-loaded from conf.d)"
    fi

    # 4. Add source lines to rc files
    for shell in bash zsh; do
        local rc_file
        case "$shell" in
            bash) rc_file="$HOME/.bashrc" ;;
            zsh)  rc_file="$HOME/.zshrc" ;;
        esac

        if [[ -f "$rc_file" ]]; then
            add_to_rc "$shell" "$rc_file"
        else
            info "Skipping $shell (no $rc_file found)"
        fi
    done

    # 5. Create default config if not exists
    if [[ ! -f "$CARO_CONFIG/config.toml" ]]; then
        create_default_config
        success "Created default configuration"
    fi

    # 6. Validate installation
    validate_installation

    # 7. Show next steps
    show_next_steps
}

add_to_rc() {
    local shell="$1"
    local rc_file="$2"

    # Check if already installed
    if grep -q "$MARKER_START" "$rc_file" 2>/dev/null; then
        info "Caro integration already present in $rc_file"
        return 0
    fi

    # Backup original
    cp "$rc_file" "$rc_file.caro-backup.$(date +%Y%m%d%H%M%S)"

    # Generate integration block
    local init_file="\${XDG_CONFIG_HOME:-\$HOME/.config}/caro/shell/${shell}.init"

    cat >> "$rc_file" << EOF

$MARKER_START
# Do not edit this block manually. Managed by 'caro shell install'.
[ -f "$init_file" ] && \\
    source "$init_file"
$MARKER_END
EOF

    success "Added Caro integration to $rc_file"
}

create_default_config() {
    cat > "$CARO_CONFIG/config.toml" << 'EOF'
# Caro Shell Integration Configuration
# See https://github.com/wildcard/caro for documentation

[shell]
# Safety level: "off", "passive", "active"
safety_level = "passive"

# Show fix suggestions after failed commands
show_suggestions = true

# Hotkey to invoke Caro (Ctrl+X Ctrl+C by default)
hotkey = "\\C-x\\C-c"

# Fix-it hotkey (Esc Esc by default)
fixit_hotkey = "\\e\\e"

[daemon]
# Auto-start daemon when shell starts
auto_start = true

# Socket path (default: XDG_RUNTIME_DIR/caro-$UID.sock)
# socket_path = "/custom/path/to/caro.sock"

# IPC timeout in milliseconds
timeout_ms = 50

[privacy]
# What to log (none, commands, errors, all)
logging_level = "errors"

# Redact sensitive patterns from logs
redact_secrets = true

# Command history retention (days, 0 = disabled)
history_retention_days = 30

[llm]
# Enable LLM-powered suggestions (requires model)
enabled = false

# Backend to use (mlx, ollama, vllm)
# backend = "ollama"

# Model to use for suggestions
# model = "codellama:7b"
EOF
}

validate_installation() {
    info "Validating installation..."

    local issues=0

    # Check directories
    for dir in "$CARO_CONFIG" "$CARO_DATA" "$CARO_CACHE"; do
        if [[ ! -d "$dir" ]]; then
            error "Directory not created: $dir"
            ((issues++))
        fi
    done

    # Check shell scripts
    for script in bash.init zsh.init; do
        if [[ ! -f "$CARO_CONFIG/shell/$script" ]]; then
            error "Missing script: $CARO_CONFIG/shell/$script"
            ((issues++))
        fi
    done

    if ((issues > 0)); then
        error "Installation validation failed with $issues issues"
        return 1
    fi

    success "Installation validated successfully"
}

show_next_steps() {
    echo ""
    echo -e "${GREEN}Caro shell integration installed successfully!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Restart your shell or run: source ~/.bashrc  (or ~/.zshrc)"
    echo "  2. Try it: Press Ctrl+X Ctrl+C to open Caro prompt"
    echo "  3. Fix a typo: After a failed command, press Esc Esc"
    echo ""
    echo "Configuration: $CARO_CONFIG/config.toml"
    echo "Documentation: https://github.com/wildcard/caro"
    echo ""
}

# Run installer
install_shell_integration "$@"
```

---

## Uninstallation Process

```bash
#!/usr/bin/env bash
# caro shell uninstall

set -euo pipefail

MARKER_START="# >>> caro shell integration >>>"
MARKER_END="# <<< caro shell integration <<<"

CARO_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/caro"
CARO_DATA="${XDG_DATA_HOME:-$HOME/.local/share}/caro"
CARO_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/caro"

uninstall_shell_integration() {
    echo "Uninstalling Caro shell integration..."

    # 1. Remove source lines from rc files
    for rc_file in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [[ -f "$rc_file" ]]; then
            remove_from_rc "$rc_file"
        fi
    done

    # 2. Remove fish symlinks
    rm -f "$HOME/.config/fish/conf.d/caro.fish"
    rm -f "$HOME/.config/fish/functions/__caro_"*.fish

    # 3. Stop daemon if running
    if command -v caro >/dev/null 2>&1; then
        caro daemon stop 2>/dev/null || true
    fi

    # 4. Remove socket
    rm -f "${XDG_RUNTIME_DIR:-/tmp}/caro-$UID.sock"

    # 5. Ask about data removal
    echo ""
    read -p "Remove configuration and data? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$CARO_CONFIG" "$CARO_DATA" "$CARO_CACHE"
        echo "Removed all Caro data"
    else
        echo "Configuration preserved at: $CARO_CONFIG"
    fi

    echo ""
    echo "Caro shell integration uninstalled."
    echo "Restart your shell or run: source ~/.bashrc"
}

remove_from_rc() {
    local rc_file="$1"

    if ! grep -q "$MARKER_START" "$rc_file" 2>/dev/null; then
        echo "No Caro integration found in $rc_file"
        return 0
    fi

    # Create backup
    cp "$rc_file" "$rc_file.caro-uninstall-backup.$(date +%Y%m%d%H%M%S)"

    # Remove marked block using sed
    sed -i.bak "/$MARKER_START/,/$MARKER_END/d" "$rc_file"
    rm -f "$rc_file.bak"

    echo "Removed Caro integration from $rc_file"
}

uninstall_shell_integration "$@"
```

---

## Dotfiles Manager Compatibility

### Users with Dotfiles Managers

For users who sync dotfiles with tools like:
- **chezmoi**
- **yadm**
- **stow**
- **dotbot**
- **home-manager**

#### Recommended Pattern

Instead of auto-modifying rc files, provide a snippet for users to add:

```bash
# For dotfiles users: add this to your synced .bashrc/.zshrc
[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/bash.init" ] && \
    source "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/bash.init"
```

#### Integration with chezmoi

```toml
# chezmoi.toml
[data]
caro_enabled = true
```

```bash
# .bashrc.tmpl
{{ if .caro_enabled -}}
# Caro shell integration
[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/bash.init" ] && \
    source "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/bash.init"
{{ end -}}
```

---

## Framework Compatibility

### oh-my-zsh Integration

Option 1: Source from .zshrc (recommended)
```zsh
# In ~/.zshrc, after oh-my-zsh is loaded
source "$ZSH/oh-my-zsh.sh"
# ... then Caro integration
source "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/zsh.init"
```

Option 2: As oh-my-zsh plugin
```bash
# Install as plugin
mkdir -p "$ZSH_CUSTOM/plugins/caro"
ln -s "$CARO_CONFIG/shell/zsh.init" "$ZSH_CUSTOM/plugins/caro/caro.plugin.zsh"

# Add to plugins in .zshrc
plugins=(... caro)
```

### prezto Integration

```zsh
# In .zpreztorc or .zshrc
source "${XDG_CONFIG_HOME:-$HOME/.config}/caro/shell/zsh.init"
```

### fisher (fish) Integration

```fish
# Already uses conf.d auto-loading, no extra config needed
# Symlinks installed to ~/.config/fish/conf.d/
```

### oh-my-fish Integration

```fish
# Works with conf.d, or install as plugin:
# ~/.config/fish/conf.d/caro.fish is auto-loaded
```

---

## Upgrade Process

```bash
# caro shell upgrade

upgrade_shell_integration() {
    echo "Upgrading Caro shell integration..."

    # 1. Backup current scripts
    local backup_dir="$CARO_CONFIG/shell.backup.$(date +%Y%m%d%H%M%S)"
    mv "$CARO_CONFIG/shell" "$backup_dir"

    # 2. Install new scripts
    caro --extract-shell-scripts "$CARO_CONFIG/shell"

    # 3. Re-link fish if needed
    if [[ -d "$HOME/.config/fish/conf.d" ]]; then
        ln -sf "$CARO_CONFIG/shell/fish/conf.d/caro.fish" "$HOME/.config/fish/conf.d/"
    fi

    # 4. Preserve user customizations (if any)
    if [[ -f "$backup_dir/custom.sh" ]]; then
        cp "$backup_dir/custom.sh" "$CARO_CONFIG/shell/"
    fi

    # 5. Restart daemon
    caro daemon restart

    echo "Upgrade complete. Restart your shell to apply changes."
}
```

---

## Doctor Command

Diagnostic tool for troubleshooting:

```bash
# caro doctor

doctor() {
    echo "Caro Shell Integration Diagnostics"
    echo "==================================="
    echo ""

    # Check caro binary
    echo -n "Caro binary: "
    if command -v caro >/dev/null; then
        echo "$(command -v caro) ($(caro --version 2>/dev/null || echo 'unknown'))"
    else
        echo "NOT FOUND"
    fi

    # Check shell scripts
    echo ""
    echo "Shell scripts:"
    for script in bash.init zsh.init sh.init; do
        local path="$CARO_CONFIG/shell/$script"
        echo -n "  $script: "
        if [[ -f "$path" ]]; then
            echo "OK ($path)"
        else
            echo "MISSING"
        fi
    done

    # Check fish integration
    echo -n "  fish: "
    if [[ -f "$HOME/.config/fish/conf.d/caro.fish" ]]; then
        echo "OK (symlinked)"
    elif [[ -f "$CARO_CONFIG/shell/fish/conf.d/caro.fish" ]]; then
        echo "AVAILABLE (not symlinked)"
    else
        echo "MISSING"
    fi

    # Check RC file integration
    echo ""
    echo "RC file integration:"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        local name=$(basename "$rc")
        echo -n "  $name: "
        if [[ -f "$rc" ]]; then
            if grep -q "caro shell integration" "$rc" 2>/dev/null; then
                echo "INSTALLED"
            else
                echo "NOT INSTALLED"
            fi
        else
            echo "FILE NOT FOUND"
        fi
    done

    # Check daemon
    echo ""
    echo -n "Daemon status: "
    if caro daemon status >/dev/null 2>&1; then
        echo "RUNNING"
    else
        echo "NOT RUNNING"
    fi

    # Check socket
    local socket="${XDG_RUNTIME_DIR:-/tmp}/caro-$UID.sock"
    echo -n "IPC socket: "
    if [[ -S "$socket" ]]; then
        echo "OK ($socket)"
    else
        echo "NOT FOUND"
    fi

    # Check config
    echo ""
    echo -n "Configuration: "
    if [[ -f "$CARO_CONFIG/config.toml" ]]; then
        echo "OK ($CARO_CONFIG/config.toml)"
    else
        echo "DEFAULT (no custom config)"
    fi

    # Environment
    echo ""
    echo "Environment:"
    echo "  CARO_DISABLE: ${CARO_DISABLE:-not set}"
    echo "  CARO_SOCKET: ${CARO_SOCKET:-not set (using default)}"
    echo "  CARO_DEBUG: ${CARO_DEBUG:-not set}"

    # Recommendations
    echo ""
    echo "==================================="
}
```
