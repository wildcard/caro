# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue.

---

## Active flakes

### FLAKE-001: Model download failure / hang in remote sandbox

**First observed**: 2026-05-07  
**Last observed**: 2026-09-06  
**Symptom (v1.3.0, 2026-05-07)**: `caro -p "..." --dry-run` fails with `Backend is not available: Failed to download model after 3 attempts` after 3 retries (2s, 4s backoff).  
**Symptom (v1.5.0, 2026-09-06)**: `caro -p "..." --dry-run` and `caro ai --once` hang **indefinitely** with ZERO output. No error, no retry, no progress bar (indicatif suppressed in non-TTY env). Process enters sleeping state (6 threads) blocked on model binary download.  
**Behavior change**: The v1.3.0 retry-with-backoff path is gone in v1.5.0. The root cause is `HfHubClient` (`src/cache/http_client.rs:42`) building a reqwest client with no `.timeout()` or `.connect_timeout()`. Filed as [#1440](https://github.com/wildcard/caro/issues/1440) (P1 regression).  
**Context**: Remote CI/QA sandbox where `https://huggingface.co/` returns HTTP 200 but binary blob downloads stall at the proxy layer.  
**Impact**: Slot A `--dry-run` smoke check cannot be completed in this environment. Use `caro --version`, `--help`, and `doctor` as proxy for binary health; use `cargo test --lib` for functional coverage.  
**Occurrence log**:
- 2026-05-07: v1.3.0 — retry-with-backoff, error reported after 3 attempts
- 2026-09-06: v1.5.0 — silent hang, no retries, no error (regression filed as #1440)

**Status**: Promoted to regression (behavior change constitutes a regression even under 3-in-7-days threshold). Filed as #1440.  
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
| Behavior change between versions | Reclassify as regression regardless of observation count |
