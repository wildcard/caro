# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue.

---

## Active flakes

_(none)_

---

## Resolved flakes

### FLAKE-001: Model download failure in remote sandbox

**First observed**: 2026-05-07
**Resolved**: 2026-07-24 (78 days elapsed; single observation; FLAKE-001 download-failure error not reproduced)
**Symptom**: `caro -p "..." --dry-run` fails with `Backend is not available: Failed to download model after 3 attempts`.
**Context**: Remote QA sandbox where huggingface.co HTTP 200 is reachable but binary blob downloads time out or are blocked at a lower network layer.
**Resolution**: Observed only once on 2026-05-07 (bootstrap run). The 2026-07-24 run did NOT reproduce the specific download-failure error (`Failed to download model after 3 attempts`). The dry-run smoke check failed for a different reason (known P1 #1277: CPU placeholder logic bug), which is distinct from a download error. Classified as one-time environmental artifact; FLAKE-001's download-failure symptom is not confirmed resolved — it was simply not reproduced.
**Occurrence log**:
- 2026-05-07: observed once (bootstrap)
- 2026-07-24: download-failure error not reproduced (dry-run failed for different reason: P1 #1277)

---

## Classification guide

| Observations in 7 days | Action |
|------------------------|--------|
| 1-2 | Log here as flake, note in session-log Followups |
| 3+ | Reclassify as regression, file GitHub issue with `bug` + `qa` labels, link from this file |
