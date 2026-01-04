#!/usr/bin/env bash
#
# caro installer (formerly cmdai)
#
# Usage:
#   bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
#   bash <(wget -qO- https://setup.caro.sh)
#   curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
#   wget -qO- https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
REPO="wildcard/caro"
BINARY_NAME="caro"
INSTALL_DIR="${CARO_INSTALL_DIR:-$HOME/.local/bin}"

# Installation preferences (set by interactive prompts)
INTERACTIVE_MODE="${CARO_INTERACTIVE:-true}"
INSTALL_METHOD=""  # "cargo" or "binary"
SETUP_SHELL_COMPLETION="true"
SETUP_PATH_AUTO="true"
CONFIGURE_SAFETY_LEVEL="true"

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

    # Use cargo if explicitly chosen or if no method specified and cargo exists
    if [ "$INSTALL_METHOD" = "cargo" ] || { [ -z "$INSTALL_METHOD" ] && command_exists cargo; }; then
        echo -e "${BLUE}Installing via cargo...${NC}"

        # Detect if on macOS with Apple Silicon for MLX optimization
        local cargo_features=""
        if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
            echo -e "${GREEN}Building with MLX optimization for Apple Silicon...${NC}"
            cargo_features="--features embedded-mlx"
        fi

        # Run cargo install and capture output while showing progress
        echo -e "${BLUE}Building from source (this may take several minutes)...${NC}"
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
            echo -e "${GREEN}✓ Installed caro successfully via cargo${NC}"
            return 0
        fi

        # Check for edition2024 or Rust version compatibility issues
        if echo "$cargo_output" | grep -q "edition2024\|feature.*is required\|newer version of Cargo"; then
            echo -e "${RED}✗ Cargo install failed due to Rust version incompatibility${NC}"
            echo -e "${YELLOW}A dependency requires a newer version of Rust${NC}"
            echo -e "${BLUE}You can either:${NC}"
            echo -e "  ${GREEN}1. Update Rust: rustup update${NC}"
            echo -e "  ${GREEN}2. Use the pre-built binary (attempting now...)${NC}"
            echo ""
            # Fall through to binary download
        # Check for missing C++ standard library headers (common on macOS)
        elif echo "$cargo_output" | grep -q "fatal error:.*file not found\|'algorithm' file not found\|'cstdint' file not found\|'vector' file not found"; then
            echo -e "${RED}✗ Cargo install failed due to missing C++ headers${NC}"
            echo -e "${YELLOW}This usually means Xcode Command Line Tools need to be installed or updated${NC}"
            echo -e "${BLUE}To fix this, run:${NC}"
            echo -e "  ${GREEN}xcode-select --install${NC}"
            echo ""
            echo -e "${BLUE}If already installed, try resetting:${NC}"
            echo -e "  ${GREEN}sudo xcode-select --reset${NC}"
            echo ""
            echo -e "${YELLOW}Falling back to pre-built binary...${NC}"
            echo ""
            # Fall through to binary download
        else
            # For other errors, show the output
            echo -e "${RED}✗ Failed to install via cargo${NC}"
            echo "$cargo_output" | tail -20
            echo ""
            echo -e "${YELLOW}Attempting to download pre-built binary as fallback...${NC}"
            echo ""
        fi
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

    # Map platform to base asset name
    local base_asset_name
    case "$platform" in
        linux-amd64)    base_asset_name="linux-amd64" ;;
        linux-arm64)    base_asset_name="linux-arm64" ;;
        macos-amd64)    base_asset_name="macos-intel" ;;
        macos-arm64)    base_asset_name="macos-silicon" ;;
        windows-amd64)  base_asset_name="windows-amd64.exe" ;;
        *)
            echo -e "${RED}Unsupported platform: $platform${NC}"
            exit 1
            ;;
    esac

    # Try versioned asset name first (v1.0.3+), fall back to non-versioned (v1.0.2)
    local versioned_asset_name="caro-${version}-${base_asset_name}"
    local legacy_asset_name="caro-${base_asset_name}"
    local asset_name="$versioned_asset_name"
    local binary_url="https://github.com/${REPO}/releases/download/v${version}/${versioned_asset_name}"
    local checksum_url="${binary_url}.sha256"

    echo -e "${BLUE}Downloading caro v${version} for ${platform}...${NC}"

    # Try versioned name first, fall back to legacy name
    local download_success=false
    if command_exists curl; then
        if curl -fsSL "$binary_url" -o "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null; then
            download_success=true
        else
            # Try legacy non-versioned name
            asset_name="$legacy_asset_name"
            binary_url="https://github.com/${REPO}/releases/download/v${version}/${legacy_asset_name}"
            checksum_url="${binary_url}.sha256"
            echo -e "${YELLOW}Versioned binary not found, trying legacy name...${NC}"
            curl -fsSL "$binary_url" -o "${INSTALL_DIR}/${BINARY_NAME}" && download_success=true
        fi
    elif command_exists wget; then
        if wget -qO "${INSTALL_DIR}/${BINARY_NAME}" "$binary_url" 2>/dev/null; then
            download_success=true
        else
            # Try legacy non-versioned name
            asset_name="$legacy_asset_name"
            binary_url="https://github.com/${REPO}/releases/download/v${version}/${legacy_asset_name}"
            checksum_url="${binary_url}.sha256"
            echo -e "${YELLOW}Versioned binary not found, trying legacy name...${NC}"
            wget -qO "${INSTALL_DIR}/${BINARY_NAME}" "$binary_url" && download_success=true
        fi
    fi

    if [ "$download_success" = false ]; then
        echo -e "${RED}Error: Failed to download binary${NC}"
        exit 1
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
        echo -e "${YELLOW}For MLX optimization, update Rust and rebuild from source:${NC}"
        echo -e "  ${GREEN}rustup update${NC}"
        echo -e "  ${GREEN}cargo install caro --features embedded-mlx --force${NC}"
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

