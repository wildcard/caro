#!/usr/bin/env bash
#
# caro installer (formerly cmdai)
#
# Usage:
#   bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
#   bash <(wget -qO- https://setup.caro.sh)
#   curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
#   wget -qO- https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash

# Don't use set -e - we handle errors explicitly for better resilience
set -u

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

# Auto-detect non-interactive mode (piped execution)
if [ ! -t 0 ]; then
    # stdin is not a terminal (piped or redirected)
    INTERACTIVE_MODE="false"
else
    # stdin is a terminal, check env var or default to true
    INTERACTIVE_MODE="${CARO_INTERACTIVE:-true}"
fi

# Installation preferences (set by interactive prompts)
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

# Check network connectivity to GitHub
check_network_connectivity() {
    echo -e "${BLUE}Checking network connectivity...${NC}"

    # Try to reach GitHub with a timeout
    if command_exists curl; then
        if curl -fsSL --connect-timeout 5 --max-time 10 https://github.com >/dev/null 2>&1; then
            echo -e "${GREEN}✓ Network connection OK${NC}"
            return 0
        fi
    elif command_exists wget; then
        if wget --timeout=10 --tries=1 -q --spider https://github.com 2>/dev/null; then
            echo -e "${GREEN}✓ Network connection OK${NC}"
            return 0
        fi
    fi

    echo -e "${RED}✗ Cannot reach GitHub${NC}"
    echo -e "${YELLOW}Please check your internet connection and proxy settings:${NC}"
    echo -e "  echo \$HTTP_PROXY"
    echo -e "  echo \$HTTPS_PROXY"
    echo -e "  ping github.com"
    return 1
}

# Try to install via cargo with error handling
try_cargo_install() {
    if ! command_exists cargo; then
        echo -e "${YELLOW}Cargo not found - skipping cargo installation${NC}"
        return 1
    fi

    echo -e "${BLUE}Attempting installation via cargo...${NC}"

    # Detect if on macOS with Apple Silicon for MLX optimization
    local cargo_features=""
    if [[ "$(uname -s)" == "Darwin" ]] && [[ "$(uname -m)" == "arm64" ]]; then
        echo -e "${GREEN}Building with MLX optimization for Apple Silicon...${NC}"
        cargo_features="--features embedded-mlx"
    fi

    # Capture both stdout and stderr
    local install_output
    local install_exit_code

    install_output=$(cargo install caro $cargo_features 2>&1)
    install_exit_code=$?

    if [ $install_exit_code -eq 0 ]; then
        echo -e "${GREEN}✓ Successfully installed via cargo${NC}"
        return 0
    else
        echo -e "${RED}✗ Cargo installation failed${NC}"
        echo -e "${YELLOW}Error output:${NC}"
        echo "$install_output" | tail -10
        echo ""
        echo -e "${BLUE}Will try binary installation instead...${NC}"
        return 1
    fi
}

# Download file with retry logic
download_with_retry() {
    local url="$1"
    local output_path="$2"
    local max_retries=3
    local retry_delay=2

    for attempt in $(seq 1 $max_retries); do
        echo -e "${BLUE}Download attempt $attempt/$max_retries...${NC}"

        local download_success=false
        if command_exists curl; then
            if curl -fsSL "$url" -o "$output_path" 2>/dev/null; then
                download_success=true
            fi
        elif command_exists wget; then
            if wget -qO "$output_path" "$url" 2>/dev/null; then
                download_success=true
            fi
        fi

        if [ "$download_success" = true ]; then
            echo -e "${GREEN}✓ Download successful${NC}"
            return 0
        fi

        if [ $attempt -lt $max_retries ]; then
            echo -e "${YELLOW}Download failed, retrying in ${retry_delay}s...${NC}"
            sleep $retry_delay
            retry_delay=$((retry_delay * 2))  # Exponential backoff
        fi
    done

    echo -e "${RED}✗ Download failed after $max_retries attempts${NC}"
    return 1
}

