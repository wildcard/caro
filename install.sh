#!/usr/bin/env bash
#
# caro installer (formerly cmdai)
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
#   wget -qO- https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="wildcard/caro"
BINARY_NAME="caro"
INSTALL_DIR="${CARO_INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)
            echo -e "${RED}Unsupported operating system: $(uname -s)${NC}"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="amd64" ;;
        aarch64|arm64)  arch="arm64" ;;
        *)
            echo -e "${RED}Unsupported architecture: $(uname -m)${NC}"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download and install binary
install_binary() {
    echo -e "${BLUE}Installing caro...${NC}"

    # Check if cargo is available for direct installation
    if command_exists cargo; then
        echo -e "${BLUE}Installing via cargo...${NC}"

        # Detect if on macOS with Apple Silicon for MLX optimization
        local cargo_features=""
        if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
            echo -e "${GREEN}Detected Apple Silicon - building with MLX optimization${NC}"
            cargo_features="--features embedded-mlx"
        fi

        cargo install caro $cargo_features
        return 0
    fi

    # Fallback: Download pre-built binary from GitHub releases
    local platform
    platform=$(detect_platform)

    echo -e "${YELLOW}Cargo not found. Downloading pre-built binary...${NC}"

    # Try to get latest release tag from GitHub
    local latest_url="https://api.github.com/repos/${REPO}/releases/latest"
    local release_info

    if command_exists curl; then
        release_info=$(curl -s "$latest_url")
    elif command_exists wget; then
        release_info=$(wget -qO- "$latest_url")
    else
        echo -e "${RED}Error: Neither curl nor wget found. Please install one of them.${NC}"
        exit 1
    fi

    # Extract tag name (version)
    local version
    version=$(echo "$release_info" | grep '"tag_name":' | sed -E 's/.*"tag_name": "v?([^"]+)".*/\1/')

    if [ -z "$version" ]; then
        echo -e "${RED}Error: Could not determine latest version.${NC}"
        echo -e "${YELLOW}Please install Rust and cargo: https://rustup.rs/${NC}"
        echo -e "${YELLOW}Then run: cargo install caro${NC}"
        exit 1
    fi

    # Map platform to asset name
    local asset_name
    case "$platform" in
        linux-amd64)    asset_name="caro-linux-amd64" ;;
        linux-arm64)    asset_name="caro-linux-arm64" ;;
        macos-amd64)    asset_name="caro-macos-intel" ;;
        macos-arm64)    asset_name="caro-macos-silicon" ;;
        windows-amd64)  asset_name="caro-windows-amd64.exe" ;;
        *)
            echo -e "${RED}Unsupported platform: $platform${NC}"
            exit 1
            ;;
    esac

    # Construct download URLs
    local binary_url="https://github.com/${REPO}/releases/download/v${version}/${asset_name}"
    local checksum_url="${binary_url}.sha256"

    echo -e "${BLUE}Downloading caro v${version} for ${platform}...${NC}"

    # Download binary
    if command_exists curl; then
        curl -fsSL "$binary_url" -o "${INSTALL_DIR}/${BINARY_NAME}" || {
            echo -e "${RED}Error: Failed to download binary${NC}"
            exit 1
        }
    elif command_exists wget; then
        wget -qO "${INSTALL_DIR}/${BINARY_NAME}" "$binary_url" || {
            echo -e "${RED}Error: Failed to download binary${NC}"
            exit 1
        }
    fi

    # Make binary executable
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    # Download and verify checksum
    local checksum_file
    checksum_file=$(mktemp)

    if command_exists curl; then
        curl -fsSL "$checksum_url" -o "$checksum_file" 2>/dev/null || {
            echo -e "${YELLOW}Warning: Could not download checksum file${NC}"
            rm -f "$checksum_file"
            return 0
        }
    elif command_exists wget; then
        wget -qO "$checksum_file" "$checksum_url" 2>/dev/null || {
            echo -e "${YELLOW}Warning: Could not download checksum file${NC}"
            rm -f "$checksum_file"
            return 0
        }
    fi

    # Verify checksum if available
    if [ -f "$checksum_file" ]; then
        local expected_hash
        expected_hash=$(awk '{print $1}' "$checksum_file")

        if command_exists shasum; then
            local actual_hash
            actual_hash=$(shasum -a 256 "${INSTALL_DIR}/${BINARY_NAME}" | awk '{print $1}')

            if [ "$expected_hash" = "$actual_hash" ]; then
                echo -e "${GREEN}✓ Checksum verified${NC}"
            else
                echo -e "${YELLOW}Warning: Checksum mismatch (expected: $expected_hash, got: $actual_hash)${NC}"
            fi
        elif command_exists sha256sum; then
            local actual_hash
            actual_hash=$(sha256sum "${INSTALL_DIR}/${BINARY_NAME}" | awk '{print $1}')

            if [ "$expected_hash" = "$actual_hash" ]; then
                echo -e "${GREEN}✓ Checksum verified${NC}"
            else
                echo -e "${YELLOW}Warning: Checksum mismatch (expected: $expected_hash, got: $actual_hash)${NC}"
            fi
        else
            echo -e "${YELLOW}Warning: No checksum tool available (shasum or sha256sum)${NC}"
        fi

        rm -f "$checksum_file"
    fi

    echo -e "${GREEN}✓ Binary installed to ${INSTALL_DIR}/${BINARY_NAME}${NC}"

    # Note about MLX support for Apple Silicon
    if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
        echo ""
        echo -e "${BLUE}Note: You're on Apple Silicon!${NC}"
        echo -e "${YELLOW}For MLX optimization, you can rebuild from source:${NC}"
        echo -e "  ${GREEN}cargo install caro --features embedded-mlx${NC}"
    fi

    return 0
}

