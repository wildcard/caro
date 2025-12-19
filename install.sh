#!/usr/bin/env bash
#
# cmdai (caro) installer
#
# This script downloads and installs pre-built binaries for cmdai.
# It will automatically detect your platform and download the appropriate binary.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
#   wget -qO- https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
#
# Environment Variables:
#   CMDAI_INSTALL_DIR - Installation directory (default: ~/.local/bin)
#   CMDAI_VERSION     - Specific version to install (default: latest)
#   CMDAI_NO_MODIFY_PATH - Set to 1 to skip PATH modification
#   CMDAI_FORCE_CARGO - Set to 1 to force cargo installation

set -euo pipefail

# Configuration
readonly REPO_OWNER="wildcard"
readonly REPO_NAME="cmdai"
readonly BINARY_NAME="cmdai"
readonly ALIAS_NAME="caro"
readonly GITHUB_API="https://api.github.com"
readonly GITHUB_RELEASES="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"

INSTALL_DIR="${CMDAI_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${CMDAI_VERSION:-latest}"
FORCE_CARGO="${CMDAI_FORCE_CARGO:-0}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Logging functions
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

fatal() {
    error "$1"
    exit 1
}

# Check if a command exists
has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# Detect the operating system
detect_os() {
    local os
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="darwin"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os="windows"
            ;;
        *)
            fatal "Unsupported operating system: $(uname -s)"
            ;;
    esac
    echo "$os"
}

# Detect the architecture
detect_arch() {
    local arch
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        armv7l|armhf)
            arch="armv7"
            ;;
        *)
            fatal "Unsupported architecture: $(uname -m)"
            ;;
    esac
    echo "$arch"
}

# Get the archive extension for the platform
get_archive_ext() {
    local os=$1
    if [ "$os" = "windows" ]; then
        echo "zip"
    else
        echo "tar.gz"
    fi
}

# Build the download URL for a release
get_download_url() {
    local version=$1
    local os=$2
    local arch=$3

    local archive_name="cmdai-${version}-${os}-${arch}"
    local ext
    ext=$(get_archive_ext "$os")

    echo "${GITHUB_RELEASES}/download/${version}/${archive_name}.${ext}"
}

# Fetch JSON from a URL
fetch_json() {
    local url=$1
    if has_cmd curl; then
        curl -fsSL -H "Accept: application/vnd.github.v3+json" "$url"
    elif has_cmd wget; then
        wget -qO- --header="Accept: application/vnd.github.v3+json" "$url"
    else
        fatal "Neither curl nor wget found. Please install one of them."
    fi
}

# Download a file
download_file() {
    local url=$1
    local output=$2
    info "Downloading from $url"
    if has_cmd curl; then
        curl -fsSL -o "$output" "$url"
    elif has_cmd wget; then
        wget -qO "$output" "$url"
    else
        fatal "Neither curl nor wget found. Please install one of them."
    fi
}

# Get the latest version from GitHub
get_latest_version() {
    local api_url="${GITHUB_API}/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"
    local response

    response=$(fetch_json "$api_url" 2>/dev/null) || {
        warn "Could not fetch latest release from GitHub API"
        echo ""
        return
    }

    # Extract tag_name from JSON response
    echo "$response" | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4
}

# Extract archive
extract_archive() {
    local archive=$1
    local dest=$2

    case "$archive" in
        *.tar.gz|*.tgz)
            tar -xzf "$archive" -C "$dest"
            ;;
        *.zip)
            if has_cmd unzip; then
                unzip -q "$archive" -d "$dest"
            elif has_cmd 7z; then
                7z x -o"$dest" "$archive" >/dev/null
            else
                fatal "Cannot extract zip file: neither unzip nor 7z found"
            fi
            ;;
        *)
            fatal "Unknown archive format: $archive"
            ;;
    esac
}

