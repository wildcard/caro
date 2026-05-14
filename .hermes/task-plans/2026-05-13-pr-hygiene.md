# Caro PR Hygiene Sprint — CORRECTED 2026-05-13

> **Hermes Effort Meter** (H-EM): ⚫=trivial(5m) 🟢=easy(15m) 🟡=medium(30m) 🟠=hard(60m) 🔴=extreme(>60m)
> **Risk**: ✅safe  ⚠️moderate  🚨high

---

## CORRECTIONS FROM CRITICAL VERIFICATION

Previous assessment was too hasty. Kobi correctly pushed back.

| PR | Previous Assessment | Corrected Assessment | Evidence |
|----|--------------------|--------------------|----------|
| #1036 | CLOSE (superseded) | **NOT SUPERSEDED** — 3 real bypasses remain | `chmod --recursive`, `0777`, both combined |
| #1061 | CLOSE (superseded) | **PARTIALLY SUPERSEDED** — #1074 fixed root cause but didn't add continue-on-error safety net | Need CI flakiness check |

**Action taken:** PR #1085 opened to fix the chmod gaps. TDD approach, 3 tests + regex fix.

---

## Track 1: CLOSE (4 PRs — immediate cleanup)

| PR | Title | Why Close | Effort |
|----|-------|-----------|--------|
| #1024 | npm dep bump (dup) | Duplicate of #1025 | ⚫ |
| #1025 | npm dep bump | 61 commits behind, dependabot regenerates | ⚫ |
| #927 | hf-hub bump | 66 commits behind, Cargo.lock stale | ⚫ |
| #1004 | CI + translations + flaky test | Monolithic (27 files, 3 concerns). Split out useful parts. | ⚫ |

---

## Track 2: REBASE AND MERGE (3 PRs — value recovery)

| PR | Title | Effort | Risk | Agent |
|----|-------|--------|------|-------|
| #1045 | QA rotation | ⚫ trivial (5m) | ✅ safe | pr-management-loop |
| #940 | Creative QA generator | 🟢 easy (15m) | ✅ safe | pr-management-loop |
| #1043 | Windows fix (Anastasia) | 🟠 hard (60m) | ⚠️ moderate | rust-cli-expert |

---

## Track 3: NEEDS CI CHECK (1 PR)

| PR | Title | Decision Needed |
|----|-------|----------------|
| #1061 | ChromaDB non-blocking | Check if ChromaDB 0.6.3 fixed flakiness. If tests still flaky → rebase and merge. If stable → close. |

---

## Track 4: NEEDS HUMAN DECISION (1 PR)

| PR | Title | Effort | Risk | Decision |
|----|-------|--------|------|----------|
| #993 | Design system rollout | 🔴 extreme (90m) | ⚠️ moderate | Phase 1 done (PR #1086). Phase 2+3 need follow-up. |

---

## Track 5: EXTERNAL CONTRIBUTOR (1 PR)

| PR | Title | Effort | Risk | Action |
|----|-------|--------|------|--------|
| #1065 | mkfs regex fix | ⚫ trivial | ✅ safe | Approve CI, merge when green |

---

## Sprint 1 Progress

| Task | Status | PR | Agent |
|------|--------|-----|-------|
| chmod bypasses | DONE | #1085 | Claude Code |
| Design system Phase 1 | DONE | #1086 | Claude Code |
| Guardian Positioning | In progress | — | Claude Code |
| SafetyDecision payload | In progress | — | Claude Code |

---

## Execution Order

```
Phase 1 (today, 15 min):
  ├── Close 4 stale PRs with comments
  ├── Rebase + merge #1045 (trivial)
  └── Approve CI for #1065

Phase 2 (today, 30 min):
  ├── Rebase + merge #940 (easy)
  ├── Check CI flakiness for #1061
  └── Assign #1043 to rust-cli-expert

Phase 3 (needs your input):
  └── Decision on #993 Phase 2+3
```

---

*Corrected assessment. Previous version had errors on PRs #1036 and #1061.*