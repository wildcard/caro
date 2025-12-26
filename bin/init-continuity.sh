#!/bin/bash
# Initialize Continuous-Claude session continuity for caro
# This script creates necessary directories and verifies hooks are in place

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Initializing Continuous-Claude session continuity for caro..."
echo ""

# Create thoughts directory structure
echo "Creating thoughts directory structure..."
mkdir -p "$PROJECT_DIR/thoughts/ledgers"
mkdir -p "$PROJECT_DIR/thoughts/shared/handoffs"
mkdir -p "$PROJECT_DIR/thoughts/shared/plans"
mkdir -p "$PROJECT_DIR/thoughts/shared/research"

# Create .gitkeep files to preserve directory structure
touch "$PROJECT_DIR/thoughts/.gitkeep"
touch "$PROJECT_DIR/thoughts/ledgers/.gitkeep"
touch "$PROJECT_DIR/thoughts/shared/.gitkeep"
touch "$PROJECT_DIR/thoughts/shared/handoffs/.gitkeep"
touch "$PROJECT_DIR/thoughts/shared/plans/.gitkeep"
touch "$PROJECT_DIR/thoughts/shared/research/.gitkeep"

echo "  Created thoughts/ledgers/"
echo "  Created thoughts/shared/handoffs/"
echo "  Created thoughts/shared/plans/"
echo "  Created thoughts/shared/research/"

# Verify hooks are in place
echo ""
echo "Verifying hooks..."

HOOKS_DIR="$PROJECT_DIR/.claude/hooks"
REQUIRED_HOOKS=(
    "session-start-continuity.sh"
    "pre-compact-continuity.sh"
    "session-end-cleanup.sh"
    "subagent-stop-continuity.sh"
    "handoff-index.sh"
)

MISSING_HOOKS=()
for hook in "${REQUIRED_HOOKS[@]}"; do
    if [[ -f "$HOOKS_DIR/$hook" ]]; then
        echo "  Found: $hook"
    else
        MISSING_HOOKS+=("$hook")
        echo "  MISSING: $hook"
    fi
done

# Check dist directory
if [[ -d "$HOOKS_DIR/dist" ]]; then
    echo "  Found: dist/"
else
    echo "  MISSING: dist/ (compiled hook modules)"
    MISSING_HOOKS+=("dist/")
fi

# Verify skills
echo ""
echo "Verifying skills..."

SKILLS_DIR="$PROJECT_DIR/.claude/skills"
REQUIRED_SKILLS=(
    "continuity_ledger"
    "create_handoff"
    "resume_handoff"
    "onboard"
)

MISSING_SKILLS=()
for skill in "${REQUIRED_SKILLS[@]}"; do
    if [[ -d "$SKILLS_DIR/$skill" ]]; then
        echo "  Found: $skill/"
    else
        MISSING_SKILLS+=("$skill")
        echo "  MISSING: $skill/"
    fi
done

# Verify settings.json
echo ""
echo "Verifying settings.json..."
if [[ -f "$PROJECT_DIR/.claude/settings.json" ]]; then
    echo "  Found: .claude/settings.json"
else
    echo "  MISSING: .claude/settings.json"
fi

# Summary
echo ""
echo "============================================"
if [[ ${#MISSING_HOOKS[@]} -eq 0 && ${#MISSING_SKILLS[@]} -eq 0 ]]; then
    echo "Continuous-Claude initialization complete!"
    echo ""
    echo "Available skills:"
    echo "  /continuity_ledger - Save state before /clear"
    echo "  /create_handoff    - Create end-of-session handoff"
    echo "  /resume_handoff    - Resume from handoff document"
    echo "  /onboard           - Analyze codebase and create initial ledger"
    echo ""
    echo "See CLAUDE.md for detailed usage instructions."
else
    echo "WARNING: Some components are missing!"
    echo ""
    if [[ ${#MISSING_HOOKS[@]} -gt 0 ]]; then
        echo "Missing hooks: ${MISSING_HOOKS[*]}"
    fi
    if [[ ${#MISSING_SKILLS[@]} -gt 0 ]]; then
        echo "Missing skills: ${MISSING_SKILLS[*]}"
    fi
    echo ""
    echo "Run the following to reinstall from Continuous-Claude:"
    echo "  git clone https://github.com/parcadei/Continuous-Claude.git .continuous-claude-tmp"
    echo "  # Then copy missing components from .continuous-claude-tmp/.claude/"
fi
echo "============================================"
