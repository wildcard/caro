#!/usr/bin/env bash
# Evaluation harness v2 - simpler and more robust
set -euo pipefail

CARO="${CARO_BIN:-../../target/release/caro}"
DATASET="${1:-datasets/correctness/file_operations.json}"
BACKEND="${2:-embedded}"

echo "Caro Evaluation Harness v2"
echo "Dataset: $DATASET"
echo "Backend: $BACKEND"
echo "================================"
echo

TOTAL=0
PASSED=0
FAILED=0
RESULTS_FILE="/tmp/caro_eval_results_$$.txt"

# Extract test cases and process each
jq -c '.test_cases[]' "$DATASET" | while read -r test_case; do
    TOTAL=$((TOTAL + 1))

    ID=$(echo "$test_case" | jq -r '.id')
    PROMPT=$(echo "$test_case" | jq -r '.prompt')
    EXPECTED=$(echo "$test_case" | jq -r '.expected_command')

    # Run caro and get generated command
    ACTUAL=$("$CARO" --backend "$BACKEND" --output json "$PROMPT" 2>&1 | jq -r '.generated_command' 2>/dev/null || echo "ERROR")

    # Simple string comparison
    if [ "$ACTUAL" = "$EXPECTED" ]; then
        echo "✓ $ID: PASS"
        PASSED=$((PASSED + 1))
    else
        echo "✗ $ID: FAIL"
        echo "  Expected: $EXPECTED"
        echo "  Actual:   $ACTUAL"
        FAILED=$((FAILED + 1))
    fi

    # Save result
    echo "$ID|$PROMPT|$EXPECTED|$ACTUAL" >> "$RESULTS_FILE"
done

echo
echo "================================"
echo "Summary:"
echo "  Total:  $TOTAL"
echo "  Passed: $PASSED"
echo "  Failed: $FAILED"
if [ $TOTAL -gt 0 ]; then
    PASS_RATE=$(echo "scale=1; $PASSED * 100 / $TOTAL" | bc)
    echo "  Pass Rate: $PASS_RATE%"
fi
echo "================================"
echo
echo "Results saved to: $RESULTS_FILE"
