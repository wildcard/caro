# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue.

---

## Active flakes

### FLAKE-001: Model download failure / CPU stub fallback in remote sandbox

**First observed**: 2026-05-07  
**Last observed**: 2026-07-21 (behavior changed — see note)  
**Symptom (2026-05-07)**: `caro -p "..." --dry-run` fails with `Backend is not available: Failed to download model after 3 attempts` after 3 retries (2s, 4s backoff).  
**Symptom (2026-07-21)**: `caro -p "..." --dry-run` and `caro ai --once "..."` return `echo 'Please clarify your request'` silently with exit code 0. No explicit download error shown. Model still not downloaded per `caro doctor`. WARN `Timeout approaching, skipping refinement` visible in agent output.  
**Context**: Remote CI/QA sandbox where `https://huggingface.co/` returns HTTP 200 but binary blob downloads time out or are blocked at a lower network layer. Proxy detected at `http://127.0.0.1:34433`.  
**Note on behavior change**: The 2026-07-21 manifestation is caused by two confirmed bugs filed as GitHub issues (#1361, #1362). The CPU backend stub's `prompt.contains("rm")` matches substring "rm" in system-prompt words like "format"; Pattern 43 static matcher regex is over-anchored. These are bugs, not environment flakes — FLAKE-001 is now partially reclassified: the environment limitation (model download blocked) remains a flake, but the degraded output is a code bug.  
**Impact**: Slot A `--dry-run` smoke check returns degraded output in this environment. Use `caro --version`, `--help`, and `doctor` as proxy for binary health; use `cargo test --lib` for functional coverage.  
**Occurrence log**:
- 2026-05-07: explicit download failure message
- 2026-07-21: silent CPU-stub fallback (different manifestation; bugs filed #1361, #1362)

**Promotion threshold**: File regression issue if explicit download failure observed 3 times in 7 days OR if it reproduces on a known-good environment with a pre-downloaded model.  
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
