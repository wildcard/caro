# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue.

---

## Active flakes

### FLAKE-002: `caro ai --once` hangs silently with no model present

**First observed**: 2026-10-10 (documented in PR #1560 body)  
**Symptom**: `caro ai --once '<prompt>'` produces zero output and silently waits when no model is downloaded. Exit code 124 when killed by a 15-second QA timeout. No error message, no stderr, no progress indicator.  
**Context**: Remote sandbox with no pre-downloaded model. `caro doctor` shows "No ready backend available" but the `ai` subcommand does not check backend availability before blocking on `generate_command`.  
**Root cause**: `src/main.rs:1152-1161` — `run_ai_once` calls `backend.generate_command().await` without checking model-download readiness. `src/ai/runner.rs:117` — generation blocks waiting for the embedded backend to download a missing model (blocked by sandbox networking). Note: `embedded_backend.is_available()` always returns `true`, so a pre-flight check on that method would not prevent the hang; the fix requires checking model-file readiness or adding a download timeout.  
**Stall watchdog caveat** (per Codex review): `src/model_loader.rs:337-338` defines `DOWNLOAD_STALL_TIMEOUT = 60s`. The QA 15-second timeout is shorter than this watchdog, so the observation does not prove an indefinite hang — the process may terminate naturally after its three 60-second attempts and retry backoffs with an error (the #1561 fix). Verify with a ≥200-second timeout in a future pass; if exit 1 with an error message is observed, reclassify as "slow/silent download" and close this flake as resolved.  
**Impact**: Slot C testing of `ai` surfaces is blocked in sandbox. Use `--dry-run` for prompt-generation coverage; use unit tests for `ai` session logic.  
**Occurrence log**:
- 2026-10-10: observed (per PR #1560)
- 2026-10-11: observed (confirmed this run, exit 124 after 15s timeout)

**Promotion threshold**: Not applicable — known root cause, tracked in open issues #1272 and #1408. File a separate regression issue only if behavior changes (e.g. starts producing an error message, or starts hanging on a different backend).  
**Workaround**: Test `caro ai --once` only in an environment with a pre-downloaded model or Ollama available.

---

## Resolved flakes

### FLAKE-001: Model download failure in remote sandbox — `--dry-run`

**First observed**: 2026-05-07  
**Resolved**: 2026-10-11  
**Resolution**: `caro --dry-run` now uses the static-matcher fallback when no model is present, returning a deterministic result without attempting a download. Confirmed PASS on 2026-10-11 (`ls -la` returned in ~100ms).  
**Commit**: Not identified; `#1517` changes Python-file search behavior, not the static-matcher fallback.

**Symptom was**: `caro -p "..." --dry-run` fails with `Backend is not available: Failed to download model after 3 attempts` after 3 retries (2s, 4s backoff).  
**Occurrence log**:
- 2026-05-07: observed once

---

## Classification guide

| Observations in 7 days | Action |
|------------------------|--------|
| 1-2 | Log here as flake, note in session-log Followups |
| 3+ | Reclassify as regression, file GitHub issue with `bug` + `qa` labels, link from this file |
