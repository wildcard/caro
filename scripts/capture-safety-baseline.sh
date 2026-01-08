#!/bin/bash
# scripts/capture-safety-baseline.sh
# Captures current safety pattern state as a baseline for regression detection

set -e

# Default output file
OUTPUT_FILE="${1:-safety-baseline.json}"

echo "📸 Capturing Safety Pattern Baseline"
echo "===================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Ensure we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must run from project root (Cargo.toml not found)"
    exit 1
fi

# Ensure the binary is built
if [ ! -f "target/release/caro" ]; then
    echo "⚙️  Building release binary..."
    cargo build --release --quiet
fi

echo "Step 1: Extracting pattern metadata"
echo "-----------------------------------"

# Extract patterns from patterns.rs
PATTERN_COUNT=$(grep -c "DangerPattern {" src/safety/patterns.rs || echo "0")
echo -e "${GREEN}✅${NC} Found ${PATTERN_COUNT} patterns"

# Count by risk level
CRITICAL_COUNT=$(grep "RiskLevel::Critical" src/safety/patterns.rs | wc -l | tr -d ' ')
HIGH_COUNT=$(grep "RiskLevel::High" src/safety/patterns.rs | wc -l | tr -d ' ')
MEDIUM_COUNT=$(grep "RiskLevel::Medium" src/safety/patterns.rs | wc -l | tr -d ' ')

echo "   - Critical: ${CRITICAL_COUNT}"
echo "   - High: ${HIGH_COUNT}"
echo "   - Medium: ${MEDIUM_COUNT}"
echo ""

echo "Step 2: Running static matcher test suite"
echo "-----------------------------------------"

# Run tests if test-cases.yaml exists
if [ -f ".claude/beta-testing/test-cases.yaml" ]; then
    # Run tests and capture results
    TEST_OUTPUT=$(./target/release/caro test --backend static --suite .claude/beta-testing/test-cases.yaml 2>&1 || true)

    # Parse pass/fail counts
    PASS_COUNT=$(echo "$TEST_OUTPUT" | grep "✅ PASS" | wc -l | tr -d ' ')
    FAIL_COUNT=$(echo "$TEST_OUTPUT" | grep "❌ FAIL" | wc -l | tr -d ' ')
    # Ensure we have numbers
    PASS_COUNT=${PASS_COUNT:-0}
    FAIL_COUNT=${FAIL_COUNT:-0}
    TOTAL_COUNT=$((PASS_COUNT + FAIL_COUNT))

    if [ $TOTAL_COUNT -gt 0 ]; then
        PASS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASS_COUNT/$TOTAL_COUNT)*100}")
    else
        PASS_RATE="0.0"
    fi

    echo -e "${GREEN}✅${NC} Static matcher tests: ${PASS_COUNT}/${TOTAL_COUNT} passed (${PASS_RATE}%)"
else
    echo "ℹ️  No test-cases.yaml found - skipping test suite"
    PASS_COUNT=0
    FAIL_COUNT=0
    TOTAL_COUNT=0
    PASS_RATE="N/A"
fi
echo ""

echo "Step 3: Generating baseline JSON"
echo "--------------------------------"

# Generate timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Generate git info
GIT_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

# Create JSON baseline
cat > "$OUTPUT_FILE" <<EOF
{
  "metadata": {
    "captured_at": "$TIMESTAMP",
    "git_commit": "$GIT_COMMIT",
    "git_branch": "$GIT_BRANCH",
    "script_version": "1.0.0"
  },
  "patterns": {
    "total_count": $PATTERN_COUNT,
    "by_risk_level": {
      "critical": $CRITICAL_COUNT,
      "high": $HIGH_COUNT,
      "medium": $MEDIUM_COUNT
    }
  },
  "test_results": {
    "static_matcher": {
      "total": $TOTAL_COUNT,
      "passed": $PASS_COUNT,
      "failed": $FAIL_COUNT,
      "pass_rate": "$PASS_RATE"
    }
  }
}
EOF

echo -e "${GREEN}✅${NC} Baseline captured to: $OUTPUT_FILE"
echo ""

# Pretty print the baseline
echo "Baseline Summary:"
echo "================="
cat "$OUTPUT_FILE" | python3 -m json.tool 2>/dev/null || cat "$OUTPUT_FILE"
echo ""

echo -e "${BLUE}🎉${NC} Baseline capture complete!"
echo ""
echo "Usage:"
echo "  - Store this baseline in git or CI artifacts"
echo "  - Compare future runs with: ./scripts/check-safety-regressions.sh $OUTPUT_FILE"
