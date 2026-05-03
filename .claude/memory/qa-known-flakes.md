# caro-qa-agent — Known Flakes

**Last updated**: 2026-05-03 by caro-qa-agent (post-verification)
**Purpose**: Distinguish flake from regression. If a surface fails once but passed on a later retry, it's a flake — record it here. If it consistently fails, it's a regression — file a GH issue.

A surface stays here until it has either (a) a fix landed, or (b) been reclassified as a real regression.

---

| Surface | First flake date | Symptom | Workaround | Verified consistent fail? | Linked issue |
|---|---|---|---|---|---|
| `caro::agent: Timeout approaching, skipping refinement` warning during embedded-backend prompt | 2026-04-26 | Agent module times out before refinement step on simple prompts; correct command (`ls`) still produced | None needed — output is correct | No (only seen on first sample after long idle; possibly cold-start) | none yet |

---

## Findings needing follow-up verification (NOT YET FILED)

These are real signals captured during a QA pass but where claim-verification needs a different build/environment before filing a bug. Promote to GH issue once verified, or remove if the next pass shows they were build-flag artifacts.

### ✗ F-2026-05-03-A: REFUTED — `caro doctor` MLX-gating mismatch

**Captured**: 2026-05-03, Slot C surface #9 first try.
**Verified on default-feature rebuild**: 2026-05-03, same day. Generation worked. `doctor`'s "Embedded (ready)" was correct.
**Disposition**: Build-flag artifact — only manifests on `--no-default-features --features embedded-cpu`. Default `cargo install caro` users get MLX. No bug to file.
**Lesson banked**: Add an explicit "Build flags" line to `caro doctor` output so maintainers building without default features see what's compiled in. Logged as a polish observation, NOT filed.

### ✓ F-2026-05-03-B: PROMOTED — Generation-failure exit code is 0

**Captured**: 2026-05-03, Slot C surface #9 first try.
**Verified on default-feature rebuild**: 2026-05-03, same day. Reproduces across 5 distinct error classes (safety block, validation failure on sudo pipe, dangerous-pattern detection, etc.). Always exit 0.
**Filed as**: [#1035](https://github.com/wildcard/caro/issues/1035) — `cli: generation/safety/validation errors all exit 0 (automation-blocking)` (P1).

### ✓ G-2026-05-03-1: PROMOTED — `chmod -R 777 /` bypasses --safety strict

**Captured**: 2026-05-03, Slot C surface #9 verification rebuild — surfaced because the working LLM let me see beyond F-A.
**Verified**: 2026-05-03, 2 stable repros. Root cause at `src/safety/patterns.rs:75` — regex `chmod\s+777\s+/` doesn't match `chmod -R 777 /` because of the `-R` flag in between.
**Filed as**: [#1034](https://github.com/wildcard/caro/issues/1034) — `safety: chmod -R 777 / bypasses --safety strict` (P0). Spawn-task escalation issued for the fixer agent.

---

## Reclassification rules

- 3 flakes within 7 days → reclassify as regression, file GH issue.
- 0 flakes for 14 days → remove from this list.
- Flake on multiple surfaces simultaneously → likely environmental (Ollama down, network, model download); note as such, do not file individual issues.
