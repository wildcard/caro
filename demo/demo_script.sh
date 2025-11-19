#!/usr/bin/env bash
#
# cmdai ASCII Cinema Demo Script
#
# This script demonstrates cmdai's features in a visual, engaging way.
# Can be recorded with asciinema or executed for live demos.
#
# Usage:
#   chmod +x demo_script.sh
#   asciinema rec demo.cast -c ./demo_script.sh
#
# Or for live demo:
#   ./demo_script.sh

set -e

# Configuration
DEMO_SPEED="${DEMO_SPEED:-50}"  # Characters per second for typing effect
PAUSE_SHORT=1.5
PAUSE_MEDIUM=2.5
PAUSE_LONG=4.0

# Colors (ANSI escape codes)
RESET='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'

# Foreground colors
BLACK='\033[30m'
RED='\033[31m'
GREEN='\033[32m'
YELLOW='\033[33m'
BLUE='\033[34m'
MAGENTA='\033[35m'
CYAN='\033[36m'
WHITE='\033[37m'

# Bright/Bold colors
BRIGHT_BLACK='\033[90m'
BRIGHT_RED='\033[91m'
BRIGHT_GREEN='\033[92m'
BRIGHT_YELLOW='\033[93m'
BRIGHT_BLUE='\033[94m'
BRIGHT_MAGENTA='\033[95m'
BRIGHT_CYAN='\033[96m'
BRIGHT_WHITE='\033[97m'

