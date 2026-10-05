# v2.0.0 Release State

**Last Updated**: 2026-10-05 (weekly planning agent run)
**Routine ID**: `trig_01KTFtDwFfs4xHiJ2JVbCgEV`
**Schedule**: Every Monday at 09:00 AM PDT (16:00 UTC)
**Manage at**: https://claude.ai/code/routines/trig_01KTFtDwFfs4xHiJ2JVbCgEV

## Current Status

**Completion**: 61.7% (29/47 GitHub milestone items)
**Core feature completion**: 0% (0/5 gantt features implemented)
**Open blockers**: 4 identified
**Target release**: June 30, 2026 → **97 DAYS OVERDUE**
**Status**: BLOCKED — requires owner scope decision

## Blockers

1. **Validation discipline (CRITICAL)** — All 5 core features need 20 user interviews each;
   current count is 0/20 across all five features
2. **Release date passed** — June 30, 2026 target; now October 5, 2026
3. **PR #1482 unmerged** — Sept-28 planning PR still open (7 days), docs-only safe to merge
4. **Scope decision deferred** — No action on A/B options since July 6 run

## Core Feature Issues

| Feature | Issue | Gantt Status | Transcripts |
|---------|-------|-------------|-------------|
| Karo Distributed Intelligence | #133 | OVERDUE (Apr 1–May 1) | 0/20 |
| Dogma Rule Engine | #1075 | OVERDUE (Apr 1–Apr 25) | 0/20 |
| Voice Synthesis | #160 | OVERDUE (Apr 15–May 5) | 0/20 |
| Self-Healing | #1151 | OVERDUE (May 1–May 26) | 0/20 |
| Local Context Indexing | #1152 | OVERDUE (May 15–Jun 15) | 0/20 |

## Next 3 Priority Items

1. Begin user discovery for Self-Healing (#1151) — run `caro.discovery` skill, 20 interviews
2. Begin user discovery for Local Context Indexing (#1152) — ChromaDB phases 1–3 already done
3. Action scope decision: reset to Q1 2027 OR ship v2.0.0 on 29 completed items

## Scope Options (owner decision needed)

**Option A** — Reset release date to Q1 2027, begin discovery work now. Preserves semver
intent: v2.0.0 ships when the major-version features (Karo, Dogma, Voice, Self-Healing, Context)
are actually built.

**Option B** — Ship v2.0.0 on the 29 already-completed items; create "v2.1.0 - Core AI Features"
for the 5 discovery-gated features. Resolves the overdue state but raises the question of what
major-version change justifies bumping from 1.x.

## Milestone Alignment Gaps

Issues referencing v2.0.0 in body but NOT assigned to GitHub milestone #3:
- #1151 (Self-Healing) — needs milestone assignment
- #1152 (Local Context Indexing) — needs milestone assignment
- #1075 (Dogma Rule Engine) — needs milestone assignment

## Weekly Reports

| Date | Report | PR |
|------|--------|----|
| 2026-05-25 | `.claude/memory/v200-weekly-report-2026-05-25.md` | #1173 (closed unmerged Jul 12) |
| 2026-07-06 | `.claude/memory/v200-weekly-report-2026-07-06.md` | n/a (direct commit) |
| 2026-09-28 | `.claude/memory/v200-weekly-report-2026-09-28.md` | #1482 (open, 7d) |
| 2026-10-05 | `.claude/memory/v200-weekly-report-2026-10-05.md` | This run |

## Notes

- GitHub milestone: "v2.0.0 - Distributed Autonomy" (milestone #3, due Jun 30 2026)
- All 5 core gantt features blocked by `.claude/rules/validation-discipline.md` Gate 1
- discovery-debt-v2.0 epic in beads tracks validation work
- See `docs/discovery/v2.0-validation-audit.md` for per-feature audit status
