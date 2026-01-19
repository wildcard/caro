#!/usr/bin/env bash
# Simple evaluation harness runner
# Usage: ./run_eval.sh [dataset_path] [backend]

set -e

CARO_BIN="${CARO_BIN:-../../target/release/caro}"
DATASET="${1:-datasets/correctness/file_operations.json}"
BACKEND="${2:-embedded}"
OUTPUT_DIR="./eval_results"

mkdir -p "$OUTPUT_DIR"

echo "===================================================================="
echo "Caro Evaluation Harness"
echo "===================================================================="
echo "Dataset: $DATASET"
echo "Backend: $BACKEND"
echo "Caro Binary: $CARO_BIN"
echo "===================================================================="
echo

# Check if caro binary exists
if [ ! -f "$CARO_BIN" ]; then
    echo "ERROR: Caro binary not found at $CARO_BIN"
    echo "Build it with: cargo build --release"
    exit 1
fi

# Check if dataset exists
if [ ! -f "$DATASET" ]; then
    echo "ERROR: Dataset not found at $DATASET"
    exit 1
fi

# Parse the dataset and run tests
TOTAL=0
PASSED=0
FAILED=0

echo "Running tests..."
echo

# Read test cases from JSON
while IFS= read -r line; do
    # Skip if not a test case
    if ! echo "$line" | jq -e '.prompt' >/dev/null 2>&1; then
        continue
    fi

    PROMPT=$(echo "$line" | jq -r '.prompt')
    EXPECTED=$(echo "$line" | jq -r '.expected_command')
    TEST_ID=$(echo "$line" | jq -r '.id')
    CATEGORY=$(echo "$line" | jq -r '.category')

    TOTAL=$((TOTAL + 1))

    # Run caro
    ACTUAL=$("$CARO_BIN" --backend "$BACKEND" --output json "$PROMPT" 2>/dev/null | jq -r '.generated_command' || echo "ERROR")

    # Compare (simple exact match for now)
    if [ "$ACTUAL" = "$EXPECTED" ]; then
        PASSED=$((PASSED + 1))
        echo "✓ $TEST_ID: PASS"
    else
        FAILED=$((FAILED + 1))
        echo "✗ $TEST_ID: FAIL"
        echo "  Prompt:   $PROMPT"
        echo "  Expected: $EXPECTED"
        echo "  Actual:   $ACTUAL"
        echo
    fi
done < <(jq -c '.test_cases[]' "$DATASET")

echo
echo "===================================================================="
echo "Results Summary"
echo "===================================================================="
echo "Total:  $TOTAL"
echo "Passed: $PASSED"
echo "Failed: $FAILED"

if [ $TOTAL -gt 0 ]; then
    PASS_RATE=$(echo "scale=2; $PASSED * 100 / $TOTAL" | bc)
    echo "Pass Rate: $PASS_RATE%"
fi

echo "===================================================================="

# Exit with failure if any tests failed
[ $FAILED -eq 0 ]
