#!/bin/bash
# scripts/check-safety-regressions.sh
# Checks for regressions in safety patterns by comparing against a baseline

set -e

# Baseline file to compare against
BASELINE_FILE="${1:-safety-baseline.json}"

echo "🔍 Safety Pattern Regression Check"
echo "==================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if baseline exists
if [ ! -f "$BASELINE_FILE" ]; then
    echo -e "${YELLOW}⚠️  Warning:${NC} Baseline file not found: $BASELINE_FILE"
    echo "ℹ️  Creating new baseline instead..."
    ./scripts/capture-safety-baseline.sh "$BASELINE_FILE"
    echo "✅ Baseline created. Run again to check for regressions."
    exit 0
fi

# Ensure we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌${NC} Error: Must run from project root (Cargo.toml not found)"
    exit 1
fi

# Ensure the binary is built
if [ ! -f "target/release/caro" ]; then
    echo "⚙️  Building release binary..."
    cargo build --release --quiet
fi

echo "Step 1: Loading baseline"
echo "-----------------------"
echo "📂 Baseline: $BASELINE_FILE"

# Extract baseline values using Python JSON parsing
BASELINE_PATTERN_COUNT=$(python3 -c "import json; print(json.load(open('$BASELINE_FILE'))['patterns']['total_count'])" 2>/dev/null || echo "0")
BASELINE_CRITICAL=$(python3 -c "import json; print(json.load(open('$BASELINE_FILE'))['patterns']['by_risk_level']['critical'])" 2>/dev/null || echo "0")
BASELINE_HIGH=$(python3 -c "import json; print(json.load(open('$BASELINE_FILE'))['patterns']['by_risk_level']['high'])" 2>/dev/null || echo "0")
BASELINE_MEDIUM=$(python3 -c "import json; print(json.load(open('$BASELINE_FILE'))['patterns']['by_risk_level']['medium'])" 2>/dev/null || echo "0")
BASELINE_PASS_COUNT=$(python3 -c "import json; print(json.load(open('$BASELINE_FILE'))['test_results']['static_matcher']['passed'])" 2>/dev/null || echo "0")
BASELINE_TOTAL_COUNT=$(python3 -c "import json; print(json.load(open('$BASELINE_FILE'))['test_results']['static_matcher']['total'])" 2>/dev/null || echo "0")

echo "   Patterns: ${BASELINE_PATTERN_COUNT}"
echo "   Critical: ${BASELINE_CRITICAL}, High: ${BASELINE_HIGH}, Medium: ${BASELINE_MEDIUM}"
echo "   Test Pass: ${BASELINE_PASS_COUNT}/${BASELINE_TOTAL_COUNT}"
echo ""

echo "Step 2: Capturing current state"
echo "-------------------------------"

# Extract current patterns
CURRENT_PATTERN_COUNT=$(grep -c "DangerPattern {" src/safety/patterns.rs || echo "0")
CURRENT_CRITICAL=$(grep "RiskLevel::Critical" src/safety/patterns.rs | wc -l | tr -d ' ')
CURRENT_HIGH=$(grep "RiskLevel::High" src/safety/patterns.rs | wc -l | tr -d ' ')
CURRENT_MEDIUM=$(grep "RiskLevel::Medium" src/safety/patterns.rs | wc -l | tr -d ' ')

echo "   Patterns: ${CURRENT_PATTERN_COUNT}"
echo "   Critical: ${CURRENT_CRITICAL}, High: ${CURRENT_HIGH}, Medium: ${CURRENT_MEDIUM}"

# Run current tests if test-cases.yaml exists
if [ -f ".claude/beta-testing/test-cases.yaml" ] && [ "$BASELINE_TOTAL_COUNT" -gt 0 ]; then
    TEST_OUTPUT=$(./target/release/caro test --backend static --suite .claude/beta-testing/test-cases.yaml 2>&1 || true)
    CURRENT_PASS_COUNT=$(echo "$TEST_OUTPUT" | grep "✅ PASS" | wc -l | tr -d ' ')
    CURRENT_FAIL_COUNT=$(echo "$TEST_OUTPUT" | grep "❌ FAIL" | wc -l | tr -d ' ')
    CURRENT_PASS_COUNT=${CURRENT_PASS_COUNT:-0}
    CURRENT_FAIL_COUNT=${CURRENT_FAIL_COUNT:-0}
    CURRENT_TOTAL_COUNT=$((CURRENT_PASS_COUNT + CURRENT_FAIL_COUNT))
    echo "   Test Pass: ${CURRENT_PASS_COUNT}/${CURRENT_TOTAL_COUNT}"
else
    CURRENT_PASS_COUNT=0
    CURRENT_TOTAL_COUNT=0
    echo "   Test Pass: N/A (no test suite)"
fi
echo ""

echo "Step 3: Checking for regressions"
echo "--------------------------------"

REGRESSION_FOUND=0

