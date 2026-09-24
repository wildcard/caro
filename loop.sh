#!/usr/bin/env bash
# loop.sh - Ralph outer loop for caro
#
# Based on Geoffrey Huntley's Ralph methodology:
# "Ralph isn't just a loop that codes. It's a funnel with 3 Phases, 2 Prompts, and 1 Loop."
#
# Usage:
#   ./loop.sh plan    # Planning mode - generate/update implementation plan
#   ./loop.sh build   # Building mode - implement tasks iteratively
#   ./loop.sh         # Default: building mode
#
# Environment:
#   RALPH_MAX_ITERATIONS    - Maximum iterations (default: 20; 0 = unlimited, logged as a warning)
#   RALPH_ITERATION_TIMEOUT - Max runtime of one iteration, as a `timeout` duration (default: 30m)
#   RALPH_PAUSE_SECONDS     - Pause between iterations (default: 2)
#   RALPH_RETRY_SECONDS     - Pause after a failed iteration (default: 5)
#   RALPH_LOG_FILE          - Log file path (default: ralph.log)

set -euo pipefail

# Configuration
MODE="${1:-build}"
PROMPT_FILE="PROMPT_${MODE}.md"
MAX_ITERATIONS="${RALPH_MAX_ITERATIONS:-20}"  # 0 = unlimited (explicit opt-in only)
ITERATION_TIMEOUT="${RALPH_ITERATION_TIMEOUT:-30m}"
PAUSE_SECONDS="${RALPH_PAUSE_SECONDS:-2}"
RETRY_SECONDS="${RALPH_RETRY_SECONDS:-5}"
LOG_FILE="${RALPH_LOG_FILE:-ralph.log}"
ITERATION=0

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"

    case "$level" in
        INFO)  echo -e "${BLUE}[$timestamp]${NC} $message" ;;
        WARN)  echo -e "${YELLOW}[$timestamp]${NC} $message" ;;
        ERROR) echo -e "${RED}[$timestamp]${NC} $message" ;;
        OK)    echo -e "${GREEN}[$timestamp]${NC} $message" ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    if [[ ! -f "$PROMPT_FILE" ]]; then
        log ERROR "Prompt file not found: $PROMPT_FILE"
        log INFO "Available modes: plan, build"
        log INFO "Create $PROMPT_FILE or use an existing mode"
        exit 1
    fi

    if ! command -v claude &> /dev/null; then
        log ERROR "Claude CLI not found. Install it first."
        exit 1
    fi

    # At most 9 digits: larger values would overflow bash arithmetic (wrapping
    # to a negative number, i.e. no cap).
    if ! [[ "$MAX_ITERATIONS" =~ ^[0-9]{1,9}$ ]]; then
        log ERROR "RALPH_MAX_ITERATIONS must be an integer from 0 to 999999999 (got: $MAX_ITERATIONS)"
        exit 1
    fi
    # Force base 10: bash would parse a leading-zero value such as 08 as octal.
    MAX_ITERATIONS=$((10#$MAX_ITERATIONS))
    if [[ "$MAX_ITERATIONS" -eq 0 ]]; then
        log WARN "RALPH_MAX_ITERATIONS=0: running with NO iteration cap"
    fi

    # Per-iteration time limit (GNU timeout, or gtimeout from coreutils on macOS)
    TIMEOUT_CMD=()
    if command -v timeout &> /dev/null; then
        TIMEOUT_CMD=(timeout --signal=TERM --kill-after=30s "$ITERATION_TIMEOUT")
    elif command -v gtimeout &> /dev/null; then
        TIMEOUT_CMD=(gtimeout --signal=TERM --kill-after=30s "$ITERATION_TIMEOUT")
    else
        log WARN "timeout/gtimeout not found: iterations run WITHOUT a time limit"
    fi

    # Ensure we're in a git repo
    if ! git rev-parse --git-dir &> /dev/null; then
        log ERROR "Not in a git repository"
        exit 1
    fi
}

# Display banner
show_banner() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║${NC}     ${BLUE}Ralph Loop for Caro${NC}               ${GREEN}║${NC}"
    echo -e "${GREEN}║${NC}     Mode: ${YELLOW}$MODE${NC}                        ${GREEN}║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    echo ""
    log INFO "Starting Ralph loop in $MODE mode"
    log INFO "Press Ctrl+C to stop"
    echo ""
}

# Run a single iteration
run_iteration() {
    ITERATION=$((ITERATION + 1))

    log INFO "=== Iteration $ITERATION starting ==="

    # Record start time
    local start_time
    start_time=$(date +%s)

    # Feed prompt to Claude
    # Claude will study the codebase, select a task, implement it, and exit
    if ${TIMEOUT_CMD[@]+"${TIMEOUT_CMD[@]}"} claude --print < "$PROMPT_FILE" 2>&1 | tee -a "$LOG_FILE"; then
        local end_time
        end_time=$(date +%s)
        local duration=$((end_time - start_time))
        log OK "Iteration $ITERATION completed in ${duration}s"
    else
        local exit_code=$?
        log WARN "Claude exited with code $exit_code"

        # Check for common issues
        if [[ $exit_code -eq 124 || $exit_code -eq 137 ]]; then
            log WARN "Iteration $ITERATION timed out after $ITERATION_TIMEOUT"
        fi
        if [[ $exit_code -eq 130 ]]; then
            log INFO "Interrupted by user"
            exit 0
        fi

        log INFO "Waiting ${RETRY_SECONDS}s before retry..."
        sleep "$RETRY_SECONDS"
    fi

    log INFO "=== Iteration $ITERATION finished ==="
    echo ""
}

# Main loop
main() {
    check_prerequisites
    show_banner

    # Create log file if it doesn't exist
    touch "$LOG_FILE"
    log INFO "Logging to $LOG_FILE"

    # Show current git status
    local branch
    branch=$(git branch --show-current)
    log INFO "Working on branch: $branch"

    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        log WARN "Uncommitted changes detected"
    fi

    # Run the loop
    while true; do
        run_iteration

        # Check iteration limit
        if [[ $MAX_ITERATIONS -gt 0 ]] && [[ $ITERATION -ge $MAX_ITERATIONS ]]; then
            log INFO "Reached maximum iterations ($MAX_ITERATIONS)"
            break
        fi

        # Brief pause to allow interrupt and prevent runaway
        sleep "$PAUSE_SECONDS"
    done

    log OK "Ralph loop completed after $ITERATION iterations"
}

# Handle Ctrl+C gracefully
trap 'echo ""; log INFO "Interrupted by user after $ITERATION iterations"; exit 0' INT TERM

# Run main
main
