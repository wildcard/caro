#!/bin/bash
# Script to configure Xcode and download Metal Toolchain

set -e

echo "🔧 Configuring Xcode for MLX Development"
echo "=========================================="
echo ""

echo "Step 1: Switching xcode-select to Xcode..."
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer

echo "✅ Switched to Xcode"
echo ""

echo "Step 2: Verifying xcode-select path..."
xcode-select -p

echo ""
echo "Step 3: Accepting Xcode license (if needed)..."
sudo xcodebuild -license accept 2>/dev/null || echo "License already accepted"

echo ""
echo "Step 4: Downloading Metal Toolchain (may take a few minutes)..."
xcodebuild -downloadComponent MetalToolchain

echo ""
echo "Step 5: Verifying Metal compiler..."
xcrun --find metal

echo ""
echo "Step 6: Testing Metal version..."
metal --version

echo ""
echo "=========================================="
echo "✅ Xcode Configuration Complete!"
echo "=========================================="
echo ""
echo "You can now build cmdai with MLX support:"
echo "  cd /Users/kobi/personal/cmdai"
echo "  cargo clean"
echo "  cargo build --release --features embedded-mlx"
echo ""
