# caro-qa-agent — Known Flakes

**Last updated**: 2026-05-03 by caro-qa-agent
**Purpose**: Distinguish flake from regression. If a surface fails once but passed on a later retry, it's a flake — record it here. If it consistently fails, it's a regression — file a GH issue.

A surface stays here until it has either (a) a fix landed, or (b) been reclassified as a real regression.

---

| Surface | First flake date | Symptom | Workaround | Verified consistent fail? | Linked issue |
|---|---|---|---|---|---|
| `caro::agent: Timeout approaching, skipping refinement` warning during embedded-backend prompt | 2026-04-26 | Agent module times out before refinement step on simple prompts; correct command (`ls`) still produced | None needed — output is correct | No (only seen on first sample after long idle; possibly cold-start) | none yet |

---

## Findings needing follow-up verification (NOT YET FILED)

These are real signals captured during a QA pass but where claim-verification needs a different build/environment before filing a bug. Promote to GH issue once verified, or remove if the next pass shows they were build-flag artifacts.

### F-2026-05-03-A: `caro doctor` reports "Embedded (ready)" while MLX-gated runtime path fails

**Captured**: 2026-05-03 during Slot C surface #9 (safety strict mode) test.

**What I saw**: A binary built with `cargo build --release --no-default-features --features embedded-cpu` reports `✓ Embedded (ready)` from `caro doctor` on macOS aarch64. Every actual generation prompt then fails with:

> Error: Command generation failed: Model generation failed: Failed to load model: Configuration error: MLX backend not enabled. Rebuild with --features embedded-mlx

The error message originates from `src/backends/embedded/mlx.rs:200` and `:275` — the dispatcher routes to MLX on Apple Silicon even though the binary only has `embedded-cpu` compiled in.

**Why I'm not filing yet**: Default features include both `embedded-mlx` and `embedded-cpu` (Cargo.toml line 1, `default = ["embedded-mlx", "embedded-cpu", "cve-rules"]`). A regular `cargo install caro` user gets MLX, so the runtime gating mismatch may never reach them. The bug class is real (`doctor` checks model-file presence, not feature-flag/dispatch alignment — see `src/cli/doctor.rs:139` printing "Embedded (ready)" purely off `backend_status.embedded_available`), but priority is unclear without a default-feature rebuild + repro.

**Promotion criteria**: rebuild with default features, repro the prompt; if the same `MLX backend not enabled` error appears on a default build, file P1; if generation works on default build but `doctor` still claims "ready" on `embedded-cpu`-only builds, file P2 (limited to maintainers/CI).

**Reproduction**:
```bash
cargo build --release --no-default-features --features embedded-cpu
./target/release/caro doctor              # claims Embedded (ready)
./target/release/caro --safety strict --dry-run -p "list files"
# Error: ... MLX backend not enabled. Rebuild with --features embedded-mlx
```

### F-2026-05-03-B: Generation-failure exit code is 0, not non-zero

**Captured**: 2026-05-03 during the same safety strict mode test.

**What I saw**: Every one of the 5 dangerous-prompt tests printed `Error: Command generation failed: …` to stderr but exited with `$? = 0`. Automation that pipes `caro` cannot distinguish success from failure on the exit-code channel.

**Why I'm not filing yet**: This may be coupled to F-A — if MLX is missing the binary takes an early-exit path that does not propagate the error code. A default-feature build may exit non-zero on real generation failures. Verification needs default build + a reliably-failing prompt.

**Promotion criteria**: with a default build, induce a real generation failure (e.g. prompt the LLM with no model in cache, or kill ollama mid-stream) and check exit code. If still 0, file P1 (automation-blocking).

---

## Reclassification rules

- 3 flakes within 7 days → reclassify as regression, file GH issue.
- 0 flakes for 14 days → remove from this list.
- Flake on multiple surfaces simultaneously → likely environmental (Ollama down, network, model download); note as such, do not file individual issues.