# Install via cargo as fallback
install_via_cargo() {
    if ! has_cmd cargo; then
        warn "Cargo not found. Please install Rust first: https://rustup.rs"
        info "Or set CMDAI_VERSION to a specific version to use pre-built binaries"
        fatal "No installation method available"
    fi

    info "Installing via cargo..."

    local cargo_args=()

    # Add MLX feature on Apple Silicon
    if [ "$(detect_os)" = "darwin" ] && [ "$(detect_arch)" = "aarch64" ]; then
        info "Detected Apple Silicon - building with MLX optimization"
        cargo_args+=(--features embedded-mlx)
    fi

    if [ "$VERSION" != "latest" ]; then
        cargo_args+=(--version "${VERSION#v}")
    fi

    if cargo install cmdai "${cargo_args[@]}"; then
        success "Installed cmdai via cargo"
        return 0
    else
        fatal "Failed to install via cargo"
    fi
}

# Install pre-built binary
install_binary() {
    local os arch version download_url

    os=$(detect_os)
    arch=$(detect_arch)

    info "Detected platform: ${os}-${arch}"

    # Get version to install
    if [ "$VERSION" = "latest" ]; then
        info "Fetching latest version..."
        version=$(get_latest_version)
        if [ -z "$version" ]; then
            warn "Could not determine latest version, falling back to cargo"
            install_via_cargo
            return
        fi
        info "Latest version: $version"
    else
        version="$VERSION"
        # Ensure version has 'v' prefix
        [[ "$version" != v* ]] && version="v$version"
    fi

    # Build download URL
    download_url=$(get_download_url "$version" "$os" "$arch")

    # Create temporary directory
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    local archive_ext
    archive_ext=$(get_archive_ext "$os")
    local archive_path="${tmp_dir}/cmdai.${archive_ext}"

    # Download the archive
    if ! download_file "$download_url" "$archive_path" 2>/dev/null; then
        warn "Pre-built binary not available for ${os}-${arch} (version: $version)"

        if [ "$FORCE_CARGO" = "1" ] || [ "$os" = "windows" ]; then
            install_via_cargo
            return
        fi

        # Try musl build for Linux x86_64
        if [ "$os" = "linux" ] && [ "$arch" = "x86_64" ]; then
            info "Trying musl (statically linked) build..."
            download_url=$(get_download_url "$version" "${os}" "${arch}-musl")
            if ! download_file "$download_url" "$archive_path" 2>/dev/null; then
                warn "Musl build also not available, falling back to cargo"
                install_via_cargo
                return
            fi
        else
            install_via_cargo
            return
        fi
    fi

    success "Downloaded release archive"

    # Extract the archive
    info "Extracting archive..."
    extract_archive "$archive_path" "$tmp_dir"

    # Find the binary
    local binary_path
    if [ "$os" = "windows" ]; then
        binary_path=$(find "$tmp_dir" -name "${BINARY_NAME}.exe" -type f | head -1)
    else
        binary_path=$(find "$tmp_dir" -name "${BINARY_NAME}" -type f | head -1)
    fi

    if [ -z "$binary_path" ]; then
        fatal "Could not find binary in archive"
    fi

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Install the binary
    local install_path="${INSTALL_DIR}/${BINARY_NAME}"
    if [ "$os" = "windows" ]; then
        install_path="${INSTALL_DIR}/${BINARY_NAME}.exe"
    fi

    cp "$binary_path" "$install_path"
    chmod +x "$install_path"

    success "Installed ${BINARY_NAME} to ${install_path}"
}

# Setup shell alias
setup_alias() {
    local shell_config=""
    local shell_name=""

    # Detect shell from SHELL environment variable
    case "${SHELL:-}" in
        */bash)
            shell_name="bash"
            if [ -f "$HOME/.bashrc" ]; then
                shell_config="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_config="$HOME/.bash_profile"
            fi
            ;;
        */zsh)
            shell_name="zsh"
            shell_config="${ZDOTDIR:-$HOME}/.zshrc"
            ;;
        */fish)
            shell_name="fish"
            shell_config="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
            ;;
        *)
            warn "Could not detect shell. Please manually add alias:"
            echo "    alias ${ALIAS_NAME}='${BINARY_NAME}'"
            return
            ;;
    esac

    if [ -z "$shell_config" ]; then
        warn "Shell config file not found. Please manually add alias:"
        echo "    alias ${ALIAS_NAME}='${BINARY_NAME}'"
        return
    fi

    # Check if alias already exists
    if grep -q "alias ${ALIAS_NAME}=" "$shell_config" 2>/dev/null; then
        info "Alias '${ALIAS_NAME}' already exists in ${shell_config}"
        return
    fi

    # Add alias
    info "Adding alias '${ALIAS_NAME}' to ${shell_config}..."
    {
        echo ""
        echo "# cmdai (caro) alias"
        echo "alias ${ALIAS_NAME}='${BINARY_NAME}'"
    } >> "$shell_config"

    success "Alias added successfully"
}

