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

    # Run cargo install and capture output while showing progress
    say "Building from source (this may take several minutes)..."
    echo ""

    local cargo_output
    local cargo_exit_code
    local temp_output
    temp_output=$(mktemp)

    # Run cargo and tee output to both terminal and temp file for error detection
    # Use pipefail to get cargo's exit code, not tee's
    set +e
    (set -o pipefail; cargo install caro $cargo_features 2>&1 | tee "$temp_output")
    cargo_exit_code=$?
    set -e
    cargo_output=$(cat "$temp_output")
    rm -f "$temp_output"

    if [ $cargo_exit_code -eq 0 ]; then
        say_success "Installed caro successfully"
        return 0
    fi

    # Check for edition2024 or Rust version compatibility issues
    if echo "$cargo_output" | grep -q "edition2024\|feature.*is required\|newer version of Cargo"; then
        say_error "Cargo install failed due to Rust version incompatibility"
        say_warn "A dependency requires a newer version of Rust"
        say "You can either:"
        say "  1. Update Rust: rustup update"
        say "  2. Use the pre-built binary (attempting now...)"
        echo ""
        return 1
    fi

    # Check for missing C++ standard library headers (common on macOS)
    if echo "$cargo_output" | grep -q "fatal error:.*file not found\|'algorithm' file not found\|'cstdint' file not found\|'vector' file not found"; then
        say_error "Cargo install failed due to missing C++ headers"
        say_warn "This usually means Xcode Command Line Tools need to be installed or updated"
        say "To fix this, run:"
        say "  xcode-select --install"
        say ""
        say "If already installed, try resetting:"
        say "  sudo xcode-select --reset"
        say ""
        say "Falling back to pre-built binary..."
        echo ""
        return 1
    fi

    # For other errors, show the output and return failure
    say_error "Failed to install via cargo"
    echo "$cargo_output" | tail -20
    return 1
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
        setup_path "$install_dir"
    fi

    return 0
}

# Setup PATH in shell config
setup_path() {
    local install_dir="$1"
    local shell_config=""
    local shell_name=""

    # Detect shell config file
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
            say_warn "Could not detect shell config file"
            say "Please manually add to your shell config:"
            say "  export PATH=\"$install_dir:\$PATH\""
            return
            ;;
    esac

    # Check if already added
    if grep -q "# caro PATH" "$shell_config" 2>/dev/null; then
        say "PATH already configured in $shell_config"
        return
    fi

    # Create config file if it doesn't exist
    if [ ! -f "$shell_config" ]; then
        mkdir -p "$(dirname "$shell_config")"
        touch "$shell_config"
    fi

    say "Adding $install_dir to PATH in $shell_config..."

    if [[ "$shell_name" == "fish" ]]; then
        echo -e "\n# caro PATH" >> "$shell_config"
        echo "set -gx PATH $install_dir \$PATH" >> "$shell_config"
    else
        echo -e "\n# caro PATH" >> "$shell_config"
        echo "export PATH=\"$install_dir:\$PATH\"" >> "$shell_config"
    fi

    say_success "PATH updated in $shell_config"
    say "Run 'source $shell_config' or restart your terminal to apply"
}

# Setup shell alias
setup_alias() {
    local shell_config=""
    local shell_name=""

    # Detect shell - prioritize $SHELL env var over subprocess shell version vars
    # This is important when script is run via 'bash <(curl ...)' where BASH_VERSION
    # would be set even if user's actual shell is zsh/fish
    case "$SHELL" in
        */bash)
            shell_name="bash"
            if [ -f "$HOME/.bashrc" ]; then
                shell_config="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_config="$HOME/.bash_profile"
            else
                shell_config="$HOME/.bashrc"  # default to .bashrc
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
            # Fallback to checking version variables if $SHELL is not set or unknown
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

    if [ -z "$shell_config" ]; then
        say_warn "Could not detect shell config file."
        return
    fi

    if [ ! -f "$shell_config" ]; then
        say_warn "Shell config file not found. Creating $shell_config"
        # Create parent directory if needed (e.g., for fish config)
        mkdir -p "$(dirname "$shell_config")"
        touch "$shell_config"
    fi

    # Check if old cmdai alias exists and inform user
    if grep -q "alias caro='cmdai'" "$shell_config" 2>/dev/null; then
        say_warn "Found old 'cmdai' alias in $shell_config"
        say "You can remove it - the binary is now named 'caro' directly"
        echo ""
    fi
}

# Main installation
main() {
    say "Starting Caro installation..."
    echo ""

    # Check prerequisites
    if ! check_cmd curl && ! check_cmd wget; then
        err "Neither curl nor wget found. Please install one of them."
    fi

    local install_success=false

    # Check for cargo
    if check_cmd cargo; then
        # Cargo is available - try it first (especially important for Apple Silicon MLX support)
        if install_via_cargo; then
            install_success=true
        else
            # Cargo install failed - try binary as fallback
            say "Attempting to install pre-built binary as fallback..."
            echo ""
            if install_via_binary; then
                install_success=true
                if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
                    echo ""
                    say_warn "Note: Pre-built binary installed without MLX optimization"
                    say "For MLX support, update Rust and rebuild:"
                    say "  rustup update"
                    say "  cargo install caro --features embedded-mlx --force"
                fi
            else
                err "Both cargo and binary installation failed. Please update Rust (rustup update) and try again."
            fi
        fi
    else
        # No cargo - try binary installation first
        say_warn "Cargo not found."
        echo ""

        # Attempt binary installation
        if install_via_binary; then
            install_success=true
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
            if install_via_cargo; then
                install_success=true
            else
                err "Installation failed. Please update Rust (rustup update) and try again."
            fi
        fi
    fi

    if [ "$install_success" != "true" ]; then
        err "Installation failed"
    fi
    echo ""

    # Check for legacy alias
    setup_alias
    echo ""

    # Determine shell config for final message
    local shell_config_msg=""
    case "$SHELL" in
        */bash)
            if [ -f "$HOME/.bash_profile" ]; then
                shell_config_msg="source ~/.bash_profile"
            else
                shell_config_msg="source ~/.bashrc"
            fi
            ;;
        */zsh)
            shell_config_msg="source ~/.zshrc"
            ;;
        */fish)
            shell_config_msg="source ~/.config/fish/config.fish"
            ;;
        *)
            shell_config_msg="source your shell config file"
            ;;
    esac

    # Success message
    cat << EOF
═══════════════════════════════════════════════════════════
  Installation Complete! 🎉
═══════════════════════════════════════════════════════════

Usage:
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

To start using caro:
  • Restart your shell, or
  • Run: $shell_config_msg

EOF
}

main "$@"
