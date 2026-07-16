# QA Known Flakes

Document flaky behaviours observed during QA runs. A flake observed 3+ times in 7 days should be reclassified as a regression and filed as a GitHub issue.

---

## Active flakes

### FLAKE-001: Model download failure in remote sandbox

**First observed**: 2026-05-07  
**Symptom**: `caro -p "..." --dry-run` hangs or fails after 3 retries with `Backend is not available: Failed to download model after 3 attempts`. In the 2026-07-16 observation, the command hung for 30+ seconds (terminated by timeout) after displaying the telemetry consent prompt and setting telemetry to disabled. The binary-blob download is blocked at a lower network layer despite `huggingface.co` returning HTTP 200.  
**Context**: Remote CI/QA sandbox where `https://huggingface.co/` returns HTTP 200 but binary blob downloads time out or are blocked at a lower network layer.  
**Impact**: Slot A `--dry-run` smoke check cannot be completed in this environment. Use `caro --version`, `--help`, and `doctor` as proxy for binary health; use `cargo test --lib` for functional coverage.  
**Occurrence log**:
- 2026-05-07: observed once (model download failed after 3 retries with explicit error)
- 2026-07-16: observed once (command hung 30s, terminated by timeout; same root cause)

**Promotion threshold**: File regression issue if observed 3 times in 7 days OR if it reproduces on a known-good environment with a pre-downloaded model.  
**Workaround**: Run `caro -p "..." --dry-run` from an environment with `~/.cache/caro/models/` pre-populated, or with Ollama installed as fallback backend. Alternatively use `caro --dry-run ai --once` via stdin which routes through the static backend without model download.

---

## Resolved flakes

_(none yet)_

---

## Classification guide

| Observations in 7 days | Action |
|------------------------|--------|
| 1-2 | Log here as flake, note in session-log Followups |
| 3+ | Reclassify as regression, file GitHub issue with `bug` + `qa` labels, link from this file |