# Add install directory to PATH
setup_path() {
    if [ "${CMDAI_NO_MODIFY_PATH:-0}" = "1" ]; then
        return
    fi

    # Check if install dir is already in PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*)
            return
            ;;
    esac

    local shell_config=""

    case "${SHELL:-}" in
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
            shell_config="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
            ;;
    esac

    if [ -z "$shell_config" ]; then
        warn "${INSTALL_DIR} is not in your PATH"
        warn "Please add it manually to your shell configuration"
        return
    fi

    # Check if PATH is already configured
    if grep -q "${INSTALL_DIR}" "$shell_config" 2>/dev/null; then
        return
    fi

    info "Adding ${INSTALL_DIR} to PATH in ${shell_config}..."

    case "${SHELL:-}" in
        */fish)
            echo "fish_add_path ${INSTALL_DIR}" >> "$shell_config"
            ;;
        *)
            echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$shell_config"
            ;;
    esac

    success "PATH updated"
}

# Verify installation
verify_installation() {
    local binary_path="${INSTALL_DIR}/${BINARY_NAME}"

    if [ ! -x "$binary_path" ]; then
        # Check if it's in PATH from cargo install
        if has_cmd "$BINARY_NAME"; then
            binary_path=$(command -v "$BINARY_NAME")
        else
            fatal "Installation verification failed: binary not found"
        fi
    fi

    local version_output
    version_output=$("$binary_path" --version 2>/dev/null) || version_output="unknown"
    success "Verified installation: ${version_output}"
}

# Print usage information
print_usage() {
    cat << EOF

${BOLD}Installation Complete!${NC}

${BLUE}Usage:${NC}
    ${BINARY_NAME} "list all files in this directory"
    ${ALIAS_NAME} "find files modified in the last 24 hours"

${BLUE}Execute directly:${NC}
    ${ALIAS_NAME} -x "show disk usage"

${BLUE}Get help:${NC}
    ${ALIAS_NAME} --help

${BLUE}Documentation:${NC}
    https://caro.sh
    https://github.com/${REPO_OWNER}/${REPO_NAME}

${YELLOW}Note:${NC} You may need to restart your shell or run:
    source ~/.bashrc  (or ~/.zshrc, etc.)

EOF
}

# Main function
main() {
    echo -e "${BOLD}"
    cat << 'EOF'
   ____
  / ___|__ _ _ __ ___
 | |   / _` | '__/ _ \
 | |__| (_| | | | (_) |
  \____\__,_|_|  \___/

EOF
    echo -e "${NC}"
    echo "Natural Language to Shell Commands"
    echo "==================================="
    echo ""

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --force-cargo)
                FORCE_CARGO=1
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --version VERSION     Install specific version (default: latest)"
                echo "  --install-dir DIR     Installation directory (default: ~/.local/bin)"
                echo "  --force-cargo         Force installation via cargo"
                echo "  --help                Show this help message"
                exit 0
                ;;
            *)
                warn "Unknown option: $1"
                shift
                ;;
        esac
    done

    # Force cargo installation if requested
    if [ "$FORCE_CARGO" = "1" ]; then
        install_via_cargo
    else
        install_binary
    fi

    # Setup PATH and alias
    setup_path
    setup_alias

    # Verify installation
    verify_installation

    # Print usage
    print_usage
}

main "$@"
