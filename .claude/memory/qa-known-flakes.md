# caro-qa-agent — Known Flakes

**Last updated**: 2026-04-26 by caro-qa-agent
**Purpose**: Distinguish flake from regression. If a surface fails once but passed on a later retry, it's a flake — record it here. If it consistently fails, it's a regression — file a GH issue.

A surface stays here until it has either (a) a fix landed, or (b) been reclassified as a real regression.

---

| Surface | First flake date | Symptom | Workaround | Verified consistent fail? | Linked issue |
|---|---|---|---|---|---|
| `caro::agent: Timeout approaching, skipping refinement` warning during embedded-backend prompt | 2026-04-26 | Agent module times out before refinement step on simple prompts; correct command (`ls`) still produced | None needed — output is correct | No (only seen on first sample after long idle; possibly cold-start) | none yet |

---

## Reclassification rules

- 3 flakes within 7 days → reclassify as regression, file GH issue.
- 0 flakes for 14 days → remove from this list.
- Flake on multiple surfaces simultaneously → likely environmental (Ollama down, network, model download); note as such, do not file individual issues.
