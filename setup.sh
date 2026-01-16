#!/usr/bin/env bash
#
# Caro Setup Script
#
# Usage:
#   bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
#   bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh) -- --force
#   bash <(wget -qO- https://setup.caro.sh)
#
# Options:
#   --force    Force reinstall even if same version is already installed

set -e

# Parse command line arguments
FORCE_INSTALL="false"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --force|-f)
            FORCE_INSTALL="true"
            shift
            ;;
        *)
            shift
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

    # Build cargo install command
    local cargo_cmd="cargo install caro $cargo_features"
    if [ "$FORCE_INSTALL" = "true" ]; then
        cargo_cmd="$cargo_cmd --force"
        say "Force install requested"
    fi

    if $cargo_cmd; then
        say_success "Installed caro successfully"
        return 0
    else
        err "Failed to install via cargo"
    fi
}

# Add directory to PATH in shell config
add_to_path() {
    local dir_to_add="$1"
    local shell_config=""
    local shell_name=""
    local path_export=""

    # Detect shell - prioritize $SHELL env var
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
            path_export="export PATH=\"$dir_to_add:\$PATH\""
            ;;
        */zsh)
            shell_name="zsh"
            shell_config="${ZDOTDIR:-$HOME}/.zshrc"
            path_export="export PATH=\"$dir_to_add:\$PATH\""
            ;;
        */fish)
            shell_name="fish"
            shell_config="$HOME/.config/fish/config.fish"
            path_export="fish_add_path $dir_to_add"
            ;;
        *)
            say_warn "Could not detect shell type from \$SHELL ($SHELL)"
            say "Please add $dir_to_add to your PATH manually"
            return
            ;;
    esac

    # Create shell config if it doesn't exist
    if [ ! -f "$shell_config" ]; then
        say "Creating $shell_config"
        mkdir -p "$(dirname "$shell_config")"
        touch "$shell_config"
    fi

    # Check if PATH is already configured for this directory
    if grep -q "$dir_to_add" "$shell_config" 2>/dev/null; then
        say "$dir_to_add already in $shell_config"
        return
    fi

    # Add PATH export to shell config
    say "Adding $dir_to_add to PATH in $shell_config"
    {
        echo ""
        echo "# Added by caro installer"
        echo "$path_export"
    } >> "$shell_config"
    say_success "PATH updated in $shell_config"
    say "Restart your terminal or run: source $shell_config"
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
        add_to_path "$install_dir"
    fi

    return 0
}

# Check for legacy cmdai alias in shell config
check_legacy_alias() {
    local shell_config=""

    # Detect shell config file
    case "$SHELL" in
        */bash)
            if [ -f "$HOME/.bashrc" ]; then
                shell_config="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_config="$HOME/.bash_profile"
            fi
            ;;
        */zsh)
            shell_config="${ZDOTDIR:-$HOME}/.zshrc"
            ;;
        */fish)
            shell_config="$HOME/.config/fish/config.fish"
            ;;
    esac

    if [ -n "$shell_config" ] && [ -f "$shell_config" ]; then
        # Check if old cmdai alias exists and inform user
        if grep -q "alias caro='cmdai'" "$shell_config" 2>/dev/null; then
            say_warn "Found old 'cmdai' alias in $shell_config"
            say "You can remove it - the binary is now named 'caro' directly"
            echo ""
        fi
    fi
}

# Check for conflicting caro installations in PATH
check_conflicting_installations() {
    # Find all caro binaries in PATH
    local caro_locations
    caro_locations=$(which -a caro 2>/dev/null || true)

    if [ -z "$caro_locations" ]; then
        return 0
    fi

    # Count installations
    local count
    count=$(echo "$caro_locations" | wc -l | tr -d ' ')

    if [ "$count" -gt 1 ]; then
        echo ""
        say_warn "═══════════════════════════════════════════════════════"
        say_warn "     Multiple caro installations detected!              "
        say_warn "═══════════════════════════════════════════════════════"
        echo ""
        say "Found caro in these locations:"
        echo "$caro_locations" | while read -r loc; do
            if [ -x "$loc" ]; then
                local ver
                ver=$("$loc" --version 2>/dev/null | head -1 || echo "unknown version")
                echo "  $loc → $ver"
            fi
        done
        echo ""

        # Show which one will be used
        local active_caro
        active_caro=$(which caro 2>/dev/null)
        say_success "Active (first in PATH): $active_caro"
        echo ""

        # Suggest cleanup
        say_warn "To use the newly installed version, remove old installations:"
        echo "$caro_locations" | while read -r loc; do
            if [ "$loc" != "$HOME/.cargo/bin/caro" ] && [ "$loc" != "${CARO_INSTALL_DIR:-$HOME/.local/bin}/caro" ]; then
                echo "  sudo rm $loc"
            fi
        done
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

    # Check for conflicting installations
    check_conflicting_installations
    echo ""

    # Determine shell-specific reload command
    local shell_config_hint=""
    case "$SHELL" in
        */zsh)  shell_config_hint="source ~/.zshrc" ;;
        */bash)
            if [ -f "$HOME/.bash_profile" ]; then
                shell_config_hint="source ~/.bash_profile"
            else
                shell_config_hint="source ~/.bashrc"
            fi
            ;;
        */fish) shell_config_hint="source ~/.config/fish/config.fish" ;;
        *)      shell_config_hint="source your shell config file" ;;
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
  • Open a new terminal window, or
  • Run: $shell_config_hint

EOF
}

main "$@"