# Helper functions
type_command() {
    local cmd="$1"
    local speed="${2:-$DEMO_SPEED}"

    # Type character by character
    for ((i=0; i<${#cmd}; i++)); do
        echo -n "${cmd:$i:1}"
        sleep $(awk "BEGIN {print 1/$speed}")
    done
    echo
}

pause() {
    local duration="${1:-$PAUSE_SHORT}"
    sleep "$duration"
}

print_prompt() {
    echo -ne "${BRIGHT_GREEN}$${RESET} "
}

clear_screen() {
    clear
    # Move cursor to top
    tput cup 0 0
}

print_banner() {
    local text="$1"
    local width=66

    echo -e "${BRIGHT_CYAN}"
    echo "╔══════════════════════════════════════════════════════════════════╗"
    printf "║%-66s║\n" ""
    printf "║%*s%*s║\n" $(((${#text}+$width)/2)) "$text" $(((${#text}+$width)/2-${#text})) ""
    printf "║%-66s║\n" ""
    echo "╚══════════════════════════════════════════════════════════════════╝"
    echo -e "${RESET}"
}

print_section_header() {
    local text="$1"
    echo
    echo -e "${BRIGHT_BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo -e "${BRIGHT_BLUE}${BOLD}  $text${RESET}"
    echo -e "${BRIGHT_BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo
}

# Mock cmdai responses
mock_simple_list() {
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}ls -la${RESET}"
    echo
    echo -e "${BOLD}Explanation:${RESET}"
    echo "  Lists all files including hidden files with detailed information"
    echo
    echo -e "${DIM}⚡ Generated in 342ms${RESET}"
}

mock_complex_find() {
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}find ~/Downloads -name \"*.pdf\" -size +10M -printf \"%T@ %p\\n\" | sort -n | cut -d' ' -f2-${RESET}"
    echo
    echo -e "${BOLD}Explanation:${RESET}"
    echo "  Searches Downloads directory for PDF files over 10MB and sorts them"
    echo "  by modification time in ascending order"
    echo
    echo -e "${BOLD}Alternatives:${RESET}"
    echo -e "  ${DIM}• find ~/Downloads -name \"*.pdf\" -size +10M -ls | sort -k8,9${RESET}"
    echo -e "  ${DIM}• ls -lhS ~/Downloads/*.pdf | awk '\$5 ~ /[0-9]+M/ && \$5+0 > 10'${RESET}"
    echo
    echo -e "${DIM}⚡ Generated in 581ms${RESET}"
}

mock_dangerous_blocked() {
    echo -e "${BRIGHT_RED}${BOLD}🚫 Command Blocked${RESET}"
    echo
    echo -e "${BOLD}Generated command:${RESET}"
    echo -e "  ${RED}rm -rf /${RESET}"
    echo
    echo -e "${BOLD}Risk Level: ${BRIGHT_RED}CRITICAL${RESET}"
    echo
    echo -e "${RED}Blocked:${RESET} Detected 2 dangerous pattern(s) at Critical risk level (deletion, recursive)"
    echo -e "  ${RED}• Critical: Recursive deletion of root directory${RESET}"
    echo -e "  ${RED}• Critical: Command would delete entire filesystem${RESET}"
    echo
    echo -e "${YELLOW}This command is blocked in strict mode for safety.${RESET}"
    echo -e "${DIM}Use --safety permissive to override (NOT RECOMMENDED).${RESET}"
    echo
    echo -e "${DIM}⚡ Generated in 198ms${RESET}"
}

mock_moderate_risk_prompt() {
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}find . -name \"*.log\" -type f -mtime +30 -delete${RESET}"
    echo
    echo -e "${BOLD}Explanation:${RESET}"
    echo "  Finds and removes log files not modified in the last 30 days"
    echo
    echo -e "${YELLOW}⚠️  Warning: High risk level (deletion, recursive)${RESET}"
    echo -e "  ${YELLOW}• High: Recursive file deletion operation${RESET}"
    echo
}

mock_cancelled() {
    echo -e "${YELLOW}⚠️  Operation cancelled by user.${RESET}"
    echo
    echo -e "${DIM}⚡ Generated in 445ms${RESET}"
}

mock_verbose_output() {
    echo -e "${DIM}[DEBUG] Loading configuration from ~/.config/cmdai/config.toml${RESET}"
    pause 0.3
    echo -e "${DIM}[DEBUG] Using backend: Embedded (MLX - Apple Silicon)${RESET}"
    pause 0.3
    echo -e "${DIM}[DEBUG] Model: Qwen2.5-Coder-1.5B-Instruct-Q8${RESET}"
    pause 0.3
    echo -e "${DIM}[DEBUG] Safety level: Moderate${RESET}"
    pause 0.5
    echo
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}du -sh * | sort -h${RESET}"
    echo
    echo -e "${BOLD}Explanation:${RESET}"
    echo "  Displays disk usage for all items in current directory,"
    echo "  sorted from smallest to largest in human-readable format"
    echo
    echo -e "${BOLD}Debug Info:${RESET}"
    echo -e "  ${DIM}• Backend: Embedded/MLX${RESET}"
    echo -e "  ${DIM}• Model tokens: 127 (prompt), 43 (completion)${RESET}"
    echo -e "  ${DIM}• Inference time: 1,243ms${RESET}"
    echo -e "  ${DIM}• Safety checks: 0 patterns matched${RESET}"
    echo -e "  ${DIM}• Risk level: Safe${RESET}"
    echo
    echo -e "${DIM}⚡ Total: 1,334ms (generation: 1,243ms, safety: 91ms)${RESET}"
}

mock_json_output() {
    cat <<'EOF'
{
  "generated_command": "find . -maxdepth 1 -type d -exec sh -c 'echo -n \"{}: \"; find \"{}\" -type f | wc -l' \\;",
  "explanation": "Counts files in each immediate subdirectory",
  "executed": false,
  "blocked_reason": null,
  "requires_confirmation": false,
  "alternatives": [
    "for dir in */; do echo \"$dir: $(find \"$dir\" -type f | wc -l)\"; done"
  ],
  "shell_used": "bash",
  "risk_level": "Safe",
  "timing_info": {
    "generation_time_ms": 512,
    "safety_validation_ms": 34,
    "total_time_ms": 546
  }
}
EOF
}

mock_fish_shell() {
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}ps aux | awk '\$6 > 102400 {printf \"%s\\t%sMB\\t%s\\n\", \$2, \$6/1024, \$11}'${RESET}"
    echo
    echo -e "${BOLD}Explanation:${RESET}"
    echo "  Lists processes using more than 100MB of memory"
    echo "  (Fish-compatible POSIX command)"
    echo
    echo -e "${BOLD}Target Shell:${RESET} fish"
    echo -e "${GREEN}Safety: ✓ Safe${RESET} (no dangerous patterns detected)"
    echo
    echo -e "${DIM}⚡ Generated in 423ms${RESET}"
}

mock_workflow_response() {
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}mkdir -p archive && find ~/Downloads -type f \\( -iname \"*.jpg\" -o -iname \"*.jpeg\" -o -iname \"*.png\" \\) -exec sh -c 'mogrify -quality 80 \"\$1\" && mv \"\$1\" archive/' _ {} \\;${RESET}"
    echo
    echo -e "${BOLD}Explanation:${RESET}"
    echo "  Creates archive directory if needed, finds all image files (JPG, JPEG, PNG)"
    echo "  in Downloads, compresses each to 80% quality, and moves them to archive/"
    echo
    echo -e "${YELLOW}⚠️  Note: Requires ImageMagick (mogrify) to be installed${RESET}"
    echo -e "${YELLOW}⚠️  Note: This operation modifies original files before moving${RESET}"
    echo
    echo -e "${BOLD}Alternatives:${RESET}"
    echo -e "  ${DIM}• Use convert instead: for img in ~/Downloads/*.{jpg,png}; do convert \"\$img\" -quality 80 \"archive/\$(basename \"\$img\")\"; done${RESET}"
    echo
}

mock_confirmed() {
    echo -e "${GREEN}✓ Confirmed. Proceeding with command execution.${RESET}"
    echo
    echo -e "${BOLD}Command:${RESET}"
    echo -e "  ${BRIGHT_CYAN}${BOLD}mkdir -p archive && find ~/Downloads -type f \\( -iname \"*.jpg\" -o -iname \"*.jpeg\" -o -iname \"*.png\" \\) -exec sh -c 'mogrify -quality 80 \"\$1\" && mv \"\$1\" archive/' _ {} \\;${RESET}"
    echo
    echo -e "${DIM}⚡ Generated in 734ms${RESET}"
}

# Main demo sequence
main() {
    clear_screen

    # Scene 1: Title Card
    echo
    echo
    print_banner "cmdai"
    echo
    echo -e "        ${BRIGHT_WHITE}Natural Language → Safe Shell Commands${RESET}"
    echo
    echo -e "              ${DIM}Powered by Local LLMs${RESET}"
    echo
    echo
    pause $PAUSE_LONG

    clear_screen

    # Scene 2: Simple Command
    print_section_header "Scene 1: Simple Query"
    print_prompt
    type_command 'cmdai "list all files in the current directory"'
    pause 0.8
    mock_simple_list
    pause $PAUSE_MEDIUM

    # Scene 3: Complex Command
    print_section_header "Scene 2: Complex Query with Multiple Conditions"
    print_prompt
    type_command 'cmdai "find all PDF files in Downloads larger than 10MB and sort by date"'
    pause 1.2
    mock_complex_find
    pause $PAUSE_MEDIUM

    # Scene 4: Dangerous Command Blocked
    print_section_header "Scene 3: Safety Feature - Dangerous Command Blocked"
    print_prompt
    type_command 'cmdai "delete everything in root directory recursively"'
    pause 1.0
    mock_dangerous_blocked
    pause $PAUSE_LONG

    # Scene 5: Interactive Confirmation
    print_section_header "Scene 4: Interactive Confirmation for Risky Operations"
    print_prompt
    type_command 'cmdai "remove all log files older than 30 days"'
    pause 1.0
    mock_moderate_risk_prompt
    echo -ne "Execute this command? (y/N) "
    pause 1.5
    type_command "n" 80
    pause 0.5
    echo
    mock_cancelled
    pause $PAUSE_MEDIUM

    # Scene 6: Verbose Mode
    print_section_header "Scene 5: Verbose Mode with Debug Information"
    print_prompt
    type_command 'cmdai --verbose "show disk usage sorted by size"'
    pause 1.0
    mock_verbose_output
    pause $PAUSE_MEDIUM

    # Scene 7: JSON Output
    print_section_header "Scene 6: JSON Output for Scripting"
    print_prompt
    type_command 'cmdai --output json "count files in each subdirectory"'
    pause 0.8
    mock_json_output
    echo
    echo -e "${DIM}⚡ Generated in 546ms${RESET}"
    pause $PAUSE_MEDIUM

    # Scene 8: Different Shell
    print_section_header "Scene 7: Cross-Shell Support"
    print_prompt
    type_command 'cmdai --shell fish "list processes using more than 100MB memory"'
    pause 1.0
    mock_fish_shell
    pause $PAUSE_MEDIUM

    # Scene 9: Real-World Workflow
    print_section_header "Scene 8: Real-World Workflow Example"
    print_prompt
    type_command 'cmdai "compress all images in downloads to 80% quality and move to archive folder"'
    pause 1.2
    mock_workflow_response
    echo -ne "Execute this command? (y/N) "
    pause 1.5
    type_command "y" 80
    pause 0.5
    echo
    mock_confirmed
    pause $PAUSE_LONG

    # Scene 10: Feature Summary
    clear_screen
    echo
    echo
    print_banner "cmdai Features"
    echo
    echo -e "  ${GREEN}✓${RESET} Natural language → Shell commands"
    pause 0.3
    echo -e "  ${GREEN}✓${RESET} Comprehensive safety validation"
    pause 0.3
    echo -e "  ${GREEN}✓${RESET} Interactive confirmation for risky operations"
    pause 0.3
    echo -e "  ${GREEN}✓${RESET} Multiple output formats (JSON, YAML, Plain)"
    pause 0.3
    echo -e "  ${GREEN}✓${RESET} Cross-shell support (bash, zsh, fish, sh)"
    pause 0.3
    echo -e "  ${GREEN}✓${RESET} Local LLM inference (Apple Silicon optimized)"
    pause 0.3
    echo -e "  ${GREEN}✓${RESET} Fast generation (<2s on M1 Mac)"
    pause 0.3
    echo -e "  ${GREEN}✓${RESET} POSIX-compliant commands"
    echo
    echo
    echo -e "  ${BRIGHT_BLUE}Get started:${RESET} ${CYAN}cargo install cmdai${RESET}"
    echo -e "  ${BRIGHT_BLUE}Learn more:${RESET}  ${CYAN}github.com/wildcard/cmdai${RESET}"
    echo
    echo
    pause $PAUSE_LONG

    # End
    clear_screen
    echo
    echo -e "${BRIGHT_GREEN}Demo complete!${RESET}"
    echo
    echo "Thank you for watching. Star us on GitHub! ⭐"
    echo
}

# Run the demo
main
