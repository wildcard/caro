# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue.

---

## Active flakes

### FLAKE-001: Model download failure in remote sandbox

**First observed**: 2026-05-07  
**Symptom**: Two manifestations observed: (a) `caro -p "..." --dry-run` fails with `Backend is not available: Failed to download model after 3 attempts` after 3 retries (2s, 4s backoff) — bootstrap/early occurrences; (b) silent hang for 15s then timeout with no error message — occurrences 4–5 (2026-09-26, 2026-09-27). Both trace to the same root cause: HuggingFace binary blob download blocked at network layer.  
**Context**: Remote CI/QA sandbox where `https://huggingface.co/` returns HTTP 200 but binary blob downloads time out or are blocked at a lower network layer.  
**Impact**: Slot A `--dry-run` smoke check cannot be completed in this environment. Use `caro --version`, `--help`, and `doctor` as proxy for binary health; use `cargo test --lib` for functional coverage.  
**Occurrence log**:
- 2026-05-07: 1st occurrence (bootstrap)
- 2026-07-25: 2nd occurrence (PR #1373, unmerged)
- 2026-09-07: 3rd occurrence (PR #1443, unmerged)
- 2026-09-26: 4th occurrence (PR #1477, unmerged)
- 2026-09-27: 5th occurrence (this PR)

**Promotion threshold**: File regression issue if observed 3 times in 7 days OR if it reproduces on a known-good environment with a pre-downloaded model.  
**Workaround**: Run `caro -p "..." --dry-run` from an environment with `~/.cache/caro/models/` pre-populated, or with Ollama installed as fallback backend.

---

## Resolved flakes

_(none yet)_

---

## Classification guide

| Observations in 7 days | Action |
|------------------------|--------|
| 1-2 | Log here as flake, note in session-log Followups |
| 3+ | Reclassify as regression, file GitHub issue with `bug` + `qa` labels, link from this file |
