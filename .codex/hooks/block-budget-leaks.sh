#!/usr/bin/env bash
# PreToolUse hook: block git commits that include dollar amounts or
# quantitative cost data under .beads/ or .claude/memory/ paths.
#
# Caro is the only PUBLIC repo with beads + memory tracking. The budget-watch
# system writes findings into beads/memory only after passing through the
# redact.py filter — but if a human or another agent slips raw figures into
# those paths, this hook is the last line of defense before they reach the
# public mirror.
#
# Companion to ~/.claude/projects/-Users-kobik-private-workspace-caro/memory/
#   feedback_no_finance_in_public_repos.md

set -euo pipefail

TOOL_NAME="${CLAUDE_TOOL_NAME:-}"
if [[ "$TOOL_NAME" != "Bash" ]]; then
  exit 0
fi

COMMAND="${CLAUDE_TOOL_PARAMS_COMMAND:-}"
if [[ ! "$COMMAND" =~ git[[:space:]]+commit ]]; then
  exit 0
fi

# Only act inside caro repo (let other repos use their own rules).
ORIGIN_URL=$(git config --get remote.origin.url 2>/dev/null || echo "")
if [[ ! "$ORIGIN_URL" =~ wildcard/caro ]]; then
  exit 0
fi

# Patterns we refuse to commit into sensitive paths.
# - dollar amounts:    $42, $1.5, $1,234
# - explicit USD:      42 USD
# - minute counts:     1234 minutes
# - byte counts:       5 GB / 200 MB
SENSITIVE_RE='\$[0-9]+([.,][0-9]+)?|\b[0-9]+[.,]?[0-9]*[[:space:]]*USD\b|\b[0-9]+[[:space:]]*(minutes?|mins?)\b|\b[0-9]+[[:space:]]*(GB|MB|TB|KB)\b'

# Look only at staged files in protected paths.
PROTECTED_FILES=$(git diff --cached --name-only --diff-filter=ACMRT 2>/dev/null \
  | grep -E '^(\.beads/|\.claude/memory/)' || true)

if [[ -z "$PROTECTED_FILES" ]]; then
  exit 0
fi

# Scan staged diff (added lines only) for sensitive patterns.
LEAK=$(git diff --cached --unified=0 -- $PROTECTED_FILES 2>/dev/null \
  | grep -E '^\+' \
  | grep -v '^\+\+\+' \
  | grep -E -i "$SENSITIVE_RE" || true)

if [[ -n "$LEAK" ]]; then
  cat >&2 <<EOF

🚫 BLOCKED: financial data detected in protected paths

The following added lines under .beads/ or .claude/memory/ contain dollar
amounts, USD totals, minute counts, or byte counts — caro is a PUBLIC repo
and this data must stay in ~/.openclaw/workspace/budget/ only.

$LEAK

What to do:
1. Run the content through the redaction filter:
   python3 ~/.openclaw/workspace/budget/bin/redact.py < <(git diff --cached -- $PROTECTED_FILES)
2. Update the staged file(s) with redacted versions.
3. Re-stage and re-commit.

If this is a false positive (e.g. a release-version reference like "v1.3.0"),
unstage the affected line and commit it separately, or move the data to a
private repo.

See .claude/rules/git-workflow.md and the budget-watch feedback memory.

EOF
  exit 1
fi

exit 0
