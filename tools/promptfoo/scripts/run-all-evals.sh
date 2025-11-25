#!/bin/bash
#
# Run all promptfoo evaluations for cmdai
#
# This script runs all evaluation configurations and generates
# a comprehensive report comparing different approaches.

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPTFOO_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$(dirname "$PROMPTFOO_DIR")")"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  cmdai Promptfoo Evaluation Runner${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Change to promptfoo directory
cd "$PROMPTFOO_DIR"

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}⚠ Node modules not found. Running npm install...${NC}"
    npm install
    echo ""
fi

# Check if cmdai binary exists
CMDAI_BINARY="$ROOT_DIR/target/release/cmdai"
if [ ! -f "$CMDAI_BINARY" ]; then
    echo -e "${RED}✗ cmdai binary not found at: $CMDAI_BINARY${NC}"
    echo -e "${YELLOW}  Please build cmdai first:${NC}"
    echo -e "${YELLOW}    cd $ROOT_DIR && cargo build --release${NC}"
    exit 1
fi

echo -e "${GREEN}✓ cmdai binary found${NC}"
echo ""

# Parse command line arguments
RUN_CMDAI=true
RUN_PROMPTS=true
RUN_PROVIDERS=true
VIEW_RESULTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --only-cmdai)
            RUN_PROMPTS=false
            RUN_PROVIDERS=false
            shift
            ;;
        --only-prompts)
            RUN_CMDAI=false
            RUN_PROVIDERS=false
            shift
            ;;
        --only-providers)
            RUN_CMDAI=false
            RUN_PROMPTS=false
            shift
            ;;
        --view)
            VIEW_RESULTS=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --only-cmdai      Run only cmdai binary evaluation"
            echo "  --only-prompts    Run only prompt variations evaluation"
            echo "  --only-providers  Run only provider comparison evaluation"
            echo "  --view            Open web UI after evaluation"
            echo "  --help, -h        Show this help message"
            echo ""
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Function to run an evaluation
run_eval() {
    local config=$1
    local name=$2

    echo -e "${BLUE}──────────────────────────────────────────────────${NC}"
    echo -e "${BLUE}Running: $name${NC}"
    echo -e "${BLUE}Config: $config${NC}"
    echo -e "${BLUE}──────────────────────────────────────────────────${NC}"
    echo ""

    if npx promptfoo eval -c "$config"; then
        echo -e "${GREEN}✓ $name completed successfully${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ $name failed${NC}"
        echo ""
        return 1
    fi
}

# Track results
SUCCESS_COUNT=0
FAIL_COUNT=0
TOTAL_COUNT=0

# Run evaluations
if [ "$RUN_CMDAI" = true ]; then
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    if run_eval "configs/cmdai-binary.yaml" "cmdai Binary Evaluation"; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
fi

if [ "$RUN_PROMPTS" = true ]; then
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    if run_eval "configs/prompt-variations.yaml" "Prompt Variations Evaluation"; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
fi

if [ "$RUN_PROVIDERS" = true ]; then
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    if run_eval "configs/providers-comparison.yaml" "Provider Comparison Evaluation"; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
fi

# Print summary
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Evaluation Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""
echo -e "Total evaluations: $TOTAL_COUNT"
echo -e "${GREEN}Successful: $SUCCESS_COUNT${NC}"
if [ $FAIL_COUNT -gt 0 ]; then
    echo -e "${RED}Failed: $FAIL_COUNT${NC}"
fi
echo ""

# List output files
echo -e "${BLUE}Output files:${NC}"
if [ -d "outputs" ]; then
    ls -lh outputs/*.json 2>/dev/null || echo "  (no JSON results yet)"
fi
echo ""

# Open web UI if requested
if [ "$VIEW_RESULTS" = true ]; then
    echo -e "${BLUE}Opening promptfoo web UI...${NC}"
    npx promptfoo view
else
    echo -e "${YELLOW}💡 Tip: Run 'npm run view' to see results in web UI${NC}"
    echo -e "${YELLOW}💡 Or: npx promptfoo view latest${NC}"
fi

echo ""

# Exit with appropriate code
if [ $FAIL_COUNT -gt 0 ]; then
    exit 1
else
    exit 0
fi
