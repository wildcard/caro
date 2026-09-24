# caro-research--scoping-process — Run Report

**Date**: 2026-07-03
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-030 produced

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02, 06-05, 06-08) — ADR-015…019 already scoped.
2. Noted that since the last run, other sessions landed ADR-020…029
   (approval protocol, attribution, safety library/CI, and the six headless
   ADRs 024–028 + 029). None covers #662.
3. Audited ROADMAP.md v2.0.0 for remaining un-scoped items tagged
   *"extends caro-core, exempt"*: **Handy.Computer Integration (#662)** was
   the only one with no ADR (security hardening #6 is partially covered by
   ADR-010/012; Exo #162 is already implemented in `src/backends/remote/exo.rs`).
4. Researched Handy live (README fetched 2026-07-03) and verified its
   companion crate `transcribe-rs` 0.3.11 on crates.io.

## New ADR Produced

**ADR-030** — `docs/adr/ADR-030-handy-voice-input.md`

- `caro --voice` + `caro voice transcribe` — offline STT feeding the existing
  safety-validated pipeline; pure subprocess, no daemon
- Parakeet V3 int8 via `transcribe-rs` (`onnx` + `vad-silero` features only;
  Whisper path excluded — it is Handy's open crash class)
- New `voice-input` feature flag, off by default; **Phase 0 build spike
  required first** per `.claude/rules/external-sdk-integration.md`
- Versioned JSON contract (`TranscriptionResult`, schema_version 1) + new
  exit codes 30/31/32 to be registered against ADR-024's registry
- 6 files changed (2 new); model fetched through existing `src/cache`
- Deterministic tests via `--audio-file` fixtures + `MockVoiceEngine`

## Key Research Findings

1. **Handy has no structured output contract at all.** Its CLI flags and
   SIGUSR1/2 only remote-control a running GUI instance; the transcript is
   never returned to the caller — it is pasted into the focused window.
   This is the gap caro's contract-first design exploits.
2. **Handy's warm-model strategy is daemon residency**, which violates the
   task's "pure subprocess" constraint. ADR-030 answers with a ≤2 s
   cold-start budget on the Parakeet int8 CPU path (mmap'd ONNX, cached
   model bytes).
3. **`transcribe-rs` 0.3.11 (MIT) declares no `rust-version`** on any
   published release — the build spike must verify MSRV 1.83 explicitly and
   `cargo deny check licenses` must sweep `ort`'s transitive tree.
4. Handy's Whisper backend crashes on certain Windows/Linux configs (their
   own README, "help wanted") — reason to ship Parakeet-only in v1.
5. The root-level `speech-to-text-integration-plan.md` (cmdai-era,
   whisper-rs-first) is superseded in part by ADR-030's Parakeet-first path.

## Housekeeping Notes (not fixed this run)

- `docs/adr/README.md` index is stale: its table ends at ADR-015 even though
  ADR-016…029 exist. Worth an alignment PR; not touched here to avoid
  colliding with parallel sessions.
- Files were **written to the working tree only, not committed** (git
  workflow forbids commits to main; repo also has a broken worktree ref
  `006-replace-ascii-morph` and an unremovable `index.lock` visible from
  the sandbox). Suggested branch: `feat/adr-030-voice-input-scope`.
- P0 from 2026-06-02 remains open: fill the `[FEATURE NAME]` placeholder in
  the scheduled task, or accept autonomous target selection as the norm.

## Suggested Next Targets

- Security hardening (#6) gap analysis vs ADR-010/012 (what's left un-scoped)
- Yappus-Term follow-up (#153/#185) — `docs/research/YAPPUS_TERM_GAP_ANALYSIS.md` exists but has no ADR
- Freshness pass on ADR-024…029 once implementation PRs start landing
