#!/bin/bash
# Website Hero Demo - Quick and impactful
# Perfect for embedding on caro.sh homepage

# Find caro/cmdai binary
if command -v caro &> /dev/null; then
    CARO_CMD="caro"
elif command -v cmdai &> /dev/null; then
    CARO_CMD="cmdai"
elif [ -f "../../target/release/cmdai" ]; then
    CARO_CMD="../../target/release/cmdai"
elif [ -f "../../target/debug/cmdai" ]; then
    CARO_CMD="../../target/debug/cmdai"
else
    echo "Error: Cannot find caro or cmdai binary"
    echo "Please install cmdai or run 'cargo build --release' first"
    exit 1
fi

# Quick intro
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Caro.sh - Your Terminal's AI Companion"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
sleep 1

# Demo 1: File listing
echo "$ caro \"list all files\""
sleep 0.5
$CARO_CMD -x "list all files"
sleep 1.5
echo ""

# Demo 2: Find files
echo "$ caro \"find JavaScript files\""
sleep 0.5
$CARO_CMD -x "find JavaScript files"
sleep 1.5
echo ""

# Demo 3: Count lines
echo "$ caro \"find all JavaScript files and count their total lines\""
sleep 0.5
$CARO_CMD -x "find all JavaScript files and count their total lines"
sleep 1.5
echo ""

# Demo 4: Show README
echo "$ caro \"show first 10 lines of README.md\""
sleep 0.5
$CARO_CMD -x "show first 10 lines of README.md"
sleep 1.5
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🌐 github.com/wildcard/caro.sh"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
sleep 2
