#!/bin/bash
# Vancouver.Dev Live Demo Script
# Run this to test all commands before the presentation

set -e

CMDAI="./target/release/cmdai"
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Caro.sh Vancouver.Dev Demo Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check binary exists
if [ ! -f "$CMDAI" ]; then
    echo -e "${RED}Error: Binary not found at $CMDAI${NC}"
    echo "Run: cargo build --release --features embedded-mlx"
    exit 1
fi

echo -e "${GREEN}✓ Binary found${NC}"
echo ""

# Test each demo command
demo_commands=(
    "show system uptime and load average"
    "show top 10 processes by CPU usage"
    "find all rust files modified in the last 7 days"
    "archive current directory"
    "count lines in all log files"
    "find files with setuid bit enabled"
    "show users who logged in today"
    "check DNS resolution for api.example.com"
)

echo -e "${BLUE}Testing demo commands...${NC}"
echo ""

for cmd in "${demo_commands[@]}"; do
    echo -e "${BLUE}Testing: ${NC}\"$cmd\""
    output=$($CMDAI "$cmd" 2>&1 | grep -A1 "^Command:" | tail -1 | xargs)
    
    if [[ "$output" == "echo 'Unable to generate command'" ]]; then
        echo -e "${RED}✗ FAILED${NC} - Model couldn't generate"
    else
        echo -e "${GREEN}✓ Generated:${NC} $output"
    fi
    echo ""
    sleep 1
done

echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}Demo test complete!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Ready for Vancouver.Dev presentation!"
