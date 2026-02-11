#!/bin/bash
# TaskCompleted hook - Quality gate for Agent Team task completion
#
# Exit codes:
#   0 = Allow task to be marked complete
#   2 = Prevent completion, send feedback to teammate
#
# This hook runs when a task is being marked complete in the shared task list.

# For Phase 1 (read-only teams), this is a no-op.
# Uncomment the checks below when entering Phase 2.

# --- Phase 2 checks (uncomment when ready) ---
# Verify tests pass before allowing implementation tasks to complete
# if echo "$CLAUDE_TASK_DESCRIPTION" | grep -qiE '(implement|build|create|add)'; then
#   if ! make test 2>/dev/null; then
#     echo "Tests must pass before marking implementation task as complete. Run 'make check' and fix failures."
#     exit 2
#   fi
# fi

exit 0
