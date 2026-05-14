# Caro PR Hygiene Sprint — 2026-05-13

> **Hermes Effort Meter** (H-EM): ⚫=trivial(5m) 🟢=easy(15m) 🟡=medium(30m) 🟠=hard(60m) 🔴=extreme(>60m)
> **Risk**: ✅safe  ⚠️moderate  🚨high

---

## Track 1: CLOSE (6 PRs — immediate cleanup)

| PR | Title | Why Close | Effort |
|----|-------|-----------|--------|
| #1036 | chmod safety fix | Superseded by #1046 (merged) | ⚫ |
| #1061 | ChromaDB non-blocking | Superseded by #1074 (merged) | ⚫ |
| #1024 | npm dep bump (dup) | Duplicate of #1025 | ⚫ |
| #1025 | npm dep bump | 61 commits behind, dependabot regenerates | ⚫ |
| #927 | hf-hub bump | 66 commits behind, Cargo.lock stale | ⚫ |
| #1004 | CI + translations + flaky test | Monolithic (27 files, 3 concerns). Split out useful parts. | ⚫ |

**Action:** Claude Code closes all 6 with explanatory comments.
**Agent:** `pr-management-loop` (auto-close with reason)

---

## Track 2: REBASE AND MERGE (3 PRs — value recovery)

| PR | Title | Effort | Risk | Agent |
|----|-------|--------|------|-------|
| #1045 | QA rotation | ⚫ trivial (5m) | ✅ safe | `pr-management-loop` |
| #940 | Creative QA generator | 🟢 easy (15m) | ✅ safe | `pr-management-loop` |
| #1043 | Windows fix (Anastasia) | 🟠 hard (60m) | ⚠️ moderate | `rust-cli-expert` |

**PR #1045** — Docs-only (QA memory files). Cherry-pick 4 files.
**PR #940** — All new files (agent def, config, corpus). 3 new files additive, 1 manual merge.
**PR #1043** — High value (real Windows bugs). Needs rebase against CaroML-restructured main.rs. Requires `rust-cli-expert` who understands both the Windows fix intent and the new code paths.

---

## Track 3: NEEDS HUMAN DECISION (1 PR)

| PR | Title | Effort | Risk | Decision |
|----|-------|--------|------|----------|
| #993 | Design system rollout | 🔴 extreme (90m) | ⚠️ moderate | Rebase Rust-only? Close and rebuild? |

**Context:** 134 files, mostly font assets. The `src/ui/` module is valuable and additive. But `main.rs`/`cli/mod.rs` overlap heavily with CaroML. Tokens.css brand tokens never landed.
**Recommendation:** Close the PR. Salvage `src/ui/` module into a focused follow-up PR.

---

## Track 4: EXTERNAL CONTRIBUTOR (1 PR)

| PR | Title | Effort | Risk | Action |
|----|-------|--------|------|--------|
| #1065 | mkfs regex fix | ⚫ trivial | ✅ safe | Approve CI, merge when green |

**Status:** CLA signed, CLA check re-triggered. Needs workflow approval.

---

## Execution Order

```
Phase 1 (today, 15 min):
  ├── Close 6 stale PRs with comments
  ├── Rebase + merge #1045 (trivial)
  └── Approve CI for #1065

Phase 2 (today, 30 min):
  ├── Rebase + merge #940 (easy)
  └── Assign #1043 to rust-cli-expert

Phase 3 (needs your input):
  └── Decision on #993 (design system)
```
