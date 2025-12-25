#!/usr/bin/env bash
#
# cmdai Short Demo Script (30 seconds)
#
# Optimized for:
# - GitHub README GIFs
# - Social media posts
# - Quick feature previews
# - File size < 10MB
#
# Usage:
#   chmod +x demo_short.sh
#   asciinema rec demo_short.cast -c ./demo_short.sh
#   agg demo_short.cast demo_short.gif --speed 2.0

set -e

# Configuration - Faster pacing for short demo
DEMO_SPEED="${DEMO_SPEED:-60}"
PAUSE_SHORT=1.0
PAUSE_MEDIUM=1.8

# Colors
RESET='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'
RED='\033[31m'
GREEN='\033[32m'
YELLOW='\033[33m'
CYAN='\033[36m'
BRIGHT_CYAN='\033[96m'
BRIGHT_GREEN='\033[92m'
BRIGHT_RED='\033[91m'

# Helper functions
type_command() {
    local cmd="$1"
    local speed="${2:-$DEMO_SPEED}"
    for ((i=0; i<${#cmd}; i++)); do
        echo -n "${cmd:$i:1}"
        sleep $(awk "BEGIN {print 1/$speed}")
    done
    echo
}

pause() {
    sleep "${1:-$PAUSE_SHORT}"
}

print_prompt() {
    echo -ne "${BRIGHT_GREEN}$${RESET} "
}

# Short demo scenes

scene_title() {
    clear
    echo
    echo -e "${BRIGHT_CYAN}${BOLD}"
    echo "  ╔════════════════════════════════════════════════╗"
    echo "  ║                                                ║"
    echo "  ║                   cmdai                        ║"
    echo "  ║                                                ║"
    echo "  ║     Natural Language → Shell Commands          ║"
    echo "  ║                                                ║"
    echo "  ╚════════════════════════════════════════════════╝"
    echo -e "${RESET}"
    pause 2.0
}

scene_simple() {
    clear
    echo -e "${DIM}# Simple query${RESET}"
    echo
    print_prompt
    type_command 'cmdai "list all files in current directory"'
    pause 0.5
    echo
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}ls -la${RESET}"
    echo
    echo -e "${DIM}⚡ 342ms${RESET}"
    pause $PAUSE_MEDIUM
}

scene_complex() {
    clear
    echo -e "${DIM}# Complex query${RESET}"
    echo
    print_prompt
    type_command 'cmdai "find PDFs in Downloads over 10MB"'
    pause 0.5
    echo
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}find ~/Downloads -name \"*.pdf\" -size +10M${RESET}"
    echo
    echo -e "${DIM}⚡ 581ms${RESET}"
    pause $PAUSE_MEDIUM
}

scene_dangerous() {
    clear
    echo -e "${DIM}# Safety validation${RESET}"
    echo
    print_prompt
    type_command 'cmdai "delete everything in root"'
    pause 0.5
    echo
    echo -e "${BRIGHT_RED}${BOLD}🚫 Blocked${RESET}"
    echo
    echo -e "${RED}rm -rf /${RESET}"
    echo
    echo -e "${YELLOW}Critical risk: Would delete entire filesystem${RESET}"
    echo
    echo -e "${DIM}⚡ 198ms${RESET}"
    pause $PAUSE_MEDIUM
}

scene_interactive() {
    clear
    echo -e "${DIM}# Interactive confirmation${RESET}"
    echo
    print_prompt
    type_command 'cmdai "remove old log files"'
    pause 0.5
    echo
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}find . -name \"*.log\" -mtime +30 -delete${RESET}"
    echo
    echo -e "${YELLOW}⚠️  High risk: File deletion${RESET}"
    echo
    echo -ne "Execute? (y/N) "
    pause 1.0
    type_command "n" 80
    pause 0.3
    echo
    echo -e "${YELLOW}Cancelled by user${RESET}"
    echo
    echo -e "${DIM}⚡ 445ms${RESET}"
    pause $PAUSE_MEDIUM
}

scene_features() {
    clear
    echo
    echo -e "${BRIGHT_CYAN}${BOLD}cmdai Features${RESET}"
    echo
    echo -e "  ${GREEN}✓${RESET} Natural language interface"
    echo -e "  ${GREEN}✓${RESET} Safety validation"
    echo -e "  ${GREEN}✓${RESET} Interactive confirmations"
    echo -e "  ${GREEN}✓${RESET} Local LLM inference"
    echo -e "  ${GREEN}✓${RESET} Fast (<2s on M1)"
    echo
    echo -e "  ${CYAN}github.com/wildcard/cmdai${RESET}"
    echo
    pause 2.5
}

# Main sequence
main() {
    scene_title
    scene_simple
    scene_complex
    scene_dangerous
    scene_interactive
    scene_features

    # Optional: Loop indicator
    clear
    echo
    echo -e "${DIM}[Demo loops in 3 seconds...]${RESET}"
    pause 3.0
}

main
