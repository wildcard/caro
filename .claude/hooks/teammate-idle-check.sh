#!/bin/bash
# TeammateIdle hook - Enforce completion standards for Agent Team teammates
#
# Exit codes:
#   0 = Allow teammate to go idle (work is complete)
#   2 = Send feedback and keep teammate working (work incomplete)
#
# This hook runs when a teammate is about to go idle.
# It checks whether the teammate produced deliverables.

# The hook receives context via environment variables:
# CLAUDE_TEAMMATE_NAME - Name of the teammate
# CLAUDE_TEAM_NAME - Name of the team

# For Phase 1 (read-only teams), this is a no-op.
# Uncomment the checks below when entering Phase 2.

# --- Phase 2 checks (uncomment when ready) ---
# Check if teammate mentioned PASS/FAIL verdict (for release reviews)
# if echo "$CLAUDE_TEAMMATE_OUTPUT" | grep -qiE '(PASS|FAIL|verdict|finding)'; then
#   exit 0
# else
#   echo "Please provide a clear PASS/FAIL verdict or list your findings before finishing."
#   exit 2
# fi

exit 0
