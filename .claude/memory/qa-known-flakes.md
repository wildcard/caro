# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue.

---

## Active flakes

### FLAKE-001: Model download failure in remote sandbox

**First observed**: 2026-05-07  
**Symptom**: `caro -p "..." --dry-run` fails with `Backend is not available: Failed to download model after 3 attempts` after 3 retries (2s, 4s backoff).  
**Context**: Remote CI/QA sandbox where `https://huggingface.co/` returns HTTP 200 but binary blob downloads time out or are blocked at a lower network layer.  
**Impact**: Queries requiring LLM inference cannot complete in this environment. Slot A dry-run using simple queries (e.g. "list files in current directory") uses the static matcher and passes without model download. Only prompts that fall outside static-matcher coverage are blocked; do not substitute proxy checks for a dry-run that the static matcher can serve.  
**Occurrence log**:
- 2026-05-07: observed once (Slot A dry-run)
- 2026-10-07: observed again (Slot C `caro ai --once`; hang >45s; same root cause — binary download stalls)

**Promotion threshold**: File regression issue if observed 3 times in 7 days OR if it reproduces on a known-good environment with a pre-downloaded model.  
**Workaround**: Run `caro -p "..." --dry-run` from an environment with `~/.cache/caro/models/` pre-populated, or with Ollama installed as fallback backend.
**Note**: The `caro ai --once` variant of this flake is separately tracked as open issue #1440 (no download timeout) + #1179 (no static fallback). In the main `caro -p` path, the static matcher handles common commands without LLM, so FLAKE-001 surfaces only for queries that need LLM.

---

## Resolved flakes

_(none yet)_

---

## Classification guide

| Observations in 7 days | Action |
|------------------------|--------|
| 1-2 | Log here as flake, note in session-log Followups |
| 3+ | Reclassify as regression, file GitHub issue with `bug` + `qa` labels, link from this file |
