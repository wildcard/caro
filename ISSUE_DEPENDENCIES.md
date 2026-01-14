# Exploration Agent: Issue Dependencies

## Epic Overview

**Epic #449**: Exploration Agent: Complete Integration & Rollout
- **Status**: Core implementation complete, integration needed
- **Branch**: `feature/agentic-context-loop`
- **Total Issues**: 5 child issues

---

## Dependency Graph

```
                    Epic #449
                    Exploration Agent
                         |
                         |
         +---------------+---------------+
         |                               |
    Phase 1-2                        Phase 3-4
         |                               |
         v                               v
    
    [#430] CLI Integration          [P0 - CRITICAL]
    └─ Blocks: ALL other issues
    └─ Time: 30-45 min
    └─ Status: Ready to start
         |
         +----------------+----------------+
         |                |                |
         v                v                v
    
    [#431]           [#434]           [#439]
    Vancouver        Async            Documentation
    Validation       Exploration      
    └─ P1            └─ P2            └─ P2
    └─ 30-45 min     └─ 45-60 min     └─ 1-2 hours
         |                |                |
         |                v                |
         |           [#436]                |
         |           Interactive           |
         |           Selection             |
         |           └─ P2                 |
         |           └─ 45-60 min          |
         |                                 |
         +----------------+----------------+
                          |
                          v
                    ALL COMPLETE
                          |
                          v
                    Merge to main
                    Release v0.x.0
```

---

## Issue Summary

### 🚨 Critical Path (Must Do First)

#### #430: CLI Integration
- **Priority**: P0 (BLOCKS EVERYTHING)
- **Time**: 30-45 minutes
- **Dependencies**: None
- **Blocks**: #431, #434, #436, #439
- **Description**: Add `--explore` flag to CLI, wire exploration agent
- **Deliverable**: `cmdai "query" --explore` works end-to-end

---

### ✅ Validation (High Priority)

#### #431: Vancouver Demo Validation
- **Priority**: P1
- **Time**: 30-45 minutes
- **Dependencies**: Requires #430
- **Blocks**: None
- **Description**: Validate 6/6 Vancouver demos pass with exploration
- **Deliverable**: `demos/vancouver_validation.md` with results

---

### 💡 Enhancements (Nice to Have)

#### #434: Async Exploration
- **Priority**: P2
- **Time**: 45-60 minutes
- **Dependencies**: Requires #430
- **Blocks**: #436
- **Description**: Progressive enhancement (quick result + background exploration)
- **Deliverable**: `--explore-wait` and `--explore-show` flags work

#### #436: Interactive Selection
- **Priority**: P2
- **Time**: 45-60 minutes
- **Dependencies**: Requires #430, #434
- **Blocks**: None
- **Description**: Interactive alternative selection on failure
- **Deliverable**: `--alternatives` and `--last` flags work

---

### 📚 Documentation (Pre-Release)

#### #439: Documentation Updates
- **Priority**: P2
- **Time**: 1-2 hours
- **Dependencies**: Requires #430 (optional: #434, #436 for complete docs)
- **Blocks**: None
- **Description**: User-facing documentation, examples, tutorials
- **Deliverable**: README, USER_GUIDE, EXAMPLES updated

---

## Work Sequence Options

### Option A: Minimum Viable Product (MVP)
**Goal**: Get exploration in users' hands ASAP
**Time**: ~2-3 hours

1. ✅ #430 - CLI Integration (30-45 min)
2. ✅ #431 - Vancouver Validation (30-45 min)
3. ✅ #439 - Documentation (1-2 hours)
4. 🚀 **Merge & Release**

**Result**: Users can use `--explore`, docs explain how, validation proves it works

---

### Option B: Enhanced UX
**Goal**: Best user experience with async and interactive features
**Time**: ~4-6 hours

1. ✅ #430 - CLI Integration (30-45 min)
2. ✅ #434 - Async Exploration (45-60 min)
3. ✅ #436 - Interactive Selection (45-60 min)
4. ✅ #431 - Vancouver Validation (30-45 min)
5. ✅ #439 - Documentation (1-2 hours)
6. 🚀 **Merge & Release**

**Result**: Full-featured exploration with progressive enhancement

---

### Option C: Parallel Execution (Fastest)
**Goal**: Complete everything in minimal time using parallel work
**Time**: ~3-4 hours (with multiple agents)

**Agent 1** (Critical Path):
1. ✅ #430 - CLI Integration (30-45 min)
2. ✅ #434 - Async Exploration (45-60 min)
3. ✅ #436 - Interactive Selection (45-60 min)

**Agent 2** (Validation & Docs):
1. ⏸️  Wait for #430 to complete (~30 min)
2. ✅ #431 - Vancouver Validation (30-45 min)
3. ✅ #439 - Documentation (1-2 hours)

**Coordination**: Agent 2 starts #431 and #439 as soon as Agent 1 finishes #430

**Result**: All features complete in ~3-4 hours wall time

---

## Dependency Matrix

| Issue | Depends On | Blocks | Can Parallel With |
|-------|-----------|--------|-------------------|
| #430 | None | All | None (must go first) |
| #431 | #430 | None | #434, #436, #439 |
| #434 | #430 | #436 | #431, #439 |
| #436 | #430, #434 | None | #431, #439 |
| #439 | #430 | None | #431, #434, #436 |

---

## Labels & Organization

All issues are tagged with:
- `exploration-agent` - Epic group
- `enhancement` - Feature addition
- `P0` / `P1` / `P2` - Priority level

Epic #449 includes:
- `epic` - Parent issue marker
- `tracking` - Status tracking

---

## Success Criteria

### Per-Issue
Each issue has clear acceptance criteria in its description.

### Overall Epic (#449)
- [ ] All 5 child issues closed
- [ ] `cmdai "query" --explore` works end-to-end
- [ ] 6/6 Vancouver demos passing (target)
- [ ] User documentation complete
- [ ] PR merged to main
- [ ] Release published

---

## Current Status

### Completed ✅
- Phase 0: Complexity Assessment
- Phase 1: Tool Discovery
- Phase 2: Context Enrichment
- Phase 3: Multi-Command Generation
- E2E Testing (23/23 tests passing)
- Technical Documentation (~1600 lines)

### In Progress 🚧
- None (waiting to start #430)

### Blocked ⏸️
- #430 - Ready to start (no blockers)
- #431 - Blocked by #430
- #434 - Blocked by #430
- #436 - Blocked by #430, #434
- #439 - Blocked by #430

---

## Quick Reference

### Start Here
```bash
# Checkout branch
git checkout feature/agentic-context-loop
git pull origin feature/agentic-context-loop

# Review context
cat WORK_TRACKING.md
cat EXPLORATION_SUMMARY.md

# View epic
gh issue view 449

# Start with critical path
gh issue view 430
```

### Commands to Track Progress
```bash
# View all exploration issues
gh issue list --label exploration-agent

# View epic
gh issue view 449

# View specific issue
gh issue view 430  # (or 431, 434, 436, 439)

# Close issue when complete
gh issue close 430 --comment "CLI integration complete, tested with Vancouver demos"
```

---

## Notes

- **ALL work depends on #430 being completed first**
- Issues #431, #434, #439 can be done in parallel after #430
- Issue #436 should wait for #434 (uses async infrastructure)
- Documentation (#439) benefits from having all features complete but can proceed with just #430
- Estimated total time: 4-6 hours for full rollout
- Core system is production-ready, just needs wiring

---

**Last Updated**: 2025-01-24
**Epic Status**: Waiting for #430 (CLI Integration)
**Next Action**: Start work on #430