# Prompt user for yes/no question
ask_yes_no() {
    local question="$1"
    local default="${2:-y}"
    local response

    while true; do
        if [ "$default" = "y" ]; then
            echo -ne "${CYAN}${question} [Y/n]: ${NC}"
        else
            echo -ne "${CYAN}${question} [y/N]: ${NC}"
        fi

        read -r response
        response="${response:-$default}"

        case "$response" in
            [Yy]*) return 0 ;;
            [Nn]*) return 1 ;;
            *) echo -e "${YELLOW}Please answer yes or no.${NC}" ;;
        esac
    done
}

# Prompt user for choice from options
ask_choice() {
    local question="$1"
    shift
    local options=("$@")
    local choice

    echo -e "${CYAN}${question}${NC}"
    for i in "${!options[@]}"; do
        echo -e "  ${BLUE}[$((i+1))]${NC} ${options[$i]}"
    done

    while true; do
        echo -ne "${CYAN}Enter choice [1-${#options[@]}]: ${NC}"
        read -r choice

        if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le "${#options[@]}" ]; then
            return $((choice-1))
        else
            echo -e "${YELLOW}Please enter a number between 1 and ${#options[@]}.${NC}"
        fi
    done
}

# Interactive configuration prompts
run_interactive_setup() {
    if [ "$INTERACTIVE_MODE" != "true" ]; then
        return 0
    fi

    echo -e "${BOLD}${BLUE}╔═══════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${BLUE}║   Caro Installation Setup             ║${NC}"
    echo -e "${BOLD}${BLUE}╚═══════════════════════════════════════╝${NC}"
    echo ""

    # Detect platform and show info
    local platform
    platform=$(detect_platform)
    local os="${platform%-*}"
    local arch="${platform#*-}"

    echo -e "${GREEN}Detected environment:${NC}"
    echo -e "  OS:           ${BOLD}$os${NC}"
    echo -e "  Architecture: ${BOLD}$arch${NC}"
    echo -e "  Shell:        ${BOLD}$(basename "$SHELL")${NC}"
    echo ""

    # Ask about installation method
    if command_exists cargo; then
        echo -e "${GREEN}✓ Rust/Cargo detected${NC}"

        if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
            echo -e "${BLUE}Note: Building from source enables MLX optimization for Apple Silicon${NC}"
        fi
        echo ""

        if ask_yes_no "Build from source with cargo?" "y"; then
            INSTALL_METHOD="cargo"
        else
            INSTALL_METHOD="binary"
        fi
    else
        echo -e "${YELLOW}Cargo not found - will download pre-built binary${NC}"
        INSTALL_METHOD="binary"

        if ask_yes_no "Would you like to install Rust/Cargo for future builds?" "n"; then
            echo -e "${BLUE}Visit: ${BOLD}https://rustup.rs${NC}"
            echo -e "${YELLOW}Re-run this installer after installing Rust${NC}"
            exit 0
        fi
    fi
    echo ""

    # Ask about shell completion
    if ask_yes_no "Set up shell completion (tab completion for caro commands)?" "y"; then
        SETUP_SHELL_COMPLETION="true"
    else
        SETUP_SHELL_COMPLETION="false"
    fi
    echo ""

    # Ask about PATH setup
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}$INSTALL_DIR is not currently in your PATH${NC}"
        if ask_yes_no "Automatically add to PATH?" "y"; then
            SETUP_PATH_AUTO="true"
        else
            SETUP_PATH_AUTO="false"
        fi
    else
        echo -e "${GREEN}✓ $INSTALL_DIR is already in PATH${NC}"
        SETUP_PATH_AUTO="false"
    fi
    echo ""

    # Ask about safety level configuration
    if ask_yes_no "Configure default safety level?" "y"; then
        ask_choice "Choose default safety level:" \
            "Strict (recommended - blocks potentially dangerous commands)" \
            "Moderate (warns about risky commands)" \
            "Permissive (minimal safety checks)"

        case $? in
            0) SAFETY_LEVEL="strict" ;;
            1) SAFETY_LEVEL="moderate" ;;
            2) SAFETY_LEVEL="permissive" ;;
        esac
        CONFIGURE_SAFETY_LEVEL="true"
    else
        SAFETY_LEVEL="strict"
        CONFIGURE_SAFETY_LEVEL="false"
    fi
    echo ""

    echo -e "${GREEN}Configuration complete!${NC}"
    echo ""
}

