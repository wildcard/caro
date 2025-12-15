#!/bin/bash
# Vancouver.Dev Community Demo - December 16th Edition
# Showcasing real-world developer and sysadmin scenarios

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
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🦀 Caro.sh - Natural Language → Shell Commands"
echo "  Vancouver.dev Edition - December 16, 2024"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
sleep 2

# Demo 1: Git detective work - find that breaking commit
echo "📍 Demo 1: Git Archaeology"
echo "$ caro \"show git commits from last 2 weeks with author names\""
sleep 1
$CARO_CMD -x "show git commits from last 2 weeks with author names"
sleep 3
echo ""

# Demo 2: System health check - production debugging
echo "📍 Demo 2: System Health Check"
echo "$ caro \"show top 5 processes by CPU usage\""
sleep 1
$CARO_CMD -x "show top 5 processes by CPU usage"
sleep 3
echo ""

# Demo 3: Code archaeology - find where things are implemented
echo "📍 Demo 3: Code Search"
echo "$ caro \"find all Rust files modified in the last 7 days\""
sleep 1
$CARO_CMD -x "find all Rust files modified in the last 7 days"
sleep 3
echo ""

# Demo 4: Network debugging - is the service even listening?
echo "📍 Demo 4: Network Debugging"
echo "$ caro \"show all listening TCP ports\""
sleep 1
$CARO_CMD -x "show all listening TCP ports"
sleep 3
echo ""

# Demo 5: Disk space investigation - who's eating all the storage?
echo "📍 Demo 5: Disk Space Analysis"
echo "$ caro \"show disk usage sorted by size\""
sleep 1
$CARO_CMD -x "show disk usage sorted by size"
sleep 3
echo ""

# Demo 6: Log analysis - find those error spikes
echo "📍 Demo 6: Log Analysis"
echo "$ caro \"find all log files and count total lines\""
sleep 1
$CARO_CMD -x "find all log files and count total lines"
sleep 3
echo ""

# Stats and features
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ✨ Key Features:"
echo "     • 100% local - no cloud, no tracking"
echo "     • Optimized for Apple Silicon (MLX)"
echo "     • Safety-first: blocks dangerous commands"
echo "     • <100ms startup, <2s inference"
echo ""
echo "  🚀 Try it yourself:"
echo "     brew tap wildcard/tap"
echo "     brew install caro"
echo ""
echo "  📦 Open Source:"
echo "     github.com/wildcard/caro"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
sleep 3
