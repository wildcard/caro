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
export CARO_STATUS_BRANCH=automation/routine-status

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

# Concurrent writers: every record must land (push conflicts retry with a
# randomized backoff; a fixed delay let only one writer through per round).
for i in 1 2 3 4 5 6 7 8; do
  "$TOOL" record "parallel-$i" Succeeded "run $i" >/dev/null &
done
wait
check "8 concurrent records all land (got $(records) total)" test "$(records)" = 9
landed() { git -C "$REMOTE" show automation/routine-status:runs.jsonl | grep -c "\"routine\":\"parallel-$1\""; }
for i in 1 2 3 4 5 6 7 8; do
  check "parallel-$i landed exactly once" test "$(landed "$i")" = 1
done

# A push that can't succeed reports git's own error and cleans up after itself.
before="$(find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'tmp.*' 2>/dev/null | wc -l)"
err="$(CARO_STATUS_ATTEMPTS=2 CARO_STATUS_REMOTE="$TMP/missing.git" "$TOOL" record qa-routine Failed "x" 2>&1)"
rc=$?
check "an unpushable record exits non-zero (rc=$rc)" test "$rc" -ne 0
check "the failure names the git error" bash -c "grep -q 'Last error: .*missing.git' <<<\"\$1\"" _ "$err"
after="$(find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'tmp.*' 2>/dev/null | wc -l)"
check "a failed record leaves no temp clone behind" test "$after" -le "$before"

check "show --check passes when all latest runs succeeded" "$TOOL" show --check
"$TOOL" record sweep Blocked "no GitHub tools in session" >/dev/null
check "show marks a Blocked routine" bash -c "'$TOOL' show | grep -q '^! sweep .*Blocked'"
check "show --check fails when a routine is not Succeeded" fails "$TOOL" show --check

"$TOOL" record sweep Succeeded "fixed BW-005" --started 2020-01-01T00:00:00Z >/dev/null
check "latest record per routine wins" bash -c "'$TOOL' show | grep -q '^  sweep .*Succeeded'"
check "--max-age-hours 0 flags every routine as stale" fails "$TOOL" show --check --max-age-hours 0

# Per-routine cadence: a weekly routine 3 days old is fine; a daily one is overdue.
old_ts="$(python3 -c 'from datetime import datetime,timedelta,timezone; print((datetime.now(timezone.utc)-timedelta(hours=72)).strftime("%Y-%m-%dT%H:%M:%SZ"))')"
python3 - "$REMOTE" "$old_ts" "$TMP/aged-clone" <<'PY'
import json, subprocess, sys, os
remote, ts, d = sys.argv[1], sys.argv[2], sys.argv[3]
subprocess.run(["git", "clone", "-q", "--branch", "automation/routine-status", remote, d], check=True)
with open(os.path.join(d, "runs.jsonl"), "a") as f:
    for name, age in (("weekly-job", 170), ("daily-job", 26)):
        f.write(json.dumps({"routine": name, "phase": "Succeeded", "reason": "ok", "started": ts,
                            "finished": ts, "pr": None, "issues": [], "max_age_hours": age}) + "\n")
subprocess.run(["git", "-C", d, "commit", "-qam", "aged records"], check=True)
subprocess.run(["git", "-C", d, "push", "-q", "origin", "HEAD:automation/routine-status"], check=True)
PY
check "a weekly routine 72h old is not overdue" bash -c "'$TOOL' show | grep -q '^  weekly-job '"
check "a daily routine 72h old is overdue" bash -c "'$TOOL' show | grep -q '^! daily-job '"
check "record stores --max-age-hours" bash -c \
  "'$TOOL' record cadence-job Succeeded ok --max-age-hours 170 | grep -q '\"max_age_hours\":170'"
check "non-positive --max-age-hours is refused" fails "$TOOL" record cadence-job Succeeded ok --max-age-hours 0

# An unreachable remote is an error, never an empty history.
check "show on an unreachable remote exits non-zero" fails env CARO_STATUS_REMOTE="$TMP/missing.git" "$TOOL" show
check "show on an unreachable remote does not claim 'no records'" bash -c \
  "! CARO_STATUS_REMOTE='$TMP/missing.git' '$TOOL' show 2>&1 | grep -q 'no status records yet'"

echo "routine-status: $pass passed, $fail failed"
[[ "$fail" == 0 ]]
