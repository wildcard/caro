#!/usr/bin/env bash
# Shared input parsing for PreToolUse guard hooks. Source it, don't run it.
#
# Claude Code sends the hook payload as JSON on stdin (not env vars), and only
# exit code 2 blocks the tool call. After sourcing:
#   TOOL_NAME   tool being called, e.g. "Bash"
#   COMMAND     tool_input.command for Bash calls
#   HOOK_CWD    session working directory
#   BLOCK       exit code that blocks the call (2)
#
# effective_git_dir resolves where a git command will actually run, following a
# leading `cd <dir> &&` or `git -C <dir>`, so guards judge the right checkout.

BLOCK=2

_HOOK_JSON="$(cat)"

_json_field() {
  if command -v jq >/dev/null 2>&1; then
    jq -r "$1 // empty" <<<"$_HOOK_JSON" 2>/dev/null
  else
    python3 -c 'import json,sys
d=json.loads(sys.stdin.read() or "{}")
for k in sys.argv[1].lstrip(".").split("."):
    d=d.get(k) if isinstance(d,dict) else None
print(d if d is not None else "")' "$1" <<<"$_HOOK_JSON" 2>/dev/null
  fi
}

TOOL_NAME="$(_json_field .tool_name)"
COMMAND="$(_json_field .tool_input.command)"
HOOK_CWD="$(_json_field .cwd)"
HOOK_CWD="${HOOK_CWD:-$PWD}"

effective_git_dir() {
  local dir="$HOOK_CWD" re_cd='^[[:space:]]*cd[[:space:]]+([^;&|[:space:]]+)' re_c='git[[:space:]]+-C[[:space:]]+([^[:space:]]+)'
  if [[ "$COMMAND" =~ $re_cd ]]; then
    dir="${BASH_REMATCH[1]}"
  elif [[ "$COMMAND" =~ $re_c ]]; then
    dir="${BASH_REMATCH[1]}"
  fi
  dir="${dir/#\~/$HOME}"
  [[ "$dir" = /* ]] || dir="$HOOK_CWD/$dir"
  printf '%s\n' "$dir"
}
