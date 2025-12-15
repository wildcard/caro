#!/bin/bash
# Setup script for caro alias

set -e

echo "Setting up caro alias..."

# Get the absolute path to the project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY_PATH="$PROJECT_ROOT/target/release/cmdai"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Error: Binary not found at $BINARY_PATH"
    echo "Please build first: cargo build --release --features embedded-mlx"
    exit 1
fi

# Create alias for current session
alias caro="$BINARY_PATH"

# Add to shell config
if [ -n "$ZSH_VERSION" ]; then
    # zsh
    SHELL_CONFIG="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    # bash
    SHELL_CONFIG="$HOME/.bashrc"
else
    echo "⚠️  Unknown shell, please add alias manually:"
    echo "alias caro='$BINARY_PATH'"
    exit 0
fi

# Check if alias already exists
if grep -q "alias caro=" "$SHELL_CONFIG" 2>/dev/null; then
    echo "✅ Alias already exists in $SHELL_CONFIG"
else
    echo "" >> "$SHELL_CONFIG"
    echo "# Caro - Terminal AI Companion" >> "$SHELL_CONFIG"
    echo "alias caro='$BINARY_PATH'" >> "$SHELL_CONFIG"
    echo "✅ Added alias to $SHELL_CONFIG"
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "To use caro in this session:"
echo "  alias caro='$BINARY_PATH'"
echo ""
echo "Or restart your shell:"
echo "  source $SHELL_CONFIG"
echo ""
echo "Then run:"
echo "  caro \"list all files\""
