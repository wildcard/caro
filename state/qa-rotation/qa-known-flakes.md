# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue. A behavior change between versions is also grounds for immediate reclassification regardless of observation count.

---

## Active flakes

### FLAKE-001: Model download failure / hang in remote sandbox

**First observed**: 2026-05-07
**Last observed**: 2026-09-06  
**Symptom (2026-05-07)**: `caro -p "..." --dry-run` fails with `Backend is not available: Failed to download model after 3 attempts` after 3 retries (2s, 4s backoff).  
**Symptom (2026-07-21)**: `caro -p "..." --dry-run` and `caro ai --once "..."` return `echo 'Please clarify your request'` silently with exit code 0. No explicit download error shown. Model still not downloaded per `caro doctor`. WARN `Timeout approaching, skipping refinement` visible in agent output.  
**Symptom (v1.3.0, 2026-05-07)**: `caro -p "..." --dry-run` fails with `Backend is not available: Failed to download model after 3 attempts` after 3 retries (2s, 4s backoff).  
**Symptom (v1.5.0, 2026-09-06)**: `caro -p "..." --dry-run` and `caro ai --once` hang **indefinitely** with ZERO output. No error, no retry, no progress bar (indicatif suppressed in non-TTY env). Process enters sleeping state (6 threads) blocked on model binary download.  
**Behavior change**: The v1.3.0 retry-with-backoff path is gone in v1.5.0. The root cause is `ModelLoader::download_model_attempt` (`src/model_loader.rs:214-240`) calling `hf_hub::api::tokio::Api::new()` with no timeout configured. The `hf_hub` API constructs its own reqwest client internally; when the blob download stalls at the proxy layer, `repo.get().await` never returns and the retry loop in `download_model_with_retry` (lines 167-211) can never advance. Filed as [#1440](https://github.com/wildcard/caro/issues/1440) (P1 regression).  
**Symptom**: `caro ai --once <query>` hangs indefinitely with zero stdout/stderr output when no backend model is downloaded; exit only via SIGTERM. (`--dry-run` is no longer affected — static matcher fallback added in v1.5.0.)  
**Context**: Remote CI/QA sandbox where `https://huggingface.co/` returns HTTP 200 but binary blob downloads time out or are blocked at a lower network layer. Additionally, `www.caro.sh` (redirect target from apex `caro.sh`) is unreachable from sandbox.  
**Note on behavior change**: The 2026-07-21 manifestation is caused by two confirmed bugs filed as GitHub issues (#1361, #1362). The CPU backend stub's `prompt.contains("rm")` matches substring "rm" in system-prompt words like "format"; Pattern 43 static matcher regex is over-anchored. These are bugs, not environment flakes — FLAKE-001 is now partially reclassified: the environment limitation (model download blocked) remains a flake, but the degraded output is a code bug.  
**Impact**: `caro ai --once` cannot complete in this environment — backend init blocks on model download with no timeout and no fallback. Use `caro --version`, `--help`, `doctor`, and `--dry-run` as proxy for binary health; use `cargo test --lib` for functional coverage.  
**Occurrence log**:
- 2026-05-07: observed once on `caro -p ... --dry-run` (pre-v1.5.0; static matcher fallback not yet present)
- 2026-05-23: observed again (16-day gap; separate 7-day windows, threshold not triggered)
- 2026-05-24: observed twice (Slot A dry-run + Slot C `caro ai --once`)
- 2026-05-25: observed twice more (Slot A dry-run + Slot C caro ai --once). Total: 3 observations but spread across 2 dates; check 7-day window on next run.
- 2026-05-26: observed again (2nd logged occurrence; also triggered in all intermediate QA runs 2026-05-15 through 2026-05-25 that did not update memory files — pattern is consistent across all runs)
- 2026-05-27: observed again (2nd occurrence; 20 days after first — NOT within 7-day promotion window)
- 2026-06-13: observed again (2nd total; new 7-day window — first was 2026-05-07, >30 days prior)
- 2026-07-11: observed again (fresh sandbox, same symptom — hang after telemetry banner)
- 2026-07-12: observed (second proximity) — model download slow ~60s+ in sandbox; 30s timeout killed process; 90s timeout succeeded but CPU stub bug (#1269) produced wrong output
- 2026-07-16: observed once (command hung 30s, terminated by timeout; same root cause)
- 2026-07-21: silent CPU-stub fallback (different manifestation; bugs filed #1361, #1362)
- 2026-07-22: model download succeeded ([size omitted]) but took >60s; agent loop timed out before completion; first invocation returned wrong output; model was available on second invocation but CPU stub bug (#1277) dominated
- 2026-07-24: download-failure error not reproduced (dry-run failed for different reason: P1 #1277)
- 2026-07-25: 2nd occurrence (PR #1373, unmerged)
- 2026-08-01: reproduced (second observation; 85-day gap, not within 7-day promotion window)
- 2026-09-06: v1.5.0 — silent hang, no retries, no error (regression filed as #1440)
- 2026-09-07: 3rd occurrence (PR #1443, unmerged)
- 2026-09-08: observed again on `caro ai --once 'list files'` (exit 143 after 25s timeout); also on `caro -p 'list files' --dry-run` (blocked at telemetry consent in fresh sandbox — different surface, same environment limitation)
- 2026-09-12: observed again (2nd total; 128 days since first; NOT within 7-day window; no reclassification)
- 2026-09-24: NOT observed — model downloaded successfully ([size omitted] GGUF); prior observation is 140 days old, outside 7-day window; streak reset
- 2026-09-26: 4th occurrence (PR #1477, unmerged)
- 2026-09-27: 5th occurrence (this PR)
- 2026-09-28: observed again (2nd occurrence; dates far apart, does NOT trigger 3×/7-day reclassification)
- 2026-09-29: observed (occurrence #2); `caro -p 'list files' --dry-run` timed out after 30s with exit code 124; same sandbox environment
- 2026-09-30: observed again (Slot A dry-run + Slot C `caro ai --once`); EXIT:124 (30s timeout, no error surfaced) — note: signature differs from documented symptom (silent hang vs. error after 3 retries); session log also maps this to hang issues #1404/#1393/#1455/#1468. Proxy at 127.0.0.1:41305 detected; HuggingFace HTTP 200 reachable but binary download still blocked.
- 2026-10-01: not exercised (inconclusive) — Slot A dry-run used the static matcher and skipped model download, so the download-failure path from 2026-05-07 was not retested this pass
- 2026-10-02: NOT reproduced — model downloaded successfully (qwen2.5-coder-1.5b-instruct-q4_k_m.gguf) during background test invocation; dry-run smoke used static matcher and did not trigger a download
- 2026-10-03: `caro ai --once` hung ~100s waiting for model download before falling back to CPU stub (distinct from previous — `--dry-run` did NOT trigger FLAKE-001 this pass; static matcher resolved it)
- 2026-10-04: did NOT reproduce — model qwen2.5-coder-1.5b-instruct-q4_k_m.gguf was downloaded successfully to /root/.cache/caro/models/. Sandbox network policy may have changed, or the model was pre-seeded.
- 2026-10-05: observed (`caro ai --once list files` — process terminated after external timeout with no output; confirmed NOT stdin blocking per #1503 fix — model download timeout confirmed in fresh sandbox)
- 2026-10-06: **not triggered** — static matcher handled `ls` query; FLAKE-001 not observed this pass
- 2026-10-07: observed again (Slot C `caro ai --once`; hang >45s; code defect — no download timeout (#1440), no static fallback (#1179), no init feedback (#1408), distinct from the sandbox download stall)
- 2026-10-08: observed again on `caro ai --once "list files"` — hangs silently when no backend is ready (filed as [#1540](https://github.com/wildcard/caro/issues/1540); `--dry-run` no longer flakes in v1.5.0 — static matcher fallback added; `ai --once` has no static-matcher fallback so model-download path is still affected)
- 2026-10-09: NOT reproduced — model downloaded successfully to `~/.cache/caro/models/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf` during session; FLAKE-001 not active this run

**Promotion threshold**: File regression issue if explicit download failure observed 3 times in 7 days OR if it reproduces on a known-good environment with a pre-downloaded model.  
**Status**: Promoted to regression (behavior change constitutes a regression even under 3-in-7-days threshold). Filed as #1440.  
**Fix (2026-10-11)**: [#1561](https://github.com/wildcard/caro/pull/1561) merged and closed #1440. A stalled model download now fails instead of hanging, and the 3-attempt retry ends with the troubleshooting error. #1179 (no static fallback) and #1408 (no init feedback) are separate and stay open. Next run: re-test `caro ai --once` before marking this flake resolved.  
**Workaround**: Run `caro ai --once <query>` from an environment with `~/.cache/caro/models/` pre-populated, or with Ollama installed as fallback backend.
**Note**: The `caro ai --once` variant of this flake is separately tracked as open issue #1440 (no download timeout) + #1179 (no static fallback) + #1408 (no init feedback). In the main `caro -p` path, the static matcher handles common commands without LLM, so FLAKE-001 surfaces only for queries that need LLM.

---

### FLAKE-002: `caro ai --once` silent hang when no model available
**First observed**: 2026-10-10  
**Duplicate of**: FLAKE-001's `caro ai --once` variant (observed 2026-10-05, 10-07 and 10-08) and #1540 (closed as a duplicate). The 2026-10-10 run could not see those entries. Open causes: #1408 (no init feedback) and #1179 (no static fallback). #1561 fixed #1440 (stalled download) on 2026-10-11.  
**Symptom**: `caro ai --once '<prompt>'` hangs indefinitely with zero output — no progress indicator, no error message, no timeout. External kill required.  
**Context**: Remote QA sandbox with no pre-downloaded model and no configured remote backend. `caro ai` calls `backend_arc()` directly (bypassing the `AgentLoop` and its static matcher), so the embedded backend attempts `generate_command` which blocks on model download.  
**Impact**: `caro ai` is untestable in sandboxes without a pre-downloaded model or remote backend. Distinct from FLAKE-001: FLAKE-001's `--dry-run` path eventually surfaces an error after 3 retries; this path produces no output at all.  
**Occurrence log**:
- 2026-10-10: `caro ai --once 'show me the current date'` — hung silently until 20s external timeout; stdin `/dev/null` made no difference (Slot C surface #10).
**Promotion threshold**: File regression issue if observed 3 times in 7 days OR if it reproduces with a remote backend configured.  
**Workaround**: Run `caro ai` from an environment with `~/.cache/caro/models/` pre-populated, or with Ollama configured as a backend.
## Resolved flakes

_(none yet)_

---

## Classification guide

| Observations in 7 days | Action |
|------------------------|--------|
| 1-2 | Log here as flake, note in session-log Followups |
| 3+ | Reclassify as regression, file GitHub issue with `bug` + `qa` labels, link from this file |
**Note**: A behavior change between versions (same symptom that previously errored now silently hangs, or vice versa) is grounds for immediate reclassification as a regression regardless of observation count — behavior changes are not flukes.
