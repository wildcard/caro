#!/usr/bin/env bash
# cmdai installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
#
# This script installs cmdai to /usr/local/bin by default
# You can customize the installation with:
#   curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash -s -- --install-dir ~/.local/bin

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
INSTALL_DIR="${CMDAI_INSTALL_DIR:-/usr/local/bin}"
REPO="wildcard/cmdai"
BINARY_NAME="cmdai"
USE_SUDO=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --no-sudo)
            USE_SUDO="no"
            shift
            ;;
        --help|-h)
            echo "cmdai installer"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install-dir DIR    Install to DIR (default: /usr/local/bin)"
            echo "  --no-sudo            Don't use sudo for installation"
            echo "  --help, -h           Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Helper functions
info() {
    echo -e "${BLUE}==>${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1" >&2
}

warning() {
    echo -e "${YELLOW}!${NC} $1"
}

# Detect OS and architecture
detect_platform() {
    local os arch

    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="macos"
            ;;
        msys*|mingw*|cygwin*)
            OS="windows"
            ;;
        *)
            error "Unsupported operating system: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    info "Detected platform: $OS-$ARCH"
}

# Get the latest release version
get_latest_version() {
    info "Fetching latest version..."

    # Try GitHub API first
    LATEST_VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

    if [ -z "$LATEST_VERSION" ]; then
        error "Failed to fetch latest version"
        exit 1
    fi

    # Remove 'v' prefix if present
    VERSION="${LATEST_VERSION#v}"
    success "Latest version: $VERSION"
}

# Construct download URL
get_download_url() {
    local filename

    if [ "$OS" = "macos" ]; then
        if [ "$ARCH" = "arm64" ]; then
            filename="cmdai-macos-silicon"
        else
            filename="cmdai-macos-intel"
        fi
    elif [ "$OS" = "linux" ]; then
        filename="cmdai-linux-${ARCH}"
    elif [ "$OS" = "windows" ]; then
        filename="cmdai-windows-${ARCH}.exe"
    fi

    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_VERSION/$filename"
    CHECKSUM_URL="https://github.com/$REPO/releases/download/$LATEST_VERSION/${filename}.sha256"

    info "Download URL: $DOWNLOAD_URL"
}

# Download and verify binary
download_binary() {
    local tmp_dir tmp_file

    tmp_dir=$(mktemp -d)
    tmp_file="$tmp_dir/$BINARY_NAME"

    info "Downloading cmdai $VERSION..."

    if ! curl -fSL "$DOWNLOAD_URL" -o "$tmp_file"; then
        error "Failed to download cmdai"
        rm -rf "$tmp_dir"
        exit 1
    fi

    success "Downloaded successfully"

    # Download and verify checksum
    info "Verifying checksum..."
    if curl -fSL "$CHECKSUM_URL" -o "$tmp_file.sha256"; then
        cd "$tmp_dir"
        if command -v sha256sum >/dev/null 2>&1; then
            echo "$(cat "$BINARY_NAME.sha256")" | sha256sum -c - || {
                error "Checksum verification failed"
                rm -rf "$tmp_dir"
                exit 1
            }
        elif command -v shasum >/dev/null 2>&1; then
            shasum -a 256 -c "$BINARY_NAME.sha256" || {
                error "Checksum verification failed"
                rm -rf "$tmp_dir"
                exit 1
            }
        else
            warning "No checksum tool found, skipping verification"
        fi
        cd - >/dev/null
        success "Checksum verified"
    else
        warning "Could not download checksum file, skipping verification"
    fi

    # Make binary executable
    chmod +x "$tmp_file"

    TEMP_BINARY="$tmp_file"
    TEMP_DIR="$tmp_dir"
}

# Install binary
install_binary() {
    info "Installing to $INSTALL_DIR..."

    # Create install directory if it doesn't exist
    if [ ! -d "$INSTALL_DIR" ]; then
        if [ "$USE_SUDO" != "no" ] && [ ! -w "$(dirname "$INSTALL_DIR")" ]; then
            sudo mkdir -p "$INSTALL_DIR"
        else
            mkdir -p "$INSTALL_DIR"
        fi
    fi

    # Check if we need sudo for installation
    local use_sudo_cmd=""
    if [ "$USE_SUDO" != "no" ] && [ ! -w "$INSTALL_DIR" ]; then
        use_sudo_cmd="sudo"
        warning "Requires sudo for installation to $INSTALL_DIR"
    fi

    # Install binary
    if ! $use_sudo_cmd cp "$TEMP_BINARY" "$INSTALL_DIR/$BINARY_NAME"; then
        error "Failed to install cmdai to $INSTALL_DIR"
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    # Clean up
    rm -rf "$TEMP_DIR"

    success "cmdai installed successfully!"
}

# Check if install directory is in PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warning "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add this to your shell configuration (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
    fi
}

# Print post-install instructions
post_install() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    success "cmdai $VERSION installed successfully!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Get started with:"
    echo -e "  ${GREEN}cmdai --help${NC}"
    echo ""
    echo "Generate your first command:"
    echo -e "  ${GREEN}cmdai \"list all PDF files larger than 10MB\"${NC}"
    echo ""
    echo "Optional: Add shell alias for quicker access"
    echo "  Add to your ~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish:"
    echo ""
    echo -e "    ${BLUE}alias ai='cmdai'${NC}"
    echo ""
    echo "Configuration (optional):"
    echo "  Create: ~/.config/cmdai/config.toml"
    echo "  See: https://github.com/$REPO#configuration"
    echo ""
    echo "For more information:"
    echo "  Documentation: https://github.com/$REPO"
    echo "  Full install guide: https://github.com/$REPO/blob/main/INSTALL.md"
    echo ""
}

# Show installation configuration and get confirmation
show_config_and_confirm() {
    echo ""
    echo "  Configuration"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}>${NC} Install directory: ${GREEN}$INSTALL_DIR${NC}"
    echo -e "${BLUE}>${NC} Platform:          ${GREEN}$OS${NC}"
    echo -e "${BLUE}>${NC} Architecture:      ${GREEN}$ARCH${NC}"
    echo -e "${BLUE}>${NC} Version:           ${GREEN}$VERSION${NC}"
    echo ""
    echo -e "${BLUE}>${NC} Download URL:"
    echo "  $DOWNLOAD_URL"
    echo ""

    # Ask for confirmation
    read -p "? Install cmdai $VERSION to $INSTALL_DIR? [y/N] " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        warning "Installation cancelled by user"
        exit 0
    fi
}

# Main installation flow
main() {
    echo ""
    echo "╔═══════════════════════════════════════╗"
    echo "║      cmdai Installer v1.0.0          ║"
    echo "╚═══════════════════════════════════════╝"
    echo ""

    detect_platform
    get_latest_version
    get_download_url
    show_config_and_confirm
    download_binary
    install_binary
    check_path
    post_install
}

main "$@"
