#!/usr/bin/env bash
#
# caro agent-assisted installer
#
# This script outputs structured JSON for consumption by AI agents and LLMs.
# It detects the system environment and recommends installation steps.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install-agent.sh | bash
#   wget -qO- https://raw.githubusercontent.com/wildcard/caro/main/install-agent.sh | bash
#
# Output format: JSON with system detection and installation recommendations

set -e

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
        *)          os="unknown" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="amd64" ;;
        aarch64|arm64)  arch="arm64" ;;
        *)              arch="unknown" ;;
    esac

    echo "${os}-${arch}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect shell type
detect_shell() {
    local shell_path="$SHELL"
    local shell_name=$(basename "$shell_path" 2>/dev/null || echo "unknown")

    # Get version if possible
    local version="unknown"
    if command_exists "$shell_name"; then
        version=$("$shell_name" --version 2>/dev/null | head -1 || echo "unknown")
    fi

    echo "$shell_name"
}

# Check network connectivity
check_network() {
    if command_exists curl; then
        if curl -fsSL --connect-timeout 5 --max-time 10 https://github.com >/dev/null 2>&1; then
            echo "true"
            return
        fi
    elif command_exists wget; then
        if wget --timeout=10 --tries=1 -q --spider https://github.com 2>/dev/null; then
            echo "true"
            return
        fi
    fi
    echo "false"
}

# Detect proxy settings
detect_proxy() {
    local http_proxy="${HTTP_PROXY:-${http_proxy:-}}"
    local https_proxy="${HTTPS_PROXY:-${https_proxy:-}}"

    if [ -n "$https_proxy" ]; then
        echo "$https_proxy"
    elif [ -n "$http_proxy" ]; then
        echo "$http_proxy"
    else
        echo "none"
    fi
}

# Check if caro is already installed
check_existing_installation() {
    if command_exists caro; then
        local version=$(caro --version 2>/dev/null | head -1 || echo "unknown")
        echo "{\"installed\": true, \"version\": \"$version\", \"path\": \"$(command -v caro)\"}"
    else
        echo "{\"installed\": false}"
    fi
}

# Recommend installation method based on system
recommend_method() {
    local has_cargo="false"
    local has_curl="false"
    local has_wget="false"

    command_exists cargo && has_cargo="true"
    command_exists curl && has_curl="true"
    command_exists wget && has_wget="true"

    local platform=$(detect_platform)
    local recommended="cargo"
    local reason="Cargo allows building from source with optimizations"

    if [ "$has_cargo" = "false" ]; then
        if [ "$has_curl" = "true" ] || [ "$has_wget" = "true" ]; then
            recommended="binary"
            reason="Pre-built binary is fastest when cargo is not available"
        else
            recommended="manual"
            reason="Neither cargo nor download tools (curl/wget) are available"
        fi
    fi

    # Special case: macOS Apple Silicon should prefer cargo for MLX optimization
    if [[ "$platform" == "macos-arm64" ]] && [ "$has_cargo" = "true" ]; then
        recommended="cargo"
        reason="Building from source enables MLX optimization for Apple Silicon"
    fi

    echo "{\"method\": \"$recommended\", \"reason\": \"$reason\"}"
}

# Generate installation steps based on recommended method
generate_steps() {
    local method="$1"
    local platform=$(detect_platform)

    case "$method" in
        cargo)
            if [[ "$platform" == "macos-arm64" ]]; then
                cat <<'EOF'
[
  {
    "step": 1,
    "description": "Install caro via cargo with MLX optimization",
    "command": "cargo install caro --features embedded-mlx",
    "expected_output": "Installing caro",
    "success_indicator": "Installed package `caro"
  },
  {
    "step": 2,
    "description": "Verify installation",
    "command": "caro --version",
    "expected_output": "caro",
    "success_indicator": "caro"
  }
]
EOF
            else
                cat <<'EOF'
[
  {
    "step": 1,
    "description": "Install caro via cargo",
    "command": "cargo install caro",
    "expected_output": "Installing caro",
    "success_indicator": "Installed package `caro"
  },
  {
    "step": 2,
    "description": "Verify installation",
    "command": "caro --version",
    "expected_output": "caro",
    "success_indicator": "caro"
  }
]
EOF
            fi
            ;;
        binary)
            cat <<'EOF'
[
  {
    "step": 1,
    "description": "Run the standard installation script",
    "command": "curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | CARO_INTERACTIVE=false bash",
    "expected_output": "Installation Complete",
    "success_indicator": "✓"
  },
  {
    "step": 2,
    "description": "Verify installation",
    "command": "caro --version",
    "expected_output": "caro",
    "success_indicator": "caro"
  }
]
EOF
            ;;
        manual)
            cat <<'EOF'
[
  {
    "step": 1,
    "description": "Install Rust toolchain",
    "command": "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
    "expected_output": "Rust is installed now",
    "success_indicator": "rustc"
  },
  {
    "step": 2,
    "description": "Source cargo environment",
    "command": "source $HOME/.cargo/env",
    "expected_output": "",
    "success_indicator": ""
  },
  {
    "step": 3,
    "description": "Install caro via cargo",
    "command": "cargo install caro",
    "expected_output": "Installing caro",
    "success_indicator": "Installed package `caro"
  },
  {
    "step": 4,
    "description": "Verify installation",
    "command": "caro --version",
    "expected_output": "caro",
    "success_indicator": "caro"
  }
]
EOF
            ;;
    esac
}

# Main: Output JSON report
main() {
    local platform=$(detect_platform)
    local os="${platform%-*}"
    local arch="${platform#*-}"
    local shell=$(detect_shell)
    local network=$(check_network)
    local proxy=$(detect_proxy)
    local has_cargo=$(command_exists cargo && echo "true" || echo "false")
    local has_curl=$(command_exists curl && echo "true" || echo "false")
    local has_wget=$(command_exists wget && echo "true" || echo "false")
    local existing=$(check_existing_installation)
    local recommendation=$(recommend_method)
    local recommended_method=$(echo "$recommendation" | grep -o '"method": "[^"]*"' | cut -d'"' -f4)
    local steps=$(generate_steps "$recommended_method")

    # Output structured JSON
    cat <<EOF
{
  "version": "1.0.0",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "system": {
    "os": "$os",
    "architecture": "$arch",
    "platform": "$platform",
    "shell": "$shell"
  },
  "environment": {
    "has_cargo": $has_cargo,
    "has_curl": $has_curl,
    "has_wget": $has_wget,
    "network_available": $network,
    "proxy": "$proxy",
    "install_dir": "$INSTALL_DIR"
  },
  "existing_installation": $existing,
  "recommendation": $recommendation,
  "installation_steps": $steps,
  "troubleshooting": {
    "no_network": "Check internet connection and proxy settings (HTTP_PROXY, HTTPS_PROXY)",
    "no_cargo": "Install Rust from https://rustup.rs or download pre-built binary",
    "no_download_tools": "Install curl or wget to download pre-built binaries",
    "permission_denied": "Ensure $INSTALL_DIR is writable or set CARO_INSTALL_DIR to a writable location"
  },
  "links": {
    "documentation": "https://caro.sh",
    "repository": "https://github.com/$REPO",
    "releases": "https://github.com/$REPO/releases/latest",
    "issues": "https://github.com/$REPO/issues"
  }
}
EOF
}

main "$@"
