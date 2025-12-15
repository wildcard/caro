#!/bin/bash
# Vancouver.Dev Community Demo - Asciinema version
# Real command execution, clean output

# Find caro/cmdai binary
if command -v caro &> /dev/null; then
    CARO_CMD="caro"
elif command -v cmdai &> /dev/null; then
    CARO_CMD="cmdai"
elif [ -f "../../../target/release/cmdai" ]; then
    CARO_CMD="../../../target/release/cmdai"
elif [ -f "../../../target/debug/cmdai" ]; then
    CARO_CMD="../../../target/debug/cmdai"
else
    echo "Error: Cannot find caro or cmdai binary"
    echo "Please install cmdai or run 'cargo build --release' first"
    exit 1
fi

# Banner
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Caro.sh - Your Terminal's AI Companion"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
sleep 2

# Demo 1: List files
echo "$ caro \"list all files in this directory\""
sleep 1
$CARO_CMD -x "list all files in this directory"
sleep 2
echo ""

# Demo 2: Find JavaScript files
echo "$ caro \"find all JavaScript files\""
sleep 1
$CARO_CMD -x "find all JavaScript files"
sleep 2
echo ""

# Demo 3: Count lines in source files
echo "$ caro \"find all JavaScript files and count their total lines\""
sleep 1
$CARO_CMD -x "find all JavaScript files and count their total lines"
sleep 2
echo ""

# Demo 4: Show README
echo "$ caro \"show first 10 lines of README.md\""
sleep 1
$CARO_CMD -x "show first 10 lines of README.md"
sleep 2
echo ""

# Closing
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🌐 github.com/wildcard/caro"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
sleep 2
