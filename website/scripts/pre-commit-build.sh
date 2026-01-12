#!/bin/bash
# Pre-commit hook for website: runs build to catch Astro/JSX syntax errors
# Install: ln -sf ../../website/scripts/pre-commit-build.sh .git/hooks/pre-commit

set -e

# Check if any website files are staged
STAGED_WEBSITE_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep "^website/" || true)

if [ -z "$STAGED_WEBSITE_FILES" ]; then
    exit 0
fi

echo "Website files staged for commit. Running build verification..."

# Save current directory
ORIGINAL_DIR=$(pwd)

# Navigate to website directory
cd "$(git rev-parse --show-toplevel)/website"

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install --silent
fi

# Run build
echo "Running: npm run build"
if npm run build > /tmp/website-build.log 2>&1; then
    echo "Build passed!"
    cd "$ORIGINAL_DIR"
    exit 0
else
    echo ""
    echo "BUILD FAILED! Please fix errors before committing."
    echo ""
    echo "Common issues in .astro files:"
    echo "  - Unescaped { } in code blocks (use &#123; and &#125;)"
    echo "  - Unescaped < > in code blocks (use &lt; and &gt;)"
    echo "  - Unescaped & in code blocks (use &amp;)"
    echo ""
    echo "Build output:"
    tail -30 /tmp/website-build.log
    cd "$ORIGINAL_DIR"
    exit 1
fi
