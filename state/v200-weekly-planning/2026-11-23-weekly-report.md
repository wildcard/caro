# v2.0.0 Weekly Planning Report — 2026-11-23

## Completion
- Total roadmap items: 50 in GitHub milestone #3 ("v2.0.0 - Distributed Autonomy"): 23 issues + 27 PRs
- Issues created/tracked: 23 issues (0 created this run; every ROADMAP v2.0.0 item has an issue)
- Closed/done: 29 (27 merged/closed PRs + issues #7, #11)
- Completion %: 58% by milestone items (29/50); 8.7% by issues only (2/23)
- Core gated features (Karo #133, Dogma #1075, Voice #160, Self-Healing #1151, Local Context #1152): 0/5, each 0/20 transcripts

## This week
- New issues created: none. No untracked ROADMAP item. The `milestone:v2.0.0` label still has 0 issues; tracking uses milestone #3.
- PRs with CI failures: none to flag. No open PR is in milestone #3, so no PR comments were posted.
- Blockers identified:
  1. Validation-discipline Gate 1: 0/20 transcripts for all five research lines. `docs/discovery/transcripts/` holds only `README.md`. Unchanged since 2026-05-25 (26 weeks).
  2. Target date 2026-06-30 passed 146 days ago. Milestone #3 and ROADMAP.md still show it.
  3. Routine drift: the runs on 10-19, 10-26, 11-02, 11-09 and 11-16 each opened a PR (#1694, #1725, #1745, #1757, #1769) instead of writing state to `automation/routine-status` (CONTRACT.md section 5). All five are still open. None recorded a status, so the last state on the branch was 2026-10-12. This run writes to the branch and opens no PR.
  4. ROADMAP.md v2.0.0 header says "38% Complete (13 open, 8 closed)". Milestone #3 has 21 open and 29 closed items.
- Tooling: `bd` is not installed in this session, so beads were not checked.

## Next milestone items
1. Run `caro.discovery` interviews for Self-Healing (#1151): 0/20 transcripts.
2. Run `caro.discovery` interviews for Local Context Indexing (#1152): 0/20 transcripts.
3. Owner: set a new target date on milestone #3 (Decision D1 keeps v2.0.0 for validated features), and close planning PRs #1694, #1725, #1745, #1757, #1769 as superseded by the status branch.

Unstarted non-gated work that needs no discovery: #4, #5, #6 (code quality and security hardening, all open since 2025).

## Status
Blocked
