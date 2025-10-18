#!/bin/bash

# TDD Watcher for cmdai - Continuous test runner
# Watches for file changes and automatically runs tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Test file
TEST_FILE="tests/test_basic_command_generation.rs"
TEST_BINARY="/tmp/test_basic_tdd"

# Function to run tests
run_tests() {
    echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} Running tests..."
    
    # Compile the test
    if rustc --test "$TEST_FILE" -o "$TEST_BINARY" 2>/dev/null; then
        # Run the test and capture output
        if output=$("$TEST_BINARY" 2>&1); then
            # Tests passed
            echo -e "${GREEN}✅ All tests passing!${NC}"
            echo "$output" | grep -E "test result:|running"
        else
            # Tests failed
            echo -e "${RED}❌ Tests failing!${NC}"
            echo "$output"
        fi
    else
        # Compilation failed
        echo -e "${RED}❌ Compilation error!${NC}"
        rustc --test "$TEST_FILE" -o "$TEST_BINARY" 2>&1 | head -20
    fi
    echo ""
}

# Function to show TDD status
show_tdd_status() {
    echo -e "${MAGENTA}=== TDD Continuous Test Watcher ===${NC}"
    echo -e "${YELLOW}Watching:${NC} $TEST_FILE"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""
}

# Initial run
clear
show_tdd_status
run_tests

# Watch for changes using a simple loop
echo -e "${BLUE}Watching for changes...${NC}"

# Store the last modification time
last_mod=$(stat -c %Y "$TEST_FILE" 2>/dev/null || stat -f %m "$TEST_FILE" 2>/dev/null)

while true; do
    sleep 1
    
    # Check if file has been modified
    current_mod=$(stat -c %Y "$TEST_FILE" 2>/dev/null || stat -f %m "$TEST_FILE" 2>/dev/null)
    
    if [ "$current_mod" != "$last_mod" ]; then
        clear
        show_tdd_status
        echo -e "${YELLOW}File changed, running tests...${NC}"
        run_tests
        last_mod=$current_mod
        echo -e "${BLUE}Watching for changes...${NC}"
    fi
done