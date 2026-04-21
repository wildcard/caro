#!/usr/bin/env bash
# Update Homebrew formula with correct SHA256 checksums for a release
#
# Usage: ./scripts/update-homebrew-formula.sh [VERSION]
#
# If VERSION is not provided, reads from Cargo.toml

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FORMULA_PATH="$PROJECT_ROOT/homebrew-tap/Formula/caro.rb"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get version from argument or Cargo.toml
if [[ $# -ge 1 ]]; then
    VERSION="$1"
else
    VERSION=$(grep -m1 '^version = ' "$CARGO_TOML" | cut -d'"' -f2)
fi

if [[ -z "$VERSION" ]]; then
    log_error "Could not determine version"
    exit 1
fi

log_info "Updating Homebrew formula for version $VERSION"

# GitHub release URL base
RELEASE_BASE="https://github.com/wildcard/caro/releases/download/v${VERSION}"

# Platforms to update
declare -A PLATFORMS=(
    ["macos-silicon"]="PLACEHOLDER_MACOS_SILICON_SHA256"
    ["macos-intel"]="PLACEHOLDER_MACOS_INTEL_SHA256"
    ["linux-arm64"]="PLACEHOLDER_LINUX_ARM64_SHA256"
    ["linux-amd64"]="PLACEHOLDER_LINUX_AMD64_SHA256"
)

# Create a temporary directory
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Download and compute SHA256 for each platform
declare -A CHECKSUMS
for platform in "${!PLATFORMS[@]}"; do
    BINARY_URL="${RELEASE_BASE}/caro-${VERSION}-${platform}"
    log_info "Downloading $platform binary..."

    if curl -fsSL "$BINARY_URL" -o "$TMPDIR/caro-${platform}" 2>/dev/null; then
        CHECKSUM=$(sha256sum "$TMPDIR/caro-${platform}" | cut -d' ' -f1)
        CHECKSUMS[$platform]="$CHECKSUM"
        log_info "  SHA256: $CHECKSUM"
    else
        log_warn "Could not download $platform binary from $BINARY_URL"
        log_warn "  The release may not be available yet. Skipping..."
    fi
done

# Check if we have all checksums
if [[ ${#CHECKSUMS[@]} -ne ${#PLATFORMS[@]} ]]; then
    log_warn "Not all platforms were available. Formula will have placeholder values."
    log_warn "Available checksums:"
    for platform in "${!CHECKSUMS[@]}"; do
        echo "  $platform: ${CHECKSUMS[$platform]}"
    done
fi

# Update the formula file
log_info "Updating formula at $FORMULA_PATH"

# First, update the version
sed -i "s/version \".*\"/version \"$VERSION\"/" "$FORMULA_PATH"

# Update each platform's SHA256
for platform in "${!CHECKSUMS[@]}"; do
    placeholder="${PLATFORMS[$platform]}"
    checksum="${CHECKSUMS[$platform]}"

    # Handle both placeholder and existing SHA256 patterns
    # First try to replace a placeholder
    if grep -q "$placeholder" "$FORMULA_PATH"; then
        sed -i "s/$placeholder/$checksum/" "$FORMULA_PATH"
    else
        # Replace existing SHA256 for this platform's URL
        # This is more complex - we need to find the right sha256 line
        case "$platform" in
            "macos-silicon")
                sed -i "/macos-silicon/,/sha256/{s/sha256 \"[a-f0-9]*\"/sha256 \"$checksum\"/}" "$FORMULA_PATH"
                ;;
            "macos-intel")
                sed -i "/macos-intel/,/sha256/{s/sha256 \"[a-f0-9]*\"/sha256 \"$checksum\"/}" "$FORMULA_PATH"
                ;;
            "linux-arm64")
                sed -i "/linux-arm64/,/sha256/{s/sha256 \"[a-f0-9]*\"/sha256 \"$checksum\"/}" "$FORMULA_PATH"
                ;;
            "linux-amd64")
                sed -i "/linux-amd64/,/sha256/{s/sha256 \"[a-f0-9]*\"/sha256 \"$checksum\"/}" "$FORMULA_PATH"
                ;;
        esac
    fi
done

log_info "Formula updated successfully!"
log_info ""
log_info "Next steps:"
log_info "  1. Review the changes: git diff homebrew-tap/Formula/caro.rb"
log_info "  2. Commit: git add homebrew-tap && git commit -m 'chore(brew): update formula to v$VERSION'"
log_info "  3. Push to main branch"
echo ""
