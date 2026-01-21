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

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
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
        say_success "Installed caro v${version}"
    else
        err "Binary installed but failed version check"
    fi

    return 0
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
    echo ""
    echo -e "${BLUE}Setting up Caro...${NC}"
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
    setup_alias

    # Check for conflicting installations
    check_conflicting_installations

    # Get installed version and location
    local installed_version="unknown"
    local install_location=""

    if check_cmd caro; then
        installed_version=$(caro --version 2>/dev/null | head -1 | sed 's/caro //' || echo "unknown")
        install_location=$(which caro 2>/dev/null)
    elif [ -x "$HOME/.cargo/bin/caro" ]; then
        installed_version=$("$HOME/.cargo/bin/caro" --version 2>/dev/null | head -1 | sed 's/caro //' || echo "unknown")
        install_location="$HOME/.cargo/bin/caro"
    elif [ -x "${CARO_INSTALL_DIR:-$HOME/.local/bin}/caro" ]; then
        installed_version=$("${CARO_INSTALL_DIR:-$HOME/.local/bin}/caro" --version 2>/dev/null | head -1 | sed 's/caro //' || echo "unknown")
        install_location="${CARO_INSTALL_DIR:-$HOME/.local/bin}/caro"
    fi

    # Display install location with ~ for home dir
    local display_location="$install_location"
    display_location="${display_location/#$HOME/~}"

    echo ""
    echo -e "${GREEN}✔ Caro successfully installed!${NC}"
    echo ""
    echo -e "  Version:  ${BOLD}${installed_version}${NC}"
    echo ""
    echo -e "  Location: ${BOLD}${display_location}${NC}"
    echo ""
    echo ""
    echo -e "  Next: Run ${GREEN}caro --help${NC} to get started"
    echo ""

    # Collect setup notes
    local setup_notes=()
    local shell_name
    shell_name=$(basename "$SHELL")

    # Determine install directory
    local install_dir
    install_dir=$(dirname "$install_location")

    # Check if PATH needs to be configured
    if [[ -n "$install_dir" ]] && [[ ":$PATH:" != *":$install_dir:"* ]]; then
        # Determine shell config file
        local shell_config_file=""
        case "$shell_name" in
            bash)
                if [ -f "$HOME/.bash_profile" ]; then
                    shell_config_file="~/.bash_profile"
                else
                    shell_config_file="~/.bashrc"
                fi
                ;;
            zsh)
                shell_config_file="~/.zshrc"
                ;;
            fish)
                shell_config_file="~/.config/fish/config.fish"
                ;;
            *)
                shell_config_file="~/.profile"
                ;;
        esac

        local display_dir="${install_dir/#$HOME/~}"
        if [[ "$shell_name" == "fish" ]]; then
            setup_notes+=("${display_dir} is not in your PATH. Run:\n\n  ${GREEN}set -Ux fish_user_paths ${install_dir} \$fish_user_paths${NC}")
        else
            setup_notes+=("${display_dir} is not in your PATH. Run:\n\n  ${GREEN}echo 'export PATH=\"${install_dir}:\$PATH\"' >> ${shell_config_file} && source ${shell_config_file}${NC}")
        fi
    fi

    # Display setup notes if any
    if [ ${#setup_notes[@]} -gt 0 ]; then
        echo -e "${YELLOW}⚠ Setup notes:${NC}"
        for note in "${setup_notes[@]}"; do
            echo -e "  • ${note}"
            echo ""
        done
    fi
}

main "$@"
