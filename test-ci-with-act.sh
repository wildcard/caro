#!/bin/bash
# Helper script for running GitHub Actions locally with act
# Usage: ./test-ci-with-act.sh [job-name] [event]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Local GitHub Actions Testing (act)   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Check if act is installed
if ! command -v act &> /dev/null; then
    echo -e "${RED}Error: act is not installed${NC}"
    echo ""
    echo "Install act with:"
    echo "  macOS:   brew install act"
    echo "  Linux:   curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"
    echo "  Manual:  https://github.com/nektos/act#installation"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo -e "${RED}Error: Docker is not running${NC}"
    echo "Please start Docker Desktop or the Docker daemon"
    exit 1
fi

# Parse arguments
JOB_NAME="${1:-}"
EVENT="${2:-push}"

# Show available workflows and jobs
if [ "$JOB_NAME" == "--list" ] || [ "$JOB_NAME" == "-l" ]; then
    echo -e "${GREEN}Available workflows and jobs:${NC}"
    act -l
    exit 0
fi

# Show help
if [ "$JOB_NAME" == "--help" ] || [ "$JOB_NAME" == "-h" ]; then
    cat << EOF
${GREEN}Usage:${NC}
  ./test-ci-with-act.sh [job-name] [event]
  
${GREEN}Arguments:${NC}
  job-name    Specific job to run (optional, runs all if omitted)
  event       GitHub event to simulate (default: push)
              Common events: push, pull_request, release
  
${GREEN}Options:${NC}
  --list, -l  List all available workflows and jobs
  --help, -h  Show this help message
  
${GREEN}Examples:${NC}
  # Run all jobs for push event
  ./test-ci-with-act.sh
  
  # Run specific job
  ./test-ci-with-act.sh test
  
  # Run job with specific event
  ./test-ci-with-act.sh test pull_request
  
  # List all available jobs
  ./test-ci-with-act.sh --list

${GREEN}Common jobs in caro:${NC}
  test        - Run full test suite (fmt, clippy, tests)
  security    - Run cargo audit
  build       - Build release binaries for all platforms
  benchmark   - Run performance benchmarks
  coverage    - Generate code coverage report

${GREEN}Notes:${NC}
  - act uses Docker to emulate GitHub runners
  - First run downloads runner images (~2GB)
  - Some platform-specific tests may not work (macOS MLX tests)
  - Cross-compilation jobs may fail without proper setup
  - Create .secrets file for jobs requiring secrets (see .secrets.example)

${GREEN}Alternative:${NC}
  For native testing without Docker, use:
    ./test-ci-locally.sh

EOF
    exit 0
fi

echo -e "${YELLOW}Configuration:${NC}"
echo "  Event: $EVENT"
echo "  Job: ${JOB_NAME:-all}"
echo ""

# Check for secrets file
if [ -f ".secrets" ]; then
    echo -e "${GREEN}✓ Found .secrets file${NC}"
    SECRETS_FLAG="--secret-file .secrets"
else
    echo -e "${YELLOW}⚠ No .secrets file found (some jobs may fail)${NC}"
    echo "  Create one from .secrets.example if needed"
    SECRETS_FLAG=""
fi
echo ""

# Build act command
ACT_CMD="act $EVENT"

if [ -n "$JOB_NAME" ]; then
    ACT_CMD="$ACT_CMD -j $JOB_NAME"
fi

if [ -n "$SECRETS_FLAG" ]; then
    ACT_CMD="$ACT_CMD $SECRETS_FLAG"
fi

echo -e "${BLUE}Running:${NC} $ACT_CMD"
echo ""
echo -e "${YELLOW}Note: First run will download runner images (~2GB)${NC}"
echo -e "${YELLOW}Note: .actrc configuration will be used automatically${NC}"
echo ""

# Run act with nice error handling
if eval "$ACT_CMD"; then
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║          ✓ All jobs passed!            ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    exit 0
else
    EXIT_CODE=$?
    echo ""
    echo -e "${RED}╔════════════════════════════════════════╗${NC}"
    echo -e "${RED}║          ✗ Some jobs failed            ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Troubleshooting tips:${NC}"
    echo "  - Check Docker has enough resources (CPU/Memory)"
    echo "  - Some jobs require secrets (.secrets file)"
    echo "  - Platform-specific tests may fail (e.g., macOS MLX)"
    echo "  - Cross-compilation needs additional setup"
    echo "  - Try native testing instead: ./test-ci-locally.sh"
    echo "  - Review the output above for specific errors"
    exit $EXIT_CODE
fi
