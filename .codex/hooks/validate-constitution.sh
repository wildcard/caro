#!/bin/bash
# Constitution Validation Hook
# Runs after git push to validate changes against consolidated knowledge rules

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo ""
echo "=========================================="
echo "  Constitution Validation Check"
echo "=========================================="

VIOLATIONS=0

# Rule 1: Check for forbidden installation script patterns
echo ""
echo "Checking installation script patterns..."

# Check for pipe-to-bash patterns (forbidden)
PIPE_VIOLATIONS=$(grep -rn "curl.*caro.sh/install.sh" --include="*.md" --include="*.astro" --include="*.ts" --include="*.tsx" . 2>/dev/null || true)
if [ -n "$PIPE_VIOLATIONS" ]; then
    echo -e "${RED}VIOLATION: Found forbidden installation script patterns:${NC}"
    echo "$PIPE_VIOLATIONS"
    echo ""
    echo -e "${YELLOW}Use instead: bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Check for curl | bash patterns
CURL_PIPE=$(grep -rn "curl.*|.*bash" --include="*.md" --include="*.astro" --include="*.ts" --include="*.tsx" . 2>/dev/null | grep -v "\.git" | grep -v "node_modules" || true)
if [ -n "$CURL_PIPE" ]; then
    echo -e "${RED}VIOLATION: Found pipe-to-bash patterns:${NC}"
    echo "$CURL_PIPE"
    echo ""
    echo -e "${YELLOW}Use process substitution instead: bash <(curl ...)${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Rule 2: Check for missing internal links
echo ""
echo "Checking internal page links..."

# Check for "installation guide" without href
MISSING_INSTALL_LINK=$(grep -rn "installation guide" --include="*.astro" --include="*.md" website/ 2>/dev/null | grep -v "href=" | grep -v "/#download" || true)
if [ -n "$MISSING_INSTALL_LINK" ]; then
    echo -e "${YELLOW}WARNING: Found 'installation guide' without link to /#download:${NC}"
    echo "$MISSING_INSTALL_LINK"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Check for "see our FAQ" or "see the FAQ" without link
MISSING_FAQ_LINK=$(grep -rn -E "(see (our |the )?FAQ|visit the FAQ)" --include="*.astro" --include="*.md" website/ 2>/dev/null | grep -v "href=" | grep -v "/faq" || true)
if [ -n "$MISSING_FAQ_LINK" ]; then
    echo -e "${YELLOW}WARNING: Found FAQ reference without link:${NC}"
    echo "$MISSING_FAQ_LINK"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Summary
echo ""
echo "=========================================="
if [ $VIOLATIONS -eq 0 ]; then
    echo -e "${GREEN}Constitution Check: PASSED${NC}"
    echo "No violations found."
else
    echo -e "${RED}Constitution Check: $VIOLATIONS violation(s) found${NC}"
    echo ""
    echo "Please fix these issues. The canonical installation command is:"
    echo ""
    echo "  bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)"
    echo ""
    echo "When referencing pages, use proper links:"
    echo "  - Installation guide → /#download"
    echo "  - FAQ → /faq"
    echo "  - Telemetry → /telemetry"
fi
echo "=========================================="
echo ""

# Don't fail the hook - just warn
# Remove 'exit 0' if you want to block pushes with violations
exit 0
