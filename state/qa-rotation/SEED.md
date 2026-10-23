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

## Added after seeding: #1604 (2026-10-11)

The 14:07 run on 2026-10-11 still used the routine's old prompt. So it read main's May copy, filed #1603 about that stale copy, and opened PR #1604. Its changes were folded in by hand, with the rules above:

- **Session log:** its entry goes on top.
- **Its rows win:** #1408 (now P1), the 2026-10-11 smoke row, and surfaces 1, 2, 3, 10 and 25. #1272 and #1603 are new rows.
- **FLAKE-002:** takes its labels, its new "Root cause" and "Stall watchdog caveat" lines, and the 2026-10-11 observation. The "Duplicate of" line stays.
- **FLAKE-001 stays active.** #1604 marked it resolved, because `--dry-run` passed through the static matcher. It did not retest prompts that need the LLM, so a dated line records the observation instead.
- **Not taken:** its resolved-list line for #1044 ("Fixed 2026-05-09"). #1044 closed as a duplicate of #1098, which #1478 fixed on 2026-10-11.

With #1604, 34 of the 35 runs tested Slot C surface #10. This branch is the fix that #1603 asks for: #1603 can close after a run reads this state with `state get`.

## Added after seeding: #1653 (2026-10-13)

The 14:08 run on 2026-10-13 also used the old prompt. It read main's May copy, filed no issue and opened PR #1653. Its changes were folded in by hand, with the rules above:

- **Session log:** its entry goes on top, with a note on what it could not see. One duration figure became "[duration omitted]".
- **Its rows win:** the 2026-10-13 smoke row, the dates of surfaces 10 and 32, and #1408's status ("open; fix PR #1415 pending").
- **New surface:** `caro export` is surface 35, because the seed already uses 33 and 34.
- **Flakes:** FLAKE-001 gets a "not triggered" line. FLAKE-002 gets the 2026-10-13 observation; #1653 had filed it under #1408 in FLAKE-001's section.
- **Not taken:** its #1044 edits. The seed already lists #1044 as closed.

With #1653, 35 of the 36 runs tested Slot C surface #10.

## Added after seeding: #1674 (2026-10-15)

The 14:08 run on 2026-10-15 also used the old prompt. It read main's May copy and opened PR #1674. It filed #1673 (P0), which is the CPU stub of #1269 (BW-006): the root cause of #1494 and the other closed duplicates. Its changes were folded in by hand, with the rules above:

- **Session log:** its entry goes on top, with a note on what it could not see. One duration figure became "[duration omitted]".
- **Its rows win:** the 2026-10-15 smoke row and surface 10 (FAIL). Surface 10 links #1269, because #1673 is its duplicate.
- **Backlog:** #1673 gets a row marked as a duplicate of #1269. The #1408 row keeps its 2026-10-15 review text.
- **Flakes:** FLAKE-001 and FLAKE-002 each get a "not triggered" line. The download finished, so `ai --once` reached the stub instead of hanging.
- **Not taken:** its "Resolved (closed issues)" line for #1673, because #1673 is open.
- **Also added:** surface 11 links #1671 (the `?` widget cannot take a request), filed by the maintainer session on 2026-10-15.

With #1674, 36 of the 37 runs tested Slot C surface #10.

## Added after seeding: #1678, #1679, #1690, #1695, #1698 (2026-10-16 to 2026-10-21)

Five runs used the old prompt. Each read main's May copy and opened a PR (2026-10-17 has none). They were folded on 2026-10-22, oldest first, with the rules above:

- **Session log:** each entry goes on top with a note on what the run could not see. Build durations, one test-query size and one curl time became "[duration omitted]" or "[size omitted]".
- **Their rows win, matched by surface name:** smoke rows for all five dates; surfaces 2, 10, 15, 25 and 31 get newer dates; surfaces 11 (FAIL, the CPU stub of #1269) and 16 (PASS) are tested for the first time.
- **Renumbered:** `caro guard test / setup` is surface 36, because the seed already uses 33. The caro.sh first-request flake is FLAKE-003, because the seed already uses FLAKE-002.
- **Backlog:** #1677 (closed, fixed by #1680), #1697 (open) and #1116 (re-confirmed). #1269 keeps P1, not the P0 one run wrote. The #1408 row keeps its 2026-10-15 review text.
- **Not taken:** surface 12 stays excluded from Slot C; the runs re-tested a disabled command three times because the May copy listed it as "never".

Four of these five runs tested surface 10 again, because the May copy still lists it as "never".

## Added after seeding: #1705 (2026-10-22)

The 2026-10-22 run used the old prompt and read main's May copy. It was folded on the same day with the rules above:

- **Session log:** its entry goes on top with a note. The build duration became "[duration omitted]".
- **Its rows win:** a smoke row for 2026-10-22; surfaces 2 (PASS) and 10 (FAIL, the CPU stub of #1269) get the new date. Surface 10 keeps the seed's links.
- **From Slot B:** its `caro guard test` checks (deny with exit 2, also after `&&`; allow with exit 0) give surface 36 the new date. It did not run `caro guard setup`, so the result stays PARTIAL.
- **Backlog:** no new issues. #1269 is re-confirmed and keeps P1.
- **Corrected:** #1673 was closed as a duplicate of #1269 on 2026-10-15, but the matrix and the backlog still said "open". The run's log had it right.
- **Flakes:** FLAKE-001 and FLAKE-002 were not triggered.

Like almost every run since May, it tested surface 10, because the May copy still lists it as "never".

## Added after seeding: #1709 (2026-10-23)

The 2026-10-23 run used the old prompt and read main's May copy. It was folded on the same day with the rules above:

- **Session log:** its entry goes on top with a note. The build duration became "[duration omitted]".
- **Its rows win:** a smoke row for 2026-10-23, and surface 14 (`caro config get/set/show/reset`) moves from PARTIAL to PASS. The seed's note that show and reset were untested is replaced, and the run's unfiled cosmetic note (the `safety bogus` error repeats its message) is kept.
- **Backlog:** no change. The run filed no issues, and #1044, which it cites, is already closed in the seed.
- **Flakes:** FLAKE-001 was not triggered.

This run picked surface 14 instead of surface 10, because the May copy lists both as "never".
