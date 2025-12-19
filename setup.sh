#!/usr/bin/env bash
#
# Caro Setup Script
#
# Usage:
#   bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
#   bash <(wget -qO- https://setup.caro.sh)
#
# Options:
#   --no-manpage      Skip installing the man page
#   --no-completions  Skip installing shell completions
#   --help            Show this help message

set -e

# Default options (all enabled)
INSTALL_MANPAGE=true
INSTALL_COMPLETIONS=true

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-manpage)
            INSTALL_MANPAGE=false
            shift
            ;;
        --no-completions)
            INSTALL_COMPLETIONS=false
            shift
            ;;
        --help|-h)
            cat << 'HELPEOF'
Caro Setup Script

Usage:
  bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
  bash <(curl -sSfL https://setup.caro.sh) -- [OPTIONS]

Options:
  --no-manpage      Skip installing the man page
  --no-completions  Skip installing shell completions
  --help, -h        Show this help message

Examples:
  # Full installation (default)
  bash <(curl -sSfL https://setup.caro.sh)

  # Install without man page
  bash <(curl -sSfL https://setup.caro.sh) -- --no-manpage

  # Install without completions
  bash <(curl -sSfL https://setup.caro.sh) -- --no-completions

  # Minimal install (binary only, no extras)
  bash <(curl -sSfL https://setup.caro.sh) -- --no-manpage --no-completions
HELPEOF
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Run with --help for usage information"
            exit 1
            ;;
    esac
done

cat << 'EOF'
   ____
  / ___|__ _ _ __ ___
 | |   / _` | '__/ _ \
 | |__| (_| | | | (_) |
  \____\__,_|_|  \___/

Your Terminal's AI Companion
Natural Language → Shell Commands

https://caro.sh
https://github.com/wildcard/caro

═══════════════════════════════════════════════════════════

EOF

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

say() {
    echo -e "${BLUE}→${NC} $1"
}

say_success() {
    echo -e "${GREEN}✓${NC} $1"
}

say_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

say_error() {
    echo -e "${RED}✗${NC} $1"
}

say_skip() {
    echo -e "${CYAN}○${NC} $1 (skipped)"
}

err() {
    say_error "$1"
    exit 1
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

need_cmd() {
    if ! check_cmd "$1"; then
        err "Required command not found: $1"
    fi
}

# Detect platform
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        *)
            err "Unsupported operating system: $(uname -s)"
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="amd64" ;;
        aarch64|arm64)  arch="arm64" ;;
        *)
            err "Unsupported architecture: $(uname -m)"
            ;;
    esac

    echo "${os}-${arch}"
}

# Detect shell and get config file
detect_shell_config() {
    local shell_config=""
    local shell_name=""

    # Detect shell - prioritize $SHELL env var over subprocess shell version vars
    case "$SHELL" in
        */bash)
            shell_name="bash"
            if [ -f "$HOME/.bashrc" ]; then
                shell_config="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_config="$HOME/.bash_profile"
            else
                shell_config="$HOME/.bashrc"
            fi
            ;;
        */zsh)
            shell_name="zsh"
            shell_config="${ZDOTDIR:-$HOME}/.zshrc"
            ;;
        */fish)
            shell_name="fish"
            shell_config="$HOME/.config/fish/config.fish"
            ;;
        *)
            # Fallback to checking version variables
            if [ -n "$ZSH_VERSION" ]; then
                shell_name="zsh"
                shell_config="${ZDOTDIR:-$HOME}/.zshrc"
            elif [ -n "$BASH_VERSION" ]; then
                shell_name="bash"
                if [ -f "$HOME/.bashrc" ]; then
                    shell_config="$HOME/.bashrc"
                elif [ -f "$HOME/.bash_profile" ]; then
                    shell_config="$HOME/.bash_profile"
                else
                    shell_config="$HOME/.bashrc"
                fi
            elif [ -n "$FISH_VERSION" ]; then
                shell_name="fish"
                shell_config="$HOME/.config/fish/config.fish"
            else
                say_warn "Could not detect shell."
                return
            fi
            ;;
    esac

    echo "$shell_name:$shell_config"
}

# Install via cargo
install_via_cargo() {
    say "Installing via cargo..."

    # Detect if on macOS with Apple Silicon for MLX optimization
    local cargo_features=""
    if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
        say "Detected Apple Silicon - building with MLX optimization"
        cargo_features="--features embedded-mlx"
    fi

    if cargo install caro $cargo_features; then
        say_success "Installed caro successfully"
        return 0
    else
        err "Failed to install via cargo"
    fi
}

# Install via pre-built binary
install_via_binary() {
    say "Downloading pre-built binary..."

    local platform repo binary_name install_dir
    repo="wildcard/caro"
    binary_name="caro"
    install_dir="${CARO_INSTALL_DIR:-$HOME/.local/bin}"

    # Detect platform
    case "$(uname -s)" in
        Linux*)
            case "$(uname -m)" in
                x86_64|amd64)   platform="linux-amd64" ;;
                aarch64|arm64)  platform="linux-arm64" ;;
                *) err "Unsupported architecture: $(uname -m)" ;;
            esac
            ;;
        Darwin*)
            case "$(uname -m)" in
                x86_64|amd64)   platform="macos-intel" ;;
                aarch64|arm64)  platform="macos-silicon" ;;
                *) err "Unsupported architecture: $(uname -m)" ;;
            esac
            ;;
        *)
            err "Unsupported operating system: $(uname -s)"
            ;;
    esac

    # Get latest release
    local latest_url="https://api.github.com/repos/${repo}/releases/latest"
    local release_info

    if check_cmd curl; then
        release_info=$(curl -s "$latest_url")
    elif check_cmd wget; then
        release_info=$(wget -qO- "$latest_url")
    else
        err "Neither curl nor wget found"
    fi

    # Extract version
    local version
    version=$(echo "$release_info" | grep '"tag_name":' | sed -E 's/.*"tag_name": "v?([^"]+)".*/\1/')

    if [ -z "$version" ]; then
        err "Could not determine latest version"
    fi

    # Map platform to base asset name
    local base_asset_name
    case "$platform" in
        linux-amd64)    base_asset_name="linux-amd64" ;;
        linux-arm64)    base_asset_name="linux-arm64" ;;
        macos-intel)    base_asset_name="macos-intel" ;;
        macos-silicon)  base_asset_name="macos-silicon" ;;
        *) err "Unsupported platform: $platform" ;;
    esac

    # Try versioned asset name first (v1.0.3+), fall back to non-versioned (v1.0.2)
    local versioned_asset_name="caro-${version}-${base_asset_name}"
    local legacy_asset_name="caro-${base_asset_name}"
    local asset_name="$versioned_asset_name"
    local binary_url="https://github.com/${repo}/releases/download/v${version}/${versioned_asset_name}"

    # Create install directory
    mkdir -p "$install_dir"

    say "Downloading caro v${version} for ${platform}..."

    # Try versioned name first, fall back to legacy name
    local download_success=false
    if check_cmd curl; then
        if curl -fsSL "$binary_url" -o "${install_dir}/${binary_name}" 2>/dev/null; then
            download_success=true
        else
            # Try legacy non-versioned name
            asset_name="$legacy_asset_name"
            binary_url="https://github.com/${repo}/releases/download/v${version}/${legacy_asset_name}"
            say_warn "Versioned binary not found, trying legacy name..."
            curl -fsSL "$binary_url" -o "${install_dir}/${binary_name}" && download_success=true
        fi
    elif check_cmd wget; then
        if wget -qO "${install_dir}/${binary_name}" "$binary_url" 2>/dev/null; then
            download_success=true
        else
            # Try legacy non-versioned name
            asset_name="$legacy_asset_name"
            binary_url="https://github.com/${repo}/releases/download/v${version}/${legacy_asset_name}"
            say_warn "Versioned binary not found, trying legacy name..."
            wget -qO "${install_dir}/${binary_name}" "$binary_url" && download_success=true
        fi
    fi

    if [ "$download_success" = false ]; then
        err "Failed to download binary"
    fi

    # Make executable
    chmod +x "${install_dir}/${binary_name}"

    # Verify with version check
    if "${install_dir}/${binary_name}" --version >/dev/null 2>&1; then
        say_success "Installed caro v${version} successfully"
    else
        err "Binary installed but failed version check"
    fi

    # Add to PATH if needed
    if [[ ":$PATH:" != *":$install_dir:"* ]]; then
        say_warn "$install_dir is not in your PATH"
        say "You may need to restart your shell or add to PATH manually"
    fi

    return 0
}

# Check for legacy cmdai alias and inform user
check_legacy_alias() {
    local shell_info
    shell_info=$(detect_shell_config)
    local shell_config="${shell_info#*:}"

    if [ -z "$shell_config" ]; then
        return
    fi

    if [ ! -f "$shell_config" ]; then
        return
    fi

    # Check if old cmdai alias exists and inform user
    if grep -q "alias caro='cmdai'" "$shell_config" 2>/dev/null; then
        say_warn "Found old 'cmdai' alias in $shell_config"
        say "You can remove it - the binary is now named 'caro' directly"
        echo ""
    fi
}

# Install man page
setup_manpage() {
    if [ "$INSTALL_MANPAGE" != "true" ]; then
        say_skip "Man page installation"
        return
    fi

    say "Setting up man page..."

    # Determine man page directory
    local man_dir=""
    local use_sudo=false

    # Check for user-local man directory first
    if [ -d "$HOME/.local/share/man/man1" ]; then
        man_dir="$HOME/.local/share/man/man1"
    elif [ -d "$HOME/.local/share/man" ]; then
        mkdir -p "$HOME/.local/share/man/man1"
        man_dir="$HOME/.local/share/man/man1"
    elif [ -w "/usr/local/share/man/man1" ]; then
        man_dir="/usr/local/share/man/man1"
    elif [ -d "/usr/local/share/man" ] && check_cmd sudo; then
        man_dir="/usr/local/share/man/man1"
        use_sudo=true
    else
        # Create user-local man directory
        mkdir -p "$HOME/.local/share/man/man1"
        man_dir="$HOME/.local/share/man/man1"
    fi

    # Download man page from repository
    local man_url="https://raw.githubusercontent.com/wildcard/caro/main/docs/man/caro.1"
    local temp_man=$(mktemp)

    if check_cmd curl; then
        if curl -fsSL "$man_url" -o "$temp_man" 2>/dev/null; then
            :
        else
            say_warn "Could not download man page from repository"
            rm -f "$temp_man"
            return
        fi
    elif check_cmd wget; then
        if wget -qO "$temp_man" "$man_url" 2>/dev/null; then
            :
        else
            say_warn "Could not download man page from repository"
            rm -f "$temp_man"
            return
        fi
    else
        say_warn "Neither curl nor wget available for downloading man page"
        return
    fi

    # Install man page
    if [ "$use_sudo" = true ]; then
        say "Installing man page to $man_dir (requires sudo)..."
        if sudo mkdir -p "$man_dir" && sudo cp "$temp_man" "$man_dir/caro.1"; then
            say_success "Man page installed (use 'man caro')"
        else
            say_warn "Failed to install man page with sudo"
        fi
    else
        if cp "$temp_man" "$man_dir/caro.1"; then
            say_success "Man page installed to $man_dir"

            # Check if MANPATH includes the directory
            if [[ ":$MANPATH:" != *":$HOME/.local/share/man:"* ]] && [[ "$man_dir" == "$HOME/.local/share/man/man1" ]]; then
                local shell_info
                shell_info=$(detect_shell_config)
                local shell_config="${shell_info#*:}"

                if [ -n "$shell_config" ] && [ -f "$shell_config" ]; then
                    if ! grep -q "MANPATH.*\.local/share/man" "$shell_config" 2>/dev/null; then
                        echo "" >> "$shell_config"
                        echo "# Caro man page path" >> "$shell_config"
                        echo 'export MANPATH="$HOME/.local/share/man:$MANPATH"' >> "$shell_config"
                        say "Added MANPATH to $shell_config"
                    fi
                fi
            fi
        else
            say_warn "Failed to install man page"
        fi
    fi

    rm -f "$temp_man"
}

# Install shell completions
setup_completions() {
    if [ "$INSTALL_COMPLETIONS" != "true" ]; then
        say_skip "Shell completions"
        return
    fi

    # Check if caro is available
    if ! check_cmd caro; then
        say_warn "caro not found in PATH, skipping completion setup"
        return
    fi

    say "Setting up shell completions..."

    local shell_info
    shell_info=$(detect_shell_config)
    local shell_name="${shell_info%%:*}"
    local shell_config="${shell_info#*:}"

    case "$shell_name" in
        bash)
            setup_bash_completions "$shell_config"
            ;;
        zsh)
            setup_zsh_completions "$shell_config"
            ;;
        fish)
            setup_fish_completions
            ;;
        *)
            say_warn "Unsupported shell for completions: $shell_name"
            say "You can generate completions manually with: caro --completions <shell>"
            ;;
    esac
}

setup_bash_completions() {
    local shell_config="$1"
    local completion_dir="$HOME/.local/share/bash-completion/completions"

    mkdir -p "$completion_dir"

    if caro --completions bash > "$completion_dir/caro" 2>/dev/null; then
        say_success "Bash completions installed to $completion_dir"

        # Source completions in bashrc if not already done
        if [ -n "$shell_config" ] && [ -f "$shell_config" ]; then
            if ! grep -q "bash-completion/completions/caro" "$shell_config" 2>/dev/null; then
                echo "" >> "$shell_config"
                echo "# Caro shell completions" >> "$shell_config"
                echo '[ -f "$HOME/.local/share/bash-completion/completions/caro" ] && source "$HOME/.local/share/bash-completion/completions/caro"' >> "$shell_config"
            fi
        fi
    else
        say_warn "Failed to generate bash completions"
    fi
}

setup_zsh_completions() {
    local shell_config="$1"
    local completion_dir="$HOME/.local/share/zsh/site-functions"

    mkdir -p "$completion_dir"

    if caro --completions zsh > "$completion_dir/_caro" 2>/dev/null; then
        say_success "Zsh completions installed to $completion_dir"

        # Add to fpath if not already there
        if [ -n "$shell_config" ] && [ -f "$shell_config" ]; then
            if ! grep -q "fpath.*\.local/share/zsh/site-functions" "$shell_config" 2>/dev/null; then
                # Add fpath before compinit if possible
                echo "" >> "$shell_config"
                echo "# Caro shell completions" >> "$shell_config"
                echo 'fpath=("$HOME/.local/share/zsh/site-functions" $fpath)' >> "$shell_config"
                say "Added completion directory to fpath in $shell_config"
                say "Run 'autoload -Uz compinit && compinit' or restart your shell"
            fi
        fi
    else
        say_warn "Failed to generate zsh completions"
    fi
}

setup_fish_completions() {
    local completion_dir="$HOME/.config/fish/completions"

    mkdir -p "$completion_dir"

    if caro --completions fish > "$completion_dir/caro.fish" 2>/dev/null; then
        say_success "Fish completions installed to $completion_dir"
    else
        say_warn "Failed to generate fish completions"
    fi
}

# Main installation
main() {
    say "Starting Caro installation..."
    echo ""

    # Show installation options
    echo -e "${CYAN}Installation options:${NC}"
    echo -e "  Man page:    $([ "$INSTALL_MANPAGE" = true ] && echo "${GREEN}yes${NC}" || echo "${YELLOW}no${NC}")"
    echo -e "  Completions: $([ "$INSTALL_COMPLETIONS" = true ] && echo "${GREEN}yes${NC}" || echo "${YELLOW}no${NC}")"
    echo ""

    # Check prerequisites
    if ! check_cmd curl && ! check_cmd wget; then
        err "Neither curl nor wget found. Please install one of them."
    fi

    # Check for cargo
    if check_cmd cargo; then
        # Cargo is available - use it (especially important for Apple Silicon MLX support)
        install_via_cargo
    else
        # No cargo - try binary installation first
        say_warn "Cargo not found."
        echo ""

        # Attempt binary installation
        if install_via_binary 2>/dev/null; then
            # Binary install succeeded
            if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
                echo ""
                say_warn "For Apple Silicon MLX optimization, consider installing Rust:"
                say "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                say "  cargo install caro --features embedded-mlx"
            fi
        else
            # Binary install failed - install Rust as fallback
            say "Pre-built binary not available. Installing Rust..."
            echo ""

            if check_cmd curl; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            elif check_cmd wget; then
                wget -qO- https://sh.rustup.rs | sh -s -- -y
            fi

            # Source cargo env
            if [ -f "$HOME/.cargo/env" ]; then
                # shellcheck source=/dev/null
                . "$HOME/.cargo/env"
            fi

            if ! check_cmd cargo; then
                err "Failed to install Rust. Please install it manually from https://rustup.rs"
            fi

            say_success "Rust installed successfully"
            echo ""

            # Now install via cargo
            install_via_cargo
        fi
    fi
    echo ""

    # Check for legacy alias
    check_legacy_alias
    echo ""

    # Setup man page
    setup_manpage
    echo ""

    # Setup shell completions
    setup_completions
    echo ""

    # Success message
    cat << 'EOF'
═══════════════════════════════════════════════════════════
  Installation Complete! 🎉
═══════════════════════════════════════════════════════════

Usage:
  caro "list all files in this directory"

Execute directly:
  caro -x "show disk usage sorted by size"

Get help:
  caro --help
  man caro

Generate completions for other shells:
  caro --completions bash
  caro --completions zsh
  caro --completions fish
  caro --completions powershell

Examples:
  caro "find all JavaScript files modified in last 7 days"
  caro "show top 5 processes by CPU usage"
  caro "find which process is using port 8080"

Documentation:
  https://caro.sh
  https://github.com/wildcard/caro

═══════════════════════════════════════════════════════════

To start using caro:
  • Restart your shell, or
  • Run: source ~/.bashrc (or ~/.zshrc, etc.)

EOF
}

main "$@"