# Setup shell completion
setup_shell_completion() {
    if [ "$SETUP_SHELL_COMPLETION" != "true" ]; then
        return 0
    fi

    local shell_config=""
    local completion_cmd=""

    # Detect shell and set appropriate completion command
    if [ -n "$BASH_VERSION" ] || [[ "$SHELL" == */bash ]]; then
        shell_config="$HOME/.bashrc"
        [ -f "$HOME/.bash_profile" ] && shell_config="$HOME/.bash_profile"
        completion_cmd='eval "$(caro --completion bash)"'
    elif [ -n "$ZSH_VERSION" ] || [[ "$SHELL" == */zsh ]]; then
        shell_config="$HOME/.zshrc"
        completion_cmd='eval "$(caro --completion zsh)"'
    elif [ -n "$FISH_VERSION" ] || [[ "$SHELL" == */fish ]]; then
        shell_config="$HOME/.config/fish/config.fish"
        completion_cmd='caro --completion fish | source'
    fi

    if [ -n "$shell_config" ] && [ -n "$completion_cmd" ]; then
        if ! grep -q "caro --completion" "$shell_config" 2>/dev/null; then
            echo -e "${BLUE}Setting up shell completion...${NC}"
            echo -e "\n# caro shell completion" >> "$shell_config"
            echo "$completion_cmd" >> "$shell_config"
            echo -e "${GREEN}✓ Shell completion configured${NC}"
        fi
    fi
}

