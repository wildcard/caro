#!/bin/bash
# scripts/verify-week-1-completion.sh
# Comprehensive verification of Week 1 deliverables

set -e

echo "=== Week 1 Deliverables Verification ==="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Counter for pass/fail
PASS=0
FAIL=0

check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✅${NC} $2"
        ((PASS++))
    else
        echo -e "${RED}❌${NC} $2 (missing: $1)"
        ((FAIL++))
    fi
}

check_executable() {
    if [ -x "$1" ]; then
        echo -e "${GREEN}✅${NC} $2"
        ((PASS++))
    else
        echo -e "${RED}❌${NC} $2 (not executable: $1)"
        ((FAIL++))
    fi
}

check_content() {
    if grep -q "$2" "$1" 2>/dev/null; then
        echo -e "${GREEN}✅${NC} $3"
        ((PASS++))
    else
        echo -e "${RED}❌${NC} $3 (pattern not found in $1)"
        ((FAIL++))
    fi
}

echo "Phase 1: Quick Wins"
echo "-------------------"
check_file ".claude/hookify.validate-patterns-before-commit.local.md" "Hookify rule"
check_executable ".git/hooks/pre-commit" "Git pre-commit hook"
check_content "CONTRIBUTING.md" "Safety Pattern Development" "TDD workflow documentation"
check_file ".claude/workflows/add-safety-pattern.md" "Pattern contribution workflow"
check_file ".github/workflows/safety-validation.yml" "CI/CD pipeline"
check_executable "scripts/capture-safety-baseline.sh" "Baseline capture script"
check_executable "scripts/check-safety-regressions.sh" "Regression checker script"

echo ""
echo "Phase 2: Skills"
echo "---------------"
check_file ".claude/skills/safety-pattern-developer/SKILL.md" "Pattern developer skill"
check_file ".claude/skills/safety-pattern-developer/README.md" "Pattern developer README"
check_file ".claude/skills/safety-pattern-developer/examples/example-rm-rf-parent.md" "Pattern developer example 1"
check_file ".claude/skills/safety-pattern-developer/examples/example-dd-reverse.md" "Pattern developer example 2"

check_file ".claude/skills/safety-pattern-auditor/SKILL.md" "Pattern auditor skill"
check_file ".claude/skills/safety-pattern-auditor/README.md" "Pattern auditor README"
check_file ".claude/skills/safety-pattern-auditor/examples/audit-report-template.md" "Audit report template"
check_file ".claude/skills/safety-pattern-auditor/QUICK-REFERENCE.md" "Auditor quick reference"

check_file ".claude/skills/backend-safety-integrator/SKILL.md" "Backend integrator skill"
check_file ".claude/skills/backend-safety-integrator/README.md" "Backend integrator README"
check_file ".claude/skills/backend-safety-integrator/examples/simple-backend-integration.md" "Backend example 1"
check_file ".claude/skills/backend-safety-integrator/examples/async-backend-integration.md" "Backend example 2"

echo ""
echo "Gap Analyzer"
echo "------------"
check_executable "scripts/analyze-pattern-gaps.py" "Gap analyzer CLI"
check_file "scripts/pattern_analyzer/__init__.py" "Pattern analyzer module"
check_file "scripts/pattern_analyzer/parser.py" "Parser module"
check_file "scripts/pattern_analyzer/argument_detector.py" "Argument order detector"
check_file "scripts/pattern_analyzer/path_detector.py" "Path variant detector"
check_file "scripts/pattern_analyzer/wildcard_detector.py" "Wildcard detector"
check_file "scripts/pattern_analyzer/platform_detector.py" "Platform detector"
check_file "scripts/tests/test_pattern_gap_analyzer.py" "Gap analyzer tests"

echo ""
echo "Documentation"
echo "-------------"
check_file ".claude/recommendations/README.md" "Implementation roadmap"
check_file ".claude/recommendations/lifecycle-improvements.md" "Lifecycle improvements"
check_file ".claude/recommendations/skill-improvements.md" "Skill improvements"
check_file ".claude/recommendations/agent-improvements.md" "Agent improvements"
check_file ".claude/recommendations/one-week-implementation-plan.md" "One week plan"
check_file ".claude/recommendations/week-1-completion-report.md" "Week 1 completion report"

echo ""
echo "=== Summary ==="
echo -e "Passed: ${GREEN}${PASS}${NC}"
echo -e "Failed: ${RED}${FAIL}${NC}"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ All Week 1 deliverables present!${NC}"
    echo ""

    # Run additional functional tests
    echo "=== Running Functional Tests ==="
    echo ""

    echo "1. Testing Git Pre-Commit Hook..."
    if .git/hooks/pre-commit 2>&1 | grep -q "pre-commit safety pattern validation"; then
        echo -e "${GREEN}✅${NC} Git hook executes"
    else
        echo -e "${RED}❌${NC} Git hook doesn't execute properly"
        ((FAIL++))
    fi

    echo ""
    echo "2. Testing Gap Analyzer..."
    if [ -x scripts/analyze-pattern-gaps.py ]; then
        if scripts/analyze-pattern-gaps.py src/safety/patterns.rs > /tmp/gap-test.md 2>&1; then
            echo -e "${GREEN}✅${NC} Gap analyzer runs successfully"
            echo "   Generated report: /tmp/gap-test.md"
        else
            echo -e "${RED}❌${NC} Gap analyzer failed to run"
            ((FAIL++))
        fi
    fi

    echo ""
    echo "3. Testing Pattern Compilation..."
    if cargo build --lib --quiet 2>&1; then
        echo -e "${GREEN}✅${NC} Patterns compile successfully"
    else
        echo -e "${RED}❌${NC} Pattern compilation failed"
        ((FAIL++))
    fi

    echo ""
    if [ $FAIL -eq 0 ]; then
        echo -e "${GREEN}🎉 Week 1 Implementation COMPLETE and VERIFIED!${NC}"
        exit 0
    else
        echo -e "${RED}⚠️  Some functional tests failed${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Missing deliverables detected!${NC}"
    echo "Please complete missing items before marking Week 1 as done."
    exit 1
fi
