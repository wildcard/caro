#!/bin/bash
# Check if Caro or caro is installed and provide installation guidance

set -euo pipefail

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Checking for Caro/caro installation..."
echo ""

# Check for caro alias (preferred)
if command -v caro &> /dev/null; then
    echo -e "${GREEN}✓ caro found:${NC} $(which caro)"
    echo ""
    caro --version
    echo ""
    echo -e "${GREEN}✓ Installation verified!${NC}"
    echo ""
    echo "You can use 'caro' to generate commands:"
    echo "  $ caro \"your natural language description\""
    exit 0
fi

# Check for Caro binary
if command -v Caro &> /dev/null; then
    echo -e "${GREEN}✓ Caro found:${NC} $(which Caro)"
    echo ""
    Caro --version
    echo ""
    echo -e "${YELLOW}Note:${NC} Caro is installed, but 'caro' alias is not set up."
    echo ""
    echo "Add this alias to your shell config:"
    echo "  alias caro='Caro'"
    echo ""
    echo "Or use 'Caro' directly:"
    echo "  $ Caro \"your natural language description\""
    exit 0
fi

# Neither found - provide installation instructions
echo -e "${RED}✗ Caro/caro not found${NC}"
echo ""
echo "Install Caro using one of these methods:"
echo ""
echo "Option 1: One-line setup (Recommended)"
echo -e "${GREEN}  bash <(curl -sSfL https://setup.caro.sh)${NC}"
echo ""
echo "Option 2: Using cargo"
echo -e "${GREEN}  cargo install Caro${NC}"
echo ""
echo "Option 3: Pre-built binaries"
echo "  Download from: https://github.com/wildcard/caro/releases/latest"
echo ""
echo "After installation, use 'caro' as the command alias."
exit 1