# Try to install pre-built binary with error handling
try_binary_install() {
    echo -e "${BLUE}Attempting installation via pre-built binary...${NC}"

    # Check network connectivity first
    if ! check_network_connectivity; then
        return 1
    fi

    local platform
    platform=$(detect_platform) || return 1

    # Try to get latest release tag from GitHub
    local latest_url="https://api.github.com/repos/${REPO}/releases/latest"
    local release_info

    if command_exists curl; then
        release_info=$(curl -s "$latest_url" 2>/dev/null)
    elif command_exists wget; then
        release_info=$(wget -qO- "$latest_url" 2>/dev/null)
    else
        echo -e "${RED}Error: Neither curl nor wget found. Please install one of them.${NC}"
        return 1
    fi

    if [ -z "$release_info" ]; then
        echo -e "${RED}✗ Could not fetch release information from GitHub${NC}"
        return 1
    fi

    # Extract tag name (version)
    local version
    version=$(echo "$release_info" | grep '"tag_name":' | sed -E 's/.*"tag_name": "v?([^"]+)".*/\1/')

    if [ -z "$version" ]; then
        echo -e "${RED}✗ Could not determine latest version${NC}"
        return 1
    fi

    echo -e "${BLUE}Latest version: v${version}${NC}"

    # Map platform to base asset name
    local base_asset_name
    case "$platform" in
        linux-amd64)    base_asset_name="linux-amd64" ;;
        linux-arm64)    base_asset_name="linux-arm64" ;;
        macos-amd64)    base_asset_name="macos-intel" ;;
        macos-arm64)    base_asset_name="macos-silicon" ;;
        windows-amd64)  base_asset_name="windows-amd64.exe" ;;
        *)
            echo -e "${RED}✗ Unsupported platform: $platform${NC}"
            return 1
            ;;
    esac

    # Try versioned asset name first (v1.0.3+), fall back to non-versioned (v1.0.2)
    local versioned_asset_name="caro-${version}-${base_asset_name}"
    local legacy_asset_name="caro-${base_asset_name}"
    local asset_name="$versioned_asset_name"
    local binary_url="https://github.com/${REPO}/releases/download/v${version}/${versioned_asset_name}"
    local checksum_url="${binary_url}.sha256"

    echo -e "${BLUE}Downloading caro v${version} for ${platform}...${NC}"

    # Create temp file for download
    local temp_binary
    temp_binary=$(mktemp)

    # Try versioned name first, fall back to legacy name
    if download_with_retry "$binary_url" "$temp_binary"; then
        # Success with versioned name
        true
    else
        echo -e "${YELLOW}Versioned binary not found, trying legacy name...${NC}"
        asset_name="$legacy_asset_name"
        binary_url="https://github.com/${REPO}/releases/download/v${version}/${legacy_asset_name}"
        checksum_url="${binary_url}.sha256"

        if ! download_with_retry "$binary_url" "$temp_binary"; then
            rm -f "$temp_binary"
            echo -e "${RED}✗ Could not download binary for your platform${NC}"
            return 1
        fi
    fi

    # Move to final location
    mv "$temp_binary" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    # Download and verify checksum (optional - don't fail if checksum unavailable)
    local checksum_file
    checksum_file=$(mktemp)

    if command_exists curl; then
        curl -fsSL "$checksum_url" -o "$checksum_file" 2>/dev/null || true
    elif command_exists wget; then
        wget -qO "$checksum_file" "$checksum_url" 2>/dev/null || true
    fi

    # Verify checksum if available
    if [ -f "$checksum_file" ] && [ -s "$checksum_file" ]; then
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
        fi
    fi

    rm -f "$checksum_file"

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

# Download and install binary with fallback chain
install_binary() {
    echo -e "${BLUE}Installing caro...${NC}"

    # Fallback chain: cargo -> binary -> guided manual

    # Try cargo installation if explicitly chosen or available
    if [ "$INSTALL_METHOD" = "cargo" ] || { [ -z "$INSTALL_METHOD" ] && command_exists cargo; }; then
        if try_cargo_install; then
            return 0
        fi
        echo -e "${YELLOW}Cargo installation failed, falling back to binary...${NC}"
        echo ""
    fi

    # Try binary installation
    if try_binary_install; then
        return 0
    fi

    # All methods failed - provide guided manual installation
    echo ""
    echo -e "${RED}═══════════════════════════════════════════════════════${NC}"
    echo -e "${RED}     All automatic installation methods failed         ${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${BOLD}${YELLOW}Manual installation options:${NC}"
    echo ""

    echo -e "${CYAN}Option 1: Install Rust and use cargo${NC}"
    echo -e "  1. Install Rust from: ${BOLD}https://rustup.rs${NC}"
    echo -e "  2. Run: ${GREEN}cargo install caro${NC}"
    echo ""

    echo -e "${CYAN}Option 2: Download binary manually${NC}"
    echo -e "  1. Visit: ${BOLD}https://github.com/${REPO}/releases/latest${NC}"
    echo -e "  2. Download the binary for your platform"
    echo -e "  3. Move it to ${INSTALL_DIR}"
    echo -e "  4. Run: ${GREEN}chmod +x ${INSTALL_DIR}/caro${NC}"
    echo ""

    echo -e "${CYAN}Option 3: Check network connectivity${NC}"
    echo -e "  ${YELLOW}Test your connection:${NC}"
    echo -e "    ping github.com"
    echo -e "    curl -I https://github.com"
    echo -e "  ${YELLOW}Check proxy settings:${NC}"
    echo -e "    echo \$HTTP_PROXY"
    echo -e "    echo \$HTTPS_PROXY"
    echo ""

    echo -e "${BLUE}Need help? Visit: ${BOLD}https://github.com/${REPO}/issues${NC}"
    echo ""

    exit 1
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

    # Show mode indicator
    if [ "$INTERACTIVE_MODE" != "true" ]; then
        echo -e "${YELLOW}Running in non-interactive mode (piped execution detected)${NC}"
        echo -e "${YELLOW}Using default settings for automated installation${NC}"
        echo ""
    fi

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