# Note: No alias setup needed anymore since the binary is now named 'caro'
# This function is kept for backward compatibility and information
check_legacy_alias() {
    local shell_config=""

    # Detect shell config
    if [ -n "$BASH_VERSION" ] || [[ "$SHELL" == */bash ]]; then
        shell_config="$HOME/.bashrc"
        [ -f "$HOME/.bash_profile" ] && shell_config="$HOME/.bash_profile"
    elif [ -n "$ZSH_VERSION" ] || [[ "$SHELL" == */zsh ]]; then
        shell_config="$HOME/.zshrc"
    elif [ -n "$FISH_VERSION" ] || [[ "$SHELL" == */fish ]]; then
        shell_config="$HOME/.config/fish/config.fish"
    fi

    if [ -n "$shell_config" ] && [ -f "$shell_config" ]; then
        # Check if old cmdai alias exists
        if grep -q "alias caro='cmdai'" "$shell_config" 2>/dev/null; then
            echo -e "${YELLOW}Found old 'cmdai' alias in $shell_config${NC}"
            echo -e "${BLUE}You can remove it - the binary is now named 'caro' directly${NC}"
        fi
    fi
}

# Add install directory to PATH if needed
setup_path() {
    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}$INSTALL_DIR is not in your PATH${NC}"
        
        local shell_config=""
        
        # Detect shell config
        if [ -n "$BASH_VERSION" ] || [[ "$SHELL" == */bash ]]; then
            shell_config="$HOME/.bashrc"
            [ -f "$HOME/.bash_profile" ] && shell_config="$HOME/.bash_profile"
        elif [ -n "$ZSH_VERSION" ] || [[ "$SHELL" == */zsh ]]; then
            shell_config="$HOME/.zshrc"
        elif [ -n "$FISH_VERSION" ] || [[ "$SHELL" == */fish ]]; then
            shell_config="$HOME/.config/fish/config.fish"
        fi

        if [ -n "$shell_config" ]; then
            echo -e "${BLUE}Adding $INSTALL_DIR to PATH in $shell_config...${NC}"
            
            if [[ "$shell_config" == *"fish"* ]]; then
                echo -e "\n# caro PATH" >> "$shell_config"
                echo "set -gx PATH $INSTALL_DIR \$PATH" >> "$shell_config"
            else
                echo -e "\n# caro PATH" >> "$shell_config"
                echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_config"
            fi
            
            echo -e "${GREEN}✓ PATH updated${NC}"
        else
            echo -e "${YELLOW}Please manually add $INSTALL_DIR to your PATH${NC}"
        fi
    fi
}

# Main installation flow
main() {
    echo -e "${BLUE}╔═══════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║      Caro Installer                   ║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════╝${NC}"
    echo ""

    # Create install directory if it doesn't exist
    if [ ! -d "$INSTALL_DIR" ]; then
        echo -e "${BLUE}Creating install directory: $INSTALL_DIR${NC}"
        mkdir -p "$INSTALL_DIR"
    fi

    # Install the binary
    install_binary

    # Setup PATH if needed
    setup_path

    # Check for legacy alias
    check_legacy_alias

    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   Installation Complete! 🎉           ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Usage:${NC}"
    echo -e "  ${GREEN}caro \"list all files in this directory\"${NC}"
    echo ""
    echo -e "${BLUE}For more information:${NC}"
    echo -e "  ${GREEN}caro --help${NC}"
    echo -e "  ${GREEN}https://github.com/${REPO}${NC}"
    echo -e "  ${GREEN}https://caro.sh${NC}"
    echo ""
}

main "$@"
