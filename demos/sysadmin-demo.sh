#!/bin/bash
# SysAdmin/DevOps/SRE Demo for Asciinema
# Realistic operations showcasing working commands

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

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Caro.sh - For SysAdmins, DevOps, and SREs"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
sleep 2

# Demo 1: Show log tail
echo "$ caro \"show last 3 lines in the file logs/app.log\""
sleep 1
$CARO_CMD -x "show last 3 lines in the file logs/app.log"
sleep 2
echo ""

# Demo 2: Find JavaScript files
echo "$ caro \"find all script files\""
sleep 1
$CARO_CMD -x "find all script files"
sleep 2
echo ""

# Demo 3: Show disk usage
echo "$ caro \"show disk usage summary\""
sleep 1
$CARO_CMD -x "show disk usage summary"
sleep 2
echo ""

# Demo 4: Show docker containers
echo "$ caro \"show docker containers\""
sleep 1
$CARO_CMD -x "show docker containers"
sleep 2
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🌐 github.com/wildcard/caro.sh"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
sleep 2
