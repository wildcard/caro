#!/usr/bin/env bash
#
# cmdai (caro) installer
# 
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
#   wget -qO- https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="wildcard/cmdai"
BINARY_NAME="cmdai"
ALIAS_NAME="caro"
INSTALL_DIR="${CMDAI_INSTALL_DIR:-$HOME/.local/bin}"

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
    echo -e "${BLUE}Installing cmdai...${NC}"

    # Check if cargo is available for direct installation
    if command_exists cargo; then
        echo -e "${BLUE}Installing via cargo...${NC}"
        cargo install cmdai
        return 0
    fi

    # Fallback: Check for pre-built binaries on GitHub releases
    local platform
    platform=$(detect_platform)
    
    echo -e "${YELLOW}Cargo not found. Checking for pre-built binaries...${NC}"
    
    # Try to get latest release from GitHub
    local latest_url="https://api.github.com/repos/${REPO}/releases/latest"
    
    if command_exists curl; then
        local release_info
        release_info=$(curl -s "$latest_url")
    elif command_exists wget; then
        local release_info
        release_info=$(wget -qO- "$latest_url")
    else
        echo -e "${RED}Error: Neither curl nor wget found. Please install one of them.${NC}"
        exit 1
    fi

    # Check if we got a valid response
    if echo "$release_info" | grep -q "Not Found"; then
        echo -e "${YELLOW}No pre-built binaries available yet.${NC}"
        echo -e "${YELLOW}Please install Rust and cargo: https://rustup.rs/${NC}"
        echo -e "${YELLOW}Then run: cargo install cmdai${NC}"
        exit 1
    fi

    # For now, since there are no pre-built binaries yet, we'll guide users to cargo
    echo -e "${YELLOW}Pre-built binaries are not available yet.${NC}"
    echo -e "${BLUE}Installing Rust and cargo is recommended:${NC}"
    echo -e "  ${GREEN}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    echo -e "${BLUE}Then run:${NC}"
    echo -e "  ${GREEN}cargo install cmdai${NC}"
    exit 1
}

# Setup alias in shell configuration
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
        shell_config="$HOME/.zshrc"
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
                shell_config="$HOME/.zshrc"
                ;;
            */fish)
                shell_name="fish"
                shell_config="$HOME/.config/fish/config.fish"
                ;;
            *)
                echo -e "${YELLOW}Could not detect shell. Please manually add alias:${NC}"
                echo -e "  ${GREEN}alias caro='cmdai'${NC}"
                return
                ;;
        esac
    fi

    if [ -z "$shell_config" ] || [ ! -f "$shell_config" ]; then
        echo -e "${YELLOW}Shell config file not found. Please manually add alias to your shell config:${NC}"
        echo -e "  ${GREEN}alias caro='cmdai'${NC}"
        return
    fi

    # Check if alias already exists
    if grep -q "alias $ALIAS_NAME=" "$shell_config" 2>/dev/null; then
        echo -e "${YELLOW}Alias '$ALIAS_NAME' already exists in $shell_config${NC}"
        return
    fi

    # Add alias based on shell type
    echo -e "${BLUE}Adding alias '$ALIAS_NAME' to $shell_config...${NC}"
    
    if [ "$shell_name" = "fish" ]; then
        echo -e "\n# cmdai (caro) alias" >> "$shell_config"
        echo "alias $ALIAS_NAME='$BINARY_NAME'" >> "$shell_config"
    else
        echo -e "\n# cmdai (caro) alias" >> "$shell_config"
        echo "alias $ALIAS_NAME='$BINARY_NAME'" >> "$shell_config"
    fi

    echo -e "${GREEN}✓ Alias added successfully${NC}"
    echo -e "${BLUE}Run 'source $shell_config' or restart your shell to use the alias${NC}"
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
                echo -e "\n# cmdai PATH" >> "$shell_config"
                echo "set -gx PATH $INSTALL_DIR \$PATH" >> "$shell_config"
            else
                echo -e "\n# cmdai PATH" >> "$shell_config"
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
    echo -e "${BLUE}║   cmdai (caro) Installer              ║${NC}"
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

    # Setup alias
    setup_alias

    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   Installation Complete! 🎉           ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Usage:${NC}"
    echo -e "  ${GREEN}cmdai \"list all files in this directory\"${NC}"
    echo -e "  ${GREEN}caro \"list all files in this directory\"${NC}"
    echo ""
    echo -e "${BLUE}For more information:${NC}"
    echo -e "  ${GREEN}cmdai --help${NC}"
    echo -e "  ${GREEN}https://github.com/${REPO}${NC}"
    echo ""
}

main "$@"
