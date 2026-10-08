# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue.

---

## Active flakes

### FLAKE-001: Model download failure in remote sandbox

**First observed**: 2026-05-07  
**Symptom**: `caro ai --once <query>` hangs indefinitely with zero stdout/stderr output when no backend model is downloaded; exit only via SIGTERM. (`--dry-run` is no longer affected — static matcher fallback added in v1.5.0.)  
**Context**: Remote CI/QA sandbox where `https://huggingface.co/` returns HTTP 200 but binary blob downloads time out or are blocked at a lower network layer.  
**Impact**: `caro ai --once` cannot complete in this environment — backend init blocks on model download with no timeout and no fallback. Use `caro --version`, `--help`, `doctor`, and `--dry-run` as proxy for binary health; use `cargo test --lib` for functional coverage.  
**Occurrence log**:
- 2026-05-07: observed once on `caro -p ... --dry-run` (pre-v1.5.0; static matcher fallback not yet present)
- 2026-10-08: observed again on `caro ai --once "list files"` — hangs silently when no backend is ready (filed as [#1540](https://github.com/wildcard/caro/issues/1540); `--dry-run` no longer flakes in v1.5.0 — static matcher fallback added; `ai --once` has no static-matcher fallback so model-download path is still affected)

**Promotion threshold**: File regression issue if observed 3 times in 7 days OR if it reproduces on a known-good environment with a pre-downloaded model.  
**Workaround**: Run `caro ai --once <query>` from an environment with `~/.cache/caro/models/` pre-populated, or with Ollama installed as fallback backend.

---

## Resolved flakes

_(none yet)_

---

## Classification guide

| Observations in 7 days | Action |
|------------------------|--------|
| 1-2 | Log here as flake, note in session-log Followups |
| 3+ | Reclassify as regression, file GitHub issue with `bug` + `qa` labels, link from this file |
