#!/usr/bin/env bash
#
# Caro Setup Script
# 
# Usage:
#   bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
#   bash <(wget -qO- https://setup.caro.sh)

set -e

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

# Install via cargo
install_via_cargo() {
    say "Installing via cargo..."
    
    # Detect if on macOS with Apple Silicon for MLX optimization
    local cargo_features=""
    if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
        say "Detected Apple Silicon - building with MLX optimization"
        cargo_features="--features embedded-mlx"
    fi
    
    if cargo install cmdai $cargo_features; then
        say_success "Installed cmdai successfully"
        return 0
    else
        err "Failed to install via cargo"
    fi
}

# Setup shell alias
setup_alias() {
    local shell_config=""
    local shell_name=""

    # Detect shell
    if [ -n "$BASH_VERSION" ]; then
        shell_name="bash"
        if [ -f "$HOME/.bashrc" ]; then
            shell_config="$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
            shell_config="$HOME/.bash_profile"
        fi
    elif [ -n "$ZSH_VERSION" ]; then
        shell_name="zsh"
        shell_config="${ZDOTDIR:-$HOME}/.zshrc"
    elif [ -n "$FISH_VERSION" ]; then
        shell_name="fish"
        shell_config="$HOME/.config/fish/config.fish"
    else
        # Try to detect from SHELL environment variable
        case "$SHELL" in
            */bash)
                shell_name="bash"
                shell_config="$HOME/.bashrc"
                [ -f "$HOME/.bash_profile" ] && shell_config="$HOME/.bash_profile"
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
                say_warn "Could not detect shell. Please manually add alias:"
                echo "  alias caro='cmdai'"
                return
                ;;
        esac
    fi

    if [ -z "$shell_config" ] || [ ! -f "$shell_config" ]; then
        say_warn "Shell config file not found. Creating $shell_config"
        touch "$shell_config"
    fi

    # Check if alias already exists
    if grep -q "alias caro=" "$shell_config" 2>/dev/null; then
        say "Alias 'caro' already exists in $shell_config"
        return
    fi

    # Add alias
    say "Adding alias 'caro' to $shell_config..."
    echo "" >> "$shell_config"
    echo "# Caro alias" >> "$shell_config"
    echo "alias caro='cmdai'" >> "$shell_config"
    
    say_success "Alias added successfully"
    echo ""
    say "Run 'source $shell_config' or restart your shell to use the alias"
}

# Main installation
main() {
    say "Starting Caro installation..."
    echo ""

    # Check prerequisites
    if ! check_cmd curl && ! check_cmd wget; then
        err "Neither curl nor wget found. Please install one of them."
    fi

    # Check for cargo
    if ! check_cmd cargo; then
        say_warn "Cargo not found. Installing Rust..."
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
    fi

    # Install cmdai
    install_via_cargo
    echo ""

    # Setup alias
    setup_alias
    echo ""

    # Success message
    cat << 'EOF'
═══════════════════════════════════════════════════════════
  Installation Complete! 🎉
═══════════════════════════════════════════════════════════

Usage:
  cmdai "list all files in this directory"
  caro "list all files in this directory"

Execute directly:
  caro -x "show disk usage sorted by size"

Get help:
  caro --help

Examples:
  caro "find all JavaScript files modified in last 7 days"
  caro "show top 5 processes by CPU usage"
  caro "find which process is using port 8080"

Documentation:
  https://caro.sh
  https://github.com/wildcard/caro

═══════════════════════════════════════════════════════════

To start using caro, either:
  • Restart your shell, or
  • Run: source ~/.bashrc (or ~/.zshrc, etc.)

EOF
}

main "$@"