# Check 1: Pattern count should not decrease
if [ "$CURRENT_PATTERN_COUNT" -lt "$BASELINE_PATTERN_COUNT" ]; then
    echo -e "${RED}❌ REGRESSION${NC}: Pattern count decreased"
    echo "   Baseline: $BASELINE_PATTERN_COUNT patterns"
    echo "   Current:  $CURRENT_PATTERN_COUNT patterns"
    echo "   Δ: -$((BASELINE_PATTERN_COUNT - CURRENT_PATTERN_COUNT)) patterns"
    REGRESSION_FOUND=1
else
    echo -e "${GREEN}✅${NC} Pattern count: $CURRENT_PATTERN_COUNT (no decrease)"
fi

# Check 2: Critical patterns should not decrease
if [ "$CURRENT_CRITICAL" -lt "$BASELINE_CRITICAL" ]; then
    echo -e "${RED}❌ REGRESSION${NC}: Critical patterns decreased"
    echo "   Baseline: $BASELINE_CRITICAL"
    echo "   Current:  $CURRENT_CRITICAL"
    echo "   Δ: -$((BASELINE_CRITICAL - CURRENT_CRITICAL))"
    REGRESSION_FOUND=1
else
    echo -e "${GREEN}✅${NC} Critical patterns: $CURRENT_CRITICAL (no decrease)"
fi

# Check 3: High patterns should not decrease
if [ "$CURRENT_HIGH" -lt "$BASELINE_HIGH" ]; then
    echo -e "${RED}❌ REGRESSION${NC}: High risk patterns decreased"
    echo "   Baseline: $BASELINE_HIGH"
    echo "   Current:  $CURRENT_HIGH"
    echo "   Δ: -$((BASELINE_HIGH - CURRENT_HIGH))"
    REGRESSION_FOUND=1
else
    echo -e "${GREEN}✅${NC} High risk patterns: $CURRENT_HIGH (no decrease)"
fi

# Check 4: Test pass rate should not decrease (if tests exist)
if [ "$BASELINE_TOTAL_COUNT" -gt 0 ] && [ "$CURRENT_TOTAL_COUNT" -gt 0 ]; then
    if [ "$CURRENT_PASS_COUNT" -lt "$BASELINE_PASS_COUNT" ]; then
        echo -e "${RED}❌ REGRESSION${NC}: Test pass count decreased"
        echo "   Baseline: $BASELINE_PASS_COUNT/$BASELINE_TOTAL_COUNT passed"
        echo "   Current:  $CURRENT_PASS_COUNT/$CURRENT_TOTAL_COUNT passed"
        echo "   Δ: -$((BASELINE_PASS_COUNT - CURRENT_PASS_COUNT)) fewer passes"
        REGRESSION_FOUND=1
    else
        echo -e "${GREEN}✅${NC} Test pass count: $CURRENT_PASS_COUNT/$CURRENT_TOTAL_COUNT (no regression)"
    fi
fi

echo ""

# Check 5: Pattern compilation must pass
echo "Step 4: Checking pattern compilation"
echo "------------------------------------"
if cargo build --lib --quiet 2>&1 | grep -qi "error"; then
    echo -e "${RED}❌ REGRESSION${NC}: Pattern compilation failed!"
    cargo build --lib 2>&1 | grep -i "error" | head -10
    REGRESSION_FOUND=1
else
    echo -e "${GREEN}✅${NC} All patterns compile successfully"
fi

echo ""
echo "========================================="

if [ $REGRESSION_FOUND -eq 1 ]; then
    echo -e "${RED}❌ REGRESSIONS DETECTED${NC}"
    echo ""
    echo "One or more safety pattern regressions were found."
    echo "Please review the changes and fix the regressions before committing."
    echo ""
    echo "Common causes:"
    echo "  - Accidentally deleted patterns"
    echo "  - Downgraded risk levels incorrectly"
    echo "  - Introduced regex syntax errors"
    echo "  - Made patterns too specific (false negatives)"
    echo ""
    exit 1
else
    echo -e "${GREEN}✅ NO REGRESSIONS DETECTED${NC}"
    echo ""
    echo "All safety pattern checks passed!"
    echo ""

    # Show improvements if any
    if [ "$CURRENT_PATTERN_COUNT" -gt "$BASELINE_PATTERN_COUNT" ]; then
        IMPROVEMENT=$((CURRENT_PATTERN_COUNT - BASELINE_PATTERN_COUNT))
        echo -e "${GREEN}📈 Improvement:${NC} +$IMPROVEMENT patterns added"
    fi

    if [ "$BASELINE_TOTAL_COUNT" -gt 0 ] && [ "$CURRENT_PASS_COUNT" -gt "$BASELINE_PASS_COUNT" ]; then
        IMPROVEMENT=$((CURRENT_PASS_COUNT - BASELINE_PASS_COUNT))
        echo -e "${GREEN}📈 Improvement:${NC} +$IMPROVEMENT more tests passing"
    fi

    echo ""
    exit 0
fi