# Configure safety level
configure_safety() {
    if [ "$CONFIGURE_SAFETY_LEVEL" != "true" ]; then
        return 0
    fi

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
        echo -e "${BLUE}Configuring safety level: $SAFETY_LEVEL${NC}"

        # Remove existing CARO_SAFETY_LEVEL if present
        if grep -q "CARO_SAFETY_LEVEL" "$shell_config" 2>/dev/null; then
            # Create temp file without CARO_SAFETY_LEVEL lines
            grep -v "CARO_SAFETY_LEVEL" "$shell_config" > "${shell_config}.tmp"
            mv "${shell_config}.tmp" "$shell_config"
        fi

        if [[ "$shell_config" == *"fish"* ]]; then
            echo -e "\n# caro safety level" >> "$shell_config"
            echo "set -gx CARO_SAFETY_LEVEL $SAFETY_LEVEL" >> "$shell_config"
        else
            echo -e "\n# caro safety level" >> "$shell_config"
            echo "export CARO_SAFETY_LEVEL=\"$SAFETY_LEVEL\"" >> "$shell_config"
        fi

        echo -e "${GREEN}✓ Safety level set to: $SAFETY_LEVEL${NC}"
    fi
}

# Add install directory to PATH if needed
setup_path() {
    if [ "$SETUP_PATH_AUTO" != "true" ]; then
        return 0
    fi

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
    echo -e "${BOLD}${BLUE}╔═══════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${BLUE}║      Caro Installer                   ║${NC}"
    echo -e "${BOLD}${BLUE}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${CYAN}Welcome to the Caro installer!${NC}"
    echo -e "${CYAN}This will install caro - your AI-powered shell command assistant.${NC}"
    echo ""

    # Run interactive setup (asks configuration questions)
    run_interactive_setup

    # Create install directory if it doesn't exist
    if [ ! -d "$INSTALL_DIR" ]; then
        echo -e "${BLUE}Creating install directory: $INSTALL_DIR${NC}"
        mkdir -p "$INSTALL_DIR"
    fi

    # Install the binary
    install_binary

    # Setup PATH if needed
    setup_path

    # Setup shell completion
    setup_shell_completion

    # Configure safety level
    configure_safety

    # Check for legacy alias
    check_legacy_alias

    echo ""
    echo -e "${BOLD}${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${GREEN}║   Installation Complete! 🎉           ║${NC}"
    echo -e "${BOLD}${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""

    # Show next steps
    echo -e "${BOLD}${BLUE}Next steps:${NC}"
    echo ""

    if [ "$SETUP_PATH_AUTO" = "true" ] || [ "$SETUP_SHELL_COMPLETION" = "true" ] || [ "$CONFIGURE_SAFETY_LEVEL" = "true" ]; then
        local shell_name
        shell_name=$(basename "$SHELL")
        echo -e "${YELLOW}→ Reload your shell to apply changes:${NC}"
        if [[ "$shell_name" == "fish" ]]; then
            echo -e "  ${GREEN}source ~/.config/fish/config.fish${NC}"
        else
            local shell_config="$HOME/.${shell_name}rc"
            [ "$shell_name" = "bash" ] && [ -f "$HOME/.bash_profile" ] && shell_config="$HOME/.bash_profile"
            echo -e "  ${GREEN}source $shell_config${NC}"
        fi
        echo -e "  ${CYAN}Or open a new terminal window${NC}"
        echo ""
    fi

    echo -e "${YELLOW}→ Try it out:${NC}"
    echo -e "  ${GREEN}caro \"list all files in this directory\"${NC}"
    echo ""

    echo -e "${YELLOW}→ Get help:${NC}"
    echo -e "  ${GREEN}caro --help${NC}"
    echo ""

    echo -e "${BLUE}Documentation:${NC}"
    echo -e "  ${GREEN}https://caro.sh${NC}"
    echo -e "  ${GREEN}https://github.com/${REPO}${NC}"
    echo ""

    if [ "$CONFIGURE_SAFETY_LEVEL" = "true" ]; then
        echo -e "${CYAN}Safety level configured: ${BOLD}$SAFETY_LEVEL${NC}"
        echo -e "${CYAN}Change anytime with: ${GREEN}export CARO_SAFETY_LEVEL=<strict|moderate|permissive>${NC}"
        echo ""
    fi
}

main "$@"
