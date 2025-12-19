#!/usr/bin/env bash
#
# Caro Setup Script
#
# Usage:
#   bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
#   bash <(wget -qO- https://setup.caro.sh)
#
# Options:
#   --no-alias        Skip setting up the 'caro' alias
#   --no-manpage      Skip installing the man page
#   --no-completions  Skip installing shell completions
#   --help            Show this help message

set -e

# Default options (all enabled)
INSTALL_ALIAS=true
INSTALL_MANPAGE=true
INSTALL_COMPLETIONS=true

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-alias)
            INSTALL_ALIAS=false
            shift
            ;;
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
  --no-alias        Skip setting up the 'caro' alias
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
  bash <(curl -sSfL https://setup.caro.sh) -- --no-alias --no-manpage --no-completions
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

    if cargo install cmdai $cargo_features; then
        say_success "Installed cmdai successfully"
        return 0
    else
        err "Failed to install via cargo"
    fi
}

# Setup shell alias
setup_alias() {
    if [ "$INSTALL_ALIAS" != "true" ]; then
        say_skip "Alias setup"
        return
    fi

    local shell_info
    shell_info=$(detect_shell_config)
    local shell_name="${shell_info%%:*}"
    local shell_config="${shell_info#*:}"

    if [ -z "$shell_config" ]; then
        say_warn "Could not detect shell config file. Please manually add alias:"
        echo "  alias caro='cmdai'"
        return
    fi

    if [ ! -f "$shell_config" ]; then
        say_warn "Shell config file not found. Creating $shell_config"
        mkdir -p "$(dirname "$shell_config")"
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
        if sudo mkdir -p "$man_dir" && sudo cp "$temp_man" "$man_dir/caro.1" && sudo cp "$temp_man" "$man_dir/cmdai.1"; then
            say_success "Man page installed (use 'man caro' or 'man cmdai')"
        else
            say_warn "Failed to install man page with sudo"
        fi
    else
        if cp "$temp_man" "$man_dir/caro.1" && cp "$temp_man" "$man_dir/cmdai.1"; then
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

    # Check if cmdai is available
    if ! check_cmd cmdai; then
        say_warn "cmdai not found in PATH, skipping completion setup"
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
            say "You can generate completions manually with: cmdai --completions <shell>"
            ;;
    esac
}

setup_bash_completions() {
    local shell_config="$1"
    local completion_dir="$HOME/.local/share/bash-completion/completions"

    mkdir -p "$completion_dir"

    if cmdai --completions bash > "$completion_dir/cmdai" 2>/dev/null; then
        # Also create caro completion (same as cmdai but with different name in the file)
        cp "$completion_dir/cmdai" "$completion_dir/caro"
        say_success "Bash completions installed to $completion_dir"

        # Source completions in bashrc if not already done
        if [ -n "$shell_config" ] && [ -f "$shell_config" ]; then
            if ! grep -q "bash-completion/completions/cmdai" "$shell_config" 2>/dev/null; then
                echo "" >> "$shell_config"
                echo "# Caro shell completions" >> "$shell_config"
                echo '[ -f "$HOME/.local/share/bash-completion/completions/cmdai" ] && source "$HOME/.local/share/bash-completion/completions/cmdai"' >> "$shell_config"
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

    if cmdai --completions zsh > "$completion_dir/_cmdai" 2>/dev/null; then
        cp "$completion_dir/_cmdai" "$completion_dir/_caro"
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

    if cmdai --completions fish > "$completion_dir/cmdai.fish" 2>/dev/null; then
        cp "$completion_dir/cmdai.fish" "$completion_dir/caro.fish"
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
    echo -e "  Alias:       $([ "$INSTALL_ALIAS" = true ] && echo "${GREEN}yes${NC}" || echo "${YELLOW}no${NC}")"
    echo -e "  Man page:    $([ "$INSTALL_MANPAGE" = true ] && echo "${GREEN}yes${NC}" || echo "${YELLOW}no${NC}")"
    echo -e "  Completions: $([ "$INSTALL_COMPLETIONS" = true ] && echo "${GREEN}yes${NC}" || echo "${YELLOW}no${NC}")"
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
  cmdai "list all files in this directory"
  caro "list all files in this directory"

Execute directly:
  caro -x "show disk usage sorted by size"

Get help:
  caro --help
  man caro

Generate completions for other shells:
  cmdai --completions bash
  cmdai --completions zsh
  cmdai --completions fish
  cmdai --completions powershell

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
