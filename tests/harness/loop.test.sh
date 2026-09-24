#!/usr/bin/env bash
# Regression guard for loop.sh budgets.
#
# loop.sh used to default to unlimited iterations with no per-iteration time
# limit, so a stuck or looping agent could run (and spend) forever. These
# tests run loop.sh against a stub `claude` and assert the budgets hold.
#
# Usage: tests/harness/loop.test.sh

set -uo pipefail

LOOP="$(cd "$(dirname "$0")/../.." && pwd)/loop.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

pass=0
fail=0

check() {
  # check <description> <command...>: passes when the command succeeds.
  local desc="$1"
  shift
  if "$@"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc" >&2
  fi
}

# new_env <name> <body>: work dir with a stub `claude` running <body>.
new_env() {
  local dir="$TMP/$1" body="$2"
  mkdir -p "$dir/bin"
  printf '#!/usr/bin/env bash\n%s\n' "$body" > "$dir/bin/claude"
  chmod +x "$dir/bin/claude"
  git init -q "$dir/work"
  echo "do the thing" > "$dir/work/PROMPT_build.md"
  printf '%s\n' "$dir"
}

# run_loop <env-dir> [VAR=value...]: run loop.sh (bounded by an outer timeout).
run_loop() {
  local dir="$1"
  shift
  (cd "$dir/work" && env PATH="$dir/bin:$PATH" RALPH_PAUSE_SECONDS=0 \
    RALPH_RETRY_SECONDS=0 RALPH_LOG_FILE="$dir/ralph.log" "$@" \
    timeout 60 "$LOOP" build >"$dir/out.log" 2>&1)
}

iterations() {
  local n
  n="$(grep -c 'Iteration [0-9]* starting' "$1/ralph.log" 2>/dev/null)"
  echo "${n:-0}"
}

# Default cap: finite (20), not unlimited.
d="$(new_env default 'echo ok')"
run_loop "$d"
check "default run stops after 20 iterations (got $(iterations "$d"))" test "$(iterations "$d")" = 20

# Explicit cap.
d="$(new_env capped 'echo ok')"
run_loop "$d" RALPH_MAX_ITERATIONS=3
check "RALPH_MAX_ITERATIONS=3 stops at 3 (got $(iterations "$d"))" test "$(iterations "$d")" = 3

# Per-iteration timeout kills a hung agent.
d="$(new_env hung 'sleep 30')"
start=$(date +%s)
run_loop "$d" RALPH_MAX_ITERATIONS=1 RALPH_ITERATION_TIMEOUT=1s
elapsed=$(( $(date +%s) - start ))
check "hung iteration is killed by RALPH_ITERATION_TIMEOUT (took ${elapsed}s)" test "$elapsed" -lt 20
check "timeout is logged" grep -q 'timed out' "$d/ralph.log"

# Invalid cap is rejected, not treated as unlimited.
d="$(new_env invalid 'echo ok')"
run_loop "$d" RALPH_MAX_ITERATIONS=lots
rc=$?
check "non-integer RALPH_MAX_ITERATIONS exits non-zero (rc=$rc)" test "$rc" -ne 0
check "non-integer cap runs no iterations" test "$(iterations "$d")" = 0

echo "loop.sh: $pass passed, $fail failed"
[[ "$fail" == 0 ]]
