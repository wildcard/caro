# QA rotation state: seed

Built 2026-10-10 and rebuilt 2026-10-11 for `state/qa-rotation/` on `automation/routine-status`.

## Where it comes from

- **Base:** main's copies of the four `.claude/memory/qa-*.md` files, last changed 2026-05-14.
- **34 scheduled-rotation PRs, #1166 to #1560** (2026-05-23 to 2026-10-10). Each one edited that same base copy, and no run saw another run's PR.
- **#886**, an earlier bootstrap (Apr 26 to May 3) that never merged. Only its two dated session entries are kept, at the end of the log under a note.

## How the PRs were merged

- **Session log:** one entry per run, newest first, then main's entry, then #886's entries. #1543's edit of main's 2026-05-07 entry replaces that entry.
- **Tables:** one row per key, the first cell. For a row that main already had, the newest PR that changed or removed it decides. A row that only PRs added uses the newest PR's version. Date-keyed rows are sorted by date.
- **`**Label**:` lines and `### FLAKE-NNN` headings:** the newest PR that changed them wins. All of a flake's lines merge into one section.
- **List items:**
  - an item that starts with a date is kept once per date, in the newest PR's wording;
  - an item that cites an issue is kept once per issue;
  - the rest are the union.
- **Prose:** a line that a PR rewrote in place takes the newest PR's version (the two intro lines). Table separator widths keep main's version.
- **FLAKE-001:** #1371 marked it resolved on 2026-07-24, but later runs kept it under Active and observed it again. So the stale "Resolved" and "Resolution" lines are dropped.

## After the merge

- **Status cells:** for issue-keyed rows, the cell shows the issue's GitHub state on 2026-10-11. 80 cells changed: of the 56 issues tracked, 12 are open, 36 closed as duplicates and 8 completed.
- **Removed figures:** 13 size and duration figures were replaced with "[size omitted]" or "[duration omitted]". The QA routine's own rule says: no $ / minute / GB metrics in caro files.
- **Notes added by hand (2026-10-11),** so the next run files no duplicate:
  - FLAKE-001 gets a "Fix" line: #1561 closed #1440.
  - FLAKE-002 (new on 2026-10-10) gets a "Duplicate of" line. It is FLAKE-001's `caro ai --once` variant and #1540.
  - The 2026-10-10 run could not file its two findings, so it logged them as `pending-1` and `pending-2`. Their Status cells now say "do not file": `pending-1` is fixed by #1478, and `pending-2` duplicates #1408 and #1179. The matching row in the coverage matrix says the same.

## What the merged state shows

- **33 of the 34 runs tested the same Slot C surface,** #10 (`caro ai --once`). Slot C picks the oldest "Last tested" row. In the shared May copy, #10 is the lowest-numbered "never" row, so every run picked it, and several logged it as its first test. 16 of the 34 surfaces are still untested.
- **FLAKE-001 was observed 3 times in 7 days** (2026-10-05, 10-07 and 10-08), but each run logged a "2nd occurrence", because no run could see the others. The hang behind it was filed as #1440, which #1561 fixed on 2026-10-11 (see FLAKE-001's "Fix" line). On 2026-10-10 a run saw the same hang and logged it as a new FLAKE-002.
- **The 2026-10-10 run reports `GH_TOKEN invalid`**, so it filed no issues (see its session-log entry).

## Before seeding

The status cells match GitHub on 2026-10-11. If seeding happens on a later day, re-run `refresh_status.py` with fresh issue states first.

## Rebuild

From the session that built it:

1. Run `merge.py <workdir> <prs.txt>`. The PR heads come from `git fetch origin refs/pull/<n>/head`.
2. Run `refresh_status.py <seed> <issue-states.txt>`.
3. Run `annotate_v2.py <first-seed> <seed>` to add the notes above.
