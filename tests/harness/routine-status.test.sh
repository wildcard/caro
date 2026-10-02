#!/usr/bin/env bash
# Regression guard for bin/routine-status (routine status write-back).
#
# Routines used to report only the platform's "session ended" status, so a run
# that did nothing still looked green. These tests run the helper against a
# local bare remote and check that records land on the status branch (never
# main), that bad input is refused, that concurrent writers all land, and that
# `show --check` flags failed and stale routines.
#
# Usage: tests/harness/routine-status.test.sh

set -uo pipefail

TOOL="$(cd "$(dirname "$0")/../.." && pwd)/bin/routine-status"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
REMOTE="$TMP/remote.git"
git init -q --bare "$REMOTE"
git -C "$REMOTE" symbolic-ref HEAD refs/heads/main
export CARO_STATUS_REMOTE="$REMOTE"

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
fails() { ! "$@" >/dev/null 2>&1; }
records() { git -C "$REMOTE" show automation/routine-status:runs.jsonl 2>/dev/null | wc -l | tr -d ' '; }

check "show with no records says so" bash -c "'$TOOL' show | grep -q 'no status records yet'"
check "show --check with no records fails" fails "$TOOL" show --check

check "record a success" "$TOOL" record qa-routine Succeeded "filed 2 issues" --pr 12 --issues 3,4
check "record lands on the status branch" test "$(records)" = 1
check "nothing is written to main" fails git -C "$REMOTE" rev-parse -q --verify refs/heads/main
check "record is valid JSON with the fields" bash -c \
  "git -C '$REMOTE' show automation/routine-status:runs.jsonl | python3 -c 'import json,sys; r=json.loads(sys.stdin.readline()); assert r[\"routine\"]==\"qa-routine\" and r[\"pr\"]==12 and r[\"issues\"]==[3,4] and r[\"phase\"]==\"Succeeded\"'"

check "invalid phase is refused" fails "$TOOL" record qa-routine Done "x"
check "empty reason is refused" fails "$TOOL" record qa-routine Failed ""
check "non-numeric --pr is refused" fails "$TOOL" record qa-routine Failed "x" --pr abc
check "writing to main is refused" fails env CARO_STATUS_BRANCH=main "$TOOL" record qa-routine Failed "x"
check "refused records were not written" test "$(records)" = 1

# Concurrent writers: every record must land (push conflicts retry).
for i in 1 2 3 4; do
  "$TOOL" record "parallel-$i" Succeeded "run $i" >/dev/null &
done
wait
check "4 concurrent records all land (got $(records) total)" test "$(records)" = 5

check "show --check passes when all latest runs succeeded" "$TOOL" show --check
"$TOOL" record sweep Blocked "no GitHub tools in session" >/dev/null
check "show marks a Blocked routine" bash -c "'$TOOL' show | grep -q '^! sweep .*Blocked'"
check "show --check fails when a routine is not Succeeded" fails "$TOOL" show --check

"$TOOL" record sweep Succeeded "fixed BW-005" --started 2020-01-01T00:00:00Z >/dev/null
check "latest record per routine wins" bash -c "'$TOOL' show | grep -q '^  sweep .*Succeeded'"
check "--max-age-hours 0 flags every routine as stale" fails "$TOOL" show --check --max-age-hours 0

echo "routine-status: $pass passed, $fail failed"
[[ "$fail" == 0 ]]
